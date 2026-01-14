use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub name: String,
    pub host: String,
    pub latency_ms: Option<f64>,
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub packet_loss: f64,
    pub history: Vec<f64>,
    pub is_testing: bool,
}

pub struct App {
    pub endpoints: Vec<Endpoint>,
    pub selected: usize,
    pub tick_count: u64,
    pub auto_test: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            endpoints: vec![
                Endpoint { name: "Google DNS".into(), host: "8.8.8.8".into(), latency_ms: Some(12.5), min_ms: 10.0, max_ms: 25.0, avg_ms: 14.2, packet_loss: 0.0, history: vec![12.0, 13.5, 12.5, 14.0, 12.5], is_testing: false },
                Endpoint { name: "Cloudflare DNS".into(), host: "1.1.1.1".into(), latency_ms: Some(8.2), min_ms: 6.0, max_ms: 15.0, avg_ms: 9.1, packet_loss: 0.0, history: vec![8.0, 9.2, 8.5, 8.2, 7.8], is_testing: false },
                Endpoint { name: "Local Gateway".into(), host: "192.168.1.1".into(), latency_ms: Some(0.8), min_ms: 0.5, max_ms: 2.0, avg_ms: 0.9, packet_loss: 0.0, history: vec![0.8, 0.9, 0.7, 0.8, 1.0], is_testing: false },
                Endpoint { name: "AWS us-east-1".into(), host: "ec2.us-east-1.amazonaws.com".into(), latency_ms: Some(45.2), min_ms: 40.0, max_ms: 80.0, avg_ms: 52.3, packet_loss: 0.5, history: vec![45.0, 48.0, 42.0, 55.0, 45.2], is_testing: false },
                Endpoint { name: "Unreachable Host".into(), host: "10.255.255.1".into(), latency_ms: None, min_ms: 0.0, max_ms: 0.0, avg_ms: 0.0, packet_loss: 100.0, history: vec![], is_testing: false },
            ],
            selected: 0,
            tick_count: 0,
            auto_test: true,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        if self.auto_test && self.tick_count % 4 == 0 {
            // Simulate latency updates
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as f64;

            for (i, ep) in self.endpoints.iter_mut().enumerate() {
                if ep.packet_loss < 100.0 {
                    let base = ep.avg_ms;
                    let jitter = ((secs / 1000.0 + i as f64) % 10.0 - 5.0) * 0.2;
                    let new_latency = (base + jitter).max(ep.min_ms);
                    ep.latency_ms = Some(new_latency);
                    ep.history.push(new_latency);
                    if ep.history.len() > 20 { ep.history.remove(0); }
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.endpoints.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('t') => {
                if let Some(ep) = self.endpoints.get_mut(self.selected) {
                    ep.is_testing = true;
                    // Would trigger actual ping here
                }
            },
            KeyCode::Char('a') => {
                self.auto_test = !self.auto_test;
            },
            KeyCode::Char('r') => {
                for ep in &mut self.endpoints {
                    ep.history.clear();
                }
            },
            KeyCode::Char('n') => {
                self.endpoints.push(Endpoint {
                    name: "New Endpoint".into(),
                    host: "example.com".into(),
                    latency_ms: None,
                    min_ms: 0.0, max_ms: 0.0, avg_ms: 0.0,
                    packet_loss: 0.0,
                    history: vec![],
                    is_testing: false,
                });
            },
            KeyCode::Char('d') => {
                if !self.endpoints.is_empty() {
                    self.endpoints.remove(self.selected);
                    self.selected = self.selected.min(self.endpoints.len().saturating_sub(1));
                }
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        format!("j/k:nav t:test a:auto({}) n:add d:delete r:reset q:quit",
            if self.auto_test { "on" } else { "off" })
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
