//! Unified notification system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Unique notification ID.
pub type NotificationId = u64;

/// A notification from an app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique ID.
    pub id: NotificationId,
    /// Source app name.
    pub source: String,
    /// Notification level.
    pub level: NotificationLevel,
    /// Message content.
    pub message: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Optional action.
    pub action: Option<NotificationAction>,
    /// Priority (higher = stays visible longer).
    pub priority: u8,
}

impl Notification {
    /// Create a new notification.
    pub fn new(source: impl Into<String>, level: NotificationLevel, message: impl Into<String>) -> Self {
        Self {
            id: 0, // Set by queue
            source: source.into(),
            level,
            message: message.into(),
            timestamp: Utc::now(),
            action: None,
            priority: level.default_priority(),
        }
    }

    /// Create an info notification.
    pub fn info(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(source, NotificationLevel::Info, message)
    }

    /// Create a success notification.
    pub fn success(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(source, NotificationLevel::Success, message)
    }

    /// Create a warning notification.
    pub fn warning(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(source, NotificationLevel::Warning, message)
    }

    /// Create an error notification.
    pub fn error(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(source, NotificationLevel::Error, message)
    }

    /// Set an action.
    pub fn with_action(mut self, action: NotificationAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Notification level/severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    /// Informational.
    Info,
    /// Success.
    Success,
    /// Warning.
    Warning,
    /// Error.
    Error,
}

impl NotificationLevel {
    /// Get default priority for this level.
    pub fn default_priority(&self) -> u8 {
        match self {
            Self::Info => 1,
            Self::Success => 2,
            Self::Warning => 3,
            Self::Error => 4,
        }
    }

    /// Get display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    /// Get icon character.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "ℹ",
            Self::Success => "✓",
            Self::Warning => "⚠",
            Self::Error => "✗",
        }
    }
}

/// An action that can be triggered from a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    /// Action label.
    pub label: String,
    /// Command to execute.
    pub command: String,
    /// Optional arguments.
    pub args: Vec<String>,
}

impl NotificationAction {
    /// Create a new action.
    pub fn new(label: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            command: command.into(),
            args: Vec::new(),
        }
    }

    /// Add arguments.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
}

/// Configuration for notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Minimum level to display.
    pub show_level: NotificationLevel,
    /// Maximum notifications in collapsed marquee.
    pub max_visible: usize,
    /// Maximum notifications in history.
    pub max_history: usize,
    /// Auto-dismiss times by level (0 = never).
    pub auto_dismiss_secs: HashMap<String, u64>,
    /// Marquee scroll speed.
    pub marquee_speed: MarqueeSpeed,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        let mut auto_dismiss = HashMap::new();
        auto_dismiss.insert("info".to_string(), 5);
        auto_dismiss.insert("success".to_string(), 5);
        auto_dismiss.insert("warning".to_string(), 10);
        auto_dismiss.insert("error".to_string(), 0);

        Self {
            show_level: NotificationLevel::Info,
            max_visible: 3,
            max_history: 100,
            auto_dismiss_secs: auto_dismiss,
            marquee_speed: MarqueeSpeed::Normal,
        }
    }
}

/// Marquee scroll speed.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarqueeSpeed {
    Slow,
    #[default]
    Normal,
    Fast,
}

impl MarqueeSpeed {
    /// Get delay in milliseconds between scroll steps.
    pub fn delay_ms(&self) -> u64 {
        match self {
            Self::Slow => 200,
            Self::Normal => 100,
            Self::Fast => 50,
        }
    }
}

/// Queue of notifications.
pub struct NotificationQueue {
    /// Active notifications.
    notifications: VecDeque<Notification>,
    /// Configuration.
    config: NotificationConfig,
    /// Whether panel is expanded.
    panel_expanded: bool,
    /// Next notification ID.
    next_id: NotificationId,
}

