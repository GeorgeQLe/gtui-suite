# db-client

Multi-database SQL client with procedure IDE.

## Architecture Decisions

### Query Execution Blocking
- **Non-blocking + indicator**: Query runs in background
- UI shows spinner, user can browse schema or edit other queries
- Show running query count, warn on concurrent queries
- Prevents wasted time during long queries

### Schema-Qualified Autocomplete
- **Context-aware**: Omit schema if table is in search_path/default schema
- Include schema qualification for cross-schema references
- Show schema in completion popup for clarity
- Minimal typing for common case, correct for complex cases

### Query Cancellation
- **Cancel button + timeout**: Show cancel button during execution
- Configurable timeout (default 30 seconds)
- Long-running queries show elapsed time and cancel option
- Works with PostgreSQL pg_cancel_backend, MySQL KILL QUERY

## Features

### Database Support

**PostgreSQL:**
- Full SQL support
- pg-specific types (JSONB, arrays)
- EXPLAIN ANALYZE visualization
- Stored procedure debugging

**MySQL:**
- Full SQL support
- MySQL-specific syntax
- Procedure execution

**SQLite:**
- Local file databases
- In-memory databases

### Connection Management

```rust
pub struct Connection {
    pub id: Uuid,
    pub name: String,
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password_source: PasswordSource,
    pub ssl_mode: SslMode,
}

pub enum PasswordSource {
    Prompt,
    Keyring,
    EnvVar(String),
}
```

### Schema Browser

**Tree View:**
```
Database
├── public (schema)
│   ├── Tables
│   │   ├── users
│   │   │   ├── id (integer, PK)
│   │   │   ├── name (varchar)
│   │   │   └── email (varchar, unique)
│   │   └── orders
│   ├── Views
│   ├── Functions
│   └── Sequences
└── other_schema
```

**Foreign Keys:**
- Visual relationship indicators
- Navigate to related table

### Query Editor

**Features:**
- Syntax highlighting
- Auto-complete (tables, columns, keywords)
- Multi-statement execution
- Query history
- Saved queries

**Execution:**
- Run selected text
- Run all statements
- Transaction support
- Cancel long queries

### Results Viewer

**Data Table:**
- Virtual scrolling
- Column resize
- Sort (in-memory)
- Copy cells/rows
- Export (CSV, JSON)

**Multiple Result Sets:**
- Tab per result
- Navigate between

### Query Plan Visualization

**EXPLAIN ANALYZE:**
- Tree view of plan
- Cost per node
- Actual vs estimated rows
- Timing per node
- Identify bottlenecks

### Stored Procedure IDE

**Browse:**
- List procedures/functions
- View source code

**Edit:**
- Syntax highlighting
- Create/alter/drop

**Debug:**
- Print-style (RAISE NOTICE)
- Breakpoints (PostgreSQL with pldebugger)
- Step through
- Variable inspection

### Export

- Query results to CSV/JSON
- Schema DDL generation
- Data export

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch panels |
| `enter` | Expand/select |
| `Ctrl+enter` | Execute query |
| `Ctrl+s` | Save query |
| `Ctrl+l` | Clear editor |
| `F5` | Execute |
| `F9` | Toggle breakpoint |
| `F10` | Step over |
| `F11` | Step into |
| `e` | Edit (procedures) |
| `d` | Describe table |
| `h` | Query history |
| `/` | Search schema |
| `x` | Export results |
| `q` | Quit |

## Configuration

```toml
# ~/.config/db-client/config.toml
[editor]
tab_size = 4
auto_complete = true
syntax_theme = "monokai"

[execution]
auto_commit = false
limit_results = 1000
timeout_secs = 30

[display]
null_display = "NULL"
max_column_width = 100
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
rusqlite = { workspace = true }
serde = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }
sqlx = { version = "0.8", features = ["postgres", "mysql", "sqlite", "runtime-tokio"] }
sqlparser = "0.52"
syntect = "5"
keyring = "3"
```
