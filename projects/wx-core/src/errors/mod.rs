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
    /// An unknown error.
    Window {
        error: windows::core::Error,
    },
    Custom {
        message: String,
    },
    UnsupportedOffset {
        version: String,
        field: String,
    },
    InvalidKey {
        key: [u8; 32],
        path: PathBuf,
    },
}
