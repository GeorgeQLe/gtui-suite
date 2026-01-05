//! Color types and palettes.

use ratatui::style::{Color as RatatuiColor, Modifier};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serialize Modifier as a list of strings
pub fn serialize_modifier<S>(modifier: &Modifier, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut names = Vec::new();
    if modifier.contains(Modifier::BOLD) { names.push("bold"); }
    if modifier.contains(Modifier::DIM) { names.push("dim"); }
    if modifier.contains(Modifier::ITALIC) { names.push("italic"); }
    if modifier.contains(Modifier::UNDERLINED) { names.push("underlined"); }
    if modifier.contains(Modifier::SLOW_BLINK) { names.push("slow_blink"); }
    if modifier.contains(Modifier::RAPID_BLINK) { names.push("rapid_blink"); }
    if modifier.contains(Modifier::REVERSED) { names.push("reversed"); }
    if modifier.contains(Modifier::HIDDEN) { names.push("hidden"); }
    if modifier.contains(Modifier::CROSSED_OUT) { names.push("crossed_out"); }
    names.serialize(serializer)
}

/// Deserialize Modifier from a list of strings
pub fn deserialize_modifier<'de, D>(deserializer: D) -> Result<Modifier, D::Error>
where
    D: Deserializer<'de>,
{
    let names: Vec<String> = Vec::deserialize(deserializer)?;
    let mut modifier = Modifier::empty();
    for name in names {
        match name.to_lowercase().as_str() {
            "bold" => modifier |= Modifier::BOLD,
            "dim" => modifier |= Modifier::DIM,
            "italic" => modifier |= Modifier::ITALIC,
            "underlined" | "underline" => modifier |= Modifier::UNDERLINED,
            "slow_blink" => modifier |= Modifier::SLOW_BLINK,
            "rapid_blink" => modifier |= Modifier::RAPID_BLINK,
            "reversed" | "reverse" => modifier |= Modifier::REVERSED,
            "hidden" => modifier |= Modifier::HIDDEN,
            "crossed_out" | "strikethrough" => modifier |= Modifier::CROSSED_OUT,
            _ => {}
        }
    }
    Ok(modifier)
}

/// Terminal color depth capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorDepth {
    /// 24-bit true color (16 million colors)
    #[default]
    TrueColor,
    /// 256-color palette
    Color256,
    /// 16-color ANSI
    Color16,
}

impl ColorDepth {
    /// Detect terminal color capability from environment.
    pub fn detect() -> Self {
        if let Ok(term) = std::env::var("COLORTERM") {
            if term == "truecolor" || term == "24bit" {
                return Self::TrueColor;
            }
        }

        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") {
                return Self::Color256;
            }
        }

        Self::Color16
    }
}

/// A color value that can be specified in multiple formats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    /// Hex color (e.g., "#1a1b26")
    Hex(String),
    /// RGB values
    Rgb { r: u8, g: u8, b: u8 },
    /// 256-color index
    Index(u8),
    /// Named ANSI color
    Named(String),
    /// No color (transparent/inherit)
    None,
}

impl Color {
    /// Create a hex color.
    pub fn hex(s: impl Into<String>) -> Self {
        Self::Hex(s.into())
    }

    /// Create an RGB color.
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb { r, g, b }
    }

    /// Create an indexed color.
    pub fn index(i: u8) -> Self {
        Self::Index(i)
    }

    /// Create a named color.
    pub fn named(s: impl Into<String>) -> Self {
        Self::Named(s.into())
    }

    /// Convert to Ratatui color.
    pub fn to_ratatui(&self) -> Option<RatatuiColor> {
        match self {
            Self::Hex(s) => {
                let s = s.trim_start_matches('#');
                if s.len() == 6 {
                    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                    Some(RatatuiColor::Rgb(r, g, b))
                } else {
                    None
                }
            }
            Self::Rgb { r, g, b } => Some(RatatuiColor::Rgb(*r, *g, *b)),
            Self::Index(i) => Some(RatatuiColor::Indexed(*i)),
            Self::Named(name) => match name.to_lowercase().as_str() {
                "black" => Some(RatatuiColor::Black),
                "red" => Some(RatatuiColor::Red),
                "green" => Some(RatatuiColor::Green),
                "yellow" => Some(RatatuiColor::Yellow),
                "blue" => Some(RatatuiColor::Blue),
                "magenta" => Some(RatatuiColor::Magenta),
                "cyan" => Some(RatatuiColor::Cyan),
                "white" => Some(RatatuiColor::White),
                "gray" | "grey" => Some(RatatuiColor::Gray),
                "darkgray" | "darkgrey" => Some(RatatuiColor::DarkGray),
                "lightred" => Some(RatatuiColor::LightRed),
                "lightgreen" => Some(RatatuiColor::LightGreen),
                "lightyellow" => Some(RatatuiColor::LightYellow),
                "lightblue" => Some(RatatuiColor::LightBlue),
                "lightmagenta" => Some(RatatuiColor::LightMagenta),
                "lightcyan" => Some(RatatuiColor::LightCyan),
                _ => None,
            },
            Self::None => None,
        }
    }

    /// Parse to RGB values for color manipulation.
    pub fn to_rgb(&self) -> Option<(u8, u8, u8)> {
        match self {
            Self::Hex(s) => {
                let s = s.trim_start_matches('#');
                if s.len() == 6 {
                    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                    Some((r, g, b))
                } else {
                    None
                }
            }
            Self::Rgb { r, g, b } => Some((*r, *g, *b)),
            _ => None,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::None
    }
}

