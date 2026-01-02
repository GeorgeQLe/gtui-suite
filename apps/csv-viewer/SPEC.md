# csv-viewer

High-performance CSV/Parquet viewer using Polars.

## Architecture Decisions

### Malformed CSV Handling
- **Pad/truncate + prominent warning**: Use first row column count as schema
- Pad missing columns with null, truncate extra columns
- Show prominent warning with affected row numbers
- Highlight affected rows in table with visual indicator
- Allows viewing problematic files while making issues visible

## Features

### Large File Support

**Multi-GB Files:**
- Chunked loading with Polars
- Virtual scrolling (only render visible rows)
- Lazy evaluation

**Formats:**
- CSV (auto-detect delimiter)
- TSV
- Parquet
- JSON Lines

### Column Operations

**Sort:**
- Click column header to sort
- Multi-column sort
- Ascending/descending

**Filter:**
- Expression-based filtering
- Per-column filters
- Combine with AND/OR

**Hide/Show:**
- Toggle column visibility
- Reorder columns
- Pin columns left/right

### Data Analysis

**Summary Statistics:**
Per column:
- Count, null count
- Min, max, mean, median
- Std deviation
- Unique values

**Value Distribution:**
- Histogram for numeric
- Frequency for categorical

### Query Interface

SQL-like queries:

```sql
SELECT name, age FROM data
WHERE age > 30
ORDER BY name
LIMIT 100
```

### Export

- Export filtered/sorted data
- CSV, JSON, Parquet
- Selected columns only

## Data Engine

```rust
use polars::prelude::*;

pub struct DataViewer {
    lazy_frame: LazyFrame,
    schema: Schema,
    visible_range: Range<usize>,
    filters: Vec<Expr>,
    sort_by: Option<(String, bool)>,
}

impl DataViewer {
    pub fn load_csv(path: &Path) -> Result<Self>;
    pub fn load_parquet(path: &Path) -> Result<Self>;
    pub fn get_rows(&self, range: Range<usize>) -> Result<DataFrame>;
    pub fn filter(&mut self, expr: Expr);
    pub fn sort(&mut self, column: &str, ascending: bool);
    pub fn column_stats(&self, column: &str) -> ColumnStats;
}
```

## Views

**Table View:**
- Scrollable data table
- Resizable columns
- Frozen header

**Summary View:**
- All column statistics
- Data types
- Null counts

**Chart View (simple):**
- Bar charts for categorical
- Histograms for numeric

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Scroll rows |
| `h/l` | Scroll columns |
| `J/K` | Page down/up |
| `g/G` | First/last row |
| `0/$` | First/last column |
| `s` | Sort by current column |
| `f` | Filter dialog |
| `c` | Column visibility |
| `Tab` | Summary view |
| `/` | Search |
| `q` | SQL query |
| `x` | Export |
| `Ctrl+c` | Copy selection |
| `r` | Refresh file |
| `Esc` | Clear filters |
| `Q` | Quit |

## Configuration

```toml
# ~/.config/csv-viewer/config.toml
[parsing]
auto_detect_delimiter = true
default_delimiter = ","
has_header = true
infer_schema_rows = 1000

[display]
max_column_width = 50
null_display = "<null>"
float_precision = 2
thousands_separator = true

[performance]
chunk_size = 10000
cache_chunks = 10
```

## Filter Expressions

Support Polars expressions:

```
# Numeric comparisons
col("age") > 30
col("price").between(10, 100)

# String operations
col("name").str.contains("John")
col("email").str.ends_with("@gmail.com")

# Null handling
col("value").is_null()
col("value").is_not_null()

# Logical
(col("a") > 10) & (col("b") < 20)
(col("status") == "active") | (col("status") == "pending")
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
polars = { workspace = true }
csv = "1"
```
