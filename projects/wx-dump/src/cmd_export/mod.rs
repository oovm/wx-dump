use crate::WxArguments;
use clap::Parser;
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};
use tokio::fs::DirEntry;
use tracing::{error, trace};
use wx_core::{WxExport, WxScanner};

#[derive(Clone, Debug, Parser)]
pub struct RunExport {
    /// 数据库目录
    pub path: Option<String>,
}

impl RunExport {
    pub async fn run(&self, args: WxArguments) -> anyhow::Result<()> {
        match self.path.as_ref() {
            Some(s) => self.export_db(&args, PathBuf::from(s)).await?,
            None => {
                let dump = current_dir()?.join("wx-dump");
                trace!("dump dir: {}", dump.display());
                for dir in std::fs::read_dir(dump)? {
                    match dir {
                        Ok(o) => match self.export_db(&args, o.path()).await {
                            Ok(_) => {}
                            Err(e) => error!("{}", e),
                        },
                        Err(e) => error!("{}", e),
                    }
                }
            }
        };
        Ok(())
    }
    pub async fn export_db(&self, c: &WxArguments, dir: PathBuf) -> anyhow::Result<()> {
        trace!("dump file: {}", dir.display());
        let wx = WxExport { db: dir };
        wx.export_message().await?;
        Ok(())
    }
}
