use std::path::PathBuf;

/// 导出微信数据库中的数据
#[derive(Debug)]
pub struct WxExport {
    /// 数据库所在文件路径
    pub db: PathBuf,
}
