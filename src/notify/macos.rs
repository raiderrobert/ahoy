use anyhow::Result;
use std::process::Command;
use tracing::info;

use crate::client::message::Notification;

pub fn show(notification: &Notification) -> Result<()> {
    info!("Attempting to show macOS notification via terminal-notifier...");

    // Use terminal-notifier - it's a signed app that shows proper banner notifications
    // Must use full path since launchd doesn't have homebrew in PATH
    let output = Command::new("/opt/homebrew/bin/terminal-notifier")
        .arg("-title")
        .arg(&notification.title)
        .arg("-message")
        .arg(&notification.body)
        .arg("-sound")
        .arg("Glass")
        .arg("-ignoreDnD") // Show even in Do Not Disturb
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
