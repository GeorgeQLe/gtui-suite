//! Cell content types for DataTable.

use ratatui::style::Color;
use std::cmp::Ordering;
use std::fmt;

/// Content that can be displayed in a table cell.
#[derive(Debug, Clone)]
pub enum CellContent {
    /// Plain text
    Text(String),
    /// Numeric value (for sorting)
    Number(f64),
    /// Progress bar
    Progress { value: f32, max: f32 },
    /// Badge with label and color
    Badge { label: String, color: Color },
    /// Sparkline graph
    Sparkline(Vec<f64>),
}

impl CellContent {
    /// Create a text cell.
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    /// Create a number cell.
    pub fn number(n: f64) -> Self {
        Self::Number(n)
    }

    /// Create a progress bar cell.
    pub fn progress(value: f32, max: f32) -> Self {
        Self::Progress { value, max }
    }

    /// Create a badge cell.
    pub fn badge(label: impl Into<String>, color: Color) -> Self {
        Self::Badge {
            label: label.into(),
            color,
        }
    }

    /// Create a sparkline cell.
    pub fn sparkline(values: Vec<f64>) -> Self {
        Self::Sparkline(values)
    }

    /// Get the sortable value for comparison.
    fn sort_key(&self) -> SortKey {
        match self {
            Self::Text(s) => SortKey::Text(s.to_lowercase()),
            Self::Number(n) => SortKey::Number(*n),
            Self::Progress { value, max } => {
                SortKey::Number(if *max == 0.0 { 0.0 } else { *value as f64 / *max as f64 })
            }
            Self::Badge { label, .. } => SortKey::Text(label.to_lowercase()),
            Self::Sparkline(values) => {
                SortKey::Number(values.last().copied().unwrap_or(0.0))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum SortKey {
    Text(String),
    Number(f64),
}

impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for SortKey {}

impl Ord for SortKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (SortKey::Text(a), SortKey::Text(b)) => a.cmp(b),
            (SortKey::Number(a), SortKey::Number(b)) => {
                a.partial_cmp(b).unwrap_or(Ordering::Equal)
            }
            (SortKey::Text(_), SortKey::Number(_)) => Ordering::Greater,
            (SortKey::Number(_), SortKey::Text(_)) => Ordering::Less,
        }
    }
}

impl PartialEq for CellContent {
    fn eq(&self, other: &Self) -> bool {
        self.sort_key() == other.sort_key()
    }
}

impl Eq for CellContent {}

impl PartialOrd for CellContent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CellContent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

impl fmt::Display for CellContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(s) => write!(f, "{}", s),
            Self::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{:.2}", n)
                }
            }
            Self::Progress { value, max } => {
                let pct = if *max == 0.0 { 0.0 } else { value / max * 100.0 };
                write!(f, "{:.0}%", pct)
            }
            Self::Badge { label, .. } => write!(f, "{}", label),
            Self::Sparkline(values) => {
                if let Some(last) = values.last() {
                    write!(f, "{:.1}", last)
                } else {
                    write!(f, "-")
                }
            }
        }
    }
}

impl From<String> for CellContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for CellContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

impl From<f64> for CellContent {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<i32> for CellContent {
    fn from(n: i32) -> Self {
        Self::Number(n as f64)
    }
}

impl From<i64> for CellContent {
    fn from(n: i64) -> Self {
        Self::Number(n as f64)
    }
}

impl From<u32> for CellContent {
    fn from(n: u32) -> Self {
        Self::Number(n as f64)
    }
}

impl From<u64> for CellContent {
    fn from(n: u64) -> Self {
        Self::Number(n as f64)
    }
}

impl From<usize> for CellContent {
    fn from(n: usize) -> Self {
        Self::Number(n as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_display() {
        assert_eq!(CellContent::text("hello").to_string(), "hello");
        assert_eq!(CellContent::number(42.0).to_string(), "42");
        assert_eq!(CellContent::number(3.14159).to_string(), "3.14");
        assert_eq!(
            CellContent::progress(75.0, 100.0).to_string(),
            "75%"
        );
    }

    #[test]
    fn test_cell_ordering() {
        let a = CellContent::number(10.0);
        let b = CellContent::number(20.0);
        assert!(a < b);

        let c = CellContent::text("apple");
        let d = CellContent::text("banana");
        assert!(c < d);
    }

    #[test]
    fn test_cell_from() {
        let _: CellContent = "hello".into();
        let _: CellContent = String::from("world").into();
        let _: CellContent = 42i32.into();
        let _: CellContent = 3.14f64.into();
    }
}
