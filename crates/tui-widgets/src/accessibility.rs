//! Accessibility support for TUI widgets.
//!
//! Provides screen reader hints, sound cues, and text-based announcements
//! for accessible terminal interfaces.

/// Sound cues for accessibility feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCue {
    /// Selection or activation sound
    Select,
    /// Error or invalid action sound
    Error,
    /// Success or completion sound
    Success,
    /// Warning or caution sound
    Warning,
    /// Navigation movement sound
    Navigate,
}

impl SoundCue {
    /// Get the default frequency for this sound cue (in Hz).
    pub fn frequency(&self) -> u32 {
        match self {
            Self::Select => 800,
            Self::Error => 200,
            Self::Success => 1000,
            Self::Warning => 400,
            Self::Navigate => 600,
        }
    }

    /// Get the default duration for this sound cue (in ms).
    pub fn duration_ms(&self) -> u32 {
        match self {
            Self::Select => 50,
            Self::Error => 200,
            Self::Success => 100,
            Self::Warning => 150,
            Self::Navigate => 30,
        }
    }
}

/// Accessibility configuration for widgets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessibilityConfig {
    /// Enable screen reader hints (ARIA-like roles and states)
    pub screen_reader_hints: bool,
    /// Enable sound cues for actions
    pub sound_cues: bool,
    /// Enable text-based status announcements
    pub status_announcements: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader_hints: false,
            sound_cues: false,
            status_announcements: false,
        }
    }
}

impl AccessibilityConfig {
    /// Create a new accessibility config with all features enabled.
    pub fn full() -> Self {
        Self {
            screen_reader_hints: true,
            sound_cues: true,
            status_announcements: true,
        }
    }

    /// Create a new accessibility config with screen reader support only.
    pub fn screen_reader() -> Self {
        Self {
            screen_reader_hints: true,
            sound_cues: false,
            status_announcements: true,
        }
    }
}

/// Trait for accessible widgets.
///
/// Widgets implementing this trait can provide information to screen readers
/// and other assistive technologies.
pub trait Accessible {
    /// Get the ARIA-like role for this widget.
    ///
    /// Common roles: "button", "listbox", "tree", "grid", "dialog", "textbox"
    fn aria_role(&self) -> &str;

    /// Get a human-readable label for the widget.
    fn aria_label(&self) -> String;

    /// Announce a message to the screen reader.
    ///
    /// This queues a message to be read by the screen reader.
    fn announce(&self, message: &str);

    /// Play an audio cue.
    ///
    /// This provides audio feedback for actions.
    fn play_sound(&self, sound: SoundCue);

    /// Get the current value or state as a string.
    ///
    /// For example, a checkbox might return "checked" or "unchecked".
    fn aria_value(&self) -> Option<String> {
        None
    }

    /// Get whether the widget is expanded (for tree nodes, accordions, etc.).
    fn aria_expanded(&self) -> Option<bool> {
        None
    }

    /// Get the currently selected item description.
    fn aria_selected(&self) -> Option<String> {
        None
    }

    /// Get the position in a set (e.g., "3 of 10").
    fn aria_position(&self) -> Option<(usize, usize)> {
        None
    }
}

/// Announcement buffer for screen reader messages.
///
/// Widgets can queue announcements here, and the application can
/// poll for pending announcements to display in a status line.
#[derive(Debug, Default)]
pub struct AnnouncementBuffer {
    messages: Vec<Announcement>,
}

/// A single announcement with priority.
#[derive(Debug, Clone)]
pub struct Announcement {
    /// The message text
    pub message: String,
    /// Priority level (higher = more important)
    pub priority: AnnouncementPriority,
    /// Whether this announcement interrupts others
    pub interrupt: bool,
}

/// Priority levels for announcements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnnouncementPriority {
    /// Low priority, can be skipped if busy
    Low,
    /// Normal priority
    Normal,
    /// High priority, should be announced soon
    High,
    /// Critical, interrupt current announcement
    Critical,
}

impl AnnouncementBuffer {
    /// Create a new empty announcement buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue an announcement.
    pub fn announce(&mut self, message: impl Into<String>, priority: AnnouncementPriority) {
        self.messages.push(Announcement {
            message: message.into(),
            priority,
            interrupt: priority == AnnouncementPriority::Critical,
        });
    }

    /// Get the next announcement to read.
    ///
    /// Returns the highest priority announcement, or None if empty.
    pub fn next(&mut self) -> Option<Announcement> {
        if self.messages.is_empty() {
            return None;
        }

        // Find highest priority
        let max_priority = self.messages.iter().map(|a| a.priority).max()?;
        let idx = self.messages.iter().position(|a| a.priority == max_priority)?;
        Some(self.messages.remove(idx))
    }

    /// Check if there are any pending announcements.
    pub fn has_pending(&self) -> bool {
        !self.messages.is_empty()
    }

    /// Clear all pending announcements.
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_announcement_priority() {
        let mut buffer = AnnouncementBuffer::new();

        buffer.announce("low", AnnouncementPriority::Low);
        buffer.announce("high", AnnouncementPriority::High);
        buffer.announce("normal", AnnouncementPriority::Normal);

        // Should get high first
        assert_eq!(buffer.next().unwrap().message, "high");
        assert_eq!(buffer.next().unwrap().message, "normal");
        assert_eq!(buffer.next().unwrap().message, "low");
        assert!(buffer.next().is_none());
    }

    #[test]
    fn test_sound_cue_properties() {
        assert!(SoundCue::Error.frequency() < SoundCue::Success.frequency());
        assert!(SoundCue::Navigate.duration_ms() < SoundCue::Error.duration_ms());
    }
}
