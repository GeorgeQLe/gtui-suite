//! Input simulation for testing.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::time::Duration;

/// A sequence of input events for testing.
#[derive(Debug, Clone, Default)]
pub struct InputSequence {
    /// The events in the sequence.
    events: Vec<InputItem>,
    /// Whether to respect timing delays.
    real_timing: bool,
}

/// An item in an input sequence.
#[derive(Debug, Clone)]
enum InputItem {
    /// A crossterm event.
    Event(Event),
    /// A delay between events.
    Delay(Duration),
}

impl InputSequence {
    /// Create a new empty input sequence.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable real timing (respect delays).
    pub fn with_real_timing(mut self) -> Self {
        self.real_timing = true;
        self
    }

    /// Check if real timing is enabled.
    pub fn uses_real_timing(&self) -> bool {
        self.real_timing
    }

    /// Add a key event.
    pub fn key(&mut self, key: KeyCode) -> &mut Self {
        self.key_mod(key, KeyModifiers::NONE)
    }

    /// Add a key event with modifiers.
    pub fn key_mod(&mut self, key: KeyCode, modifiers: KeyModifiers) -> &mut Self {
        let event = KeyEvent::new(key, modifiers);
        self.events.push(InputItem::Event(Event::Key(event)));
        self
    }

    /// Add a character key event.
    pub fn char(&mut self, c: char) -> &mut Self {
        self.key(KeyCode::Char(c))
    }

    /// Add a text string as character events.
    pub fn text(&mut self, s: &str) -> &mut Self {
        for c in s.chars() {
            self.char(c);
        }
        self
    }

    /// Add a delay.
    pub fn delay(&mut self, ms: u64) -> &mut Self {
        self.events.push(InputItem::Delay(Duration::from_millis(ms)));
        self
    }

    /// Add a Ctrl+key event.
    pub fn ctrl(&mut self, c: char) -> &mut Self {
        self.key_mod(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    /// Add an Alt+key event.
    pub fn alt(&mut self, c: char) -> &mut Self {
        self.key_mod(KeyCode::Char(c), KeyModifiers::ALT)
    }

    /// Add a Shift+key event.
    pub fn shift(&mut self, c: char) -> &mut Self {
        self.key_mod(KeyCode::Char(c), KeyModifiers::SHIFT)
    }

    /// Add an Enter key event.
    pub fn enter(&mut self) -> &mut Self {
        self.key(KeyCode::Enter)
    }

    /// Add an Escape key event.
    pub fn esc(&mut self) -> &mut Self {
        self.key(KeyCode::Esc)
    }

    /// Add a Tab key event.
    pub fn tab(&mut self) -> &mut Self {
        self.key(KeyCode::Tab)
    }

    /// Add a Shift+Tab key event.
    pub fn shift_tab(&mut self) -> &mut Self {
        self.key_mod(KeyCode::Tab, KeyModifiers::SHIFT)
    }

    /// Add a Backspace key event.
    pub fn backspace(&mut self) -> &mut Self {
        self.key(KeyCode::Backspace)
    }

    /// Add a Delete key event.
    pub fn delete(&mut self) -> &mut Self {
        self.key(KeyCode::Delete)
    }

    /// Add a Space key event.
    pub fn space(&mut self) -> &mut Self {
        self.key(KeyCode::Char(' '))
    }

    /// Add an Up arrow key event.
    pub fn up(&mut self) -> &mut Self {
        self.key(KeyCode::Up)
    }

    /// Add a Down arrow key event.
    pub fn down(&mut self) -> &mut Self {
        self.key(KeyCode::Down)
    }

    /// Add a Left arrow key event.
    pub fn left(&mut self) -> &mut Self {
        self.key(KeyCode::Left)
    }

    /// Add a Right arrow key event.
    pub fn right(&mut self) -> &mut Self {
        self.key(KeyCode::Right)
    }

    /// Add a Home key event.
    pub fn home(&mut self) -> &mut Self {
        self.key(KeyCode::Home)
    }

    /// Add an End key event.
    pub fn end(&mut self) -> &mut Self {
        self.key(KeyCode::End)
    }

    /// Add a PageUp key event.
    pub fn page_up(&mut self) -> &mut Self {
        self.key(KeyCode::PageUp)
    }

    /// Add a PageDown key event.
    pub fn page_down(&mut self) -> &mut Self {
        self.key(KeyCode::PageDown)
    }

    /// Add a function key event.
    pub fn f(&mut self, n: u8) -> &mut Self {
        self.key(KeyCode::F(n))
    }

    /// Add a mouse click event.
    pub fn click(&mut self, x: u16, y: u16) -> &mut Self {
        self.mouse_event(MouseEventKind::Down(MouseButton::Left), x, y)
    }

    /// Add a right-click event.
    pub fn right_click(&mut self, x: u16, y: u16) -> &mut Self {
        self.mouse_event(MouseEventKind::Down(MouseButton::Right), x, y)
    }

    /// Add a mouse drag event (from start to end).
    pub fn drag(&mut self, from: (u16, u16), to: (u16, u16)) -> &mut Self {
        self.mouse_event(MouseEventKind::Down(MouseButton::Left), from.0, from.1);

        // Generate drag events along the path
        let dx = (to.0 as i32 - from.0 as i32).signum();
        let dy = (to.1 as i32 - from.1 as i32).signum();
        let mut x = from.0 as i32;
        let mut y = from.1 as i32;

        while x != to.0 as i32 || y != to.1 as i32 {
            if x != to.0 as i32 {
                x += dx;
            }
            if y != to.1 as i32 {
                y += dy;
            }
            self.mouse_event(MouseEventKind::Drag(MouseButton::Left), x as u16, y as u16);
        }

        self.mouse_event(MouseEventKind::Up(MouseButton::Left), to.0, to.1);
        self
    }

    /// Add a scroll event.
    pub fn scroll(&mut self, x: u16, y: u16, delta: i16) -> &mut Self {
        let kind = if delta > 0 {
            MouseEventKind::ScrollUp
        } else {
            MouseEventKind::ScrollDown
        };

        let count = delta.unsigned_abs();
        for _ in 0..count {
            self.mouse_event(kind, x, y);
        }
        self
    }

    /// Add a mouse event.
    fn mouse_event(&mut self, kind: MouseEventKind, x: u16, y: u16) -> &mut Self {
        let event = MouseEvent {
            kind,
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        };
        self.events.push(InputItem::Event(Event::Mouse(event)));
        self
    }

    /// Get all events (excluding delays unless real_timing is enabled).
    pub fn events(&self) -> Vec<Event> {
        self.events
            .iter()
            .filter_map(|item| match item {
                InputItem::Event(e) => Some(e.clone()),
                InputItem::Delay(_) => None,
            })
            .collect()
    }

    /// Get all key events.
    pub fn key_events(&self) -> Vec<KeyEvent> {
        self.events
            .iter()
            .filter_map(|item| match item {
                InputItem::Event(Event::Key(e)) => Some(*e),
                _ => None,
            })
            .collect()
    }

    /// Get the total count of events (excluding delays).
    pub fn len(&self) -> usize {
        self.events
            .iter()
            .filter(|item| matches!(item, InputItem::Event(_)))
            .count()
    }

    /// Check if the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over the sequence items.
    pub fn iter(&self) -> impl Iterator<Item = SequenceItem<'_>> {
        self.events.iter().map(|item| match item {
            InputItem::Event(e) => SequenceItem::Event(e),
            InputItem::Delay(d) => SequenceItem::Delay(*d),
        })
    }

    /// Append another sequence.
    pub fn append(&mut self, other: &InputSequence) -> &mut Self {
        for item in &other.events {
            self.events.push(item.clone());
        }
        self
    }

    /// Repeat the current sequence n times.
    pub fn repeat(&mut self, n: usize) -> &mut Self {
        let events = self.events.clone();
        for _ in 0..n - 1 {
            for item in &events {
                self.events.push(item.clone());
            }
        }
        self
    }
}

/// An item yielded by the sequence iterator.
#[derive(Debug, Clone)]
pub enum SequenceItem<'a> {
    /// An event.
    Event(&'a Event),
    /// A delay.
    Delay(Duration),
}

/// Builder for creating common input patterns.
pub struct InputPatterns;

impl InputPatterns {
    /// Navigate down n rows and select.
    pub fn select_row(n: usize) -> InputSequence {
        let mut seq = InputSequence::new();
        for _ in 0..n {
            seq.down();
        }
        seq.enter();
        seq
    }

