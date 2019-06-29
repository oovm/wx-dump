use crate::WxArguments;
use clap::Parser;
use wx_core::WxScanner;

#[derive(Clone, Debug, Parser)]
pub struct RunCopy {
    /// 登录名，将依次值寻找复制聊天记录数据库
    #[arg(short, long)]
    account: Option<String>,
}

impl RunCopy {
    pub fn run(self, c: WxArguments) -> anyhow::Result<()> {
        let mut wechat_info = WxScanner::default();
        wechat_info.open_wechat_process(&c.offset_map, &c.process_id, &c.process_name, &c.module_name)?;
        println!("{:#?}", wechat_info.profile);
        Ok(())
    }
}
