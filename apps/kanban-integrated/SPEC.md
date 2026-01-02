# kanban-integrated

Kanban board with external service integrations.

## Features

All features from kanban-standalone plus external integrations.

### GitHub Issues

**Sync:**
- Pull issues from repositories
- Map to kanban columns by label
- Bidirectional updates
- Create issues from cards

**Mapping:**
```toml
[github.label_mapping]
"todo" = "Backlog"
"in-progress" = "In Progress"
"done" = "Done"
```

### GitLab Issues

Same as GitHub:
- Issue sync
- Label mapping
- Bidirectional updates

### Trello Import

**One-Time Import:**
- Import boards
- Import lists as columns
- Import cards
- Preserve checklists, labels

### Jira Import

**Import:**
- Import projects
- Map statuses to columns
- Import issues with metadata

**Optional Sync:**
- Read-only sync from Jira
- Status updates

### Authentication

**OAuth:**
- GitHub OAuth App
- GitLab OAuth
- Jira OAuth

**Tokens:**
- Personal access tokens
- Stored in keyring

### Conflict Resolution

When external changes conflict:
- Detect conflict
- Show diff
- User chooses resolution
- Automatic merge when safe

### Selective Sync

Choose what to sync:
- Specific repositories
- Specific projects
- Label filters
- Assignee filters

### Team Sync

Same protocol as kanban-standalone:
- Real-time sync
- Conflict resolution
- Multi-user support

## Data Model

```rust
pub struct ExternalSource {
    pub id: Uuid,
    pub source_type: SourceType,
    pub config: serde_json::Value,
    pub last_sync: Option<DateTime<Utc>>,
}

pub enum SourceType {
    GitHub { owner: String, repo: String },
    GitLab { project_id: String },
    Trello { board_id: String },
    Jira { project_key: String },
}

pub struct LinkedCard {
    pub card_id: Uuid,
    pub source_id: Uuid,
    pub external_id: String,
    pub external_url: String,
    pub last_synced: DateTime<Utc>,
    pub sync_status: SyncStatus,
}

pub enum SyncStatus {
    Synced,
    LocalChanges,
    RemoteChanges,
    Conflict,
}
```

## Keybindings

Inherits from kanban-standalone, plus:

| Key | Action |
|-----|--------|
| `I` | Import from external |
| `X` | External sync settings |
| `S` | Sync now |
| `C` | View conflicts |
| `g` | Open in GitHub/GitLab |
| `L` | Link card to issue |
| `U` | Unlink card |

## Configuration

```toml
# ~/.config/kanban-integrated/config.toml
[github]
token_source = "keyring"
repos = ["owner/repo1", "owner/repo2"]

[github.label_mapping]
"status: backlog" = "Backlog"
"status: doing" = "In Progress"
"status: done" = "Done"

[gitlab]
url = "https://gitlab.com"
token_source = "keyring"
projects = ["group/project"]

[sync]
interval_secs = 300
auto_sync = true
conflict_strategy = "prompt"  # prompt, local, remote

[team_sync]
enabled = true
server = "ws://localhost:8080"
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
tokio-tungstenite = "0.24"
reqwest = { workspace = true }
octocrab = "0.41"
keyring = "3"
oauth2 = "4"
```
