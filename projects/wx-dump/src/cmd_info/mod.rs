use crate::WxArguments;
use clap::Parser;
use wx_core::WxScanner;

#[derive(Clone, Debug, Parser)]
pub struct RunInfo {}

impl RunInfo {
    #[cfg(windows)]
    pub fn run(self, c: WxArguments) -> anyhow::Result<()> {
        let mut wechat_info = WxScanner::default();
        wechat_info.open_wechat_process(&c.offset_map, &c.process_id, &c.process_name, &c.module_name)?;
        println!("{:#?}", wechat_info.profile);
        Ok(())
    }
    #[cfg(not(windows))]
    pub fn run(self, _: WxArguments) -> anyhow::Result<()> {
        Ok(())
    }
}
