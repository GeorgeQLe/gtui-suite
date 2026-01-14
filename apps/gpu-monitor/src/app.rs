use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Gpu {
    pub name: String,
    pub vendor: String,
    pub utilization: u8,
    pub memory_used: u64,
    pub memory_total: u64,
    pub temperature: f32,
    pub power_draw: f32,
    pub clock_speed: u32,
    pub memory_clock: u32,
}

pub struct App {
    pub gpus: Vec<Gpu>,
    pub selected: usize,
    pub utilization_history: Vec<u8>,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            gpus: vec![
                Gpu {
                    name: "NVIDIA GeForce RTX 3080".into(),
                    vendor: "NVIDIA".into(),
                    utilization: 35,
                    memory_used: 4096,
                    memory_total: 10240,
                    temperature: 52.0,
                    power_draw: 180.0,
                    clock_speed: 1710,
                    memory_clock: 9501,
                },
                Gpu {
                    name: "Intel UHD Graphics 630".into(),
                    vendor: "Intel".into(),
                    utilization: 5,
                    memory_used: 256,
                    memory_total: 1024,
                    temperature: 45.0,
                    power_draw: 15.0,
                    clock_speed: 1150,
                    memory_clock: 0,
                },
            ],
            selected: 0,
            utilization_history: vec![30, 32, 35, 33, 35, 40, 38, 35, 33, 35],
            tick_count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        // Simulate GPU activity
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        if let Some(gpu) = self.gpus.get_mut(0) {
            gpu.utilization = (30 + ((secs % 30) as u8 * 2)).min(100);
            gpu.temperature = 50.0 + (gpu.utilization as f32 * 0.3);
            gpu.power_draw = 100.0 + (gpu.utilization as f32 * 2.0);

            self.utilization_history.push(gpu.utilization);
            if self.utilization_history.len() > 30 { self.utilization_history.remove(0); }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                self.selected = (self.selected + 1) % self.gpus.len().max(1);
            },
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.checked_sub(1).unwrap_or(self.gpus.len().saturating_sub(1));
            },
            KeyCode::Char('r') => {
                self.utilization_history.clear();
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        "j/k:select r:reset-history q:quit".into()
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
