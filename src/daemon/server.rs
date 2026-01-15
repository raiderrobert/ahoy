use std::path::Path;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixListener;
use tracing::{error, info};

use crate::client::message::Notification;
use crate::notify;

pub async fn run(socket_path: &Path) -> Result<()> {
    // Remove existing socket if present
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    info!("Listening on {:?}", socket_path);

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream).await {
                        error!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn handle_connection(stream: tokio::net::UnixStream) -> Result<()> {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        match serde_json::from_str::<Notification>(&line) {
            Ok(notification) => {
                info!("Received notification: {:?}", notification);
                if let Err(e) = notify::show(&notification) {
                    error!("Failed to show notification: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to parse notification: {}", e);
            }
        }
    }

    Ok(())
}
