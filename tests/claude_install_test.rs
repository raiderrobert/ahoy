use ahoy::install::claude;
use serde_json::{json, Value};
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

// Helper to set up a test home directory
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    unsafe {
        std::env::set_var("AHOY_TEST_HOME", temp_dir.path());
    }
    temp_dir
}

// Helper to write settings.json
fn write_settings(temp_dir: &TempDir, content: Value) {
    let claude_dir = temp_dir.path().join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();
    let settings_path = claude_dir.join("settings.json");
    fs::write(&settings_path, serde_json::to_string_pretty(&content).unwrap()).unwrap();
}

// Helper to read settings.json
fn read_settings(temp_dir: &TempDir) -> Value {
    let settings_path = temp_dir.path().join(".claude/settings.json");
    let content = fs::read_to_string(&settings_path).unwrap();
    serde_json::from_str(&content).unwrap()
}

#[tokio::test]
#[serial]
async fn test_install_creates_settings_file() {
    let temp_dir = setup_test_env();

    claude::install().await.unwrap();

    assert!(temp_dir.path().join(".claude/settings.json").exists());
}

#[tokio::test]
#[serial]
async fn test_install_to_empty_settings() {
    let temp_dir = setup_test_env();
    write_settings(&temp_dir, json!({}));

    claude::install().await.unwrap();

    let settings = read_settings(&temp_dir);
    assert!(settings["hooks"].is_object());
    assert!(settings["hooks"]["Stop"].is_array());
    assert!(settings["hooks"]["Notification"].is_array());
}

#[tokio::test]
#[serial]
async fn test_install_adds_stop_hook() {
    let temp_dir = setup_test_env();
    write_settings(&temp_dir, json!({}));

    claude::install().await.unwrap();

    let settings = read_settings(&temp_dir);
    let stop_hooks = settings["hooks"]["Stop"].as_array().unwrap();

    assert_eq!(stop_hooks.len(), 1);

    let hook_command = stop_hooks[0]["hooks"][0]["command"].as_str().unwrap();
    assert!(hook_command.contains("ahoy"));
    assert!(hook_command.contains("--from-claude"));
}

#[tokio::test]
#[serial]
async fn test_install_adds_notification_hooks() {
    let temp_dir = setup_test_env();
    write_settings(&temp_dir, json!({}));

    claude::install().await.unwrap();

    let settings = read_settings(&temp_dir);
    let notification_hooks = settings["hooks"]["Notification"].as_array().unwrap();

    // Should add 2 notification hooks (idle_prompt and permission_prompt)
    assert_eq!(notification_hooks.len(), 2);

    assert_eq!(notification_hooks[0]["matcher"], "idle_prompt");
    assert_eq!(notification_hooks[1]["matcher"], "permission_prompt");
}

#[tokio::test]
#[serial]
async fn test_install_idempotent() {
    let temp_dir = setup_test_env();
    write_settings(&temp_dir, json!({}));

    // Install twice
    claude::install().await.unwrap();
    claude::install().await.unwrap();

    let settings = read_settings(&temp_dir);
    let stop_hooks = settings["hooks"]["Stop"].as_array().unwrap();

    // Should still only have 1 stop hook
    assert_eq!(stop_hooks.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_is_installed_true_after_install() {
    let temp_dir = setup_test_env();
    write_settings(&temp_dir, json!({}));

    assert!(!claude::is_installed());

    claude::install().await.unwrap();

    assert!(claude::is_installed());
}

#[tokio::test]
#[serial]
async fn test_is_installed_false_no_file() {
    let _temp_dir = setup_test_env();
    // No settings file created

    assert!(!claude::is_installed());
}

#[tokio::test]
#[serial]
async fn test_is_installed_false_empty_settings() {
    let temp_dir = setup_test_env();
    write_settings(&temp_dir, json!({}));

    assert!(!claude::is_installed());
}

#[tokio::test]
#[serial]
async fn test_uninstall_removes_hooks() {
    let temp_dir = setup_test_env();
    write_settings(&temp_dir, json!({}));

    claude::install().await.unwrap();
    assert!(claude::is_installed());

    claude::uninstall().await.unwrap();

    let settings = read_settings(&temp_dir);
    let stop_hooks = settings["hooks"]["Stop"].as_array().unwrap();

    assert_eq!(stop_hooks.len(), 0);
    assert!(!claude::is_installed());
}

#[tokio::test]
#[serial]
async fn test_uninstall_removes_notification_hooks() {
    let temp_dir = setup_test_env();
    write_settings(&temp_dir, json!({}));

    claude::install().await.unwrap();
    claude::uninstall().await.unwrap();

    let settings = read_settings(&temp_dir);
    let notification_hooks = settings["hooks"]["Notification"].as_array().unwrap();

    assert_eq!(notification_hooks.len(), 0);
}

#[tokio::test]
#[serial]
async fn test_uninstall_no_settings_file() {
    let _temp_dir = setup_test_env();
    // No settings file

    // Should not error
    claude::uninstall().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_uninstall_preserves_other_hooks() {
    let temp_dir = setup_test_env();

    // Create settings with ahoy hooks AND other hooks
    write_settings(&temp_dir, json!({
        "hooks": {
            "Stop": [
                {
                    "matcher": "",
                    "hooks": [{
                        "type": "command",
                        "command": "/other/tool --flag",
                        "timeout": 5000
                    }]
                }
            ]
        }
    }));

    claude::install().await.unwrap();

    // Should now have 2 Stop hooks (other + ahoy)
    let settings = read_settings(&temp_dir);
    assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 2);

    claude::uninstall().await.unwrap();

    // Should only have 1 Stop hook left (other tool)
    let settings = read_settings(&temp_dir);
    let stop_hooks = settings["hooks"]["Stop"].as_array().unwrap();
    assert_eq!(stop_hooks.len(), 1);

    let remaining_command = stop_hooks[0]["hooks"][0]["command"].as_str().unwrap();
    assert!(remaining_command.contains("/other/tool"));
    assert!(!remaining_command.contains("ahoy"));
}

#[tokio::test]
#[serial]
async fn test_install_creates_parent_directory() {
    let temp_dir = setup_test_env();
    // Don't create .claude directory

    claude::install().await.unwrap();

    assert!(temp_dir.path().join(".claude").exists());
    assert!(temp_dir.path().join(".claude/settings.json").exists());
}
