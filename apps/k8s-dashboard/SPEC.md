# k8s-dashboard

Kubernetes cluster dashboard with multi-cluster support.

## Architecture Decisions

### Multi-Container Log Viewing
- **Container selector**: Dropdown to select which container to view
- View one container at a time for clean, focused logs
- Quick switch keybind to toggle between containers
- Remember last selection per pod

### Operation Failure Feedback
- **Toast + event detail**: Show toast notification on failure
- Toast includes brief reason, links to events view
- Full failure details available in filtered events
- Non-blocking, doesn't interrupt other work

### Offline Mode
- **Full kubectl parity**: Cache complete cluster state for offline use
- Queue mutations (scale, delete, apply) for execution on reconnect
- Supports all read operations offline from cache
- Sync queue visible with pending operation count
- Cache invalidation on reconnect with conflict detection

## Features

### Multi-Cluster

**Context Management:**
- Switch between kubectl contexts
- Multiple clusters simultaneously
- Kubeconfig parsing

### Resource Views

**Workloads:**
- Pods (logs, exec, delete)
- Deployments (scale, restart)
- StatefulSets
- DaemonSets
- Jobs, CronJobs

**Config:**
- ConfigMaps
- Secrets (with show/hide)

**Networking:**
- Services
- Ingresses
- NetworkPolicies

**Storage:**
- PersistentVolumeClaims
- PersistentVolumes
- StorageClasses

**Cluster:**
- Nodes (cordon, drain)
- Namespaces
- Events

### Pod Operations

**Logs:**
- Stream logs
- Previous container
- Multi-container selection
- Follow mode

**Exec:**
- Shell into container
- Run commands

**Port Forward:**
- Local port forwarding
- Manage active forwards

### Observability

**metrics-server Integration:**
- CPU/memory per pod
- Node resource usage
- Graphs over time

**Events Stream:**
- Real-time events
- Filter by namespace/type

### Actions

**Deployments:**
- Scale replicas
- Restart (rollout restart)
- View rollout status

**Apply Manifests:**
- Apply YAML from file
- Delete resources
- Edit in $EDITOR

### Watch Mode

Real-time updates:
- Pod status changes
- Event stream
- Resource counts

## Data Model

```rust
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: PodStatus,
    pub ready: String,  // "2/2"
    pub restarts: u32,
    pub age: Duration,
    pub node: String,
    pub containers: Vec<ContainerInfo>,
    pub cpu: Option<f64>,
    pub memory: Option<u64>,
}

pub enum PodStatus {
    Running,
    Pending,
    Succeeded,
    Failed,
    Unknown,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `enter` | View details |
| `Tab` | Switch resource type |
| `n` | Change namespace |
| `c` | Change context |
| `l` | View logs |
| `e` | Exec into pod |
| `d` | Delete (confirm) |
| `s` | Scale dialog |
| `R` | Restart rollout |
| `y` | View YAML |
| `E` | Edit YAML |
| `a` | Apply manifest |
| `f` | Port forward |
| `g` | Toggle graphs |
| `w` | Toggle watch mode |
| `/` | Search |
| `q` | Quit |

## Configuration

```toml
# ~/.config/k8s-dashboard/config.toml
[kubernetes]
kubeconfig = "~/.kube/config"
default_namespace = "default"
context = ""  # Empty = current context

[display]
refresh_secs = 5
show_metrics = true
watch_mode = true

[logs]
max_lines = 1000
follow = true
timestamps = true

[resources]
default_view = "pods"
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
kube = { version = "0.96", features = ["client", "runtime", "derive"] }
k8s-openapi = { version = "0.23", features = ["latest"] }
futures = "0.3"
```
