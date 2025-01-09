use super::*;
use rust_xlsxwriter::{ Format, IntoExcelData,  Worksheet, XlsxError};

impl Error for WxError {}

impl Debug for WxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.kind, f)
    }
}

impl Display for WxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.kind, f)
    }
}

impl Display for WxErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom { message } => write!(f, "{}", message),
            Self::UnsupportedOffset { version, field } => write!(f, "微信版本 {} 不支持读取 {}", version, field),
            #[cfg(windows)]
            Self::Window { error } => write!(f, "系统错误: {}", error),
            Self::InvalidKey { key: _, path } => write!(f, "秘钥不匹配, 无法解密 {}", path.display()),
            Self::DatabaseError { error } => write!(f, "数据库错误: {}", error),
            Self::DecodeError { algorithm, message } => write!(f, "{} 解码错误: {}", algorithm, message),
        }
    }
}

impl IntoExcelData for WxError {
    fn write(self, worksheet: &mut Worksheet, row: u32, col: u16) -> Result<&mut Worksheet, XlsxError> {
        worksheet.write_string(row, col, &self.to_string())
    }

    fn write_with_format<'a>(self, ws: &'a mut Worksheet, r: u32, c: u16, f: &Format) -> Result<&'a mut Worksheet, XlsxError> {
        ws.write_string_with_format(r, c, &self.to_string(), f)
    }
}
