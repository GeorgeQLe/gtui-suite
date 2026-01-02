//! Context types for context-aware keybindings.

use serde::{Deserialize, Serialize};

/// Input context for context-aware keybindings.
///
/// This is the canonical Context enum, defined here and re-exported to other crates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Context {
    /// Normal mode (default navigation)
    #[default]
    Normal,
    /// Insert/edit mode
    Insert,
    /// Visual selection mode
    Visual,
    /// Command line mode
    Command,
    /// Popup/menu mode
    Popup,
    /// Dialog mode
    Dialog,
    /// Custom context for app-specific needs
    Custom(String),
}

impl Context {
    /// Create a custom context.
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(name.into())
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Insert => write!(f, "insert"),
            Self::Visual => write!(f, "visual"),
            Self::Command => write!(f, "command"),
            Self::Popup => write!(f, "popup"),
            Self::Dialog => write!(f, "dialog"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Condition for when a keybinding should be active.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextCondition {
    /// The context this binding applies to
    pub context: Context,
    /// Optional expression for more specific conditions
    ///
    /// Expression syntax supports:
    /// - Variable names: `editorFocus`, `readOnly`, `hasSelection`
    /// - Boolean operators: `&&`, `||`, `!`
    /// - Parentheses for grouping
    ///
    /// Example: `"editorFocus && !readOnly && hasChanges"`
    #[serde(default)]
    pub when: Option<String>,
}

impl ContextCondition {
    /// Create a condition for a specific context.
    pub fn new(context: Context) -> Self {
        Self {
            context,
            when: None,
        }
    }

    /// Create a condition with an expression.
    pub fn with_expression(context: Context, when: impl Into<String>) -> Self {
        Self {
            context,
            when: Some(when.into()),
        }
    }

    /// Check if this condition matches a given context and state.
    pub fn matches(&self, current: &Context, state: &ConditionState) -> bool {
        if self.context != *current {
            return false;
        }

        if let Some(ref expr) = self.when {
            evaluate_expression(expr, state)
        } else {
            true
        }
    }
}

/// State variables for condition evaluation.
#[derive(Debug, Clone, Default)]
pub struct ConditionState {
    /// Named boolean variables
    pub variables: std::collections::HashMap<String, bool>,
}

impl ConditionState {
    /// Create new empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable.
    pub fn set(&mut self, name: impl Into<String>, value: bool) {
        self.variables.insert(name.into(), value);
    }

    /// Get a variable.
    pub fn get(&self, name: &str) -> bool {
        self.variables.get(name).copied().unwrap_or(false)
    }
}

/// Evaluate a simple boolean expression.
///
/// Supports: `&&`, `||`, `!`, parentheses, variable names
fn evaluate_expression(expr: &str, state: &ConditionState) -> bool {
    let expr = expr.trim();

    if expr.is_empty() {
        return true;
    }

    // Handle NOT
    if let Some(rest) = expr.strip_prefix('!') {
        return !evaluate_expression(rest.trim(), state);
    }

    // Handle parentheses
    if expr.starts_with('(') {
        if let Some(close) = find_matching_paren(expr) {
            let inner = &expr[1..close];
            let remaining = expr[close + 1..].trim();

            let inner_result = evaluate_expression(inner, state);

            if remaining.is_empty() {
                return inner_result;
            }

            // Handle operator after parentheses
            if let Some(rest) = remaining.strip_prefix("&&") {
                return inner_result && evaluate_expression(rest.trim(), state);
            }
            if let Some(rest) = remaining.strip_prefix("||") {
                return inner_result || evaluate_expression(rest.trim(), state);
            }
        }
    }

    // Handle AND (lower precedence)
    if let Some(pos) = find_operator(expr, "&&") {
        let left = &expr[..pos];
        let right = &expr[pos + 2..];
        return evaluate_expression(left.trim(), state)
            && evaluate_expression(right.trim(), state);
    }

    // Handle OR (lowest precedence)
    if let Some(pos) = find_operator(expr, "||") {
        let left = &expr[..pos];
        let right = &expr[pos + 2..];
        return evaluate_expression(left.trim(), state)
            || evaluate_expression(right.trim(), state);
    }

    // Variable lookup
    state.get(expr)
}

fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.chars().enumerate() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_operator(s: &str, op: &str) -> Option<usize> {
    let mut depth = 0;
    let chars: Vec<char> = s.chars().collect();

    for i in 0..chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }

        if depth == 0 && s[i..].starts_with(op) {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_display() {
        assert_eq!(Context::Normal.to_string(), "normal");
        assert_eq!(Context::Insert.to_string(), "insert");
        assert_eq!(Context::custom("editor").to_string(), "editor");
    }

    #[test]
    fn test_condition_simple() {
        let condition = ContextCondition::new(Context::Normal);
        let state = ConditionState::new();

        assert!(condition.matches(&Context::Normal, &state));
        assert!(!condition.matches(&Context::Insert, &state));
    }

    #[test]
    fn test_condition_expression() {
        let condition = ContextCondition::with_expression(
            Context::Normal,
            "editorFocus && !readOnly",
        );

        let mut state = ConditionState::new();
        state.set("editorFocus", true);
        state.set("readOnly", false);

        assert!(condition.matches(&Context::Normal, &state));

        state.set("readOnly", true);
        assert!(!condition.matches(&Context::Normal, &state));
    }

    #[test]
    fn test_evaluate_simple() {
        let mut state = ConditionState::new();
        state.set("foo", true);
        state.set("bar", false);

        assert!(evaluate_expression("foo", &state));
        assert!(!evaluate_expression("bar", &state));
        assert!(!evaluate_expression("unknown", &state));
    }

    #[test]
    fn test_evaluate_operators() {
        let mut state = ConditionState::new();
        state.set("a", true);
        state.set("b", false);

        assert!(evaluate_expression("a && !b", &state));
        assert!(evaluate_expression("a || b", &state));
        assert!(!evaluate_expression("a && b", &state));
        assert!(evaluate_expression("!b", &state));
    }
}
