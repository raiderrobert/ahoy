mod server;

use anyhow::Result;
use tracing::info;

use crate::config;

pub async fn run() -> Result<()> {
    config::ensure_home_dir()?;

    let socket_path = config::socket_path();
    info!("Starting ahoy daemon on {:?}", socket_path);

    server::run(&socket_path).await
}
