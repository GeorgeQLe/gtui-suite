use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone)]
pub struct Dashboard {
    pub uid: String,
    pub title: String,
    pub folder: String,
    pub tags: Vec<String>,
    pub starred: bool,
    pub panels: Vec<Panel>,
}

#[derive(Clone)]
pub struct Panel {
    pub id: u32,
    pub title: String,
    pub panel_type: PanelType,
    pub datasource: String,
}

#[derive(Clone)]
pub enum PanelType {
    Graph,
    Stat,
    Gauge,
    Table,
    Text,
    Heatmap,
    Logs,
}

impl PanelType {
    pub fn as_str(&self) -> &str {
        match self {
            PanelType::Graph => "graph",
            PanelType::Stat => "stat",
            PanelType::Gauge => "gauge",
            PanelType::Table => "table",
            PanelType::Text => "text",
            PanelType::Heatmap => "heatmap",
            PanelType::Logs => "logs",
        }
    }
}

#[derive(Clone)]
pub struct Folder {
    pub uid: String,
    pub title: String,
    pub dashboard_count: usize,
}

pub enum View {
    Folders,
    Dashboards,
    Panels,
}

pub struct App {
    pub folders: Vec<Folder>,
    pub dashboards: Vec<Dashboard>,
    pub selected_folder: usize,
    pub selected_dashboard: usize,
    pub selected_panel: usize,
    pub current_view: View,
    pub show_help: bool,
    pub show_starred_only: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            folders: vec![
                Folder {
                    uid: "general".to_string(),
                    title: "General".to_string(),
                    dashboard_count: 3,
                },
                Folder {
                    uid: "kubernetes".to_string(),
                    title: "Kubernetes".to_string(),
                    dashboard_count: 5,
                },
                Folder {
                    uid: "infrastructure".to_string(),
                    title: "Infrastructure".to_string(),
                    dashboard_count: 4,
                },
                Folder {
                    uid: "applications".to_string(),
                    title: "Applications".to_string(),
                    dashboard_count: 6,
                },
            ],
            dashboards: Vec::new(),
            selected_folder: 0,
            selected_dashboard: 0,
            selected_panel: 0,
            current_view: View::Folders,
            show_help: false,
            show_starred_only: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            self.show_help = false;
            return false;
        }

        match &self.current_view {
            View::Folders => self.handle_folders_key(key),
            View::Dashboards => self.handle_dashboards_key(key),
            View::Panels => self.handle_panels_key(key),
        }
    }

    fn handle_folders_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_folder < self.folders.len().saturating_sub(1) {
                    self.selected_folder += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_folder = self.selected_folder.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.load_dashboards();
                self.current_view = View::Dashboards;
            }
            KeyCode::Char('s') => {
                self.show_starred_only = !self.show_starred_only;
            }
            _ => {}
        }
        false
    }

    fn handle_dashboards_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_view = View::Folders;
                self.dashboards.clear();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let filtered = self.filtered_dashboards();
                if self.selected_dashboard < filtered.len().saturating_sub(1) {
                    self.selected_dashboard += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_dashboard = self.selected_dashboard.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.selected_panel = 0;
                self.current_view = View::Panels;
            }
            KeyCode::Char('*') => {
                // Toggle star (demo)
                let filtered = self.filtered_dashboards();
                if let Some(dashboard) = filtered.get(self.selected_dashboard) {
                    let uid = dashboard.uid.clone();
                    if let Some(d) = self.dashboards.iter_mut().find(|d| d.uid == uid) {
                        d.starred = !d.starred;
                    }
                }
            }
            KeyCode::Char('s') => {
                self.show_starred_only = !self.show_starred_only;
                self.selected_dashboard = 0;
            }
            _ => {}
        }
        false
    }

    fn handle_panels_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_view = View::Dashboards;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(dashboard) = self.current_dashboard() {
                    if self.selected_panel < dashboard.panels.len().saturating_sub(1) {
                        self.selected_panel += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_panel = self.selected_panel.saturating_sub(1);
            }
            KeyCode::Enter => {
                // Open panel (demo)
            }
            _ => {}
        }
        false
    }

    fn load_dashboards(&mut self) {
        self.selected_dashboard = 0;
        let folder = &self.folders[self.selected_folder];

        self.dashboards = match folder.uid.as_str() {
            "general" => vec![
                Dashboard {
                    uid: "overview".to_string(),
                    title: "System Overview".to_string(),
                    folder: "General".to_string(),
                    tags: vec!["overview".to_string()],
                    starred: true,
                    panels: vec![
                        Panel { id: 1, title: "CPU Usage".to_string(), panel_type: PanelType::Graph, datasource: "Prometheus".to_string() },
                        Panel { id: 2, title: "Memory Usage".to_string(), panel_type: PanelType::Graph, datasource: "Prometheus".to_string() },
                        Panel { id: 3, title: "Uptime".to_string(), panel_type: PanelType::Stat, datasource: "Prometheus".to_string() },
                    ],
                },
                Dashboard {
                    uid: "network".to_string(),
                    title: "Network Stats".to_string(),
                    folder: "General".to_string(),
                    tags: vec!["network".to_string()],
                    starred: false,
                    panels: vec![
                        Panel { id: 1, title: "Bandwidth".to_string(), panel_type: PanelType::Graph, datasource: "Prometheus".to_string() },
                        Panel { id: 2, title: "Connections".to_string(), panel_type: PanelType::Stat, datasource: "Prometheus".to_string() },
                    ],
                },
            ],
            "kubernetes" => vec![
                Dashboard {
                    uid: "k8s-cluster".to_string(),
                    title: "Cluster Overview".to_string(),
                    folder: "Kubernetes".to_string(),
                    tags: vec!["kubernetes".to_string(), "cluster".to_string()],
                    starred: true,
                    panels: vec![
                        Panel { id: 1, title: "Node Count".to_string(), panel_type: PanelType::Stat, datasource: "Prometheus".to_string() },
                        Panel { id: 2, title: "Pod Count".to_string(), panel_type: PanelType::Stat, datasource: "Prometheus".to_string() },
                        Panel { id: 3, title: "CPU by Node".to_string(), panel_type: PanelType::Graph, datasource: "Prometheus".to_string() },
                    ],
                },
                Dashboard {
                    uid: "k8s-pods".to_string(),
                    title: "Pod Metrics".to_string(),
                    folder: "Kubernetes".to_string(),
                    tags: vec!["kubernetes".to_string(), "pods".to_string()],
                    starred: false,
                    panels: vec![
                        Panel { id: 1, title: "Pod CPU".to_string(), panel_type: PanelType::Graph, datasource: "Prometheus".to_string() },
                        Panel { id: 2, title: "Pod Memory".to_string(), panel_type: PanelType::Graph, datasource: "Prometheus".to_string() },
                    ],
                },
                Dashboard {
                    uid: "k8s-deployments".to_string(),
                    title: "Deployments".to_string(),
                    folder: "Kubernetes".to_string(),
                    tags: vec!["kubernetes".to_string(), "deployments".to_string()],
                    starred: false,
                    panels: vec![
                        Panel { id: 1, title: "Deployment Status".to_string(), panel_type: PanelType::Table, datasource: "Prometheus".to_string() },
                    ],
                },
            ],
            _ => vec![
                Dashboard {
                    uid: "default".to_string(),
                    title: "Default Dashboard".to_string(),
                    folder: folder.title.clone(),
                    tags: vec![],
                    starred: false,
                    panels: vec![
                        Panel { id: 1, title: "Sample Panel".to_string(), panel_type: PanelType::Graph, datasource: "Prometheus".to_string() },
                    ],
                },
            ],
        };
    }

    pub fn filtered_dashboards(&self) -> Vec<&Dashboard> {
        self.dashboards
            .iter()
            .filter(|d| !self.show_starred_only || d.starred)
            .collect()
    }

    pub fn current_dashboard(&self) -> Option<&Dashboard> {
        self.filtered_dashboards()
            .get(self.selected_dashboard)
            .copied()
    }

    pub fn current_folder_name(&self) -> &str {
        &self.folders[self.selected_folder].title
    }
}
