use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use chrono::{DateTime, Local};

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol { Tcp, Udp, Icmp, Arp, Dns, Http, Https, Unknown }

#[derive(Debug, Clone)]
pub struct Packet {
    pub id: u64,
    pub timestamp: DateTime<Local>,
    pub protocol: Protocol,
    pub src_ip: String,
    pub src_port: Option<u16>,
    pub dst_ip: String,
    pub dst_port: Option<u16>,
    pub length: usize,
    pub info: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterProtocol { All, Tcp, Udp, Icmp, Http, Dns }

pub struct App {
    pub packets: Vec<Packet>,
    pub selected: usize,
    pub is_capturing: bool,
    pub filter: FilterProtocol,
    pub tick_count: u64,
    pub packet_counter: u64,
    pub scroll_offset: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            packets: vec![
                Packet { id: 1, timestamp: Local::now(), protocol: Protocol::Tcp, src_ip: "192.168.1.100".into(), src_port: Some(52341), dst_ip: "142.250.190.78".into(), dst_port: Some(443), length: 66, info: "SYN".into() },
                Packet { id: 2, timestamp: Local::now(), protocol: Protocol::Tcp, src_ip: "142.250.190.78".into(), src_port: Some(443), dst_ip: "192.168.1.100".into(), dst_port: Some(52341), length: 66, info: "SYN-ACK".into() },
                Packet { id: 3, timestamp: Local::now(), protocol: Protocol::Tcp, src_ip: "192.168.1.100".into(), src_port: Some(52341), dst_ip: "142.250.190.78".into(), dst_port: Some(443), length: 54, info: "ACK".into() },
                Packet { id: 4, timestamp: Local::now(), protocol: Protocol::Https, src_ip: "192.168.1.100".into(), src_port: Some(52341), dst_ip: "142.250.190.78".into(), dst_port: Some(443), length: 517, info: "TLS Client Hello".into() },
                Packet { id: 5, timestamp: Local::now(), protocol: Protocol::Dns, src_ip: "192.168.1.100".into(), src_port: Some(53421), dst_ip: "8.8.8.8".into(), dst_port: Some(53), length: 72, info: "A google.com".into() },
                Packet { id: 6, timestamp: Local::now(), protocol: Protocol::Dns, src_ip: "8.8.8.8".into(), src_port: Some(53), dst_ip: "192.168.1.100".into(), dst_port: Some(53421), length: 88, info: "A 142.250.190.78".into() },
                Packet { id: 7, timestamp: Local::now(), protocol: Protocol::Icmp, src_ip: "192.168.1.100".into(), src_port: None, dst_ip: "8.8.8.8".into(), dst_port: None, length: 84, info: "Echo request".into() },
                Packet { id: 8, timestamp: Local::now(), protocol: Protocol::Icmp, src_ip: "8.8.8.8".into(), src_port: None, dst_ip: "192.168.1.100".into(), dst_port: None, length: 84, info: "Echo reply".into() },
            ],
            selected: 0,
            is_capturing: false,
            filter: FilterProtocol::All,
            tick_count: 0,
            packet_counter: 8,
            scroll_offset: 0,
        }
    }

    pub fn filtered_packets(&self) -> Vec<&Packet> {
        self.packets.iter()
            .filter(|p| match self.filter {
                FilterProtocol::All => true,
                FilterProtocol::Tcp => matches!(p.protocol, Protocol::Tcp | Protocol::Https | Protocol::Http),
                FilterProtocol::Udp => p.protocol == Protocol::Udp,
                FilterProtocol::Icmp => p.protocol == Protocol::Icmp,
                FilterProtocol::Http => matches!(p.protocol, Protocol::Http | Protocol::Https),
                FilterProtocol::Dns => p.protocol == Protocol::Dns,
            })
            .collect()
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        if self.is_capturing && self.tick_count % 3 == 0 {
            self.packet_counter += 1;
            let protocols = [Protocol::Tcp, Protocol::Udp, Protocol::Dns, Protocol::Https, Protocol::Icmp];
            let proto = protocols[self.packet_counter as usize % protocols.len()].clone();
            let info = match proto {
                Protocol::Tcp => "ACK",
                Protocol::Udp => "UDP Data",
                Protocol::Dns => "A example.com",
                Protocol::Https => "TLS Data",
                Protocol::Icmp => "Echo request",
                _ => "Data",
            };
            self.packets.push(Packet {
                id: self.packet_counter,
                timestamp: Local::now(),
                protocol: proto,
                src_ip: "192.168.1.100".into(),
                src_port: Some(50000 + (self.packet_counter % 1000) as u16),
                dst_ip: "10.0.0.1".into(),
                dst_port: Some(80),
                length: 64 + (self.packet_counter % 500) as usize,
                info: info.into(),
            });
            if self.packets.len() > 1000 {
                self.packets.remove(0);
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let filtered = self.filtered_packets();
                if self.selected < filtered.len().saturating_sub(1) { self.selected += 1; }
            },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char(' ') => self.is_capturing = !self.is_capturing,
            KeyCode::Char('c') => { self.packets.clear(); self.selected = 0; },
            KeyCode::Char('a') => { self.filter = FilterProtocol::All; self.selected = 0; },
            KeyCode::Char('t') => { self.filter = FilterProtocol::Tcp; self.selected = 0; },
            KeyCode::Char('u') => { self.filter = FilterProtocol::Udp; self.selected = 0; },
            KeyCode::Char('i') => { self.filter = FilterProtocol::Icmp; self.selected = 0; },
            KeyCode::Char('h') => { self.filter = FilterProtocol::Http; self.selected = 0; },
            KeyCode::Char('d') => { self.filter = FilterProtocol::Dns; self.selected = 0; },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        let filter_str = match self.filter {
            FilterProtocol::All => "ALL",
            FilterProtocol::Tcp => "TCP",
            FilterProtocol::Udp => "UDP",
            FilterProtocol::Icmp => "ICMP",
            FilterProtocol::Http => "HTTP",
            FilterProtocol::Dns => "DNS",
        };
        format!("j/k:nav space:capture({}) c:clear a/t/u/i/h/d:filter[{}] q:quit",
            if self.is_capturing { "on" } else { "off" }, filter_str)
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
