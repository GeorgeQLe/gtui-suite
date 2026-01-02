//! Plugin capabilities.

use serde::{Deserialize, Serialize};

/// Capabilities a plugin can provide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Adds commands to command palette.
    Commands,

    /// Adds keybindings.
    Keybindings,

    /// Adds theme colors/styles.
    Theming,

    /// Adds file type handlers.
    FileHandler {
        /// File extensions this handler supports.
        extensions: Vec<String>,
    },

    /// Adds data transformers.
    Transformer,

    /// Provides syntax highlighting.
    SyntaxHighlight {
        /// Languages supported.
        languages: Vec<String>,
    },

    /// Provides code completion.
    Completion,

    /// Provides diagnostics/linting.
    Diagnostics,

    /// Provides formatter.
    Formatter,

    /// Custom capability.
    Custom(String),
}

impl Capability {
    /// Get the capability name.
    pub fn name(&self) -> &str {
        match self {
            Self::Commands => "commands",
            Self::Keybindings => "keybindings",
            Self::Theming => "theming",
            Self::FileHandler { .. } => "file_handler",
            Self::Transformer => "transformer",
            Self::SyntaxHighlight { .. } => "syntax_highlight",
            Self::Completion => "completion",
            Self::Diagnostics => "diagnostics",
            Self::Formatter => "formatter",
            Self::Custom(name) => name,
        }
    }

    /// Check if this is a custom capability.
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Parse from a string.
    pub fn from_str_simple(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "commands" => Self::Commands,
            "keybindings" => Self::Keybindings,
            "theming" => Self::Theming,
            "transformer" => Self::Transformer,
            "completion" => Self::Completion,
            "diagnostics" => Self::Diagnostics,
            "formatter" => Self::Formatter,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileHandler { extensions } => {
                write!(f, "file_handler({})", extensions.join(", "))
            }
            Self::SyntaxHighlight { languages } => {
                write!(f, "syntax_highlight({})", languages.join(", "))
            }
            _ => write!(f, "{}", self.name()),
        }
    }
}

/// Set of capabilities for querying.
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    capabilities: Vec<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a list of capabilities.
    pub fn from_capabilities(capabilities: Vec<Capability>) -> Self {
        Self { capabilities }
    }

    /// Add a capability.
    pub fn add(&mut self, capability: Capability) {
        if !self.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Check if a capability is present.
    pub fn contains(&self, capability: &Capability) -> bool {
        self.capabilities.iter().any(|c| c.name() == capability.name())
    }

    /// Check if commands capability is present.
    pub fn has_commands(&self) -> bool {
        self.contains(&Capability::Commands)
    }

    /// Check if keybindings capability is present.
    pub fn has_keybindings(&self) -> bool {
        self.contains(&Capability::Keybindings)
    }

    /// Check if theming capability is present.
    pub fn has_theming(&self) -> bool {
        self.contains(&Capability::Theming)
    }

    /// Get file handler extensions if present.
    pub fn file_handler_extensions(&self) -> Option<&[String]> {
        for cap in &self.capabilities {
            if let Capability::FileHandler { extensions } = cap {
                return Some(extensions);
            }
        }
        None
    }

    /// Get all capabilities.
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.iter()
    }

    /// Get number of capabilities.
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }
}

impl IntoIterator for CapabilitySet {
    type Item = Capability;
    type IntoIter = std::vec::IntoIter<Capability>;

    fn into_iter(self) -> Self::IntoIter {
        self.capabilities.into_iter()
    }
}

impl<'a> IntoIterator for &'a CapabilitySet {
    type Item = &'a Capability;
    type IntoIter = std::slice::Iter<'a, Capability>;

    fn into_iter(self) -> Self::IntoIter {
        self.capabilities.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_name() {
        assert_eq!(Capability::Commands.name(), "commands");
        assert_eq!(Capability::Custom("my_cap".to_string()).name(), "my_cap");
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(Capability::Commands.to_string(), "commands");
        assert_eq!(
            Capability::FileHandler {
                extensions: vec!["txt".to_string(), "md".to_string()]
            }
            .to_string(),
            "file_handler(txt, md)"
        );
    }

    #[test]
    fn test_capability_set() {
        let mut set = CapabilitySet::new();
        set.add(Capability::Commands);
        set.add(Capability::Keybindings);

        assert!(set.has_commands());
        assert!(set.has_keybindings());
        assert!(!set.has_theming());
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_from_str_simple() {
        assert_eq!(Capability::from_str_simple("commands"), Capability::Commands);
        assert_eq!(
            Capability::from_str_simple("unknown"),
            Capability::Custom("unknown".to_string())
        );
    }
}
