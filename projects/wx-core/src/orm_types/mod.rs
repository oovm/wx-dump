use crate::{WxExport, WxResult};
use chrono::{DateTime, Local};
use sqlx::{
    sqlite::{SqlitePoolOptions, SqliteRow}, Error, FromRow,
    Row,
};
use std::{
    fmt::{Debug, Formatter},
    path::PathBuf,
};


struct MessageRow {
    r#type: MessageType,
    time: DateTime<Local>,
    message: String,
    binary: Vec<u8>,
    binary_extra: Vec<u8>,
    is_sender: bool,
    // room_id: String,
    room_id: String,
    room_name: String,
}

impl Debug for MessageRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageRaw")
            .field("type", &self.r#type)
            .field("time", &self.time)
            .field("message", &self.message)
            .field("binary", &self.binary.len())
            .field("binary_extra", &self.binary_extra.len())
            .field("is_sender", &self.is_sender)
            .field("room_id", &self.room_id)
            .field("room_name", &self.room_name)
            .finish()
    }
}

#[derive(Debug)]
pub enum MessageType {
    /// 纯文本
    Text,
    /// 图片
    Image,
    /// 二进制文件
    File,
    /// 未知类型
    Unknown {
        /// 类别 id
        type_id: i32,
        /// 子类 id
        sub_id: i32,
    },
}

impl From<(i32, i32)> for MessageType {
    fn from(value: (i32, i32)) -> Self {
        match value {
            (1, 0) => Self::Text,
            (3, 0) => Self::Image,
            (49, 6) => Self::File,
            (x, y) => Self::Unknown { type_id: x, sub_id: y },
        }
    }
}

impl<'a> FromRow<'a, SqliteRow> for MessageRow {
    fn from_row(row: &'a SqliteRow) -> Result<Self, Error> {
        let time: i32 = row.try_get("CreateTime")?;
        let is_sender: bool = row.try_get("IsSender")?;

        let ty = row.try_get("Type")?;
        let sub = row.try_get("SubType")?;
        let kind = (ty, sub).into();

        let message = row.try_get("StrContent")?;
        let binary: Vec<u8> = row.try_get("CompressContent")?;
        let binary_extra: Vec<u8> = row.try_get("BytesExtra")?;
        let user_id = row.try_get("StrTalker")?;
        let user_name = row.try_get("strNickName")?;
        // let room_id = row.try_get("UsrName")?;

        let utc_datetime = DateTime::from_timestamp(time as i64, 0).unwrap();
        let local_datetime: DateTime<Local> = utc_datetime.with_timezone(&Local);
        Ok(MessageRow {
            r#type: kind,
            message,
            time: local_datetime,
            is_sender,
            // room_id,
            room_id: user_id,
            binary,
            room_name: user_name,
            binary_extra
        })
    }
}

impl WxExport {
    pub async fn read(&self) -> WxResult<()> {
        let micro_msg = self.db.join("MicroMsg.db");
        let msg = self.db.join("Multi/MSG0.db");
        let path = micro_msg.to_str().unwrap_or_default();
        let db = SqlitePoolOptions::new();
        let db = db.connect(&msg.to_str().unwrap_or_default()).await.unwrap();
        let out: Vec<MessageRow> = sqlx::query_as(include_str!("get_message.sql")).bind(path).fetch_all(&db).await.unwrap();
        println!("{:#?}", out);
        Ok(())
    }
}

// use sqlx::{query, SqlitePool};
//
#[tokio::test]
async fn main() -> WxResult<()> {
    let wx = WxExport { db: PathBuf::from(r#""#) };
    wx.read().await?;

    Ok(())
}
