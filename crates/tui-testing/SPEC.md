# tui-testing

Testing utilities for TUI applications.

## Purpose

Provide comprehensive testing tools for TUI apps: snapshot testing, input simulation, and property-based testing.

## Architecture Decisions

### Snapshot Storage Format
- **Structured buffer**: Serialize Ratatui Buffer (cells with style info)
- Custom binary format optimized for TUI cell data
- Includes position, character, foreground, background, modifiers per cell

### Snapshot Implementation
- **Custom implementation**: Build TUI-specific snapshot system (not insta)
- Optimized for Buffer comparisons and structured diffs
- Better control over TUI-specific formatting and diff output

### Async Runtime Support
- **Multi-runtime**: Separate harness variants for each runtime
- `TokioTestHarness`, `AsyncStdTestHarness`, `SmolTestHarness`
- Common trait for runtime-agnostic test code

### Frame Assertions
- **Checkpoint assertions**: Explicit `app.assert_frame()` during test
- Manual control over when to capture and assert
- Supports animation and multi-step interaction testing

### Input Timing
- **Configurable**: Default ignores delays for fast tests
- Opt-in to real timing with `.with_real_timing()` for timing-sensitive code

### Diff Reporting
- **Structured diff**: Show which cells changed with positions and style differences
- Cell-by-cell comparison with clear position indicators
- Style changes shown explicitly (fg, bg, modifiers)

### Fixtures
- **Both types**: Deterministic fixtures AND random generators
- Deterministic for reproducible tests, random for fuzzing

### Property Generators
- **Both**: Fixed defaults with optional builder for customization
- Sensible out-of-box, configurable when needed

## Features

### Snapshot Testing

Capture terminal output and compare against golden files:

```rust
pub struct SnapshotTest {
    name: String,
    terminal: TestTerminal,
    snapshots_dir: PathBuf,
}

impl SnapshotTest {
    pub fn new(name: &str, width: u16, height: u16) -> Self;

    /// Render a widget and compare to golden file
    pub fn assert_snapshot<W: Widget>(&mut self, widget: W);

    /// Update golden files (run with UPDATE_SNAPSHOTS=1)
    pub fn update_snapshot<W: Widget>(&mut self, widget: W);

    /// Assert current buffer matches snapshot (checkpoint)
    pub fn assert_frame(&self, checkpoint_name: &str);

    /// Capture frame without asserting (for later comparison)
    pub fn capture_frame(&self, checkpoint_name: &str) -> CapturedFrame;
}

/// Structured snapshot format
#[derive(Serialize, Deserialize)]
pub struct BufferSnapshot {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<CellSnapshot>,
    pub version: u32,  // Format version for compatibility
}

#[derive(Serialize, Deserialize)]
pub struct CellSnapshot {
    pub x: u16,
    pub y: u16,
    pub symbol: String,
    pub fg: Color,
    pub bg: Color,
    pub modifiers: Modifier,
}

// Usage
#[test]
fn test_table_rendering() {
    let mut test = SnapshotTest::new("table_basic", 80, 24);
    let table = DataTable::new(columns, data);
    test.assert_snapshot(table);
}

#[test]
fn test_multi_step_interaction() {
    let mut test = SnapshotTest::new("table_edit", 80, 24);
    let mut app = App::new();

    app.render(&mut test.terminal);
    test.assert_frame("initial");

    app.handle_key(KeyCode::Enter);
    app.render(&mut test.terminal);
    test.assert_frame("after_enter");

    app.handle_key(KeyCode::Char('x'));
    app.render(&mut test.terminal);
    test.assert_frame("after_edit");
}
```

### Structured Diff Output

When snapshots don't match:

```rust
pub struct SnapshotDiff {
    pub changed_cells: Vec<CellDiff>,
    pub added_cells: Vec<CellSnapshot>,
    pub removed_cells: Vec<CellSnapshot>,
}

pub struct CellDiff {
    pub position: (u16, u16),
    pub expected: CellSnapshot,
    pub actual: CellSnapshot,
    pub changes: CellChanges,
}

pub struct CellChanges {
    pub symbol_changed: bool,
    pub fg_changed: bool,
    pub bg_changed: bool,
    pub modifiers_changed: bool,
}

// Output format:
// Snapshot mismatch in "table_basic":
//   Cell (5, 3): symbol '█' → '▓', fg #ff0000 → #00ff00
//   Cell (6, 3): modifiers [Bold] → [Bold, Italic]
//   Cell (10, 5): added 'X' (fg: white, bg: red)
//   2 cells removed in region (0,0)-(5,2)
```

### Golden File Management

```rust
pub struct GoldenFiles {
    base_path: PathBuf,
}

impl GoldenFiles {
    /// Get path for a snapshot
    pub fn snapshot_path(&self, name: &str) -> PathBuf;

    /// List all snapshots
    pub fn list_snapshots(&self) -> Vec<String>;

    /// Clean orphaned snapshots (no matching test)
    pub fn cleanup(&self) -> Result<CleanupReport>;

    /// Verify all snapshots have matching tests
    pub fn verify_coverage(&self) -> Result<CoverageReport>;
}

pub struct CleanupReport {
    pub removed: Vec<PathBuf>,
    pub kept: Vec<PathBuf>,
}
```

