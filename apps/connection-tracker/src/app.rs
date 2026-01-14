use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol { Tcp, Udp, Tcp6, Udp6 }

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState { Established, Listen, TimeWait, CloseWait, SynSent, SynRecv, FinWait1, FinWait2, Closing, LastAck, Closed }

#[derive(Debug, Clone)]
pub struct Connection {
    pub protocol: Protocol,
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub state: ConnectionState,
    pub pid: Option<u32>,
    pub process: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterState { All, Established, Listen, TimeWait }

pub struct App {
    pub connections: Vec<Connection>,
    pub selected: usize,
    pub filter: FilterState,
    pub show_ipv6: bool,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            connections: vec![
                Connection { protocol: Protocol::Tcp, local_addr: "0.0.0.0".into(), local_port: 22, remote_addr: "0.0.0.0".into(), remote_port: 0, state: ConnectionState::Listen, pid: Some(1234), process: Some("sshd".into()) },
                Connection { protocol: Protocol::Tcp, local_addr: "192.168.1.100".into(), local_port: 22, remote_addr: "192.168.1.50".into(), remote_port: 52341, state: ConnectionState::Established, pid: Some(5678), process: Some("sshd".into()) },
                Connection { protocol: Protocol::Tcp, local_addr: "0.0.0.0".into(), local_port: 80, remote_addr: "0.0.0.0".into(), remote_port: 0, state: ConnectionState::Listen, pid: Some(2345), process: Some("nginx".into()) },
                Connection { protocol: Protocol::Tcp, local_addr: "192.168.1.100".into(), local_port: 80, remote_addr: "10.0.0.5".into(), remote_port: 45123, state: ConnectionState::Established, pid: Some(2346), process: Some("nginx".into()) },
                Connection { protocol: Protocol::Tcp, local_addr: "192.168.1.100".into(), local_port: 80, remote_addr: "10.0.0.6".into(), remote_port: 45124, state: ConnectionState::TimeWait, pid: None, process: None },
                Connection { protocol: Protocol::Tcp, local_addr: "0.0.0.0".into(), local_port: 443, remote_addr: "0.0.0.0".into(), remote_port: 0, state: ConnectionState::Listen, pid: Some(2345), process: Some("nginx".into()) },
                Connection { protocol: Protocol::Tcp6, local_addr: "::".into(), local_port: 22, remote_addr: "::".into(), remote_port: 0, state: ConnectionState::Listen, pid: Some(1234), process: Some("sshd".into()) },
                Connection { protocol: Protocol::Udp, local_addr: "0.0.0.0".into(), local_port: 53, remote_addr: "0.0.0.0".into(), remote_port: 0, state: ConnectionState::Listen, pid: Some(3456), process: Some("dnsmasq".into()) },
            ],
            selected: 0,
            filter: FilterState::All,
            show_ipv6: true,
            tick_count: 0,
        }
    }

    pub fn filtered_connections(&self) -> Vec<&Connection> {
        self.connections.iter()
            .filter(|c| {
                if !self.show_ipv6 && matches!(c.protocol, Protocol::Tcp6 | Protocol::Udp6) {
                    return false;
                }
                match self.filter {
                    FilterState::All => true,
                    FilterState::Established => c.state == ConnectionState::Established,
                    FilterState::Listen => c.state == ConnectionState::Listen,
                    FilterState::TimeWait => c.state == ConnectionState::TimeWait,
                }
            })
            .collect()
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let filtered = self.filtered_connections();
                if self.selected < filtered.len().saturating_sub(1) { self.selected += 1; }
            },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('a') => { self.filter = FilterState::All; self.selected = 0; },
            KeyCode::Char('e') => { self.filter = FilterState::Established; self.selected = 0; },
            KeyCode::Char('l') => { self.filter = FilterState::Listen; self.selected = 0; },
            KeyCode::Char('t') => { self.filter = FilterState::TimeWait; self.selected = 0; },
            KeyCode::Char('6') => { self.show_ipv6 = !self.show_ipv6; self.selected = 0; },
            KeyCode::Char('r') => { /* refresh */ },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        let filter_str = match self.filter {
            FilterState::All => "ALL",
            FilterState::Established => "ESTAB",
            FilterState::Listen => "LISTEN",
            FilterState::TimeWait => "TIME_WAIT",
        };
        format!("j/k:nav a:all e:estab l:listen t:timewait 6:ipv6({}) r:refresh q:quit [{}]",
            if self.show_ipv6 { "on" } else { "off" }, filter_str)
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
