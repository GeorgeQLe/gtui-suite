use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Timezone {
    pub region: String,
    pub city: String,
    pub offset: String,
}

pub struct App {
    pub timezones: Vec<Timezone>,
    pub selected: usize,
    pub current_tz: String,
    pub filter: String,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            timezones: vec![
                Timezone { region: "America".into(), city: "New_York".into(), offset: "UTC-5".into() },
                Timezone { region: "America".into(), city: "Los_Angeles".into(), offset: "UTC-8".into() },
                Timezone { region: "America".into(), city: "Chicago".into(), offset: "UTC-6".into() },
                Timezone { region: "Europe".into(), city: "London".into(), offset: "UTC+0".into() },
                Timezone { region: "Europe".into(), city: "Paris".into(), offset: "UTC+1".into() },
                Timezone { region: "Europe".into(), city: "Berlin".into(), offset: "UTC+1".into() },
                Timezone { region: "Asia".into(), city: "Tokyo".into(), offset: "UTC+9".into() },
                Timezone { region: "Asia".into(), city: "Shanghai".into(), offset: "UTC+8".into() },
                Timezone { region: "Asia".into(), city: "Singapore".into(), offset: "UTC+8".into() },
                Timezone { region: "Australia".into(), city: "Sydney".into(), offset: "UTC+11".into() },
                Timezone { region: "Pacific".into(), city: "Auckland".into(), offset: "UTC+13".into() },
            ],
            selected: 0,
            current_tz: "America/New_York".into(),
            filter: String::new(),
            status_message: None,
        }
    }

    pub fn filtered_timezones(&self) -> Vec<(usize, &Timezone)> {
        self.timezones.iter().enumerate()
            .filter(|(_, tz)| {
                if self.filter.is_empty() { return true; }
                let full = format!("{}/{}", tz.region, tz.city).to_lowercase();
                full.contains(&self.filter.to_lowercase())
            })
            .collect()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let filtered = self.filtered_timezones();
                if self.selected < filtered.len().saturating_sub(1) { self.selected += 1; }
            },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Enter => {
                let filtered = self.filtered_timezones();
                if let Some((_, tz)) = filtered.get(self.selected) {
                    self.current_tz = format!("{}/{}", tz.region, tz.city);
                    self.status_message = Some(format!("Set timezone: {}", self.current_tz));
                }
            },
            KeyCode::Char('/') => self.status_message = Some("Filter mode (type to filter)...".into()),
            KeyCode::Char('c') => { self.filter.clear(); self.selected = 0; },
            KeyCode::Backspace => { self.filter.pop(); self.selected = 0; },
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.filter.push(c);
                self.selected = 0;
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        if !self.filter.is_empty() {
            format!("Filter: {} | j/k:nav enter:select c:clear q:quit", self.filter)
        } else {
            self.status_message.clone().unwrap_or_else(|| "j/k:nav enter:select /:filter q:quit".into())
        }
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