### Input Simulation

Simulate keyboard and mouse input with configurable timing:

```rust
pub struct InputSequence {
    events: Vec<Event>,
    delays: Vec<Duration>,
    real_timing: bool,
}

impl InputSequence {
    pub fn new() -> Self;

    /// Enable real timing (respect delays)
    pub fn with_real_timing(mut self) -> Self {
        self.real_timing = true;
        self
    }

    pub fn key(&mut self, key: KeyCode) -> &mut Self;
    pub fn key_mod(&mut self, key: KeyCode, modifiers: KeyModifiers) -> &mut Self;
    pub fn char(&mut self, c: char) -> &mut Self;
    pub fn text(&mut self, s: &str) -> &mut Self;
    pub fn delay(&mut self, ms: u64) -> &mut Self;

    pub fn ctrl(&mut self, c: char) -> &mut Self {
        self.key_mod(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    pub fn enter(&mut self) -> &mut Self { self.key(KeyCode::Enter) }
    pub fn esc(&mut self) -> &mut Self { self.key(KeyCode::Esc) }
    pub fn tab(&mut self) -> &mut Self { self.key(KeyCode::Tab) }
    pub fn up(&mut self) -> &mut Self { self.key(KeyCode::Up) }
    pub fn down(&mut self) -> &mut Self { self.key(KeyCode::Down) }

    // Mouse events
    pub fn click(&mut self, x: u16, y: u16) -> &mut Self;
    pub fn drag(&mut self, from: (u16, u16), to: (u16, u16)) -> &mut Self;
    pub fn scroll(&mut self, x: u16, y: u16, delta: i16) -> &mut Self;
}

// Usage
#[test]
fn test_navigation() {
    let input = InputSequence::new()
        .down().down().down()
        .enter()
        .text("new value")
        .enter();

    app.process_input(&input);
    assert_eq!(app.selected_row(), 3);
}

#[test]
fn test_with_timing() {
    let input = InputSequence::new()
        .with_real_timing()
        .text("search")
        .delay(300)  // Wait for debounce
        .enter();

    app.process_input(&input);
}
```

### Test Terminal

Virtual terminal for headless testing:

```rust
pub struct TestTerminal {
    backend: TestBackend,
    size: Rect,
    frame_history: Vec<BufferSnapshot>,  // For checkpoint assertions
}

impl TestTerminal {
    pub fn new(width: u16, height: u16) -> Self;

    /// Render a frame
    pub fn draw<F>(&mut self, f: F) where F: FnOnce(&mut Frame);

    /// Get current buffer content
    pub fn buffer(&self) -> &Buffer;

    /// Get content as string (for quick debugging)
    pub fn to_string(&self) -> String;

    /// Assert buffer matches expected string
    pub fn assert_buffer(&self, expected: &str);

    /// Get buffer as structured snapshot
    pub fn snapshot(&self) -> BufferSnapshot;

    /// Resize terminal
    pub fn resize(&mut self, width: u16, height: u16);
}
```

### Async Test Harnesses

Multi-runtime support:

```rust
/// Common trait for all async harnesses
pub trait AsyncHarness {
    async fn run_until<F>(&mut self, condition: F, timeout: Duration) -> Result<()>
    where
        F: Fn(&Self) -> bool;

    async fn wait_for_render(&mut self);
    async fn send_input(&mut self, input: InputSequence);
    fn terminal(&self) -> &TestTerminal;
}

/// Tokio-specific harness
pub struct TokioTestHarness {
    runtime: tokio::runtime::Runtime,
    terminal: TestTerminal,
}

impl TokioTestHarness {
    pub fn new(width: u16, height: u16) -> Self;

    pub async fn run_until<F>(&mut self, condition: F, timeout: Duration) -> Result<()>
    where
        F: Fn(&TestTerminal) -> bool;

    pub async fn wait_for_render(&mut self);
    pub async fn send_input(&mut self, input: InputSequence);
}

/// async-std specific harness
pub struct AsyncStdTestHarness {
    terminal: TestTerminal,
}

/// smol specific harness
pub struct SmolTestHarness {
    executor: smol::Executor<'static>,
    terminal: TestTerminal,
}

// Usage
#[tokio::test]
async fn test_async_loading() {
    let mut harness = TokioTestHarness::new(80, 24);
    let mut app = AsyncApp::new();

    harness.send_input(InputSequence::new().text("search")).await;

    // Wait for loading to complete
    harness.run_until(
        |term| !term.to_string().contains("Loading..."),
        Duration::from_secs(5)
    ).await.unwrap();

    harness.terminal().assert_buffer(/* expected */);
}
```

### Property-Based Testing

Fixed defaults with configurable builders:

