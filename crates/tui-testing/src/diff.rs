//! Structured diff for buffer snapshots.

use crate::snapshot::{BufferSnapshot, CellSnapshot, SerializableColor};

/// Changes detected in a single cell.
#[derive(Debug, Clone, Default)]
pub struct CellChanges {
    /// Whether the symbol changed.
    pub symbol_changed: bool,
    /// Whether the foreground color changed.
    pub fg_changed: bool,
    /// Whether the background color changed.
    pub bg_changed: bool,
    /// Whether the modifiers changed.
    pub modifiers_changed: bool,
}

impl CellChanges {
    /// Check if any changes occurred.
    pub fn any(&self) -> bool {
        self.symbol_changed || self.fg_changed || self.bg_changed || self.modifiers_changed
    }

    /// Count how many things changed.
    pub fn count(&self) -> usize {
        let mut count = 0;
        if self.symbol_changed {
            count += 1;
        }
        if self.fg_changed {
            count += 1;
        }
        if self.bg_changed {
            count += 1;
        }
        if self.modifiers_changed {
            count += 1;
        }
        count
    }
}

/// Difference between expected and actual cell.
#[derive(Debug, Clone)]
pub struct CellDiff {
    /// Position of the cell.
    pub position: (u16, u16),
    /// Expected cell value.
    pub expected: CellSnapshot,
    /// Actual cell value.
    pub actual: CellSnapshot,
    /// What changed.
    pub changes: CellChanges,
}

impl CellDiff {
    /// Compare two cells and create a diff if they differ.
    pub fn compare(expected: &CellSnapshot, actual: &CellSnapshot) -> Option<Self> {
        let changes = CellChanges {
            symbol_changed: expected.symbol != actual.symbol,
            fg_changed: expected.fg != actual.fg,
            bg_changed: expected.bg != actual.bg,
            modifiers_changed: expected.modifiers != actual.modifiers,
        };

        if changes.any() {
            Some(Self {
                position: (expected.x, expected.y),
                expected: expected.clone(),
                actual: actual.clone(),
                changes,
            })
        } else {
            None
        }
    }

    /// Format this diff for display.
    pub fn format(&self) -> String {
        let mut parts = Vec::new();

        if self.changes.symbol_changed {
            parts.push(format!(
                "symbol '{}' → '{}'",
                self.expected.symbol, self.actual.symbol
            ));
        }

        if self.changes.fg_changed {
            parts.push(format!(
                "fg {} → {}",
                format_color(&self.expected.fg),
                format_color(&self.actual.fg)
            ));
        }

        if self.changes.bg_changed {
            parts.push(format!(
                "bg {} → {}",
                format_color(&self.expected.bg),
                format_color(&self.actual.bg)
            ));
        }

        if self.changes.modifiers_changed {
            parts.push(format!(
                "modifiers [{}] → [{}]",
                self.expected.modifiers.names().join(", "),
                self.actual.modifiers.names().join(", ")
            ));
        }

        format!(
            "Cell ({}, {}): {}",
            self.position.0,
            self.position.1,
            parts.join(", ")
        )
    }
}

/// Complete diff between two buffer snapshots.
#[derive(Debug, Clone, Default)]
pub struct SnapshotDiff {
    /// Cells that changed.
    pub changed_cells: Vec<CellDiff>,
    /// Cells that were added (in actual but not expected).
    pub added_cells: Vec<CellSnapshot>,
    /// Cells that were removed (in expected but not actual).
    pub removed_cells: Vec<CellSnapshot>,
    /// Whether the dimensions changed.
    pub size_changed: Option<((u16, u16), (u16, u16))>,
}

impl SnapshotDiff {
    /// Compare two buffer snapshots.
    pub fn compare(expected: &BufferSnapshot, actual: &BufferSnapshot) -> Self {
        let mut diff = Self::default();

        // Check size
        if expected.width != actual.width || expected.height != actual.height {
            diff.size_changed = Some((
                (expected.width, expected.height),
                (actual.width, actual.height),
            ));
        }

        // Compare cells at same positions
        for expected_cell in &expected.cells {
            if let Some(actual_cell) = actual.cell_at(expected_cell.x, expected_cell.y) {
                if let Some(cell_diff) = CellDiff::compare(expected_cell, actual_cell) {
                    diff.changed_cells.push(cell_diff);
                }
            } else {
                diff.removed_cells.push(expected_cell.clone());
            }
        }

        // Find added cells
        for actual_cell in &actual.cells {
            if expected.cell_at(actual_cell.x, actual_cell.y).is_none() {
                diff.added_cells.push(actual_cell.clone());
            }
        }

        diff
    }

    /// Check if there are any changes.
    pub fn has_changes(&self) -> bool {
        self.size_changed.is_some()
            || !self.changed_cells.is_empty()
            || !self.added_cells.is_empty()
            || !self.removed_cells.is_empty()
    }

    /// Get total number of differences.
    pub fn count(&self) -> usize {
        let size_diff = if self.size_changed.is_some() { 1 } else { 0 };
        size_diff + self.changed_cells.len() + self.added_cells.len() + self.removed_cells.len()
    }

