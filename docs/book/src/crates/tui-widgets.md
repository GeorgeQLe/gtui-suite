# tui-widgets

Reusable TUI components for the application suite.

## Overview

`tui-widgets` provides high-quality, accessible UI components that all apps can share, ensuring consistent UX across the suite.

## Components

| Component | Description |
|-----------|-------------|
| [DataTable](./widgets/datatable.md) | Sortable, filterable table with virtual scrolling |
| [TreeView](./widgets/treeview.md) | Expandable hierarchical view with lazy loading |
| [FormBuilder](./widgets/formbuilder.md) | Declarative form construction with validation |
| [CommandPalette](./widgets/commandpalette.md) | Fuzzy-search command launcher |

## Architecture

### State Management

Widgets own their internal state (selection, scroll position, etc.):

```rust
// Simple API - widgets handle their own state
table.handle_key(event);

// Apps read state via getters
let selected = table.selected();
let offset = table.scroll_offset();
```

### Widget Trait

All widgets implement ratatui's `StatefulWidget` trait for ecosystem compatibility:

```rust
impl StatefulWidget for DataTable<T> {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Rendering logic
    }
}
```

### Focus System

- **Tab** moves between widgets (visual order: left-to-right, top-to-bottom)
- **Arrow keys** / **j/k** navigate within widgets
- All widgets support `.disabled(bool)` - grays out and skips in tab order
- Focus indicators: border color change AND title decoration

### Accessibility

All widgets support:
- Screen reader hints (where terminal supports)
- High contrast mode compatibility
- Full keyboard navigation
- Colorblind-friendly palettes

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
tui-widgets = { workspace = true }
```

Basic example:

```rust
use tui_widgets::{DataTable, Column};

// Define columns
let columns = vec![
    Column::new("Name", |item: &User| item.name.clone().into()),
    Column::new("Email", |item: &User| item.email.clone().into()),
];

// Create table with data
let table = DataTable::new(columns, users);

// Handle input
table.handle_key(event);

// Render
frame.render_stateful_widget(table, area, &mut state);
```

## Animation

Widgets support smooth animations with configurable easing:

```rust
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    CubicBezier(f32, f32, f32, f32),
}
```

Animations are event-driven (on-demand tick) for efficiency.
