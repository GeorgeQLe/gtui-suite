use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum SocketType { Tcp, Udp, Unix, Raw }

#[derive(Debug, Clone, PartialEq)]
pub enum SocketState { Listen, Established, TimeWait, CloseWait, SynSent, FinWait, Closing, Unknown }

#[derive(Debug, Clone)]
pub struct Socket {
    pub socket_type: SocketType,
    pub state: SocketState,
    pub recv_q: u64,
    pub send_q: u64,
    pub local_addr: String,
    pub peer_addr: String,
    pub pid: Option<u32>,
    pub process: Option<String>,
    pub inode: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterType { All, Tcp, Udp, Unix, Listening }

pub struct App {
    pub sockets: Vec<Socket>,
    pub selected: usize,
    pub filter: FilterType,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            sockets: vec![
                Socket { socket_type: SocketType::Tcp, state: SocketState::Listen, recv_q: 0, send_q: 0, local_addr: "0.0.0.0:22".into(), peer_addr: "*:*".into(), pid: Some(1234), process: Some("sshd".into()), inode: 12345 },
                Socket { socket_type: SocketType::Tcp, state: SocketState::Established, recv_q: 0, send_q: 128, local_addr: "192.168.1.100:22".into(), peer_addr: "192.168.1.50:52341".into(), pid: Some(5678), process: Some("sshd".into()), inode: 12346 },
                Socket { socket_type: SocketType::Tcp, state: SocketState::Listen, recv_q: 0, send_q: 0, local_addr: "0.0.0.0:80".into(), peer_addr: "*:*".into(), pid: Some(2345), process: Some("nginx".into()), inode: 12347 },
                Socket { socket_type: SocketType::Tcp, state: SocketState::TimeWait, recv_q: 0, send_q: 0, local_addr: "192.168.1.100:80".into(), peer_addr: "10.0.0.5:45123".into(), pid: None, process: None, inode: 0 },
                Socket { socket_type: SocketType::Udp, state: SocketState::Unknown, recv_q: 0, send_q: 0, local_addr: "0.0.0.0:53".into(), peer_addr: "*:*".into(), pid: Some(3456), process: Some("dnsmasq".into()), inode: 12348 },
                Socket { socket_type: SocketType::Udp, state: SocketState::Unknown, recv_q: 256, send_q: 0, local_addr: "0.0.0.0:68".into(), peer_addr: "*:*".into(), pid: Some(4567), process: Some("dhclient".into()), inode: 12349 },
                Socket { socket_type: SocketType::Unix, state: SocketState::Listen, recv_q: 0, send_q: 0, local_addr: "/var/run/dbus/system_bus_socket".into(), peer_addr: "".into(), pid: Some(789), process: Some("dbus-daemon".into()), inode: 12350 },
                Socket { socket_type: SocketType::Unix, state: SocketState::Established, recv_q: 0, send_q: 0, local_addr: "/run/systemd/journal/stdout".into(), peer_addr: "".into(), pid: Some(1), process: Some("systemd".into()), inode: 12351 },
            ],
            selected: 0,
            filter: FilterType::All,
            tick_count: 0,
        }
    }

    pub fn filtered_sockets(&self) -> Vec<&Socket> {
        self.sockets.iter()
            .filter(|s| match self.filter {
                FilterType::All => true,
                FilterType::Tcp => s.socket_type == SocketType::Tcp,
                FilterType::Udp => s.socket_type == SocketType::Udp,
                FilterType::Unix => s.socket_type == SocketType::Unix,
                FilterType::Listening => s.state == SocketState::Listen,
            })
            .collect()
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        // Simulate queue changes
        if self.tick_count % 4 == 0 {
            for socket in &mut self.sockets {
                if socket.state == SocketState::Established {
                    socket.send_q = (socket.send_q + 64) % 512;
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let filtered = self.filtered_sockets();
                if self.selected < filtered.len().saturating_sub(1) { self.selected += 1; }
            },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('a') => { self.filter = FilterType::All; self.selected = 0; },
            KeyCode::Char('t') => { self.filter = FilterType::Tcp; self.selected = 0; },
            KeyCode::Char('u') => { self.filter = FilterType::Udp; self.selected = 0; },
            KeyCode::Char('x') => { self.filter = FilterType::Unix; self.selected = 0; },
            KeyCode::Char('l') => { self.filter = FilterType::Listening; self.selected = 0; },
            KeyCode::Char('r') => { /* refresh */ },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        let filter_str = match self.filter {
            FilterType::All => "ALL",
            FilterType::Tcp => "TCP",
            FilterType::Udp => "UDP",
            FilterType::Unix => "UNIX",
            FilterType::Listening => "LISTEN",
        };
        format!("j/k:nav a:all t:tcp u:udp x:unix l:listen r:refresh q:quit [{}]", filter_str)
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
