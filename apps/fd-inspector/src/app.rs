use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdType {
    Regular,
    Socket,
    Pipe,
    Directory,
    Device,
    Unknown,
}

impl FdType {
    pub fn name(&self) -> &'static str {
        match self {
            FdType::Regular => "REG",
            FdType::Socket => "SOCK",
            FdType::Pipe => "PIPE",
            FdType::Directory => "DIR",
            FdType::Device => "DEV",
            FdType::Unknown => "???",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FdType::Regular => "📄",
            FdType::Socket => "🔌",
            FdType::Pipe => "🔗",
            FdType::Directory => "📁",
            FdType::Device => "💾",
            FdType::Unknown => "❓",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub fd: i32,
    pub fd_type: FdType,
    pub path: String,
    pub mode: String,
    pub size: Option<u64>,
    pub offset: u64,
    pub flags: String,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub fd_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    All,
    Regular,
    Socket,
    Pipe,
    Directory,
}

pub struct App {
    pub processes: Vec<ProcessInfo>,
    pub descriptors: Vec<FileDescriptor>,
    pub selected_process: usize,
    pub selected_fd: usize,
    pub filter: FilterType,
    pub show_details: bool,
    pub tick_count: u64,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let processes = create_demo_processes();
        let descriptors = create_demo_fds();

