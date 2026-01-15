use anyhow::Result;

use crate::client::message::Notification;

pub fn show(_notification: &Notification) -> Result<()> {
    anyhow::bail!("Windows notifications not yet implemented")
}
