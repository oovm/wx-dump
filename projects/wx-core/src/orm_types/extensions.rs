use crate::{WxError, WxResult};
use rusqlite::{
    Connection, Error, Params, Result, Row,
    functions::{Context, FunctionFlags},
};
use std::{ops::Coroutine};
use wx_proto::{Message, proto::MsgBytesExtra};
pub struct SqliteHelper {
    db: Connection,
}

impl SqliteHelper {
    pub fn open(jdbc: &str) -> WxResult<Self> {
        let db = Connection::open(jdbc)?;
        add_builtin(&db)?;
        Ok(Self { db })
    }
    pub fn query_as<T>(&self, sql: &str, args: impl Params) -> impl Coroutine<Yield = T, Return = WxResult<()>>
    where
        T: for<'a, 't> TryFrom<&'a Row<'t>, Error = WxError>,
    {
        #[coroutine]
        static || {
            let mut s = self.db.prepare(sql)?;
            let mut m = s.query(args)?;
            while let Some(row) = m.next()? {
                yield T::try_from(row)?
            }
            Ok(())
        }
    }
}

fn add_builtin(db: &Connection) -> Result<()> {
    db.create_scalar_function(
        "get_sender_id",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        get_sender_id,
    )?;
    db.create_scalar_function("get_user_id2", 1, FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC, move |ctx| {
        let text = ctx.get_raw(1).as_blob().map_err(|e| Error::UserFunctionError(e.into()))?;
        Ok(true)
    })
}

fn get_sender_id(ctx: &Context) -> Result<String, Error> {
    let text = ctx.get_raw(0).as_blob()?;
    match MsgBytesExtra::decode(text) {
        Ok(o) => Ok(o.get_sender_id().to_string()),
        Err(e) => Err(Error::UserFunctionError(Box::new(e))),
    }
}
