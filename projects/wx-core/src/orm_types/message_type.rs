use crate::orm_types::MessageData;
use rust_xlsxwriter::{ColNum, Format, IntoExcelData, RowNum, Worksheet, XlsxError};
use std::fmt::{Display, Formatter};

/// 消息类型
#[derive(Copy, Clone, Debug)]
pub enum MessageType {
    /// 纯文本
    Text,
    /// 带有引用的文本消息
    ///
    /// 这种类型下 `StrContent` 为空，发送和引用的内容均在 `CompressContent` 中
    TextReference,
    /// 图片
    Image,
    /// 语音
    Voice,
    /// 视频
    Video,
    /// 视频推荐
    VideoRecommend,
    /// 动画表情
    ///
    /// 第三方开发的表情包
    Emoji,
    /// GIF 表情
    ///
    /// 用户上传的表情包
    /// - `CompressContent` 中有 CDN 链接
    EmojiGif,
    /// 二进制文件
    ///
    /// - `CompressContent` 中有文件名和下载链接
    /// - `BytesExtra` 中有本地保存的路径
    File,
    /// 电话
    PhoneCall,
    /// 分享的小程序
    ///
    /// - `CompressContent` 中有卡片信息
    /// - `BytesExtra` 中有封面缓存位置
    MiniProgram,
    /// 拍一拍
    FriendPatPat,
    /// 朋友推荐
    FriendRecommend,
    /// 群公告
    GroupNotice,
    /// 系统通知
    ///
    /// 居中出现的那种灰色文字
    SystemNotice,
    /// 邀请通知
    ///
    /// 特别包含你邀请别人加入群聊
    SystemInvite,
    /// 消息撤回
    Revoke,
    /// 未知类型
    Unknown {
        /// 类别 id
        type_id: i32,
        /// 子类 id
        sub_id: i32,
    },
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => f.write_str("Text"),
            Self::TextReference => f.write_str("TextReference"),
            Self::Image => f.write_str("Image"),
            Self::Voice => f.write_str("Voice"),
            Self::Video => f.write_str("Video"),
            Self::VideoRecommend => f.write_str("VideoRecommend"),
            Self::Emoji => f.write_str("Emoji"),
            Self::EmojiGif => f.write_str("EmojiGif"),
            Self::File => f.write_str("File"),
            Self::PhoneCall => f.write_str("PhoneCall"),
            Self::MiniProgram => f.write_str("MiniProgram"),
            Self::FriendPatPat => f.write_str("FriendPatPat"),
            Self::FriendRecommend => f.write_str("FriendRecommend"),
            Self::SystemNotice => f.write_str("SystemNotice"),
            Self::SystemInvite => f.write_str("SystemInvite"),
            Self::Unknown { type_id, sub_id } => write!(f, "Unknown({}, {})", type_id, sub_id),
            Self::GroupNotice => f.write_str("GroupNotice"),
            Self::Revoke => f.write_str("Revoke"),
        }
    }
}

impl MessageData {
    pub fn get_type(&self) -> MessageType {
        match (self.Type, self.SubType) {
            (1, 0) => MessageType::Text,
            (3, 0) => MessageType::Image,
            (34, 0) => MessageType::Voice,
            (43, 0) => MessageType::Video,
            (47, 0) => MessageType::Emoji,
            (49, 6) => MessageType::File,
            (49, 8) => MessageType::EmojiGif,
            (49, 33) => MessageType::MiniProgram,
            (49, 36) => MessageType::MiniProgram,
            (49, 57) => MessageType::TextReference,
            (49, 63) => MessageType::VideoRecommend,
            (49, 87) => MessageType::GroupNotice,
            (49, 88) => MessageType::VideoRecommend,
            (50, 0) => MessageType::PhoneCall,
            (65, 0) => MessageType::FriendRecommend,
            (10000, 0) => MessageType::SystemNotice,
            (10000, 1) => MessageType::Revoke,
            (10000, 4) => MessageType::FriendPatPat,
            (10000, 57) => MessageType::Revoke,
            (10000, 8000) => MessageType::SystemInvite,
            (x, y) => MessageType::Unknown { type_id: x, sub_id: y },
        }
    }
}

impl IntoExcelData for MessageType {
    fn write(self, worksheet: &mut Worksheet, row: RowNum, col: ColNum) -> Result<&mut Worksheet, XlsxError> {
        format!("{}", self).write(worksheet, row, col)
    }

    fn write_with_format<'a>(self, ws: &'a mut Worksheet, r: u32, c: u16, f: &Format) -> Result<&'a mut Worksheet, XlsxError> {
        ws.write_string_with_format(r, c, format!("{:?}", self), f)
    }
}
