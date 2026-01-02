//! # tui-widgets
//!
//! Reusable TUI components for the TUI Suite applications.
//!
//! This crate provides high-quality, accessible UI components that all apps can share,
//! ensuring consistent UX across the suite.
//!
//! ## Components
//!
//! - [`DataTable`] - Sortable, filterable table with virtual scrolling
//! - [`TreeView`] - Expandable hierarchical view with lazy loading
//! - [`FormBuilder`] - Declarative form construction with validation
//! - [`CommandPalette`] - Fuzzy-search command launcher
//!
//! ## Architecture
//!
//! All widgets:
//! - Own their internal state (selection, scroll position, etc.)
//! - Implement Ratatui's `StatefulWidget` trait
//! - Support accessibility features (screen reader hints, sound cues)
//! - Use visual order Tab navigation between widgets

mod accessibility;
mod animation;
mod command;
mod table;
mod tree;
mod form;
mod palette;

pub use accessibility::{Accessible, AccessibilityConfig, SoundCue};
pub use animation::EasingFunction;
pub use command::{Command, CommandError};
pub use table::{
    AggregateFunc, CellContent, Column, ColumnWidth, DataProvider, DataSource, DataTable,
    Selection, SortDirection, TableState,
};
pub use tree::{LoadErrorAction, TreeChildren, TreeNode, TreeState, TreeView};
pub use form::{
    Field, Form, FormBuilder, FormData, FormState, InputType, RowBuilder, Section, Validator,
    Value,
};
pub use palette::{CommandPalette, PaletteState, Parameter};

/// Compact mode setting for widgets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompactMode {
    /// Reduced padding/margins for dense displays
    Compact,
    /// Default spacing
    #[default]
    Comfortable,
    /// Extra spacing for readability
    Spacious,
}

/// Focus indicator style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusIndicator {
    /// Change border color when focused
    BorderColor,
    /// Add decoration to title when focused
    TitleDecoration,
    /// Both border color and title decoration
    #[default]
    Both,
}

/// Common widget configuration
#[derive(Debug, Clone)]
pub struct WidgetConfig {
    /// Compact mode setting
    pub compact_mode: CompactMode,
    /// Focus indicator style
    pub focus_indicator: FocusIndicator,
    /// Whether the widget is disabled
    pub disabled: bool,
    /// Accessibility configuration
    pub accessibility: AccessibilityConfig,
}

impl Default for WidgetConfig {
    fn default() -> Self {
        Self {
            compact_mode: CompactMode::default(),
            focus_indicator: FocusIndicator::default(),
            disabled: false,
            accessibility: AccessibilityConfig::default(),
        }
    }
}
