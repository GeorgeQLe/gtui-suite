use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct MountPoint {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
    pub options: Vec<String>,
    pub total_size: u64,
    pub used: u64,
    pub available: u64,
}

impl MountPoint {
    pub fn usage_percent(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.used as f64 / self.total_size as f64) * 100.0
        }
    }

    pub fn size_formatted(&self) -> String {
        format_size(self.total_size)
    }

    pub fn used_formatted(&self) -> String {
        format_size(self.used)
    }

    pub fn available_formatted(&self) -> String {
        format_size(self.available)
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1}T", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Detail,
}

pub struct App {
    pub mounts: Vec<MountPoint>,
    pub selected: usize,
    pub view: View,
    pub show_all: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            mounts: Vec::new(),
            selected: 0,
            view: View::List,
            show_all: false,
            status_message: None,
        }
    }

    pub fn load_mounts(&mut self) {
        self.mounts = create_demo_mounts();
        if !self.show_all {
            self.mounts
                .retain(|m| !m.fs_type.starts_with("tmp") && !m.mount_point.starts_with("/sys"));
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::List => self.handle_list_key(key),
            View::Detail => self.handle_detail_key(key),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.mounts.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.view = View::Detail;
            }
            KeyCode::Char('a') => {
                self.show_all = !self.show_all;
                self.load_mounts();
                self.selected = 0;
                self.status_message = Some(format!(
                    "Show all: {}",
                    if self.show_all { "on" } else { "off" }
                ));
            }
            KeyCode::Char('r') => {
                self.load_mounts();
                self.status_message = Some("Refreshed".to_string());
            }
            KeyCode::Char('u') => {
                if let Some(mount) = self.mounts.get(self.selected) {
                    self.status_message = Some(format!("Unmounting {}...", mount.mount_point));
                }
            }
            KeyCode::Char('m') => {
                self.status_message = Some("Mount dialog would open here".to_string());
            }
            _ => {}
        }
        false
    }

    fn handle_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::List;
            }
            _ => {}
        }
        false
    }

    pub fn selected_mount(&self) -> Option<&MountPoint> {
        self.mounts.get(self.selected)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::List => format!(
                "{} mounts | Enter:detail a:all r:refresh u:unmount m:mount",
                self.mounts.len()
            ),
            View::Detail => "Esc:back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_mounts() -> Vec<MountPoint> {
    vec![
        MountPoint {
            device: "/dev/sda1".to_string(),
            mount_point: "/".to_string(),
            fs_type: "ext4".to_string(),
            options: vec!["rw".to_string(), "relatime".to_string()],
            total_size: 500 * 1024 * 1024 * 1024,
            used: 120 * 1024 * 1024 * 1024,
            available: 380 * 1024 * 1024 * 1024,
        },
        MountPoint {
            device: "/dev/sda2".to_string(),
            mount_point: "/home".to_string(),
            fs_type: "ext4".to_string(),
            options: vec!["rw".to_string(), "relatime".to_string()],
            total_size: 1000 * 1024 * 1024 * 1024,
            used: 450 * 1024 * 1024 * 1024,
            available: 550 * 1024 * 1024 * 1024,
        },
        MountPoint {
            device: "/dev/sdb1".to_string(),
            mount_point: "/mnt/data".to_string(),
            fs_type: "xfs".to_string(),
            options: vec!["rw".to_string(), "noatime".to_string()],
            total_size: 2000 * 1024 * 1024 * 1024,
            used: 1200 * 1024 * 1024 * 1024,
            available: 800 * 1024 * 1024 * 1024,
        },
        MountPoint {
            device: "/dev/nvme0n1p1".to_string(),
            mount_point: "/boot/efi".to_string(),
            fs_type: "vfat".to_string(),
            options: vec!["rw".to_string(), "umask=0077".to_string()],
            total_size: 512 * 1024 * 1024,
            used: 45 * 1024 * 1024,
            available: 467 * 1024 * 1024,
        },
        MountPoint {
            device: "tmpfs".to_string(),
            mount_point: "/tmp".to_string(),
            fs_type: "tmpfs".to_string(),
            options: vec!["rw".to_string(), "nosuid".to_string(), "nodev".to_string()],
            total_size: 8 * 1024 * 1024 * 1024,
            used: 256 * 1024 * 1024,
            available: 8 * 1024 * 1024 * 1024 - 256 * 1024 * 1024,
        },
        MountPoint {
            device: "//server/share".to_string(),
            mount_point: "/mnt/network".to_string(),
            fs_type: "cifs".to_string(),
            options: vec!["rw".to_string(), "credentials=/etc/samba/creds".to_string()],
            total_size: 5000 * 1024 * 1024 * 1024,
            used: 2500 * 1024 * 1024 * 1024,
            available: 2500 * 1024 * 1024 * 1024,
        },
    ]
}
