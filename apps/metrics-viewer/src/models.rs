use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    pub metric: HashMap<String, String>,
    pub values: Vec<(f64, f64)>, // (timestamp, value)
}

impl MetricResult {
    pub fn new() -> Self {
        Self {
            metric: HashMap::new(),
            values: Vec::new(),
        }
    }

    pub fn with_labels(labels: Vec<(&str, &str)>) -> Self {
        let mut metric = HashMap::new();
        for (k, v) in labels {
            metric.insert(k.to_string(), v.to_string());
        }
        Self {
            metric,
            values: Vec::new(),
        }
    }

    pub fn label(&self, key: &str) -> Option<&str> {
        self.metric.get(key).map(|s| s.as_str())
    }

    pub fn latest_value(&self) -> Option<f64> {
        self.values.last().map(|(_, v)| *v)
    }

    pub fn display_name(&self) -> String {
        if let Some(name) = self.metric.get("__name__") {
            let labels: Vec<String> = self
                .metric
                .iter()
                .filter(|(k, _)| *k != "__name__")
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect();

            if labels.is_empty() {
                name.clone()
            } else {
                format!("{}{{{}}} ", name, labels.join(","))
            }
        } else {
            "unknown".to_string()
        }
    }
}

impl Default for MetricResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub status: String,
    pub data: QueryData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryData {
    pub result_type: String,
    pub result: Vec<MetricResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub name: String,
    pub state: AlertState,
    pub severity: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub active_at: DateTime<Utc>,
    pub value: Option<f64>,
}

impl Alert {
    pub fn new(name: &str, state: AlertState) -> Self {
        Self {
            name: name.to_string(),
            state,
            severity: "warning".to_string(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            active_at: Utc::now(),
            value: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertState {
    Firing,
    Pending,
    Resolved,
}

impl AlertState {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertState::Firing => "firing",
            AlertState::Pending => "pending",
            AlertState::Resolved => "resolved",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AlertState::Firing => "🔴",
            AlertState::Pending => "🟡",
            AlertState::Resolved => "🟢",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub name: String,
    pub refresh_secs: u64,
    pub panels: Vec<Panel>,
}

impl Dashboard {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            refresh_secs: 30,
            panels: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panel {
    pub title: String,
    pub panel_type: PanelType,
    pub query: String,
    pub position: PanelPosition,
    pub unit: Option<String>,
}

impl Panel {
    pub fn new(title: &str, panel_type: PanelType, query: &str) -> Self {
        Self {
            title: title.to_string(),
            panel_type,
            query: query.to_string(),
            position: PanelPosition::default(),
            unit: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelType {
    Graph,
    Stat,
    Table,
    Gauge,
}

impl PanelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PanelType::Graph => "graph",
            PanelType::Stat => "stat",
            PanelType::Table => "table",
            PanelType::Gauge => "gauge",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelPosition {
    pub row: u16,
    pub col: u16,
    pub width: u16,
    pub height: u16,
}

impl Default for PanelPosition {
    fn default() -> Self {
        Self {
            row: 0,
            col: 0,
            width: 1,
            height: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub step: u64, // seconds
}

impl TimeRange {
    pub fn last(duration: chrono::Duration) -> Self {
        let end = Utc::now();
        let start = end - duration;
        let step = duration.num_seconds() as u64 / 100; // ~100 data points
        Self {
            start,
            end,
            step: step.max(1),
        }
    }

    pub fn display(&self) -> String {
        let duration = self.end - self.start;
        if duration.num_days() > 0 {
            format!("{}d", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{}h", duration.num_hours())
        } else {
            format!("{}m", duration.num_minutes())
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeRangePreset {
    pub label: &'static str,
    pub duration: chrono::Duration,
}

impl TimeRangePreset {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                label: "5m",
                duration: chrono::Duration::minutes(5),
            },
            Self {
                label: "15m",
                duration: chrono::Duration::minutes(15),
            },
            Self {
                label: "1h",
                duration: chrono::Duration::hours(1),
            },
            Self {
                label: "6h",
                duration: chrono::Duration::hours(6),
            },
            Self {
                label: "24h",
                duration: chrono::Duration::hours(24),
            },
            Self {
                label: "7d",
                duration: chrono::Duration::days(7),
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Sparkline {
    pub data: Vec<f64>,
    pub min: f64,
    pub max: f64,
}

impl Sparkline {
    pub fn from_values(values: &[(f64, f64)]) -> Self {
        let data: Vec<f64> = values.iter().map(|(_, v)| *v).collect();
        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        Self {
            data,
            min: if min.is_finite() { min } else { 0.0 },
            max: if max.is_finite() { max } else { 100.0 },
        }
    }

    pub fn render(&self, width: usize) -> String {
        if self.data.is_empty() || width == 0 {
            return String::new();
        }

        let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let range = self.max - self.min;

        // Sample data to fit width
        let step = self.data.len() as f64 / width as f64;
        let mut result = String::with_capacity(width);

        for i in 0..width {
            let idx = (i as f64 * step) as usize;
            let value = self.data.get(idx).copied().unwrap_or(0.0);

            let normalized = if range > 0.0 {
                ((value - self.min) / range).clamp(0.0, 1.0)
            } else {
                0.5
            };

            let char_idx = (normalized * 7.0) as usize;
            result.push(chars[char_idx.min(7)]);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_result_display_name() {
        let mut result = MetricResult::new();
        result.metric.insert("__name__".to_string(), "cpu_usage".to_string());
        result.metric.insert("instance".to_string(), "localhost:9090".to_string());

        let name = result.display_name();
        assert!(name.contains("cpu_usage"));
        assert!(name.contains("instance"));
    }

    #[test]
    fn test_sparkline_render() {
        let values: Vec<(f64, f64)> = (0..10).map(|i| (i as f64, (i * 10) as f64)).collect();
        let sparkline = Sparkline::from_values(&values);
        let rendered = sparkline.render(10);
        assert_eq!(rendered.chars().count(), 10);
    }

    #[test]
    fn test_time_range() {
        let range = TimeRange::last(chrono::Duration::hours(1));
        assert_eq!(range.display(), "1h");
    }
}
