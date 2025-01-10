#![doc = "辅助函数"]
pub use self::lazy_xml::LazyXML;
use crate::{WxError, WxResult};
use std::{
    collections::BTreeMap,
    fs::read_dir,
    path::{Path, PathBuf},
};
use url::Url;

mod lazy_xml;



/// 获取微信主目录
pub fn get_wechat_path(given: &Option<String>) -> WxResult<PathBuf> {
    let path = match given {
        Some(wechat_path) => PathBuf::from(wechat_path),
        None => match dirs::document_dir() {
            Some(s) => s.join("WeChat Files"),
            None => Err(WxError::custom("fail to get document directory"))?,
        },
    };
    if path.exists() {
        if !path.is_dir() {
            Err(WxError::custom("指定的微信主目录不是文件夹，请检查。"))?;
        }
    }
    else {
        Err(WxError::custom(format!("指定的微信主目录不存在，请检查。{:?}", path.display())))?;
    }
    Ok(path)
}
/// 读取数据库
pub async fn read_database(wechat_path: &Option<String>) -> WxResult<BTreeMap<String, PathBuf>> {
    let mut map = BTreeMap::new();
    let wechat_path_buf = get_wechat_path(wechat_path)?;

    for entity in read_dir(wechat_path_buf)? {
        let entity = entity?;
        if entity.file_name() == "All Users" || entity.file_name() == "Applet" || entity.file_name() == "WMPF" {
            continue;
        }
        if entity.file_type()?.is_dir() {
            match entity.file_name().into_string() {
                Ok(o) => {
                    map.insert(o, entity.path().join("Msg"));
                }
                Err(_) => {}
            }
        }
    }
    Ok(map)
}
/// 显示 url 路径
pub fn url_display(path: &Path, fmt: fn(Url)) {
    match Url::from_file_path(path) {
        Ok(o) => fmt(o),
        Err(_) => {}
    }
}
