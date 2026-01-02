# tui-theme

Full-featured theming engine for the TUI application suite.

## Purpose

Provide consistent, customizable styling across all apps with accessibility support.

## Architecture Decisions

### Color Depth Support
- **Multiple palettes**: Theme author provides separate palettes for 24-bit, 256-color, and 16-color terminals
- System auto-detects terminal capabilities and uses appropriate palette

### Theme Inheritance & Overrides
- **Layered overrides** with **theme inheritance**
- Base theme + user overrides merged at runtime (CSS cascade style)
- Themes can extend other themes, overriding specific values
- Example: `OneDarkVivid` extends `OneDark`, overrides accent colors

### Terminal Capability Detection
- **Auto-detection**: Detect Unicode/color support, auto-select appropriate styles
- Themes define multiple variants (Unicode borders vs ASCII, true color vs 256)

### Hot Reload
- Watch theme files for changes, auto-reload during development
- Useful for theme authors; can be disabled in production

## Theme Discovery

**Layered search** (first match wins):
```
1. ~/.config/{app-name}/themes/{theme}.toml    # App-specific override
2. ~/.config/tui-suite/themes/{theme}.toml     # User shared themes
3. /usr/share/tui-suite/themes/{theme}.toml    # System-wide (package installed)
```

## Features

### Semantic Colors

Define colors by purpose with full style information:

```rust
pub struct ColorToken {
    pub color: Color,
    pub modifiers: Modifier,  // Bold, Italic, Underline, Dim, etc.
}

pub struct ColorPalette {
    // Color depth variants
    pub true_color: ColorSet,
    pub color_256: ColorSet,
    pub color_16: ColorSet,
}

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
```

### State-Level Widget Styles

Full state matrix for granular control:

```rust
pub struct WidgetStyle {
    pub fg: ColorToken,
    pub bg: ColorToken,
    pub border_style: BorderStyle,
    pub border_color: ColorToken,
    pub padding: Spacing,
    pub modifiers: Modifier,
}

pub struct StateStyles {
    pub default: WidgetStyle,
    pub hover: WidgetStyle,
    pub focused: WidgetStyle,
    pub pressed: WidgetStyle,
    pub disabled: WidgetStyle,
    pub selected: WidgetStyle,
    pub selected_focused: WidgetStyle,
}

pub struct TableStyles {
    pub container: StateStyles,
    pub header: StateStyles,
    pub row: StateStyles,
    pub cell: StateStyles,
    pub selected_row: StateStyles,
    pub footer: StateStyles,
}

pub struct BorderStyle {
    pub style: BorderType,  // Plain, Rounded, Double, Thick
    pub visible: Sides,     // Top, Bottom, Left, Right
    pub unicode: String,    // Unicode border chars
    pub ascii: String,      // ASCII fallback
}
```

### Spacing System

Combined absolute and relative spacing:

```rust
pub struct Spacing {
    pub value: u16,  // Base value in cells
}

impl Spacing {
    pub fn resolve(&self, variant: ThemeVariant) -> u16 {
        match variant {
            ThemeVariant::Compact => self.value.saturating_sub(1).max(0),
            ThemeVariant::Comfortable => self.value,
            ThemeVariant::Spacious => self.value + 1,
        }
    }
}

// T-shirt size helpers
pub const SPACING_XS: Spacing = Spacing { value: 0 };
pub const SPACING_SM: Spacing = Spacing { value: 1 };
pub const SPACING_MD: Spacing = Spacing { value: 2 };
pub const SPACING_LG: Spacing = Spacing { value: 3 };
pub const SPACING_XL: Spacing = Spacing { value: 4 };
```

### Theme Variants

Named variants for different contexts:

```rust
pub enum ThemeVariant {
    Compact,      // Minimal padding, dense information
    Comfortable,  // Balanced (default)
    Spacious,     // More padding, larger spacing
}
```

### Animation Timing

Theme-controlled animation parameters:

```rust
pub struct AnimationConfig {
    pub duration_fast: Duration,    // e.g., 100ms
    pub duration_normal: Duration,  // e.g., 200ms
    pub duration_slow: Duration,    // e.g., 400ms
    pub easing: EasingFunction,     // Default easing for this theme
}
```

**Note**: `EasingFunction` is defined in tui-widgets and re-exported here. See `tui-widgets/SPEC.md` for the enum definition including `CubicBezier(f32, f32, f32, f32)` variant for custom curves.

### Built-in Presets

Compiled as Rust structs for performance:

- **Default Light/Dark**: Carefully designed default themes
- **Base16**: All Base16 color schemes
- **Catppuccin**: Latte, Frappe, Macchiato, Mocha
- **Nord**: Nord color palette
- **Solarized**: Light and Dark
- **Gruvbox**: Light and Dark
- **Dracula**
- **One Dark**

```rust
pub fn builtin_themes() -> HashMap<&'static str, Theme> {
    // Returns all built-in themes as compiled Rust structs
}
```

### High Contrast Themes

**Hybrid approach**:
- Pre-made high-contrast variants for built-in themes (manually crafted, tested)
- "Increase contrast" filter applicable to any user theme

```rust
pub struct HighContrastConfig {
    pub min_contrast_ratio: f32,  // WCAG AA = 4.5, AAA = 7.0
    pub boost_factor: f32,        // How much to increase contrast
}

impl Theme {
    pub fn to_high_contrast(&self, config: HighContrastConfig) -> Theme;
}
```

