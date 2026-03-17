use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tracing::{error, info};

use crate::config::Config;
use crate::registry::SessionRegistry;
use crate::session::handle_connection;

pub async fn run(config: Config) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;
    let registry = Arc::new(SessionRegistry::new());
    let config = Arc::new(config);

    info!(addr = %addr, "WebRTT edge node listening");

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                info!(peer = %peer_addr, "new connection");
                let registry = Arc::clone(&registry);
                let config = Arc::clone(&config);

                tokio::spawn(async move {
                    match accept_async(stream).await {
                        Ok(ws_stream) => {
                            if let Err(e) =
                                handle_connection(ws_stream, registry, config, peer_addr).await
                            {
                                error!(peer = %peer_addr, error = %e, "connection error");
                            }
                        }
                        Err(e) => {
                            error!(peer = %peer_addr, error = %e, "websocket handshake failed")
                        }
                    }
                });
            }
            Err(e) => error!(error = %e, "accept failed"),
        }
    }
}
