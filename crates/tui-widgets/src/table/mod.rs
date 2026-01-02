//! DataTable widget - sortable, filterable table with virtual scrolling.
//!
//! # Example
//!
//! ```ignore
//! use tui_widgets::{DataTable, Column, ColumnWidth};
//!
//! #[derive(Clone)]
//! struct User {
//!     name: String,
//!     age: u32,
//!     email: String,
//! }
//!
//! let columns = vec![
//!     Column::new("Name", |u: &User| u.name.clone().into())
//!         .sortable(true),
//!     Column::new("Age", |u: &User| (u.age as f64).into())
//!         .width(ColumnWidth::Fixed(8)),
//!     Column::new("Email", |u: &User| u.email.clone().into()),
//! ];
//!
//! let users = vec![
//!     User { name: "Alice".into(), age: 30, email: "alice@example.com".into() },
//!     User { name: "Bob".into(), age: 25, email: "bob@example.com".into() },
//! ];
//!
//! let table = DataTable::new(columns, users);
//! ```

mod cell;
mod column;
mod selection;
mod state;

pub use cell::CellContent;
pub use column::{AggregateFunc, Column, ColumnWidth};
pub use selection::Selection;
pub use state::TableState;

use crate::accessibility::{Accessible, SoundCue};
use crate::WidgetConfig;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, StatefulWidget, Widget};

use std::cmp::Ordering;
use std::marker::PhantomData;

/// Sort direction for columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    /// Toggle the sort direction.
    pub fn toggle(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// Data source for the table.
pub enum DataSource<T> {
    /// All data loaded in memory
    Vec(Vec<T>),
    /// Paginated/virtualized data provider
    Provider(Box<dyn DataProvider<T>>),
}

impl<T: Clone> DataSource<T> {
    /// Get the total number of rows.
    pub fn len(&self) -> usize {
        match self {
            Self::Vec(v) => v.len(),
            Self::Provider(p) => p.total_count(),
        }
    }

    /// Check if the data source is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a row by index.
    pub fn get(&self, index: usize) -> Option<T> {
        match self {
            Self::Vec(v) => v.get(index).cloned(),
            Self::Provider(p) => p.get_row(index),
        }
    }
}

/// Trait for paginated/virtualized data loading.
pub trait DataProvider<T>: Send + Sync {
    /// Get the total number of rows.
    fn total_count(&self) -> usize;

    /// Get a single row by index.
    fn get_row(&self, index: usize) -> Option<T>;

    /// Get a range of rows for display.
    fn get_range(&self, start: usize, count: usize) -> Vec<T>;

    /// Invalidate cache (called when data changes).
    fn invalidate(&mut self);
}

/// Sortable, filterable table with virtual scrolling.
pub struct DataTable<T: Clone> {
    /// Column definitions
    columns: Vec<Column<T>>,
    /// Data source
    data: DataSource<T>,
    /// Widget configuration
    config: WidgetConfig,
    /// Block wrapper
    block: Option<Block<'static>>,
    /// Sorted indices (if sorting is active)
    sorted_indices: Option<Vec<usize>>,
    /// Filtered indices (if filtering is active)
    filtered_indices: Option<Vec<usize>>,
    /// Clipboard format
    clipboard_format: ClipboardFormat,
    /// Callbacks
    on_select: Option<Box<dyn Fn(&T)>>,
    on_delete: Option<Box<dyn Fn(Vec<&T>)>>,
    on_edit: Option<Box<dyn Fn(&T, &str, String)>>,
    on_sort: Option<Box<dyn Fn(usize, SortDirection)>>,
    /// Marker for T
    _marker: PhantomData<T>,
}

/// Clipboard format for copy operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipboardFormat {
    /// Tab-separated values (Excel/Sheets compatible)
    #[default]
    Tsv,
    /// Comma-separated values
    Csv,
    /// JSON array
    Json,
    /// Markdown table
    Markdown,
}

impl<T: Clone> DataTable<T> {
    /// Create a new DataTable with columns and data.
    pub fn new(columns: Vec<Column<T>>, data: Vec<T>) -> Self {
        Self {
            columns,
            data: DataSource::Vec(data),
            config: WidgetConfig::default(),
            block: None,
            sorted_indices: None,
            filtered_indices: None,
            clipboard_format: ClipboardFormat::default(),
            on_select: None,
            on_delete: None,
            on_edit: None,
            on_sort: None,
            _marker: PhantomData,
        }
    }