impl NotificationQueue {
    /// Create a new notification queue.
    pub fn new(config: NotificationConfig) -> Self {
        Self {
            notifications: VecDeque::new(),
            config,
            panel_expanded: false,
            next_id: 1,
        }
    }

    /// Push a notification.
    pub fn push(&mut self, mut notif: Notification) {
        notif.id = self.next_id;
        self.next_id += 1;

        // Insert by priority (higher priority first)
        let pos = self
            .notifications
            .iter()
            .position(|n| n.priority < notif.priority)
            .unwrap_or(self.notifications.len());

        self.notifications.insert(pos, notif);

        // Trim history
        while self.notifications.len() > self.config.max_history {
            self.notifications.pop_back();
        }
    }

    /// Dismiss a notification.
    pub fn dismiss(&mut self, id: NotificationId) {
        self.notifications.retain(|n| n.id != id);
    }

    /// Dismiss all notifications.
    pub fn dismiss_all(&mut self) {
        self.notifications.clear();
    }

    /// Toggle the expanded panel.
    pub fn toggle_panel(&mut self) {
        self.panel_expanded = !self.panel_expanded;
    }

    /// Check if panel is expanded.
    pub fn is_expanded(&self) -> bool {
        self.panel_expanded
    }

    /// Get visible notifications (priority-sorted).
    pub fn get_visible(&self) -> Vec<&Notification> {
        self.notifications
            .iter()
            .filter(|n| n.level as u8 >= self.config.show_level as u8)
            .take(self.config.max_visible)
            .collect()
    }

    /// Get all notifications in history.
    pub fn get_history(&self) -> Vec<&Notification> {
        self.notifications.iter().collect()
    }

    /// Get notification count.
    pub fn count(&self) -> usize {
        self.notifications.len()
    }

    /// Check if there are notifications.
    pub fn has_notifications(&self) -> bool {
        !self.notifications.is_empty()
    }

    /// Get the most recent notification.
    pub fn latest(&self) -> Option<&Notification> {
        self.notifications.front()
    }

    /// Process auto-dismiss based on age.
    pub fn process_auto_dismiss(&mut self) {
        let now = Utc::now();

        self.notifications.retain(|n| {
            let level_name = n.level.name();
            let dismiss_secs = self
                .config
                .auto_dismiss_secs
                .get(level_name)
                .copied()
                .unwrap_or(0);

            if dismiss_secs == 0 {
                return true; // Never auto-dismiss
            }

            let age = now.signed_duration_since(n.timestamp);
            age.num_seconds() < dismiss_secs as i64
        });
    }
}

impl Default for NotificationQueue {
    fn default() -> Self {
        Self::new(NotificationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notif = Notification::info("test-app", "Hello");
        assert_eq!(notif.source, "test-app");
        assert_eq!(notif.message, "Hello");
        assert_eq!(notif.level, NotificationLevel::Info);
    }

    #[test]
    fn test_notification_queue() {
        let mut queue = NotificationQueue::default();
        assert!(!queue.has_notifications());

        queue.push(Notification::info("app", "Test"));
        assert!(queue.has_notifications());
        assert_eq!(queue.count(), 1);
    }

    #[test]
    fn test_priority_ordering() {
        let mut queue = NotificationQueue::default();

        queue.push(Notification::info("app", "Info"));
        queue.push(Notification::error("app", "Error"));
        queue.push(Notification::warning("app", "Warning"));

        let visible = queue.get_visible();
        assert_eq!(visible[0].level, NotificationLevel::Error);
    }

    #[test]
    fn test_dismiss() {
        let mut queue = NotificationQueue::default();
        queue.push(Notification::info("app", "Test"));

        let id = queue.latest().unwrap().id;
        queue.dismiss(id);
        assert!(!queue.has_notifications());
    }

    #[test]
    fn test_level_priority() {
        assert!(NotificationLevel::Error.default_priority() > NotificationLevel::Info.default_priority());
    }
}
