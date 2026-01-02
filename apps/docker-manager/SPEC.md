# docker-manager

Docker/Podman container and image management.

## Architecture Decisions

### Multi-Runtime Selection
- **Prompt on start**: Ask which runtime when both available
- Option to set choice as default for future sessions
- Keybind to switch runtime during session
- Clear indicator of active runtime in UI

### Exec Shell Detection
- **Detect available**: Try /bin/bash, then /bin/sh, then available shells
- Best shell experience for each container
- Handles minimal containers gracefully
- Slight startup delay for detection acceptable

### Runtime Switch Cache Handling
- **Clear cache on switch**: Warn user, then clear all cached container/image data
- Prevents confusion from stale data across runtimes
- Docker and Podman caches are completely separate
- User acknowledges before switch proceeds

## Features

### Runtime Detection

Auto-detect available runtime:
- Docker (via /var/run/docker.sock)
- Podman (via XDG_RUNTIME_DIR/podman/podman.sock)
- User selection if both available

### Container Operations

**List:**
- Running containers
- All containers (including stopped)
- Filter by name, image, status

**Actions:**
- Start / Stop / Restart
- Pause / Unpause
- Remove
- Kill (with signal)

**Logs:**
- Stream container logs
- Follow mode
- Timestamps
- Filter by time

**Exec:**
- Execute command in container
- Interactive shell

**Inspect:**
- Container details
- Environment variables
- Mounts
- Network settings
- Resource usage

### Image Operations

**List:**
- All images
- Dangling images
- Filter by name

**Actions:**
- Pull image
- Remove image
- Tag image
- Push image

**Build:**
- Build from Dockerfile
- Build context selection
- Progress display

### Volume Management

- List volumes
- Create volume
- Remove volume
- Inspect volume

### Network Management

- List networks
- Create network
- Remove network
- Connect/disconnect containers

### Docker Compose

If docker-compose.yml detected:
- Up / Down
- Logs (all services)
- Scale services
- Service status

## Data Model

```rust
pub struct Container {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub command: String,
    pub created: DateTime<Utc>,
    pub state: ContainerState,
    pub status: String,
    pub ports: Vec<PortBinding>,
    pub mounts: Vec<Mount>,
}

pub enum ContainerState {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
}
```

## Views

**Containers View:**
- Container list
- Status indicators
- Quick actions

**Images View:**
- Image list
- Size, created date
- Tags

**Logs View:**
- Container logs
- Real-time streaming
- Search

**Compose View:**
- Service list
- Status per service
- Combined logs

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `enter` | View details |
| `Tab` | Switch views |
| `s` | Start container |
| `S` | Stop container |
| `r` | Restart container |
| `R` | Remove container |
| `l` | View logs |
| `f` | Follow logs |
| `e` | Exec shell |
| `i` | Inspect |
| `p` | Pull image |
| `/` | Search |
| `c` | Compose menu |
| `q` | Quit |

## Configuration

```toml
# ~/.config/docker-manager/config.toml
[runtime]
prefer = "auto"  # auto, docker, podman
docker_socket = "/var/run/docker.sock"
podman_socket = "$XDG_RUNTIME_DIR/podman/podman.sock"

[display]
show_all_containers = false
show_sizes = true

[logs]
max_lines = 1000
timestamps = true

[compose]
detect_files = ["docker-compose.yml", "docker-compose.yaml", "compose.yml"]
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
chrono = { workspace = true }
tokio = { workspace = true }
bollard = "0.17"  # Docker API client
hyper = "1"
```
