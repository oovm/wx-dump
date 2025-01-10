use crate::{WxError, WxResult};
use regex::Regex;
use rusqlite::{
    Connection, Error, Params, Result, Row, ToSql,
    functions::{Context, FunctionFlags, SqlFnOutput},
};
use wx_proto::{Message, proto::MsgBytesExtra};
pub struct SqliteHelper {
    db: Connection,
}

#[derive(Debug)]
pub struct Sender {
    SenderId: String,
}

impl SqliteHelper {
    pub fn open(jdbc: &str) -> WxResult<Self> {
        let db = Connection::open(jdbc)?;
        add_builtin(&db)?;
        Ok(Self { db })
    }
    pub fn query<T>(
        &self,
        query: &str,
        params: &[&dyn rusqlite::ToSql],
        f: impl FnOnce(&rusqlite::Row) -> Result<T>,
    ) -> WxResult<T> {
        Ok(self.db.query_row(query, params, f)?)
    }
    pub fn query_as<T>(&self, query: &str, args: impl Params) -> Result<T>
    where
        T: for<'a, 't> TryFrom<&'a Row<'t>, Error = rusqlite::Error>,
    {
        self.db.query_row(query, args, |row: &Row| T::try_from(row))
    }
}

fn add_builtin(db: &Connection) -> Result<()> {
    db.create_scalar_function(
        "get_sender_id",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| get_sender_id(ctx).map_err(|e| Error::UserFunctionError(e.kind)),
    )?;
    db.create_scalar_function("get_user_id2", 1, FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC, move |ctx| {
        let text = ctx.get_raw(1).as_blob().map_err(|e| Error::UserFunctionError(e.into()))?;
        Ok(true)
    })
}

fn get_sender_id(ctx: &Context) -> WxResult<impl SqlFnOutput> {
    let text = ctx.get_raw(0).as_blob()?;
    let data = MsgBytesExtra::decode(text)?;
    Ok(data.get_sender_id().to_string())
}

impl TryFrom<&Row<'_>> for Sender {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> std::result::Result<Self, Self::Error> {
        Ok(Self { SenderId: value.get("SenderId").unwrap_or_default() })
    }
}

#[test]
fn main() -> WxResult<()> {
    let db = SqliteHelper::open(r#"E:/RustroverProjects/wx_dump_rs/target/wxid_2mq6n5f8ovvf22/Multi/MSG0.db"#)?;

    let is_match: Sender = db.query_as(
        "SELECT get_sender_id(BytesExtra) AS SenderId FROM MSG;",
        [],
    )?;

    println!("{:#?}", is_match);
    Ok(())
}
