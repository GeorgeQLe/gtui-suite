//! Prefix key handling for shell commands.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Prefix key state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixState {
    /// Not in prefix mode.
    Inactive,
    /// Prefix key pressed, waiting for command.
    Active,
    /// Prefix timed out.
    TimedOut,
}

/// Shell command triggered by prefix key sequence.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShellCommand {
    /// Show command palette.
    CommandPalette,
    /// Show app launcher.
    Launcher,
    /// Focus next app.
    NextApp,
    /// Focus previous app.
    PrevApp,
    /// Focus app by number (1-9).
    FocusApp(u8),
    /// Switch to next workspace.
    NextWorkspace,
    /// Switch to previous workspace.
    PrevWorkspace,
    /// Switch to workspace by number.
    SwitchWorkspace(u8),
    /// Create new workspace.
    NewWorkspace,
    /// Close current app.
    CloseApp,
    /// Kill current app.
    KillApp,
    /// Maximize/restore current app.
    ToggleMaximize,
    /// Toggle fullscreen.
    ToggleFullscreen,
    /// Show notification panel.
    NotificationPanel,
    /// Dismiss notifications.
    DismissNotifications,
    /// Show help.
    Help,
    /// Show settings.
    Settings,
    /// Quit shell.
    Quit,
    /// Split horizontally.
    SplitHorizontal,
    /// Split vertically.
    SplitVertical,
    /// Focus direction.
    FocusDirection(Direction),
    /// Swap with direction.
    SwapDirection(Direction),
    /// Resize in direction.
    Resize(Direction, i16),
    /// Custom command.
    Custom(String),
}

/// Direction for navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Prefix key handler.
pub struct PrefixKeyHandler {
    /// Prefix key.
    prefix_key: KeyEvent,
    /// Timeout duration.
    timeout: Duration,
    /// Current state.
    state: PrefixState,
    /// When prefix was activated.
    activated_at: Option<Instant>,
    /// Key bindings (key after prefix -> command).
    bindings: HashMap<KeyEvent, ShellCommand>,
    /// Whether to show indicator.
    show_indicator: bool,
}

impl PrefixKeyHandler {
    /// Create new handler with default prefix (Ctrl+Space).
    pub fn new() -> Self {
        let mut handler = Self {
            prefix_key: KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL),
            timeout: Duration::from_millis(500),
            state: PrefixState::Inactive,
            activated_at: None,
            bindings: HashMap::new(),
            show_indicator: true,
        };

