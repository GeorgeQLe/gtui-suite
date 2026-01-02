//! FormBuilder widget - declarative form construction with validation.
//!
//! # Example
//!
//! ```ignore
//! use tui_widgets::{FormBuilder, Field, InputType, Validator};
//!
//! let form = FormBuilder::new()
//!     .row(|r| {
//!         r.field(Field::new("username", "Username")
//!             .input_type(InputType::Text)
//!             .validator(Validator::Required)
//!             .validator(Validator::MinLength(3)))
//!         .field(Field::new("email", "Email")
//!             .input_type(InputType::Text)
//!             .validator(Validator::Required))
//!     })
//!     .row(|r| {
//!         r.field(Field::new("password", "Password")
//!             .input_type(InputType::Password)
//!             .validator(Validator::Required)
//!             .validator(Validator::MinLength(8)))
//!     })
//!     .build();
//! ```

mod state;
mod validation;

pub use state::FormState;
pub use validation::Validator;

use crate::accessibility::{Accessible, SoundCue};
use crate::WidgetConfig;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, StatefulWidget, Widget};

use std::collections::HashMap;

/// Input type for form fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputType {
    /// Single-line text input
    Text,
    /// Password input (masked)
    Password,
    /// Numeric input
    Number,
    /// Date input
    Date,
    /// Dropdown select
    Select(Vec<String>),
    /// Multi-select
    MultiSelect(Vec<String>),
    /// Checkbox
    Checkbox,
    /// Radio button group
    Radio(Vec<String>),
    /// Multi-line text area
    TextArea,
}

impl Default for InputType {
    fn default() -> Self {
        Self::Text
    }
}

/// Value type for form fields.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Empty/null value
    None,
    /// String value
    String(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Bool(bool),
    /// List of selected values (for multi-select)
    List(Vec<String>),
}

impl Default for Value {
    fn default() -> Self {
        Self::None
    }
}

impl Value {
    /// Get as string, or empty string if not a string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::String(s) => s,
            _ => "",
        }
    }

    /// Check if the value is empty/none.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::None => true,
            Self::String(s) => s.is_empty(),
            Self::List(l) => l.is_empty(),
            _ => false,
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

/// Form field definition.
#[derive(Debug, Clone)]
pub struct Field {
    /// Field name (used as key in form data)
    pub name: String,
    /// Display label
    pub label: String,
    /// Input type
    pub input_type: InputType,
    /// Validators
    pub validators: Vec<Validator>,
    /// Default value
    pub default_value: Option<Value>,
    /// Whether the field is disabled
    pub disabled: bool,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Help text
    pub help: Option<String>,
}

impl Field {
    /// Create a new field.
    pub fn new(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Text,
            validators: Vec::new(),
            default_value: None,
            disabled: false,
            placeholder: None,
            help: None,
        }
    }

    /// Set the input type.
    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }

    /// Add a validator.
    pub fn validator(mut self, validator: Validator) -> Self {
        self.validators.push(validator);
        self
    }

    /// Set the default value.
    pub fn default_value(mut self, value: impl Into<Value>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set placeholder text.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set help text.
    pub fn help(mut self, text: impl Into<String>) -> Self {
        self.help = Some(text.into());
        self
    }
}

/// A row of fields in the form.
pub struct RowBuilder {
    fields: Vec<Field>,
    full_width: bool,
}

impl RowBuilder {
    /// Create a new row builder.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            full_width: false,
        }
    }

    /// Add a field to the row.
    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    /// Make the row span full width.
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }
}

impl Default for RowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A section in the form.
#[derive(Debug, Clone)]
pub struct Section {
    /// Section title
    pub title: Option<String>,
    /// Rows in this section
    pub rows: Vec<Vec<Field>>,
}

/// Form builder for declarative form construction.
pub struct FormBuilder {
    sections: Vec<Section>,
    current_section: Section,
}

