//! Snapshot testing for TUI applications.

use crate::diff::SnapshotDiff;
use crate::terminal::TestTerminal;
use crate::{TestError, TestResult, SNAPSHOT_FORMAT_VERSION};
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};
use ratatui::widgets::Widget;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A cell in a buffer snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellSnapshot {
    /// X position (column).
    pub x: u16,
    /// Y position (row).
    pub y: u16,
    /// The symbol/character at this cell.
    pub symbol: String,
    /// Foreground color.
    pub fg: SerializableColor,
    /// Background color.
    pub bg: SerializableColor,
    /// Style modifiers.
    pub modifiers: SerializableModifier,
}

/// Serializable wrapper for ratatui Color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SerializableColor {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb { r: u8, g: u8, b: u8 },
    Indexed(u8),
}

impl From<Color> for SerializableColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Reset => Self::Reset,
            Color::Black => Self::Black,
            Color::Red => Self::Red,
            Color::Green => Self::Green,
            Color::Yellow => Self::Yellow,
            Color::Blue => Self::Blue,
            Color::Magenta => Self::Magenta,
            Color::Cyan => Self::Cyan,
            Color::Gray => Self::Gray,
            Color::DarkGray => Self::DarkGray,
            Color::LightRed => Self::LightRed,
            Color::LightGreen => Self::LightGreen,
            Color::LightYellow => Self::LightYellow,
            Color::LightBlue => Self::LightBlue,
            Color::LightMagenta => Self::LightMagenta,
            Color::LightCyan => Self::LightCyan,
            Color::White => Self::White,
            Color::Rgb(r, g, b) => Self::Rgb { r, g, b },
            Color::Indexed(i) => Self::Indexed(i),
        }
    }
}

impl From<SerializableColor> for Color {
    fn from(color: SerializableColor) -> Self {
        match color {
            SerializableColor::Reset => Self::Reset,
            SerializableColor::Black => Self::Black,
            SerializableColor::Red => Self::Red,
            SerializableColor::Green => Self::Green,
            SerializableColor::Yellow => Self::Yellow,
            SerializableColor::Blue => Self::Blue,
            SerializableColor::Magenta => Self::Magenta,
            SerializableColor::Cyan => Self::Cyan,
            SerializableColor::Gray => Self::Gray,
            SerializableColor::DarkGray => Self::DarkGray,
            SerializableColor::LightRed => Self::LightRed,
            SerializableColor::LightGreen => Self::LightGreen,
            SerializableColor::LightYellow => Self::LightYellow,
            SerializableColor::LightBlue => Self::LightBlue,
            SerializableColor::LightMagenta => Self::LightMagenta,
            SerializableColor::LightCyan => Self::LightCyan,
            SerializableColor::White => Self::White,
            SerializableColor::Rgb { r, g, b } => Self::Rgb(r, g, b),
            SerializableColor::Indexed(i) => Self::Indexed(i),
        }
    }
}

/// Serializable wrapper for ratatui Modifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SerializableModifier {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underlined: bool,
    pub slow_blink: bool,
    pub rapid_blink: bool,
    pub reversed: bool,
    pub hidden: bool,
    pub crossed_out: bool,
}

impl From<Modifier> for SerializableModifier {
    fn from(modifier: Modifier) -> Self {
        Self {
            bold: modifier.contains(Modifier::BOLD),
            dim: modifier.contains(Modifier::DIM),
            italic: modifier.contains(Modifier::ITALIC),
            underlined: modifier.contains(Modifier::UNDERLINED),
            slow_blink: modifier.contains(Modifier::SLOW_BLINK),
            rapid_blink: modifier.contains(Modifier::RAPID_BLINK),
            reversed: modifier.contains(Modifier::REVERSED),
            hidden: modifier.contains(Modifier::HIDDEN),
            crossed_out: modifier.contains(Modifier::CROSSED_OUT),
        }
    }
}

impl From<SerializableModifier> for Modifier {
    fn from(m: SerializableModifier) -> Self {
        let mut modifier = Modifier::empty();
        if m.bold {
            modifier |= Modifier::BOLD;
        }
        if m.dim {
            modifier |= Modifier::DIM;
        }
        if m.italic {
            modifier |= Modifier::ITALIC;
        }
        if m.underlined {
            modifier |= Modifier::UNDERLINED;
        }
        if m.slow_blink {
            modifier |= Modifier::SLOW_BLINK;
        }
        if m.rapid_blink {
            modifier |= Modifier::RAPID_BLINK;
        }
        if m.reversed {
            modifier |= Modifier::REVERSED;
        }
        if m.hidden {
            modifier |= Modifier::HIDDEN;
        }
        if m.crossed_out {
            modifier |= Modifier::CROSSED_OUT;
        }
        modifier
    }
}

