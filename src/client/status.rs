use anyhow::Result;
use tokio::net::UnixStream;

use crate::config;

pub async fn run() -> Result<()> {
    let socket_path = config::socket_path();

    if !socket_path.exists() {
        println!("Daemon: not running (socket not found)");
        println!("Socket: {:?}", socket_path);
        return Ok(());
    }

    // Try to connect to verify daemon is actually running
    match UnixStream::connect(&socket_path).await {
        Ok(_) => {
            println!("Daemon: running");
            println!("Socket: {:?}", socket_path);
        }
        Err(_) => {
            println!("Daemon: not running (socket exists but unresponsive)");
            println!("Socket: {:?}", socket_path);
            println!("Hint: Remove stale socket with: rm {:?}", socket_path);
        }
    }

    Ok(())
}
