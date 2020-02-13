use crate::{
    WxExport, WxResult,
    dsv_writer::{CsvLine, DsvWriter},
};
use async_stream::try_stream;
use chrono::{DateTime, Local};
use futures_util::stream::TryStreamExt;
use sqlx::{
    Error, FromRow, Row, Sqlite,
    sqlite::{SqlitePoolOptions, SqliteRow},
};
use std::{
    fmt::{Debug, Formatter},
    path::PathBuf,
};
use tokio::{fs::File, io::AsyncWriteExt};

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
    /// 带有引用的文本消息
    ///
    /// 这种类型下 `StrContent` 为空，发送和引用的内容均在 `CompressContent` 中
    TextReference,
    /// 图片
    Image,
    /// 语音
    Voice,
    /// 视频
    Video,
    /// 动画表情
    ///
    /// 第三方开发的表情包
    Emoji,
    /// GIF 表情
    ///
    /// 用户上传的表情包
    /// - `CompressContent` 中有 CDN 链接
    EmojiGif,
    /// 二进制文件
    ///
    /// - `CompressContent` 中有文件名和下载链接
    /// - `BytesExtra` 中有本地保存的路径
    File,
    /// 电话
    PhoneCall,
    /// 分享的小程序
    ///
    /// - `CompressContent` 中有卡片信息
    /// - `BytesExtra` 中有封面缓存位置
    MiniProgram,
    /// 拍一拍
    PatFriend,
    /// 系统通知
    ///
    /// 居中出现的那种灰色文字
    SystemNotice,
    /// 邀请通知
    ///
    /// 特别包含你邀请别人加入群聊
    SystemInvite,
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
            (34, 0) => Self::Voice,
            (43, 0) => Self::Video,
            (47, 0) => Self::Emoji,
            (49, 6) => Self::File,
            (49, 8) => Self::EmojiGif,
            (49, 33) => Self::MiniProgram,
            (49, 36) => Self::MiniProgram,
            (49, 57) => Self::TextReference,
            (50, 0) => Self::PhoneCall,
            (10000, 0) => Self::SystemNotice,
            (10000, 8000) => Self::SystemInvite,
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
            binary_extra,
        })
    }
}

impl WxExport {
    /// 导出消息
    pub async fn export_message(&self) -> WxResult<()> {
        let mut file = File::create(self.db.join("MSG.csv")).await?;
        // UTF8 HEAD for Excel
        file.write_all(&[0xEF, 0xBB, 0xBF]).await?;
        let mut line = CsvLine::new();
        line.push_str("日期");
        line.push_str("会话");
        line.push_str("内容");
        line.push_str("类型");
        line.push_str("事件");
        file.write_all(line.finish().as_bytes()).await?;
        for id in 0..99 {
            let db_path = self.db.join(format!("Multi/MSG{}.db", id));
            self.export_message_on(db_path, &mut file).await?;
        }
        Ok(())
    }
    async fn export_message_on(&self, msg: PathBuf, file: &mut File) -> WxResult<()> {
        let micro_msg = self.db.join("MicroMsg.db");
        let path = micro_msg.to_str().unwrap_or_default();
        let db = SqlitePoolOptions::new();
        let db = db.connect(&msg.to_str().unwrap_or_default()).await?;
        let mut rows = sqlx::query_as::<Sqlite, MessageRow>(include_str!("get_msg.sql"))
            .bind(path) //
            .fetch(&db);
        while let Some(row) = rows.try_next().await? {
            let mut line = CsvLine::new();
            line.push_str(&row.time.format("%Y-%m-%d %H:%M:%S").to_string());
            line.push_str(&row.room_name);

            match row.r#type {
                MessageType::Text => {
                    line.push_str(&row.message);
                    line.push_str("Text");
                }
                MessageType::TextReference => {
                    line.push_str(&String::from_utf8_lossy(&row.binary));
                    line.push_str("TextReference")
                }
                MessageType::PatFriend => {
                    line.push_str(&row.message);
                    line.push_str("PatFriend")
                }
                MessageType::Unknown { type_id, sub_id } => {
                    line.push_str(&row.message);
                    line.push_str(&format!("Unknown({type_id},{sub_id})"))
                }
                _ => continue,
            }
            if row.is_sender {
                line.push_str("发送");
            }
            else {
                line.push_str("接收");
            }
            file.write_all(line.finish().as_bytes()).await?;
        }
        Ok(())
    }
}
