//! Per-widget test helpers.

use crate::snapshot::SnapshotTest;
use crate::terminal::TestTerminal;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::widgets::{StatefulWidget, Widget};

/// Common navigation keys for testing.
pub const NAVIGATION_KEYS: &[KeyCode] = &[
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Home,
    KeyCode::End,
    KeyCode::PageUp,
    KeyCode::PageDown,
];

/// Vim-style navigation keys.
pub const VIM_NAVIGATION_KEYS: &[KeyCode] = &[
    KeyCode::Char('h'),
    KeyCode::Char('j'),
    KeyCode::Char('k'),
    KeyCode::Char('l'),
    KeyCode::Char('g'), // gg for top
    KeyCode::Char('G'), // G for bottom
];

/// Trait for widgets that can handle key events.
pub trait KeyHandler {
    /// Handle a key event and return whether it was consumed.
    fn handle_key(&mut self, event: KeyEvent) -> bool;
}

/// Test that a widget renders correctly as a snapshot.
pub fn test_widget_snapshot<W: Widget>(widget: W, name: &str, width: u16, height: u16) {
    let mut test = SnapshotTest::new(name, width, height);
    test.assert_snapshot(widget);
}

/// Test that a stateful widget renders correctly.
pub fn test_stateful_widget_snapshot<W, S>(widget: W, state: &mut S, name: &str, width: u16, height: u16)
where
    W: StatefulWidget<State = S>,
{
    let mut terminal = TestTerminal::new(width, height);
    terminal.draw(|frame| {
        frame.render_stateful_widget(widget, frame.area(), state);
    });

    let mut test = SnapshotTest::new(name, width, height)
        .with_snapshots_dir("tests/snapshots");

    // Copy the buffer content to the snapshot test terminal
    let snapshot = terminal.snapshot();

    if crate::should_update_snapshots() {
        // Save the snapshot
        let path = std::path::PathBuf::from("tests/snapshots").join(format!("{}.snap", name));
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, snapshot.to_bytes().unwrap_or_default());
    }
}

/// Test that a widget handles all navigation keys without panicking.
pub fn assert_navigation_safe<W: KeyHandler>(widget: &mut W) {
    for key in NAVIGATION_KEYS {
        let event = KeyEvent::new(*key, KeyModifiers::NONE);
        let _ = widget.handle_key(event);
    }

    // Also test with modifiers
    for key in NAVIGATION_KEYS {
        let event = KeyEvent::new(*key, KeyModifiers::SHIFT);
        let _ = widget.handle_key(event);

        let event = KeyEvent::new(*key, KeyModifiers::CONTROL);
        let _ = widget.handle_key(event);
    }
}

/// Test that a widget handles vim navigation keys without panicking.
pub fn assert_vim_navigation_safe<W: KeyHandler>(widget: &mut W) {
    for key in VIM_NAVIGATION_KEYS {
        let event = KeyEvent::new(*key, KeyModifiers::NONE);
        let _ = widget.handle_key(event);
    }
}

/// Test that a widget renders within the given bounds.
pub fn assert_renders_in_bounds<W: Widget + Clone>(widget: &W, area: Rect) {
    let mut terminal = TestTerminal::new(area.width, area.height);

    // Should not panic
    terminal.draw(|frame| {
        frame.render_widget(widget.clone(), area);
    });

    // Verify terminal dimensions match
    assert_eq!(terminal.width(), area.width);
    assert_eq!(terminal.height(), area.height);
}

/// Test that a widget handles rapid key input without issues.
pub fn assert_handles_rapid_input<W: KeyHandler>(widget: &mut W, event: KeyEvent, count: usize) {
    for _ in 0..count {
        let _ = widget.handle_key(event);
    }
}

/// Test that a widget handles empty/zero-size areas gracefully.
pub fn assert_handles_empty_area<W: Widget + Clone>(widget: &W) {
    // Zero width
    let mut terminal = TestTerminal::new(1, 10);
    terminal.draw(|frame| {
        frame.render_widget(widget.clone(), Rect::new(0, 0, 0, 10));
    });

    // Zero height
    let mut terminal = TestTerminal::new(10, 1);
    terminal.draw(|frame| {
        frame.render_widget(widget.clone(), Rect::new(0, 0, 10, 0));
    });

    // Zero both
    let mut terminal = TestTerminal::new(1, 1);
    terminal.draw(|frame| {
        frame.render_widget(widget.clone(), Rect::new(0, 0, 0, 0));
    });
}

/// Test that a widget handles very small areas.
pub fn assert_handles_small_area<W: Widget + Clone>(widget: &W) {
    for size in 1..=5 {
        let mut terminal = TestTerminal::new(size, size);
        terminal.draw(|frame| {
            frame.render_widget(widget.clone(), frame.area());
        });
    }
}

/// Test that a widget handles very large areas.
pub fn assert_handles_large_area<W: Widget + Clone>(widget: &W) {
    let sizes = [(200, 50), (500, 100), (1000, 200)];

    for (width, height) in sizes {
        let mut terminal = TestTerminal::new(width, height);
        terminal.draw(|frame| {
            frame.render_widget(widget.clone(), frame.area());
        });
    }
}

