use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub uid: u32,
    pub gid: u32,
    pub home: String,
    pub shell: String,
    pub groups: Vec<String>,
}

pub struct App {
    pub users: Vec<User>,
    pub selected: usize,
    pub show_system_users: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            users: vec![
                User { username: "root".into(), uid: 0, gid: 0, home: "/root".into(), shell: "/bin/bash".into(), groups: vec!["root".into()] },
                User { username: "admin".into(), uid: 1000, gid: 1000, home: "/home/admin".into(), shell: "/bin/bash".into(), groups: vec!["admin".into(), "sudo".into(), "docker".into()] },
                User { username: "developer".into(), uid: 1001, gid: 1001, home: "/home/developer".into(), shell: "/bin/zsh".into(), groups: vec!["developer".into(), "docker".into()] },
                User { username: "nobody".into(), uid: 65534, gid: 65534, home: "/nonexistent".into(), shell: "/usr/sbin/nologin".into(), groups: vec!["nogroup".into()] },
                User { username: "www-data".into(), uid: 33, gid: 33, home: "/var/www".into(), shell: "/usr/sbin/nologin".into(), groups: vec!["www-data".into()] },
            ],
            selected: 0,
            show_system_users: false,
            status_message: None,
        }
    }

    pub fn visible_users(&self) -> Vec<&User> {
        self.users.iter()
            .filter(|u| self.show_system_users || (u.uid >= 1000 && u.uid < 65534) || u.uid == 0)
            .collect()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let visible = self.visible_users();
                if self.selected < visible.len().saturating_sub(1) { self.selected += 1; }
            },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('a') => self.status_message = Some("Would add user...".into()),
            KeyCode::Char('d') => self.status_message = Some("Would delete user...".into()),
            KeyCode::Char('e') => self.status_message = Some("Would edit user...".into()),
            KeyCode::Char('p') => self.status_message = Some("Would change password...".into()),
            KeyCode::Char('g') => self.status_message = Some("Would manage groups...".into()),
            KeyCode::Char('s') => {
                self.show_system_users = !self.show_system_users;
                self.selected = 0;
                self.status_message = Some(format!("System users: {}", if self.show_system_users { "shown" } else { "hidden" }));
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| "j/k:nav a:add d:delete e:edit p:password g:groups s:toggle-system q:quit".into())
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
