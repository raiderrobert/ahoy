use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A notification message sent to the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Notification title
    pub title: String,

    /// Notification body text
    pub body: String,

    /// Optional icon identifier (e.g., "claude", "codex", "gemini")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// Bundle ID to activate when notification is clicked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activate: Option<String>,

    /// Optional metadata for extensibility
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Notification {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            icon: None,
            activate: None,
            metadata: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[allow(dead_code)]
    pub fn with_activate(mut self, bundle_id: impl Into<String>) -> Self {
        self.activate = Some(bundle_id.into());
        self
    }
}
