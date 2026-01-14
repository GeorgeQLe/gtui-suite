use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Timer {
    pub name: String,
    pub duration_secs: u64,
    pub remaining_secs: u64,
    pub running: bool,
    pub started_at: Option<Instant>,
    pub paused_remaining: Option<u64>,
}

impl Timer {
    pub fn new(name: &str, duration_secs: u64) -> Self {
        Self {
            name: name.to_string(),
            duration_secs,
            remaining_secs: duration_secs,
            running: false,
            started_at: None,
            paused_remaining: None,
        }
    }

    pub fn start(&mut self) {
        if !self.running && self.remaining_secs > 0 {
            self.running = true;
            self.started_at = Some(Instant::now());
            if self.paused_remaining.is_some() {
                self.remaining_secs = self.paused_remaining.take().unwrap();
            }
        }
    }

    pub fn pause(&mut self) {
        if self.running {
            self.running = false;
            self.paused_remaining = Some(self.remaining_secs);
            self.started_at = None;
        }
    }

    pub fn reset(&mut self) {
        self.running = false;
        self.remaining_secs = self.duration_secs;
        self.started_at = None;
        self.paused_remaining = None;
    }

    pub fn tick(&mut self) {
        if self.running {
            if let Some(started) = self.started_at {
                let elapsed = started.elapsed().as_secs();
                let base = self.paused_remaining.unwrap_or(self.duration_secs);
                self.remaining_secs = base.saturating_sub(elapsed);

                if self.remaining_secs == 0 {
                    self.running = false;
                    self.started_at = None;
                }
            }
        }
    }

    pub fn format_time(&self) -> String {
        let hours = self.remaining_secs / 3600;
        let minutes = (self.remaining_secs % 3600) / 60;
        let seconds = self.remaining_secs % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }

    pub fn progress(&self) -> f64 {
        if self.duration_secs == 0 {
            return 0.0;
        }
        (self.duration_secs - self.remaining_secs) as f64 / self.duration_secs as f64
    }
}

pub struct App {
    pub timers: Vec<Timer>,
    pub selected: usize,
    pub adding: bool,
    pub add_input: String,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            timers: vec![
                Timer::new("Pomodoro", 25 * 60),
                Timer::new("Short Break", 5 * 60),
                Timer::new("Long Break", 15 * 60),
                Timer::new("1 Minute", 60),
            ],
            selected: 0,
            adding: false,
            add_input: String::new(),
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        if self.adding {
            return self.handle_add_key(key);
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.timers.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if let Some(timer) = self.timers.get_mut(self.selected) {
                    if timer.running {
                        timer.pause();
                        self.status_message = Some(format!("Paused: {}", timer.name));
                    } else {
                        timer.start();
                        self.status_message = Some(format!("Started: {}", timer.name));
                    }
                }
            }
            KeyCode::Char('r') => {
                if let Some(timer) = self.timers.get_mut(self.selected) {
                    timer.reset();
                    self.status_message = Some(format!("Reset: {}", timer.name));
                }
            }
            KeyCode::Char('a') => {
                self.adding = true;
                self.add_input.clear();
            }
            KeyCode::Char('d') => {
                if self.timers.len() > 1 {
                    self.timers.remove(self.selected);
                    self.selected = self.selected.min(self.timers.len().saturating_sub(1));
                    self.status_message = Some("Timer removed".to_string());
                }
            }
            _ => {}
        }
        false
    }

    fn handle_add_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.adding = false;
                self.add_input.clear();
            }
            KeyCode::Enter => {
                if let Some((name, secs)) = self.parse_timer_input() {
                    self.timers.push(Timer::new(&name, secs));
                    self.status_message = Some(format!("Added: {}", name));
                }
                self.adding = false;
                self.add_input.clear();
            }
            KeyCode::Backspace => {
                self.add_input.pop();
            }
            KeyCode::Char(c) => {
                self.add_input.push(c);
            }
            _ => {}
        }
        false
    }

    fn parse_timer_input(&self) -> Option<(String, u64)> {
        // Format: "name:MM" or "name:HH:MM:SS"
        let parts: Vec<&str> = self.add_input.split(':').collect();
        if parts.len() < 2 {
            return None;
        }

        let name = parts[0].trim();
        if name.is_empty() {
            return None;
        }

        let secs = if parts.len() == 2 {
            parts[1].parse::<u64>().ok()? * 60
        } else if parts.len() == 3 {
            let mins: u64 = parts[1].parse().ok()?;
            let secs: u64 = parts[2].parse().ok()?;
            mins * 60 + secs
        } else {
            let hours: u64 = parts[1].parse().ok()?;
            let mins: u64 = parts[2].parse().ok()?;
            let secs: u64 = parts[3].parse().ok()?;
            hours * 3600 + mins * 60 + secs
        };

        Some((name.to_string(), secs))
    }

    pub fn tick(&mut self) {
        for timer in &mut self.timers {
            timer.tick();
        }
    }

    pub fn active_count(&self) -> usize {
        self.timers.iter().filter(|t| t.running).count()
    }

    pub fn status_text(&self) -> String {
        if self.adding {
            return format!("Add timer (name:minutes): {}_ | Esc:cancel Enter:add", self.add_input);
        }
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "Space:start/pause r:reset a:add d:delete q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
