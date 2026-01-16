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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_new() {
        let notif = Notification::new("Title", "Body");
        assert_eq!(notif.title, "Title");
        assert_eq!(notif.body, "Body");
        assert!(notif.icon.is_none());
        assert!(notif.activate.is_none());
        assert!(notif.metadata.is_empty());
    }

    #[test]
    fn test_notification_with_icon() {
        let notif = Notification::new("Title", "Body").with_icon("claude");
        assert_eq!(notif.icon, Some("claude".to_string()));
    }

    #[test]
    fn test_notification_with_activate() {
        let notif = Notification::new("Title", "Body")
            .with_activate("com.apple.Terminal");
        assert_eq!(notif.activate, Some("com.apple.Terminal".to_string()));
    }

    #[test]
    fn test_notification_builder_chain() {
        let notif = Notification::new("Title", "Body")
            .with_icon("test")
            .with_activate("bundle.id");

        assert_eq!(notif.title, "Title");
        assert_eq!(notif.body, "Body");
        assert_eq!(notif.icon, Some("test".to_string()));
        assert_eq!(notif.activate, Some("bundle.id".to_string()));
    }

    #[test]
    fn test_notification_serialization() {
        let notif = Notification::new("Test", "Message");
        let json = serde_json::to_string(&notif).unwrap();

        // Should only contain title and body (no null fields)
        assert!(json.contains("\"title\""));
        assert!(json.contains("\"body\""));
        assert!(!json.contains("\"icon\""));
        assert!(!json.contains("\"activate\""));
        assert!(!json.contains("\"metadata\""));
    }

    #[test]
    fn test_notification_serialization_with_optionals() {
        let notif = Notification::new("Test", "Message")
            .with_icon("test-icon")
            .with_activate("test.bundle");

        let json = serde_json::to_string(&notif).unwrap();
        assert!(json.contains("\"icon\""));
        assert!(json.contains("\"activate\""));
    }

    #[test]
    fn test_notification_deserialization() {
        let json = r#"{"title":"Test","body":"Message"}"#;
        let notif: Notification = serde_json::from_str(json).unwrap();

        assert_eq!(notif.title, "Test");
        assert_eq!(notif.body, "Message");
        assert!(notif.icon.is_none());
        assert!(notif.activate.is_none());
        assert!(notif.metadata.is_empty());
    }

    #[test]
    fn test_notification_deserialization_with_all_fields() {
        let json = r#"{
            "title":"Test",
            "body":"Message",
            "icon":"test-icon",
            "activate":"test.bundle",
            "metadata":{"key":"value"}
        }"#;
        let notif: Notification = serde_json::from_str(json).unwrap();

        assert_eq!(notif.title, "Test");
        assert_eq!(notif.body, "Message");
        assert_eq!(notif.icon, Some("test-icon".to_string()));
        assert_eq!(notif.activate, Some("test.bundle".to_string()));
        assert_eq!(notif.metadata.len(), 1);
        assert!(notif.metadata.contains_key("key"));
    }
}
