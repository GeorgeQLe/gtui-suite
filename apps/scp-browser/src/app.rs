use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum PaneType { Local, Remote }

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, Clone)]
pub struct Transfer {
    pub filename: String,
    pub direction: String,
    pub progress: u8,
    pub size: u64,
    pub speed: String,
}

pub struct App {
    pub active_pane: PaneType,
    pub local_path: String,
    pub remote_path: String,
    pub local_files: Vec<FileEntry>,
    pub remote_files: Vec<FileEntry>,
    pub local_selected: usize,
    pub remote_selected: usize,
    pub transfers: Vec<Transfer>,
    pub connected_host: String,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            active_pane: PaneType::Local,
            local_path: "/home/user".into(),
            remote_path: "/var/www".into(),
            local_files: vec![
                FileEntry { name: "..".into(), is_dir: true, size: 0, modified: "".into() },
                FileEntry { name: "Documents".into(), is_dir: true, size: 0, modified: "2024-01-15".into() },
                FileEntry { name: "Downloads".into(), is_dir: true, size: 0, modified: "2024-01-14".into() },
                FileEntry { name: "project.tar.gz".into(), is_dir: false, size: 15728640, modified: "2024-01-15".into() },
                FileEntry { name: "config.yaml".into(), is_dir: false, size: 2048, modified: "2024-01-14".into() },
            ],
            remote_files: vec![
                FileEntry { name: "..".into(), is_dir: true, size: 0, modified: "".into() },
                FileEntry { name: "html".into(), is_dir: true, size: 0, modified: "2024-01-15".into() },
                FileEntry { name: "logs".into(), is_dir: true, size: 0, modified: "2024-01-14".into() },
                FileEntry { name: "index.html".into(), is_dir: false, size: 4096, modified: "2024-01-15".into() },
                FileEntry { name: "app.js".into(), is_dir: false, size: 65536, modified: "2024-01-14".into() },
            ],
            local_selected: 0,
            remote_selected: 0,
            transfers: vec![
                Transfer { filename: "backup.zip".into(), direction: "->".into(), progress: 67, size: 104857600, speed: "12.5 MB/s".into() },
            ],
            connected_host: "server.example.com".into(),
            tick_count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        for transfer in &mut self.transfers {
            if transfer.progress < 100 {
                transfer.progress = (transfer.progress + 3).min(100);
            }
        }
        self.transfers.retain(|t| t.progress < 100);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Tab => self.active_pane = if self.active_pane == PaneType::Local { PaneType::Remote } else { PaneType::Local },
            KeyCode::Char('j') | KeyCode::Down => {
                let (selected, files) = if self.active_pane == PaneType::Local { (&mut self.local_selected, &self.local_files) } else { (&mut self.remote_selected, &self.remote_files) };
                if *selected < files.len().saturating_sub(1) { *selected += 1; }
            },
            KeyCode::Char('k') | KeyCode::Up => {
                let selected = if self.active_pane == PaneType::Local { &mut self.local_selected } else { &mut self.remote_selected };
                *selected = selected.saturating_sub(1);
            },
            KeyCode::Enter => {
                let (selected, files, path) = if self.active_pane == PaneType::Local {
                    (self.local_selected, &self.local_files, &mut self.local_path)
                } else {
                    (self.remote_selected, &self.remote_files, &mut self.remote_path)
                };
                if let Some(file) = files.get(selected) {
                    if file.is_dir && file.name != ".." {
                        *path = format!("{}/{}", path, file.name);
                    }
                }
            },
            KeyCode::Char(' ') | KeyCode::Char('t') => {
                // Transfer selected file
                let (selected, files, direction) = if self.active_pane == PaneType::Local {
                    (self.local_selected, &self.local_files, "->")
                } else {
                    (self.remote_selected, &self.remote_files, "<-")
                };
                if let Some(file) = files.get(selected) {
                    if !file.is_dir {
                        self.transfers.push(Transfer {
                            filename: file.name.clone(),
                            direction: direction.into(),
                            progress: 0,
                            size: file.size,
                            speed: "0 B/s".into(),
                        });
                    }
                }
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        format!("tab:switch j/k:nav enter:open space:transfer q:quit | {} transfers", self.transfers.len())
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
