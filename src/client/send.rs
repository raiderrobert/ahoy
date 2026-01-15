use anyhow::{bail, Context, Result};
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
    // For Notification hooks (permission_prompt)
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
    // Read stdin
    let mut stdin_data = String::new();
    io::stdin().read_to_string(&mut stdin_data)?;

    if stdin_data.is_empty() {
        return Ok(Notification::new(
            title.to_string(),
            "Task finished".to_string(),
        ));
    }

    // Parse the hook data
    let hook_data: ClaudeHookData = serde_json::from_str(&stdin_data)
        .context("Failed to parse Claude hook data from stdin")?;

    // Get project name from cwd
    let project_name = hook_data
        .cwd
        .as_ref()
        .and_then(|cwd| cwd.split('/').last())
        .unwrap_or("project");

    // Check if this is a Notification hook with tool info (permission_prompt)
    if let Some(tool_name) = &hook_data.tool_name {
        // Extract a brief description of the tool input
        let tool_desc = if let Some(input) = &hook_data.tool_input {
            // Try to get command for Bash, or file_path for Read/Write/Edit
            input.get("command")
                .or_else(|| input.get("file_path"))
                .or_else(|| input.get("pattern"))
                .and_then(|v| v.as_str())
                .map(|s| {
                    // Truncate long commands
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

    // For Stop hooks: Try to get the last prompt from transcript
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

        if let Ok(entry) = serde_json::from_str::<TranscriptLine>(&line) {
            if entry.line_type.as_deref() == Some("user") {
                if let Some(msg) = entry.message {
                    if let Some(content) = msg.content {
                        // Content can be a string or array
                        let text = match content {
                            serde_json::Value::String(s) => s,
                            serde_json::Value::Array(arr) => {
                                // Extract text from content blocks
                                arr.iter()
                                    .filter_map(|item| {
                                        item.get("text").and_then(|t| t.as_str())
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            }
                            _ => continue,
                        };

                        // Clean up the text (remove newlines, trim)
                        let cleaned = text
                            .lines()
                            .next()
                            .unwrap_or(&text)
                            .trim()
                            .to_string();

                        if !cleaned.is_empty() {
                            last_user_content = Some(cleaned);
                        }
                    }
                }
            }
        }
    }

    last_user_content.ok_or_else(|| anyhow::anyhow!("No user message found in transcript"))
}

fn send_notification(notification: &Notification) -> Result<()> {
    info!("Showing notification: {:?}", notification);
    notify::show(notification)
}
