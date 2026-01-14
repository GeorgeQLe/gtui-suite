use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapType {
    Partition,
    File,
    Zram,
}

impl SwapType {
    pub fn name(&self) -> &'static str {
        match self {
            SwapType::Partition => "Partition",
            SwapType::File => "File",
            SwapType::Zram => "ZRAM",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapDevice {
    pub path: String,
    pub swap_type: SwapType,
    pub size_mb: u64,
    pub used_mb: u64,
    pub priority: i32,
    pub enabled: bool,
}

impl SwapDevice {
    pub fn usage_percent(&self) -> f64 {
        if self.size_mb > 0 {
            (self.used_mb as f64 / self.size_mb as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapSample {
    pub timestamp: u64,
    pub used_mb: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessSwap {
    pub pid: u32,
    pub name: String,
    pub swap_mb: u64,
    pub rss_mb: u64,
}

pub struct App {
    pub devices: Vec<SwapDevice>,
    pub processes: Vec<ProcessSwap>,
    pub history: Vec<SwapSample>,
    pub selected: usize,
    pub show_processes: bool,
    pub swappiness: u32,
    pub tick_count: u64,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            devices: create_demo_devices(),
            processes: create_demo_processes(),
            history: create_demo_history(),
            selected: 0,
            show_processes: false,
            swappiness: 60,
            tick_count: 0,
            status_message: None,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;

        if self.tick_count % 10 == 0 {
            let base = 500.0 + (self.tick_count as f64 * 0.03).sin() * 200.0;
            self.history.push(SwapSample {
                timestamp: self.tick_count,
                used_mb: base as u64,
            });

            if self.history.len() > 60 {
                self.history.remove(0);
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
                    self.devices.len()
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
                    "Viewing processes using swap".to_string()
                } else {
                    "Viewing swap devices".to_string()
                });
            }
            KeyCode::Char('e') => {
                if !self.show_processes {
                    if let Some(device) = self.devices.get_mut(self.selected) {
                        device.enabled = !device.enabled;
                        self.status_message = Some(format!(
                            "{} {}",
                            device.path,
                            if device.enabled { "enabled" } else { "disabled" }
                        ));
                    }
                }
            }
            KeyCode::Char('+') => {
                if self.swappiness < 100 {
                    self.swappiness += 10;
                    self.status_message = Some(format!("Swappiness: {}", self.swappiness));
                }
            }
            KeyCode::Char('-') => {
                if self.swappiness > 0 {
                    self.swappiness -= 10;
                    self.status_message = Some(format!("Swappiness: {}", self.swappiness));
                }
            }
            KeyCode::Char('c') => {
                self.status_message = Some("Would clear swap cache...".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn total_swap(&self) -> u64 {
        self.devices.iter().filter(|d| d.enabled).map(|d| d.size_mb).sum()
    }

    pub fn used_swap(&self) -> u64 {
        self.devices.iter().filter(|d| d.enabled).map(|d| d.used_mb).sum()
    }

    pub fn swap_percent(&self) -> f64 {
        let total = self.total_swap();
        if total > 0 {
            (self.used_swap() as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav Tab:toggle e:enable/disable +/-:swappiness c:clear q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_devices() -> Vec<SwapDevice> {
    vec![
        SwapDevice {
            path: "/dev/sda2".to_string(),
            swap_type: SwapType::Partition,
            size_mb: 8192,
            used_mb: 512,
            priority: -2,
            enabled: true,
        },
        SwapDevice {
            path: "/swapfile".to_string(),
            swap_type: SwapType::File,
            size_mb: 4096,
            used_mb: 256,
            priority: -3,
            enabled: true,
        },
        SwapDevice {
            path: "/dev/zram0".to_string(),
            swap_type: SwapType::Zram,
            size_mb: 2048,
            used_mb: 1024,
            priority: 100,
            enabled: true,
        },
        SwapDevice {
            path: "/dev/nvme0n1p3".to_string(),
            swap_type: SwapType::Partition,
            size_mb: 16384,
            used_mb: 0,
            priority: -1,
            enabled: false,
        },
    ]
}

fn create_demo_processes() -> Vec<ProcessSwap> {
    vec![
        ProcessSwap { pid: 1234, name: "firefox".to_string(), swap_mb: 450, rss_mb: 1200 },
        ProcessSwap { pid: 2345, name: "code".to_string(), swap_mb: 280, rss_mb: 800 },
        ProcessSwap { pid: 3456, name: "slack".to_string(), swap_mb: 180, rss_mb: 500 },
        ProcessSwap { pid: 4567, name: "spotify".to_string(), swap_mb: 120, rss_mb: 350 },
        ProcessSwap { pid: 5678, name: "java".to_string(), swap_mb: 350, rss_mb: 2000 },
        ProcessSwap { pid: 6789, name: "chrome".to_string(), swap_mb: 200, rss_mb: 1500 },
    ]
}

fn create_demo_history() -> Vec<SwapSample> {
    (0..30)
        .map(|i| SwapSample {
            timestamp: i,
            used_mb: (500.0 + (i as f64 * 0.1).sin() * 200.0) as u64,
        })
        .collect()
}