        Self {
            processes,
            descriptors,
            selected_process: 0,
            selected_fd: 0,
            filter: FilterType::All,
            show_details: false,
            tick_count: 0,
            status_message: None,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.show_details {
                    if self.selected_fd < self.filtered_fds().len().saturating_sub(1) {
                        self.selected_fd += 1;
                    }
                } else if self.selected_process < self.processes.len().saturating_sub(1) {
                    self.selected_process += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.show_details {
                    self.selected_fd = self.selected_fd.saturating_sub(1);
                } else {
                    self.selected_process = self.selected_process.saturating_sub(1);
                }
            }
            KeyCode::Enter => {
                self.show_details = true;
                self.selected_fd = 0;
                self.status_message = Some("Viewing file descriptors".to_string());
            }
            KeyCode::Backspace => {
                self.show_details = false;
                self.status_message = Some("Back to process list".to_string());
            }
            KeyCode::Char('1') => {
                self.filter = FilterType::All;
                self.selected_fd = 0;
                self.status_message = Some("Filter: All".to_string());
            }
            KeyCode::Char('2') => {
                self.filter = FilterType::Regular;
                self.selected_fd = 0;
                self.status_message = Some("Filter: Regular files".to_string());
            }
            KeyCode::Char('3') => {
                self.filter = FilterType::Socket;
                self.selected_fd = 0;
                self.status_message = Some("Filter: Sockets".to_string());
            }
            KeyCode::Char('4') => {
                self.filter = FilterType::Pipe;
                self.selected_fd = 0;
                self.status_message = Some("Filter: Pipes".to_string());
            }
            KeyCode::Char('5') => {
                self.filter = FilterType::Directory;
                self.selected_fd = 0;
                self.status_message = Some("Filter: Directories".to_string());
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Refreshing...".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn filtered_fds(&self) -> Vec<&FileDescriptor> {
        self.descriptors
            .iter()
            .filter(|fd| match self.filter {
                FilterType::All => true,
                FilterType::Regular => fd.fd_type == FdType::Regular,
                FilterType::Socket => fd.fd_type == FdType::Socket,
                FilterType::Pipe => fd.fd_type == FdType::Pipe,
                FilterType::Directory => fd.fd_type == FdType::Directory,
            })
            .collect()
    }

    pub fn total_fd_count(&self) -> usize {
        self.processes.iter().map(|p| p.fd_count).sum()
    }

    pub fn socket_count(&self) -> usize {
        self.descriptors.iter().filter(|fd| fd.fd_type == FdType::Socket).count()
    }

    pub fn current_process(&self) -> Option<&ProcessInfo> {
        self.processes.get(self.selected_process)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        if self.show_details {
            "j/k:nav 1-5:filter Backspace:back r:refresh q:quit".to_string()
        } else {
            "j/k:nav Enter:view r:refresh q:quit".to_string()
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_processes() -> Vec<ProcessInfo> {
    vec![
        ProcessInfo {
            pid: 1234,
            name: "node".to_string(),
            user: "user".to_string(),
            fd_count: 45,
        },
        ProcessInfo {
            pid: 5678,
            name: "postgres".to_string(),
            user: "postgres".to_string(),
            fd_count: 128,
        },
        ProcessInfo {
            pid: 9012,
            name: "nginx".to_string(),
            user: "www-data".to_string(),
            fd_count: 32,
        },
        ProcessInfo {
            pid: 3456,
            name: "redis-server".to_string(),
            user: "redis".to_string(),
            fd_count: 18,
        },
        ProcessInfo {
            pid: 7890,
            name: "docker".to_string(),
            user: "root".to_string(),
            fd_count: 64,
        },
        ProcessInfo {
            pid: 2345,
            name: "python".to_string(),
            user: "user".to_string(),
            fd_count: 22,
        },
    ]
}

fn create_demo_fds() -> Vec<FileDescriptor> {
    vec![
        FileDescriptor {
            fd: 0,
            fd_type: FdType::Device,
            path: "/dev/null".to_string(),
            mode: "r".to_string(),
            size: None,
            offset: 0,
            flags: "O_RDONLY".to_string(),
        },
        FileDescriptor {
            fd: 1,
            fd_type: FdType::Pipe,
            path: "pipe:[12345]".to_string(),
            mode: "w".to_string(),
            size: None,
            offset: 0,
            flags: "O_WRONLY".to_string(),
        },
        FileDescriptor {
            fd: 2,
            fd_type: FdType::Pipe,
            path: "pipe:[12346]".to_string(),
            mode: "w".to_string(),
            size: None,
            offset: 0,
            flags: "O_WRONLY".to_string(),
        },
        FileDescriptor {
            fd: 3,
            fd_type: FdType::Socket,
            path: "socket:[23456]".to_string(),
            mode: "rw".to_string(),
            size: None,
            offset: 0,
            flags: "O_RDWR|O_NONBLOCK".to_string(),
        },
        FileDescriptor {
            fd: 4,
            fd_type: FdType::Regular,
            path: "/var/log/app.log".to_string(),
            mode: "a".to_string(),
            size: Some(1024000),
            offset: 1024000,
            flags: "O_WRONLY|O_APPEND".to_string(),
        },
        FileDescriptor {
            fd: 5,
            fd_type: FdType::Regular,
            path: "/etc/config.json".to_string(),
            mode: "r".to_string(),
            size: Some(2048),
            offset: 0,
            flags: "O_RDONLY".to_string(),
        },
        FileDescriptor {
            fd: 6,
            fd_type: FdType::Socket,
            path: "TCP *:8080 (LISTEN)".to_string(),
            mode: "rw".to_string(),
            size: None,
            offset: 0,
            flags: "O_RDWR|O_NONBLOCK".to_string(),
        },
        FileDescriptor {
            fd: 7,
            fd_type: FdType::Socket,
            path: "TCP 192.168.1.1:8080->10.0.0.5:54321".to_string(),
            mode: "rw".to_string(),
            size: None,
            offset: 0,
            flags: "O_RDWR|O_NONBLOCK".to_string(),
        },
        FileDescriptor {
            fd: 8,
            fd_type: FdType::Directory,
            path: "/home/user/project".to_string(),
            mode: "r".to_string(),
            size: None,
            offset: 0,
            flags: "O_RDONLY|O_DIRECTORY".to_string(),
        },
        FileDescriptor {
            fd: 9,
            fd_type: FdType::Regular,
            path: "/tmp/cache.db".to_string(),
            mode: "rw".to_string(),
            size: Some(4096000),
            offset: 2048000,
            flags: "O_RDWR".to_string(),
        },
    ]
}
