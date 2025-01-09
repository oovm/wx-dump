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
mod xlsx_writer;

pub use crate::{
    errors::{WxError, WxErrorKind, WxResult},
    orm_types::{MessageType},
    wx_decrypt::WxDecryptor,
    wx_export::WxExport,
    wx_scanner::{WeChatProfile, WxScanner},
    xlsx_writer::XlsxWriter,
};
