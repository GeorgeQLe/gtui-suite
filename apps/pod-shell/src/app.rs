use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone)]
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub containers: Vec<String>,
    pub status: PodStatus,
    pub node: String,
    pub restarts: u32,
    pub age: String,
}

#[derive(Clone, PartialEq)]
pub enum PodStatus {
    Running,
    Pending,
    Failed,
    Succeeded,
    Unknown,
}

impl PodStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PodStatus::Running => "Running",
            PodStatus::Pending => "Pending",
            PodStatus::Failed => "Failed",
            PodStatus::Succeeded => "Succeeded",
            PodStatus::Unknown => "Unknown",
        }
    }
}

pub struct ShellSession {
    pub pod: String,
    pub container: String,
    pub output: Vec<String>,
    pub input: String,
}

pub enum View {
    PodList,
    ContainerSelect,
    Shell,
}

pub struct App {
    pub pods: Vec<Pod>,
    pub selected_pod: usize,
    pub selected_container: usize,
    pub current_view: View,
    pub session: Option<ShellSession>,
    pub namespace_filter: String,
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            pods: vec![
                Pod {
                    name: "nginx-deployment-7fb96c846b-2xljm".to_string(),
                    namespace: "default".to_string(),
                    containers: vec!["nginx".to_string()],
                    status: PodStatus::Running,
                    node: "node-1".to_string(),
                    restarts: 0,
                    age: "3d".to_string(),
                },
                Pod {
                    name: "redis-master-0".to_string(),
                    namespace: "cache".to_string(),
                    containers: vec!["redis".to_string(), "redis-exporter".to_string()],
                    status: PodStatus::Running,
                    node: "node-2".to_string(),
                    restarts: 1,
                    age: "7d".to_string(),
                },
                Pod {
                    name: "prometheus-server-5f6d7c5b67-kxzwm".to_string(),
                    namespace: "monitoring".to_string(),
                    containers: vec!["prometheus".to_string(), "configmap-reload".to_string()],
                    status: PodStatus::Running,
                    node: "node-1".to_string(),
                    restarts: 0,
                    age: "14d".to_string(),
                },
                Pod {
                    name: "postgres-0".to_string(),
                    namespace: "database".to_string(),
                    containers: vec!["postgres".to_string()],
                    status: PodStatus::Running,
                    node: "node-3".to_string(),
                    restarts: 2,
                    age: "30d".to_string(),
                },
                Pod {
                    name: "failed-job-xyz123".to_string(),
                    namespace: "jobs".to_string(),
                    containers: vec!["worker".to_string()],
                    status: PodStatus::Failed,
                    node: "node-2".to_string(),
                    restarts: 5,
                    age: "1d".to_string(),
                },
                Pod {
                    name: "api-gateway-5c4d6f7b8-mnopq".to_string(),
                    namespace: "api".to_string(),
                    containers: vec!["gateway".to_string(), "envoy-sidecar".to_string()],
                    status: PodStatus::Running,
                    node: "node-1".to_string(),
                    restarts: 0,
                    age: "5d".to_string(),
                },
            ],
            selected_pod: 0,
            selected_container: 0,
            current_view: View::PodList,
            session: None,
            namespace_filter: String::new(),
            show_help: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            self.show_help = false;
            return false;
        }

        match &self.current_view {
            View::PodList => self.handle_pod_list_key(key),
            View::ContainerSelect => self.handle_container_select_key(key),
            View::Shell => self.handle_shell_key(key),
        }
    }

    fn handle_pod_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_pod < self.pods.len().saturating_sub(1) {
                    self.selected_pod += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_pod = self.selected_pod.saturating_sub(1);
            }
            KeyCode::Enter => {
                let pod = &self.pods[self.selected_pod];
                if pod.containers.len() > 1 {
                    self.selected_container = 0;
                    self.current_view = View::ContainerSelect;
                } else {
                    self.start_shell(0);
                }
            }
            KeyCode::Char('l') => {
                // View logs (demo)
            }
            KeyCode::Char('d') => {
                // Describe pod (demo)
            }
            _ => {}
        }
        false
    }

    fn handle_container_select_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.current_view = View::PodList;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let pod = &self.pods[self.selected_pod];
                if self.selected_container < pod.containers.len().saturating_sub(1) {
                    self.selected_container += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_container = self.selected_container.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.start_shell(self.selected_container);
            }
            _ => {}
        }
        false
    }

    fn handle_shell_key(&mut self, key: KeyEvent) -> bool {
        if let Some(ref mut session) = self.session {
            match key.code {
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.session = None;
                    self.current_view = View::PodList;
                }
                KeyCode::Esc => {
                    self.session = None;
                    self.current_view = View::PodList;
                }
                KeyCode::Enter => {
                    let cmd = session.input.clone();
                    session.output.push(format!("$ {}", cmd));
                    session.output.push(self.simulate_command(&cmd));
                    session.input.clear();
                }
                KeyCode::Char(c) => {
                    session.input.push(c);
                }
                KeyCode::Backspace => {
                    session.input.pop();
                }
                _ => {}
            }
        }
        false
    }

    fn start_shell(&mut self, container_idx: usize) {
        let pod = &self.pods[self.selected_pod];
        let container = &pod.containers[container_idx];

        self.session = Some(ShellSession {
            pod: pod.name.clone(),
            container: container.clone(),
            output: vec![
                format!("Connected to pod: {}", pod.name),
                format!("Container: {}", container),
                "Type commands or Ctrl+D/Esc to exit".to_string(),
                String::new(),
            ],
            input: String::new(),
        });
        self.current_view = View::Shell;
    }

    fn simulate_command(&self, cmd: &str) -> String {
        match cmd.trim() {
            "ls" => "bin  etc  home  lib  proc  root  sys  tmp  usr  var".to_string(),
            "pwd" => "/".to_string(),
            "whoami" => "root".to_string(),
            "hostname" => self.pods[self.selected_pod].name.clone(),
            "ps aux" => "PID   USER     TIME  COMMAND\n  1   root     0:05  /app/server".to_string(),
            "df -h" => "Filesystem      Size  Used Avail Use% Mounted on\noverlay         100G   20G   80G  20% /".to_string(),
            "cat /etc/os-release" => "NAME=\"Alpine Linux\"\nVERSION_ID=3.18".to_string(),
            _ => format!("Command executed: {}", cmd),
        }
    }

    pub fn selected_pod_containers(&self) -> &[String] {
        &self.pods[self.selected_pod].containers
    }
}
