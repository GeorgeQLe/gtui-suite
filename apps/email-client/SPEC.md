# email-client

Full-featured email client with offline sync.

## Architecture Decisions

### Email Threading Algorithm
- **References header + subject fallback**: Use References/In-Reply-To headers for threading
- Fall back to subject matching for broken threads from misbehaving clients
- Subject fallback restricted to same day and same participants to reduce false positives
- Handles most real-world threading scenarios correctly

### Offline Draft Handling
- **Local drafts with IMAP sync**: Save drafts locally, sync to Drafts folder when online
- Works fully offline, syncs across devices when connection restored
- Mark local-only drafts visually to indicate sync status
- Conflict detection when same draft edited on multiple devices

### Attachment Size Limits
- **SMTP SIZE extension**: Query server for maximum message size on connect
- Warn user before composing if attachment would exceed limit
- Prevents failed sends after lengthy composition
- Falls back to conservative 25MB default if server doesn't advertise limit

## Features

### Protocols

**IMAP:**
- Full IMAP4rev1 support
- IDLE for push notifications
- Folder management
- Search (server-side)

**SMTP:**
- Send emails
- Attachments
- HTML and plain text

### Storage

**SQLite + Disk Attachments:**
- Headers and body in SQLite
- Attachments as files
- Fast search via FTS5

### Offline Sync

**Download Strategy:**
- Headers always downloaded
- Bodies on-demand or pre-fetch
- Configurable sync depth

**Outbox:**
- Queue messages when offline
- Send when connected
- Retry failed sends

### Multiple Accounts

**Unified Inbox:**
- All accounts in one view
- Account indicators
- Per-account folders

**Account Config:**
```rust
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub imap: ImapConfig,
    pub smtp: SmtpConfig,
    pub signature: Option<String>,
}

pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub security: Security,
    pub user: String,
    pub password_source: PasswordSource,
}
```

### Features

**Threading:**
- Conversation view
- Group by subject/references
- Expand/collapse threads

**Search:**
- Full-text search
- Filter by folder, date, sender
- Saved searches

**Labels/Folders:**
- Standard folders (Inbox, Sent, Drafts, Trash)
- Custom folders
- Virtual folders (saved searches)

**Filters/Rules:**
```rust
pub struct Rule {
    pub name: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

pub enum Condition {
    From(String),
    To(String),
    Subject(String),
    Header { name: String, value: String },
}

pub enum Action {
    MoveTo(String),
    MarkRead,
    MarkStarred,
    Delete,
    Forward(String),
}
```

**Signatures:**
- Per-account signatures
- Plain text or markdown

**Address Book:**
- Store contacts
- Auto-complete in compose
- Import/export vCard

**Attachments:**
- Preview in-app
- Download to disk
- Send attachments

### Compose

- Plain text and HTML
- Markdown composition
- Reply/Reply-All/Forward
- Draft saving
- Attachment handling

### GPG (Optional)

- Sign emails
- Encrypt emails
- Verify signatures
- Decrypt incoming

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate messages |
| `J/K` | Navigate folders |
| `enter` | Open message |
| `c` | Compose new |
| `r` | Reply |
| `R` | Reply all |
| `f` | Forward |
| `d` | Delete |
| `a` | Archive |
| `s` | Star/unstar |
| `m` | Move to folder |
| `u` | Mark unread |
| `/` | Search |
| `g` | Go to folder |
| `Tab` | Next attachment |
| `Ctrl+s` | Send draft |
| `q` | Quit |

## Configuration

```toml
# ~/.config/email-client/config.toml
[[accounts]]
name = "Personal"
email = "user@example.com"

[accounts.imap]
host = "imap.example.com"
port = 993
security = "tls"
user = "user@example.com"
password_source = "keyring"

[accounts.smtp]
host = "smtp.example.com"
port = 587
security = "starttls"

[[accounts]]
name = "Work"
# ...

[sync]
days_to_sync = 30
download_bodies = "on_open"  # on_open, prefetch, never

[display]
thread_view = true
preview_lines = 2
date_format = "%b %d"

[compose]
default_format = "markdown"
```

## Database Schema

```sql
CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    config TEXT NOT NULL
);

CREATE TABLE folders (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    special_use TEXT,
    unread_count INTEGER DEFAULT 0
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    folder_id TEXT NOT NULL REFERENCES folders(id),
    message_id TEXT,  -- RFC Message-ID
    subject TEXT,
    sender TEXT,
    recipients TEXT,
    date TEXT,
    flags TEXT,
    body_text TEXT,
    body_html TEXT,
    has_attachments INTEGER DEFAULT 0,
    thread_id TEXT
);

CREATE VIRTUAL TABLE messages_fts USING fts5(subject, body_text, sender);

CREATE TABLE attachments (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES messages(id),
    filename TEXT NOT NULL,
    content_type TEXT,
    size INTEGER,
    path TEXT  -- Path to file on disk
);
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
rusqlite = { workspace = true, features = ["bundled-sqlcipher"] }
serde = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }
async-imap = "0.10"
async-smtp = "0.9"
mail-parser = "0.9"
mail-builder = "0.3"
keyring = "3"
gpgme = { version = "0.11", optional = true }
pulldown-cmark = "0.11"
```
