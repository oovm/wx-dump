use crate::WxResult;
use chrono::{DateTime, Local};
use sqlx::{
    Error, FromRow, Row,
    sqlite::{SqlitePoolOptions, SqliteRow},
};

#[derive(Debug)]
pub struct MessageData {
    r#type: MessageType,
    time: DateTime<Local>,
    message: String,
}

#[derive(Debug)]
pub enum MessageType {
    /// 纯文本
    Text,
    Image,
    Unknown {
        type_id: i32,
        sub_id: i32,
    },
}

impl From<(i32, i32)> for MessageType {
    fn from(value: (i32, i32)) -> Self {
        match value {
            (1, 0) => Self::Text,
            (3, 0) => Self::Image,
            (x, y) => Self::Unknown { type_id: x, sub_id: y },
        }
    }
}

impl<'a> FromRow<'a, SqliteRow> for MessageData {
    fn from_row(row: &'a SqliteRow) -> Result<Self, Error> {
        let time: i32 = row.try_get("CreateTime")?;

        let ty = row.try_get("Type")?;
        let sub = row.try_get("SubType")?;
        let kind = (ty, sub).into();

        let message = row.try_get("StrContent")?;

        let utc_datetime = DateTime::from_timestamp(time as i64, 0).unwrap();
        let local_datetime: DateTime<Local> = utc_datetime.with_timezone(&Local);
        Ok(MessageData { r#type: kind, message, time: local_datetime })
    }
}

// use sqlx::{query, SqlitePool};
//
#[tokio::test]
async fn main() -> WxResult<()> {
    let db = SqlitePoolOptions::new()
        .connect(r#""#)
        .await
        .unwrap();
    let out: Vec<MessageData> = sqlx::query_as(include_str!("get_message.sql")).fetch_all(&db).await.unwrap();
    println!("{:#?}", out);

    Ok(())
}
