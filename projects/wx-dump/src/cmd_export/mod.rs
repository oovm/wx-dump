use crate::{DEFAULT_SAVE_DIR, WxArguments};
use clap::Parser;
use std::{env::current_dir, path::PathBuf};
use tracing::{error, trace};
use wx_core::{WxExport, helpers::url_display};
use wx_core::helpers::get_wechat_path;

#[derive(Clone, Debug, Parser)]
pub struct RunExport {
    /// 数据库目录
    pub path: Option<String>,
    /// 显示会话 ID
    #[arg(long, default_value_t = false)]
    pub room_id: bool,
    /// 显示发送者 ID
    #[arg(long, default_value_t = false)]
    pub sender_id: bool,
    /// 破译 `CompressContent` 中的信息
    #[arg(long, default_value_t = false)]
    pub compress_content: bool,
    /// 破译 `BytesExtra` 中的信息
    #[arg(long, default_value_t = false)]
    pub bytes_extra: bool,
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
    pub async fn export_db(&self, args: &WxArguments, dir: PathBuf) -> anyhow::Result<()> {
        url_display(&dir, |url| println!("正在导出个人目录: {}", url));
        let wx = WxExport {
            wx_in: args.wechat_path()?,
            wx_out: dir,
            dat: Default::default(),
            room_id: self.room_id,
            sender_id: self.sender_id,
            compress_content: self.compress_content,
            extra_content: self.bytes_extra,
        };
        wx.export_message().await?;
        Ok(())
    }
}
