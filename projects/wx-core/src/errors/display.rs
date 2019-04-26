use super::*;

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
            Self::Window { error } => write!(f, "系统错误: {}", error),
            Self::InvalidKey { key: _, path } => write!(f, "秘钥不匹配, 无法解密 {}", path.display()),
            Self::DatabaseError { error } => write!(f, "数据库错误: {}", error),
        }
    }
}
