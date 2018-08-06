
use crate::WxResult;
use sqlx::{FromRow, SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;

#[derive(FromRow, Debug)]
pub struct MessageData {
    #[sqlx(rename = "StrContent")]
    message: String
}

// use sqlx::{query, SqlitePool};
//
#[tokio::test]
async fn main() -> WxResult<()> {
    let db = SqlitePoolOptions::new()
        .connect(r#""#)
        .await
        .unwrap();
    let out: Vec<MessageData> = sqlx::query_as("select * from MSG").fetch_all(&db).await.unwrap();
    println!("{:?}", out);

    Ok(())
}
