# diff-tool

File and directory diff with three-way merge support.

## Architecture Decisions

### Semantic Move Detection
- **Conservative**: Only detect exact matches or very high similarity (>90%)
- Fewer false positives in diff display
- Clearly mark detected moves with visual indicator
- Configurable threshold for power users

### Auto-Merge Review
- **Show for review**: Auto-merge runs but user must confirm before saving
- Auto-merged sections highlighted differently from conflicts
- Allows quick approval of clean merges
- Prevents unexpected changes from being applied silently

## Features

### File Diff

**Unified View:**
- Standard unified diff format
- Line numbers
- Context lines

**Side-by-Side:**
- Two panes
- Synchronized scrolling
- Highlight differences

**Word-Level:**
- Character-by-character diff within lines
- Highlight exact changes

**Semantic Awareness:**
- Detect moved blocks
- Function/class renaming detection
- Language-aware parsing

### Directory Diff

Compare two directories:
- New files
- Deleted files
- Modified files
- Navigate into files

### Three-Way Merge

For conflict resolution:
- Base, ours, theirs
- Auto-merge where possible
- Manual conflict resolution

**Hunk Picking:**
- Accept/reject individual hunks
- Take left/right/both
- Edit manually

### Git Integration

- diff staged changes
- diff HEAD
- diff between commits
- Integrate with git-client

### Options

- Ignore whitespace
- Ignore case
- Ignore blank lines
- Word diff threshold

## Views

**Unified View:**
```
--- a/file.txt
+++ b/file.txt
@@ -10,6 +10,7 @@
 context line
-removed line
+added line
 context line
```

**Side-by-Side:**
```
│ file.txt (old)              │ file.txt (new)              │
├─────────────────────────────┼─────────────────────────────┤
│ 10: context line            │ 10: context line            │
│ 11: removed line        <<<│ 11: added line          >>> │
│ 12: context line            │ 12: context line            │
```

**Merge View:**
```
│ Base          │ Ours          │ Theirs        │ Result        │
├───────────────┼───────────────┼───────────────┼───────────────┤
│ original      │ our change    │ their change  │ [resolved]    │
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate hunks |
| `J/K` | Navigate lines |
| `Tab` | Switch panes |
| `enter` | View/edit hunk |
| `a` | Accept hunk |
| `r` | Reject hunk |
| `l` | Take left (ours) |
| `R` | Take right (theirs) |
| `b` | Take both |
| `e` | Edit manually |
| `n/p` | Next/previous conflict |
| `w` | Toggle whitespace |
| `s` | Save merged result |
| `/` | Search |
| `q` | Quit |

## Configuration

```toml
# ~/.config/diff-tool/config.toml
[display]
default_view = "side-by-side"  # unified, side-by-side
context_lines = 3
syntax_highlight = true
word_diff = true

[ignore]
whitespace = false
case = false
blank_lines = false

[merge]
auto_merge = true
conflict_style = "diff3"
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
similar = "2"
syntect = "5"
tree-sitter = "0.24"  # For semantic diff
walkdir = "2"
```
