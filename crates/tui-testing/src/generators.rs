//! Property-based testing generators.
//!
//! This module provides proptest strategies for generating random test data.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use proptest::prelude::*;
use proptest::strategy::{BoxedStrategy, Strategy};

/// Generate random key events with sensible defaults.
pub fn key_event() -> impl Strategy<Value = KeyEvent> {
    KeyEventGen::new().build()
}

/// Generate random navigation key sequences.
pub fn navigation_sequence(len: usize) -> impl Strategy<Value = Vec<KeyEvent>> {
    NavigationGen::new(len).build()
}

/// Generate random printable character events.
pub fn printable_char() -> impl Strategy<Value = KeyEvent> {
    KeyEventGen::new().only_printable().build()
}

/// Generate random table data.
pub fn table_data<T: Arbitrary + 'static>(
    rows: impl Into<std::ops::Range<usize>>,
    cols: impl Into<std::ops::Range<usize>>,
) -> impl Strategy<Value = Vec<Vec<T>>> {
    let rows = rows.into();
    let cols = cols.into();

    (rows, cols).prop_flat_map(|(row_count, col_count)| {
        prop::collection::vec(
            prop::collection::vec(any::<T>(), col_count),
            row_count,
        )
    })
}

/// Configurable key event generator.
#[derive(Debug, Clone)]
pub struct KeyEventGen {
    include_modifiers: bool,
    include_function_keys: bool,
    include_special_keys: bool,
    include_navigation_keys: bool,
    allowed_chars: Option<Vec<char>>,
    modifier_probability: f64,
}

impl KeyEventGen {
    /// Create a new generator with default settings.
    pub fn new() -> Self {
        Self {
            include_modifiers: true,
            include_function_keys: true,
            include_special_keys: true,
            include_navigation_keys: true,
            allowed_chars: None,
            modifier_probability: 0.2,
        }
    }

    /// Enable modifier keys in generation.
    pub fn with_modifiers(mut self) -> Self {
        self.include_modifiers = true;
        self
    }

    /// Disable modifier keys.
    pub fn without_modifiers(mut self) -> Self {
        self.include_modifiers = false;
        self
    }

    /// Enable function keys.
    pub fn with_function_keys(mut self) -> Self {
        self.include_function_keys = true;
        self
    }

    /// Disable function keys.
    pub fn without_function_keys(mut self) -> Self {
        self.include_function_keys = false;
        self
    }

    /// Enable special keys (Enter, Tab, Esc, etc.).
    pub fn with_special_keys(mut self) -> Self {
        self.include_special_keys = true;
        self
    }

    /// Disable special keys.
    pub fn without_special_keys(mut self) -> Self {
        self.include_special_keys = false;
        self
    }

    /// Enable navigation keys (arrows, Home, End, etc.).
    pub fn with_navigation_keys(mut self) -> Self {
        self.include_navigation_keys = true;
        self
    }

    /// Disable navigation keys.
    pub fn without_navigation_keys(mut self) -> Self {
        self.include_navigation_keys = false;
        self
    }

    /// Only generate printable characters.
    pub fn only_printable(mut self) -> Self {
        self.include_function_keys = false;
        self.include_special_keys = false;
        self.include_navigation_keys = false;
        self.allowed_chars = Some(
            ('a'..='z')
                .chain('A'..='Z')
                .chain('0'..='9')
                .chain([' ', '.', ',', '!', '?', '-', '_', ':', ';'])
                .collect(),
        );
        self
    }

    /// Only allow specific characters.
    pub fn only_chars(mut self, chars: &[char]) -> Self {
        self.allowed_chars = Some(chars.to_vec());
        self
    }

    /// Set modifier probability (0.0 to 1.0).
    pub fn modifier_probability(mut self, prob: f64) -> Self {
        self.modifier_probability = prob.clamp(0.0, 1.0);
        self
    }

