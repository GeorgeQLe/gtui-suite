# log-anomaly-detector

Log analysis with pattern matching, statistics, and ML-based detection.

## Architecture Decisions

### Baseline Training Duration
- **Rolling window**: Continuously update baseline with sliding 7-day window
- Adapts automatically to changing workload patterns
- "Exclude period" capability to mark known-bad periods for exclusion from baseline
- No fixed training phase needed, starts learning immediately

### False Positive Handling
- **Auto-adjust rule + learn**: When user marks alert as false positive, adjust threshold and add to learned exceptions
- Show "Marked as false positive" count on each rule
- Allow review and reset of learned exceptions
- Reduces noise over time while maintaining ability to audit adjustments

### Threshold Drift Prevention
- **Weekly learning review**: After N false positives per rule, prompt user to review learned exceptions
- Prevents unbounded threshold drift from accumulated false positive markings
- Shows summary of adjusted thresholds with option to reset
- Keeps detection effective over time

## Features

### Detection Approaches

**Pattern/Regex Rules:**
```rust
pub struct PatternRule {
    pub name: String,
    pub pattern: Regex,
    pub severity: Severity,
    pub description: String,
    pub enabled: bool,
}
```

Pre-defined patterns:
- Error messages
- Stack traces
- Failed logins
- Suspicious IPs
- SQL injection attempts

**Statistical Baseline:**
- Learn normal log frequency
- Detect unusual spikes or drops
- Time-based patterns (hourly, daily)

**Heuristic Highlighting:**
- Highlight errors, warnings
- IP addresses
- Timestamps
- URLs
- Email addresses

**ML Classification (Optional):**
- Train on labeled data
- Classify new entries
- Anomaly scoring

### Training Mode

For statistical baseline:
```rust
pub struct BaselineConfig {
    pub training_period: Duration,
    pub metrics: Vec<BaselineMetric>,
}

pub enum BaselineMetric {
    EventFrequency { window: Duration },
    ErrorRate,
    UniqueIps,
    ResponseTime,
}
```

### Alert Generation

```rust
pub struct Alert {
    pub id: Uuid,
    pub rule_name: String,
    pub severity: Severity,
    pub message: String,
    pub log_entries: Vec<LogEntry>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub count: u32,
    pub acknowledged: bool,
}

pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}
```

### Integration

**With log-viewer:**
- Jump to relevant log entries
- Context around anomalies

**Continuous Monitoring:**
- Background service mode
- Real-time alerting
- Desktop notifications

### Features

**Rule Management:**
- Create/edit/delete rules
- Test rules against sample data
- Import/export rules

**Export Findings:**
- Report generation
- CSV export
- JSON export

## Views

**Dashboard:**
- Alert summary
- Recent anomalies
- Baseline status

**Alerts View:**
- Active alerts
- Acknowledge/dismiss
- View related logs

**Rules View:**
- Rule list
- Enable/disable
- Test rules

**Training View:**
- Baseline progress
- Learned patterns
- Adjust thresholds

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `enter` | View details |
| `Tab` | Switch views |
| `a` | Acknowledge alert |
| `d` | Dismiss alert |
| `l` | View logs |
| `r` | Rules view |
| `n` | New rule |
| `t` | Train baseline |
| `e` | Export findings |
| `R` | Refresh |
| `q` | Quit |

## Configuration

```toml
# ~/.config/log-anomaly-detector/config.toml
[input]
files = ["/var/log/syslog", "/var/log/auth.log"]
watch = true

[baseline]
enabled = true
training_days = 7
sensitivity = "medium"  # low, medium, high

[rules]
builtin = true
custom_path = "~/.config/log-anomaly-detector/rules"

[alerts]
notification_cmd = "notify-send"
email = ""
min_severity = "warning"

[ml]
enabled = false
model_path = "~/.local/share/log-anomaly-detector/model"
```

## Built-in Rules

```toml
# Example built-in rules
[[rules]]
name = "Failed SSH Login"
pattern = "Failed password for .* from (\\d+\\.\\d+\\.\\d+\\.\\d+)"
severity = "warning"
description = "SSH authentication failure"

[[rules]]
name = "Out of Memory"
pattern = "Out of memory|OOM|oom-killer"
severity = "critical"
description = "System out of memory event"

[[rules]]
name = "Disk Full"
pattern = "No space left on device"
severity = "critical"
description = "Disk space exhausted"
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
regex = "1"
notify = "6"
statrs = "0.17"  # Statistics
linfa = { version = "0.7", optional = true }  # ML
notify-rust = "4"
```
