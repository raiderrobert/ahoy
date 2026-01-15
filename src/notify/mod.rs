#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

use crate::client::message::Notification;
use anyhow::Result;

/// Show a native OS notification
pub fn show(notification: &Notification) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::show(notification)
    }

    #[cfg(target_os = "linux")]
    {
        linux::show(notification)
    }

    #[cfg(target_os = "windows")]
    {
        windows::show(notification)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        anyhow::bail!("Notifications not supported on this platform")
    }
}
