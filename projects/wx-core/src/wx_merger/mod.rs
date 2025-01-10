use crate::WxResult;
use std::path::PathBuf;
use sqlx::sqlite::SqliteConnectOptions;

/// 导出微信数据库中的数据
#[derive(Debug)]
pub struct WxMerger {
    dir: PathBuf,
}

impl WxMerger {
    /// 数据库所在文件路径
    pub fn new(path: PathBuf) -> Self {
        WxMerger { dir: path }
    }
    async fn merge(&self) -> WxResult<()> {
        let _ = SqliteConnectOptions::new()
            .filename(self.dir.join("MSG.db"))
            .create_if_missing(true);
        todo!()
    }
}
