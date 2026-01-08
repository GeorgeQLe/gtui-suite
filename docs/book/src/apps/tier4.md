# Tier 4 - Advanced Applications

Tier 4 applications are advanced tools with complex data handling, async operations, and rich feature sets.

## Implemented Apps

### hex-editor

A binary file hex editor with undo/redo support.

**Features:**
- View files in hex and ASCII
- Edit bytes in place
- Full undo/redo with operation stack
- Search by hex pattern or ASCII
- Go to offset
- Insert and delete bytes

**Run:** `cargo run -p hex-editor -- <file>`

### csv-viewer

A CSV/TSV data viewer with sorting and filtering.

**Features:**
- Auto-detect CSV vs TSV format
- Navigate cells with cursor
- Sort by any column (numeric-aware)
- Filter column by text
- Search across all columns
- Highlight search results

**Run:** `cargo run -p csv-viewer -- <file.csv>`

### kanban-standalone

A local Kanban board for task management.

**Features:**
- Multiple boards
- Drag cards between columns
- Due dates and labels
- Card descriptions
- SQLite persistence
- Swimlanes (optional)

**Run:** `cargo run -p kanban-standalone`

### git-client

A full-featured Git client.

**Features:**
- Repository overview (branch, status)
- File staging (add/remove)
- Commit with message editor
- Branch management (create, switch, delete)
- Diff viewer
- Log viewer with commit details
- Stash management

**Run:** `cargo run -p git-client`

### api-tester

HTTP API testing tool (like Postman in terminal).

**Features:**
- All HTTP methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
- Request headers editor
- Request body editor
- Authentication: Basic, Bearer token, API key
- Response viewer with syntax highlighting
- Collections for organizing requests
- Request history
- Generate cURL commands

**Run:** `cargo run -p api-tester`

## Planned Apps (Not Yet Implemented)

These apps are specified but not yet implemented:

- **ci-dashboard** - CI/CD pipeline monitoring
- **k8s-dashboard** - Kubernetes cluster management
- **db-client** - Database client (PostgreSQL, MySQL, SQLite)
- **metrics-viewer** - Prometheus metrics visualization

## Common Patterns

Tier 4 apps share these patterns:

### Async Operations

```rust
pub struct App {
    pub rt: tokio::runtime::Runtime,
    // ...
}

impl App {
    pub fn send_request(&mut self) {
        match self.rt.block_on(http_client::send_request(&request)) {
            Ok(response) => { /* handle success */ }
            Err(e) => { /* handle error */ }
        }
    }
}
```

### Rich Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRequest {
    pub id: Uuid,
    pub name: String,
    pub method: Method,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Option<String>,
    pub auth: AuthConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Undo/Redo Support

```rust
pub struct EditOperation {
    pub offset: usize,
    pub old_value: u8,
    pub new_value: u8,
}

pub struct Buffer {
    pub data: Vec<u8>,
    pub undo_stack: Vec<EditOperation>,
    pub redo_stack: Vec<EditOperation>,
}
```

## Running Tier 4 Apps

```bash
# Build all Tier 4 apps
cargo build -p hex-editor -p csv-viewer -p kanban-standalone -p git-client -p api-tester

# Run tests
cargo test -p hex-editor -p csv-viewer -p kanban-standalone -p git-client -p api-tester
```
