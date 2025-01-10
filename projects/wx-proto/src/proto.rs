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
    pub fn pop_image_path(&mut self) -> Option<String> {
        // self.pop_thumb();
        let index = self.string.iter().position(|s| s.r#type == 4)?;
        let item = self.string.remove(index);
        Some(item.message)
    }
    // pub fn pop_thumb(&mut self) -> Option<String> {
    //     let index = self.string.iter().position(|s| s.r#type == 3)?;
    //     let item = self.string.remove(index);
    //     Some(item.message)
    // }
}