    /// Create a new DataTable with a data provider.
    pub fn with_provider(columns: Vec<Column<T>>, provider: impl DataProvider<T> + 'static) -> Self {
        Self {
            columns,
            data: DataSource::Provider(Box::new(provider)),
            config: WidgetConfig::default(),
            block: None,
            sorted_indices: None,
            filtered_indices: None,
            clipboard_format: ClipboardFormat::default(),
            on_select: None,
            on_delete: None,
            on_edit: None,
            on_sort: None,
            _marker: PhantomData,
        }
    }

    /// Set the block wrapper.
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set compact mode.
    pub fn compact(mut self, compact: bool) -> Self {
        self.config.compact_mode = if compact {
            crate::CompactMode::Compact
        } else {
            crate::CompactMode::Comfortable
        };
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// Set clipboard format.
    pub fn clipboard_format(mut self, format: ClipboardFormat) -> Self {
        self.clipboard_format = format;
        self
    }

    /// Set selection callback.
    pub fn on_select(mut self, f: impl Fn(&T) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Set delete callback.
    pub fn on_delete(mut self, f: impl Fn(Vec<&T>) + 'static) -> Self {
        self.on_delete = Some(Box::new(f));
        self
    }

    /// Set edit callback.
    pub fn on_edit(mut self, f: impl Fn(&T, &str, String) + 'static) -> Self {
        self.on_edit = Some(Box::new(f));
        self
    }

    /// Set sort callback.
    pub fn on_sort(mut self, f: impl Fn(usize, SortDirection) + 'static) -> Self {
        self.on_sort = Some(Box::new(f));
        self
    }

    /// Get the number of rows.
    pub fn row_count(&self) -> usize {
        if let Some(ref filtered) = self.filtered_indices {
            filtered.len()
        } else {
            self.data.len()
        }
    }

    /// Get data index from display index.
    fn data_index(&self, display_index: usize) -> Option<usize> {
        if let Some(ref filtered) = self.filtered_indices {
            if let Some(ref sorted) = self.sorted_indices {
                filtered.get(display_index).and_then(|&i| sorted.get(i).copied())
            } else {
                filtered.get(display_index).copied()
            }
        } else if let Some(ref sorted) = self.sorted_indices {
            sorted.get(display_index).copied()
        } else {
            Some(display_index)
        }
    }

    /// Get a row by display index.
    pub fn get_row(&self, display_index: usize) -> Option<T> {
        self.data_index(display_index).and_then(|i| self.data.get(i))
    }

    /// Sort by column.
    pub fn sort_by(&mut self, column_index: usize, direction: SortDirection, state: &mut TableState) {
        if column_index >= self.columns.len() {
            return;
        }

        let col = &self.columns[column_index];
        if !col.sortable {
            return;
        }

        // Build sorted indices
        let mut indices: Vec<usize> = (0..self.data.len()).collect();

        indices.sort_by(|&a, &b| {
            let row_a = self.data.get(a);
            let row_b = self.data.get(b);

            match (row_a, row_b) {
                (Some(ra), Some(rb)) => {
                    let cell_a = (col.accessor)(&ra);
                    let cell_b = (col.accessor)(&rb);
                    let ord = cell_a.cmp(&cell_b);
                    if direction == SortDirection::Descending {
                        ord.reverse()
                    } else {
                        ord
                    }
                }
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (None, None) => Ordering::Equal,
            }
        });

        self.sorted_indices = Some(indices);
        state.sort_column = Some(column_index);
        state.sort_direction = direction;

        if let Some(ref callback) = self.on_sort {
            callback(column_index, direction);
        }
    }

    /// Filter rows by predicate.
    pub fn filter(&mut self, query: &str, state: &mut TableState) {
        if query.is_empty() {
            self.filtered_indices = None;
            state.filter = None;
            return;
        }

        let query_lower = query.to_lowercase();
        let mut indices = Vec::new();

        let source_len = if let Some(ref sorted) = self.sorted_indices {
            sorted.len()
        } else {
            self.data.len()
        };

        for i in 0..source_len {
            let data_idx = if let Some(ref sorted) = self.sorted_indices {
                sorted[i]
            } else {
                i
            };

            if let Some(row) = self.data.get(data_idx) {
                let matches = self.columns.iter().any(|col| {
                    if !col.filterable {
                        return false;
                    }
                    let cell = (col.accessor)(&row);
                    cell.to_string().to_lowercase().contains(&query_lower)
                });
                if matches {
                    indices.push(i);
                }
            }
        }

        self.filtered_indices = Some(indices);
        state.filter = Some(query.to_string());
    }

    /// Clear all filters and sorting.
    pub fn reset(&mut self, state: &mut TableState) {
        self.sorted_indices = None;
        self.filtered_indices = None;
        state.sort_column = None;
        state.filter = None;
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: KeyEvent, state: &mut TableState) -> bool {
        if self.config.disabled {
            return false;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection(-1, key.modifiers.contains(KeyModifiers::SHIFT), state);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection(1, key.modifiers.contains(KeyModifiers::SHIFT), state);
                true
            }
            KeyCode::PageUp => {
                self.move_selection(-10, key.modifiers.contains(KeyModifiers::SHIFT), state);
                true
            }
            KeyCode::PageDown => {
                self.move_selection(10, key.modifiers.contains(KeyModifiers::SHIFT), state);
                true
            }
            KeyCode::Home => {
                self.select_first(key.modifiers.contains(KeyModifiers::SHIFT), state);
                true
            }
            KeyCode::End => {
                self.select_last(key.modifiers.contains(KeyModifiers::SHIFT), state);
                true
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.select_all(state);
                true
            }
            KeyCode::Enter => {
                if let Selection::Single(idx) = state.selected {
                    if let Some(row) = self.get_row(idx) {
                        if let Some(ref callback) = self.on_select {
                            callback(&row);
                        }
                    }
                }
                true
            }
            KeyCode::Esc => {
                state.selected = Selection::None;
                state.edit_cell = None;
                true
            }
            _ => false,
        }
    }

    fn move_selection(&mut self, delta: isize, extend: bool, state: &mut TableState) {
        let row_count = self.row_count();
        if row_count == 0 {
            return;
        }

        let current = match state.selected {
            Selection::None => 0,
            Selection::Single(idx) => idx,
            Selection::Multi(ref set) => *set.iter().max().unwrap_or(&0),
        };

        let new_idx = if delta < 0 {
            current.saturating_sub((-delta) as usize)
        } else {
            (current + delta as usize).min(row_count - 1)
        };

        if extend {
            state.extend_selection(new_idx);
        } else {
            state.selected = Selection::Single(new_idx);
        }

        // Ensure visible
        self.ensure_visible(new_idx, state);
    }

    fn select_first(&mut self, extend: bool, state: &mut TableState) {
        if self.row_count() == 0 {
            return;
        }
        if extend {
            state.extend_selection(0);
        } else {
            state.selected = Selection::Single(0);
        }
        state.scroll_offset = 0;
    }

    fn select_last(&mut self, extend: bool, state: &mut TableState) {
        let row_count = self.row_count();
        if row_count == 0 {
            return;
        }
        let last = row_count - 1;
        if extend {
            state.extend_selection(last);
        } else {
            state.selected = Selection::Single(last);
        }
        self.ensure_visible(last, state);
    }

    fn select_all(&mut self, state: &mut TableState) {
        let row_count = self.row_count();
        if row_count == 0 {
            return;
        }
        state.selected = Selection::Multi((0..row_count).collect());
    }

    fn ensure_visible(&self, row: usize, state: &mut TableState) {
        // This would need the visible height, which we get during render
        // For now, basic implementation
        if row < state.scroll_offset {
            state.scroll_offset = row;
        }
    }

    /// Copy selected rows to clipboard format.
    pub fn copy_to_clipboard(&self, state: &TableState) -> String {
        let indices: Vec<usize> = match &state.selected {
            Selection::None => return String::new(),
            Selection::Single(idx) => vec![*idx],
            Selection::Multi(set) => {
                let mut v: Vec<_> = set.iter().copied().collect();
                v.sort();
                v
            }
        };

        let rows: Vec<Vec<String>> = indices
            .iter()
            .filter_map(|&idx| {
                self.get_row(idx).map(|row| {
                    self.columns
                        .iter()
                        .map(|col| (col.accessor)(&row).to_string())
                        .collect()
                })
            })
            .collect();

        match self.clipboard_format {
            ClipboardFormat::Tsv => rows
                .iter()
                .map(|r| r.join("\t"))
                .collect::<Vec<_>>()
                .join("\n"),
            ClipboardFormat::Csv => rows
                .iter()
                .map(|r| {
                    r.iter()
                        .map(|c| {
                            if c.contains(',') || c.contains('"') || c.contains('\n') {
                                format!("\"{}\"", c.replace('"', "\"\""))
                            } else {
                                c.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .collect::<Vec<_>>()
                .join("\n"),
            ClipboardFormat::Json => {
                let objects: Vec<_> = rows
                    .iter()
                    .map(|row| {
                        self.columns
                            .iter()
                            .zip(row.iter())
                            .map(|(col, val)| format!("\"{}\":\"{}\"", col.header, val.replace('"', "\\\"")))
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .map(|obj| format!("{{{}}}", obj))
                    .collect();
                format!("[{}]", objects.join(","))
            }
            ClipboardFormat::Markdown => {
                let mut result = String::new();
                // Header
                let headers: Vec<_> = self.columns.iter().map(|c| c.header.clone()).collect();
                result.push_str(&format!("| {} |\n", headers.join(" | ")));
                result.push_str(&format!(
                    "| {} |\n",
                    headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")
                ));
                // Rows
                for row in &rows {
                    result.push_str(&format!("| {} |\n", row.join(" | ")));
                }
                result
            }
        }
    }
}

impl<T: Clone> StatefulWidget for DataTable<T> {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render block if present
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.width < 3 || inner.height < 2 {
            return;
        }

        let row_count = self.row_count();
        if row_count == 0 {
            // Empty state
            let msg = "No data";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(Color::DarkGray));
            return;
        }

        // Calculate column widths
        let total_width = inner.width as usize;
        let column_widths: Vec<u16> = self.calculate_column_widths(total_width);

        // Header row
        let mut x = inner.x;
        for (i, col) in self.columns.iter().enumerate() {
            let width = column_widths.get(i).copied().unwrap_or(0);
            if width == 0 {
                continue;
            }

            let mut header = col.header.clone();
            if state.sort_column == Some(i) {
                let arrow = match state.sort_direction {
                    SortDirection::Ascending => " \u{25b2}",
                    SortDirection::Descending => " \u{25bc}",
                };
                header.push_str(arrow);
            }

            let style = Style::default().add_modifier(Modifier::BOLD);
            let display = truncate_with_ellipsis(&header, width as usize);
            buf.set_string(x, inner.y, &display, style);
            x += width + 1; // +1 for separator
        }

        // Data rows
        let visible_height = (inner.height - 1) as usize; // -1 for header
        state.visible_rows = visible_height;

        // Adjust scroll if needed
        if let Selection::Single(idx) = state.selected {
            if idx >= state.scroll_offset + visible_height {
                state.scroll_offset = idx.saturating_sub(visible_height - 1);
            } else if idx < state.scroll_offset {
                state.scroll_offset = idx;
            }
        }

        for row_offset in 0..visible_height {
            let display_idx = state.scroll_offset + row_offset;
            if display_idx >= row_count {
                break;
            }

            let y = inner.y + 1 + row_offset as u16;
            let row = match self.get_row(display_idx) {
                Some(r) => r,
                None => continue,
            };

            // Determine row style
            let is_selected = state.selected.contains(display_idx);
            let row_style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else if row_offset % 2 == 0 {
                Style::default()
            } else {
                Style::default().bg(Color::Rgb(30, 30, 30))
            };

            // Clear row background
            for col_x in inner.x..inner.x + inner.width {
                buf[(col_x, y)].set_style(row_style);
            }

            // Render cells
            let mut x = inner.x;
            for (col_idx, col) in self.columns.iter().enumerate() {
                let width = column_widths.get(col_idx).copied().unwrap_or(0);
                if width == 0 {
                    continue;
                }

                let cell = (col.accessor)(&row);
                let text = cell.to_string();
                let display = truncate_with_ellipsis(&text, width as usize);
                buf.set_string(x, y, &display, row_style);
                x += width + 1;
            }
        }

        // Scrollbar (simple)
        if row_count > visible_height {
            let scrollbar_height = (visible_height as f32 / row_count as f32 * inner.height as f32)
                .max(1.0) as u16;
            let scrollbar_pos = (state.scroll_offset as f32 / (row_count - visible_height) as f32
                * (inner.height - scrollbar_height) as f32) as u16;

            for y in inner.y..inner.y + inner.height {
                let ch = if y >= inner.y + scrollbar_pos && y < inner.y + scrollbar_pos + scrollbar_height
                {
                    '\u{2588}' // Full block
                } else {
                    '\u{2591}' // Light shade
                };
                buf[(inner.x + inner.width - 1, y)].set_char(ch);
            }
        }
    }
}

impl<T: Clone> DataTable<T> {
    fn calculate_column_widths(&self, total_width: usize) -> Vec<u16> {
        let col_count = self.columns.len();
        if col_count == 0 {
            return vec![];
        }

        // Reserve space for separators
        let separators = col_count.saturating_sub(1);
        let available = total_width.saturating_sub(separators);

        let mut widths = vec![0u16; col_count];
        let mut remaining = available;
        let mut flex_count = 0;

        // First pass: fixed and percentage widths
        for (i, col) in self.columns.iter().enumerate() {
            match col.width {
                ColumnWidth::Fixed(w) => {
                    widths[i] = w;
                    remaining = remaining.saturating_sub(w as usize);
                }
                ColumnWidth::Percentage(p) => {
                    let w = (available as f32 * p / 100.0) as u16;
                    widths[i] = w;
                    remaining = remaining.saturating_sub(w as usize);
                }
                ColumnWidth::Flex(_) => flex_count += 1,
            }
        }

        // Second pass: distribute remaining to flex columns
        if flex_count > 0 {
            let total_flex: u16 = self
                .columns
                .iter()
                .filter_map(|c| match c.width {
                    ColumnWidth::Flex(f) => Some(f),
                    _ => None,
                })
                .sum();

            for (i, col) in self.columns.iter().enumerate() {
                if let ColumnWidth::Flex(f) = col.width {
                    let w = (remaining as f32 * f as f32 / total_flex as f32) as u16;
                    widths[i] = w;
                }
            }
        }

        widths
    }
}

impl<T: Clone> Accessible for DataTable<T> {
    fn aria_role(&self) -> &str {
        "grid"
    }

    fn aria_label(&self) -> String {
        format!("Data table with {} rows", self.data.len())
    }

    fn announce(&self, _message: &str) {
        // Would integrate with announcement buffer
    }

    fn play_sound(&self, _sound: SoundCue) {
        // Would integrate with sound system
    }
}

/// Truncate a string with ellipsis if too long.
fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    if s.len() <= max_len {
        return s.to_string();
    }
    if max_len <= 3 {
        return s.chars().take(max_len).collect();
    }
    format!("{}...", &s[..max_len - 3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestRow {
        name: String,
        value: i32,
    }

    fn test_columns() -> Vec<Column<TestRow>> {
        vec![
            Column::new("Name", |r: &TestRow| r.name.clone().into()),
            Column::new("Value", |r: &TestRow| (r.value as f64).into()),
        ]
    }

    fn test_data() -> Vec<TestRow> {
        vec![
            TestRow { name: "Alice".into(), value: 10 },
            TestRow { name: "Bob".into(), value: 20 },
            TestRow { name: "Charlie".into(), value: 15 },
        ]
    }

    #[test]
    fn test_table_creation() {
        let table = DataTable::new(test_columns(), test_data());
        assert_eq!(table.row_count(), 3);
    }

    #[test]
    fn test_selection() {
        let table = DataTable::new(test_columns(), test_data());
        let mut state = TableState::default();

        state.selected = Selection::Single(1);
        assert!(state.selected.contains(1));
        assert!(!state.selected.contains(0));
    }

    #[test]
    fn test_clipboard_tsv() {
        let table = DataTable::new(test_columns(), test_data());
        let mut state = TableState::default();
        state.selected = Selection::Single(0);

        let clipboard = table.copy_to_clipboard(&state);
        assert_eq!(clipboard, "Alice\t10");
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
        assert_eq!(truncate_with_ellipsis("hello world", 8), "hello...");
        assert_eq!(truncate_with_ellipsis("hi", 2), "hi");
    }
}
