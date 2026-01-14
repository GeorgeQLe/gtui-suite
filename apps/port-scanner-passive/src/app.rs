use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::net::{IpAddr, Ipv4Addr};

use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Devices,
    DeviceDetails,
    Timeline,
    Stats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Ip,
    Hostname,
    LastSeen,
    FirstSeen,
    Type,
}

pub struct App {
    pub view: View,
    pub input_mode: InputMode,

    // Device list
    pub devices: Vec<Device>,
    pub selected_device: usize,
    pub sort_field: SortField,
    pub sort_ascending: bool,

    // Search/filter
    pub search_query: String,

    // Timeline
    pub events: Vec<NetworkEvent>,
    pub selected_event: usize,

    // Monitoring state
    pub monitoring: bool,
    pub stats: MonitoringStats,

    // Enabled protocols
    pub protocols: Vec<(DiscoveryMethod, bool)>,

    // Status
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::Devices,
            input_mode: InputMode::Normal,
            devices: Vec::new(),
            selected_device: 0,
            sort_field: SortField::LastSeen,
            sort_ascending: false,
            search_query: String::new(),
            events: Vec::new(),
            selected_event: 0,
            monitoring: false,
            stats: MonitoringStats::new(),
            protocols: vec![
                (DiscoveryMethod::Arp, true),
                (DiscoveryMethod::Mdns, true),
                (DiscoveryMethod::Ssdp, true),
                (DiscoveryMethod::NetBios, true),
                (DiscoveryMethod::Dhcp, true),
            ],
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        // Add demo devices
        if self.devices.is_empty() {
            self.devices = create_demo_devices();
            self.events = create_demo_events(&self.devices);
            self.stats.update(&self.devices);
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Global shortcuts
        if let KeyCode::Char('q') = key.code {
            if is_ctrl || (self.input_mode == InputMode::Normal && self.view == View::Devices) {
                return true;
            }
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Search => self.handle_search_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match self.view {
            View::Devices => self.handle_devices_key(key),
            View::DeviceDetails => self.handle_details_key(key),
            View::Timeline => self.handle_timeline_key(key),
            View::Stats => self.handle_stats_key(key),
        }
    }

    fn handle_devices_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let filtered = self.filtered_devices();
                if self.selected_device < filtered.len().saturating_sub(1) {
                    self.selected_device += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_device = self.selected_device.saturating_sub(1);
            }
            KeyCode::Enter => {
                if !self.filtered_devices().is_empty() {
                    self.view = View::DeviceDetails;
                }
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }
            KeyCode::Char('m') => {
                self.toggle_monitoring();
            }
            KeyCode::Char('s') => {
                self.cycle_sort();
            }
            KeyCode::Char('t') => {
                self.view = View::Timeline;
            }
            KeyCode::Char('i') => {
                self.view = View::Stats;
            }
            KeyCode::Char('x') => {
                self.export_devices();
            }
            KeyCode::Tab => {
                self.view = View::Timeline;
            }
            _ => {}
        }
        false
    }

    fn handle_details_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => {
                self.view = View::Devices;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                // Scroll services
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // Scroll services
            }
            _ => {}
        }
        false
    }

    fn handle_timeline_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_event < self.events.len().saturating_sub(1) {
                    self.selected_event += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_event = self.selected_event.saturating_sub(1);
            }
            KeyCode::Esc | KeyCode::Tab => {
                self.view = View::Devices;
            }
            KeyCode::Char('q') => {
                self.view = View::Devices;
            }
            _ => {}
        }
        false
    }

    fn handle_stats_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Tab | KeyCode::Char('q') => {
                self.view = View::Devices;
            }
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.selected_device = 0;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.selected_device = 0;
            }
            _ => {}
        }
        false
    }

    fn toggle_monitoring(&mut self) {
        self.monitoring = !self.monitoring;
        if self.monitoring {
            self.stats = MonitoringStats::new();
            self.stats.update(&self.devices);
            self.status_message = Some("Monitoring started".to_string());
        } else {
            self.status_message = Some("Monitoring stopped".to_string());
        }
    }

    fn cycle_sort(&mut self) {
        self.sort_field = match self.sort_field {
            SortField::Ip => SortField::Hostname,
            SortField::Hostname => SortField::LastSeen,
            SortField::LastSeen => SortField::FirstSeen,
            SortField::FirstSeen => SortField::Type,
            SortField::Type => SortField::Ip,
        };
        self.sort_devices();
        self.status_message = Some(format!("Sorted by {:?}", self.sort_field));
    }

    fn sort_devices(&mut self) {
        match self.sort_field {
            SortField::Ip => {
                self.devices.sort_by(|a, b| a.ip.to_string().cmp(&b.ip.to_string()));
            }
            SortField::Hostname => {
                self.devices.sort_by(|a, b| a.display_name().cmp(&b.display_name()));
            }
            SortField::LastSeen => {
                self.devices.sort_by(|a, b| b.last_seen.cmp(&a.last_seen));
            }
            SortField::FirstSeen => {
                self.devices.sort_by(|a, b| b.first_seen.cmp(&a.first_seen));
            }
            SortField::Type => {
                self.devices.sort_by(|a, b| {
                    a.device_type.map(|t| t.as_str()).cmp(&b.device_type.map(|t| t.as_str()))
                });
            }
        }
        if !self.sort_ascending {
            self.devices.reverse();
        }
    }

    fn export_devices(&mut self) {
        let filename = format!(
            "network_devices_{}.json",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        self.status_message = Some(format!("Exported {} devices to {}", self.devices.len(), filename));
    }

    pub fn filtered_devices(&self) -> Vec<&Device> {
        if self.search_query.is_empty() {
            self.devices.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.devices
                .iter()
                .filter(|d| {
                    d.ip.to_string().contains(&query)
                        || d.hostname.as_ref().map_or(false, |h| h.to_lowercase().contains(&query))
                        || d.mac.as_ref().map_or(false, |m| m.to_lowercase().contains(&query))
                        || d.vendor.as_ref().map_or(false, |v| v.to_lowercase().contains(&query))
                })
                .collect()
        }
    }

    pub fn selected_device_data(&self) -> Option<&Device> {
        self.filtered_devices().get(self.selected_device).copied()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Devices => {
                let status = if self.monitoring { "MONITORING" } else { "STOPPED" };
                format!(
                    "{} | {} devices ({} online) | m:monitor /:search s:sort Enter:details Tab:timeline",
                    status,
                    self.devices.len(),
                    self.devices.iter().filter(|d| d.is_online()).count()
                )
            }
            View::DeviceDetails => "Esc:back".to_string(),
            View::Timeline => format!(
                "{} events | j/k:navigate Tab:devices",
                self.events.len()
            ),
            View::Stats => "Esc:back".to_string(),
        }
    }
}

