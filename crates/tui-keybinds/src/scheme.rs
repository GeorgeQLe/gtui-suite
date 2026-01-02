//! Key scheme configuration.

use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

/// Key input scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum KeyScheme {
    /// Only Ctrl+Key, Alt+Key, Shift+Key. No multi-key sequences.
    Simple,

    /// Leader key pattern: leader key starts a chord, followed by single key.
    Leader {
        /// The leader key
        leader: KeyCode,
        /// Timeout in milliseconds
        timeout_ms: u64,
    },

    /// Full chord support like vim's 'gg' or emacs 'C-x C-s'.
    Chords {
        /// Timeout in milliseconds
        timeout_ms: u64,
    },
}

impl Default for KeyScheme {
    fn default() -> Self {
        Self::Simple
    }
}

impl KeyScheme {
    /// Create a leader key scheme with Space as leader.
    pub fn leader_space() -> Self {
        Self::Leader {
            leader: KeyCode::Char(' '),
            timeout_ms: 500,
        }
    }

    /// Create a leader key scheme with a custom leader.
    pub fn leader(leader: KeyCode, timeout_ms: u64) -> Self {
        Self::Leader { leader, timeout_ms }
    }

    /// Create a chord scheme with default timeout.
    pub fn chords() -> Self {
        Self::Chords { timeout_ms: 1000 }
    }

    /// Get the timeout for this scheme.
    pub fn timeout_ms(&self) -> u64 {
        match self {
            Self::Simple => 0,
            Self::Leader { timeout_ms, .. } => *timeout_ms,
            Self::Chords { timeout_ms } => *timeout_ms,
        }
    }

    /// Check if this scheme supports multi-key sequences.
    pub fn supports_sequences(&self) -> bool {
        !matches!(self, Self::Simple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_scheme() {
        let scheme = KeyScheme::default();
        assert!(matches!(scheme, KeyScheme::Simple));
        assert!(!scheme.supports_sequences());
    }

    #[test]
    fn test_leader_scheme() {
        let scheme = KeyScheme::leader_space();
        assert!(scheme.supports_sequences());
        assert_eq!(scheme.timeout_ms(), 500);
    }

    #[test]
    fn test_chords_scheme() {
        let scheme = KeyScheme::chords();
        assert!(scheme.supports_sequences());
        assert_eq!(scheme.timeout_ms(), 1000);
    }
}
