mod macos;

use crate::client::message::Notification;
use anyhow::Result;

/// Show a native macOS notification
pub fn show(notification: &Notification) -> Result<()> {
    macos::show(notification)
}
