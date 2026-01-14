use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    Arp,
    Mdns,
    Ssdp,
    NetBios,
    Dhcp,
    Dns,
    Manual,
}

impl DiscoveryMethod {
    pub fn as_str(&self) -> &str {
        match self {
            DiscoveryMethod::Arp => "ARP",
            DiscoveryMethod::Mdns => "mDNS",
            DiscoveryMethod::Ssdp => "SSDP",
            DiscoveryMethod::NetBios => "NetBIOS",
            DiscoveryMethod::Dhcp => "DHCP",
            DiscoveryMethod::Dns => "DNS",
            DiscoveryMethod::Manual => "Manual",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            DiscoveryMethod::Arp => "📡",
            DiscoveryMethod::Mdns => "🍎",
            DiscoveryMethod::Ssdp => "🔌",
            DiscoveryMethod::NetBios => "🪟",
            DiscoveryMethod::Dhcp => "📋",
            DiscoveryMethod::Dns => "🌐",
            DiscoveryMethod::Manual => "✏️",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            DiscoveryMethod::Arp => "Address Resolution Protocol",
            DiscoveryMethod::Mdns => "Multicast DNS (Bonjour)",
            DiscoveryMethod::Ssdp => "Simple Service Discovery Protocol (UPnP)",
            DiscoveryMethod::NetBios => "Windows Network Browser",
            DiscoveryMethod::Dhcp => "DHCP Lease Monitor",
            DiscoveryMethod::Dns => "DNS Query Monitor",
            DiscoveryMethod::Manual => "Manually Added",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: Uuid,
    pub ip: IpAddr,
    pub mac: Option<String>,
    pub vendor: Option<String>,
    pub hostname: Option<String>,
    pub device_type: Option<DeviceType>,
    pub discovery_method: DiscoveryMethod,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub services: Vec<Service>,
    pub metadata: HashMap<String, String>,
}

impl Device {
    pub fn new(ip: IpAddr, method: DiscoveryMethod) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            ip,
            mac: None,
            vendor: None,
            hostname: None,
            device_type: None,
            discovery_method: method,
            first_seen: now,
            last_seen: now,
            services: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn display_name(&self) -> String {
        self.hostname
            .clone()
            .unwrap_or_else(|| self.ip.to_string())
    }

    pub fn is_online(&self) -> bool {
        let duration = Utc::now() - self.last_seen;
        duration.num_minutes() < 5
    }

    pub fn uptime(&self) -> chrono::Duration {
        self.last_seen - self.first_seen
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Router,
    Computer,
    Phone,
    Tablet,
    Printer,
    SmartTV,
    IoT,
    Server,
    NetworkStorage,
    Camera,
    Unknown,
}

impl DeviceType {
    pub fn as_str(self) -> &'static str {
        match self {
            DeviceType::Router => "Router",
            DeviceType::Computer => "Computer",
            DeviceType::Phone => "Phone",
            DeviceType::Tablet => "Tablet",
            DeviceType::Printer => "Printer",
            DeviceType::SmartTV => "Smart TV",
            DeviceType::IoT => "IoT Device",
            DeviceType::Server => "Server",
            DeviceType::NetworkStorage => "NAS",
            DeviceType::Camera => "Camera",
            DeviceType::Unknown => "Unknown",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            DeviceType::Router => "🌐",
            DeviceType::Computer => "💻",
            DeviceType::Phone => "📱",
            DeviceType::Tablet => "📲",
            DeviceType::Printer => "🖨️",
            DeviceType::SmartTV => "📺",
            DeviceType::IoT => "🏠",
            DeviceType::Server => "🖥️",
            DeviceType::NetworkStorage => "💾",
            DeviceType::Camera => "📷",
            DeviceType::Unknown => "❓",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub protocol: String,
    pub port: Option<u16>,
    pub txt_records: HashMap<String, String>,
    pub discovered_at: DateTime<Utc>,
}

impl Service {
    pub fn new(name: &str, protocol: &str) -> Self {
        Self {
            name: name.to_string(),
            protocol: protocol.to_string(),
            port: None,
            txt_records: HashMap::new(),
            discovered_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub device_id: Option<Uuid>,
    pub description: String,
}

impl NetworkEvent {
    pub fn new(event_type: EventType, description: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            device_id: None,
            description: description.to_string(),
        }
    }

    pub fn with_device(mut self, device_id: Uuid) -> Self {
        self.device_id = Some(device_id);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    DeviceDiscovered,
    DeviceDisappeared,
    ServiceAnnounced,
    ServiceRemoved,
    IpChanged,
    HostnameChanged,
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            EventType::DeviceDiscovered => "Device Discovered",
            EventType::DeviceDisappeared => "Device Disappeared",
            EventType::ServiceAnnounced => "Service Announced",
            EventType::ServiceRemoved => "Service Removed",
            EventType::IpChanged => "IP Changed",
            EventType::HostnameChanged => "Hostname Changed",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            EventType::DeviceDiscovered => "➕",
            EventType::DeviceDisappeared => "➖",
            EventType::ServiceAnnounced => "📢",
            EventType::ServiceRemoved => "🔇",
            EventType::IpChanged => "🔄",
            EventType::HostnameChanged => "✏️",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonitoringStats {
    pub total_devices: usize,
    pub online_devices: usize,
    pub total_services: usize,
    pub packets_captured: u64,
    pub monitoring_since: DateTime<Utc>,
}

impl MonitoringStats {
    pub fn new() -> Self {
        Self {
            total_devices: 0,
            online_devices: 0,
            total_services: 0,
            packets_captured: 0,
            monitoring_since: Utc::now(),
        }
    }

    pub fn update(&mut self, devices: &[Device]) {
        self.total_devices = devices.len();
        self.online_devices = devices.iter().filter(|d| d.is_online()).count();
        self.total_services = devices.iter().map(|d| d.services.len()).sum();
    }

    pub fn duration(&self) -> chrono::Duration {
        Utc::now() - self.monitoring_since
    }
}

impl Default for MonitoringStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorInfo {
    pub prefix: String,
    pub vendor: String,
    pub country: Option<String>,
}

pub fn lookup_vendor(mac: &str) -> Option<&'static str> {
    let prefix = mac.replace([':', '-'], "").to_uppercase();
    if prefix.len() < 6 {
        return None;
    }
    let oui = &prefix[..6];

    match oui {
        "AABBCC" => Some("Apple, Inc."),
        "001A2B" => Some("Cisco Systems"),
        "00155D" => Some("Microsoft Corporation"),
        "001122" => Some("Intel Corporation"),
        "002348" => Some("Samsung Electronics"),
        "B8E856" => Some("Apple, Inc."),
        "DCCFFF" => Some("Huawei Technologies"),
        "F4F5D8" => Some("Google, Inc."),
        "EC8EB5" => Some("Amazon Technologies"),
        "B0EABC" => Some("QNAP Systems"),
        _ => None,
    }
}

pub fn infer_device_type(services: &[Service], vendor: Option<&str>) -> DeviceType {
    // Check services first
    for service in services {
        match service.name.as_str() {
            "_printer._tcp" | "_ipp._tcp" | "_pdl-datastream._tcp" => return DeviceType::Printer,
            "_airplay._tcp" | "_raop._tcp" => return DeviceType::SmartTV,
            "_homekit._tcp" | "_hap._tcp" => return DeviceType::IoT,
            "_smb._tcp" | "_afpovertcp._tcp" => return DeviceType::NetworkStorage,
            "_ssh._tcp" | "_sftp._tcp" => return DeviceType::Server,
            "_http._tcp" if service.txt_records.get("model").map_or(false, |m| m.contains("Camera")) => {
                return DeviceType::Camera;
            }
            _ => {}
        }
    }

    // Check vendor
    if let Some(v) = vendor {
        let v_lower = v.to_lowercase();
        if v_lower.contains("apple") {
            return DeviceType::Computer;
        } else if v_lower.contains("samsung") || v_lower.contains("huawei") {
            return DeviceType::Phone;
        } else if v_lower.contains("cisco") || v_lower.contains("netgear") || v_lower.contains("tp-link") {
            return DeviceType::Router;
        } else if v_lower.contains("raspberry") {
            return DeviceType::IoT;
        }
    }

    DeviceType::Unknown
}
