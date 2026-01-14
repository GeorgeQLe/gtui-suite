use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Hop {
    pub number: u8,
    pub host: Option<String>,
    pub ip: Option<String>,
    pub loss_percent: f64,
    pub sent: u32,
    pub received: u32,
    pub last_ms: Option<f64>,
    pub avg_ms: f64,
    pub best_ms: f64,
    pub worst_ms: f64,
    pub stdev_ms: f64,
}

pub struct App {
    pub target: String,
    pub hops: Vec<Hop>,
    pub selected: usize,
    pub is_running: bool,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            target: "google.com".into(),
            hops: vec![
                Hop { number: 1, host: Some("router.local".into()), ip: Some("192.168.1.1".into()), loss_percent: 0.0, sent: 100, received: 100, last_ms: Some(0.8), avg_ms: 0.9, best_ms: 0.5, worst_ms: 2.1, stdev_ms: 0.3 },
                Hop { number: 2, host: Some("isp-gw1.example.net".into()), ip: Some("10.0.0.1".into()), loss_percent: 0.0, sent: 100, received: 100, last_ms: Some(8.5), avg_ms: 9.2, best_ms: 7.0, worst_ms: 15.0, stdev_ms: 1.5 },
                Hop { number: 3, host: None, ip: Some("72.14.215.85".into()), loss_percent: 5.0, sent: 100, received: 95, last_ms: Some(12.3), avg_ms: 14.5, best_ms: 10.0, worst_ms: 35.0, stdev_ms: 4.2 },
                Hop { number: 4, host: Some("core-rtr1.google.com".into()), ip: Some("142.250.169.174".into()), loss_percent: 0.0, sent: 100, received: 100, last_ms: Some(11.2), avg_ms: 12.0, best_ms: 10.5, worst_ms: 18.0, stdev_ms: 1.8 },
                Hop { number: 5, host: None, ip: None, loss_percent: 100.0, sent: 100, received: 0, last_ms: None, avg_ms: 0.0, best_ms: 0.0, worst_ms: 0.0, stdev_ms: 0.0 },
                Hop { number: 6, host: Some("google.com".into()), ip: Some("142.250.190.78".into()), loss_percent: 0.0, sent: 100, received: 100, last_ms: Some(12.8), avg_ms: 13.5, best_ms: 11.0, worst_ms: 20.0, stdev_ms: 2.0 },
            ],
            selected: 0,
            is_running: true,
            tick_count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        if self.is_running && self.tick_count % 2 == 0 {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as f64;

            for hop in &mut self.hops {
                if hop.loss_percent < 100.0 {
                    let jitter = ((secs / 1000.0 + hop.number as f64) % 5.0 - 2.5) * 0.5;
                    let new_latency = (hop.avg_ms + jitter).max(hop.best_ms);
                    hop.last_ms = Some(new_latency);
                    hop.sent += 1;
                    if (secs as u64 + hop.number as u64) % 20 != 0 || hop.loss_percent == 0.0 {
                        hop.received += 1;
                    }
                    hop.loss_percent = (1.0 - hop.received as f64 / hop.sent as f64) * 100.0;
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.hops.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char(' ') | KeyCode::Char('p') => self.is_running = !self.is_running,
            KeyCode::Char('r') => {
                for hop in &mut self.hops {
                    hop.sent = 0;
                    hop.received = 0;
                    hop.loss_percent = 0.0;
                }
            },
            KeyCode::Char('n') => {
                // Would prompt for new target
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        format!("j/k:nav p:pause({}) r:reset n:new-target q:quit",
            if self.is_running { "running" } else { "paused" })
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
