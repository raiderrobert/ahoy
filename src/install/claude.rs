use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::PathBuf;

use crate::config;

const HOOK_MARKER: &str = "ahoy";

fn settings_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".claude/settings.json")
}

fn ahoy_bin_path() -> String {
    config::bin_dir().join("ahoy").to_string_lossy().to_string()
}

fn create_ahoy_hook() -> Value {
    json!({
        "matcher": "",
        "hooks": [
            {
                "type": "command",
                "command": format!(
                    "{} send \"Task finished in $(basename $PWD)\" -t 'Claude Code'",
                    ahoy_bin_path()
                ),
                "timeout": 5000
            }
        ]
    })
}

pub async fn install() -> Result<()> {
    let settings_file = settings_path();

    // Read existing settings or create empty object
    let mut settings: Value = if settings_file.exists() {
        let content = fs::read_to_string(&settings_file)
            .context("Failed to read Claude settings.json")?;
        serde_json::from_str(&content)
            .context("Failed to parse Claude settings.json")?
    } else {
        // Create parent directory if needed
        if let Some(parent) = settings_file.parent() {
            fs::create_dir_all(parent)?;
        }
        json!({})
    };

    // Ensure settings is an object
    let settings_obj = settings.as_object_mut()
        .context("Claude settings.json is not a JSON object")?;

    // Get or create hooks section
    if !settings_obj.contains_key("hooks") {
        settings_obj.insert("hooks".to_string(), json!({}));
    }
    let hooks = settings_obj.get_mut("hooks")
        .and_then(|h| h.as_object_mut())
        .context("hooks is not a JSON object")?;

    // Get or create Stop hooks array
    if !hooks.contains_key("Stop") {
        hooks.insert("Stop".to_string(), json!([]));
    }
    let stop_hooks = hooks.get_mut("Stop")
        .and_then(|s| s.as_array_mut())
        .context("Stop is not a JSON array")?;

    // Check if ahoy hook already exists
    let already_installed = stop_hooks.iter().any(|hook| {
        hook.get("hooks")
            .and_then(|h| h.as_array())
            .map(|arr| arr.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|cmd| cmd.contains(HOOK_MARKER))
                    .unwrap_or(false)
            }))
            .unwrap_or(false)
    });

    if already_installed {
        println!("Ahoy hook is already installed for Claude Code");
        return Ok(());
    }

    // Add ahoy hook
    stop_hooks.push(create_ahoy_hook());

    // Write back settings
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_file, &content)
        .context("Failed to write Claude settings.json")?;

    println!("Installed ahoy hook for Claude Code");
    println!("Settings file: {}", settings_file.display());
    println!();
    println!("Claude Code will now notify you when tasks finish!");

    Ok(())
}

pub async fn uninstall() -> Result<()> {
    let settings_file = settings_path();

    if !settings_file.exists() {
        println!("Claude settings.json not found - nothing to uninstall");
        return Ok(());
    }

    let content = fs::read_to_string(&settings_file)
        .context("Failed to read Claude settings.json")?;
    let mut settings: Value = serde_json::from_str(&content)
        .context("Failed to parse Claude settings.json")?;

    // Navigate to Stop hooks
    let removed = if let Some(hooks) = settings.get_mut("hooks")
        .and_then(|h| h.as_object_mut())
    {
        if let Some(stop_hooks) = hooks.get_mut("Stop")
            .and_then(|s| s.as_array_mut())
        {
            let original_len = stop_hooks.len();
            stop_hooks.retain(|hook| {
                !hook.get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|arr| arr.iter().any(|h| {
                        h.get("command")
                            .and_then(|c| c.as_str())
                            .map(|cmd| cmd.contains(HOOK_MARKER))
                            .unwrap_or(false)
                    }))
                    .unwrap_or(false)
            });
            stop_hooks.len() < original_len
        } else {
            false
        }
    } else {
        false
    };

    if removed {
        let content = serde_json::to_string_pretty(&settings)?;
        fs::write(&settings_file, &content)
            .context("Failed to write Claude settings.json")?;
        println!("Removed ahoy hook from Claude Code");
    } else {
        println!("Ahoy hook was not installed for Claude Code");
    }

    Ok(())
}

pub fn is_installed() -> bool {
    let settings_file = settings_path();

    if !settings_file.exists() {
        return false;
    }

    let Ok(content) = fs::read_to_string(&settings_file) else {
        return false;
    };

    let Ok(settings) = serde_json::from_str::<Value>(&content) else {
        return false;
    };

    settings.get("hooks")
        .and_then(|h| h.get("Stop"))
        .and_then(|s| s.as_array())
        .map(|arr| arr.iter().any(|hook| {
            hook.get("hooks")
                .and_then(|h| h.as_array())
                .map(|arr| arr.iter().any(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .map(|cmd| cmd.contains(HOOK_MARKER))
                        .unwrap_or(false)
                }))
                .unwrap_or(false)
        }))
        .unwrap_or(false)
}
