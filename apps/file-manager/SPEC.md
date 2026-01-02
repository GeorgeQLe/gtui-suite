# file-manager

Dual-pane Midnight Commander style file manager with VFS support.

## Architecture Decisions

### Copy/Move Progress Display
- **Modal with cancel**: Full-screen modal during file operations
- Shows file-by-file progress, ETA, transfer speed
- Cancel button to abort mid-operation
- Blocks other operations for safety

### Archive Write Operations
- **Extract + modify**: Write operations extract to temp, modify, re-archive
- Full functionality inside archives (rename, delete, create)
- Warning about potential slowness for large archives
- Atomic operation with rollback on failure

## Features

### Dual Pane Layout

```
┌─────────────────────────────┬─────────────────────────────┐
│  /home/user/projects        │  /home/user/downloads       │
├─────────────────────────────┼─────────────────────────────┤
│  ..                         │  ..                         │
│  > documents/               │  file1.pdf                  │
│  > pictures/                │  file2.zip                  │
│    file.txt                 │  archive.tar.gz             │
│    notes.md                 │                             │
└─────────────────────────────┴─────────────────────────────┘
│ 5 files selected, 12.5 MB                  Space: 45.2 GB │
└─────────────────────────────────────────────────────────────┘
```

### File Operations

**Basic:**
- Copy, Move, Delete, Rename
- Create directory
- Create file
- Symlink creation

**Bulk Operations:**
- Select multiple files
- Mass rename (patterns)
- Batch operations

**Permissions:**
- View permissions (rwxrwxrwx)
- Edit permissions
- Change owner/group

### Virtual Filesystem

**Archives as Folders:**
- ZIP, tar, tar.gz, tar.bz2
- Navigate into archives
- Extract files
- Create archives

**SFTP/SSH:**
- Browse remote servers
- Copy between local and remote
- Use ssh-agent

**S3:**
- Browse S3 buckets
- Upload/download files
- Requires credentials config

### Preview

**Text Files:**
- Syntax highlighted preview
- Scroll within preview

**Images:**
- ASCII art preview
- Dimensions info

**Directories:**
- Item count
- Total size

### Search

- Find files by name
- Recursive search
- Regex support
- Find in current directory

### Bookmarks

- Save favorite directories
- Quick navigation
- Persistent

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `h/l` | Parent / Enter directory |
| `Tab` | Switch pane |
| `enter` | Open / Enter |
| `space` | Toggle selection |
| `*` | Invert selection |
| `c` | Copy selected to other pane |
| `m` | Move selected to other pane |
| `d` | Delete selected |
| `r` | Rename |
| `n` | New file |
| `N` | New directory |
| `p` | Preview toggle |
| `/` | Search |
| `b` | Bookmarks |
| `B` | Add bookmark |
| `.` | Toggle hidden files |
| `s` | Sort menu |
| `o` | Open with... |
| `q` | Quit |

## Configuration

```toml
# ~/.config/file-manager/config.toml
[display]
show_hidden = false
show_icons = true
confirm_delete = true
preview_pane = true

[sort]
default = "name"  # name, size, date, type
directories_first = true
case_sensitive = false

[vfs]
archives = ["zip", "tar", "tar.gz", "tar.bz2", "tar.xz", "7z"]

[sftp]
use_ssh_agent = true

[s3]
# Credentials via environment or config
access_key_id = ""
secret_access_key = ""
region = "us-east-1"

[bookmarks]
home = "~"
downloads = "~/Downloads"
projects = "~/projects"
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
chrono = { workspace = true }
walkdir = "2"
zip = "2"
tar = "0.4"
flate2 = "1"
ssh2 = "0.9"
aws-sdk-s3 = "1"
syntect = "5"
mime_guess = "2"
copypasta = "0.10"
```
