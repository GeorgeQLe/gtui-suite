use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum FanMode { Auto, Manual }

#[derive(Debug, Clone)]
pub struct Fan {
    pub name: String,
    pub rpm: u32,
    pub max_rpm: u32,
    pub pwm: u8,
    pub mode: FanMode,
}

pub struct App {
    pub fans: Vec<Fan>,
    pub selected: usize,
    pub cpu_temp: f32,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            fans: vec![
                Fan { name: "CPU Fan".into(), rpm: 1200, max_rpm: 3000, pwm: 40, mode: FanMode::Auto },
                Fan { name: "Case Fan 1".into(), rpm: 800, max_rpm: 2000, pwm: 30, mode: FanMode::Auto },
                Fan { name: "Case Fan 2".into(), rpm: 750, max_rpm: 2000, pwm: 28, mode: FanMode::Auto },
                Fan { name: "GPU Fan".into(), rpm: 0, max_rpm: 4000, pwm: 0, mode: FanMode::Auto },
            ],
            selected: 0,
            cpu_temp: 45.0,
            status_message: None,
        }
    }

    pub fn tick(&mut self) {
        // Simulate temperature and fan speed changes
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.cpu_temp = 45.0 + ((secs % 20) as f32 * 0.5);

        for fan in &mut self.fans {
            if fan.mode == FanMode::Auto {
                let target_pwm = ((self.cpu_temp - 40.0) * 3.0).clamp(0.0, 100.0) as u8;
                fan.pwm = target_pwm;
                fan.rpm = (fan.max_rpm as f32 * (fan.pwm as f32 / 100.0)) as u32;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.fans.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('m') => {
                if let Some(fan) = self.fans.get_mut(self.selected) {
                    fan.mode = match fan.mode {
                        FanMode::Auto => FanMode::Manual,
                        FanMode::Manual => FanMode::Auto,
                    };
                    self.status_message = Some(format!("{}: {:?} mode", fan.name, fan.mode));
                }
            },
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if let Some(fan) = self.fans.get_mut(self.selected) {
                    if fan.mode == FanMode::Manual {
                        fan.pwm = (fan.pwm + 5).min(100);
                        fan.rpm = (fan.max_rpm as f32 * (fan.pwm as f32 / 100.0)) as u32;
                    }
                }
            },
            KeyCode::Char('-') => {
                if let Some(fan) = self.fans.get_mut(self.selected) {
                    if fan.mode == FanMode::Manual {
                        fan.pwm = fan.pwm.saturating_sub(5);
                        fan.rpm = (fan.max_rpm as f32 * (fan.pwm as f32 / 100.0)) as u32;
                    }
                }
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| "j/k:nav m:toggle-mode +/-:speed(manual) q:quit".into())
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
