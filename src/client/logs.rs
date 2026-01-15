use std::process::Command;

use anyhow::Result;

use crate::config;

pub async fn run(lines: usize, follow: bool) -> Result<()> {
    let log_path = config::log_path();

    if !log_path.exists() {
        println!("No log file found at {:?}", log_path);
        return Ok(());
    }

    let mut cmd = if follow {
        let mut c = Command::new("tail");
        c.arg("-f").arg("-n").arg(lines.to_string()).arg(&log_path);
        c
    } else {
        let mut c = Command::new("tail");
        c.arg("-n").arg(lines.to_string()).arg(&log_path);
        c
    };

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("tail command failed");
    }

    Ok(())
}
