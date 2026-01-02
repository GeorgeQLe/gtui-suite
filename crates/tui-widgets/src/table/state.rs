//! State management for DataTable.

use super::{Selection, SortDirection};

/// State for DataTable widget.
#[derive(Debug, Clone, Default)]
pub struct TableState {
    /// Currently selected row(s)
    pub selected: Selection,
    /// Column being sorted
    pub sort_column: Option<usize>,
    /// Sort direction
    pub sort_direction: SortDirection,
    /// Current filter query
    pub filter: Option<String>,
    /// Scroll offset (first visible row)
    pub scroll_offset: usize,
    /// Cell being edited (row, col)
    pub edit_cell: Option<(usize, usize)>,
    /// Number of visible rows (set during render)
    pub(crate) visible_rows: usize,
    /// Anchor for range selection
    selection_anchor: Option<usize>,
}

impl TableState {
    /// Create a new empty table state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected row index (primary selection).
    pub fn selected(&self) -> Option<usize> {
        self.selected.primary()
    }

    /// Select a specific row.
    pub fn select(&mut self, index: usize) {
        self.selected = Selection::Single(index);
        self.selection_anchor = Some(index);
    }

    /// Clear selection.
    pub fn deselect(&mut self) {
        self.selected = Selection::None;
        self.selection_anchor = None;
    }

    /// Extend selection to include a new row (for Shift+Click/Arrow).
    pub fn extend_selection(&mut self, to: usize) {
        let anchor = self.selection_anchor.unwrap_or(
            self.selected.primary().unwrap_or(0)
        );
        self.selection_anchor = Some(anchor);
        self.selected.select_range(anchor, to);
    }

    /// Toggle selection of a row (for Ctrl+Click).
    pub fn toggle_selection(&mut self, index: usize) {
        self.selected.toggle(index);
        self.selection_anchor = Some(index);
    }

    /// Check if a row is selected.
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected.contains(index)
    }

    /// Get all selected indices.
    pub fn selected_indices(&self) -> Vec<usize> {
        self.selected.indices()
    }

    /// Select all rows in range.
    pub fn select_all(&mut self, count: usize) {
        if count == 0 {
            self.selected = Selection::None;
        } else {
            self.selected = Selection::Multi((0..count).collect());
        }
    }

    /// Start editing a cell.
    pub fn start_edit(&mut self, row: usize, col: usize) {
        self.edit_cell = Some((row, col));
    }

    /// Cancel editing.
    pub fn cancel_edit(&mut self) {
        self.edit_cell = None;
    }

    /// Check if currently editing.
    pub fn is_editing(&self) -> bool {
        self.edit_cell.is_some()
    }

    /// Scroll to make a row visible.
    pub fn scroll_to(&mut self, row: usize, visible_height: usize) {
        if row < self.scroll_offset {
            self.scroll_offset = row;
        } else if row >= self.scroll_offset + visible_height {
            self.scroll_offset = row.saturating_sub(visible_height - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_selection() {
        let mut state = TableState::new();

        state.select(5);
        assert_eq!(state.selected(), Some(5));
        assert!(state.is_selected(5));
        assert!(!state.is_selected(4));

        state.deselect();
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn test_state_extend_selection() {
        let mut state = TableState::new();

        state.select(2);
        state.extend_selection(5);

        assert!(state.is_selected(2));
        assert!(state.is_selected(3));
        assert!(state.is_selected(4));
        assert!(state.is_selected(5));
        assert!(!state.is_selected(1));
        assert!(!state.is_selected(6));
    }

    #[test]
    fn test_state_toggle_selection() {
        let mut state = TableState::new();

        state.toggle_selection(1);
        assert!(state.is_selected(1));

        state.toggle_selection(3);
        assert!(state.is_selected(1));
        assert!(state.is_selected(3));

        state.toggle_selection(1);
        assert!(!state.is_selected(1));
        assert!(state.is_selected(3));
    }

    #[test]
    fn test_state_scroll() {
        let mut state = TableState::new();
        state.scroll_offset = 5;

        // Row within view - no change
        state.scroll_to(7, 10);
        assert_eq!(state.scroll_offset, 5);

        // Row above view
        state.scroll_to(2, 10);
        assert_eq!(state.scroll_offset, 2);

        // Row below view
        state.scroll_to(20, 10);
        assert_eq!(state.scroll_offset, 11);
    }

    #[test]
    fn test_state_edit() {
        let mut state = TableState::new();

        assert!(!state.is_editing());

        state.start_edit(5, 2);
        assert!(state.is_editing());
        assert_eq!(state.edit_cell, Some((5, 2)));

        state.cancel_edit();
        assert!(!state.is_editing());
    }
}
