use crate::{WxArguments, utils::u8_to_string};
use clap::Parser;
use wx_core::WxScanner;

#[derive(Clone, Debug, Parser)]
pub struct RunRead {
    /// 想要从哪个位置获取值
    #[arg(short, long)]
    index: usize,
    /// 想要获取的值的长度
    #[arg(short, long)]
    len: usize,
    /// 返回值的的编码格式，可选值：[hex, base64, string, u64be, u64le, u32be, u32le, u16be, u16le, i64be, i64le, i32be, i32le, i164be, i16le]
    #[arg(short = 'd', long)]
    encode: Option<String>,
    /// offset 为真实地址
    #[arg(short = 'a', long)]
    absolute_address: bool,
}

impl RunRead {
    pub fn run(&self, c: WxArguments) -> anyhow::Result<()> {
        let mut wechat_info = WxScanner::default();
        wechat_info.open_wechat_process_with_out_info(&c.process_id, &c.process_name, &c.module_name)?;
        let data = wechat_info.read_memory(self.index, self.len, self.absolute_address)?;
        if let Some(encode) = self.encode.as_ref() {
            println!("{}", u8_to_string(&data, &encode)?);
        }
        else {
            println!("{:?}", &data);
        };
        anyhow::Ok(())
    }
}
