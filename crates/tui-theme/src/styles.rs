//! Widget style definitions.

use crate::colors::{ColorToken, deserialize_modifier, serialize_modifier};
use crate::spacing::Spacing;
use ratatui::style::Modifier;
use serde::{Deserialize, Serialize};

/// Border type options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BorderType {
    /// Simple single lines
    #[default]
    Plain,
    /// Rounded corners
    Rounded,
    /// Double lines
    Double,
    /// Thick lines
    Thick,
    /// No border
    None,
}

/// Which sides have borders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Sides {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl Sides {
    /// All sides.
    pub const ALL: Self = Self {
        top: true,
        bottom: true,
        left: true,
        right: true,
    };

    /// No sides.
    pub const NONE: Self = Self {
        top: false,
        bottom: false,
        left: false,
        right: false,
    };

    /// Horizontal sides only.
    pub const HORIZONTAL: Self = Self {
        top: true,
        bottom: true,
        left: false,
        right: false,
    };

    /// Vertical sides only.
    pub const VERTICAL: Self = Self {
        top: false,
        bottom: false,
        left: true,
        right: true,
    };
}

/// Border style configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BorderStyle {
    /// Border type
    pub style: BorderType,
    /// Which sides are visible
    pub visible: Sides,
    /// Unicode border characters
    pub unicode: Option<String>,
    /// ASCII fallback characters
    pub ascii: Option<String>,
}

impl BorderStyle {
    /// Create a plain border on all sides.
    pub fn plain() -> Self {
        Self {
            style: BorderType::Plain,
            visible: Sides::ALL,
            unicode: None,
            ascii: None,
        }
    }

    /// Create a rounded border on all sides.
    pub fn rounded() -> Self {
        Self {
            style: BorderType::Rounded,
            visible: Sides::ALL,
            unicode: None,
            ascii: None,
        }
    }

    /// Create a border with no visible sides.
    pub fn none() -> Self {
        Self {
            style: BorderType::None,
            visible: Sides::NONE,
            unicode: None,
            ascii: None,
        }
    }
}

/// Complete widget style.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WidgetStyle {
    /// Foreground color
    pub fg: ColorToken,
    /// Background color
    pub bg: ColorToken,
    /// Border style
    #[serde(default)]
    pub border_style: BorderStyle,
    /// Border color
    pub border_color: ColorToken,
    /// Padding
    #[serde(default)]
    pub padding: Spacing,
    /// Text modifiers
    #[serde(default, serialize_with = "serialize_modifier", deserialize_with = "deserialize_modifier")]
    pub modifiers: Modifier,
}

impl WidgetStyle {
    /// Convert to Ratatui style.
    pub fn to_ratatui_style(&self) -> ratatui::style::Style {
        let mut style = ratatui::style::Style::default();

        if let Some(fg) = self.fg.color.to_ratatui() {
            style = style.fg(fg);
        }
        if let Some(bg) = self.bg.color.to_ratatui() {
            style = style.bg(bg);
        }

        style = style.add_modifier(self.modifiers);
        style = style.add_modifier(self.fg.modifiers);

        style
    }
}

/// Styles for different widget states.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateStyles {
    pub default: WidgetStyle,
    pub hover: WidgetStyle,
    pub focused: WidgetStyle,
    pub pressed: WidgetStyle,
    pub disabled: WidgetStyle,
    pub selected: WidgetStyle,
    pub selected_focused: WidgetStyle,
}

/// Table widget styles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableStyles {
    pub container: StateStyles,
    pub header: StateStyles,
    pub row: StateStyles,
    pub cell: StateStyles,
    pub selected_row: StateStyles,
    pub footer: StateStyles,
}

/// Tree widget styles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TreeStyles {
    pub container: StateStyles,
    pub node: StateStyles,
    pub icon: StateStyles,
    pub label: StateStyles,
}

/// Form widget styles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormStyles {
    pub container: StateStyles,
    pub label: StateStyles,
    pub input: StateStyles,
    pub button: StateStyles,
    pub error: StateStyles,
}

/// Command palette styles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaletteStyles {
    pub container: StateStyles,
    pub input: StateStyles,
    pub item: StateStyles,
    pub category: StateStyles,
    pub shortcut: StateStyles,
}

/// Dialog styles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogStyles {
    pub container: StateStyles,
    pub title: StateStyles,
    pub content: StateStyles,
    pub button: StateStyles,
}

/// Status bar styles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusBarStyles {
    pub container: StateStyles,
    pub segment: StateStyles,
    pub active_segment: StateStyles,
}

/// Tab styles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TabStyles {
    pub container: StateStyles,
    pub tab: StateStyles,
    pub active_tab: StateStyles,
    pub separator: StateStyles,
}

/// Complete style map for all widgets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleMap {
    pub table: TableStyles,
    pub tree: TreeStyles,
    pub form: FormStyles,
    pub palette: PaletteStyles,
    pub dialog: DialogStyles,
    pub statusbar: StatusBarStyles,
    pub tabs: TabStyles,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_sides() {
        let all = Sides::ALL;
        assert!(all.top && all.bottom && all.left && all.right);

        let none = Sides::NONE;
        assert!(!none.top && !none.bottom && !none.left && !none.right);
    }

    #[test]
    fn test_border_style() {
        let plain = BorderStyle::plain();
        assert_eq!(plain.style, BorderType::Plain);
        assert_eq!(plain.visible, Sides::ALL);

        let none = BorderStyle::none();
        assert_eq!(none.style, BorderType::None);
    }

    #[test]
    fn test_widget_style_to_ratatui() {
        use crate::colors::Color;

        let style = WidgetStyle {
            fg: ColorToken::new(Color::hex("#ffffff")),
            bg: ColorToken::new(Color::hex("#000000")),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };

        let ratatui_style = style.to_ratatui_style();
        assert!(ratatui_style.fg.is_some());
        assert!(ratatui_style.bg.is_some());
    }
}
