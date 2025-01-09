#![allow(non_snake_case, missing_docs)]
use crate::{ WxResult, helpers::LazyXML};
use futures_util::stream::BoxStream;
use lz4_flex::decompress;
use sqlx::{FromRow, Pool, Sqlite};
use std::{fmt::Debug, path::Path, str::FromStr};

pub mod message_type;

#[doc = include_str!("MSG.md")]
#[derive(Debug, FromRow)]
pub struct MessageData {
    Sequence: i64,
    Type: i32,
    SubType: i32,
    CreateTime: i64,
    StrContent: String,
    CompressContent: Vec<u8>,
    BytesExtra: Vec<u8>,
    IsSender: i32,
    StrTalker: String,
    strNickName: String,
}

impl MessageData {
    pub fn image_message(&self) -> WxResult<String> {
        Ok(String::from_utf8_lossy(&self.BytesExtra).to_string())
    }
}

/// 撤回的消息
///
/// `<revokemsg>"某人" 撤回了一条消息</revokemsg>`
pub struct RevokeMessage {
    xml: LazyXML,
}
pub struct VoIPBubbleMessage {
    xml: LazyXML,
}
pub struct VoiceMessage {
    xml: LazyXML,
}
impl MessageData {
    pub fn query<'a>(db: &'a Pool<Sqlite>, path: &Path) -> BoxStream<'a, sqlx::Result<MessageData>> {
        let micro_msg = path.join("MicroMsg.db");
        sqlx::query_as::<Sqlite, MessageData>(include_str!("get_msg.sql"))
            .bind(micro_msg.to_string_lossy().to_string())
            .fetch(db)
    }
    pub fn binary_as_string(&self) -> WxResult<String> {
        let mut decompress = decompress(&self.CompressContent, 0x10004)?;
        // 移除字符串末尾的 `<NUL>`
        let tail = decompress.pop();
        debug_assert_eq!(tail, Some(0x00));
        Ok(String::from_utf8(decompress)?)
    }
}

impl MessageData {
    pub fn message_id(&self) -> i64 {
        self.Sequence
    }
    pub fn room_id(&self) -> &str {
        &self.StrTalker
    }
    pub fn room_name(&self) -> &str {
        &self.strNickName
    }
    pub fn text_message(&self) -> &str {
        &self.StrContent
    }
    /// 将 `CompressContent` 字段转为 `ReferenceText` 格式
    pub fn text_reference(&self) -> WxResult<String> {
        let xml = self.binary_as_string()?;
        Ok(xml)
    }
    /// 撤回消息
    pub fn revoke_message(&self) -> WxResult<String> {
        let xml = RevokeMessage { xml: LazyXML::from_str(&self.StrContent)? };
        let value = xml.xml.get_xpath("/revokemsg")?;
        Ok(value.into_string())
    }
    /// 语音通话
    pub fn voip_message(&self) -> WxResult<String> {
        let xml = VoIPBubbleMessage { xml: LazyXML::from_str(&self.StrContent)? };
        let value = xml.xml.get_xpath("/voipmsg/VoIPBubbleMsg/msg/text()")?;
        Ok(value.into_string())
    }
    /// 语音留言
    pub fn voice_message(&self) -> WxResult<String> {
        let xml = VoiceMessage { xml: LazyXML::from_str(&self.StrContent)? };
        let value = xml.xml.get_xpath("//voicetrans/@transtext");
        let text = value.map(|x| x.into_string()).unwrap_or_default();
        match text.as_str() {
            "" => Ok("<语音未接收>".to_string()),
            _ => Ok(text),
        }
    }
    pub fn unix_time(&self) -> i64 {
        // UTC+8
        self.CreateTime + 8 * 60 * 60
    }
    pub fn is_sender(&self) -> bool {
        self.IsSender == 1
    }
}