```rust
pub mod generators {
    /// Generate random key events with sensible defaults
    pub fn key_event() -> impl Strategy<Value = KeyEvent>;

    /// Configurable key event generator
    pub struct KeyEventGen {
        include_modifiers: bool,
        include_function_keys: bool,
        include_special_keys: bool,
        allowed_chars: Option<Vec<char>>,
    }

    impl KeyEventGen {
        pub fn new() -> Self;
        pub fn with_modifiers(mut self) -> Self;
        pub fn without_function_keys(mut self) -> Self;
        pub fn only_printable(mut self) -> Self;
        pub fn only_chars(mut self, chars: &[char]) -> Self;
        pub fn build(self) -> impl Strategy<Value = KeyEvent>;
    }

    /// Generate random navigation sequences
    pub fn navigation_sequence(len: usize) -> impl Strategy<Value = Vec<KeyEvent>>;

    /// Configurable navigation generator
    pub struct NavigationGen {
        length: usize,
        include_page_keys: bool,
        include_home_end: bool,
    }

    /// Generate random table data
    pub fn table_data<T: Arbitrary>(rows: usize, cols: usize)
        -> impl Strategy<Value = Vec<Vec<T>>>;

    /// Configurable table data generator
    pub struct TableDataGen<T> {
        rows: std::ops::Range<usize>,
        cols: std::ops::Range<usize>,
        cell_gen: Box<dyn Strategy<Value = T>>,
    }
}

// Usage with defaults
proptest! {
    #[test]
    fn table_navigation_never_panics(events in generators::navigation_sequence(100)) {
        let mut table = DataTable::new(columns, data);
        for event in events {
            table.handle_key(event);
        }
    }
}

// Usage with custom configuration
proptest! {
    #[test]
    fn text_input_handles_all_printable(
        events in KeyEventGen::new()
            .only_printable()
            .without_function_keys()
            .build()
            .prop_flat_map(|e| prop::collection::vec(Just(e), 0..50))
    ) {
        let mut input = TextInput::new();
        for event in events {
            input.handle_key(event);
        }
    }
}
```

### Fixtures

Both deterministic fixtures AND random generators:

```rust
pub mod fixtures {
    //
    // Deterministic fixtures - same data every run
    //

    /// 10-row sample table with name, age, city columns
    pub fn sample_table_data() -> Vec<Vec<String>>;

    /// 3-level sample tree with ~20 nodes
    pub fn sample_tree() -> TreeNode;

    /// Contact form with 5 fields
    pub fn sample_form() -> Form;

    /// 100 sample log lines with timestamps
    pub fn sample_logs() -> Vec<LogEntry>;

    //
    // Sized fixtures
    //

    /// Table with specified dimensions
    pub fn table_data(rows: usize, cols: usize) -> Vec<Vec<String>>;

    /// Tree with specified depth and branching factor
    pub fn tree(depth: usize, branching: usize) -> TreeNode;

    //
    // Random generators (proptest strategies)
    //

    /// Random table data generator
    pub fn random_table(
        rows: impl Into<std::ops::Range<usize>>,
        cols: impl Into<std::ops::Range<usize>>,
    ) -> impl Strategy<Value = Vec<Vec<String>>>;

    /// Random tree generator
    pub fn random_tree(
        max_depth: usize,
        max_children: usize,
    ) -> impl Strategy<Value = TreeNode>;
}

// Usage
#[test]
fn test_with_deterministic_data() {
    let data = fixtures::sample_table_data();  // Always the same
    let table = DataTable::new(columns, data);
    // ...
}

proptest! {
    #[test]
    fn test_with_random_data(
        data in fixtures::random_table(10..100, 3..10)
    ) {
        let table = DataTable::new(columns, data);
        // ...
    }
}
```

### Per-Widget Test Helpers

Convenience helpers for each widget type:

```rust
pub mod widget_tests {
    pub fn test_table_snapshot(table: &DataTable<impl Display>, name: &str);
    pub fn test_tree_snapshot(tree: &TreeView<impl TreeNode>, name: &str);
    pub fn test_form_snapshot(form: &Form, name: &str);
    pub fn test_palette_snapshot(palette: &CommandPalette, name: &str);

    /// Test that widget handles all navigation keys without panic
    pub fn assert_navigation_safe<W: Widget + HandleKey>(widget: &mut W);

    /// Test widget renders within bounds
    pub fn assert_renders_in_bounds<W: Widget>(widget: &W, area: Rect);
}
```

## Integration with CI

```rust
/// Environment variable to update snapshots
pub const UPDATE_SNAPSHOTS: &str = "UPDATE_SNAPSHOTS";

/// Check if running in CI
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
}

/// Check if snapshots should be updated
pub fn should_update_snapshots() -> bool {
    std::env::var(UPDATE_SNAPSHOTS).is_ok()
}

/// Configure test output for CI
pub struct CiConfig {
    pub colored_output: bool,
    pub verbose_diffs: bool,
    pub fail_fast: bool,
}

impl CiConfig {
    pub fn from_env() -> Self;
}
```

## Dependencies

```toml
[dependencies]
ratatui = { workspace = true }
crossterm = { workspace = true }
proptest = { workspace = true }
serde = { workspace = true }
bincode = "1"  # For structured snapshot serialization

[dev-dependencies]
tokio = { workspace = true, features = ["test-util", "rt-multi-thread"] }
async-std = { version = "1", features = ["attributes"] }
smol = "2"
```
