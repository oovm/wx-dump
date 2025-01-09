#![allow(non_snake_case, missing_docs)]
use crate::WxResult;
use futures_util::stream::BoxStream;
use lz4_flex::decompress;
use quick_xml::de::from_str;
use serde::Deserialize;
use sqlx::{FromRow, Pool, Sqlite};
use std::{fmt::Debug, path::Path, str::FromStr};
use xmltree::{Element, ParseError};

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

/// 撤回的消息
///
/// `<revokemsg>"某人" 撤回了一条消息</revokemsg>`
#[derive(Debug, Deserialize)]
pub struct RevokeMessage {
    #[serde(rename = "$value")]
    field: String,
}
pub struct VoIPBubbleMessage {
    root: Element,
}
impl VoIPBubbleMessage {
    pub fn get_message(&self) -> Option<&str> {
        // <voipmsg/>
        let x = self.root.children.get(0)?;
        // <VoIPBubbleMsg/>
        let x = x.as_element()?;
        // <msg/>
        let a = x.get_child("msg")?;
        Some(a.children.get(0)?.as_cdata()?)
    }
}

impl FromStr for VoIPBubbleMessage {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self { root: Element::parse(s.as_bytes())? })
    }
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
    pub fn text(&self) -> &str {
        &self.StrContent
    }
    /// 将 `CompressContent` 字段转为 `ReferenceText` 格式
    pub fn text_reference(&self) -> WxResult<String> {
        let xml = self.binary_as_string()?;
        Ok(xml)
    }
    /// 撤回消息
    pub fn revoke_message(&self) -> WxResult<String> {
        let xml = self.binary_as_string()?;
        let message: RevokeMessage = from_str(&self.StrContent)?;
        Ok(message.field)
    }
    /// 语音通话
    pub fn voip_bubble_message(&self) -> WxResult<VoIPBubbleMessage> {
        Ok(VoIPBubbleMessage::from_str(&self.StrContent)?)
    }
    pub fn unix_time(&self) -> i64 {
        // UTC+8
        self.CreateTime + 8 * 60 * 60
    }
    pub fn is_sender(&self) -> bool {
        self.IsSender == 1
    }
}
