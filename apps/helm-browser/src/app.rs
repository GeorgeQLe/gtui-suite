use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone)]
pub struct HelmRelease {
    pub name: String,
    pub namespace: String,
    pub chart: String,
    pub version: String,
    pub app_version: String,
    pub status: ReleaseStatus,
    pub revision: u32,
    pub updated: String,
}

#[derive(Clone, PartialEq)]
pub enum ReleaseStatus {
    Deployed,
    Failed,
    Pending,
    Uninstalling,
    Superseded,
}

impl ReleaseStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ReleaseStatus::Deployed => "deployed",
            ReleaseStatus::Failed => "failed",
            ReleaseStatus::Pending => "pending",
            ReleaseStatus::Uninstalling => "uninstalling",
            ReleaseStatus::Superseded => "superseded",
        }
    }
}

#[derive(Clone)]
pub struct HelmRepo {
    pub name: String,
    pub url: String,
}

pub enum Tab {
    Releases,
    Repositories,
    Charts,
}

pub struct App {
    pub releases: Vec<HelmRelease>,
    pub repos: Vec<HelmRepo>,
    pub charts: Vec<String>,
    pub selected_release: usize,
    pub selected_repo: usize,
    pub selected_chart: usize,
    pub current_tab: Tab,
    pub show_help: bool,
    pub show_values: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            releases: vec![
                HelmRelease {
                    name: "nginx-ingress".to_string(),
                    namespace: "ingress".to_string(),
                    chart: "ingress-nginx".to_string(),
                    version: "4.8.3".to_string(),
                    app_version: "1.9.4".to_string(),
                    status: ReleaseStatus::Deployed,
                    revision: 5,
                    updated: "2024-01-15 10:30:00".to_string(),
                },
                HelmRelease {
                    name: "prometheus".to_string(),
                    namespace: "monitoring".to_string(),
                    chart: "prometheus".to_string(),
                    version: "25.8.0".to_string(),
                    app_version: "2.48.0".to_string(),
                    status: ReleaseStatus::Deployed,
                    revision: 3,
                    updated: "2024-01-14 09:15:00".to_string(),
                },
                HelmRelease {
                    name: "grafana".to_string(),
                    namespace: "monitoring".to_string(),
                    chart: "grafana".to_string(),
                    version: "7.0.11".to_string(),
                    app_version: "10.2.3".to_string(),
                    status: ReleaseStatus::Deployed,
                    revision: 2,
                    updated: "2024-01-13 14:20:00".to_string(),
                },
                HelmRelease {
                    name: "redis".to_string(),
                    namespace: "cache".to_string(),
                    chart: "redis".to_string(),
                    version: "18.6.1".to_string(),
                    app_version: "7.2.4".to_string(),
                    status: ReleaseStatus::Failed,
                    revision: 1,
                    updated: "2024-01-12 08:00:00".to_string(),
                },
                HelmRelease {
                    name: "postgresql".to_string(),
                    namespace: "database".to_string(),
                    chart: "postgresql".to_string(),
                    version: "13.4.0".to_string(),
                    app_version: "16.1.0".to_string(),
                    status: ReleaseStatus::Deployed,
                    revision: 4,
                    updated: "2024-01-11 16:45:00".to_string(),
                },
                HelmRelease {
                    name: "cert-manager".to_string(),
                    namespace: "cert-manager".to_string(),
                    chart: "cert-manager".to_string(),
                    version: "1.13.3".to_string(),
                    app_version: "1.13.3".to_string(),
                    status: ReleaseStatus::Deployed,
                    revision: 2,
                    updated: "2024-01-10 11:30:00".to_string(),
                },
            ],
            repos: vec![
                HelmRepo {
                    name: "bitnami".to_string(),
                    url: "https://charts.bitnami.com/bitnami".to_string(),
                },
                HelmRepo {
                    name: "prometheus-community".to_string(),
                    url: "https://prometheus-community.github.io/helm-charts".to_string(),
                },
                HelmRepo {
                    name: "grafana".to_string(),
                    url: "https://grafana.github.io/helm-charts".to_string(),
                },
                HelmRepo {
                    name: "jetstack".to_string(),
                    url: "https://charts.jetstack.io".to_string(),
                },
                HelmRepo {
                    name: "ingress-nginx".to_string(),
                    url: "https://kubernetes.github.io/ingress-nginx".to_string(),
                },
            ],
            charts: vec![
                "bitnami/postgresql".to_string(),
                "bitnami/redis".to_string(),
                "bitnami/mysql".to_string(),
                "bitnami/mongodb".to_string(),
                "prometheus-community/prometheus".to_string(),
                "prometheus-community/alertmanager".to_string(),
                "grafana/grafana".to_string(),
                "grafana/loki".to_string(),
                "jetstack/cert-manager".to_string(),
                "ingress-nginx/ingress-nginx".to_string(),
            ],
            selected_release: 0,
            selected_repo: 0,
            selected_chart: 0,
            current_tab: Tab::Releases,
            show_help: false,
            show_values: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            self.show_help = false;
            return false;
        }

        if self.show_values {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('v') {
                self.show_values = false;
            }
            return false;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Tab => {
                self.current_tab = match self.current_tab {
                    Tab::Releases => Tab::Repositories,
                    Tab::Repositories => Tab::Charts,
                    Tab::Charts => Tab::Releases,
                };
            }
            KeyCode::Char('1') => self.current_tab = Tab::Releases,
            KeyCode::Char('2') => self.current_tab = Tab::Repositories,
            KeyCode::Char('3') => self.current_tab = Tab::Charts,
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('v') => {
                if matches!(self.current_tab, Tab::Releases) {
                    self.show_values = true;
                }
            }
            KeyCode::Char('u') => {
                // Upgrade release (demo)
            }
            KeyCode::Char('r') => {
                // Rollback release (demo)
            }
            KeyCode::Char('d') => {
                // Delete release (demo)
            }
            _ => {}
        }
        false
    }

    fn move_down(&mut self) {
        match self.current_tab {
            Tab::Releases => {
                if self.selected_release < self.releases.len().saturating_sub(1) {
                    self.selected_release += 1;
                }
            }
            Tab::Repositories => {
                if self.selected_repo < self.repos.len().saturating_sub(1) {
                    self.selected_repo += 1;
                }
            }
            Tab::Charts => {
                if self.selected_chart < self.charts.len().saturating_sub(1) {
                    self.selected_chart += 1;
                }
            }
        }
    }

    fn move_up(&mut self) {
        match self.current_tab {
            Tab::Releases => {
                self.selected_release = self.selected_release.saturating_sub(1);
            }
            Tab::Repositories => {
                self.selected_repo = self.selected_repo.saturating_sub(1);
            }
            Tab::Charts => {
                self.selected_chart = self.selected_chart.saturating_sub(1);
            }
        }
    }
}
