use crate::{WxArguments, utils::string_to_u8_vec};
use clap::Parser;
use wx_core::WxScanner;

#[derive(Clone, Debug, Parser)]
pub struct RunDecrypt {
    /// 获取到的 key
    #[arg(short, long)]
    pub key: Option<String>,
    /// key的编码格式，可选值：[hex,base64,string]
    #[arg(short = 'd', long, default_value = "hex")]
    pub encode: String,
    #[arg(long, default_value = "false")]
    /// 是否在解密时检查hmac
    pub check_hmac: bool,
}

impl RunDecrypt {
    #[cfg(windows)]
    pub fn run(&self, c: WxArguments) -> anyhow::Result<()> {
        let mut wechat_info = WxScanner::default();
        if let Some(key) = self.key.as_ref() {
            let key_vec = string_to_u8_vec(&key, &self.encode)?;
            if key_vec.len() < 32 {
                return Err(anyhow::anyhow!("请输入正确的key"));
            }
            else {
                wechat_info.profile.aes256 = key_vec[0..32].try_into()?;
            }
        }
        else {
            wechat_info.open_wechat_process(&c.offset_map, &c.process_id, &c.process_name, &c.module_name)?;
        };
        // decrypt(
        //     &args.save_path,
        //     &args.decrypt_path,
        //     &wechat_info.aes256_key,
        //     check_hmac,
        //     |s| println!("{s}"),
        // )?;
        Ok(())
    }
    #[cfg(not(windows))]
    pub fn run(self, _: WxArguments) -> anyhow::Result<()> {
        Ok(())
    }
}