        handler.register_defaults();
        handler
    }

    /// Create with custom prefix key.
    pub fn with_prefix(key: KeyEvent) -> Self {
        let mut handler = Self {
            prefix_key: key,
            timeout: Duration::from_millis(500),
            state: PrefixState::Inactive,
            activated_at: None,
            bindings: HashMap::new(),
            show_indicator: true,
        };

        handler.register_defaults();
        handler
    }

    /// Set timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Parse prefix key from string.
    pub fn parse_prefix(s: &str) -> Option<KeyEvent> {
        parse_key_event(s)
    }

    /// Set prefix from string.
    pub fn set_prefix(&mut self, s: &str) -> bool {
        if let Some(key) = Self::parse_prefix(s) {
            self.prefix_key = key;
            true
        } else {
            false
        }
    }

    /// Register default bindings.
    fn register_defaults(&mut self) {
        // Command palette and launcher
        self.bind(key('p'), ShellCommand::CommandPalette);
        self.bind(key(' '), ShellCommand::Launcher);

        // App navigation
        self.bind(key('n'), ShellCommand::NextApp);
        self.bind(shift_key('N'), ShellCommand::PrevApp);

        // Number keys for direct app focus
        for i in 1..=9 {
            self.bind(
                KeyEvent::new(KeyCode::Char((b'0' + i) as char), KeyModifiers::NONE),
                ShellCommand::FocusApp(i),
            );
        }

        // Workspace navigation
        self.bind(key(']'), ShellCommand::NextWorkspace);
        self.bind(key('['), ShellCommand::PrevWorkspace);
        self.bind(key('c'), ShellCommand::NewWorkspace);

        // Window management
        self.bind(key('x'), ShellCommand::CloseApp);
        self.bind(shift_key('X'), ShellCommand::KillApp);
        self.bind(key('m'), ShellCommand::ToggleMaximize);
        self.bind(key('f'), ShellCommand::ToggleFullscreen);

        // Splits
        self.bind(key('-'), ShellCommand::SplitHorizontal);
        self.bind(key('|'), ShellCommand::SplitVertical);

        // Direction focus (vim-style)
        self.bind(key('h'), ShellCommand::FocusDirection(Direction::Left));
        self.bind(key('j'), ShellCommand::FocusDirection(Direction::Down));
        self.bind(key('k'), ShellCommand::FocusDirection(Direction::Up));
        self.bind(key('l'), ShellCommand::FocusDirection(Direction::Right));

        // Direction swap
        self.bind(shift_key('H'), ShellCommand::SwapDirection(Direction::Left));
        self.bind(shift_key('J'), ShellCommand::SwapDirection(Direction::Down));
        self.bind(shift_key('K'), ShellCommand::SwapDirection(Direction::Up));
        self.bind(shift_key('L'), ShellCommand::SwapDirection(Direction::Right));

        // Arrow key alternatives
        self.bind(
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            ShellCommand::FocusDirection(Direction::Left),
        );
        self.bind(
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            ShellCommand::FocusDirection(Direction::Down),
        );
        self.bind(
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            ShellCommand::FocusDirection(Direction::Up),
        );
        self.bind(
            KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            ShellCommand::FocusDirection(Direction::Right),
        );

        // Notifications
        self.bind(key('!'), ShellCommand::NotificationPanel);
        self.bind(key('d'), ShellCommand::DismissNotifications);

        // Meta
        self.bind(key('?'), ShellCommand::Help);
        self.bind(key(','), ShellCommand::Settings);
        self.bind(key('q'), ShellCommand::Quit);
    }

    /// Bind a key to a command.
    pub fn bind(&mut self, key: KeyEvent, command: ShellCommand) {
        self.bindings.insert(key, command);
    }

    /// Unbind a key.
    pub fn unbind(&mut self, key: &KeyEvent) {
        self.bindings.remove(key);
    }

    /// Get current state.
    pub fn state(&self) -> PrefixState {
        self.state
    }

    /// Check if prefix is active.
    pub fn is_active(&self) -> bool {
        self.state == PrefixState::Active
    }

    /// Handle key event.
    pub fn handle(&mut self, key: KeyEvent) -> PrefixResult {
        // Check timeout
        if self.state == PrefixState::Active {
            if let Some(activated_at) = self.activated_at {
                if activated_at.elapsed() > self.timeout {
                    self.state = PrefixState::TimedOut;
                    self.activated_at = None;
                    return PrefixResult::TimedOut;
                }
            }
        }

        match self.state {
            PrefixState::Inactive | PrefixState::TimedOut => {
                if is_key_match(&key, &self.prefix_key) {
                    self.state = PrefixState::Active;
                    self.activated_at = Some(Instant::now());
                    PrefixResult::Activated
                } else {
                    PrefixResult::PassThrough
                }
            }
            PrefixState::Active => {
                self.state = PrefixState::Inactive;
                self.activated_at = None;

                // Double-tap prefix to pass through
                if is_key_match(&key, &self.prefix_key) {
                    return PrefixResult::PassPrefix;
                }

                // Escape to cancel
                if key.code == KeyCode::Esc {
                    return PrefixResult::Cancelled;
                }

                // Look up binding
                if let Some(command) = self.bindings.get(&key).cloned() {
                    PrefixResult::Command(command)
                } else {
                    PrefixResult::UnknownKey(key)
                }
            }
        }
    }

    /// Cancel prefix mode.
    pub fn cancel(&mut self) {
        self.state = PrefixState::Inactive;
        self.activated_at = None;
    }

    /// Get all bindings.
    pub fn bindings(&self) -> &HashMap<KeyEvent, ShellCommand> {
        &self.bindings
    }

    /// Get binding for a command.
    pub fn key_for(&self, command: &ShellCommand) -> Option<&KeyEvent> {
        self.bindings.iter().find(|(_, c)| *c == command).map(|(k, _)| k)
    }

    /// Get prefix key display string.
    pub fn prefix_display(&self) -> String {
        format_key_event(&self.prefix_key)
    }

    /// Get key display string.
    pub fn key_display(&self, key: &KeyEvent) -> String {
        format_key_event(key)
    }

    /// Should show indicator.
    pub fn show_indicator(&self) -> bool {
        self.show_indicator && self.state == PrefixState::Active
    }

    /// Set indicator visibility.
    pub fn set_show_indicator(&mut self, show: bool) {
        self.show_indicator = show;
    }

    /// Get remaining timeout.
    pub fn remaining_timeout(&self) -> Option<Duration> {
        if self.state == PrefixState::Active {
            self.activated_at.map(|at| {
                self.timeout.saturating_sub(at.elapsed())
            })
        } else {
            None
        }
    }
}

