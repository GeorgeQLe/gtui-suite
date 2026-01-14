use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct QuotaEntry {
    pub user: String,
    pub filesystem: String,
    pub blocks_used: u64,
    pub blocks_soft: u64,
    pub blocks_hard: u64,
    pub inodes_used: u64,
    pub inodes_soft: u64,
    pub inodes_hard: u64,
    pub grace_time: Option<String>,
}

impl QuotaEntry {
    pub fn block_percent(&self) -> f64 {
        if self.blocks_hard > 0 {
            (self.blocks_used as f64 / self.blocks_hard as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn inode_percent(&self) -> f64 {
        if self.inodes_hard > 0 {
            (self.inodes_used as f64 / self.inodes_hard as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn is_over_soft(&self) -> bool {
        self.blocks_used > self.blocks_soft || self.inodes_used > self.inodes_soft
    }

    pub fn is_over_hard(&self) -> bool {
        self.blocks_used >= self.blocks_hard || self.inodes_used >= self.inodes_hard
    }
}

pub struct App {
    pub quotas: Vec<QuotaEntry>,
    pub selected: usize,
    pub show_groups: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            quotas: create_demo_quotas(),
            selected: 0,
            show_groups: false,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.quotas.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('g') => {
                self.show_groups = !self.show_groups;
                self.status_message = Some(if self.show_groups {
                    "Showing group quotas".to_string()
                } else {
                    "Showing user quotas".to_string()
                });
            }
            KeyCode::Char('e') => {
                if let Some(quota) = self.quotas.get(self.selected) {
                    self.status_message = Some(format!("Would edit quota for: {}", quota.user));
                }
            }
            KeyCode::Char('s') => {
                if let Some(quota) = self.quotas.get_mut(self.selected) {
                    quota.blocks_soft = quota.blocks_used + 1000000;
                    self.status_message = Some("Soft limit increased".to_string());
                }
            }
            KeyCode::Char('h') => {
                if let Some(quota) = self.quotas.get_mut(self.selected) {
                    quota.blocks_hard = quota.blocks_used + 2000000;
                    self.status_message = Some("Hard limit increased".to_string());
                }
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Refreshing quotas...".to_string());
            }
            KeyCode::Char('R') => {
                self.status_message = Some("Would generate quota report...".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn over_quota_count(&self) -> usize {
        self.quotas.iter().filter(|q| q.is_over_soft()).count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav g:groups e:edit s:soft h:hard r:refresh R:report q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_quotas() -> Vec<QuotaEntry> {
    vec![
        QuotaEntry {
            user: "alice".to_string(),
            filesystem: "/home".to_string(),
            blocks_used: 4500000,
            blocks_soft: 5000000,
            blocks_hard: 6000000,
            inodes_used: 25000,
            inodes_soft: 50000,
            inodes_hard: 60000,
            grace_time: None,
        },
        QuotaEntry {
            user: "bob".to_string(),
            filesystem: "/home".to_string(),
            blocks_used: 5200000,
            blocks_soft: 5000000,
            blocks_hard: 6000000,
            inodes_used: 45000,
            inodes_soft: 50000,
            inodes_hard: 60000,
            grace_time: Some("6 days".to_string()),
        },
        QuotaEntry {
            user: "charlie".to_string(),
            filesystem: "/home".to_string(),
            blocks_used: 2000000,
            blocks_soft: 5000000,
            blocks_hard: 6000000,
            inodes_used: 15000,
            inodes_soft: 50000,
            inodes_hard: 60000,
            grace_time: None,
        },
        QuotaEntry {
            user: "david".to_string(),
            filesystem: "/data".to_string(),
            blocks_used: 9500000,
            blocks_soft: 10000000,
            blocks_hard: 12000000,
            inodes_used: 80000,
            inodes_soft: 100000,
            inodes_hard: 120000,
            grace_time: None,
        },
        QuotaEntry {
            user: "eve".to_string(),
            filesystem: "/home".to_string(),
            blocks_used: 6000000,
            blocks_soft: 5000000,
            blocks_hard: 6000000,
            inodes_used: 55000,
            inodes_soft: 50000,
            inodes_hard: 60000,
            grace_time: Some("2 days".to_string()),
        },
    ]
}
