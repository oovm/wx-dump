#![allow(non_snake_case, missing_docs)]
use crate::{WxError, WxResult, helpers::LazyXML, orm_types::extensions::SqliteHelper};
use futures_util::stream::BoxStream;
use lz4_flex::decompress;
use prost::Message;
use rusqlite::Row;
use sqlx::{FromRow, Sqlite};
use std::{fmt::Debug, ops::Coroutine, path::Path, str::FromStr};
use wx_proto::proto::MsgBytesExtra;

pub mod message_type;

pub mod extensions;

#[doc = include_str!("MSG.md")]
#[derive(Debug, FromRow)]
pub struct MessageData {
    Sequence: i64,
    Type: i32,
    SubType: i32,
    CreateTime: i64,
    StrContent: String,
    CompressContent: Vec<u8>,
    BytesExtra: MsgBytesExtra,
    SenderId: String,
    IsSender: i32,
    StrTalker: String,
    strNickName: String,
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
    pub fn query<'a>(sql: &'a mut SqliteHelper, path: &Path) -> BoxStream<'a, Result<MessageData, sqlx::Error>> {
        let micro_msg = path.join("MicroMsg.db");
        sqlx::query_as::<Sqlite, MessageData>(include_str!("msg_query.sql"))
            .bind(micro_msg.to_string_lossy().to_string())
            .fetch(&mut sql.db)
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
    pub fn extra_info(&self) -> WxResult<String> {
        Ok(format!("{:?}", self.BytesExtra))
    }
    /// 将 `CompressContent` 字段转为 `ReferenceText` 格式
    pub fn text_reference(&self) -> WxResult<String> {
        let xml = self.binary_as_string()?;
        Ok(xml)
    }
    pub fn sender_id(&self) -> &str {
        &self.SenderId
    }
    pub fn image_path(&self) -> String {
        "self.BytesExtra.pop_image_path()".to_string()
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
