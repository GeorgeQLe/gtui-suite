# tui-widgets

Reusable TUI components for the application suite.

## Purpose

Provide high-quality, accessible UI components that all apps can share, ensuring consistent UX across the suite.

## Architecture Decisions

### State Management
- **Internal state**: Widgets own their state (selection, scroll position, etc.)
- Simple API: `table.handle_event(e)` mutates internal state
- Apps read state via getters: `table.selected()`, `table.scroll_offset()`

### Widget Trait
- Implements Ratatui's `StatefulWidget` trait for ecosystem compatibility
- Rich lifecycle events handled internally

### Focus System
- **Visual order navigation**: Tab follows left-to-right, top-to-bottom visual positions
- Tab moves between widgets; arrows/j/k navigate within widgets
- Exception: FormBuilder uses Tab to move between fields (web form convention)
- All widgets support `.disabled(bool)` - grays out and skips in tab order
- Focus indicators: Border color change AND title decoration (configurable)

### Animation System
- **Full animation support** with on-demand tick (event-driven when idle)
- Primary use: smooth scrolling, expand/collapse transitions, selection highlights

**EasingFunction Type:**
```rust
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    CubicBezier(f32, f32, f32, f32),  // Custom control points
}
```
- Enum with named variants for common curves
- `CubicBezier` variant for custom curves (CSS-style control points)
- Used by tui-theme's AnimationConfig

### Command Trait
- **Shared trait for CommandPalette integration**
- Apps implement `Command` trait for their actions
- Shell wraps app commands for unified launcher

```rust
pub trait Command: Send + Sync {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn keywords(&self) -> &[&str];
    fn execute(&self, params: HashMap<String, String>) -> Result<(), CommandError>;
}
```

### Compact Mode
- Theme variant sets default (Compact, Comfortable, Spacious)
- Per-widget `.compact(bool)` override available

## Components

### DataTable

Sortable, filterable table with virtual scrolling.

**Features:**
- Column definitions with type hints (string, number, date)
- Sortable columns (click header or keyboard shortcut)
- Filterable rows (per-column or global)
- Virtual scrolling for large datasets (10k+ rows)
- Column resize: keyboard by default, mouse drag opt-in via feature flag
- Row selection: Full Excel-style (Shift+Click range, Ctrl+Click toggle, Ctrl+A all) - configurable
- Keyboard navigation (j/k, arrows, Page Up/Down)
- Configurable row height
- Footer with summary stats (global + per-column aggregates, toggleable)

**Scroll Behavior:**
- Hybrid cursor/position stability: cursor stability when selected row visible, position stability otherwise
- Best-effort performance optimization (no strict frame budget)

**Sort Behavior:**
- Selection follows data: if row "Alice" is selected, "Alice" stays selected in new sorted position

**Cell Rendering:**
- Rich cell widgets supported: ProgressBar, Badge, Sparkline, custom widgets
- Inline edit mode: Enter on cell to edit, Escape to cancel
- Content overflow: Truncate with ellipsis, show full content in tooltip on focus/hover

**Clipboard:**
- Default format: TSV (tab-separated values) for Excel/Sheets compatibility
- Configurable per-table: TSV, CSV, JSON, Markdown

**Data Loading:**
- Simple mode: `Vec<T>` with all data in memory
- Advanced mode: `DataProvider` trait for paginated/virtualized fetch

