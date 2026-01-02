# api-tester

HTTP API testing with collections, environments, and test suites.

## Architecture Decisions

### Test Variable Scoping
- **Per-run scoped**: Variables reset at start of each test run
- Ensures test reproducibility and independence
- Optional 'persist' flag for specific variables that should survive
- Prevents state pollution between test runs

### Curl Import Scope
- **Common subset**: Support ~20 most-used curl flags
- Covers -X, -H, -d, -u, -b, -c, --data-raw, etc.
- Warn on unsupported flags, show which were ignored
- Handles 95%+ of real-world curl commands

### JavaScript Assertion Sandboxing
- **Timeout + memory limit**: Assertions run with 5 second timeout and 10MB memory limit
- Uses boa_engine with built-in resource limits
- Prevents infinite loops and memory bombs in user scripts
- Sandboxed: no file/network access from assertions

## Features

### Request Building

**Methods:**
- GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS

**Headers:**
- Custom headers
- Common presets (Content-Type, Authorization)
- Header templates

**Body:**
- Raw (JSON, XML, text)
- Form data
- File upload
- Syntax highlighting

**Query Parameters:**
- Key-value editor
- URL encoding

**Authentication:**
- Basic Auth
- Bearer Token
- OAuth 2.0 (authorization code, client credentials)
- API Key (header or query)

### Response Viewing

- Syntax-highlighted body
- Headers table
- Timing breakdown (DNS, connect, TLS, first byte, total)
- Response size
- Status code

### Collections

**Postman Compatible:**
- Import from Postman JSON
- Export to Postman format
- Folder organization

**Local Storage:**
```rust
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub folders: Vec<Folder>,
    pub requests: Vec<SavedRequest>,
}

pub struct SavedRequest {
    pub id: Uuid,
    pub name: String,
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<RequestBody>,
    pub auth: Option<AuthConfig>,
}
```

### Environments

**Variables:**
```toml
[dev]
base_url = "http://localhost:3000"
api_key = "dev-key"

[staging]
base_url = "https://staging.api.example.com"
api_key = "staging-key"
```

**Usage:**
```
GET {{base_url}}/users/{{user_id}}
Authorization: Bearer {{api_key}}
```

### Test Suites

**JavaScript-like Assertions:**
```javascript
// Inline in request
response.status === 200
body.users.length > 0
body.users[0].name === "John"
response.time < 1000
headers["Content-Type"].includes("json")
```

**Test Runner:**
- Multiple requests in sequence
- Chain variables between requests
- Batch execution
- Report generation (pass/fail)

### Request History

- Recent requests
- Response caching
- Diff responses

### Import/Export

- curl import
- curl export
- OpenAPI import (generate from spec)

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Navigate sections |
| `enter` | Send request |
| `Ctrl+s` | Save request |
| `e` | Edit environment |
| `c` | Collections view |
| `h` | History |
| `t` | Test suite runner |
| `v` | View response |
| `y` | Copy as curl |
| `i` | Import |
| `/` | Search collections |
| `n` | New request |
| `Ctrl+n` | New collection |
| `q` | Quit |

## Configuration

```toml
# ~/.config/api-tester/config.toml
[http]
timeout_secs = 30
follow_redirects = true
max_redirects = 10
verify_ssl = true

[display]
syntax_theme = "monokai"
show_timing = true

[export]
default_format = "curl"

[test]
stop_on_failure = false
parallel = false
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
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
reqwest = { workspace = true }
tokio = { workspace = true }
syntect = "5"
boa_engine = "0.19"  # JavaScript for assertions
jsonpath-rust = "0.6"
```
