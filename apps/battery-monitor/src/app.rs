use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Battery {
    pub name: String,
    pub status: String,
    pub percentage: u8,
    pub voltage: f32,
    pub power_draw: f32,
    pub time_remaining: Option<String>,
}

pub struct App {
    pub batteries: Vec<Battery>,
    pub selected: usize,
    pub history: Vec<u8>,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            batteries: vec![
                Battery {
                    name: "BAT0".into(),
                    status: "Discharging".into(),
                    percentage: 78,
                    voltage: 11.4,
                    power_draw: 12.5,
                    time_remaining: Some("2h 45m".into()),
                },
                Battery {
                    name: "BAT1".into(),
                    status: "Charging".into(),
                    percentage: 45,
                    voltage: 12.1,
                    power_draw: -25.0,
                    time_remaining: Some("1h 15m to full".into()),
                },
            ],
            selected: 0,
            history: vec![75, 76, 77, 78, 78, 77, 76, 78, 79, 78],
            tick_count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        // Simulate battery drain/charge
        if let Some(bat) = self.batteries.get_mut(0) {
            if self.tick_count % 10 == 0 && bat.percentage > 5 {
                bat.percentage = bat.percentage.saturating_sub(1);
                self.history.push(bat.percentage);
                if self.history.len() > 20 { self.history.remove(0); }
            }
        }
        if let Some(bat) = self.batteries.get_mut(1) {
            if self.tick_count % 8 == 0 && bat.percentage < 100 {
                bat.percentage += 1;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                self.selected = (self.selected + 1) % self.batteries.len().max(1);
            },
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.checked_sub(1).unwrap_or(self.batteries.len().saturating_sub(1));
            },
            KeyCode::Char('r') => {
                // Refresh
                self.history.clear();
                if let Some(bat) = self.batteries.get(0) {
                    self.history.push(bat.percentage);
                }
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        "j/k:select r:refresh q:quit".into()
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