**API:**
```rust
pub struct DataTable<T> {
    columns: Vec<Column<T>>,
    data: DataSource<T>,  // Vec<T> or DataProvider impl
    state: TableState,
}

pub struct TableState {
    selected: Selection,  // None, Single(usize), Multi(HashSet<usize>)
    sort_column: Option<usize>,
    sort_direction: SortDirection,
    filter: Option<String>,
    scroll_offset: usize,
    edit_cell: Option<(usize, usize)>,  // (row, col) being edited
}

pub struct Column<T> {
    header: String,
    width: ColumnWidth,
    accessor: fn(&T) -> CellContent,  // Returns rich cell content
    sortable: bool,
    filterable: bool,
    aggregate: Option<AggregateFunc>,  // Sum, Avg, Min, Max, Count, Custom
}

pub enum CellContent {
    Text(String),
    Number(f64),
    Progress { value: f32, max: f32 },
    Badge { label: String, color: Color },
    Sparkline(Vec<f64>),
    Custom(Box<dyn Widget>),
}

// Event callbacks
impl<T> DataTable<T> {
    pub fn on_select(self, f: impl Fn(&T)) -> Self;
    pub fn on_delete(self, f: impl Fn(Vec<&T>)) -> Self;
    pub fn on_edit(self, f: impl Fn(&T, &str, String)) -> Self;  // row, column, new_value
    pub fn on_sort(self, f: impl Fn(usize, SortDirection)) -> Self;
}
```

### TreeView

Expandable hierarchical view with lazy loading.

**Features:**
- Nested nodes with expand/collapse
- Lazy loading with configurable error handling
- Search within tree (highlights matches)
- Keyboard navigation (j/k for up/down, h/l AND Enter for collapse/expand)
- Customizable node rendering
- Selection support
- Nerd Font icons built-in (no fallback, user ensures font is available)

**Deep Nesting Handling:**
- Indent capping at 8 levels with depth indicator: `[12] NodeName`
- Breadcrumb mode toggle: shows path bar + flat list of current level

**Lazy Load Error Handling:**
- Configurable via callback with **default: inline error + retry**
- Options: inline error, toast notification, empty with indicator

**API:**
```rust
pub trait TreeNode {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    fn children(&self) -> TreeChildren;  // Sync or Async
    fn is_expandable(&self) -> bool;
    fn icon(&self) -> Option<&str>;  // Nerd Font icon
}

pub enum TreeChildren {
    Loaded(Vec<Box<dyn TreeNode>>),
    Lazy(Box<dyn Fn() -> Future<Output = Result<Vec<Box<dyn TreeNode>>>>>),
}

pub struct TreeView<T: TreeNode> {
    root: T,
    state: TreeState,
    on_load_error: Box<dyn Fn(Error) -> LoadErrorAction>,
}

pub struct TreeState {
    expanded: HashSet<String>,
    selected: Option<String>,
    search_query: Option<String>,
    breadcrumb_mode: bool,
    focus_path: Vec<String>,  // For breadcrumb navigation
}

pub enum LoadErrorAction {
    InlineError { message: String, retry: bool },
    Toast { message: String },
    EmptyPlaceholder { message: String },
}

// Event callbacks
impl<T: TreeNode> TreeView<T> {
    pub fn on_select(self, f: impl Fn(&T)) -> Self;
    pub fn on_expand(self, f: impl Fn(&T)) -> Self;
    pub fn on_collapse(self, f: impl Fn(&T)) -> Self;
}
```

### FormBuilder

Declarative form construction with validation.

**Features:**
- Input types: text, password, number, date, select, multi-select, checkbox, radio
- Validation rules (required, min/max length, regex, custom, async)
- Cross-field validation with full form context
- Inline error display with debounced async validation (300ms)
- Tab navigation between fields (form convention)
- Submit blocks until all async validations complete
- Builder pattern layout API with sections
- Disabled state support
- Default values
- Password masking: Unicode bullet (•) with fallback to asterisk if terminal doesn't support

**Layout API:**
```rust
pub struct FormBuilder {
    sections: Vec<Section>,
}

impl FormBuilder {
    pub fn new() -> Self;

    pub fn row(self, f: impl FnOnce(RowBuilder) -> RowBuilder) -> Self;

    pub fn section(self, title: &str, f: impl FnOnce(FormBuilder) -> FormBuilder) -> Self;

    pub fn build(self) -> Form;
}

pub struct RowBuilder {
    fields: Vec<Field>,
}

impl RowBuilder {
    pub fn field(self, field: Field) -> Self;
    pub fn full_width(self) -> Self;  // Span entire row
}

// Example usage:
let form = FormBuilder::new()
    .row(|r| r.field(username).field(email))
    .row(|r| r.field(password).field(confirm_password))
    .section("Address", |s| {
        s.row(|r| r.field(street).full_width())
         .row(|r| r.field(city).field(state).field(zip))
    })
    .build();
```

