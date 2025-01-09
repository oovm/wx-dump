use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    path::PathBuf,
};

mod convert;
mod display;

/// The result type of this crate.
pub type WxResult<T> = Result<T, WxError>;

/// A boxed error kind, wrapping an [WxErrorKind].
pub struct WxError {
    kind: Box<WxErrorKind>,
}

/// The kind of [WxError].
#[derive(Debug)]
pub enum WxErrorKind {
    /// Windows 系统错误
    #[cfg(windows)]
    Window {
        /// 错误对象
        error: windows::core::Error,
    },
    /// 不支持使用偏移量
    UnsupportedOffset {
        /// 当前版本
        version: String,
        /// 无法读取的字段
        field: String,
    },
    /// 解密失败的 key
    InvalidKey {
        /// 秘钥
        key: [u8; 32],
        /// 待解密的文件夹
        path: PathBuf,
    },
    /// 数据库错误
    DatabaseError {
        /// 数据库错误
        error: sqlx::Error,
    },
    /// 解码失败
    DecodeError {
        /// 解码算法
        algorithm: &'static str,
        /// 错误消息
        message: String,
    },
    /// 自定义报错
    Custom {
        /// 报错信息
        message: String,
    },
}
