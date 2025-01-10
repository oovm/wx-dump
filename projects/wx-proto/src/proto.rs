use prost::Message;
use sqlx::{Database, Decode, Sqlite, Type, error::BoxDynError, sqlite::SqliteTypeInfo};

include!(concat!(env!("OUT_DIR"), "/wx.core.rs"));

impl Type<Sqlite> for MsgBytesExtra {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&[u8] as Type<Sqlite>>::compatible(ty)
    }
}

impl<'c> Decode<'c, Sqlite> for MsgBytesExtra {
    fn decode(value: <Sqlite as Database>::ValueRef<'c>) -> Result<Self, BoxDynError> {
        let blob = <&[u8] as Decode<'c, Sqlite>>::decode(value)?;
        Ok(<MsgBytesExtra as Message>::decode(blob)?)
    }
}

impl MsgBytesExtra {
    pub fn pop_sender(&mut self) -> Option<String> {
        let index = self.string.iter().position(|s| s.r#type == 1)?;
        let item = self.string.remove(index);
        Some(item.message)
    }
    pub fn get_sender_id(&self) -> &str {
        self.string.iter().find(|pat| pat.r#type == 1).map(|pat| pat.message.as_str()).unwrap_or_default()
    }
    pub fn get_image_path(&self) -> &str {
        self.string.iter().find(|pat| pat.r#type == 4).map(|pat| pat.message.as_str()).unwrap_or_default()
    }
}
