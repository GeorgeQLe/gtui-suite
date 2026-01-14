use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Lap {
    pub number: usize,
    pub elapsed_ms: u64,
    pub split_ms: u64,
}

pub struct App {
    pub running: bool,
    pub started_at: Option<Instant>,
    pub elapsed_ms: u64,
    pub paused_elapsed: u64,
    pub laps: Vec<Lap>,
    pub selected_lap: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: false,
            started_at: None,
            elapsed_ms: 0,
            paused_elapsed: 0,
            laps: Vec::new(),
            selected_lap: 0,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char(' ') | KeyCode::Enter => {
                if self.running {
                    self.pause();
                } else {
                    self.start();
                }
            }
            KeyCode::Char('l') => {
                self.lap();
            }
            KeyCode::Char('r') => {
                self.reset();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_lap < self.laps.len().saturating_sub(1) {
                    self.selected_lap += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_lap = self.selected_lap.saturating_sub(1);
            }
            KeyCode::Char('c') => {
                self.laps.clear();
                self.selected_lap = 0;
                self.status_message = Some("Laps cleared".to_string());
            }
            _ => {}
        }
        false
    }

    fn start(&mut self) {
        if !self.running {
            self.running = true;
            self.started_at = Some(Instant::now());
            self.status_message = Some("Started".to_string());
        }
    }

    fn pause(&mut self) {
        if self.running {
            self.running = false;
            self.paused_elapsed = self.elapsed_ms;
            self.started_at = None;
            self.status_message = Some("Paused".to_string());
        }
    }

    fn lap(&mut self) {
        if self.elapsed_ms > 0 {
            let last_lap_time = self.laps.last().map(|l| l.elapsed_ms).unwrap_or(0);
            let split = self.elapsed_ms - last_lap_time;

            self.laps.push(Lap {
                number: self.laps.len() + 1,
                elapsed_ms: self.elapsed_ms,
                split_ms: split,
            });

            self.status_message = Some(format!("Lap {} recorded", self.laps.len()));
        }
    }

    fn reset(&mut self) {
        self.running = false;
        self.started_at = None;
        self.elapsed_ms = 0;
        self.paused_elapsed = 0;
        self.laps.clear();
        self.selected_lap = 0;
        self.status_message = Some("Reset".to_string());
    }

    pub fn tick(&mut self) {
        if self.running {
            if let Some(started) = self.started_at {
                self.elapsed_ms = self.paused_elapsed + started.elapsed().as_millis() as u64;
            }
        }
    }

    pub fn format_time(&self) -> String {
        format_ms(self.elapsed_ms)
    }

    pub fn best_lap(&self) -> Option<&Lap> {
        self.laps.iter().min_by_key(|l| l.split_ms)
    }

    pub fn worst_lap(&self) -> Option<&Lap> {
        self.laps.iter().max_by_key(|l| l.split_ms)
    }

    pub fn average_lap(&self) -> Option<u64> {
        if self.laps.is_empty() {
            return None;
        }
        let total: u64 = self.laps.iter().map(|l| l.split_ms).sum();
        Some(total / self.laps.len() as u64)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        if self.running {
            "Space:pause l:lap r:reset q:quit".to_string()
        } else if self.elapsed_ms > 0 {
            "Space:resume l:lap r:reset c:clear-laps q:quit".to_string()
        } else {
            "Space:start q:quit".to_string()
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_ms(ms: u64) -> String {
    let hours = ms / 3600000;
    let minutes = (ms % 3600000) / 60000;
    let seconds = (ms % 60000) / 1000;
    let centis = (ms % 1000) / 10;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}.{:02}", hours, minutes, seconds, centis)
    } else {
        format!("{:02}:{:02}.{:02}", minutes, seconds, centis)
    }
}
