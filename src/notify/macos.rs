use anyhow::Result;
use std::process::Command;
use tracing::info;

use crate::client::message::Notification;

pub fn show(notification: &Notification) -> Result<()> {
    info!("Attempting to show macOS notification via ahoy-notify...");

    // Use our Swift helper binary for native macOS notifications
    // The helper is installed alongside the main binary
    let ahoy_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".ahoy")
        .join("bin")
        .join("ahoy-notify");

    let output = Command::new(&ahoy_dir)
        .arg(&notification.title)
        .arg(&notification.body)
        .arg("--sound")
        .arg("Glass")
        .output()?;

    if output.status.success() {
        info!("Notification shown successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        info!("Notification error: {}", stderr);
        anyhow::bail!("Failed to show notification: {}", stderr)
    }
}
