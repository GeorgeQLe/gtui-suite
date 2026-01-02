# hex-editor

Hex editor with structure parsing and binary diff.

## Architecture Decisions

### Large File Editing Strategy
- **Copy-on-write pages**: Track modifications in 4KB pages
- Original file stays intact until save
- Memory-efficient for sparse edits (typical case)
- Evict pages to temp file under memory pressure
- Editing scattered bytes in multi-GB files uses minimal memory

### Structure Template Conditionals
- **Simple conditionals**: Support if/else based on previously parsed fields
- Enables handling format variations (e.g., if magic == 0x5A4D, parse PE)
- Keep DSL syntax simple with clear examples
- Sufficient for most binary format variations

### Binary Diff Strategy
- **Chunked comparison**: Compare files in chunks, stream differences
- Works on files larger than available RAM
- Show progress indicator during comparison
- Memory usage stays constant regardless of file size

## Features

### Core Editing

**Display:**
- Hex view + ASCII view
- Configurable bytes per row
- Address column
- Cursor position

**Operations:**
- Overwrite bytes
- Insert bytes
- Delete bytes
- Undo/redo

**Large Files:**
- Memory-mapped I/O
- Efficient for multi-GB files
- Only load visible region

### Search

**Hex Search:**
- Search hex patterns: `4D 5A 90 00`
- Wildcards: `4D ?? 90`

**ASCII Search:**
- String search
- Case sensitive/insensitive

**Find and Replace:**
- Replace all or one-by-one
- Preview replacements

### Structure Templates

Parse known binary formats:

```rust
pub struct StructureTemplate {
    pub name: String,
    pub fields: Vec<FieldDef>,
}

pub struct FieldDef {
    pub name: String,
    pub offset: usize,
    pub field_type: FieldType,
    pub description: Option<String>,
}

pub enum FieldType {
    Uint8,
    Uint16Le,
    Uint16Be,
    Uint32Le,
    Uint32Be,
    Int8,
    Int16Le,
    Int32Le,
    Bytes(usize),
    String(usize),
    Nested(Box<StructureTemplate>),
}
```

**Built-in Templates:**
- ELF (Linux executables)
- PE (Windows executables)
- Mach-O (macOS executables)
- PNG, JPEG, GIF
- ZIP, tar, gzip

**User-Defined:**
- Template DSL in TOML
- Hot-reload templates

### Structure View

When template matches:
```
ELF Header
├── Magic: 7F 45 4C 46 (ELF)
├── Class: 64-bit
├── Data: Little Endian
├── Version: 1
├── OS/ABI: Linux
├── Type: Executable
├── Machine: x86-64
├── Entry: 0x00401000
└── Program Headers: offset 0x40
```

### Binary Diff

Compare two binary files:
- Highlight differences
- Navigate between changes
- Generate patches

### Data Inspector

At cursor position show value as:
- Uint8, Int8
- Uint16/32/64 LE/BE
- Float32/64
- ASCII string
- Unicode string

### Bookmarks

- Mark positions
- Named bookmarks
- Navigate between

## Views

**Hex View:**
```
00000000: 7F 45 4C 46 02 01 01 00  00 00 00 00 00 00 00 00  |.ELF............|
00000010: 02 00 3E 00 01 00 00 00  00 10 40 00 00 00 00 00  |..>.......@.....|
00000020: 40 00 00 00 00 00 00 00  68 1A 00 00 00 00 00 00  |@.......h.......|
```

**Structure View:**
- Tree of parsed fields
- Click to jump to offset

**Inspector View:**
- Value interpretations
- Updates with cursor

## Keybindings

| Key | Action |
|-----|--------|
| `h/j/k/l` | Navigate |
| `g` | Go to address |
| `G` | Go to end |
| `i` | Insert mode |
| `r` | Replace byte |
| `x` | Delete byte |
| `u` | Undo |
| `Ctrl+r` | Redo |
| `/` | Search hex |
| `?` | Search ASCII |
| `n/N` | Next/prev match |
| `t` | Toggle structure view |
| `d` | Data inspector |
| `b` | Add bookmark |
| `'` | Go to bookmark |
| `D` | Binary diff |
| `Ctrl+s` | Save |
| `q` | Quit |

## Configuration

```toml
# ~/.config/hex-editor/config.toml
[display]
bytes_per_row = 16
group_size = 8
show_ascii = true
uppercase = true

[templates]
path = "~/.config/hex-editor/templates"
auto_detect = true

[editing]
backup_on_save = true
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
memmap2 = "0.9"
byteorder = "1"
regex = "1"
object = "0.36"  # For ELF/PE parsing
```