    /// Type text and confirm.
    pub fn type_and_confirm(text: &str) -> InputSequence {
        let mut seq = InputSequence::new();
        seq.text(text);
        seq.enter();
        seq
    }

    /// Cancel operation.
    pub fn cancel() -> InputSequence {
        let mut seq = InputSequence::new();
        seq.esc();
        seq
    }

    /// Navigate with vim-style keys.
    pub fn vim_navigate(direction: char, count: usize) -> InputSequence {
        let mut seq = InputSequence::new();
        for _ in 0..count {
            seq.char(direction);
        }
        seq
    }

    /// Common quit pattern (Ctrl+Q).
    pub fn quit() -> InputSequence {
        let mut seq = InputSequence::new();
        seq.ctrl('q');
        seq
    }

    /// Common save pattern (Ctrl+S).
    pub fn save() -> InputSequence {
        let mut seq = InputSequence::new();
        seq.ctrl('s');
        seq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sequence() {
        let mut seq = InputSequence::new();
        seq.text("hello").enter();

        assert_eq!(seq.len(), 6); // 5 chars + enter
    }

    #[test]
    fn test_modifiers() {
        let mut seq = InputSequence::new();
        seq.ctrl('s');

        let events = seq.key_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_navigation() {
        let mut seq = InputSequence::new();
        seq.down().down().down().enter();

        assert_eq!(seq.len(), 4);
    }

    #[test]
    fn test_delay() {
        let mut seq = InputSequence::new();
        seq.text("test").delay(100).enter();

        let items: Vec<_> = seq.iter().collect();
        assert!(matches!(items[4], SequenceItem::Delay(_)));
    }

    #[test]
    fn test_mouse_click() {
        let mut seq = InputSequence::new();
        seq.click(10, 5);

        let events = seq.events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], Event::Mouse(_)));
    }

    #[test]
    fn test_patterns() {
        let seq = InputPatterns::select_row(3);
        assert_eq!(seq.len(), 4); // 3 downs + enter

        let seq = InputPatterns::type_and_confirm("test");
        assert_eq!(seq.len(), 5); // 4 chars + enter
    }

    #[test]
    fn test_real_timing() {
        let seq = InputSequence::new().with_real_timing();
        assert!(seq.uses_real_timing());

        let seq = InputSequence::new();
        assert!(!seq.uses_real_timing());
    }

    #[test]
    fn test_append() {
        let mut seq1 = InputSequence::new();
        seq1.text("ab");

        let mut seq2 = InputSequence::new();
        seq2.text("cd");

        seq1.append(&seq2);
        assert_eq!(seq1.len(), 4);
    }

    #[test]
    fn test_repeat() {
        let mut seq = InputSequence::new();
        seq.down().repeat(3);

        assert_eq!(seq.len(), 3);
    }
}