impl Default for PrefixKeyHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of prefix key handling.
#[derive(Debug, Clone, PartialEq)]
pub enum PrefixResult {
    /// Prefix mode activated.
    Activated,
    /// Command triggered.
    Command(ShellCommand),
    /// Prefix cancelled (Escape).
    Cancelled,
    /// Prefix timed out.
    TimedOut,
    /// Unknown key in prefix mode.
    UnknownKey(KeyEvent),
    /// Pass prefix key through (double-tap).
    PassPrefix,
    /// Pass key through to app.
    PassThrough,
}

// Helper functions

fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn shift_key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)
}

fn is_key_match(a: &KeyEvent, b: &KeyEvent) -> bool {
    a.code == b.code && a.modifiers == b.modifiers
}

fn parse_key_event(s: &str) -> Option<KeyEvent> {
    let s = s.to_lowercase();
    let parts: Vec<&str> = s.split('+').collect();

    let mut modifiers = KeyModifiers::NONE;
    let key_str = parts.last()?;

    for part in &parts[..parts.len().saturating_sub(1)] {
        match *part {
            "ctrl" | "c" => modifiers |= KeyModifiers::CONTROL,
            "alt" | "a" | "m" => modifiers |= KeyModifiers::ALT,
            "shift" | "s" => modifiers |= KeyModifiers::SHIFT,
            _ => {}
        }
    }

    let code = match *key_str {
        "space" | " " => KeyCode::Char(' '),
        "enter" | "return" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backspace" | "bs" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" | "pgup" => KeyCode::PageUp,
        "pagedown" | "pgdn" => KeyCode::PageDown,
        s if s.len() == 1 => KeyCode::Char(s.chars().next()?),
        s if s.starts_with('f') => {
            let n: u8 = s[1..].parse().ok()?;
            KeyCode::F(n)
        }
        _ => return None,
    };

    Some(KeyEvent::new(code, modifiers))
}

fn format_key_event(key: &KeyEvent) -> String {
    let mut parts = Vec::new();

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }

    let key_name = match key.code {
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_uppercase().to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PgUp".to_string(),
        KeyCode::PageDown => "PgDn".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => "?".to_string(),
    };

    parts.push(&key_name);
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_activation() {
        let mut handler = PrefixKeyHandler::new();
        assert_eq!(handler.state(), PrefixState::Inactive);

        let result = handler.handle(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL));
        assert_eq!(result, PrefixResult::Activated);
        assert_eq!(handler.state(), PrefixState::Active);
    }

    #[test]
    fn test_prefix_command() {
        let mut handler = PrefixKeyHandler::new();

        // Activate prefix
        handler.handle(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL));

        // Press 'p' for command palette
        let result = handler.handle(key('p'));
        assert_eq!(result, PrefixResult::Command(ShellCommand::CommandPalette));
        assert_eq!(handler.state(), PrefixState::Inactive);
    }

    #[test]
    fn test_prefix_cancel() {
        let mut handler = PrefixKeyHandler::new();

        handler.handle(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL));
        let result = handler.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert_eq!(result, PrefixResult::Cancelled);
        assert_eq!(handler.state(), PrefixState::Inactive);
    }

    #[test]
    fn test_double_tap_passthrough() {
        let mut handler = PrefixKeyHandler::new();

        handler.handle(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL));
        let result = handler.handle(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL));

        assert_eq!(result, PrefixResult::PassPrefix);
    }

    #[test]
    fn test_parse_key() {
        let key = parse_key_event("ctrl+space").unwrap();
        assert_eq!(key.code, KeyCode::Char(' '));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));

        let key = parse_key_event("alt+f1").unwrap();
        assert_eq!(key.code, KeyCode::F(1));
        assert!(key.modifiers.contains(KeyModifiers::ALT));
    }

    #[test]
    fn test_format_key() {
        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL);
        assert_eq!(format_key_event(&key), "Ctrl+Space");
    }
}
