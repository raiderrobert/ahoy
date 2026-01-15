use anyhow::Result;

use crate::client::message::Notification;

pub fn show(_notification: &Notification) -> Result<()> {
    anyhow::bail!("Linux notifications not yet implemented")
}