impl FormBuilder {
    /// Create a new form builder.
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            current_section: Section {
                title: None,
                rows: Vec::new(),
            },
        }
    }

    /// Add a row of fields.
    pub fn row(mut self, f: impl FnOnce(RowBuilder) -> RowBuilder) -> Self {
        let row = f(RowBuilder::new());
        self.current_section.rows.push(row.fields);
        self
    }

    /// Start a new section.
    pub fn section(mut self, title: &str, f: impl FnOnce(FormBuilder) -> FormBuilder) -> Self {
        // Save current section
        if !self.current_section.rows.is_empty() {
            self.sections.push(std::mem::replace(
                &mut self.current_section,
                Section {
                    title: None,
                    rows: Vec::new(),
                },
            ));
        }

        // Build section content
        let section_builder = f(FormBuilder::new());
        let mut section = section_builder.current_section;
        section.title = Some(title.to_string());

        // Add any nested sections
        for s in section_builder.sections {
            self.sections.push(s);
        }
        self.sections.push(section);

        self
    }

    /// Build the form.
    pub fn build(mut self) -> Form {
        if !self.current_section.rows.is_empty() {
            self.sections.push(self.current_section);
        }

        let mut fields = Vec::new();
        for section in &self.sections {
            for row in &section.rows {
                for field in row {
                    fields.push(field.clone());
                }
            }
        }

        Form {
            sections: self.sections,
            fields,
            config: WidgetConfig::default(),
            block: None,
            on_submit: None,
            on_cancel: None,
            on_change: None,
        }
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Form data - a collection of field values.
pub type FormData = HashMap<String, Value>;

/// The built form widget.
pub struct Form {
    sections: Vec<Section>,
    fields: Vec<Field>,
    config: WidgetConfig,
    block: Option<Block<'static>>,
    on_submit: Option<Box<dyn Fn(FormData)>>,
    on_cancel: Option<Box<dyn Fn()>>,
    on_change: Option<Box<dyn Fn(&str, &Value)>>,
}

impl Form {
    /// Set the block wrapper.
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set submit callback.
    pub fn on_submit(mut self, f: impl Fn(FormData) + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    /// Set cancel callback.
    pub fn on_cancel(mut self, f: impl Fn() + 'static) -> Self {
        self.on_cancel = Some(Box::new(f));
        self
    }

    /// Set change callback.
    pub fn on_change(mut self, f: impl Fn(&str, &Value) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Get all field definitions.
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    /// Validate all fields and return errors.
    pub fn validate(&self, data: &FormData) -> HashMap<String, String> {
        let mut errors = HashMap::new();

        for field in &self.fields {
            let value = data.get(&field.name).cloned().unwrap_or_default();

            for validator in &field.validators {
                if let Err(msg) = validator.validate(&value, data) {
                    errors.insert(field.name.clone(), msg);
                    break; // Only first error per field
                }
            }
        }

        errors
    }

    /// Handle a key event.
    pub fn handle_key(&self, key: KeyEvent, state: &mut FormState) -> bool {
        if self.config.disabled {
            return false;
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    state.focus_previous(&self.fields);
                } else {
                    state.focus_next(&self.fields);
                }
                true
            }
            KeyCode::Enter => {
                if state.is_last_field(&self.fields) {
                    // Submit
                    let errors = self.validate(&state.values);
                    if errors.is_empty() {
                        if let Some(ref callback) = self.on_submit {
                            callback(state.values.clone());
                        }
                    } else {
                        state.errors = errors;
                    }
                } else {
                    state.focus_next(&self.fields);
                }
                true
            }
            KeyCode::Esc => {
                if let Some(ref callback) = self.on_cancel {
                    callback();
                }
                true
            }
            KeyCode::Char(c) => {
                if let Some(ref name) = state.focused_field {
                    if let Some(field) = self.fields.iter().find(|f| &f.name == name) {
                        if !field.disabled {
                            let current = state.values.entry(name.clone()).or_insert(Value::String(String::new()));
                            if let Value::String(ref mut s) = current {
                                s.push(c);
                                if let Some(ref callback) = self.on_change {
                                    callback(name, current);
                                }
                            }
                        }
                    }
                }
                true
            }
            KeyCode::Backspace => {
                if let Some(ref name) = state.focused_field {
                    let current = state.values.entry(name.clone()).or_insert(Value::String(String::new()));
                    if let Value::String(ref mut s) = current {
                        s.pop();
                        if let Some(ref callback) = self.on_change {
                            callback(name, current);
                        }
                    }
                }
                true
            }
            _ => false,
        }
    }
}

impl StatefulWidget for Form {
    type State = FormState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render block if present
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.width < 10 || inner.height < 1 {
            return;
        }

        let mut y = inner.y;
        let label_width = 15u16;
        let input_width = inner.width.saturating_sub(label_width + 2);

        for section in &self.sections {
            // Section title
            if let Some(ref title) = section.title {
                if y >= inner.y + inner.height {
                    break;
                }
                let style = Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
                buf.set_string(inner.x, y, title, style);
                y += 2;
            }

            // Fields
            for row in &section.rows {
                if y >= inner.y + inner.height {
                    break;
                }

                let field_width = input_width / row.len() as u16;
                let mut x = inner.x;

                for field in row {
                    let is_focused = state.focused_field.as_ref() == Some(&field.name);
                    let has_error = state.errors.contains_key(&field.name);

                    // Label
                    let label_style = if has_error {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default()
                    };
                    let label = format!("{}:", field.label);
                    let label_display = if label.len() > label_width as usize {
                        format!("{}:", &field.label[..label_width as usize - 1])
                    } else {
                        label
                    };
                    buf.set_string(x, y, &label_display, label_style);

                    // Input box
                    let input_x = x + label_width;
                    let input_style = if is_focused {
                        Style::default().bg(Color::DarkGray)
                    } else if field.disabled {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default()
                    };

                    // Clear input area
                    for ix in input_x..input_x + field_width {
                        buf[(ix, y)].set_style(input_style);
                    }

                    // Value
                    let value = state.values.get(&field.name).cloned().unwrap_or_default();
                    let display = match (&field.input_type, &value) {
                        (InputType::Password, Value::String(s)) => {
                            "\u{2022}".repeat(s.len()) // Bullet character
                        }
                        (InputType::Checkbox, Value::Bool(b)) => {
                            if *b { "[\u{2713}]" } else { "[ ]" }.to_string()
                        }
                        (_, Value::String(s)) => s.clone(),
                        (_, Value::Number(n)) => format!("{}", n),
                        (_, Value::Bool(b)) => format!("{}", b),
                        (_, Value::List(l)) => l.join(", "),
                        (_, Value::None) => {
                            field.placeholder.clone().unwrap_or_default()
                        }
                    };

                    let display = if display.len() > field_width as usize {
                        format!("{}...", &display[..field_width as usize - 3])
                    } else {
                        display
                    };
                    buf.set_string(input_x, y, &display, input_style);

                    // Cursor for focused text fields
                    if is_focused && matches!(field.input_type, InputType::Text | InputType::Password | InputType::Number) {
                        let cursor_x = input_x + display.len() as u16;
                        if cursor_x < input_x + field_width {
                            buf[(cursor_x, y)].set_char('_');
                        }
                    }

                    x += label_width + field_width + 1;
                }

                y += 1;

                // Error message
                if let Some(error_field) = row.first() {
                    if let Some(error) = state.errors.get(&error_field.name) {
                        if y < inner.y + inner.height {
                            let error_style = Style::default().fg(Color::Red);
                            buf.set_string(inner.x + label_width, y, error, error_style);
                            y += 1;
                        }
                    }
                }
            }

            y += 1; // Space between sections
        }

        // Initialize focused field if not set
        if state.focused_field.is_none() && !self.fields.is_empty() {
            state.focused_field = Some(self.fields[0].name.clone());
        }
    }
}

