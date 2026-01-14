use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct FunctionProfile {
    pub name: String,
    pub module: String,
    pub self_time_ms: f64,
    pub total_time_ms: f64,
    pub call_count: u64,
    pub avg_time_us: f64,
}

impl FunctionProfile {
    pub fn self_percent(&self, total: f64) -> f64 {
        if total > 0.0 {
            (self.self_time_ms / total) * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct CpuSample {
    pub timestamp: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    SelfTime,
    TotalTime,
    CallCount,
    AvgTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Functions,
    FlameGraph,
    Timeline,
}

pub struct App {
    pub profiles: Vec<FunctionProfile>,
    pub cpu_samples: Vec<CpuSample>,
    pub selected: usize,
    pub sort_by: SortBy,
    pub view_mode: ViewMode,
    pub is_recording: bool,
    pub total_time_ms: f64,
    pub tick_count: u64,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let profiles = create_demo_profiles();
        let total_time_ms = profiles.iter().map(|p| p.self_time_ms).sum();

        Self {
            profiles,
            cpu_samples: create_demo_samples(),
            selected: 0,
            sort_by: SortBy::SelfTime,
            view_mode: ViewMode::Functions,
            is_recording: false,
            total_time_ms,
            tick_count: 0,
            status_message: None,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;

        if self.is_recording && self.tick_count % 10 == 0 {
            let usage = 30.0 + (self.tick_count as f64 * 0.1).sin() * 20.0;
            self.cpu_samples.push(CpuSample {
                timestamp: self.tick_count,
                usage_percent: usage.clamp(0.0, 100.0),
            });

            if self.cpu_samples.len() > 100 {
                self.cpu_samples.remove(0);
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.profiles.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('1') => {
                self.sort_by = SortBy::SelfTime;
                self.sort_profiles();
                self.status_message = Some("Sorted by self time".to_string());
            }
            KeyCode::Char('2') => {
                self.sort_by = SortBy::TotalTime;
                self.sort_profiles();
                self.status_message = Some("Sorted by total time".to_string());
            }
            KeyCode::Char('3') => {
                self.sort_by = SortBy::CallCount;
                self.sort_profiles();
                self.status_message = Some("Sorted by call count".to_string());
            }
            KeyCode::Char('4') => {
                self.sort_by = SortBy::AvgTime;
                self.sort_profiles();
                self.status_message = Some("Sorted by avg time".to_string());
            }
            KeyCode::Char('f') => {
                self.view_mode = ViewMode::Functions;
                self.status_message = Some("View: Functions".to_string());
            }
            KeyCode::Char('g') => {
                self.view_mode = ViewMode::FlameGraph;
                self.status_message = Some("View: Flame Graph".to_string());
            }
            KeyCode::Char('t') => {
                self.view_mode = ViewMode::Timeline;
                self.status_message = Some("View: Timeline".to_string());
            }
            KeyCode::Char('r') => {
                self.is_recording = !self.is_recording;
                self.status_message = Some(if self.is_recording {
                    "Recording started".to_string()
                } else {
                    "Recording stopped".to_string()
                });
            }
            KeyCode::Char('c') => {
                self.cpu_samples.clear();
                self.status_message = Some("Samples cleared".to_string());
            }
            _ => {}
        }
        false
    }

    fn sort_profiles(&mut self) {
        match self.sort_by {
            SortBy::SelfTime => self.profiles.sort_by(|a, b| {
                b.self_time_ms.partial_cmp(&a.self_time_ms).unwrap()
            }),
            SortBy::TotalTime => self.profiles.sort_by(|a, b| {
                b.total_time_ms.partial_cmp(&a.total_time_ms).unwrap()
            }),
            SortBy::CallCount => self.profiles.sort_by(|a, b| {
                b.call_count.cmp(&a.call_count)
            }),
            SortBy::AvgTime => self.profiles.sort_by(|a, b| {
                b.avg_time_us.partial_cmp(&a.avg_time_us).unwrap()
            }),
        }
    }

    pub fn current_cpu_usage(&self) -> f64 {
        self.cpu_samples.last().map(|s| s.usage_percent).unwrap_or(0.0)
    }

    pub fn avg_cpu_usage(&self) -> f64 {
        if self.cpu_samples.is_empty() {
            0.0
        } else {
            let sum: f64 = self.cpu_samples.iter().map(|s| s.usage_percent).sum();
            sum / self.cpu_samples.len() as f64
        }
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        let rec = if self.is_recording { "[REC]" } else { "" };
        format!("j/k:nav 1-4:sort f:funcs g:flame t:timeline r:record c:clear q:quit {}", rec)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_profiles() -> Vec<FunctionProfile> {
    vec![
        FunctionProfile {
            name: "process_request".to_string(),
            module: "server::handler".to_string(),
            self_time_ms: 450.5,
            total_time_ms: 1200.0,
            call_count: 10000,
            avg_time_us: 120.0,
        },
        FunctionProfile {
            name: "parse_json".to_string(),
            module: "parser::json".to_string(),
            self_time_ms: 320.0,
            total_time_ms: 320.0,
            call_count: 50000,
            avg_time_us: 6.4,
        },
        FunctionProfile {
            name: "db_query".to_string(),
            module: "database::sql".to_string(),
            self_time_ms: 280.0,
            total_time_ms: 400.0,
            call_count: 5000,
            avg_time_us: 80.0,
        },
        FunctionProfile {
            name: "serialize_response".to_string(),
            module: "server::response".to_string(),
            self_time_ms: 150.0,
            total_time_ms: 150.0,
            call_count: 10000,
            avg_time_us: 15.0,
        },
        FunctionProfile {
            name: "validate_token".to_string(),
            module: "auth::jwt".to_string(),
            self_time_ms: 100.0,
            total_time_ms: 180.0,
            call_count: 10000,
            avg_time_us: 18.0,
        },
        FunctionProfile {
            name: "log_request".to_string(),
            module: "logging".to_string(),
            self_time_ms: 80.0,
            total_time_ms: 80.0,
            call_count: 10000,
            avg_time_us: 8.0,
        },
        FunctionProfile {
            name: "cache_lookup".to_string(),
            module: "cache::redis".to_string(),
            self_time_ms: 60.0,
            total_time_ms: 60.0,
            call_count: 8000,
            avg_time_us: 7.5,
        },
        FunctionProfile {
            name: "compress_data".to_string(),
            module: "util::compression".to_string(),
            self_time_ms: 45.0,
            total_time_ms: 45.0,
            call_count: 2000,
            avg_time_us: 22.5,
        },
    ]
}

fn create_demo_samples() -> Vec<CpuSample> {
    (0..50)
        .map(|i| CpuSample {
            timestamp: i,
            usage_percent: 30.0 + (i as f64 * 0.2).sin() * 25.0,
        })
        .collect()
}