**Validation API:**
```rust
pub struct Field {
    name: String,
    label: String,
    input_type: InputType,
    validators: Vec<Validator>,
    default_value: Option<Value>,
    disabled: bool,
}

pub enum Validator {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Regex(Regex),
    Custom(Box<dyn Fn(&Value) -> Result<(), String>>),
    Async(Box<dyn Fn(&Value) -> Future<Output = Result<(), String>>>),
    CrossField(Box<dyn Fn(&Value, &FormData) -> Result<(), String>>),  // Full context
}

pub struct Form {
    builder: FormBuilder,
    state: FormState,
}

impl Form {
    pub fn on_submit(self, f: impl Fn(FormData)) -> Self;
    pub fn on_cancel(self, f: impl Fn()) -> Self;
    pub fn on_change(self, f: impl Fn(&str, &Value)) -> Self;  // field_name, new_value
}
```

### CommandPalette

Fuzzy-search command launcher.

**Features:**
- Pure fuzzy matching via nucleo-matcher (VS Code-style subsequence matching)
- Categorized commands
- Recent commands persisted to SQLite
- Keyboard shortcut display alongside commands
- Scrollable results
- Preview of selected command
- Multi-step wizard for commands with parameters
- Customizable trigger key (default: Ctrl+P)

**API:**
```rust
pub struct CommandPalette {
    commands: Vec<Command>,
    state: PaletteState,
    db: SqliteConnection,  // For recent commands
}

pub struct PaletteState {
    query: String,
    selected_index: usize,
    wizard_step: Option<WizardState>,
}

pub struct Command {
    id: String,
    label: String,
    category: Option<String>,
    shortcut: Option<KeyBinding>,
    parameters: Vec<Parameter>,  // For multi-step wizard
    action: Box<dyn Fn(HashMap<String, Value>)>,
}

pub struct Parameter {
    name: String,
    label: String,
    input_type: InputType,
    required: bool,
    default: Option<Value>,
}

// Wizard flow:
// 1. User selects "Go to Line" command
// 2. Palette shows: "Line number:" with text input
// 3. User types "42", presses Enter
// 4. Command executes with { "line": 42 }

impl CommandPalette {
    pub fn on_execute(self, f: impl Fn(&Command, HashMap<String, Value>)) -> Self;
}
```

## Accessibility

All widgets support:
- Screen reader hints (ARIA-like roles and states for terminals that support it)
- Sound cues option (audio feedback for actions - select, error, complete)
- Text-based announcements (dedicated status line for screen reader polling)
- High contrast mode compatibility
- Large text mode (reduced density via theme variant)
- Keyboard-only navigation (full functionality without mouse)

```rust
pub struct AccessibilityConfig {
    pub screen_reader_hints: bool,
    pub sound_cues: bool,
    pub status_announcements: bool,
}

pub trait Accessible {
    fn aria_role(&self) -> &str;
    fn aria_label(&self) -> String;
    fn announce(&self, message: &str);
    fn play_sound(&self, sound: SoundCue);
}

pub enum SoundCue {
    Select,
    Error,
    Success,
    Warning,
    Navigate,
}
```

## Testing

Each widget includes:
- Unit tests for logic
- Snapshot tests for rendering (with per-widget test helpers)
- Property tests for edge cases

```rust
// Per-widget test helpers provided by tui-testing
pub fn test_table_snapshot(table: &DataTable<T>, name: &str);
pub fn test_tree_snapshot(tree: &TreeView<T>, name: &str);
pub fn test_form_snapshot(form: &Form, name: &str);
pub fn test_palette_snapshot(palette: &CommandPalette, name: &str);
```

## Dependencies

```toml
[dependencies]
ratatui = { workspace = true }
crossterm = { workspace = true }
nucleo-matcher = { workspace = true }
rusqlite = { workspace = true }  # For CommandPalette recent commands
serde = { workspace = true }
```