impl SerializableModifier {
    /// Get a list of modifier names that are set.
    pub fn names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.bold {
            names.push("Bold");
        }
        if self.dim {
            names.push("Dim");
        }
        if self.italic {
            names.push("Italic");
        }
        if self.underlined {
            names.push("Underlined");
        }
        if self.slow_blink {
            names.push("SlowBlink");
        }
        if self.rapid_blink {
            names.push("RapidBlink");
        }
        if self.reversed {
            names.push("Reversed");
        }
        if self.hidden {
            names.push("Hidden");
        }
        if self.crossed_out {
            names.push("CrossedOut");
        }
        names
    }
}

/// A complete buffer snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BufferSnapshot {
    /// Buffer width.
    pub width: u16,
    /// Buffer height.
    pub height: u16,
    /// All cells in the buffer.
    pub cells: Vec<CellSnapshot>,
    /// Format version for compatibility.
    pub version: u32,
}

impl BufferSnapshot {
    /// Create a snapshot from a ratatui Buffer.
    pub fn from_buffer(buffer: &Buffer) -> Self {
        let area = buffer.area;
        let mut cells = Vec::with_capacity((area.width * area.height) as usize);

        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                let cell = buffer.cell((x, y)).expect("cell should exist");
                cells.push(CellSnapshot {
                    x,
                    y,
                    symbol: cell.symbol().to_string(),
                    fg: cell.fg.into(),
                    bg: cell.bg.into(),
                    modifiers: cell.modifier.into(),
                });
            }
        }

        Self {
            width: area.width,
            height: area.height,
            cells,
            version: SNAPSHOT_FORMAT_VERSION,
        }
    }

    /// Get a cell at the given position.
    pub fn cell_at(&self, x: u16, y: u16) -> Option<&CellSnapshot> {
        self.cells.iter().find(|c| c.x == x && c.y == y)
    }

    /// Convert to a string representation for debugging.
    pub fn to_string_content(&self) -> String {
        let mut result = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(cell) = self.cell_at(x, y) {
                    result.push_str(&cell.symbol);
                } else {
                    result.push(' ');
                }
            }
            if y < self.height - 1 {
                result.push('\n');
            }
        }
        result
    }

    /// Serialize to bytes.
    pub fn to_bytes(&self) -> TestResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| TestError::Serialization(e.to_string()))
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> TestResult<Self> {
        bincode::deserialize(bytes).map_err(|e| TestError::Serialization(e.to_string()))
    }

    /// Serialize to JSON (for human-readable snapshots).
    pub fn to_json(&self) -> TestResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| TestError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> TestResult<Self> {
        serde_json::from_str(json).map_err(|e| TestError::Serialization(e.to_string()))
    }

    /// Compare with another snapshot and return differences.
    pub fn diff(&self, other: &BufferSnapshot) -> SnapshotDiff {
        SnapshotDiff::compare(self, other)
    }
}

/// A captured frame for later comparison.
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// Name of the checkpoint.
    pub name: String,
    /// The captured buffer snapshot.
    pub snapshot: BufferSnapshot,
}

/// Main snapshot testing interface.
pub struct SnapshotTest {
    /// Test name (used for snapshot file naming).
    name: String,
    /// Test terminal.
    terminal: TestTerminal,
    /// Directory for snapshot files.
    snapshots_dir: PathBuf,
    /// Captured frames during the test.
    frames: Vec<CapturedFrame>,
}

impl SnapshotTest {
    /// Create a new snapshot test.
    pub fn new(name: &str, width: u16, height: u16) -> Self {
        let snapshots_dir = PathBuf::from("tests/snapshots");
        Self {
            name: name.to_string(),
            terminal: TestTerminal::new(width, height),
            snapshots_dir,
            frames: Vec::new(),
        }
    }

