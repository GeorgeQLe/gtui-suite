//! CommandPalette widget - fuzzy-search command launcher.
//!
//! # Example
//!
//! ```ignore
//! use tui_widgets::{CommandPalette, Command};
//! use std::collections::HashMap;
//!
//! struct OpenFileCommand;
//!
//! impl Command for OpenFileCommand {
//!     fn id(&self) -> &str { "file.open" }
//!     fn label(&self) -> &str { "Open File" }
//!     fn execute(&self, _params: HashMap<String, String>) -> Result<(), CommandError> {
//!         // Open file dialog...
//!         Ok(())
//!     }
//! }
//!
//! let mut palette = CommandPalette::new();
//! palette.register(OpenFileCommand);
//! ```

mod state;

pub use state::PaletteState;

use crate::accessibility::{Accessible, SoundCue};
use crate::command::{BoxedCommand, Command, CommandError, CommandRegistry};
use crate::form::{InputType, Value};
use crate::WidgetConfig;

use crossterm::event::{KeyCode, KeyEvent};
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, StatefulWidget, Widget};

use std::collections::HashMap;

/// Parameter definition for multi-step wizard.
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Display label
    pub label: String,
    /// Input type
    pub input_type: InputType,
    /// Whether required
    pub required: bool,
    /// Default value
    pub default: Option<Value>,
}

impl Parameter {
    /// Create a new parameter.
    pub fn new(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Text,
            required: true,
            default: None,
        }
    }

    /// Set the input type.
    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }

    /// Set whether required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set default value.
    pub fn default(mut self, value: impl Into<Value>) -> Self {
        self.default = Some(value.into());
        self
    }
}

/// A command with parameters for the palette.
pub struct PaletteCommand {
    /// The command implementation
    pub command: BoxedCommand,
    /// Parameters for wizard mode
    pub parameters: Vec<Parameter>,
    /// Keyboard shortcut hint
    pub shortcut: Option<String>,
    /// Category for grouping
    pub category: Option<String>,
}

impl PaletteCommand {
    /// Create a new palette command.
    pub fn new(command: impl Command + 'static) -> Self {
        Self {
            command: Box::new(command),
            parameters: Vec::new(),
            shortcut: None,
            category: None,
        }
    }

    /// Add a parameter.
    pub fn param(mut self, param: Parameter) -> Self {
        self.parameters.push(param);
        self
    }

