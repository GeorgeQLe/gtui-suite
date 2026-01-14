use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Zombie,
    Defunct,
    Orphan,
}

impl ProcessState {
    pub fn name(&self) -> &'static str {
        match self {
            ProcessState::Zombie => "Z (Zombie)",
            ProcessState::Defunct => "D (Defunct)",
            ProcessState::Orphan => "O (Orphan)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            ProcessState::Zombie => "Z",
            ProcessState::Defunct => "D",
            ProcessState::Orphan => "O",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZombieProcess {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub parent_name: String,
    pub state: ProcessState,
    pub cpu_time: String,
    pub start_time: String,
    pub selected: bool,
}

pub struct App {
    pub zombies: Vec<ZombieProcess>,
    pub selected: usize,
    pub show_orphans: bool,
    pub auto_refresh: bool,
    pub tick_count: u64,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            zombies: create_demo_zombies(),
            selected: 0,
            show_orphans: true,
            auto_refresh: true,
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
                if self.selected < self.filtered_zombies().len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char(' ') => {
                let filtered = self.filtered_indices();
                if let Some(&idx) = filtered.get(self.selected) {
                    self.zombies[idx].selected = !self.zombies[idx].selected;
                }
            }
            KeyCode::Char('a') => {
                let filtered = self.filtered_indices();
                for &idx in &filtered {
                    self.zombies[idx].selected = true;
                }
                self.status_message = Some("All selected".to_string());
            }
            KeyCode::Char('n') => {
                for zombie in &mut self.zombies {
                    zombie.selected = false;
                }
                self.status_message = Some("None selected".to_string());
            }
            KeyCode::Char('o') => {
                self.show_orphans = !self.show_orphans;
                self.selected = 0;
                self.status_message = Some(if self.show_orphans {
                    "Showing orphans".to_string()
                } else {
                    "Hiding orphans".to_string()
                });
            }
            KeyCode::Char('K') => {
                let selected_count = self.zombies.iter().filter(|z| z.selected).count();
                if selected_count > 0 {
                    self.status_message = Some(format!("Would kill parent of {} zombies", selected_count));
                } else {
                    let filtered = self.filtered_indices();
                    if let Some(&idx) = filtered.get(self.selected) {
                        let zombie = &self.zombies[idx];
                        self.status_message = Some(format!("Would kill parent {} (PID {})", zombie.parent_name, zombie.ppid));
                    }
                }
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Refreshing...".to_string());
            }
            KeyCode::Char('R') => {
                self.auto_refresh = !self.auto_refresh;
                self.status_message = Some(if self.auto_refresh {
                    "Auto-refresh enabled".to_string()
                } else {
                    "Auto-refresh disabled".to_string()
                });
            }
            _ => {}
        }
        false
    }

    fn filtered_indices(&self) -> Vec<usize> {
        self.zombies
            .iter()
            .enumerate()
            .filter(|(_, z)| self.show_orphans || z.state != ProcessState::Orphan)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn filtered_zombies(&self) -> Vec<&ZombieProcess> {
        self.zombies
            .iter()
            .filter(|z| self.show_orphans || z.state != ProcessState::Orphan)
            .collect()
    }

    pub fn zombie_count(&self) -> usize {
        self.zombies.iter().filter(|z| z.state == ProcessState::Zombie).count()
    }

    pub fn orphan_count(&self) -> usize {
        self.zombies.iter().filter(|z| z.state == ProcessState::Orphan).count()
    }

    pub fn selected_count(&self) -> usize {
        self.zombies.iter().filter(|z| z.selected).count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav space:select a:all n:none o:orphans K:kill r:refresh q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_zombies() -> Vec<ZombieProcess> {
    vec![
        ZombieProcess {
            pid: 12345,
            ppid: 1234,
            name: "worker".to_string(),
            parent_name: "python3".to_string(),
            state: ProcessState::Zombie,
            cpu_time: "00:00:05".to_string(),
            start_time: "14:32:15".to_string(),
            selected: false,
        },
        ZombieProcess {
            pid: 23456,
            ppid: 2345,
            name: "child".to_string(),
            parent_name: "node".to_string(),
            state: ProcessState::Zombie,
            cpu_time: "00:00:12".to_string(),
            start_time: "10:15:42".to_string(),
            selected: false,
        },
        ZombieProcess {
            pid: 34567,
            ppid: 1,
            name: "orphaned".to_string(),
            parent_name: "init".to_string(),
            state: ProcessState::Orphan,
            cpu_time: "00:01:23".to_string(),
            start_time: "22:45:00".to_string(),
            selected: false,
        },
        ZombieProcess {
            pid: 45678,
            ppid: 4567,
            name: "defunct".to_string(),
            parent_name: "bash".to_string(),
            state: ProcessState::Defunct,
            cpu_time: "00:00:00".to_string(),
            start_time: "18:20:33".to_string(),
            selected: false,
        },
        ZombieProcess {
            pid: 56789,
            ppid: 5678,
            name: "zombie_child".to_string(),
            parent_name: "java".to_string(),
            state: ProcessState::Zombie,
            cpu_time: "00:00:08".to_string(),
            start_time: "12:10:05".to_string(),
            selected: false,
        },
        ZombieProcess {
            pid: 67890,
            ppid: 1,
            name: "old_orphan".to_string(),
            parent_name: "init".to_string(),
            state: ProcessState::Orphan,
            cpu_time: "00:05:45".to_string(),
            start_time: "08:00:00".to_string(),
            selected: false,
        },
    ]
}
