use super::*;
use aes::cipher::{InvalidLength, block_padding::UnpadError};
use lz4_flex::block::DecompressError;
use std::{
    array::TryFromSliceError,
    num::ParseIntError,
    path::{Path, StripPrefixError},
    string::FromUtf8Error,
};
use rust_xlsxwriter::XlsxError;

impl From<WxErrorKind> for WxError {
    fn from(value: WxErrorKind) -> Self {
        Self { kind: Box::new(value) }
    }
}
impl From<std::io::Error> for WxError {
    fn from(error: std::io::Error) -> Self {
        WxError { kind: Box::new(WxErrorKind::Custom { message: error.to_string() }) }
    }
}
impl From<StripPrefixError> for WxError {
    fn from(error: StripPrefixError) -> Self {
        WxError { kind: Box::new(WxErrorKind::DecodeError { algorithm: "StripPrefix", message: error.to_string() }) }
    }
}

impl From<InvalidLength> for WxError {
    fn from(error: InvalidLength) -> Self {
        WxError { kind: Box::new(WxErrorKind::DecodeError { algorithm: "InvalidLength", message: error.to_string() }) }
    }
}
impl From<UnpadError> for WxError {
    fn from(error: UnpadError) -> Self {
        WxError { kind: Box::new(WxErrorKind::DecodeError { algorithm: "Unpad", message: error.to_string() }) }
    }
}

impl From<FromUtf8Error> for WxError {
    fn from(error: FromUtf8Error) -> Self {
        WxError { kind: Box::new(WxErrorKind::DecodeError { algorithm: "utf8", message: error.to_string() }) }
    }
}
impl From<ParseIntError> for WxError {
    fn from(error: ParseIntError) -> Self {
        WxError { kind: Box::new(WxErrorKind::DecodeError { algorithm: "i64", message: error.to_string() }) }
    }
}
impl From<serde_json::Error> for WxError {
    fn from(error: serde_json::Error) -> Self {
        WxError { kind: Box::new(WxErrorKind::DecodeError { algorithm: "json", message: error.to_string() }) }
    }
}
impl From<base64::DecodeError> for WxError {
    fn from(error: base64::DecodeError) -> Self {
        WxError { kind: Box::new(WxErrorKind::DecodeError { algorithm: "base64", message: error.to_string() }) }
    }
}
impl From<TryFromSliceError> for WxError {
    fn from(error: TryFromSliceError) -> Self {
        WxError { kind: Box::new(WxErrorKind::DecodeError { algorithm: "[u8]", message: error.to_string() }) }
    }
}
impl From<DecompressError> for WxError {
    fn from(error: DecompressError) -> Self {
        WxError { kind: Box::new(WxErrorKind::DecodeError { algorithm: "lz4", message: error.to_string() }) }
    }
}
#[cfg(windows)]
impl From<windows::core::Error> for WxError {
    fn from(error: windows::core::Error) -> Self {
        WxError { kind: Box::new(WxErrorKind::Window { error }) }
    }
}
impl From<sqlx::Error> for WxError {
    fn from(error: sqlx::Error) -> Self {
        WxError { kind: Box::new(WxErrorKind::DatabaseError { error }) }
    }
}
impl From<XlsxError> for WxError {
    fn from(error: XlsxError) -> Self {
        WxError { kind: Box::new(WxErrorKind::DecodeError { algorithm: "xlsx", message: error.to_string() }) }
    }
}
impl WxError {
    /// 自定义错误
    pub fn custom(message: impl ToString) -> WxError {
        WxError { kind: Box::new(WxErrorKind::Custom { message: message.to_string() }) }
    }
    /// 非法偏移
    pub fn unsupported_offset(version: &str, field: &str) -> WxError {
        WxError { kind: Box::new(WxErrorKind::UnsupportedOffset { version: version.to_string(), field: field.to_string() }) }
    }
    /// 非法秘钥
    pub fn invalid_key(key: [u8; 32], path: &Path) -> WxError {
        WxError { kind: Box::new(WxErrorKind::InvalidKey { key, path: path.to_owned() }) }
    }
}
