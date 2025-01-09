use crate::{WxArguments, utils::string_to_u8_vec};
use clap::Parser;
use wx_core::WxScanner;

#[derive(Clone, Debug, Parser)]
pub struct RunSearch {
    /// 要搜索的内容
    #[arg(short, long)]
    str: String,
    /// key的编码格式，可选值：[hex,base64,string, u64be, u64le, u32be, u32le, u16be, u16le, i64be, i64le, i32be, i32le, i164be, i16le]
    #[arg(short = 'd', long, default_value = "string")]
    encode: String,
    /// 返回真实地址
    #[arg(long)]
    real_addr: bool,
    /// 从微信的所有模块中搜索数据
    #[arg(long)]
    from_all_modules: bool,
    /// 从微信的所有数据中搜索数据
    #[arg(long)]
    from_all_data: bool,
    /// 显示未搜索到的数据信息
    #[arg(long)]
    show_no_found_info: bool,
    /// 显示错误信息
    #[arg(long)]
    show_error_info: bool,
}
impl RunSearch {
    #[cfg(windows)]
    pub fn run(&self, c: WxArguments) -> anyhow::Result<()> {
        let mut wechat_info = WxScanner::default();
        wechat_info.open_wechat_process_with_out_info(&c.process_id, &c.process_name, &c.module_name)?;
        let data = string_to_u8_vec(&self.str, &self.encode)?;
        if self.from_all_data {
            wechat_info.search_in_all_wechat_data(&data, self.real_addr, self.show_no_found_info, self.show_error_info)?;
        }
        if self.from_all_modules {
            wechat_info.search_in_all_wechat_modules(&data, self.real_addr, self.show_no_found_info, self.show_error_info)?;
        }
        if !(self.from_all_modules || self.from_all_data) {
            println!("{:?}", wechat_info.memory_search(&data, self.real_addr)?);
        }
        anyhow::Ok(())
    }
    #[cfg(not(windows))]
    pub fn run(self, _: WxArguments) -> anyhow::Result<()> {
        Ok(())
    }
}
