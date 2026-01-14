use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    Symlink,
    Hardlink,
}

impl LinkType {
    pub fn name(&self) -> &'static str {
        match self {
            LinkType::Symlink => "Symlink",
            LinkType::Hardlink => "Hardlink",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStatus {
    Valid,
    Broken,
    Circular,
}

impl LinkStatus {
    pub fn name(&self) -> &'static str {
        match self {
            LinkStatus::Valid => "Valid",
            LinkStatus::Broken => "Broken",
            LinkStatus::Circular => "Circular",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Link {
    pub path: String,
    pub target: String,
    pub link_type: LinkType,
    pub status: LinkStatus,
    pub inode: u64,
    pub link_count: u32,
}

pub struct App {
    pub links: Vec<Link>,
    pub selected: usize,
    pub filter_broken: bool,
    pub show_hardlinks: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            links: create_demo_links(),
            selected: 0,
            filter_broken: false,
            show_hardlinks: true,
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
                let filtered = self.filtered_links();
                if self.selected < filtered.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('b') => {
                self.filter_broken = !self.filter_broken;
                self.selected = 0;
                self.status_message = Some(if self.filter_broken {
                    "Showing broken links only".to_string()
                } else {
                    "Showing all links".to_string()
                });
            }
            KeyCode::Char('h') => {
                self.show_hardlinks = !self.show_hardlinks;
                self.selected = 0;
                self.status_message = Some(if self.show_hardlinks {
                    "Showing hardlinks".to_string()
                } else {
                    "Hiding hardlinks".to_string()
                });
            }
            KeyCode::Char('c') => {
                self.status_message = Some("Would create new link...".to_string());
            }
            KeyCode::Char('d') => {
                let filtered = self.filtered_indices();
                if let Some(&idx) = filtered.get(self.selected) {
                    let link = &self.links[idx];
                    self.status_message = Some(format!("Would delete: {}", link.path));
                }
            }
            KeyCode::Char('e') => {
                let filtered = self.filtered_indices();
                if let Some(&idx) = filtered.get(self.selected) {
                    let link = &self.links[idx];
                    self.status_message = Some(format!("Would edit target: {} -> {}", link.path, link.target));
                }
            }
            KeyCode::Char('f') => {
                self.status_message = Some("Would fix broken links...".to_string());
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Refreshing...".to_string());
            }
            _ => {}
        }
        false
    }

    fn filtered_indices(&self) -> Vec<usize> {
        self.links
            .iter()
            .enumerate()
            .filter(|(_, l)| {
                let type_match = self.show_hardlinks || l.link_type == LinkType::Symlink;
                let broken_match = !self.filter_broken || l.status == LinkStatus::Broken;
                type_match && broken_match
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn filtered_links(&self) -> Vec<&Link> {
        self.links
            .iter()
            .filter(|l| {
                let type_match = self.show_hardlinks || l.link_type == LinkType::Symlink;
                let broken_match = !self.filter_broken || l.status == LinkStatus::Broken;
                type_match && broken_match
            })
            .collect()
    }

    pub fn broken_count(&self) -> usize {
        self.links.iter().filter(|l| l.status == LinkStatus::Broken).count()
    }

    pub fn symlink_count(&self) -> usize {
        self.links.iter().filter(|l| l.link_type == LinkType::Symlink).count()
    }

    pub fn hardlink_count(&self) -> usize {
        self.links.iter().filter(|l| l.link_type == LinkType::Hardlink).count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav b:broken h:hardlinks c:create d:delete e:edit f:fix r:refresh q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_links() -> Vec<Link> {
    vec![
        Link {
            path: "/usr/bin/python".to_string(),
            target: "/usr/bin/python3".to_string(),
            link_type: LinkType::Symlink,
            status: LinkStatus::Valid,
            inode: 12345,
            link_count: 1,
        },
        Link {
            path: "/home/user/project".to_string(),
            target: "/mnt/data/projects/myproject".to_string(),
            link_type: LinkType::Symlink,
            status: LinkStatus::Valid,
            inode: 23456,
            link_count: 1,
        },
        Link {
            path: "/tmp/old_link".to_string(),
            target: "/nonexistent/path".to_string(),
            link_type: LinkType::Symlink,
            status: LinkStatus::Broken,
            inode: 34567,
            link_count: 1,
        },
        Link {
            path: "/etc/alternatives/editor".to_string(),
            target: "/usr/bin/vim".to_string(),
            link_type: LinkType::Symlink,
            status: LinkStatus::Valid,
            inode: 45678,
            link_count: 1,
        },
        Link {
            path: "/home/user/backup.tar".to_string(),
            target: "/home/user/backup.tar".to_string(),
            link_type: LinkType::Hardlink,
            status: LinkStatus::Valid,
            inode: 56789,
            link_count: 3,
        },
        Link {
            path: "/var/log/syslog".to_string(),
            target: "/removed/old/syslog".to_string(),
            link_type: LinkType::Symlink,
            status: LinkStatus::Broken,
            inode: 67890,
            link_count: 1,
        },
        Link {
            path: "/home/user/docs/readme.md".to_string(),
            target: "/home/user/docs/readme.md".to_string(),
            link_type: LinkType::Hardlink,
            status: LinkStatus::Valid,
            inode: 78901,
            link_count: 2,
        },
    ]
}
