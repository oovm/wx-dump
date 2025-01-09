use crate::orm_types::MessageData;

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
    PatFriend,
    /// 系统通知
    ///
    /// 居中出现的那种灰色文字
    SystemNotice,
    /// 邀请通知
    ///
    /// 特别包含你邀请别人加入群聊
    SystemInvite,
    /// 未知类型
    Unknown {
        /// 类别 id
        type_id: i32,
        /// 子类 id
        sub_id: i32,
    },
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
            (50, 0) => MessageType::PhoneCall,
            (10000, 0) => MessageType::SystemNotice,
            (10000, 4) => MessageType::PatFriend,
            (10000, 8000) => MessageType::SystemInvite,
            (x, y) => MessageType::Unknown { type_id: x, sub_id: y },
        }
    }
}
