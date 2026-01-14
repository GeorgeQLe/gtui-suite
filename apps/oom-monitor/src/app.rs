use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct OomEvent {
    pub timestamp: String,
    pub process: String,
    pub pid: u32,
    pub memory_mb: u64,
    pub oom_score: i32,
    pub killed: bool,
}

#[derive(Debug, Clone)]
pub struct MemoryPressure {
    pub timestamp: u64,
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub swap_used_mb: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessMemory {
    pub pid: u32,
    pub name: String,
    pub rss_mb: u64,
    pub oom_score: i32,
    pub oom_score_adj: i32,
}

pub struct App {
    pub events: Vec<OomEvent>,
    pub pressure_history: Vec<MemoryPressure>,
    pub processes: Vec<ProcessMemory>,
    pub selected: usize,
    pub show_processes: bool,
    pub alert_threshold: u8,
    pub tick_count: u64,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            events: create_demo_events(),
            pressure_history: create_demo_pressure(),
            processes: create_demo_processes(),
            selected: 0,
            show_processes: false,
            alert_threshold: 90,
            tick_count: 0,
            status_message: None,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;

        if self.tick_count % 10 == 0 {
            let base = 70.0 + (self.tick_count as f64 * 0.05).sin() * 15.0;
            let total: u64 = 16000;
            let used = (total as f64 * base / 100.0) as u64;

            self.pressure_history.push(MemoryPressure {
                timestamp: self.tick_count,
                total_mb: total,
                used_mb: used,
                available_mb: total - used,
                swap_used_mb: 500,
            });

            if self.pressure_history.len() > 60 {
                self.pressure_history.remove(0);
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
                let max = if self.show_processes {
                    self.processes.len()
                } else {
                    self.events.len()
                };
                if self.selected < max.saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Tab => {
                self.show_processes = !self.show_processes;
                self.selected = 0;
                self.status_message = Some(if self.show_processes {
                    "Viewing processes by OOM score".to_string()
                } else {
                    "Viewing OOM events".to_string()
                });
            }
            KeyCode::Char('+') => {
                if self.alert_threshold < 100 {
                    self.alert_threshold += 5;
                    self.status_message = Some(format!("Alert threshold: {}%", self.alert_threshold));
                }
            }
            KeyCode::Char('-') => {
                if self.alert_threshold > 50 {
                    self.alert_threshold -= 5;
                    self.status_message = Some(format!("Alert threshold: {}%", self.alert_threshold));
                }
            }
            KeyCode::Char('c') => {
                self.events.clear();
                self.status_message = Some("Events cleared".to_string());
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Refreshing...".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn current_memory_percent(&self) -> f64 {
        self.pressure_history
            .last()
            .map(|p| (p.used_mb as f64 / p.total_mb as f64) * 100.0)
            .unwrap_or(0.0)
    }

    pub fn killed_count(&self) -> usize {
        self.events.iter().filter(|e| e.killed).count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav Tab:toggle +/-:threshold c:clear r:refresh q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_events() -> Vec<OomEvent> {
    vec![
        OomEvent {
            timestamp: "2024-01-15 14:32:15".to_string(),
            process: "chrome".to_string(),
            pid: 12345,
            memory_mb: 4500,
            oom_score: 850,
            killed: true,
        },
        OomEvent {
            timestamp: "2024-01-15 10:15:42".to_string(),
            process: "java".to_string(),
            pid: 23456,
            memory_mb: 8200,
            oom_score: 920,
            killed: true,
        },
        OomEvent {
            timestamp: "2024-01-14 22:45:00".to_string(),
            process: "node".to_string(),
            pid: 34567,
            memory_mb: 2100,
            oom_score: 650,
            killed: true,
        },
        OomEvent {
            timestamp: "2024-01-14 18:20:33".to_string(),
            process: "python3".to_string(),
            pid: 45678,
            memory_mb: 3800,
            oom_score: 780,
            killed: false,
        },
    ]
}

fn create_demo_pressure() -> Vec<MemoryPressure> {
    (0..30)
        .map(|i| {
            let base = 70.0 + (i as f64 * 0.2).sin() * 15.0;
            let total: u64 = 16000;
            let used = (total as f64 * base / 100.0) as u64;
            MemoryPressure {
                timestamp: i,
                total_mb: total,
                used_mb: used,
                available_mb: total - used,
                swap_used_mb: 500,
            }
        })
        .collect()
}

fn create_demo_processes() -> Vec<ProcessMemory> {
    vec![
        ProcessMemory { pid: 1234, name: "chrome".to_string(), rss_mb: 2500, oom_score: 850, oom_score_adj: 0 },
        ProcessMemory { pid: 2345, name: "firefox".to_string(), rss_mb: 1800, oom_score: 720, oom_score_adj: 0 },
        ProcessMemory { pid: 3456, name: "code".to_string(), rss_mb: 1200, oom_score: 580, oom_score_adj: 0 },
        ProcessMemory { pid: 4567, name: "java".to_string(), rss_mb: 3500, oom_score: 920, oom_score_adj: 0 },
        ProcessMemory { pid: 5678, name: "node".to_string(), rss_mb: 800, oom_score: 450, oom_score_adj: 0 },
        ProcessMemory { pid: 6789, name: "postgres".to_string(), rss_mb: 600, oom_score: 200, oom_score_adj: -500 },
        ProcessMemory { pid: 7890, name: "dockerd".to_string(), rss_mb: 400, oom_score: 150, oom_score_adj: -999 },
    ]
}
