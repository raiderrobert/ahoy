use anyhow::Result;
use std::process::Command;
use tracing::info;

use crate::client::message::Notification;

pub fn show(notification: &Notification) -> Result<()> {
    info!("Attempting to show macOS notification via osascript...");

    // Use osascript to display notification - this is the most reliable approach
    // and works in all contexts (daemon, CLI, etc.)
    let script = format!(
        r#"display notification "{}" with title "{}" sound name "Glass""#,
        escape_applescript(&notification.body),
        escape_applescript(&notification.title)
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
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

fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
