# git-client

Full-featured Git client with GitHub/GitLab integration.

## Architecture Decisions

### Interactive Rebase UI
- **Visual + keyboard**: Show commits as visual list items
- J/K to reorder commits, s/e/d for squash/edit/drop
- Intuitive for beginners, efficient for power users
- Clear visual feedback for pending operations

### PR/MR Comment Display
- **Markdown preview**: Render markdown in comment display
- Plain text editing mode for composing
- Matches GitHub/GitLab web experience
- Reuse markdown rendering from note-manager crates

### Large Repository Performance
- **Lazy load + search**: Load only visible branches initially
- Fuzzy search for finding branches not currently loaded
- Handles repos with 1000+ branches efficiently
- Fast initial load, search handles scale

## Features

### Local Git Operations

**Staging:**
- Stage/unstage files
- Hunk-level staging
- Interactive add

**Commits:**
- Commit with message editor
- Amend last commit
- Multi-line messages

**Branches:**
- Create, delete, rename
- Switch branches
- Checkout remote branches

**Merging:**
- Merge branches
- Resolve conflicts (integrate with diff-tool)
- Abort merge

**Rebasing:**
- Interactive rebase
- Reorder, squash, edit commits
- Abort/continue rebase

**Cherry-pick:**
- Pick commits from other branches

**Stash:**
- Stash changes
- List, apply, pop, drop
- Stash with message

### Diff Viewer

**Modes:**
- Unified diff
- Side-by-side
- Word-level highlighting

**Semantic:**
- Detect moved code blocks
- Renamed variable tracking

### Log Browser

**Visualization:**
- Graph view (branches/merges)
- Flat list
- Search commits

**Details:**
- Commit info
- Changed files
- Full diff

### GitHub/GitLab Integration

**Authentication:**
- Personal Access Tokens
- OAuth Device Flow
- System keyring storage

**Pull Requests:**
- List PRs
- Create PR
- View PR details
- PR comments
- Review changes

**Issues:**
- List issues
- Create issue
- Comment

**CI Status:**
- View workflow runs
- Job status inline
- View logs

### Additional

**Blame:**
- Line-by-line attribution
- Navigate to commit

**Submodules:**
- View status
- Update submodules

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch views (status/log/branches) |
| `j/k` | Navigate |
| `enter` | View details/diff |
| `s` | Stage file/hunk |
| `u` | Unstage |
| `c` | Commit |
| `a` | Amend |
| `b` | Branch menu |
| `m` | Merge |
| `r` | Rebase |
| `p` | Push |
| `P` | Pull |
| `f` | Fetch |
| `S` | Stash |
| `l` | Log view |
| `d` | Diff view |
| `B` | Blame |
| `g` | GitHub/GitLab menu |
| `/` | Search |
| `q` | Quit |

## Configuration

```toml
# ~/.config/git-client/config.toml
[git]
sign_commits = false
default_branch = "main"

[diff]
view = "side-by-side"
word_level = true
context_lines = 3

[github]
token_source = "keyring"  # keyring, env, file

[gitlab]
url = "https://gitlab.com"
token_source = "keyring"

[ui]
show_untracked = true
confirm_dangerous = true
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
git2 = { workspace = true }
serde = { workspace = true }
chrono = { workspace = true }
tokio = { workspace = true }
reqwest = { workspace = true }
octocrab = "0.41"  # GitHub API
keyring = "3"
similar = "2"
syntect = "5"
```
