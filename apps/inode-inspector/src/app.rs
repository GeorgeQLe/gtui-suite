use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    BlockDevice,
    CharDevice,
    Socket,
    Fifo,
}

impl FileType {
    pub fn name(&self) -> &'static str {
        match self {
            FileType::Regular => "Regular",
            FileType::Directory => "Directory",
            FileType::Symlink => "Symlink",
            FileType::BlockDevice => "Block Dev",
            FileType::CharDevice => "Char Dev",
            FileType::Socket => "Socket",
            FileType::Fifo => "FIFO",
        }
    }

    pub fn code(&self) -> char {
        match self {
            FileType::Regular => '-',
            FileType::Directory => 'd',
            FileType::Symlink => 'l',
            FileType::BlockDevice => 'b',
            FileType::CharDevice => 'c',
            FileType::Socket => 's',
            FileType::Fifo => 'p',
        }
    }
}

#[derive(Debug, Clone)]
pub struct InodeInfo {
    pub inode: u64,
    pub file_type: FileType,
    pub path: String,
    pub size: u64,
    pub blocks: u64,
    pub block_size: u32,
    pub links: u32,
    pub uid: u32,
    pub gid: u32,
    pub mode: String,
    pub atime: String,
    pub mtime: String,
    pub ctime: String,
    pub device: String,
}

pub struct App {
    pub inodes: Vec<InodeInfo>,
    pub selected: usize,
    pub show_details: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            inodes: create_demo_inodes(),
            selected: 0,
            show_details: false,
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
                if self.selected < self.inodes.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.show_details = !self.show_details;
            }
            KeyCode::Char('l') => {
                if let Some(inode) = self.inodes.get(self.selected) {
                    self.status_message = Some(format!("Hard links: {}", inode.links));
                }
            }
            KeyCode::Char('t') => {
                if let Some(inode) = self.inodes.get(self.selected) {
                    self.status_message = Some(format!("Type: {}", inode.file_type.name()));
                }
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Refreshing...".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn total_size(&self) -> u64 {
        self.inodes.iter().map(|i| i.size).sum()
    }

    pub fn total_blocks(&self) -> u64 {
        self.inodes.iter().map(|i| i.blocks).sum()
    }

    pub fn current_inode(&self) -> Option<&InodeInfo> {
        self.inodes.get(self.selected)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav Enter:details l:links t:type r:refresh q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_inodes() -> Vec<InodeInfo> {
    vec![
        InodeInfo {
            inode: 12345678,
            file_type: FileType::Regular,
            path: "/home/user/document.txt".to_string(),
            size: 4096,
            blocks: 8,
            block_size: 4096,
            links: 1,
            uid: 1000,
            gid: 1000,
            mode: "-rw-r--r--".to_string(),
            atime: "2024-01-15 14:30:00".to_string(),
            mtime: "2024-01-14 10:00:00".to_string(),
            ctime: "2024-01-14 10:00:00".to_string(),
            device: "/dev/sda1".to_string(),
        },
        InodeInfo {
            inode: 23456789,
            file_type: FileType::Directory,
            path: "/home/user/projects".to_string(),
            size: 4096,
            blocks: 8,
            block_size: 4096,
            links: 5,
            uid: 1000,
            gid: 1000,
            mode: "drwxr-xr-x".to_string(),
            atime: "2024-01-15 15:00:00".to_string(),
            mtime: "2024-01-15 12:00:00".to_string(),
            ctime: "2024-01-15 12:00:00".to_string(),
            device: "/dev/sda1".to_string(),
        },
        InodeInfo {
            inode: 34567890,
            file_type: FileType::Symlink,
            path: "/home/user/link -> /tmp/target".to_string(),
            size: 11,
            blocks: 0,
            block_size: 4096,
            links: 1,
            uid: 1000,
            gid: 1000,
            mode: "lrwxrwxrwx".to_string(),
            atime: "2024-01-15 14:00:00".to_string(),
            mtime: "2024-01-10 08:00:00".to_string(),
            ctime: "2024-01-10 08:00:00".to_string(),
            device: "/dev/sda1".to_string(),
        },
        InodeInfo {
            inode: 45678901,
            file_type: FileType::BlockDevice,
            path: "/dev/sda".to_string(),
            size: 0,
            blocks: 0,
            block_size: 4096,
            links: 1,
            uid: 0,
            gid: 6,
            mode: "brw-rw----".to_string(),
            atime: "2024-01-15 08:00:00".to_string(),
            mtime: "2024-01-01 00:00:00".to_string(),
            ctime: "2024-01-01 00:00:00".to_string(),
            device: "devtmpfs".to_string(),
        },
        InodeInfo {
            inode: 56789012,
            file_type: FileType::Socket,
            path: "/var/run/docker.sock".to_string(),
            size: 0,
            blocks: 0,
            block_size: 4096,
            links: 1,
            uid: 0,
            gid: 999,
            mode: "srw-rw----".to_string(),
            atime: "2024-01-15 08:00:00".to_string(),
            mtime: "2024-01-15 08:00:00".to_string(),
            ctime: "2024-01-15 08:00:00".to_string(),
            device: "tmpfs".to_string(),
        },
    ]
}
