//! State management for Form.

use super::{Field, FormData, Value};
use std::collections::HashMap;

/// State for Form widget.
#[derive(Debug, Clone, Default)]
pub struct FormState {
    /// Current field values
    pub values: FormData,
    /// Validation errors (field name -> error message)
    pub errors: HashMap<String, String>,
    /// Currently focused field name
    pub focused_field: Option<String>,
    /// Whether form is submitting (for async validation)
    pub submitting: bool,
    /// Validation in progress for these fields
    pub validating: Vec<String>,
}

impl FormState {
    /// Create a new empty form state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create form state with initial values.
    pub fn with_values(values: FormData) -> Self {
        Self {
            values,
            ..Default::default()
        }
    }

    /// Get a field value.
    pub fn get(&self, field: &str) -> Option<&Value> {
        self.values.get(field)
    }

    /// Set a field value.
    pub fn set(&mut self, field: impl Into<String>, value: impl Into<Value>) {
        let field = field.into();
        self.values.insert(field.clone(), value.into());
        // Clear error when value changes
        self.errors.remove(&field);
    }

    /// Check if a field has an error.
    pub fn has_error(&self, field: &str) -> bool {
        self.errors.contains_key(field)
    }

    /// Get error for a field.
    pub fn get_error(&self, field: &str) -> Option<&str> {
        self.errors.get(field).map(|s| s.as_str())
    }

    /// Set error for a field.
    pub fn set_error(&mut self, field: impl Into<String>, error: impl Into<String>) {
        self.errors.insert(field.into(), error.into());
    }

    /// Clear error for a field.
    pub fn clear_error(&mut self, field: &str) {
        self.errors.remove(field);
    }

    /// Clear all errors.
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// Focus a specific field.
    pub fn focus(&mut self, field: impl Into<String>) {
        self.focused_field = Some(field.into());
    }

    /// Focus the next field.
    pub fn focus_next(&mut self, fields: &[Field]) {
        if fields.is_empty() {
            return;
        }

        let enabled_fields: Vec<_> = fields.iter().filter(|f| !f.disabled).collect();
        if enabled_fields.is_empty() {
            return;
        }

        let current_idx = self
            .focused_field
            .as_ref()
            .and_then(|name| enabled_fields.iter().position(|f| &f.name == name));

        let next_idx = match current_idx {
            Some(idx) => (idx + 1) % enabled_fields.len(),
            None => 0,
        };

        self.focused_field = Some(enabled_fields[next_idx].name.clone());
    }

    /// Focus the previous field.
    pub fn focus_previous(&mut self, fields: &[Field]) {
        if fields.is_empty() {
            return;
        }

        let enabled_fields: Vec<_> = fields.iter().filter(|f| !f.disabled).collect();
        if enabled_fields.is_empty() {
            return;
        }

        let current_idx = self
            .focused_field
            .as_ref()
            .and_then(|name| enabled_fields.iter().position(|f| &f.name == name));

        let prev_idx = match current_idx {
            Some(0) => enabled_fields.len() - 1,
            Some(idx) => idx - 1,
            None => 0,
        };

        self.focused_field = Some(enabled_fields[prev_idx].name.clone());
    }

    /// Check if the currently focused field is the last one.
    pub fn is_last_field(&self, fields: &[Field]) -> bool {
        let enabled_fields: Vec<_> = fields.iter().filter(|f| !f.disabled).collect();
        if enabled_fields.is_empty() {
            return true;
        }

        match &self.focused_field {
            Some(name) => {
                enabled_fields.last().map(|f| &f.name == name).unwrap_or(false)
            }
            None => false,
        }
    }

    /// Check if form is valid (no errors).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if form is dirty (has any values).
    pub fn is_dirty(&self) -> bool {
        self.values.values().any(|v| !v.is_empty())
    }

    /// Reset the form to initial state.
    pub fn reset(&mut self) {
        self.values.clear();
        self.errors.clear();
        self.focused_field = None;
        self.submitting = false;
        self.validating.clear();
    }

    /// Reset with new initial values.
    pub fn reset_with(&mut self, values: FormData) {
        self.reset();
        self.values = values;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_fields() -> Vec<Field> {
        vec![
            Field::new("name", "Name"),
            Field::new("email", "Email"),
            Field::new("disabled", "Disabled").disabled(true),
            Field::new("phone", "Phone"),
        ]
    }

    #[test]
    fn test_state_values() {
        let mut state = FormState::new();

        state.set("name", "Alice");
        assert_eq!(state.get("name"), Some(&Value::String("Alice".into())));

        state.set("age", Value::Number(30.0));
        assert_eq!(state.get("age"), Some(&Value::Number(30.0)));
    }

    #[test]
    fn test_state_errors() {
        let mut state = FormState::new();

        assert!(!state.has_error("name"));

        state.set_error("name", "Name is required");
        assert!(state.has_error("name"));
        assert_eq!(state.get_error("name"), Some("Name is required"));

        // Setting value clears error
        state.set("name", "Alice");
        assert!(!state.has_error("name"));
    }

    #[test]
    fn test_state_focus_navigation() {
        let fields = test_fields();
        let mut state = FormState::new();

        // First focus
        state.focus_next(&fields);
        assert_eq!(state.focused_field, Some("name".into()));

        // Next (should skip disabled)
        state.focus_next(&fields);
        assert_eq!(state.focused_field, Some("email".into()));

        state.focus_next(&fields);
        assert_eq!(state.focused_field, Some("phone".into()));

        // Wrap around
        state.focus_next(&fields);
        assert_eq!(state.focused_field, Some("name".into()));

        // Previous
        state.focus_previous(&fields);
        assert_eq!(state.focused_field, Some("phone".into()));
    }

    #[test]
    fn test_state_is_last_field() {
        let fields = test_fields();
        let mut state = FormState::new();

        state.focus("name");
        assert!(!state.is_last_field(&fields));

        state.focus("phone");
        assert!(state.is_last_field(&fields));
    }

    #[test]
    fn test_state_dirty() {
        let mut state = FormState::new();
        assert!(!state.is_dirty());

        state.set("name", "");
        assert!(!state.is_dirty());

        state.set("name", "Alice");
        assert!(state.is_dirty());
    }

    #[test]
    fn test_state_reset() {
        let mut state = FormState::new();
        state.set("name", "Alice");
        state.set_error("email", "Required");
        state.focus("name");

        state.reset();

        assert!(state.values.is_empty());
        assert!(state.errors.is_empty());
        assert!(state.focused_field.is_none());
    }
}