### Colorblind Accessibility

**Both approaches**:
- Pre-made colorblind-friendly variants for built-in themes (tested)
- Daltonization filter for user themes

```rust
pub enum ColorblindMode {
    Deuteranopia,   // Red-green (most common)
    Protanopia,     // Red-green
    Tritanopia,     // Blue-yellow
}

impl Theme {
    pub fn to_colorblind(&self, mode: ColorblindMode) -> Theme;
}
```

### Syntax Highlighting (Optional Extension)

Separate trait for apps that display code:

```rust
pub trait SyntaxTheme {
    fn keyword(&self) -> ColorToken;
    fn string(&self) -> ColorToken;
    fn comment(&self) -> ColorToken;
    fn function(&self) -> ColorToken;
    fn type_name(&self) -> ColorToken;
    fn number(&self) -> ColorToken;
    fn operator(&self) -> ColorToken;
    fn punctuation(&self) -> ColorToken;
    fn variable(&self) -> ColorToken;
    fn constant(&self) -> ColorToken;
}

// Built-in themes implement SyntaxTheme
// User themes can optionally include [syntax] section
```

### User Themes

Load themes from TOML config files:

```toml
# ~/.config/tui-suite/themes/my-theme.toml
[meta]
name = "My Theme"
extends = "one-dark"  # Optional inheritance

[colors.true_color]
bg_primary = { color = "#1a1b26", bold = false }
bg_secondary = { color = "#24283b" }
fg_primary = { color = "#c0caf5" }
accent = { color = "#7aa2f7", bold = true }
error = { color = "#f7768e" }

[colors.color_256]
# 256-color palette
bg_primary = { color = 234 }
fg_primary = { color = 253 }

[colors.color_16]
# 16-color palette
bg_primary = { color = "black" }
fg_primary = { color = "white" }

[styles.table.row.hover]
bg = "bg_hover"
fg = "fg_primary"

[styles.table.row.selected]
bg = "accent"
fg = "bg_primary"
modifiers = ["bold"]

[animation]
duration_fast = 100
duration_normal = 200
duration_slow = 400
easing = "ease-out"

[variant]
default = "comfortable"

[syntax]  # Optional
keyword = { color = "#bb9af7", bold = true }
string = { color = "#9ece6a" }
comment = { color = "#565f89", italic = true }
```

### Theme Validation

**Warning + fallback** approach:
- Log warning for missing tokens
- Use built-in defaults for missing values
- Theme still loads and works

```rust
pub struct ValidationResult {
    pub warnings: Vec<ThemeWarning>,
    pub errors: Vec<ThemeError>,  // Only for syntax errors
}

pub enum ThemeWarning {
    MissingToken { path: String, default_used: String },
    DeprecatedToken { path: String, replacement: String },
}
```

### Runtime Switching

Change themes without restart:

```rust
pub fn switch_theme(theme_name: &str) -> Result<()>;
pub fn list_themes() -> Vec<ThemeMeta>;
pub fn current_theme() -> &Theme;
pub fn apply_overlay(overrides: ThemeOverrides) -> Result<()>;

// File watcher for hot reload
pub fn watch_theme_file(path: &Path, on_change: impl Fn(&Theme));
```

## API

```rust
pub struct Theme {
    pub name: String,
    pub extends: Option<String>,
    pub colors: ColorPalette,
    pub styles: StyleMap,
    pub variant: ThemeVariant,
    pub animation: AnimationConfig,
}

pub struct StyleMap {
    pub table: TableStyles,
    pub tree: TreeStyles,
    pub form: FormStyles,
    pub palette: PaletteStyles,
    pub dialog: DialogStyles,
    pub statusbar: StatusBarStyles,
    pub tabs: TabStyles,
}

pub struct ThemeManager {
    builtin: HashMap<String, Theme>,
    user: HashMap<String, Theme>,
    current: String,
    overrides: ThemeOverrides,
    watcher: Option<FileWatcher>,
}

impl ThemeManager {
    pub fn new() -> Self;
    pub fn load_user_themes(&mut self, search_paths: &[PathBuf]) -> ValidationResult;
    pub fn get(&self, name: &str) -> Option<&Theme>;
    pub fn current(&self) -> &Theme;
    pub fn set_current(&mut self, name: &str) -> Result<()>;
    pub fn apply_overrides(&mut self, overrides: ThemeOverrides);
    pub fn enable_hot_reload(&mut self, paths: &[PathBuf]);
}

// Usage in apps
pub fn apply_style(frame: &mut Frame, area: Rect, style: &WidgetStyle) {
    let ratatui_style = style.to_ratatui_style();
    frame.set_style(area, ratatui_style);
}
```

## Configuration

Apps specify theme preferences in their config:

```toml
# ~/.config/app-name/config.toml
[theme]
name = "catppuccin-mocha"
variant = "compact"
colorblind_mode = "deuteranopia"  # Optional
high_contrast = false

[theme.overrides]
# Override specific colors
accent = "#ff79c6"

[theme.animation]
enabled = true
duration_multiplier = 1.0  # Speed up or slow down
```

## Dependencies

```toml
[dependencies]
ratatui = { workspace = true }
serde = { workspace = true }
toml = { workspace = true }
directories = "5"
notify = "6"  # For hot reload file watching
```
