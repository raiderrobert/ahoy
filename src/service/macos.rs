use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::config;

const PLIST_TEMPLATE: &str = include_str!("../../resources/macos/rs.ahoy.daemon.plist");
const LABEL: &str = "rs.ahoy.daemon";

fn plist_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join("Library/LaunchAgents/rs.ahoy.daemon.plist")
}

fn get_ahoy_bin_path() -> PathBuf {
    // Use the current executable path, or fall back to ~/.ahoy/bin/ahoy
    std::env::current_exe().unwrap_or_else(|_| config::bin_dir().join("ahoy"))
}

fn render_plist() -> String {
    let user_home = dirs::home_dir()
        .expect("Could not determine home directory")
        .to_string_lossy()
        .to_string();
    let ahoy_home = config::home_dir().to_string_lossy().to_string();
    let ahoy_bin = get_ahoy_bin_path().to_string_lossy().to_string();

    PLIST_TEMPLATE
        .replace("{{USER_HOME}}", &user_home)
        .replace("{{AHOY_HOME}}", &ahoy_home)
        .replace("{{AHOY_BIN}}", &ahoy_bin)
}

pub async fn install() -> Result<()> {
    let plist = plist_path();

    // Ensure LaunchAgents directory exists
    if let Some(parent) = plist.parent() {
        fs::create_dir_all(parent)?;
    }

    // Stop existing service if running
    let _ = stop().await;

    // Write the plist file
    let content = render_plist();
    fs::write(&plist, &content)?;

    println!("Installed launchd service: {}", plist.display());
    println!("Plist contents:");
    println!("{}", content);

    // Load the service
    start().await?;

    println!();
    println!("Service installed and started successfully!");
    println!("The daemon will now auto-start on login.");

    Ok(())
}

pub async fn uninstall() -> Result<()> {
    let plist = plist_path();

    // Stop the service first
    let _ = stop().await;

    // Remove the plist file
    if plist.exists() {
        fs::remove_file(&plist)?;
        println!("Removed launchd service: {}", plist.display());
    } else {
        println!("Service not installed (plist not found)");
    }

    Ok(())
}

pub async fn start() -> Result<()> {
    let plist = plist_path();

    if !plist.exists() {
        anyhow::bail!(
            "Service not installed. Run 'ahoy service install' first."
        );
    }

    let output = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist)
        .output()?;

    if output.status.success() {
        println!("Service started");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already loaded") || stderr.contains("Load failed: 37") {
            println!("Service is already running");
            Ok(())
        } else {
            anyhow::bail!("Failed to start service: {}", stderr)
        }
    }
}

pub async fn stop() -> Result<()> {
    let plist = plist_path();

    if !plist.exists() {
        println!("Service not installed");
        return Ok(());
    }

    let output = Command::new("launchctl")
        .args(["unload"])
        .arg(&plist)
        .output()?;

    if output.status.success() {
        println!("Service stopped");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Could not find specified service") {
            println!("Service was not running");
        } else {
            println!("Stop result: {}", stderr);
        }
    }

    Ok(())
}

pub async fn status() -> Result<()> {
    let plist = plist_path();

    println!("Service: {}", LABEL);
    println!("Plist: {}", plist.display());
    println!();

    if !plist.exists() {
        println!("Status: NOT INSTALLED");
        println!();
        println!("Run 'ahoy service install' to install the service.");
        return Ok(());
    }

    // Check if service is loaded
    let output = Command::new("launchctl")
        .args(["list", LABEL])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Status: RUNNING");
        println!();

        // Parse PID from output
        for line in stdout.lines() {
            if line.contains("PID") || line.starts_with('"') || !line.trim().is_empty() {
                println!("{}", line);
            }
        }
    } else {
        println!("Status: STOPPED (installed but not running)");
        println!();
        println!("Run 'ahoy service start' to start the service.");
    }

    Ok(())
}