fn create_demo_devices() -> Vec<Device> {
    let mut devices = Vec::new();

    // Router
    let mut router = Device::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        DiscoveryMethod::Arp,
    );
    router.mac = Some("AA:BB:CC:11:22:33".to_string());
    router.vendor = Some("NETGEAR".to_string());
    router.hostname = Some("router.local".to_string());
    router.device_type = Some(DeviceType::Router);
    devices.push(router);

    // MacBook
    let mut macbook = Device::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
        DiscoveryMethod::Mdns,
    );
    macbook.mac = Some("B8:E8:56:AA:BB:CC".to_string());
    macbook.vendor = Some("Apple, Inc.".to_string());
    macbook.hostname = Some("Johns-MacBook-Pro.local".to_string());
    macbook.device_type = Some(DeviceType::Computer);
    macbook.services.push(Service::new("_ssh._tcp", "tcp"));
    macbook.services.push(Service::new("_sftp._tcp", "tcp"));
    macbook.services.push(Service::new("_airplay._tcp", "tcp"));
    devices.push(macbook);

    // Printer
    let mut printer = Device::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)),
        DiscoveryMethod::Mdns,
    );
    printer.mac = Some("00:11:22:33:44:55".to_string());
    printer.vendor = Some("HP Inc.".to_string());
    printer.hostname = Some("HP-LaserJet-Pro".to_string());
    printer.device_type = Some(DeviceType::Printer);
    printer.services.push(Service::new("_ipp._tcp", "tcp"));
    printer.services.push(Service::new("_printer._tcp", "tcp"));
    devices.push(printer);

    // Smart TV
    let mut tv = Device::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 30)),
        DiscoveryMethod::Ssdp,
    );
    tv.mac = Some("00:23:48:AA:BB:CC".to_string());
    tv.vendor = Some("Samsung Electronics".to_string());
    tv.hostname = Some("Samsung-TV".to_string());
    tv.device_type = Some(DeviceType::SmartTV);
    tv.services.push(Service::new("_airplay._tcp", "tcp"));
    devices.push(tv);

    // Phone
    let mut phone = Device::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 40)),
        DiscoveryMethod::Dhcp,
    );
    phone.mac = Some("DC:CF:FF:11:22:33".to_string());
    phone.vendor = Some("Huawei Technologies".to_string());
    phone.hostname = Some("android-abc123".to_string());
    phone.device_type = Some(DeviceType::Phone);
    devices.push(phone);

    // NAS
    let mut nas = Device::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)),
        DiscoveryMethod::Mdns,
    );
    nas.mac = Some("B0:EA:BC:11:22:33".to_string());
    nas.vendor = Some("QNAP Systems".to_string());
    nas.hostname = Some("qnap-nas.local".to_string());
    nas.device_type = Some(DeviceType::NetworkStorage);
    nas.services.push(Service::new("_smb._tcp", "tcp"));
    nas.services.push(Service::new("_afpovertcp._tcp", "tcp"));
    nas.services.push(Service::new("_http._tcp", "tcp"));
    devices.push(nas);

    // IoT Device
    let mut iot = Device::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 60)),
        DiscoveryMethod::Mdns,
    );
    iot.mac = Some("EC:8E:B5:11:22:33".to_string());
    iot.vendor = Some("Amazon Technologies".to_string());
    iot.hostname = Some("echo-dot-kitchen".to_string());
    iot.device_type = Some(DeviceType::IoT);
    iot.services.push(Service::new("_homekit._tcp", "tcp"));
    devices.push(iot);

    devices
}

fn create_demo_events(devices: &[Device]) -> Vec<NetworkEvent> {
    let mut events = Vec::new();

    for device in devices {
        let mut event = NetworkEvent::new(
            EventType::DeviceDiscovered,
            &format!("Device {} discovered via {}", device.display_name(), device.discovery_method.as_str()),
        );
        event = event.with_device(device.id);
        events.push(event);

        for service in &device.services {
            let mut event = NetworkEvent::new(
                EventType::ServiceAnnounced,
                &format!("Service {} on {}", service.name, device.display_name()),
            );
            event = event.with_device(device.id);
            events.push(event);
        }
    }

    events
}
