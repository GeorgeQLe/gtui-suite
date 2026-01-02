//! Theme variants.

use serde::{Deserialize, Serialize};

/// Theme variant for different density/spacing preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeVariant {
    /// Minimal padding, dense information display
    Compact,
    /// Balanced spacing (default)
    #[default]
    Comfortable,
    /// More padding, larger spacing for readability
    Spacious,
}

impl ThemeVariant {
    /// Get the padding multiplier for this variant.
    pub fn padding_multiplier(&self) -> f32 {
        match self {
            Self::Compact => 0.5,
            Self::Comfortable => 1.0,
            Self::Spacious => 1.5,
        }
    }

    /// Get the line height adjustment for this variant.
    pub fn line_height(&self) -> u16 {
        match self {
            Self::Compact => 1,
            Self::Comfortable => 1,
            Self::Spacious => 2,
        }
    }
}

impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compact => write!(f, "compact"),
            Self::Comfortable => write!(f, "comfortable"),
            Self::Spacious => write!(f, "spacious"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_variant() {
        assert_eq!(ThemeVariant::default(), ThemeVariant::Comfortable);
    }

    #[test]
    fn test_padding_multiplier() {
        assert!(ThemeVariant::Compact.padding_multiplier() < ThemeVariant::Comfortable.padding_multiplier());
        assert!(ThemeVariant::Comfortable.padding_multiplier() < ThemeVariant::Spacious.padding_multiplier());
    }

    #[test]
    fn test_display() {
        assert_eq!(ThemeVariant::Compact.to_string(), "compact");
        assert_eq!(ThemeVariant::Comfortable.to_string(), "comfortable");
        assert_eq!(ThemeVariant::Spacious.to_string(), "spacious");
    }
}
