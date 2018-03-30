use std::fmt::{Debug, Formatter};

#[cfg(windows)]
mod on_windows;

/// 微信个人数据
#[derive(Default)]
pub struct WeChatProfile {
    /// 微信版本号
    pub version: String,
    /// 微信用户名
    pub user_name: String,
    /// 微信昵称
    pub nick_name: String,
    /// 微信手机号
    pub mobile: String,
    /// 微信邮箱
    pub email: String,
    /// 微信加密秘钥
    pub aes256: [u8; 32],
}

#[cfg(windows)]
#[derive(Debug, Default)]
pub struct WxScanner {
    pub profile: WeChatProfile,
    process: windows::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32,
    handle: windows::Win32::Foundation::HANDLE,
    module: windows::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32,
}

impl Debug for WeChatProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let hex_string: String = self.aes256.iter().map(|byte| format!("{:02X}", byte)).collect();
        f.debug_struct("WeChatProfile")
            .field("version", &self.version)
            .field("user_name", &self.user_name)
            .field("nick_name", &self.nick_name)
            .field("mobile", &self.mobile)
            .field("email", &self.email)
            .field("key", &hex_string)
            .finish()
    }
}
