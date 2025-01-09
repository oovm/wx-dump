use crate::{WxExport, WxResult, dsv_writer::CsvLine};

use chrono::{DateTime, Local};
use futures_util::stream::TryStreamExt;
use lz4_flex::decompress;
use rust_xlsxwriter::{IntoExcelData, Workbook, Worksheet, XlsxError};
use sqlx::{
    Error, FromRow, Row, Sqlite,
    sqlite::{SqlitePoolOptions, SqliteRow},
};
use std::{
    fmt::{Debug, Formatter},
    ops::AddAssign,
    path::{Path, PathBuf},
};
use tokio::{fs::File, io::AsyncWriteExt};

#[allow(non_snake_case)]
struct MessageRow {
    r#type: MessageType,
    time: DateTime<Local>,
    message: String,
    CompressContent: Vec<u8>,
    extra: Vec<u8>,
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
            .field("CompressContent", &self.CompressContent.len())
            .field("binary_extra", &self.extra.len())
            .field("is_sender", &self.is_sender)
            .field("room_id", &self.room_id)
            .field("room_name", &self.room_name)
            .finish()
    }
}

impl MessageRow {
    pub fn binary_as_string(&self) -> WxResult<String> {
        let mut decompress = decompress(&self.CompressContent, 0x10004)?;
        // 移除字符串末尾的 `<NUL>`
        let tail = decompress.pop();
        debug_assert_eq!(tail, Some(0x00));
        Ok(String::from_utf8(decompress)?)
    }
    /// 将 `CompressContent` 字段转为 `ReferenceText` 格式
    pub fn binary_as_message(&self) -> WxResult<String> {
        let xml = self.binary_as_string()?;
        Ok(xml)
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
            CompressContent: binary,
            room_name: user_name,
            extra: binary_extra,
        })
    }
}

pub struct ExcelWriter {
    db: Workbook,
    table: Worksheet,
    current_line: u32,
}

impl Default for ExcelWriter {
    fn default() -> Self {
        let mut wb = Workbook::new();
        let ws = wb.new_worksheet_with_constant_memory();
        Self { db: wb, table: ws, current_line: 0 }
    }
}
impl ExcelWriter {
    pub fn write_title(&mut self, index: u16, data: impl IntoExcelData, width: f64) -> Result<(), XlsxError> {
        self.table.set_column_width(index, width)?;
        self.table.write(0, index, data)?;
        Ok(())
    }
    pub fn write_data(&mut self, index: u16, data: impl IntoExcelData) -> Result<(), XlsxError> {
        self.table.write(self.current_line, index, data)?;
        Ok(())
    }
    pub fn save(&mut self, path: &Path) -> Result<(), XlsxError> {
        let mut file = std::fs::File::create(path)?;
        self.db.save_to_writer(&mut file)
    }
    pub fn next_line(&mut self) {
        self.current_line.add_assign(1)
    }
}

impl WxExport {
    /// 导出消息
    pub async fn export_message(&self) -> WxResult<()> {
        let mut excel = ExcelWriter::default();
        excel.write_title(0, "日期", 30.0)?;
        excel.write_title(1, "会话", 30.0)?;
        excel.write_title(2, "内容", 60.0)?;
        excel.write_title(3, "类型", 15.0)?;
        excel.write_title(4, "事件", 10.0)?;
        for id in 0..99 {
            let db_path = self.db.join(format!("Multi/MSG{}.db", id));
            match self.export_message_on(db_path, &mut excel).await {
                Ok(_) => {}
                Err(_) => break,
            }
        }
        Ok(excel.save(&self.db.join("MSG.xlsx"))?)
    }
    async fn export_message_on(&self, msg: PathBuf, w: &mut ExcelWriter) -> WxResult<()> {
        let micro_msg = self.db.join("MicroMsg.db");
        let path = micro_msg.to_str().unwrap_or_default();
        let db = SqlitePoolOptions::new();
        let db = db.connect(&msg.to_str().unwrap_or_default()).await?;
        let mut rows = sqlx::query_as::<Sqlite, MessageRow>(include_str!("get_msg.sql"))
            .bind(path) //
            .fetch(&db);
        while let Some(row) = rows.try_next().await? {
            w.next_line();

            w.write_data(0, row.time.to_rfc3339())?;
            w.write_data(0, row.time.to_rfc3339())?;
            w.write_data(1, &row.room_name)?;
            match row.r#type {
                MessageType::Text => {
                    w.write_data(2, row.message)?;
                    w.write_data(3, "Text")?;
                }
                MessageType::TextReference => {
                    w.write_data(2, row.binary_as_message().unwrap_or_else(|e| e.to_string()))?;
                    w.write_data(3, "TextReference")?;
                }
                MessageType::PatFriend => {
                    w.write_data(2, row.message)?;
                    w.write_data(3, "PatFriend")?;
                }
                MessageType::Unknown { type_id, sub_id } => {
                    w.write_data(2, row.message)?;
                    w.write_data(3, format!("Unknown({type_id},{sub_id})"))?;
                }
                _ => continue,
            }
            if row.is_sender {
                w.write_data(4, "发送")?;
            }
            else {
                w.write_data(4, "接收")?;
            }
        }
        Ok(())
    }
    pub fn next_line(&mut self) {
        self.line.add_assign(1);
    }
}
