use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use tracing::info;

use crate::client::message::Notification;
use crate::notify;

/// Claude Code hook stdin data
#[derive(Deserialize)]
struct ClaudeHookData {
    transcript_path: Option<String>,
    cwd: Option<String>,
    #[allow(dead_code)]
    session_id: Option<String>,
    tool_name: Option<String>,
    tool_input: Option<serde_json::Value>,
    #[allow(dead_code)]
    hook_event_name: Option<String>,
}

/// A line from the Claude transcript
#[derive(Deserialize)]
struct TranscriptLine {
    #[serde(rename = "type")]
    line_type: Option<String>,
    message: Option<TranscriptMessage>,
}

#[derive(Deserialize)]
struct TranscriptMessage {
    content: Option<serde_json::Value>,
}

pub fn run(
    message: Option<String>,
    title: String,
    json: Option<String>,
    from_claude: bool,
    activate: Option<String>,
) -> Result<()> {
    let mut notification = if from_claude {
        build_from_claude_stdin(&title)?
    } else if let Some(json_str) = json {
        serde_json::from_str(&json_str)?
    } else if let Some(body) = message {
        Notification::new(title, body)
    } else {
        bail!("Either a message or --json must be provided");
    };

    // Apply activate if provided (overrides any value from JSON/stdin)
    if let Some(bundle_id) = activate {
        notification.activate = Some(bundle_id);
    }

    send_notification(&notification)
}

fn build_from_claude_stdin(title: &str) -> Result<Notification> {
    build_from_claude_stdin_reader(io::stdin(), title)
}

