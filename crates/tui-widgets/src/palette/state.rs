//! State management for CommandPalette.

use super::WizardState;

/// State for CommandPalette widget.
#[derive(Debug, Clone, Default)]
pub struct PaletteState {
    /// Whether the palette is visible
    pub visible: bool,
    /// Current search query
    pub query: String,
    /// Selected result index
    pub selected_index: usize,
    /// Wizard state for multi-step commands
    pub wizard_step: Option<WizardState>,
}

impl PaletteState {
    /// Create a new palette state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the command palette.
    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.selected_index = 0;
        self.wizard_step = None;
    }

    /// Close the command palette.
    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.selected_index = 0;
        self.wizard_step = None;
    }

    /// Toggle palette visibility.
    pub fn toggle(&mut self) {
        if self.visible {
            self.close();
        } else {
            self.open();
        }
    }

    /// Move selection to next item.
    pub fn select_next(&mut self, max: usize) {
        if max > 0 {
            self.selected_index = (self.selected_index + 1).min(max - 1);
        }
    }

    /// Move selection to previous item.
    pub fn select_previous(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Select first item.
    pub fn select_first(&mut self) {
        self.selected_index = 0;
    }

    /// Select last item.
    pub fn select_last(&mut self, max: usize) {
        if max > 0 {
            self.selected_index = max - 1;
        }
    }

    /// Check if in wizard mode.
    pub fn is_wizard_mode(&self) -> bool {
        self.wizard_step.is_some()
    }

    /// Get current wizard step.
    pub fn wizard_step(&self) -> Option<&WizardState> {
        self.wizard_step.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_open_close() {
        let mut state = PaletteState::new();
        assert!(!state.visible);

        state.open();
        assert!(state.visible);
        assert!(state.query.is_empty());

        state.query.push_str("test");
        state.close();
        assert!(!state.visible);
        assert!(state.query.is_empty()); // Query should be cleared
    }

    #[test]
    fn test_state_toggle() {
        let mut state = PaletteState::new();

        state.toggle();
        assert!(state.visible);

        state.toggle();
        assert!(!state.visible);
    }

    #[test]
    fn test_state_navigation() {
        let mut state = PaletteState::new();

        state.select_next(5);
        assert_eq!(state.selected_index, 1);

        state.select_next(5);
        state.select_next(5);
        assert_eq!(state.selected_index, 3);

        state.select_next(5);
        assert_eq!(state.selected_index, 4);

        // Can't go past max
        state.select_next(5);
        assert_eq!(state.selected_index, 4);

        state.select_previous();
        assert_eq!(state.selected_index, 3);

        state.select_first();
        assert_eq!(state.selected_index, 0);

        state.select_last(5);
        assert_eq!(state.selected_index, 4);
    }

    #[test]
    fn test_state_bounds() {
        let mut state = PaletteState::new();

        // Can't go below 0
        state.select_previous();
        assert_eq!(state.selected_index, 0);

        // Empty list
        state.select_next(0);
        assert_eq!(state.selected_index, 0);
    }
}
