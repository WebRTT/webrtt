mod config;
mod pipeline;
mod registry;
mod server;
mod session;
mod speculation;

use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_env("RUST_LOG")
                .add_directive("webrtt_edge=debug".parse()?),
        )
        .json()
        .init();

    let config = config::Config::from_env()?;
    info!(port = config.port, "WebRTT edge node starting");

    server::run(config).await
}