// Internal function for testing - accepts any reader
fn build_from_claude_stdin_reader(mut reader: impl Read, title: &str) -> Result<Notification> {
    let mut stdin_data = String::new();
    reader.read_to_string(&mut stdin_data)?;

    if stdin_data.is_empty() {
        return Ok(Notification::new(
            title.to_string(),
            "Task finished".to_string(),
        ));
    }

    let hook_data: ClaudeHookData =
        serde_json::from_str(&stdin_data).context("Failed to parse Claude hook data from stdin")?;

    let project_name = hook_data
        .cwd
        .as_ref()
        .and_then(|cwd| cwd.split('/').next_back())
        .unwrap_or("project");

    if let Some(tool_name) = &hook_data.tool_name {
        let tool_desc = if let Some(input) = &hook_data.tool_input {
            // Try to get command for Bash, or file_path for Read/Write/Edit
            input
                .get("command")
                .or_else(|| input.get("file_path"))
                .or_else(|| input.get("pattern"))
                .and_then(|v| v.as_str())
                .map(|s| {
                    if s.len() > 60 {
                        format!("{}...", &s[..57])
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or_default()
        } else {
            String::new()
        };

        let body = if tool_desc.is_empty() {
            format!("[{}] Needs permission: {}", project_name, tool_name)
        } else {
            format!("[{}] {}: {}", project_name, tool_name, tool_desc)
        };

        return Ok(Notification::new(title.to_string(), body));
    }

    let last_prompt = if let Some(transcript_path) = &hook_data.transcript_path {
        extract_last_prompt(transcript_path).unwrap_or_else(|_| "Task finished".to_string())
    } else {
        "Task finished".to_string()
    };

    // Truncate prompt if too long (max 100 chars for notification)
    let truncated_prompt = if last_prompt.len() > 100 {
        format!("{}...", &last_prompt[..97])
    } else {
        last_prompt
    };

    let body = format!("[{}] {}", project_name, truncated_prompt);

    Ok(Notification::new(title.to_string(), body))
}

fn extract_last_prompt(transcript_path: &str) -> Result<String> {
    let file = File::open(transcript_path)?;
    let reader = BufReader::new(file);

    let mut last_user_content: Option<String> = None;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<TranscriptLine>(&line)
            && entry.line_type.as_deref() == Some("user")
                && let Some(msg) = entry.message
                    && let Some(content) = msg.content {
                        // Content can be a string or array
                        let text = match content {
                            serde_json::Value::String(s) => s,
                            serde_json::Value::Array(arr) => arr
                                .iter()
                                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                                .collect::<Vec<_>>()
                                .join(" "),
                            _ => continue,
                        };

                        let cleaned = text.lines().next().unwrap_or(&text).trim().to_string();

                        if !cleaned.is_empty() {
                            last_user_content = Some(cleaned);
                        }
                    }
    }

    last_user_content.ok_or_else(|| anyhow::anyhow!("No user message found in transcript"))
}

fn send_notification(notification: &Notification) -> Result<()> {
    info!("Showing notification: {:?}", notification);
    notify::show(notification)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_last_prompt_simple_string() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"user","message":{{"content":"Fix the bug"}}}}"#
        )
        .unwrap();

        let result = extract_last_prompt(file.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "Fix the bug");
    }

    #[test]
    fn test_extract_last_prompt_multiple_messages_returns_last() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"user","message":{{"content":"First message"}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"content":"Response"}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type":"user","message":{{"content":"Second message"}}}}"#
        )
        .unwrap();

        let result = extract_last_prompt(file.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "Second message");
    }

    #[test]
    fn test_extract_last_prompt_array_content() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"type":"user","message":{{"content":[{{"text":"First part"}},{{"text":"Second part"}}]}}}}"#).unwrap();

        let result = extract_last_prompt(file.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "First part Second part");
    }

    #[test]
    fn test_extract_last_prompt_multiline_takes_first_line() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"user","message":{{"content":"First line\nSecond line\nThird line"}}}}"#
        )
        .unwrap();

        let result = extract_last_prompt(file.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "First line");
    }

    #[test]
    fn test_extract_last_prompt_empty_file() {
        let file = NamedTempFile::new().unwrap();

        let result = extract_last_prompt(file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No user message found")
        );
    }

    #[test]
    fn test_extract_last_prompt_no_user_messages() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"content":"Only assistant"}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type":"system","message":{{"content":"Only system"}}}}"#
        )
        .unwrap();

        let result = extract_last_prompt(file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No user message found")
        );
    }

    #[test]
    fn test_extract_last_prompt_invalid_json_skipped() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid json line").unwrap();
        writeln!(
            file,
            r#"{{"type":"user","message":{{"content":"Valid message"}}}}"#
        )
        .unwrap();
        writeln!(file, "another invalid line").unwrap();

        let result = extract_last_prompt(file.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "Valid message");
    }

    #[test]
    fn test_extract_last_prompt_whitespace_only_skipped() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"type":"user","message":{{"content":"   "}}}}"#).unwrap();
        writeln!(
            file,
            r#"{{"type":"user","message":{{"content":"Real message"}}}}"#
        )
        .unwrap();

        let result = extract_last_prompt(file.path().to_str().unwrap()).unwrap();
        assert_eq!(result, "Real message");
    }

    #[test]
    fn test_extract_last_prompt_missing_file() {
        let result = extract_last_prompt("/nonexistent/file.jsonl");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_last_prompt_with_fixture() {
        // Test with the simple fixture we created
        let fixture_path = std::env::current_dir()
            .unwrap()
            .join("tests/fixtures/transcripts/simple.jsonl");

        let result = extract_last_prompt(fixture_path.to_str().unwrap()).unwrap();
        assert_eq!(result, "Write a test for it");
    }

    #[test]
    fn test_extract_last_prompt_array_fixture() {
        let fixture_path = std::env::current_dir()
            .unwrap()
            .join("tests/fixtures/transcripts/array_content.jsonl");

        let result = extract_last_prompt(fixture_path.to_str().unwrap()).unwrap();
        assert_eq!(result, "Please review this code");
    }

    #[test]
    fn test_extract_last_prompt_multiline_fixture() {
        let fixture_path = std::env::current_dir()
            .unwrap()
            .join("tests/fixtures/transcripts/multiline.jsonl");

        let result = extract_last_prompt(fixture_path.to_str().unwrap()).unwrap();
        assert_eq!(result, "First line");
    }

    #[test]
    fn test_extract_last_prompt_empty_fixture() {
        let fixture_path = std::env::current_dir()
            .unwrap()
            .join("tests/fixtures/transcripts/empty.jsonl");

        let result = extract_last_prompt(fixture_path.to_str().unwrap());
        assert!(result.is_err());
    }

    // ========== build_from_claude_stdin_reader tests ==========

    #[test]
    fn test_build_from_stdin_empty() {
        let mock_stdin = std::io::Cursor::new("");
        let result = build_from_claude_stdin_reader(mock_stdin, "Test").unwrap();

        assert_eq!(result.title, "Test");
        assert_eq!(result.body, "Task finished");
    }

    #[test]
    fn test_build_from_stdin_invalid_json() {
        let mock_stdin = std::io::Cursor::new("not valid json");
        let result = build_from_claude_stdin_reader(mock_stdin, "Test");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse"));
    }

    #[test]
    fn test_build_from_stdin_permission_prompt_with_command() {
        let json = r#"{
            "cwd": "/Users/test/myproject",
            "tool_name": "Bash",
            "tool_input": {"command": "npm install"}
        }"#;
        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Claude Code").unwrap();

        assert_eq!(result.title, "Claude Code");
        assert_eq!(result.body, "[myproject] Bash: npm install");
    }

    #[test]
    fn test_build_from_stdin_permission_prompt_with_file_path() {
        let json = r#"{
            "cwd": "/Users/test/myproject",
            "tool_name": "Read",
            "tool_input": {"file_path": "/path/to/file.rs"}
        }"#;
        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Claude Code").unwrap();

        assert_eq!(result.body, "[myproject] Read: /path/to/file.rs");
    }

    #[test]
    fn test_build_from_stdin_permission_prompt_with_pattern() {
        let json = r#"{
            "cwd": "/Users/test/myproject",
            "tool_name": "Grep",
            "tool_input": {"pattern": "TODO"}
        }"#;
        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Claude Code").unwrap();

        assert_eq!(result.body, "[myproject] Grep: TODO");
    }

    #[test]
    fn test_build_from_stdin_permission_prompt_no_tool_input() {
        let json = r#"{
            "cwd": "/Users/test/myproject",
            "tool_name": "Bash"
        }"#;
        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Claude Code").unwrap();

        assert_eq!(result.body, "[myproject] Needs permission: Bash");
    }

    #[test]
    fn test_build_from_stdin_tool_truncation_at_60_chars() {
        // Create a command that's exactly 61 chars (should truncate)
        let long_command = "a".repeat(61);
        let json = format!(
            r#"{{
            "cwd": "/Users/test/myproject",
            "tool_name": "Bash",
            "tool_input": {{"command": "{}"}}
        }}"#,
            long_command
        );
        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Test").unwrap();

        // Should be truncated to 57 chars + "..."
        assert!(result.body.contains("..."));
        let command_part = result.body.split(": ").nth(1).unwrap();
        assert_eq!(command_part.len(), 60); // 57 + "..."
    }

    #[test]
    fn test_build_from_stdin_tool_no_truncation_at_60_chars() {
        // Command exactly 60 chars should NOT truncate
        let command = "a".repeat(60);
        let json = format!(
            r#"{{
            "cwd": "/Users/test/myproject",
            "tool_name": "Bash",
            "tool_input": {{"command": "{}"}}
        }}"#,
            command
        );
        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Test").unwrap();

        assert!(!result.body.contains("..."));
    }

    #[test]
    fn test_build_from_stdin_project_name_extraction() {
        let json = r#"{"cwd": "/home/user/projects/awesome-app"}"#;
        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Test").unwrap();

        assert!(result.body.starts_with("[awesome-app]"));
    }

    #[test]
    fn test_build_from_stdin_project_name_no_cwd() {
        let json = r#"{}"#;
        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Test").unwrap();

        assert!(result.body.starts_with("[project]"));
    }

    #[test]
    fn test_build_from_stdin_project_name_trailing_slash() {
        let json = r#"{"cwd": "/home/user/myproject/"}"#;
        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Test").unwrap();

        // Trailing slash results in empty string, falls back to "project"
        assert!(result.body.starts_with("[]") || result.body.starts_with("[project]"));
    }

    #[test]
    fn test_build_from_stdin_stop_hook_with_transcript() {
        // Create a temp transcript first
        let mut transcript = NamedTempFile::new().unwrap();
        writeln!(
            transcript,
            r#"{{"type":"user","message":{{"content":"Deploy to production"}}}}"#
        )
        .unwrap();

        let json = format!(
            r#"{{
            "cwd": "/Users/test/myproject",
            "transcript_path": "{}"
        }}"#,
            transcript.path().to_str().unwrap()
        );

        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Claude Code").unwrap();

        assert_eq!(result.body, "[myproject] Deploy to production");
    }

    #[test]
    fn test_build_from_stdin_stop_hook_no_transcript() {
        let json = r#"{"cwd": "/Users/test/myproject"}"#;
        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Test").unwrap();

        assert_eq!(result.body, "[myproject] Task finished");
    }

    #[test]
    fn test_build_from_stdin_prompt_truncation_at_100_chars() {
        // Create a very long prompt (101 chars)
        let mut transcript = NamedTempFile::new().unwrap();
        let long_prompt = "a".repeat(101);
        writeln!(
            transcript,
            r#"{{"type":"user","message":{{"content":"{}"}}}}"#,
            long_prompt
        )
        .unwrap();

        let json = format!(
            r#"{{
            "cwd": "/Users/test/myproject",
            "transcript_path": "{}"
        }}"#,
            transcript.path().to_str().unwrap()
        );

        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Test").unwrap();

        // Should be truncated to 97 chars + "..."
        assert!(result.body.contains("..."));
        let prompt_part = result.body.split("] ").nth(1).unwrap();
        assert_eq!(prompt_part.len(), 100); // 97 + "..."
    }

    #[test]
    fn test_build_from_stdin_prompt_no_truncation_at_100_chars() {
        // Prompt exactly 100 chars should NOT truncate
        let mut transcript = NamedTempFile::new().unwrap();
        let prompt = "a".repeat(100);
        writeln!(
            transcript,
            r#"{{"type":"user","message":{{"content":"{}"}}}}"#,
            prompt
        )
        .unwrap();

        let json = format!(
            r#"{{
            "cwd": "/Users/test/myproject",
            "transcript_path": "{}"
        }}"#,
            transcript.path().to_str().unwrap()
        );

        let mock_stdin = std::io::Cursor::new(json);
        let result = build_from_claude_stdin_reader(mock_stdin, "Test").unwrap();

        assert!(!result.body.contains("..."));
    }
}
