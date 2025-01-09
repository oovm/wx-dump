use crate::{DEFAULT_SAVE_DIR, WxArguments};
use clap::Parser;
use std::{env::current_dir, path::PathBuf};
use tracing::{error, trace};
use wx_core::WxExport;

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
                let dump = current_dir()?.join(DEFAULT_SAVE_DIR);

                trace!("dump dir: {}", dump.display());
                for dir in std::fs::read_dir(dump)? {
                    match dir {
                        Ok(o) => {
                            if !o.file_name().to_string_lossy().starts_with("wxid_") {
                                trace!("skip: {}", o.path().display());
                                continue;
                            }
                            match self.export_db(&args, o.path()).await {
                                Ok(_) => continue,
                                Err(e) => error!("{}", e),
                            }
                        }
                        Err(e) => error!("{}", e),
                    }
                }
            }
        };
        Ok(())
    }
    pub async fn export_db(&self, _: &WxArguments, dir: PathBuf) -> anyhow::Result<()> {
        trace!("dump file: {}", dir.display());
        let wx = WxExport { db: dir };
        wx.export_message().await?;
        Ok(())
    }
}