    /// Set a custom snapshots directory.
    pub fn with_snapshots_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.snapshots_dir = dir.into();
        self
    }

    /// Get a reference to the terminal.
    pub fn terminal(&self) -> &TestTerminal {
        &self.terminal
    }

    /// Get a mutable reference to the terminal.
    pub fn terminal_mut(&mut self) -> &mut TestTerminal {
        &mut self.terminal
    }

    /// Render a widget and assert it matches the snapshot.
    pub fn assert_snapshot<W: Widget>(&mut self, widget: W) {
        self.terminal.draw(|frame| {
            frame.render_widget(widget, frame.area());
        });

        let actual = self.terminal.snapshot();
        let snapshot_path = self.snapshot_path(&self.name);

        if crate::should_update_snapshots() {
            self.save_snapshot(&snapshot_path, &actual).unwrap();
            return;
        }

        match self.load_snapshot(&snapshot_path) {
            Ok(expected) => {
                let diff = expected.diff(&actual);
                if diff.has_changes() {
                    panic!(
                        "Snapshot mismatch in \"{}\":\n{}",
                        self.name,
                        diff.format_report()
                    );
                }
            }
            Err(TestError::SnapshotNotFound(_)) => {
                panic!(
                    "Snapshot not found for \"{}\". Run with UPDATE_SNAPSHOTS=1 to create it.",
                    self.name
                );
            }
            Err(e) => panic!("Failed to load snapshot: {}", e),
        }
    }

    /// Update the snapshot for this widget.
    pub fn update_snapshot<W: Widget>(&mut self, widget: W) {
        self.terminal.draw(|frame| {
            frame.render_widget(widget, frame.area());
        });

        let snapshot = self.terminal.snapshot();
        let snapshot_path = self.snapshot_path(&self.name);
        self.save_snapshot(&snapshot_path, &snapshot).unwrap();
    }

    /// Assert current buffer matches a named checkpoint.
    pub fn assert_frame(&mut self, checkpoint_name: &str) {
        let actual = self.terminal.snapshot();
        let full_name = format!("{}_{}", self.name, checkpoint_name);
        let snapshot_path = self.snapshot_path(&full_name);

        // Store frame for later reference
        self.frames.push(CapturedFrame {
            name: checkpoint_name.to_string(),
            snapshot: actual.clone(),
        });

        if crate::should_update_snapshots() {
            self.save_snapshot(&snapshot_path, &actual).unwrap();
            return;
        }

        match self.load_snapshot(&snapshot_path) {
            Ok(expected) => {
                let diff = expected.diff(&actual);
                if diff.has_changes() {
                    panic!(
                        "Snapshot mismatch at checkpoint \"{}\" in \"{}\":\n{}",
                        checkpoint_name,
                        self.name,
                        diff.format_report()
                    );
                }
            }
            Err(TestError::SnapshotNotFound(_)) => {
                panic!(
                    "Snapshot not found for checkpoint \"{}\" in \"{}\". Run with UPDATE_SNAPSHOTS=1 to create it.",
                    checkpoint_name, self.name
                );
            }
            Err(e) => panic!("Failed to load snapshot: {}", e),
        }
    }

    /// Capture current frame without asserting.
    pub fn capture_frame(&mut self, checkpoint_name: &str) -> CapturedFrame {
        let snapshot = self.terminal.snapshot();
        let frame = CapturedFrame {
            name: checkpoint_name.to_string(),
            snapshot,
        };
        self.frames.push(frame.clone());
        frame
    }

    /// Get all captured frames.
    pub fn frames(&self) -> &[CapturedFrame] {
        &self.frames
    }

    /// Get the path for a snapshot file.
    fn snapshot_path(&self, name: &str) -> PathBuf {
        self.snapshots_dir.join(format!("{}.snap", name))
    }

    /// Save a snapshot to disk.
    fn save_snapshot(&self, path: &PathBuf, snapshot: &BufferSnapshot) -> TestResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = snapshot.to_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Load a snapshot from disk.
    fn load_snapshot(&self, path: &PathBuf) -> TestResult<BufferSnapshot> {
        if !path.exists() {
            return Err(TestError::SnapshotNotFound(
                path.display().to_string(),
            ));
        }
        let bytes = std::fs::read(path)?;
        BufferSnapshot::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn test_buffer_snapshot_creation() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 5, 2));
        buffer.set_string(0, 0, "Hello", ratatui::style::Style::default());
        buffer.set_string(0, 1, "World", ratatui::style::Style::default());

        let snapshot = BufferSnapshot::from_buffer(&buffer);
        assert_eq!(snapshot.width, 5);
        assert_eq!(snapshot.height, 2);
        assert_eq!(snapshot.cells.len(), 10);
    }

    #[test]
    fn test_snapshot_to_string() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 5, 2));
        buffer.set_string(0, 0, "Hello", ratatui::style::Style::default());
        buffer.set_string(0, 1, "World", ratatui::style::Style::default());

        let snapshot = BufferSnapshot::from_buffer(&buffer);
        assert_eq!(snapshot.to_string_content(), "Hello\nWorld");
    }

    #[test]
    fn test_snapshot_serialization() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 3, 1));
        buffer.set_string(0, 0, "Hi!", ratatui::style::Style::default());

        let snapshot = BufferSnapshot::from_buffer(&buffer);

        // Binary round-trip
        let bytes = snapshot.to_bytes().unwrap();
        let restored = BufferSnapshot::from_bytes(&bytes).unwrap();
        assert_eq!(snapshot, restored);

        // JSON round-trip
        let json = snapshot.to_json().unwrap();
        let restored = BufferSnapshot::from_json(&json).unwrap();
        assert_eq!(snapshot, restored);
    }

    #[test]
    fn test_serializable_color() {
        let color = Color::Rgb(255, 128, 64);
        let serializable: SerializableColor = color.into();
        let back: Color = serializable.into();
        assert_eq!(color, back);
    }

    #[test]
    fn test_serializable_modifier() {
        let modifier = Modifier::BOLD | Modifier::ITALIC;
        let serializable: SerializableModifier = modifier.into();
        assert!(serializable.bold);
        assert!(serializable.italic);
        assert!(!serializable.underlined);

        let back: Modifier = serializable.into();
        assert_eq!(modifier, back);
    }
}
