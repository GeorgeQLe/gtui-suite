# ci-dashboard

Multi-CI system dashboard with unified interface.

## Architecture Decisions

### Multi-System Polling Strategy
- **Independent polling**: Each CI system polled at its own optimal rate
- Respects per-system rate limits
- Show per-system 'last updated' timestamp
- Different systems may refresh at different times

### Workflow Retry Options
- **Both options**: Offer 'Retry failed jobs' and 'Retry entire workflow'
- Default highlight on 'retry failed' (common case, faster)
- Full retry available when complete rebuild needed
- Clear indication of what each option does

## Features

### CI Systems

**GitHub Actions:**
- Workflow runs
- Job status
- Log streaming
- Artifacts

**GitLab CI:**
- Pipeline status
- Job logs
- Artifacts

**Jenkins:**
- Build status
- Console output

**CircleCI:**
- Workflow status
- Job details

### Abstraction Layer

```rust
pub trait CIProvider: Send + Sync {
    async fn list_workflows(&self, repo: &str) -> Result<Vec<Workflow>>;
    async fn get_workflow_runs(&self, workflow_id: &str) -> Result<Vec<Run>>;
    async fn get_run_details(&self, run_id: &str) -> Result<RunDetails>;
    async fn get_job_logs(&self, job_id: &str) -> Result<String>;
    async fn retry_run(&self, run_id: &str) -> Result<()>;
    async fn cancel_run(&self, run_id: &str) -> Result<()>;
}

pub struct Workflow {
    pub id: String,
    pub name: String,
    pub path: String,
}

pub struct Run {
    pub id: String,
    pub workflow_id: String,
    pub status: RunStatus,
    pub conclusion: Option<Conclusion>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub jobs: Vec<Job>,
}

pub enum RunStatus {
    Queued,
    InProgress,
    Completed,
}

pub enum Conclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
}
```

### Unified View

**Dashboard:**
- All repos/projects
- Status overview
- Recent runs
- Failed runs highlighted

**Per-Workflow:**
- Run history
- Duration trends
- Success rate

**Per-Run:**
- Job breakdown
- Step-by-step status
- Duration per step

### Actions

- Retry failed run
- Cancel running job
- Trigger manual workflow
- View artifacts
- Download logs

### Notifications

- Alert on failure
- Desktop notification
- Optional sound

### Repository Grouping

- Group by organization
- Group by custom tags
- Filter by status

## Views

**Overview:**
```
┌────────────────────────────────────────────────────────────┐
│  CI Dashboard                                              │
├────────────────────────────────────────────────────────────┤
│  Repository              Workflow        Status    Time    │
│  ────────────────────────────────────────────────────────  │
│  org/repo-a              build           ● Pass    2m 34s  │
│  org/repo-a              test            ● Pass    5m 12s  │
│  org/repo-b              deploy          ⟳ Running 1m 23s  │
│  org/repo-c              build           ✗ Failed  0m 45s  │
└────────────────────────────────────────────────────────────┘
```

**Logs View:**
- Real-time log streaming
- ANSI color support
- Search within logs

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `enter` | View details |
| `l` | View logs |
| `r` | Retry run |
| `c` | Cancel run |
| `t` | Trigger workflow |
| `a` | Artifacts |
| `f` | Filter |
| `Tab` | Switch views |
| `/` | Search |
| `R` | Refresh |
| `q` | Quit |

## Configuration

```toml
# ~/.config/ci-dashboard/config.toml
[github]
token = ""  # or from keyring
repos = ["org/repo-a", "org/repo-b"]

[gitlab]
url = "https://gitlab.com"
token = ""
projects = ["group/project"]

[jenkins]
url = "https://jenkins.example.com"
user = ""
token = ""
jobs = ["job-name"]

[notifications]
on_failure = true
sound = false
command = "notify-send"

[display]
refresh_secs = 60
show_passed = true
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
tokio = { workspace = true }
reqwest = { workspace = true }
octocrab = "0.41"
keyring = "3"
```
