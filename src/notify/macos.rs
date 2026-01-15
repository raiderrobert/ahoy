use anyhow::Result;
use std::process::Command;
use tracing::info;

use crate::client::message::Notification;

pub fn show(notification: &Notification) -> Result<()> {
    info!("Attempting to show macOS notification via ahoy-notify...");

    // Use our Swift helper binary for native macOS notifications
    // The helper is inside the Ahoy.app bundle for proper icon display
    let ahoy_notify = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".ahoy")
        .join("Ahoy.app")
        .join("Contents")
        .join("MacOS")
        .join("ahoy-notify");

    let mut cmd = Command::new(&ahoy_notify);
    cmd.arg(&notification.title)
        .arg(&notification.body)
        .arg("--sound")
        .arg("Glass");

    // Pass activate bundle ID if provided
    if let Some(ref bundle_id) = notification.activate {
        cmd.arg("--activate").arg(bundle_id);
    }

    let output = cmd.output()?;

    if output.status.success() {
        info!("Notification shown successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        info!("Notification error: {}", stderr);
        anyhow::bail!("Failed to show notification: {}", stderr)
    }
}
