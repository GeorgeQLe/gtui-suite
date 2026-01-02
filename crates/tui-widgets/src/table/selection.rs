//! Selection types for DataTable.

use std::collections::HashSet;

/// Row selection state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Selection {
    /// No selection
    #[default]
    None,
    /// Single row selected
    Single(usize),
    /// Multiple rows selected
    Multi(HashSet<usize>),
}

impl Selection {
    /// Check if a row is selected.
    pub fn contains(&self, index: usize) -> bool {
        match self {
            Self::None => false,
            Self::Single(i) => *i == index,
            Self::Multi(set) => set.contains(&index),
        }
    }

    /// Get the selected indices.
    pub fn indices(&self) -> Vec<usize> {
        match self {
            Self::None => vec![],
            Self::Single(i) => vec![*i],
            Self::Multi(set) => {
                let mut v: Vec<_> = set.iter().copied().collect();
                v.sort();
                v
            }
        }
    }

    /// Get the count of selected items.
    pub fn count(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Single(_) => 1,
            Self::Multi(set) => set.len(),
        }
    }

    /// Check if nothing is selected.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Toggle selection of a single row.
    ///
    /// If single selection, converts to multi-selection.
    pub fn toggle(&mut self, index: usize) {
        match self {
            Self::None => *self = Self::Single(index),
            Self::Single(current) => {
                if *current == index {
                    *self = Self::None;
                } else {
                    let mut set = HashSet::new();
                    set.insert(*current);
                    set.insert(index);
                    *self = Self::Multi(set);
                }
            }
            Self::Multi(set) => {
                if set.contains(&index) {
                    set.remove(&index);
                    if set.len() == 1 {
                        *self = Self::Single(*set.iter().next().unwrap());
                    } else if set.is_empty() {
                        *self = Self::None;
                    }
                } else {
                    set.insert(index);
                }
            }
        }
    }

    /// Select a range of rows (for Shift+Click).
    pub fn select_range(&mut self, from: usize, to: usize) {
        let (start, end) = if from <= to { (from, to) } else { (to, from) };
        let set: HashSet<_> = (start..=end).collect();
        *self = Self::Multi(set);
    }

    /// Clear selection.
    pub fn clear(&mut self) {
        *self = Self::None;
    }

    /// Get the primary selected index (for cursor position).
    pub fn primary(&self) -> Option<usize> {
        match self {
            Self::None => None,
            Self::Single(i) => Some(*i),
            Self::Multi(set) => set.iter().max().copied(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_none() {
        let sel = Selection::None;
        assert!(sel.is_empty());
        assert_eq!(sel.count(), 0);
        assert!(!sel.contains(0));
    }

    #[test]
    fn test_selection_single() {
        let sel = Selection::Single(5);
        assert!(!sel.is_empty());
        assert_eq!(sel.count(), 1);
        assert!(sel.contains(5));
        assert!(!sel.contains(4));
        assert_eq!(sel.indices(), vec![5]);
    }

    #[test]
    fn test_selection_multi() {
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(3);
        set.insert(5);
        let sel = Selection::Multi(set);

        assert_eq!(sel.count(), 3);
        assert!(sel.contains(1));
        assert!(sel.contains(3));
        assert!(sel.contains(5));
        assert!(!sel.contains(2));
        assert_eq!(sel.indices(), vec![1, 3, 5]);
    }

    #[test]
    fn test_selection_toggle() {
        let mut sel = Selection::None;

        sel.toggle(5);
        assert_eq!(sel, Selection::Single(5));

        sel.toggle(3);
        assert!(sel.contains(3));
        assert!(sel.contains(5));
        assert_eq!(sel.count(), 2);

        sel.toggle(5);
        assert_eq!(sel, Selection::Single(3));

        sel.toggle(3);
        assert_eq!(sel, Selection::None);
    }

    #[test]
    fn test_selection_range() {
        let mut sel = Selection::None;
        sel.select_range(2, 5);

        assert_eq!(sel.count(), 4);
        assert!(sel.contains(2));
        assert!(sel.contains(3));
        assert!(sel.contains(4));
        assert!(sel.contains(5));
    }

    #[test]
    fn test_selection_primary() {
        assert_eq!(Selection::None.primary(), None);
        assert_eq!(Selection::Single(5).primary(), Some(5));

        let mut set = HashSet::new();
        set.insert(1);
        set.insert(3);
        set.insert(2);
        assert_eq!(Selection::Multi(set).primary(), Some(3));
    }
}