    /// Build the proptest strategy.
    pub fn build(self) -> BoxedStrategy<KeyEvent> {
        let mut key_strategies: Vec<BoxedStrategy<KeyCode>> = Vec::new();

        // Character keys
        if let Some(chars) = self.allowed_chars {
            key_strategies.push(
                prop::sample::select(chars)
                    .prop_map(KeyCode::Char)
                    .boxed(),
            );
        } else {
            // Default: alphanumeric + common punctuation
            let chars: Vec<char> = ('a'..='z')
                .chain('A'..='Z')
                .chain('0'..='9')
                .chain([' ', '.', ',', '!', '?', '-', '_'])
                .collect();
            key_strategies.push(
                prop::sample::select(chars)
                    .prop_map(KeyCode::Char)
                    .boxed(),
            );
        }

        // Function keys
        if self.include_function_keys {
            key_strategies.push(
                (1u8..=12)
                    .prop_map(KeyCode::F)
                    .boxed(),
            );
        }

        // Special keys
        if self.include_special_keys {
            key_strategies.push(
                prop::sample::select(vec![
                    KeyCode::Enter,
                    KeyCode::Tab,
                    KeyCode::Esc,
                    KeyCode::Backspace,
                    KeyCode::Delete,
                    KeyCode::Insert,
                ])
                .boxed(),
            );
        }

        // Navigation keys
        if self.include_navigation_keys {
            key_strategies.push(
                prop::sample::select(vec![
                    KeyCode::Up,
                    KeyCode::Down,
                    KeyCode::Left,
                    KeyCode::Right,
                    KeyCode::Home,
                    KeyCode::End,
                    KeyCode::PageUp,
                    KeyCode::PageDown,
                ])
                .boxed(),
            );
        }

        let key_strategy = prop::strategy::Union::new(key_strategies);

        // Modifier strategy
        let include_modifiers = self.include_modifiers;
        let mod_prob = self.modifier_probability;

        key_strategy
            .prop_flat_map(move |key| {
                let modifier_strat = if include_modifiers {
                    prop::bool::weighted(mod_prob)
                        .prop_flat_map(|use_mods| {
                            if use_mods {
                                prop::sample::select(vec![
                                    KeyModifiers::CONTROL,
                                    KeyModifiers::ALT,
                                    KeyModifiers::SHIFT,
                                    KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                                    KeyModifiers::ALT | KeyModifiers::SHIFT,
                                ])
                                .boxed()
                            } else {
                                Just(KeyModifiers::NONE).boxed()
                            }
                        })
                        .boxed()
                } else {
                    Just(KeyModifiers::NONE).boxed()
                };

                modifier_strat.prop_map(move |mods| KeyEvent::new(key, mods))
            })
            .boxed()
    }
}

impl Default for KeyEventGen {
    fn default() -> Self {
        Self::new()
    }
}

/// Configurable navigation sequence generator.
#[derive(Debug, Clone)]
pub struct NavigationGen {
    length: usize,
    include_page_keys: bool,
    include_home_end: bool,
    include_vim_keys: bool,
}

impl NavigationGen {
    /// Create a new navigation generator.
    pub fn new(length: usize) -> Self {
        Self {
            length,
            include_page_keys: true,
            include_home_end: true,
            include_vim_keys: false,
        }
    }

    /// Include PageUp/PageDown keys.
    pub fn with_page_keys(mut self) -> Self {
        self.include_page_keys = true;
        self
    }

    /// Exclude PageUp/PageDown keys.
    pub fn without_page_keys(mut self) -> Self {
        self.include_page_keys = false;
        self
    }

    /// Include Home/End keys.
    pub fn with_home_end(mut self) -> Self {
        self.include_home_end = true;
        self
    }

    /// Exclude Home/End keys.
    pub fn without_home_end(mut self) -> Self {
        self.include_home_end = false;
        self
    }

    /// Include vim-style navigation (hjkl).
    pub fn with_vim_keys(mut self) -> Self {
        self.include_vim_keys = true;
        self
    }

    /// Build the proptest strategy.
    pub fn build(self) -> BoxedStrategy<Vec<KeyEvent>> {
        let mut keys = vec![KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right];

        if self.include_page_keys {
            keys.push(KeyCode::PageUp);
            keys.push(KeyCode::PageDown);
        }

        if self.include_home_end {
            keys.push(KeyCode::Home);
            keys.push(KeyCode::End);
        }

        let mut strategies: Vec<BoxedStrategy<KeyCode>> = vec![
            prop::sample::select(keys).boxed()
        ];

        if self.include_vim_keys {
            strategies.push(
                prop::sample::select(vec![
                    KeyCode::Char('h'),
                    KeyCode::Char('j'),
                    KeyCode::Char('k'),
                    KeyCode::Char('l'),
                ])
                .boxed(),
            );
        }

        let key_strat = prop::strategy::Union::new(strategies);

        prop::collection::vec(
            key_strat.prop_map(|key| KeyEvent::new(key, KeyModifiers::NONE)),
            self.length,
        )
        .boxed()
    }
}

/// Configurable table data generator.
#[derive(Debug, Clone)]
pub struct TableDataGen {
    rows: std::ops::Range<usize>,
    cols: std::ops::Range<usize>,
    string_length: std::ops::Range<usize>,
}