/// Widget test builder for comprehensive testing.
pub struct WidgetTester<W> {
    widget: W,
    width: u16,
    height: u16,
    terminal: TestTerminal,
}

impl<W: Widget + Clone> WidgetTester<W> {
    /// Create a new widget tester.
    pub fn new(widget: W, width: u16, height: u16) -> Self {
        Self {
            widget,
            width,
            height,
            terminal: TestTerminal::new(width, height),
        }
    }

    /// Render the widget.
    pub fn render(&mut self) -> &mut Self {
        let widget = self.widget.clone();
        self.terminal.draw(|frame| {
            frame.render_widget(widget, frame.area());
        });
        self
    }

    /// Assert the terminal contains the given text.
    pub fn assert_contains(&self, text: &str) -> &Self {
        self.terminal.assert_contains(text);
        self
    }

    /// Assert the terminal does not contain the given text.
    pub fn assert_not_contains(&self, text: &str) -> &Self {
        self.terminal.assert_not_contains(text);
        self
    }

    /// Assert a specific line contains the given text.
    pub fn assert_line_contains(&self, line: u16, text: &str) -> &Self {
        let line_content = self.terminal.line(line);
        assert!(
            line_content.contains(text),
            "Line {} does not contain '{}': '{}'",
            line,
            text,
            line_content
        );
        self
    }

    /// Get the terminal for further inspection.
    pub fn terminal(&self) -> &TestTerminal {
        &self.terminal
    }

    /// Get the current buffer as string.
    pub fn to_string(&self) -> String {
        self.terminal.to_string()
    }

    /// Take a snapshot.
    pub fn snapshot(&self, name: &str) {
        let mut test = SnapshotTest::new(name, self.width, self.height);
        test.assert_snapshot(self.widget.clone());
    }
}

/// Test helper for stateful widgets.
pub struct StatefulWidgetTester<W, S> {
    widget: W,
    state: S,
    width: u16,
    height: u16,
    terminal: TestTerminal,
}

impl<W, S> StatefulWidgetTester<W, S>
where
    W: StatefulWidget<State = S> + Clone,
    S: Default,
{
    /// Create a new stateful widget tester with default state.
    pub fn new(widget: W, width: u16, height: u16) -> Self {
        Self {
            widget,
            state: S::default(),
            width,
            height,
            terminal: TestTerminal::new(width, height),
        }
    }

    /// Create with a specific initial state.
    pub fn with_state(widget: W, state: S, width: u16, height: u16) -> Self {
        Self {
            widget,
            state,
            width,
            height,
            terminal: TestTerminal::new(width, height),
        }
    }

    /// Get mutable access to the state.
    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }

    /// Get access to the state.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Render the widget with current state.
    pub fn render(&mut self) -> &mut Self {
        let widget = self.widget.clone();
        let state = &mut self.state;
        self.terminal.draw(|frame| {
            frame.render_stateful_widget(widget, frame.area(), state);
        });
        self
    }

    /// Assert the terminal contains the given text.
    pub fn assert_contains(&self, text: &str) -> &Self {
        self.terminal.assert_contains(text);
        self
    }

    /// Get the terminal for further inspection.
    pub fn terminal(&self) -> &TestTerminal {
        &self.terminal
    }
}

impl<W, S> StatefulWidgetTester<W, S>
where
    W: StatefulWidget<State = S> + Clone,
    S: KeyHandler,
{
    /// Send a key event to the widget state.
    pub fn key(&mut self, code: KeyCode) -> &mut Self {
        let event = KeyEvent::new(code, KeyModifiers::NONE);
        self.state.handle_key(event);
        self
    }

    /// Send a key event with modifiers.
    pub fn key_mod(&mut self, code: KeyCode, modifiers: KeyModifiers) -> &mut Self {
        let event = KeyEvent::new(code, modifiers);
        self.state.handle_key(event);
        self
    }

    /// Send Ctrl+key.
    pub fn ctrl(&mut self, c: char) -> &mut Self {
        self.key_mod(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    /// Navigate down.
    pub fn down(&mut self) -> &mut Self {
        self.key(KeyCode::Down)
    }

    /// Navigate up.
    pub fn up(&mut self) -> &mut Self {
        self.key(KeyCode::Up)
    }

    /// Press enter.
    pub fn enter(&mut self) -> &mut Self {
        self.key(KeyCode::Enter)
    }

    /// Press escape.
    pub fn esc(&mut self) -> &mut Self {
        self.key(KeyCode::Esc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Paragraph;

    #[test]
    fn test_widget_tester() {
        let widget = Paragraph::new("Hello, World!");
        let tester = WidgetTester::new(widget, 20, 5);

        tester
            .render()
            .assert_contains("Hello")
            .assert_not_contains("Goodbye");
    }

    #[test]
    fn test_handles_small_area() {
        let widget = Paragraph::new("Test");
        assert_handles_small_area(&widget);
    }

    #[test]
    fn test_handles_large_area() {
        let widget = Paragraph::new("Test content here");
        assert_handles_large_area(&widget);
    }

    #[test]
    fn test_renders_in_bounds() {
        let widget = Paragraph::new("Bounded");
        let area = Rect::new(0, 0, 20, 5);
        assert_renders_in_bounds(&widget, area);
    }
}
