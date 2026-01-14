use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanType {
    SynScan,
    ConnectScan,
    AckScan,
    FinScan,
    XmasScan,
    NullScan,
    UdpScan,
}

impl ScanType {
    pub fn as_str(&self) -> &str {
        match self {
            ScanType::SynScan => "SYN Scan",
            ScanType::ConnectScan => "Connect Scan",
            ScanType::AckScan => "ACK Scan",
            ScanType::FinScan => "FIN Scan",
            ScanType::XmasScan => "Xmas Scan",
            ScanType::NullScan => "NULL Scan",
            ScanType::UdpScan => "UDP Scan",
        }
    }

    pub fn requires_root(&self) -> bool {
        match self {
            ScanType::SynScan | ScanType::AckScan | ScanType::FinScan
            | ScanType::XmasScan | ScanType::NullScan | ScanType::UdpScan => true,
            ScanType::ConnectScan => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanProfile {
    Quick,
    Standard,
    Comprehensive,
    Stealth,
}

impl ScanProfile {
    pub fn as_str(&self) -> &str {
        match self {
            ScanProfile::Quick => "Quick",
            ScanProfile::Standard => "Standard",
            ScanProfile::Comprehensive => "Comprehensive",
            ScanProfile::Stealth => "Stealth",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ScanProfile::Quick => "Top 100 ports, fast timing",
            ScanProfile::Standard => "Top 1000 ports, version detection",
            ScanProfile::Comprehensive => "All 65535 ports, full detection",
            ScanProfile::Stealth => "SYN scan, slow timing, randomized",
        }
    }

    pub fn port_count(&self) -> usize {
        match self {
            ScanProfile::Quick => 100,
            ScanProfile::Standard => 1000,
            ScanProfile::Comprehensive => 65535,
            ScanProfile::Stealth => 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimingTemplate {
    T0, // Paranoid
    T1, // Sneaky
    T2, // Polite
    T3, // Normal
    T4, // Aggressive
    T5, // Insane
}

impl TimingTemplate {
    pub fn as_str(&self) -> &str {
        match self {
            TimingTemplate::T0 => "Paranoid (T0)",
            TimingTemplate::T1 => "Sneaky (T1)",
            TimingTemplate::T2 => "Polite (T2)",
            TimingTemplate::T3 => "Normal (T3)",
            TimingTemplate::T4 => "Aggressive (T4)",
            TimingTemplate::T5 => "Insane (T5)",
        }
    }

    pub fn delay_ms(&self) -> u64 {
        match self {
            TimingTemplate::T0 => 300000,
            TimingTemplate::T1 => 15000,
            TimingTemplate::T2 => 400,
            TimingTemplate::T3 => 0,
            TimingTemplate::T4 => 0,
            TimingTemplate::T5 => 0,
        }
    }

    pub fn concurrent(&self) -> usize {
        match self {
            TimingTemplate::T0 => 1,
            TimingTemplate::T1 => 10,
            TimingTemplate::T2 => 100,
            TimingTemplate::T3 => 500,
            TimingTemplate::T4 => 1000,
            TimingTemplate::T5 => 2000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortState {
    Open,
    Closed,
    Filtered,
    OpenFiltered,
    ClosedFiltered,
}

impl PortState {
    pub fn as_str(&self) -> &str {
        match self {
            PortState::Open => "open",
            PortState::Closed => "closed",
            PortState::Filtered => "filtered",
            PortState::OpenFiltered => "open|filtered",
            PortState::ClosedFiltered => "closed|filtered",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            PortState::Open => "●",
            PortState::Closed => "○",
            PortState::Filtered => "◐",
            PortState::OpenFiltered => "◑",
            PortState::ClosedFiltered => "◒",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTarget {
    pub host: String,
    pub resolved_ip: Option<IpAddr>,
    pub ports: Vec<u16>,
}

impl ScanTarget {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_string(),
            resolved_ip: None,
            ports: Vec::new(),
        }
    }

    pub fn with_ports(mut self, start: u16, end: u16) -> Self {
        self.ports = (start..=end).collect();
        self
    }

    pub fn with_top_ports(mut self, count: usize) -> Self {
        self.ports = get_top_ports(count);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub id: Uuid,
    pub target: String,
    pub ip: Option<IpAddr>,
    pub scan_type: ScanType,
    pub profile: ScanProfile,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub ports: Vec<PortResult>,
    pub os_detection: Option<OsDetection>,
    pub hostname: Option<String>,
}

impl ScanResult {
    pub fn new(target: &str, scan_type: ScanType, profile: ScanProfile) -> Self {
        Self {
            id: Uuid::new_v4(),
            target: target.to_string(),
            ip: None,
            scan_type,
            profile,
            started_at: Utc::now(),
            completed_at: None,
            ports: Vec::new(),
            os_detection: None,
            hostname: None,
        }
    }

    pub fn open_ports(&self) -> Vec<&PortResult> {
        self.ports.iter().filter(|p| p.state == PortState::Open).collect()
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        self.completed_at.map(|c| c - self.started_at)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortResult {
    pub port: u16,
    pub protocol: Protocol,
    pub state: PortState,
    pub service: Option<ServiceInfo>,
    pub response_time_ms: Option<u64>,
}

impl PortResult {
    pub fn new(port: u16, protocol: Protocol, state: PortState) -> Self {
        Self {
            port,
            protocol,
            state,
            service: None,
            response_time_ms: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub fn as_str(&self) -> &str {
        match self {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub product: Option<String>,
    pub version: Option<String>,
    pub extra_info: Option<String>,
    pub confidence: u8,
}

impl ServiceInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            product: None,
            version: None,
            extra_info: None,
            confidence: 100,
        }
    }

    pub fn display(&self) -> String {
        let mut parts = vec![self.name.clone()];
        if let Some(ref product) = self.product {
            parts.push(product.clone());
        }
        if let Some(ref version) = self.version {
            parts.push(version.clone());
        }
        parts.join(" ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsDetection {
    pub name: String,
    pub family: String,
    pub version: Option<String>,
    pub accuracy: u8,
    pub cpe: Option<String>,
}

impl OsDetection {
    pub fn new(name: &str, family: &str, accuracy: u8) -> Self {
        Self {
            name: name.to_string(),
            family: family.to_string(),
            version: None,
            accuracy,
            cpe: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub total_ports: usize,
    pub scanned_ports: usize,
    pub open_found: usize,
    pub current_port: Option<u16>,
    pub packets_per_second: f64,
}

impl ScanProgress {
    pub fn new(total_ports: usize) -> Self {
        Self {
            total_ports,
            scanned_ports: 0,
            open_found: 0,
            current_port: None,
            packets_per_second: 0.0,
        }
    }

    pub fn percent_complete(&self) -> f64 {
        if self.total_ports == 0 {
            100.0
        } else {
            (self.scanned_ports as f64 / self.total_ports as f64) * 100.0
        }
    }
}

pub fn get_top_ports(count: usize) -> Vec<u16> {
    let top_ports: Vec<u16> = vec![
        21, 22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445, 993, 995,
        1723, 3306, 3389, 5432, 5900, 5901, 6379, 8080, 8443, 8888, 9000, 9090,
        27017, 27018, 28017, 1433, 1434, 1521, 2049, 2121, 3000, 4443, 5000,
        5001, 5432, 5500, 5601, 6000, 6001, 7001, 8000, 8001, 8008, 8081,
        8181, 8282, 8383, 8484, 8585, 8686, 8787, 8888, 9001, 9002, 9003,
        9200, 9300, 9999, 10000, 10001, 10002, 11211, 15672, 27015, 32768,
        49152, 49153, 49154, 49155, 49156, 49157, 50000, 50001, 50002,
        636, 389, 88, 464, 749, 1024, 1025, 1026, 1027, 1028, 1029, 1030,
        1080, 1099, 1194, 1241, 1311, 1352, 1433, 1434, 1512, 1524,
    ];

    top_ports.into_iter().take(count).collect()
}

pub fn get_common_service(port: u16) -> Option<&'static str> {
    match port {
        20 => Some("ftp-data"),
        21 => Some("ftp"),
        22 => Some("ssh"),
        23 => Some("telnet"),
        25 => Some("smtp"),
        53 => Some("dns"),
        80 => Some("http"),
        110 => Some("pop3"),
        111 => Some("rpcbind"),
        135 => Some("msrpc"),
        139 => Some("netbios-ssn"),
        143 => Some("imap"),
        443 => Some("https"),
        445 => Some("microsoft-ds"),
        993 => Some("imaps"),
        995 => Some("pop3s"),
        1433 => Some("mssql"),
        1521 => Some("oracle"),
        3306 => Some("mysql"),
        3389 => Some("rdp"),
        5432 => Some("postgresql"),
        5900 => Some("vnc"),
        6379 => Some("redis"),
        8080 => Some("http-proxy"),
        8443 => Some("https-alt"),
        27017 => Some("mongodb"),
        _ => None,
    }
}
