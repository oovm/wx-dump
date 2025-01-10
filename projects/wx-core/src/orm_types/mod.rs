#![allow(non_snake_case, missing_docs)]
use crate::{
    MessageType, WxError, WxResult, helpers::LazyXML, orm_types::extensions::SqliteHelper,
    wx_decrypt::decrypt_dat::XorDecryptor,
};
use futures_util::stream::BoxStream;
use lz4_flex::decompress;
use rust_xlsxwriter::Url;
use sqlx::{FromRow, Sqlite};
use std::{fmt::Debug, fs::create_dir_all, path::Path, str::FromStr};
use tokio::fs::create_dir;
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
    #[sqlx(rename = "StrTalker")]
    RoomId: String,
    RoomName: String,
    SenderId: String,
    SenderName: String,
    IsSender: i32,
    StrContent: String,
    CompressContent: Vec<u8>,
    BytesExtra: MsgBytesExtra,
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
}

impl MessageData {
    pub fn message_id(&self) -> i64 {
        self.Sequence
    }
    pub fn room_id(&self) -> &str {
        &self.RoomId
    }
    pub fn room_name(&self) -> &str {
        &self.RoomName
    }
    pub fn text_message(&self) -> &str {
        &self.StrContent
    }
    pub fn compress_content(&self) -> WxResult<String> {
        if self.CompressContent.is_empty() {
            return Ok(String::new());
        }
        let mut decompress = decompress(&self.CompressContent, 0x10004)?;
        // 移除字符串末尾的 `<NUL>`
        let tail = decompress.pop();
        debug_assert_eq!(tail, Some(0x00));
        Ok(String::from_utf8(decompress)?)
    }
    pub fn extra_content(&self) -> WxResult<String> {
        Ok(format!("{:?}", self.BytesExtra))
    }
    /// 将 `CompressContent` 字段转为 `ReferenceText` 格式
    pub fn text_reference(&self) -> WxResult<String> {
        let xml = self.compress_content()?;
        Ok(xml)
    }
    pub fn sender_id(&self) -> &str {
        &self.SenderId
    }
    pub fn sender_name(&self) -> &str {
        if self.is_sender() {
            return "<自己>";
        }
        match self.SenderName.as_str() {
            "" => {
                if let MessageType::SystemNotice = self.get_type() {
                    "<系统>"
                }
                else if self.RoomId.ends_with("@chatroom") {
                    "<失联>"
                }
                else {
                    "<对方>"
                }
            }
            named => named,
        }
    }
    pub fn image_path(&self) -> String {
        let full = self.BytesExtra.get_image_path();
        let mut path = full.rsplit(if cfg!(windows) { "\\" } else { "/" });
        let file = path.next().unwrap_or_default();
        let dir = path.next().unwrap_or_default();
        format!("{}/{}", dir, file)
    }
    pub fn image_link(&self, decoder: &XorDecryptor, wx_in: &Path, wx_out: &Path) -> WxResult<String> {
        let relative_path = self.BytesExtra.get_image_path();
        let absolute_path = wx_in.join(relative_path);
        println!("dat: {}", absolute_path.display());
        let input = std::fs::read(&absolute_path)?;

        let (ext, output) = unsafe { decoder.decrypt_bytes(&input) };

        let file_name = absolute_path.file_stem().and_then(|x| x.to_str()).unwrap_or_default();
        println!("ext: {}, file_name: {}", ext, file_name);
        let dir = match absolute_path.parent() {
            Some(o) => o.file_name().and_then(|s| s.to_str()).unwrap_or_default(),
            None => return Err(WxError::custom("找不到负极")),
        };
        let out_dir = wx_out.join("MsgAttach").join(dir);
        println!("out_dir: {}", out_dir.display());
        let out_file = out_dir.join(file_name);
        println!("out_file: {}", out_file.display());
        create_dir_all(out_dir)?;
        std::fs::write(&out_file, output)?;
        Ok(format!("file:///./MsgAttach/{}/{}", dir, file_name))
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
