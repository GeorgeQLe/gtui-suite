# metrics-viewer

Prometheus/Grafana metrics viewer with PromQL support.

## Architecture Decisions

### High-Cardinality Series Display
- **Top N + aggregate**: Show top 10 series by value, aggregate rest into 'other'
- Keeps graph readable and performant
- Adjustable N in settings
- Click 'other' to drill down into hidden series
- Table view available for full data when needed

## Features

### Prometheus Integration

**Connection:**
- Connect to Prometheus server
- Multiple endpoints support
- Authentication (basic, bearer)

**PromQL Editor:**
- Query input with completion
- Query validation
- Range and instant queries

### Visualization

**Line Graphs:**
- ASCII sparklines
- Time series charts
- Multiple series overlay

**Tables:**
- Metric values
- Label breakdowns

**Single Stats:**
- Current value
- Delta from previous
- Trend indicator

### Dashboard Mode

**Grafana-Inspired:**
- Multiple panels
- Configurable layout
- Auto-refresh

**Panel Types:**
- Graph
- Stat
- Table
- Gauge
- Heatmap (simple)

**Save/Load:**
- Dashboard definitions in TOML
- Share configurations

### Alert Management

**View Alerts:**
- Active alerts
- Firing/pending state
- Labels and annotations

**Actions:**
- Silence alerts
- Acknowledge
- View alert history

### Query Features

**Time Range:**
- Presets (last 5m, 1h, 24h, 7d)
- Custom range
- Relative time

**Label Filtering:**
- Discover labels
- Filter by label values

**Aggregation:**
- Built-in aggregation functions
- Rate, sum, avg, etc.

## PromQL Examples

```promql
# CPU usage
100 - (avg by(instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Memory usage percentage
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100

# HTTP request rate
rate(http_requests_total[5m])
```

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch panels |
| `enter` | Execute query |
| `e` | Edit query |
| `t` | Time range |
| `r` | Refresh |
| `R` | Auto-refresh toggle |
| `a` | Alerts view |
| `d` | Dashboard view |
| `s` | Save dashboard |
| `l` | Load dashboard |
| `/` | Search metrics |
| `m` | Metric browser |
| `q` | Quit |

## Configuration

```toml
# ~/.config/metrics-viewer/config.toml
[prometheus]
url = "http://localhost:9090"
auth = "none"  # none, basic, bearer
# user = ""
# password = ""
# token = ""

[display]
refresh_secs = 30
time_range = "1h"
graph_height = 10

[dashboards]
path = "~/.config/metrics-viewer/dashboards"
```

## Dashboard Definition

```toml
# ~/.config/metrics-viewer/dashboards/system.toml
name = "System Overview"
refresh_secs = 30

[[panels]]
title = "CPU Usage"
type = "graph"
query = "100 - avg(rate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100"
position = { row = 0, col = 0, width = 2, height = 1 }

[[panels]]
title = "Memory Usage"
type = "stat"
query = "(1 - node_memory_MemAvailable_bytes/node_memory_MemTotal_bytes) * 100"
position = { row = 0, col = 2, width = 1, height = 1 }
unit = "%"
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
serde_json = { workspace = true }
toml = { workspace = true }
```