    /// Format a human-readable report.
    pub fn format_report(&self) -> String {
        if !self.has_changes() {
            return "No differences found.".to_string();
        }

        let mut lines = Vec::new();

        if let Some(((ew, eh), (aw, ah))) = self.size_changed {
            lines.push(format!("Size changed: {}x{} → {}x{}", ew, eh, aw, ah));
        }

        // Show changed cells (limit to first 20 for readability)
        let changed_count = self.changed_cells.len();
        for (i, diff) in self.changed_cells.iter().take(20).enumerate() {
            lines.push(format!("  {}", diff.format()));
            if i == 19 && changed_count > 20 {
                lines.push(format!("  ... and {} more changed cells", changed_count - 20));
            }
        }

        // Show added cells
        if !self.added_cells.is_empty() {
            if self.added_cells.len() <= 10 {
                for cell in &self.added_cells {
                    lines.push(format!(
                        "  Cell ({}, {}): added '{}' (fg: {}, bg: {})",
                        cell.x,
                        cell.y,
                        cell.symbol,
                        format_color(&cell.fg),
                        format_color(&cell.bg)
                    ));
                }
            } else {
                // Summarize region
                let min_x = self.added_cells.iter().map(|c| c.x).min().unwrap();
                let max_x = self.added_cells.iter().map(|c| c.x).max().unwrap();
                let min_y = self.added_cells.iter().map(|c| c.y).min().unwrap();
                let max_y = self.added_cells.iter().map(|c| c.y).max().unwrap();
                lines.push(format!(
                    "  {} cells added in region ({},{})-({},{})",
                    self.added_cells.len(),
                    min_x,
                    min_y,
                    max_x,
                    max_y
                ));
            }
        }

        // Show removed cells
        if !self.removed_cells.is_empty() {
            if self.removed_cells.len() <= 10 {
                for cell in &self.removed_cells {
                    lines.push(format!(
                        "  Cell ({}, {}): removed '{}' (fg: {}, bg: {})",
                        cell.x,
                        cell.y,
                        cell.symbol,
                        format_color(&cell.fg),
                        format_color(&cell.bg)
                    ));
                }
            } else {
                let min_x = self.removed_cells.iter().map(|c| c.x).min().unwrap();
                let max_x = self.removed_cells.iter().map(|c| c.x).max().unwrap();
                let min_y = self.removed_cells.iter().map(|c| c.y).min().unwrap();
                let max_y = self.removed_cells.iter().map(|c| c.y).max().unwrap();
                lines.push(format!(
                    "  {} cells removed in region ({},{})-({},{})",
                    self.removed_cells.len(),
                    min_x,
                    min_y,
                    max_x,
                    max_y
                ));
            }
        }

        lines.join("\n")
    }
}

/// Format a color for display.
fn format_color(color: &SerializableColor) -> String {
    match color {
        SerializableColor::Reset => "reset".to_string(),
        SerializableColor::Black => "black".to_string(),
        SerializableColor::Red => "red".to_string(),
        SerializableColor::Green => "green".to_string(),
        SerializableColor::Yellow => "yellow".to_string(),
        SerializableColor::Blue => "blue".to_string(),
        SerializableColor::Magenta => "magenta".to_string(),
        SerializableColor::Cyan => "cyan".to_string(),
        SerializableColor::Gray => "gray".to_string(),
        SerializableColor::DarkGray => "dark_gray".to_string(),
        SerializableColor::LightRed => "light_red".to_string(),
        SerializableColor::LightGreen => "light_green".to_string(),
        SerializableColor::LightYellow => "light_yellow".to_string(),
        SerializableColor::LightBlue => "light_blue".to_string(),
        SerializableColor::LightMagenta => "light_magenta".to_string(),
        SerializableColor::LightCyan => "light_cyan".to_string(),
        SerializableColor::White => "white".to_string(),
        SerializableColor::Rgb { r, g, b } => format!("#{:02x}{:02x}{:02x}", r, g, b),
        SerializableColor::Indexed(i) => format!("indexed({})", i),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Style;

    fn create_snapshot(content: &str, width: u16, height: u16) -> BufferSnapshot {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
        for (y, line) in content.lines().enumerate() {
            buffer.set_string(0, y as u16, line, Style::default());
        }
        BufferSnapshot::from_buffer(&buffer)
    }

    #[test]
    fn test_no_diff() {
        let snap1 = create_snapshot("Hello", 5, 1);
        let snap2 = create_snapshot("Hello", 5, 1);
        let diff = snap1.diff(&snap2);
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_symbol_diff() {
        let snap1 = create_snapshot("Hello", 5, 1);
        let snap2 = create_snapshot("Hella", 5, 1);
        let diff = snap1.diff(&snap2);

        assert!(diff.has_changes());
        assert_eq!(diff.changed_cells.len(), 1);
        assert!(diff.changed_cells[0].changes.symbol_changed);
        assert_eq!(diff.changed_cells[0].position, (4, 0));
    }

    #[test]
    fn test_size_diff() {
        let snap1 = create_snapshot("Hello", 5, 1);
        let snap2 = create_snapshot("Hello", 6, 1);
        let diff = snap1.diff(&snap2);

        assert!(diff.has_changes());
        assert!(diff.size_changed.is_some());
        assert_eq!(diff.size_changed, Some(((5, 1), (6, 1))));
    }

    #[test]
    fn test_diff_report_format() {
        let snap1 = create_snapshot("ABC", 3, 1);
        let snap2 = create_snapshot("XYZ", 3, 1);
        let diff = snap1.diff(&snap2);

        let report = diff.format_report();
        assert!(report.contains("Cell"));
        assert!(report.contains("symbol"));
    }

    #[test]
    fn test_cell_changes() {
        let changes = CellChanges {
            symbol_changed: true,
            fg_changed: true,
            bg_changed: false,
            modifiers_changed: false,
        };

        assert!(changes.any());
        assert_eq!(changes.count(), 2);
    }
}