/// A color token with optional modifiers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorToken {
    /// The color value
    pub color: Color,
    /// Text modifiers (bold, italic, etc.)
    #[serde(default, serialize_with = "serialize_modifier", deserialize_with = "deserialize_modifier")]
    pub modifiers: Modifier,
}

impl ColorToken {
    /// Create a new color token.
    pub fn new(color: Color) -> Self {
        Self {
            color,
            modifiers: Modifier::empty(),
        }
    }

    /// Add bold modifier.
    pub fn bold(mut self) -> Self {
        self.modifiers |= Modifier::BOLD;
        self
    }

    /// Add italic modifier.
    pub fn italic(mut self) -> Self {
        self.modifiers |= Modifier::ITALIC;
        self
    }

    /// Add underline modifier.
    pub fn underlined(mut self) -> Self {
        self.modifiers |= Modifier::UNDERLINED;
        self
    }

    /// Add dim modifier.
    pub fn dim(mut self) -> Self {
        self.modifiers |= Modifier::DIM;
        self
    }
}

impl From<Color> for ColorToken {
    fn from(color: Color) -> Self {
        Self::new(color)
    }
}

/// Complete color palette with all semantic tokens.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorPalette {
    /// True color (24-bit) palette
    pub true_color: ColorSet,
    /// 256-color palette
    pub color_256: ColorSet,
    /// 16-color ANSI palette
    pub color_16: ColorSet,
}

impl ColorPalette {
    /// Apply high contrast boost to all colors.
    pub fn with_increased_contrast(mut self, config: crate::HighContrastConfig) -> Self {
        self.true_color = self.true_color.with_increased_contrast(config);
        self.color_256 = self.color_256.with_increased_contrast(config);
        self.color_16 = self.color_16.with_increased_contrast(config);
        self
    }

    /// Apply colorblind filter to all colors.
    pub fn with_colorblind_filter(mut self, mode: crate::ColorblindMode) -> Self {
        self.true_color = self.true_color.with_colorblind_filter(mode);
        self.color_256 = self.color_256.with_colorblind_filter(mode);
        self.color_16 = self.color_16.with_colorblind_filter(mode);
        self
    }
}

/// A set of semantic color tokens.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorSet {
    // Backgrounds
    pub bg_primary: ColorToken,
    pub bg_secondary: ColorToken,
    pub bg_tertiary: ColorToken,
    pub bg_hover: ColorToken,
    pub bg_focused: ColorToken,
    pub bg_pressed: ColorToken,
    pub bg_disabled: ColorToken,

    // Foregrounds
    pub fg_primary: ColorToken,
    pub fg_secondary: ColorToken,
    pub fg_muted: ColorToken,
    pub fg_disabled: ColorToken,

    // Accents
    pub accent: ColorToken,
    pub accent_secondary: ColorToken,
    pub accent_hover: ColorToken,
    pub accent_focused: ColorToken,

    // Semantic
    pub success: ColorToken,
    pub warning: ColorToken,
    pub error: ColorToken,
    pub info: ColorToken,

    // Borders
    pub border: ColorToken,
    pub border_focused: ColorToken,
    pub border_error: ColorToken,

    // Animation
    pub flash_success: ColorToken,
    pub flash_error: ColorToken,
}

impl ColorSet {
    /// Apply high contrast to this color set.
    pub fn with_increased_contrast(self, _config: crate::HighContrastConfig) -> Self {
        // In a full implementation, this would analyze contrast ratios
        // and boost colors that don't meet the minimum
        self
    }

    /// Apply colorblind filter.
    pub fn with_colorblind_filter(self, _mode: crate::ColorblindMode) -> Self {
        // In a full implementation, this would apply daltonization
        // to transform colors for better colorblind visibility
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_color() {
        let color = Color::hex("#ff5500");
        assert!(color.to_ratatui().is_some());
        assert_eq!(color.to_rgb(), Some((255, 85, 0)));
    }

    #[test]
    fn test_rgb_color() {
        let color = Color::rgb(100, 150, 200);
        assert_eq!(color.to_ratatui(), Some(RatatuiColor::Rgb(100, 150, 200)));
    }

    #[test]
    fn test_named_color() {
        let color = Color::named("red");
        assert_eq!(color.to_ratatui(), Some(RatatuiColor::Red));

        let color = Color::named("Blue");
        assert_eq!(color.to_ratatui(), Some(RatatuiColor::Blue));
    }

    #[test]
    fn test_color_token_modifiers() {
        let token = ColorToken::new(Color::hex("#ffffff"))
            .bold()
            .italic();

        assert!(token.modifiers.contains(Modifier::BOLD));
        assert!(token.modifiers.contains(Modifier::ITALIC));
    }
}