impl TableDataGen {
    /// Create a new table data generator.
    pub fn new(rows: std::ops::Range<usize>, cols: std::ops::Range<usize>) -> Self {
        Self {
            rows,
            cols,
            string_length: 1..20,
        }
    }

    /// Set the string length range for cell content.
    pub fn string_length(mut self, range: std::ops::Range<usize>) -> Self {
        self.string_length = range;
        self
    }

    /// Build a strategy for string table data.
    pub fn build_strings(self) -> BoxedStrategy<Vec<Vec<String>>> {
        let string_len = self.string_length.clone();

        (self.rows, self.cols)
            .prop_flat_map(move |(row_count, col_count)| {
                let cell_strat = prop::collection::vec(
                    prop::string::string_regex("[a-zA-Z0-9 ]{1,20}")
                        .unwrap(),
                    col_count,
                );
                prop::collection::vec(cell_strat, row_count)
            })
            .boxed()
    }

    /// Build a strategy for numeric table data.
    pub fn build_numbers(self) -> BoxedStrategy<Vec<Vec<i64>>> {
        (self.rows, self.cols)
            .prop_flat_map(|(row_count, col_count)| {
                let cell_strat = prop::collection::vec(any::<i64>(), col_count);
                prop::collection::vec(cell_strat, row_count)
            })
            .boxed()
    }
}

/// Generate a random tree structure.
pub fn tree<T: Arbitrary + Clone + 'static>(
    max_depth: usize,
    max_children: usize,
) -> BoxedStrategy<TreeNode<T>> {
    let leaf = any::<T>().prop_map(|value| TreeNode {
        value,
        children: Vec::new(),
    });

    leaf.prop_recursive(max_depth as u32, 256, 10, move |inner| {
        (
            any::<T>(),
            prop::collection::vec(inner, 0..=max_children),
        )
            .prop_map(|(value, children)| TreeNode { value, children })
    })
    .boxed()
}

/// A simple tree node for testing.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode<T> {
    pub value: T,
    pub children: Vec<TreeNode<T>>,
}

impl<T> TreeNode<T> {
    /// Create a leaf node.
    pub fn leaf(value: T) -> Self {
        Self {
            value,
            children: Vec::new(),
        }
    }

    /// Create a branch node.
    pub fn branch(value: T, children: Vec<TreeNode<T>>) -> Self {
        Self { value, children }
    }

    /// Count all nodes in the tree.
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }

    /// Get the depth of the tree.
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn test_key_event_gen(event in key_event()) {
            // Just verify it generates valid events
            let _ = event.code;
        }

        #[test]
        fn test_navigation_sequence(events in navigation_sequence(10)) {
            prop_assert_eq!(events.len(), 10);
        }

        #[test]
        fn test_printable_chars(event in printable_char()) {
            match event.code {
                KeyCode::Char(c) => prop_assert!(c.is_ascii()),
                _ => prop_assert!(false, "Expected char key"),
            }
        }

        #[test]
        fn test_table_data(data in table_data::<i32>(5..10, 3..5)) {
            prop_assert!(!data.is_empty());
            prop_assert!(data.len() >= 5 && data.len() < 10);
            for row in &data {
                prop_assert!(row.len() >= 3 && row.len() < 5);
            }
        }

        #[test]
        fn test_tree_structure(node in tree::<String>(3, 4)) {
            prop_assert!(node.depth() <= 4); // max_depth + 1
            prop_assert!(node.count() >= 1);
        }
    }

    #[test]
    fn test_key_event_gen_only_printable() {
        let gen = KeyEventGen::new().only_printable();
        // Just verify it builds without error
        let _ = gen.build();
    }

    #[test]
    fn test_navigation_gen_without_page_keys() {
        let gen = NavigationGen::new(5).without_page_keys();
        let _ = gen.build();
    }

    #[test]
    fn test_table_data_gen() {
        let gen = TableDataGen::new(5..10, 3..5);
        let _ = gen.build_strings();
        let _ = gen.build_numbers();
    }

    #[test]
    fn test_tree_node() {
        let leaf = TreeNode::leaf(1);
        assert_eq!(leaf.count(), 1);
        assert_eq!(leaf.depth(), 1);

        let branch = TreeNode::branch(0, vec![
            TreeNode::leaf(1),
            TreeNode::leaf(2),
        ]);
        assert_eq!(branch.count(), 3);
        assert_eq!(branch.depth(), 2);
    }
}
