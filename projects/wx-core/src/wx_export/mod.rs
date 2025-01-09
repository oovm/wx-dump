use crate::{MessageType, WxResult, XlsxWriter, orm_types::MessageRow};
use futures_util::TryStreamExt;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::PathBuf;
use tracing::trace;

/// 导出微信数据库中的数据
#[derive(Debug)]
pub struct WxExport {
    dir: PathBuf,
}

impl WxExport {
    /// 数据库所在文件路径
    pub fn new(path: PathBuf) -> Self {
        WxExport { dir: path }
    }
}

impl WxExport {
    /// 导出消息
    pub async fn export_message(&self) -> WxResult<()> {
        let micro_msg = self.dir.join("MicroMsg.db");
        if !micro_msg.exists() {
            return Ok(());
        }
        let mut excel = XlsxWriter::default();
        excel.write_title("日期", 30.0)?;
        excel.write_title("群ID", 30.0)?;
        excel.write_title("群名", 30.0)?;
        excel.write_title("内容", 60.0)?;
        excel.write_title("类型", 15.0)?;
        excel.write_title("事件", 10.0)?;
        for id in 0..99 {
            let db_path = self.dir.join(format!("Multi/MSG{}.db", id));
            trace!("读取聊天记录: {}", db_path.display());
            match self.export_message_on(db_path, &mut excel).await {
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        let msg = self.dir.join("MSG.xlsx");
        println!("写入聊天记录: {}", msg.display());
        Ok(excel.save(&msg)?)
    }
    async fn export_message_on(&self, msg: PathBuf, w: &mut XlsxWriter) -> WxResult<()> {
        let db = SqlitePoolOptions::new();
        let db = db.connect(&msg.to_str().unwrap_or_default()).await?;
        let mut rows = MessageRow::query(&db, &self.dir);
        while let Some(row) = rows.try_next().await? {
            w.next_line();
            w.write_data(row.excel_time())?;
            w.write_data(row.room_id())?;
            w.write_data(row.room_name())?;
            match row.get_type() {
                MessageType::Text => {
                    w.write_data(row.text())?;
                    w.write_data("Text")?;
                }
                MessageType::TextReference => {
                    w.write_data(row.text_reference())?;
                    w.write_data("TextReference")?;
                }
                MessageType::PatFriend => {
                    w.write_data(row.text())?;
                    w.write_data("PatFriend")?;
                }
                MessageType::Unknown { type_id, sub_id } => {
                    w.write_data(row.text())?;
                    w.write_data(format!("Unknown({type_id},{sub_id})"))?;
                }
                _ => continue,
            }
            if row.is_sender() {
                w.write_data("发送")?;
            }
            else {
                w.write_data("接收")?;
            }
        }
        Ok(())
    }
}
