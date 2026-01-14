use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct ProcessBandwidth {
    pub pid: u32,
    pub name: String,
    pub upload_rate: u64,    // bytes per second
    pub download_rate: u64,  // bytes per second
    pub total_upload: u64,   // total bytes
    pub total_download: u64, // total bytes
}

impl ProcessBandwidth {
    pub fn upload_rate_formatted(&self) -> String {
        format_rate(self.upload_rate)
    }

    pub fn download_rate_formatted(&self) -> String {
        format_rate(self.download_rate)
    }

    pub fn total_upload_formatted(&self) -> String {
        format_size(self.total_upload)
    }

    pub fn total_download_formatted(&self) -> String {
        format_size(self.total_download)
    }
}

fn format_rate(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Download,
    Upload,
    TotalDownload,
    TotalUpload,
    Name,
}

pub struct App {
    pub processes: Vec<ProcessBandwidth>,
    pub selected: usize,
    pub sort_by: SortBy,
    pub paused: bool,
    pub total_upload: u64,
    pub total_download: u64,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            selected: 0,
            sort_by: SortBy::Download,
            paused: false,
            total_upload: 0,
            total_download: 0,
            status_message: None,
        }
    }

    pub fn load_processes(&mut self) {
        self.processes = create_demo_processes();
        self.sort_processes();
        self.update_totals();
    }

    fn sort_processes(&mut self) {
        match self.sort_by {
            SortBy::Download => self.processes.sort_by(|a, b| b.download_rate.cmp(&a.download_rate)),
            SortBy::Upload => self.processes.sort_by(|a, b| b.upload_rate.cmp(&a.upload_rate)),
            SortBy::TotalDownload => self.processes.sort_by(|a, b| b.total_download.cmp(&a.total_download)),
            SortBy::TotalUpload => self.processes.sort_by(|a, b| b.total_upload.cmp(&a.total_upload)),
            SortBy::Name => self.processes.sort_by(|a, b| a.name.cmp(&b.name)),
        }
    }

    fn update_totals(&mut self) {
        self.total_upload = self.processes.iter().map(|p| p.upload_rate).sum();
        self.total_download = self.processes.iter().map(|p| p.download_rate).sum();
    }

    pub fn update_stats(&mut self) {
        if self.paused {
            return;
        }

        // Simulate bandwidth changes
        for process in &mut self.processes {
            let factor = 0.8 + (rand_float() * 0.4);
            process.download_rate = ((process.download_rate as f64) * factor) as u64;
            process.upload_rate = ((process.upload_rate as f64) * factor) as u64;
            process.total_download += process.download_rate;
            process.total_upload += process.upload_rate;
        }

        self.sort_processes();
        self.update_totals();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.processes.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('s') => {
                self.sort_by = match self.sort_by {
                    SortBy::Download => SortBy::Upload,
                    SortBy::Upload => SortBy::TotalDownload,
                    SortBy::TotalDownload => SortBy::TotalUpload,
                    SortBy::TotalUpload => SortBy::Name,
                    SortBy::Name => SortBy::Download,
                };
                self.sort_processes();
                self.status_message = Some(format!("Sorted by {:?}", self.sort_by));
            }
            KeyCode::Char(' ') => {
                self.paused = !self.paused;
                self.status_message = Some(if self.paused {
                    "Paused".to_string()
                } else {
                    "Resumed".to_string()
                });
            }
            KeyCode::Char('r') => {
                self.load_processes();
                self.status_message = Some("Refreshed".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        format!(
            "↓{} ↑{} | s:sort space:pause r:refresh | Sort: {:?}",
            format_rate(self.total_download),
            format_rate(self.total_upload),
            self.sort_by
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn rand_float() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

fn create_demo_processes() -> Vec<ProcessBandwidth> {
    vec![
        ProcessBandwidth {
            pid: 1234,
            name: "firefox".to_string(),
            upload_rate: 50 * 1024,
            download_rate: 500 * 1024,
            total_upload: 15 * 1024 * 1024,
            total_download: 250 * 1024 * 1024,
        },
        ProcessBandwidth {
            pid: 2345,
            name: "chrome".to_string(),
            upload_rate: 30 * 1024,
            download_rate: 350 * 1024,
            total_upload: 8 * 1024 * 1024,
            total_download: 180 * 1024 * 1024,
        },
        ProcessBandwidth {
            pid: 3456,
            name: "spotify".to_string(),
            upload_rate: 5 * 1024,
            download_rate: 200 * 1024,
            total_upload: 1 * 1024 * 1024,
            total_download: 500 * 1024 * 1024,
        },
        ProcessBandwidth {
            pid: 4567,
            name: "dropbox".to_string(),
            upload_rate: 150 * 1024,
            download_rate: 80 * 1024,
            total_upload: 2 * 1024 * 1024 * 1024,
            total_download: 500 * 1024 * 1024,
        },
        ProcessBandwidth {
            pid: 5678,
            name: "slack".to_string(),
            upload_rate: 10 * 1024,
            download_rate: 25 * 1024,
            total_upload: 50 * 1024 * 1024,
            total_download: 120 * 1024 * 1024,
        },
        ProcessBandwidth {
            pid: 6789,
            name: "ssh".to_string(),
            upload_rate: 2 * 1024,
            download_rate: 5 * 1024,
            total_upload: 10 * 1024 * 1024,
            total_download: 25 * 1024 * 1024,
        },
        ProcessBandwidth {
            pid: 7890,
            name: "docker".to_string(),
            upload_rate: 100 * 1024,
            download_rate: 300 * 1024,
            total_upload: 50 * 1024 * 1024,
            total_download: 1024 * 1024 * 1024,
        },
        ProcessBandwidth {
            pid: 8901,
            name: "curl".to_string(),
            upload_rate: 1024,
            download_rate: 2 * 1024 * 1024,
            total_upload: 1024 * 1024,
            total_download: 50 * 1024 * 1024,
        },
    ]
}
