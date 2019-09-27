use anyhow::Ok;
use std::env::current_dir;

mod cmd_copy;
mod cmd_decrypt;
mod cmd_export;
mod cmd_info;
mod cmd_read;
mod cmd_read_memory;
mod cmd_search;
mod cmd_wx_path;

mod utils;

const DEFAULT_SAVE_DIR: &str = "target";

pub use crate::{
    cmd_copy::RunCopy, cmd_decrypt::RunDecrypt, cmd_export::RunExport, cmd_info::RunInfo, cmd_read::RunRead,
    cmd_search::RunSearch,
};
use clap::{Parser, Subcommand};
use wx_core::{WxDecryptor, WxScanner, helpers::read_database};

/// 微信聊天记录导出工具
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct WxDump {
    #[command(flatten)]
    args: WxArguments,
    #[command(subcommand)]
    cmds: Option<WxCommands>,
}

#[derive(Parser, Debug, Default)]
pub struct WxArguments {
    /// 指定偏移量文件
    #[arg(short = 'm', long, value_name = "json 文件", default_value = "version-list.json")]
    offset_map: Option<String>,
    /// 指定微信聊天记录的文件夹，不填写时会默认指定系统文档文件夹下的 WeChat Files 文件夹
    #[arg(short = 'd', long, value_name = "微信文件夹")]
    wechat_path: Option<String>,
    /// 解密后的数据将存放于该文件夹
    #[arg(long, value_name = "解密文件夹")]
    decrypt_path: Option<String>,
    /// 指定微信进程号
    #[arg(long)]
    process_id: Option<u32>,
    /// 指定微信进程名
    #[arg(long, default_value = "WeChat.exe")]
    process_name: String,
    /// 指定模块名
    #[arg(long, default_value = "WeChatWin.dll")]
    module_name: String,
}

#[derive(Clone, Debug, Subcommand)]
pub enum WxCommands {
    /// 显示当前登录的微信用户的信息
    Info(RunInfo),
    /// 解密聊天记录数据库
    Decrypt(RunDecrypt),
    /// 从已解密的数据库中导出聊天记录
    Export(RunExport),
    /// 从内存中搜索指定信息
    Search(RunSearch),
    /// 从内存中指定的位置搜索信息
    Read(RunRead),
    /// 从内存中指定的位置搜索信息
    Copy(RunCopy),
}

impl WxDump {
    pub async fn run(self) -> anyhow::Result<()> {
        match self.cmds {
            Some(subs) => match subs {
                WxCommands::Info(cmd) => cmd.run(self.args),
                WxCommands::Decrypt(cmd) => cmd.run(self.args),
                WxCommands::Search(cmd) => cmd.run(self.args),
                WxCommands::Read(cmd) => cmd.run(self.args),
                WxCommands::Export(cmd) => cmd.run(self.args).await,
                WxCommands::Copy(cmd) => cmd.run(self.args),
            },
            None => Self::run_auto(self.args).await,
        }
    }
    pub async fn run_auto(c: WxArguments) -> anyhow::Result<()> {
        let mut wechat_info = WxScanner::default();
        wechat_info.open_wechat_process(&c.offset_map, &c.process_id, &c.process_name, &c.module_name)?;
        println!("{:#?}", wechat_info.profile);
        let data = read_database(&c.wechat_path).await.unwrap();
        for (user, path) in data.iter() {
            let output_path = current_dir()?.join(DEFAULT_SAVE_DIR).join(user);
            let decryptor = WxDecryptor {
                source_path: path.to_path_buf(),
                output_path,
                key: wechat_info.profile.aes256.to_owned(),
                need_check_hmac: false,
            };
            match decryptor.decrypt().await {
                Err(e) => {
                    println!("{e}");
                }
                _ => {}
            }
        }
        Ok(())
    }
}
