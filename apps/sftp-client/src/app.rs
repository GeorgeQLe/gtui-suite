use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct RemoteFile {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: String,
    pub owner: String,
    pub modified: String,
}

pub struct App {
    pub host: String,
    pub username: String,
    pub current_path: String,
    pub files: Vec<RemoteFile>,
    pub selected: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            host: "server.example.com".into(),
            username: "admin".into(),
            current_path: "/home/admin".into(),
            files: vec![
                RemoteFile { name: "..".into(), is_dir: true, size: 0, permissions: "drwxr-xr-x".into(), owner: "admin".into(), modified: "".into() },
                RemoteFile { name: ".bashrc".into(), is_dir: false, size: 3771, permissions: "-rw-r--r--".into(), owner: "admin".into(), modified: "2024-01-10".into() },
                RemoteFile { name: ".ssh".into(), is_dir: true, size: 4096, permissions: "drwx------".into(), owner: "admin".into(), modified: "2024-01-15".into() },
                RemoteFile { name: "projects".into(), is_dir: true, size: 4096, permissions: "drwxr-xr-x".into(), owner: "admin".into(), modified: "2024-01-14".into() },
                RemoteFile { name: "backup.tar.gz".into(), is_dir: false, size: 52428800, permissions: "-rw-r--r--".into(), owner: "admin".into(), modified: "2024-01-15".into() },
                RemoteFile { name: "deploy.sh".into(), is_dir: false, size: 1024, permissions: "-rwxr-xr-x".into(), owner: "admin".into(), modified: "2024-01-12".into() },
                RemoteFile { name: "logs".into(), is_dir: true, size: 4096, permissions: "drwxr-xr-x".into(), owner: "admin".into(), modified: "2024-01-15".into() },
            ],
            selected: 0,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.files.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Enter => {
                if let Some(file) = self.files.get(self.selected) {
                    if file.is_dir {
                        if file.name == ".." {
                            let parts: Vec<&str> = self.current_path.rsplitn(2, '/').collect();
                            if parts.len() > 1 { self.current_path = parts[1].to_string(); }
                            if self.current_path.is_empty() { self.current_path = "/".into(); }
                        } else {
                            self.current_path = format!("{}/{}", self.current_path, file.name);
                        }
                        self.selected = 0;
                    }
                }
            },
            KeyCode::Char('d') => self.status_message = Some("Would download file...".into()),
            KeyCode::Char('u') => self.status_message = Some("Would upload file...".into()),
            KeyCode::Char('n') => self.status_message = Some("Would create directory...".into()),
            KeyCode::Char('r') => self.status_message = Some("Would rename...".into()),
            KeyCode::Char('x') => self.status_message = Some("Would delete...".into()),
            KeyCode::Char('p') => self.status_message = Some("Would change permissions...".into()),
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| "j/k:nav enter:open d:download u:upload n:mkdir r:rename x:delete p:chmod q:quit".into())
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
