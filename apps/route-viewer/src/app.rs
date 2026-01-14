use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum RouteType { Local, Gateway, Host, Default }

#[derive(Debug, Clone)]
pub struct Route {
    pub destination: String,
    pub gateway: String,
    pub netmask: String,
    pub flags: String,
    pub metric: u32,
    pub interface: String,
    pub route_type: RouteType,
}

pub struct App {
    pub routes: Vec<Route>,
    pub selected: usize,
    pub show_ipv6: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            routes: vec![
                Route { destination: "default".into(), gateway: "192.168.1.1".into(), netmask: "0.0.0.0".into(), flags: "UG".into(), metric: 100, interface: "eth0".into(), route_type: RouteType::Default },
                Route { destination: "192.168.1.0".into(), gateway: "0.0.0.0".into(), netmask: "255.255.255.0".into(), flags: "U".into(), metric: 100, interface: "eth0".into(), route_type: RouteType::Local },
                Route { destination: "192.168.1.1".into(), gateway: "0.0.0.0".into(), netmask: "255.255.255.255".into(), flags: "UH".into(), metric: 100, interface: "eth0".into(), route_type: RouteType::Host },
                Route { destination: "10.0.0.0".into(), gateway: "192.168.1.254".into(), netmask: "255.0.0.0".into(), flags: "UG".into(), metric: 200, interface: "eth0".into(), route_type: RouteType::Gateway },
                Route { destination: "172.16.0.0".into(), gateway: "192.168.1.253".into(), netmask: "255.240.0.0".into(), flags: "UG".into(), metric: 200, interface: "eth0".into(), route_type: RouteType::Gateway },
                Route { destination: "127.0.0.0".into(), gateway: "0.0.0.0".into(), netmask: "255.0.0.0".into(), flags: "U".into(), metric: 0, interface: "lo".into(), route_type: RouteType::Local },
                Route { destination: "169.254.0.0".into(), gateway: "0.0.0.0".into(), netmask: "255.255.0.0".into(), flags: "U".into(), metric: 1000, interface: "eth0".into(), route_type: RouteType::Local },
            ],
            selected: 0,
            show_ipv6: false,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.routes.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('6') => self.show_ipv6 = !self.show_ipv6,
            KeyCode::Char('a') => self.status_message = Some("Would add route...".into()),
            KeyCode::Char('d') => self.status_message = Some("Would delete route...".into()),
            KeyCode::Char('r') => self.status_message = Some("Refreshed routing table".into()),
            KeyCode::Char('f') => self.status_message = Some("Would flush routes...".into()),
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| format!("j/k:nav 6:ipv6({}) a:add d:delete r:refresh f:flush q:quit",
            if self.show_ipv6 { "on" } else { "off" }))
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
