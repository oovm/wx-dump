use std::{
    env::{current_dir, set_current_dir},
    path::{Path, PathBuf},
};
use wx_dump::{RunExport, WxArguments, WxDump};

#[test]
fn ready() {
    println!("it works!")
}

#[ignore]
#[tokio::test]
pub async fn test_export() -> anyhow::Result<()> {
    set_workspace_dir()?;
    tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();
    let run = RunExport { path: None };
    run.run(WxArguments::default()).await
}

fn set_workspace_dir() -> std::io::Result<()> {
    let package_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    match package_dir.parent().and_then(|s| s.parent()) {
        Some(s) => set_current_dir(s),
        None => Ok(()),
    }
}