impl Accessible for Form {
    fn aria_role(&self) -> &str {
        "form"
    }

    fn aria_label(&self) -> String {
        format!("Form with {} fields", self.fields.len())
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

    #[test]
    fn test_form_builder() {
        let form = FormBuilder::new()
            .row(|r| {
                r.field(Field::new("name", "Name"))
                 .field(Field::new("email", "Email"))
            })
            .row(|r| {
                r.field(Field::new("password", "Password")
                    .input_type(InputType::Password))
            })
            .build();

        assert_eq!(form.fields().len(), 3);
    }

    #[test]
    fn test_form_section() {
        let form = FormBuilder::new()
            .row(|r| r.field(Field::new("username", "Username")))
            .section("Contact", |s| {
                s.row(|r| r.field(Field::new("email", "Email")))
                 .row(|r| r.field(Field::new("phone", "Phone")))
            })
            .build();

        assert_eq!(form.fields().len(), 3);
    }

    #[test]
    fn test_field_builder() {
        let field = Field::new("test", "Test Field")
            .input_type(InputType::Password)
            .validator(Validator::Required)
            .validator(Validator::MinLength(8))
            .placeholder("Enter password")
            .disabled(false);

        assert_eq!(field.name, "test");
        assert_eq!(field.label, "Test Field");
        assert_eq!(field.input_type, InputType::Password);
        assert_eq!(field.validators.len(), 2);
    }

    #[test]
    fn test_validation() {
        let form = FormBuilder::new()
            .row(|r| {
                r.field(Field::new("name", "Name")
                    .validator(Validator::Required))
            })
            .build();

        let empty_data: FormData = HashMap::new();
        let errors = form.validate(&empty_data);
        assert!(errors.contains_key("name"));

        let mut valid_data: FormData = HashMap::new();
        valid_data.insert("name".into(), Value::String("Alice".into()));
        let errors = form.validate(&valid_data);
        assert!(errors.is_empty());
    }
}
