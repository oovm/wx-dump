use crate::WxResult;
use rusqlite::{
    Error, Result,
    functions::{Context, FunctionFlags},
};
use sqlx::{Connection, SqliteConnection};
use wx_proto::{Message, proto::MsgBytesExtra};
pub struct SqliteHelper {
   pub(crate) db: SqliteConnection,
}

impl SqliteHelper {
    pub async fn open(jdbc: &str) -> WxResult<Self> {
        let mut connect = SqliteConnection::connect(jdbc).await?;
        {
            let mut handle_lock = connect.lock_handle().await?;
            let handle = handle_lock.as_raw_handle().as_ptr();
            let rc = unsafe { rusqlite::Connection::from_handle(handle) }?;
            add_builtin(&rc)?;
        }
        Ok(Self { db: connect })
    }
}

fn add_builtin(db: &rusqlite::Connection) -> Result<()> {
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
