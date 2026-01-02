# config-editor

Edit YAML/TOML/JSON configs with validation and schema support.

## Architecture Decisions

### Schema-Constrained Value Editing
- **Hybrid approach**: UI adapts to constraint type
  - Small enums (≤10 values): Dropdown picker
  - Large enums/patterns: Inline autocomplete as you type
  - Numeric ranges: Free text with validation feedback
- Best UX for each data type

### Backup Storage Format
- **Compressed full copies**: Each backup is complete file with gzip/zstd
- Simple recovery (decompress and copy)
- Self-contained, no dependency chains
- Config files are small, space not a concern

### Unknown File Handling
- **Hybrid detection + prompt**: Parse and build tree view for any valid format
- Mark all fields as 'type unknown' in tree
- Prompt user to associate schema file for full validation
- Provides structure navigation without false validation errors

### Additional Properties Validation
- **Error on violation**: Treat extra keys not in schema as validation error
- Highlight unknown properties in editor
- Helps catch typos in config keys

## Features

### Format Support

- YAML
- TOML
- JSON
- Syntax highlighting for each

### Validation

**Syntax Validation:**
- Real-time parse errors
- Error location highlighting
- Fix suggestions

**Schema Validation:**
- JSON Schema support
- Inline error display
- Required field indicators
- Type checking

### Known Configs

Built-in schema support for common files:
- docker-compose.yml
- kubernetes manifests
- nginx.conf
- systemd units
- Cargo.toml
- package.json
- tsconfig.json
- .github/workflows/*.yml

### Features

**Type Coercion:**
- Suggest correct types
- "Did you mean true instead of 'true'?"
- Number vs string detection

**Diff View:**
- Compare current vs saved
- Compare with backup
- Side-by-side view

**Backup:**
- Auto-backup before save
- Configurable backup count
- Restore from backup

### Tree View

Structured navigation for nested configs:
```
docker-compose.yml
├── version: "3.8"
├── services
│   ├── web
│   │   ├── image: nginx
│   │   ├── ports: [...]
│   │   └── volumes: [...]
│   └── db
│       ├── image: postgres
│       └── environment: [...]
└── networks
    └── default
```

## Views

**Text View:**
- Full file editor
- Syntax highlighting
- Line numbers

**Tree View:**
- Structured navigation
- Expand/collapse sections
- Edit values inline

**Schema View:**
- Show expected structure
- Required vs optional
- Type information

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate lines/nodes |
| `h/l` | Collapse/expand (tree) |
| `enter` | Edit value (tree) |
| `Tab` | Toggle text/tree view |
| `Ctrl+s` | Save |
| `Ctrl+z` | Undo |
| `Ctrl+y` | Redo |
| `d` | Show diff |
| `v` | Validate |
| `/` | Search |
| `g` | Go to line |
| `b` | List backups |
| `q` | Quit |

## Configuration

```toml
# ~/.config/config-editor/config.toml
[editor]
tab_size = 2
insert_spaces = true
word_wrap = true
show_line_numbers = true

[validation]
validate_on_type = true
show_inline_errors = true

[backup]
enabled = true
max_backups = 5
backup_dir = "~/.config/config-editor/backups"

[schemas]
# Additional schema mappings
"my-config.yaml" = "~/.config/config-editor/schemas/my-config.json"

[known_configs]
enabled = true
auto_detect = true
```

## Built-in Schemas

Bundled JSON Schemas for common formats:

```rust
pub fn get_schema(filename: &str) -> Option<JsonSchema> {
    match filename {
        "docker-compose.yml" | "docker-compose.yaml" => Some(DOCKER_COMPOSE_SCHEMA),
        "Cargo.toml" => Some(CARGO_TOML_SCHEMA),
        "package.json" => Some(PACKAGE_JSON_SCHEMA),
        // ... etc
        _ => detect_from_content(content)
    }
}
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = "0.9"
toml = { workspace = true }
jsonschema = "0.22"
syntect = "5"
similar = "2"  # For diffs
tree-sitter = "0.24"
```
