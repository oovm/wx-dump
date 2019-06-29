use clap::Parser;
use wx_dump::WxDump;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();
    WxDump::parse().run().await
}
