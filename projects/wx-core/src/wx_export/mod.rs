use crate::{MessageType, WxResult, XlsxWriter, helpers::url_display, orm_types::MessageData};
use futures_util::TryStreamExt;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::PathBuf;
use tracing::{error, trace};
use crate::orm_types::extensions::SqliteHelper;

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
        excel.write_title("消息ID", 15.0)?;
        excel.write_title("日期", 25.0)?;
        excel.write_title("群ID", 30.0)?;
        excel.write_title("群名", 30.0)?;
        excel.write_title("发送人ID", 30.0)?;
        excel.write_title("内容", 60.0)?;
        excel.write_title("类型", 15.0)?;
        excel.write_title("事件", 10.0)?;
        excel.write_title("额外信息", 400.0)?;
        for id in 0..99 {
            let db_path = self.dir.join(format!("Multi/MSG{}.db", id));
            if !db_path.exists() {
                break;
            }

            trace!("读取聊天记录: {}", db_path.display());
            match self.export_message_on(db_path, &mut excel).await {
                Ok(_) => {}
                Err(e) => error!("读取聊天记录失败: {}", e),
            }
        }
        let msg = self.dir.join("MSG.xlsx");
        url_display(&msg, |url| println!("写入聊天记录: {}", url));
        Ok(excel.save(&msg)?)
    }
    async fn export_message_on(&self, msg: PathBuf, w: &mut XlsxWriter) -> WxResult<()> {
        let db = SqliteHelper::open(&msg.to_str().unwrap_or_default())?;
        let mut rows = MessageData::query(&db, &self.dir);
        while let Some(mut row) = rows.try_next().await? {
            w.next_line();
            w.write_id64(row.message_id())?;
            w.write_time(row.unix_time())?;
            w.write_data(row.room_id())?;
            w.write_data(row.room_name())?;
            w.write_data(row.sender_id())?;
            match row.get_type() {
                MessageType::TextReference => w.write_data(row.text_reference())?,
                MessageType::Image => w.write_data(row.image_path())?,
                MessageType::Revoke => w.write_data(row.revoke_message())?,
                MessageType::PhoneCall => w.write_data(row.voip_message())?,
                MessageType::Voice => w.write_data(row.voice_message())?,
                _ => w.write_data(row.text_message())?,
            }
            w.write_data(row.get_type())?;
            if row.is_sender() {
                w.write_data("发送")?;
            }
            else {
                w.write_data("接收")?;
            }
            w.write_data(row.extra_info())?;
        }
        Ok(())
    }
}
