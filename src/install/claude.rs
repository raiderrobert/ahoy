use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use crate::config;

const HOOK_MARKER: &str = "ahoy";

fn settings_path() -> PathBuf {
    // Allow test override via env var
    if let Ok(test_home) = std::env::var("AHOY_TEST_HOME") {
        return PathBuf::from(test_home).join(".claude/settings.json");
    }

    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".claude/settings.json")
}

fn ahoy_bin_path() -> String {
    config::bin_dir().join("ahoy").to_string_lossy().to_string()
}

fn create_stop_hook() -> Value {
    json!({
        "matcher": "",
        "hooks": [
            {
                "type": "command",
                "command": format!(
                    "{} send --from-claude -t 'Claude Code' --activate \"$__CFBundleIdentifier\"",
                    ahoy_bin_path()
                ),
                "timeout": 5000
            }
        ]
    })
}

fn create_notification_hooks() -> Vec<Value> {
    vec![
        json!({
            "matcher": "idle_prompt",
            "hooks": [
                {
                    "type": "command",
                    "command": format!(
                        "{} send -t 'Claude Code' 'Waiting for your input' --activate \"$__CFBundleIdentifier\"",
                        ahoy_bin_path()
                    ),
                    "timeout": 5000
                }
            ]
        }),
        json!({
            "matcher": "permission_prompt",
            "hooks": [
                {
                    "type": "command",
                    "command": format!(
                        "{} send --from-claude -t 'Claude Code' --activate \"$__CFBundleIdentifier\"",
                        ahoy_bin_path()
                    ),
                    "timeout": 5000
                }
            ]
        }),
    ]
}

pub fn install() -> Result<()> {
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

    // Add Stop hook
    stop_hooks.push(create_stop_hook());

    // Get or create Notification hooks array
    if !hooks.contains_key("Notification") {
        hooks.insert("Notification".to_string(), json!([]));
    }
    let notification_hooks = hooks.get_mut("Notification")
        .and_then(|s| s.as_array_mut())
        .context("Notification is not a JSON array")?;

    // Add Notification hooks (idle_prompt and permission_prompt)
    for hook in create_notification_hooks() {
        notification_hooks.push(hook);
    }

    // Write back settings
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_file, &content)
        .context("Failed to write Claude settings.json")?;

    println!("Installed ahoy hooks for Claude Code:");
    println!("  - Stop: notifies when Claude finishes");
    println!("  - Notification (idle_prompt): notifies when waiting for input");
    println!("  - Notification (permission_prompt): notifies when permission needed");
    println!();
    println!("Settings file: {}", settings_file.display());

    Ok(())
}

pub fn uninstall() -> Result<()> {
    let settings_file = settings_path();

    if !settings_file.exists() {
        println!("Claude settings.json not found - nothing to uninstall");
        return Ok(());
    }

    let content = fs::read_to_string(&settings_file)
        .context("Failed to read Claude settings.json")?;
    let mut settings: Value = serde_json::from_str(&content)
        .context("Failed to parse Claude settings.json")?;

    let mut removed_stop = false;
    let mut removed_notification = false;

    if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        // Remove Stop hooks
        if let Some(stop_hooks) = hooks.get_mut("Stop").and_then(|s| s.as_array_mut()) {
            let original_len = stop_hooks.len();
            stop_hooks.retain(|hook| !contains_ahoy_marker(hook));
            removed_stop = stop_hooks.len() < original_len;
        }

        // Remove Notification hooks
        if let Some(notification_hooks) = hooks.get_mut("Notification").and_then(|s| s.as_array_mut()) {
            let original_len = notification_hooks.len();
            notification_hooks.retain(|hook| !contains_ahoy_marker(hook));
            removed_notification = notification_hooks.len() < original_len;
        }
    }

    if removed_stop || removed_notification {
        let content = serde_json::to_string_pretty(&settings)?;
        fs::write(&settings_file, &content)
            .context("Failed to write Claude settings.json")?;
        println!("Removed ahoy hooks from Claude Code:");
        if removed_stop {
            println!("  - Stop hook");
        }
        if removed_notification {
            println!("  - Notification hooks");
        }
    } else {
        println!("Ahoy hooks were not installed for Claude Code");
    }

    Ok(())
}

fn contains_ahoy_marker(hook: &Value) -> bool {
    hook.get("hooks")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|cmd| cmd.contains(HOOK_MARKER))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_ahoy_marker_true() {
        let hook = json!({
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": "/path/to/ahoy send --from-claude",
                "timeout": 5000
            }]
        });

        assert!(contains_ahoy_marker(&hook));
    }

    #[test]
    fn test_contains_ahoy_marker_false() {
        let hook = json!({
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": "/usr/bin/other-command",
                "timeout": 5000
            }]
        });

        assert!(!contains_ahoy_marker(&hook));
    }

    #[test]
    fn test_contains_ahoy_marker_empty_hooks() {
        let hook = json!({
            "matcher": "",
            "hooks": []
        });

        assert!(!contains_ahoy_marker(&hook));
    }

    #[test]
    fn test_contains_ahoy_marker_no_hooks_field() {
        let hook = json!({
            "matcher": ""
        });

        assert!(!contains_ahoy_marker(&hook));
    }

    #[test]
    fn test_contains_ahoy_marker_missing_command() {
        let hook = json!({
            "matcher": "",
            "hooks": [{
                "type": "command",
                "timeout": 5000
            }]
        });

        assert!(!contains_ahoy_marker(&hook));
    }

    #[test]
    fn test_create_stop_hook_format() {
        let hook = create_stop_hook();

        // Verify structure
        assert_eq!(hook["matcher"], "");
        assert!(hook["hooks"].is_array());

        let hooks_array = hook["hooks"].as_array().unwrap();
        assert_eq!(hooks_array.len(), 1);

        let command_hook = &hooks_array[0];
        assert_eq!(command_hook["type"], "command");
        assert_eq!(command_hook["timeout"], 5000);

        // Verify command contains ahoy
        let command = command_hook["command"].as_str().unwrap();
        assert!(command.contains("ahoy"));
        assert!(command.contains("--from-claude"));
    }

    #[test]
    fn test_create_notification_hooks_count() {
        let hooks = create_notification_hooks();

        // Should create 2 notification hooks
        assert_eq!(hooks.len(), 2);

        // Check matchers
        assert_eq!(hooks[0]["matcher"], "idle_prompt");
        assert_eq!(hooks[1]["matcher"], "permission_prompt");

        // Both should have hook commands with ahoy
        for hook in hooks {
            let hooks_array = hook["hooks"].as_array().unwrap();
            assert!(hooks_array.len() > 0);

            let command = hooks_array[0]["command"].as_str().unwrap();
            assert!(command.contains("ahoy"));
        }
    }

    #[test]
    fn test_ahoy_bin_path_format() {
        let path = ahoy_bin_path();

        assert!(path.contains("ahoy"));
        assert!(path.contains(".ahoy") || path.starts_with("/"));
    }
}
