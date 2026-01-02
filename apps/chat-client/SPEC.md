# chat-client

Multi-protocol chat client with Matrix E2EE support.

## Architecture Decisions

### Multi-Protocol Message History
- **Unified local SQLite**: Store all messages from all protocols in single local database
- Fast full-text search across all conversations, works fully offline
- Periodic dedup against server history for protocols that support it
- Configurable retention period to manage storage growth

### Matrix E2EE Key Backup
- **Automatic server backup**: Enable Matrix server-side key backup with passphrase protection
- Prompt for passphrase on first E2EE room join
- Show periodic reminders if backup not configured
- Allows key recovery on new devices without losing encrypted history

### Message ID Namespacing
- **Composite key**: Store messages with (protocol, original_id) composite primary key
- Prevents ID collisions across protocols (IRC, Matrix, Slack, Discord)
- Enables efficient queries by protocol
- Preserves original IDs for protocol-specific features

## Features

### Protocol Support

**IRC:**
- Multiple networks
- Channel and private messages
- Nick management
- Standard IRC commands

**Matrix:**
- Full Matrix protocol
- End-to-end encryption (Megolm)
- Room management
- Federation support

**Slack Bridge:**
- Connect via Slack API
- Workspace access
- Channels and DMs

**Discord Bridge:**
- Connect via Discord API
- Server/guild access
- Text channels

### Matrix E2EE

**Megolm Encryption:**
- Room key management
- Key backup and restore
- Cross-signing
- Device verification (emoji/QR)

**Key Management:**
```rust
pub struct CryptoStore {
    pub identity_keys: IdentityKeys,
    pub one_time_keys: Vec<OneTimeKey>,
    pub megolm_sessions: HashMap<RoomId, MegolmSession>,
    pub device_keys: HashMap<UserId, HashMap<DeviceId, DeviceKeys>>,
}
```

### UI Features

**Room/Channel List:**
- Organized by server/network
- Unread indicators
- Notification badges
- Favorites/pinned

**Message View:**
- Threaded conversations
- Reply chains
- Reactions
- Rich text (markdown)
- Code blocks with syntax highlighting

**Mentions:**
- @ mentions
- Highlight notifications
- Reply notifications

**File Attachments:**
- Upload files
- Image preview (ASCII art)
- Download to local

### Features

**Unified Inbox:**
- All unread across protocols
- Jump to conversation

**Search:**
- Search message history
- Filter by room/user/date

**Notifications:**
- Per-room settings
- Mute options
- Keywords

**Presence:**
- Online/away/busy status
- Typing indicators

## Data Model

```rust
pub struct Account {
    pub id: Uuid,
    pub protocol: Protocol,
    pub display_name: String,
    pub config: AccountConfig,
}

pub enum Protocol {
    Irc { network: String },
    Matrix { homeserver: String },
    Slack { workspace: String },
    Discord { guild_id: String },
}

pub struct Room {
    pub id: String,
    pub account_id: Uuid,
    pub name: String,
    pub room_type: RoomType,
    pub encrypted: bool,
    pub unread_count: u32,
    pub last_message: Option<DateTime<Utc>>,
}

pub struct Message {
    pub id: String,
    pub room_id: String,
    pub sender: String,
    pub content: MessageContent,
    pub timestamp: DateTime<Utc>,
    pub encrypted: bool,
    pub reply_to: Option<String>,
}

pub enum MessageContent {
    Text(String),
    Image { url: String, thumbnail: Option<String> },
    File { url: String, filename: String },
    Notice(String),
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate rooms |
| `J/K` | Navigate messages |
| `enter` | Open room / Send message |
| `i` | Start typing |
| `esc` | Cancel typing |
| `r` | Reply to message |
| `e` | Edit message |
| `d` | Delete message |
| `@` | Mention user |
| `Tab` | Complete nick/room |
| `/` | Command / Search |
| `v` | Verify device |
| `n` | Notification settings |
| `q` | Quit |

## Configuration

```toml
# ~/.config/chat-client/config.toml
[[accounts]]
name = "Matrix Home"
protocol = "matrix"
homeserver = "https://matrix.org"
user_id = "@user:matrix.org"
password_source = "keyring"

[[accounts]]
name = "Libera IRC"
protocol = "irc"
server = "irc.libera.chat"
port = 6697
ssl = true
nick = "username"

[notifications]
sound = true
desktop = true
keywords = ["username", "hey"]

[display]
show_timestamps = true
time_format = "%H:%M"
show_avatars = false
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
matrix-sdk = { version = "0.7", features = ["e2e-encryption", "sqlite"] }
irc = "0.15"
reqwest = { workspace = true }
keyring = "3"
notify-rust = "4"
pulldown-cmark = "0.11"
syntect = "5"
```
