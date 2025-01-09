use crate::WxResult;
use chrono::{DateTime, Local};
use futures_util::stream::BoxStream;
use lz4_flex::decompress;
use rust_xlsxwriter::{ExcelDateTime, XlsxError};
use sqlx::{Error, FromRow, Pool, Row, Sqlite, sqlite::SqliteRow};
use std::{
    fmt::{Debug, Formatter},
    path::Path,
};

pub mod message_type;

#[allow(non_snake_case)]
#[derive(Debug, FromRow)]
pub(crate) struct MessageRow {
    Sequence: i32,
    Type: i32,
    SubType: i32,
    CreateTime: i32,
    StrContent: String,
    CompressContent: Vec<u8>,
    BytesExtra: Vec<u8>,
    IsSender: i32,
    StrTalker: String,
    strNickName: String,
}

impl MessageRow {
    pub fn query<'a>(db: &'a Pool<Sqlite>, path: &Path) -> BoxStream<'a, sqlx::Result<MessageRow>> {
        let micro_msg = path.join("MicroMsg.db");
        sqlx::query_as::<Sqlite, MessageRow>(include_str!("get_msg.sql"))
            .bind(micro_msg.to_string_lossy().to_string())
            .fetch(db)
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
    pub fn text(&self) -> &str {
        &self.StrContent
    }
    /// 将 `CompressContent` 字段转为 `ReferenceText` 格式
    pub fn text_reference(&self) -> WxResult<String> {
        let xml = self.binary_as_string()?;
        Ok(xml)
    }
    pub fn room_id(&self) -> &str {
        &self.StrTalker
    }
    pub fn room_name(&self) -> &str {
        &self.strNickName
    }
    pub fn excel_time(&self) -> ExcelDateTime {
        ExcelDateTime::from_timestamp(self.CreateTime as i64).unwrap_or_default()
    }
    pub fn is_sender(&self) -> bool {
        self.IsSender == 1
    }
}