    /// Set shortcut hint.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Set category.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

/// Fuzzy-search command launcher.
pub struct CommandPalette {
    /// Registered commands
    commands: Vec<PaletteCommand>,
    /// Widget configuration
    config: WidgetConfig,
    /// Block wrapper
    block: Option<Block<'static>>,
    /// Maximum visible results
    max_results: usize,
    /// Fuzzy matcher
    matcher: Matcher,
    /// Callback for command execution
    on_execute: Option<Box<dyn Fn(&dyn Command, HashMap<String, String>)>>,
}

impl CommandPalette {
    /// Create a new command palette.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            config: WidgetConfig::default(),
            block: None,
            max_results: 10,
            matcher: Matcher::new(Config::DEFAULT),
            on_execute: None,
        }
    }

    /// Set the block wrapper.
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set maximum visible results.
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    /// Set execution callback.
    pub fn on_execute(
        mut self,
        f: impl Fn(&dyn Command, HashMap<String, String>) + 'static,
    ) -> Self {
        self.on_execute = Some(Box::new(f));
        self
    }

    /// Register a command.
    pub fn register(&mut self, command: impl Command + 'static) {
        self.commands.push(PaletteCommand::new(command));
    }

    /// Register a command with parameters.
    pub fn register_with_params(&mut self, command: PaletteCommand) {
        self.commands.push(command);
    }

    /// Get filtered and ranked results.
    pub fn get_results(&mut self, query: &str) -> Vec<(usize, u32)> {
        if query.is_empty() {
            // Return all commands when query is empty
            return self
                .commands
                .iter()
                .enumerate()
                .filter(|(_, c)| !c.command.is_hidden() && c.command.is_enabled())
                .map(|(i, _)| (i, 0))
                .collect();
        }

        let pattern = Pattern::new(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        let mut results: Vec<(usize, u32)> = Vec::new();

        for (idx, cmd) in self.commands.iter().enumerate() {
            if cmd.command.is_hidden() || !cmd.command.is_enabled() {
                continue;
            }

            // Match against label
            let label = cmd.command.label();
            let label_utf32: Vec<char> = label.chars().collect();
            let label_str = Utf32Str::new(&label_utf32, &mut Vec::new());

            let mut indices = Vec::new();
            if let Some(score) = pattern.score(label_str, &mut self.matcher) {
                results.push((idx, score));
                continue;
            }

            // Match against keywords
            for keyword in cmd.command.keywords() {
                let kw_utf32: Vec<char> = keyword.chars().collect();
                let kw_str = Utf32Str::new(&kw_utf32, &mut Vec::new());
                if let Some(score) = pattern.score(kw_str, &mut self.matcher) {
                    results.push((idx, score / 2)); // Keywords score lower
                    break;
                }
            }

            // Match against category
            if let Some(cat) = cmd.command.category() {
                let cat_utf32: Vec<char> = cat.chars().collect();
                let cat_str = Utf32Str::new(&cat_utf32, &mut Vec::new());
                if let Some(score) = pattern.score(cat_str, &mut self.matcher) {
                    results.push((idx, score / 4)); // Category scores even lower
                }
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: KeyEvent, state: &mut PaletteState) -> bool {
        if self.config.disabled {
            return false;
        }

        // Handle wizard mode
        if let Some(ref mut wizard) = state.wizard_step {
            return self.handle_wizard_key(key, state);
        }

        match key.code {
            KeyCode::Esc => {
                state.close();
                true
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => {
                state.select_previous();
                true
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => {
                state.select_next(self.max_results);
                true
            }
            KeyCode::Enter => {
                self.execute_selected(state);
                true
            }
            KeyCode::Char(c) => {
                state.query.push(c);
                state.selected_index = 0;
                true
            }
            KeyCode::Backspace => {
                state.query.pop();
                state.selected_index = 0;
                true
            }
            _ => false,
        }
    }

    fn handle_wizard_key(&mut self, key: KeyEvent, state: &mut PaletteState) -> bool {
        let wizard = state.wizard_step.as_mut().unwrap();

        match key.code {
            KeyCode::Esc => {
                // Cancel wizard
                state.wizard_step = None;
                true
            }
            KeyCode::Enter => {
                // Save current param and move to next
                let param_name = wizard.params[wizard.current_param].name.clone();
                wizard.values.insert(param_name, wizard.current_input.clone());
                wizard.current_input.clear();
                wizard.current_param += 1;

                // Check if wizard is complete
                if wizard.current_param >= wizard.params.len() {
                    // Execute command with collected params
                    let cmd_idx = wizard.command_index;
                    let params: HashMap<String, String> = wizard.values.clone();
                    state.wizard_step = None;

                    if let Some(cmd) = self.commands.get(cmd_idx) {
                        if let Some(ref callback) = self.on_execute {
                            callback(cmd.command.as_ref(), params.clone());
                        }
                        let _ = cmd.command.execute(params);
                    }
                    state.close();
                }
                true
            }
            KeyCode::Char(c) => {
                wizard.current_input.push(c);
                true
            }
            KeyCode::Backspace => {
                wizard.current_input.pop();
                true
            }
            _ => false,
        }
    }

    fn execute_selected(&mut self, state: &mut PaletteState) {
        let results = self.get_results(&state.query);
        if let Some(&(cmd_idx, _)) = results.get(state.selected_index) {
            if let Some(cmd) = self.commands.get(cmd_idx) {
                if !cmd.parameters.is_empty() {
                    // Enter wizard mode
                    state.wizard_step = Some(WizardState {
                        command_index: cmd_idx,
                        params: cmd.parameters.clone(),
                        current_param: 0,
                        current_input: String::new(),
                        values: HashMap::new(),
                    });
                } else {
                    // Execute directly
                    if let Some(ref callback) = self.on_execute {
                        callback(cmd.command.as_ref(), HashMap::new());
                    }
                    let _ = cmd.command.execute(HashMap::new());
                    state.close();
                }
            }
        }
    }

    /// Get command at index.
    pub fn get_command(&self, index: usize) -> Option<&dyn Command> {
        self.commands.get(index).map(|c| c.command.as_ref())
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

/// Wizard state for multi-step parameter collection.
#[derive(Debug, Clone)]
pub struct WizardState {
    /// Index of command being configured
    pub command_index: usize,
    /// Parameters to collect
    pub params: Vec<Parameter>,
    /// Current parameter index
    pub current_param: usize,
    /// Current input value
    pub current_input: String,
    /// Collected values
    pub values: HashMap<String, String>,
}

impl StatefulWidget for CommandPalette {
    type State = PaletteState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.visible {
            return;
        }

        // Calculate palette dimensions
        let width = (area.width * 3 / 4).min(60).max(30);
        let height = (self.max_results + 3) as u16; // +3 for border and input

        let x = area.x + (area.width - width) / 2;
        let y = area.y + 2; // Near top

        let palette_area = Rect::new(x, y, width, height.min(area.height - y));

        // Clear background
        Clear.render(palette_area, buf);

        // Render border
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Command Palette ")
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(palette_area);
        block.render(palette_area, buf);

        if inner.width < 5 || inner.height < 2 {
            return;
        }

        // Check for wizard mode
        if let Some(ref wizard) = state.wizard_step {
            self.render_wizard(inner, buf, wizard);
            return;
        }

        // Input line
        let input_y = inner.y;
        let prompt = "> ";
        buf.set_string(inner.x, input_y, prompt, Style::default().fg(Color::Yellow));
        buf.set_string(
            inner.x + prompt.len() as u16,
            input_y,
            &state.query,
            Style::default(),
        );

        // Cursor
        let cursor_x = inner.x + prompt.len() as u16 + state.query.len() as u16;
        if cursor_x < inner.x + inner.width {
            buf[(cursor_x, input_y)].set_char('_');
        }

        // Results
        let results = self.get_results(&state.query);
        let results_y = input_y + 1;

        for (i, &(cmd_idx, score)) in results.iter().take(self.max_results).enumerate() {
            let y = results_y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }

            let cmd = &self.commands[cmd_idx];
            let is_selected = i == state.selected_index;

            let style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            // Clear line
            for col_x in inner.x..inner.x + inner.width {
                buf[(col_x, y)].set_style(style);
            }

            // Command label
            let label = cmd.command.label();
            let mut x = inner.x;

            // Category prefix
            if let Some(ref category) = cmd.category {
                let cat_style = style.add_modifier(Modifier::DIM);
                let cat = format!("{}:", category);
                buf.set_string(x, y, &cat, cat_style);
                x += cat.len() as u16 + 1;
            }

            // Label
            let max_label_len = (inner.width as usize).saturating_sub(x as usize - inner.x as usize);
            let display_label = if label.len() > max_label_len {
                format!("{}...", &label[..max_label_len.saturating_sub(3)])
            } else {
                label.to_string()
            };
            buf.set_string(x, y, &display_label, style);

            // Shortcut hint on the right
            if let Some(ref shortcut) = cmd.shortcut {
                let shortcut_x = inner.x + inner.width - shortcut.len() as u16 - 1;
                if shortcut_x > x + display_label.len() as u16 + 1 {
                    let shortcut_style = style.add_modifier(Modifier::DIM);
                    buf.set_string(shortcut_x, y, shortcut, shortcut_style);
                }
            }
        }

        // No results message
        if results.is_empty() && !state.query.is_empty() {
            let msg = "No matching commands";
            buf.set_string(
                inner.x,
                results_y,
                msg,
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}

impl CommandPalette {
    fn render_wizard(&self, area: Rect, buf: &mut Buffer, wizard: &WizardState) {
        let param = &wizard.params[wizard.current_param];

        // Progress indicator
        let progress = format!(
            "Step {} of {}",
            wizard.current_param + 1,
            wizard.params.len()
        );
        buf.set_string(
            area.x,
            area.y,
            &progress,
            Style::default().fg(Color::DarkGray),
        );

        // Parameter label
        let label = format!("{}:", param.label);
        buf.set_string(area.x, area.y + 1, &label, Style::default());

        // Input
        let input_y = area.y + 2;
        let prompt = "> ";
        buf.set_string(area.x, input_y, prompt, Style::default().fg(Color::Yellow));
        buf.set_string(
            area.x + prompt.len() as u16,
            input_y,
            &wizard.current_input,
            Style::default(),
        );

        // Cursor
        let cursor_x = area.x + prompt.len() as u16 + wizard.current_input.len() as u16;
        if cursor_x < area.x + area.width {
            buf[(cursor_x, input_y)].set_char('_');
        }

        // Help text
        let help = "Enter to continue, Esc to cancel";
        buf.set_string(
            area.x,
            area.y + 4,
            help,
            Style::default().fg(Color::DarkGray),
        );
    }
}

impl Accessible for CommandPalette {
    fn aria_role(&self) -> &str {
        "combobox"
    }

    fn aria_label(&self) -> String {
        format!("Command palette with {} commands", self.commands.len())
    }

    fn announce(&self, _message: &str) {
        // Would integrate with announcement buffer
    }

    fn play_sound(&self, _sound: SoundCue) {
        // Would integrate with sound system
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand {
        id: String,
        label: String,
    }

    impl Command for TestCommand {
        fn id(&self) -> &str {
            &self.id
        }

        fn label(&self) -> &str {
            &self.label
        }

        fn execute(&self, _params: HashMap<String, String>) -> Result<(), CommandError> {
            Ok(())
        }
    }

    #[test]
    fn test_palette_creation() {
        let mut palette = CommandPalette::new();
        palette.register(TestCommand {
            id: "test.cmd".into(),
            label: "Test Command".into(),
        });

        assert_eq!(palette.commands.len(), 1);
    }

    #[test]
    fn test_fuzzy_search() {
        let mut palette = CommandPalette::new();
        palette.register(TestCommand {
            id: "file.open".into(),
            label: "Open File".into(),
        });
        palette.register(TestCommand {
            id: "file.save".into(),
            label: "Save File".into(),
        });
        palette.register(TestCommand {
            id: "edit.copy".into(),
            label: "Copy".into(),
        });

        // Empty query returns all
        let results = palette.get_results("");
        assert_eq!(results.len(), 3);

        // "open" should match "Open File"
        let results = palette.get_results("open");
        assert!(!results.is_empty());

        // "fil" should match both file commands
        let results = palette.get_results("fil");
        assert!(results.len() >= 2);
    }

    #[test]
    fn test_palette_state() {
        let mut state = PaletteState::new();
        assert!(!state.visible);

        state.open();
        assert!(state.visible);
        assert!(state.query.is_empty());
        assert_eq!(state.selected_index, 0);

        state.select_next(10);
        assert_eq!(state.selected_index, 1);

        state.select_previous();
        assert_eq!(state.selected_index, 0);

        state.close();
        assert!(!state.visible);
    }
}
