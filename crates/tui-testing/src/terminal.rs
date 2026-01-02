//! Virtual terminal for headless testing.

use crate::snapshot::BufferSnapshot;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Terminal;

/// A virtual terminal for testing TUI applications.
pub struct TestTerminal {
    terminal: Terminal<TestBackend>,
    frame_history: Vec<BufferSnapshot>,
}

impl TestTerminal {
    /// Create a new test terminal with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend).expect("Failed to create terminal");
        Self {
            terminal,
            frame_history: Vec::new(),
        }
    }

    /// Get the current terminal size.
    pub fn size(&self) -> Rect {
        self.terminal.size().expect("Failed to get terminal size")
    }

    /// Get the width of the terminal.
    pub fn width(&self) -> u16 {
        self.size().width
    }

    /// Get the height of the terminal.
    pub fn height(&self) -> u16 {
        self.size().height
    }

    /// Draw to the terminal.
    pub fn draw<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(f).expect("Failed to draw to terminal");
    }

    /// Get a reference to the current buffer.
    pub fn buffer(&self) -> &Buffer {
        self.terminal.backend().buffer()
    }

    /// Get the buffer content as a string.
    pub fn to_string(&self) -> String {
        let buffer = self.buffer();
        let area = buffer.area;
        let mut result = String::new();

        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buffer.cell((x, y)) {
                    result.push_str(cell.symbol());
                }
            }
            if y < area.y + area.height - 1 {
                result.push('\n');
            }
        }

        result
    }

    /// Assert that the buffer matches the expected string.
    pub fn assert_buffer(&self, expected: &str) {
        let actual = self.to_string();
        if actual != expected {
            panic!(
                "Buffer mismatch:\n--- Expected ---\n{}\n--- Actual ---\n{}\n",
                expected, actual
            );
        }
    }

    /// Assert that the buffer contains the given substring.
    pub fn assert_contains(&self, needle: &str) {
        let content = self.to_string();
        if !content.contains(needle) {
            panic!(
                "Buffer does not contain \"{}\":\n{}",
                needle, content
            );
        }
    }

    /// Assert that the buffer does not contain the given substring.
    pub fn assert_not_contains(&self, needle: &str) {
        let content = self.to_string();
        if content.contains(needle) {
            panic!(
                "Buffer unexpectedly contains \"{}\":\n{}",
                needle, content
            );
        }
    }

    /// Get the content of a specific line.
    pub fn line(&self, line_num: u16) -> String {
        let buffer = self.buffer();
        let area = buffer.area;
        let y = area.y + line_num;

        if y >= area.y + area.height {
            return String::new();
        }

        let mut result = String::new();
        for x in area.x..area.x + area.width {
            if let Some(cell) = buffer.cell((x, y)) {
                result.push_str(cell.symbol());
            }
        }

        result.trim_end().to_string()
    }

    /// Get the content of a specific cell.
    pub fn cell(&self, x: u16, y: u16) -> Option<String> {
        self.buffer().cell((x, y)).map(|c| c.symbol().to_string())
    }

    /// Create a structured snapshot of the current buffer.
    pub fn snapshot(&self) -> BufferSnapshot {
        BufferSnapshot::from_buffer(self.buffer())
    }

    /// Store the current frame in history.
    pub fn capture_frame(&mut self) -> usize {
        let snapshot = self.snapshot();
        self.frame_history.push(snapshot);
        self.frame_history.len() - 1
    }

    /// Get a frame from history.
    pub fn frame(&self, index: usize) -> Option<&BufferSnapshot> {
        self.frame_history.get(index)
    }

    /// Get the most recent frame.
    pub fn last_frame(&self) -> Option<&BufferSnapshot> {
        self.frame_history.last()
    }

    /// Get all captured frames.
    pub fn frame_history(&self) -> &[BufferSnapshot] {
        &self.frame_history
    }

    /// Clear the frame history.
    pub fn clear_history(&mut self) {
        self.frame_history.clear();
    }

    /// Resize the terminal.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.terminal
            .backend_mut()
            .resize(width, height);
    }

    /// Clear the terminal.
    pub fn clear(&mut self) {
        self.terminal.clear().expect("Failed to clear terminal");
    }

    /// Get mutable access to the underlying terminal.
    pub fn inner_mut(&mut self) -> &mut Terminal<TestBackend> {
        &mut self.terminal
    }

    /// Get access to the underlying terminal.
    pub fn inner(&self) -> &Terminal<TestBackend> {
        &self.terminal
    }
}

impl Default for TestTerminal {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::{Block, Borders, Paragraph};

    #[test]
    fn test_terminal_creation() {
        let terminal = TestTerminal::new(80, 24);
        assert_eq!(terminal.width(), 80);
        assert_eq!(terminal.height(), 24);
    }

    #[test]
    fn test_draw_and_read() {
        let mut terminal = TestTerminal::new(20, 5);

        terminal.draw(|frame| {
            let para = Paragraph::new("Hello, World!");
            frame.render_widget(para, frame.area());
        });

        assert!(terminal.to_string().contains("Hello, World!"));
    }

    #[test]
    fn test_line_access() {
        let mut terminal = TestTerminal::new(20, 5);

        terminal.draw(|frame| {
            let para = Paragraph::new("Line 1\nLine 2\nLine 3");
            frame.render_widget(para, frame.area());
        });

        assert_eq!(terminal.line(0), "Line 1");
        assert_eq!(terminal.line(1), "Line 2");
        assert_eq!(terminal.line(2), "Line 3");
    }

    #[test]
    fn test_assert_contains() {
        let mut terminal = TestTerminal::new(20, 5);

        terminal.draw(|frame| {
            let para = Paragraph::new("Test Content");
            frame.render_widget(para, frame.area());
        });

        terminal.assert_contains("Test");
        terminal.assert_not_contains("Missing");
    }

    #[test]
    fn test_snapshot() {
        let mut terminal = TestTerminal::new(10, 2);

        terminal.draw(|frame| {
            let para = Paragraph::new("Hi");
            frame.render_widget(para, frame.area());
        });

        let snapshot = terminal.snapshot();
        assert_eq!(snapshot.width, 10);
        assert_eq!(snapshot.height, 2);
    }

    #[test]
    fn test_frame_history() {
        let mut terminal = TestTerminal::new(10, 2);

        terminal.draw(|frame| {
            let para = Paragraph::new("Frame 1");
            frame.render_widget(para, frame.area());
        });
        terminal.capture_frame();

        terminal.draw(|frame| {
            let para = Paragraph::new("Frame 2");
            frame.render_widget(para, frame.area());
        });
        terminal.capture_frame();

        assert_eq!(terminal.frame_history().len(), 2);
        assert!(terminal.frame(0).is_some());
        assert!(terminal.frame(1).is_some());
        assert!(terminal.frame(2).is_none());
    }

    #[test]
    fn test_resize() {
        let mut terminal = TestTerminal::new(80, 24);
        assert_eq!(terminal.width(), 80);
        assert_eq!(terminal.height(), 24);

        terminal.resize(40, 12);
        assert_eq!(terminal.width(), 40);
        assert_eq!(terminal.height(), 12);
    }

    #[test]
    fn test_with_borders() {
        let mut terminal = TestTerminal::new(20, 5);

        terminal.draw(|frame| {
            let block = Block::default()
                .title("Title")
                .borders(Borders::ALL);
            frame.render_widget(block, frame.area());
        });

        let content = terminal.to_string();
        assert!(content.contains("Title"));
    }
}
