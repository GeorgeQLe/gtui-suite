//! Spacing system for themes.

use serde::{Deserialize, Serialize};
use crate::ThemeVariant;

/// Spacing value that adapts to theme variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spacing {
    /// Base value in cells
    pub value: u16,
}

impl Spacing {
    /// Create a new spacing value.
    pub const fn new(value: u16) -> Self {
        Self { value }
    }

    /// Resolve the spacing for a given theme variant.
    pub fn resolve(&self, variant: ThemeVariant) -> u16 {
        match variant {
            ThemeVariant::Compact => self.value.saturating_sub(1),
            ThemeVariant::Comfortable => self.value,
            ThemeVariant::Spacious => self.value + 1,
        }
    }
}

impl Default for Spacing {
    fn default() -> Self {
        SPACING_MD
    }
}

impl From<u16> for Spacing {
    fn from(value: u16) -> Self {
        Self { value }
    }
}

/// Extra small spacing (0)
pub const SPACING_XS: Spacing = Spacing { value: 0 };

/// Small spacing (1)
pub const SPACING_SM: Spacing = Spacing { value: 1 };

/// Medium spacing (2)
pub const SPACING_MD: Spacing = Spacing { value: 2 };

/// Large spacing (3)
pub const SPACING_LG: Spacing = Spacing { value: 3 };

/// Extra large spacing (4)
pub const SPACING_XL: Spacing = Spacing { value: 4 };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing_resolve() {
        let spacing = SPACING_MD;

        assert_eq!(spacing.resolve(ThemeVariant::Compact), 1);
        assert_eq!(spacing.resolve(ThemeVariant::Comfortable), 2);
        assert_eq!(spacing.resolve(ThemeVariant::Spacious), 3);
    }

    #[test]
    fn test_spacing_xs_compact() {
        // XS should not go below 0
        let spacing = SPACING_XS;
        assert_eq!(spacing.resolve(ThemeVariant::Compact), 0);
    }

    #[test]
    fn test_spacing_from() {
        let spacing: Spacing = 5u16.into();
        assert_eq!(spacing.value, 5);
    }
}
