#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, rustdoc::missing_crate_level_docs)]
#![doc = include_str!("../readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]

mod errors;
pub mod helpers;
mod orm_types;
mod wx_decrypt;
mod wx_export;
mod wx_scanner;

pub use crate::{
    errors::{WxError, WxErrorKind, WxResult},
    wx_decrypt::WxDecryptor,
    wx_export::WxExport,
    wx_scanner::{WeChatProfile, WxScanner},
};
