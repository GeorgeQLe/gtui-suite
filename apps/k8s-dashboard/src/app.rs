use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::config::Config;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Resources,
    Details,
    Logs,
    Yaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    NamespaceSelect,
    ContextSelect,
    ScaleDialog,
    Confirm,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeletePod(String),
    RestartDeployment(String),
    CordonNode(String),
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,

    // Cluster state
    pub contexts: Vec<Context>,
    pub active_context: usize,
    pub namespaces: Vec<Namespace>,
    pub active_namespace: usize,

    // Resources
    pub resource_type: ResourceType,
    pub pods: Vec<Pod>,
    pub deployments: Vec<Deployment>,
    pub services: Vec<Service>,
    pub nodes: Vec<Node>,
    pub events: Vec<Event>,

    // Selection
    pub selected_index: usize,
    pub current_pod: Option<Pod>,
    pub current_deployment: Option<Deployment>,

    // UI state
    pub show_sidebar: bool,
    pub search_query: String,
    pub logs: Option<String>,
    pub yaml: Option<String>,
    pub scroll_offset: usize,

    // Dialogs
    pub scale_value: u32,
    pub confirm_action: Option<ConfirmAction>,

    // Status
    pub status_message: Option<String>,
    pub connected: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            view: View::Resources,
            input_mode: InputMode::Normal,
            contexts: Vec::new(),
            active_context: 0,
            namespaces: Vec::new(),
            active_namespace: 0,
            resource_type: ResourceType::Pods,
            pods: Vec::new(),
            deployments: Vec::new(),
            services: Vec::new(),
            nodes: Vec::new(),
            events: Vec::new(),
            selected_index: 0,
            current_pod: None,
            current_deployment: None,
            show_sidebar: true,
            search_query: String::new(),
            logs: None,
            yaml: None,
            scroll_offset: 0,
            scale_value: 1,
            confirm_action: None,
            status_message: None,
            connected: false,
        }
    }

    pub async fn refresh(&mut self) {
        // Load demo contexts
        self.contexts = vec![
            Context::new("minikube", "minikube", "minikube"),
            Context::new("production", "prod-cluster", "admin"),
            Context::new("staging", "staging-cluster", "developer"),
        ];

        // Load demo namespaces
        self.namespaces = vec![
            Namespace::new("default"),
            Namespace::new("kube-system"),
            Namespace::new("kube-public"),
            Namespace::new("monitoring"),
        ];

        // Load demo pods
        self.pods = vec![
            {
                let mut pod = Pod::new("nginx-deployment-abc123", "default");
                pod.phase = PodPhase::Running;
                pod.ready = "1/1".to_string();
                pod.restarts = 0;
                pod.node = "minikube".to_string();
                pod.cpu = Some(0.05);
                pod.memory = Some(64 * 1024 * 1024);
                pod.containers = vec![Container::new("nginx", "nginx:1.21")];
                pod
            },
            {
                let mut pod = Pod::new("redis-master-def456", "default");
                pod.phase = PodPhase::Running;
                pod.ready = "1/1".to_string();
                pod.restarts = 2;
                pod.node = "minikube".to_string();
                pod.cpu = Some(0.1);
                pod.memory = Some(128 * 1024 * 1024);
                pod
            },
            {
                let mut pod = Pod::new("api-server-ghi789", "default");
                pod.phase = PodPhase::Pending;
                pod.ready = "0/1".to_string();
                pod.restarts = 0;
                pod.node = "".to_string();
                pod
            },
            {
                let mut pod = Pod::new("worker-job-xyz", "default");
                pod.phase = PodPhase::Failed;
                pod.ready = "0/1".to_string();
                pod.restarts = 5;
                pod
            },
        ];

        // Load demo deployments
        self.deployments = vec![
            Deployment::new("nginx-deployment", "default", 3),
            Deployment::new("redis-master", "default", 1),
            Deployment::new("api-server", "default", 2),
        ];

        // Load demo services
        self.services = vec![
            {
                let mut svc = Service::new("kubernetes", "default", "ClusterIP");
                svc.ports = vec!["443/TCP".to_string()];
                svc
            },
            {
                let mut svc = Service::new("nginx", "default", "LoadBalancer");
                svc.ports = vec!["80/TCP".to_string()];
                svc.external_ip = Some("192.168.1.100".to_string());
                svc
            },
        ];

        // Load demo nodes
        self.nodes = vec![
            {
                let mut node = Node::new("minikube");
                node.roles = vec!["control-plane".to_string(), "master".to_string()];
                node.cpu_usage = Some(0.5);
                node.memory_usage = Some(2 * 1024 * 1024 * 1024);
                node
            },
            {
                let mut node = Node::new("worker-1");
                node.cpu_usage = Some(0.3);
                node.memory_usage = Some(4 * 1024 * 1024 * 1024);
                node
            },
        ];

        // Load demo events
        self.events = vec![
            Event::new("Normal", "Scheduled", "pod/nginx-abc123", "Successfully assigned to minikube"),
            Event::new("Normal", "Pulled", "pod/nginx-abc123", "Container image pulled"),
            Event::new("Normal", "Created", "pod/nginx-abc123", "Created container nginx"),
            Event::new("Warning", "BackOff", "pod/worker-job-xyz", "Back-off restarting failed container"),
        ];

        self.connected = true;

        if self.selected_index >= self.current_resource_count() {
            self.selected_index = self.current_resource_count().saturating_sub(1);
        }
    }

    fn current_resource_count(&self) -> usize {
        match self.resource_type {
            ResourceType::Pods => self.pods.len(),
            ResourceType::Deployments => self.deployments.len(),
            ResourceType::Services => self.services.len(),
            ResourceType::Nodes => self.nodes.len(),
            ResourceType::Events => self.events.len(),
            ResourceType::Namespaces => self.namespaces.len(),
            _ => 0,
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::Search => self.handle_search_key(key),
            InputMode::NamespaceSelect => self.handle_namespace_key(key),
            InputMode::ContextSelect => self.handle_context_key(key),
            InputMode::ScaleDialog => self.handle_scale_key(key).await,
            InputMode::Confirm => self.handle_confirm_key(key).await,
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') => {
                match self.view {
                    View::Logs | View::Yaml => {
                        self.view = View::Details;
                        self.logs = None;
                        self.yaml = None;
                    }
                    View::Details => {
                        self.view = View::Resources;
                        self.current_pod = None;
                        self.current_deployment = None;
                    }
                    View::Resources => return true,
                }
            }

            KeyCode::Char('j') | KeyCode::Down => {
                self.navigate_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.navigate_up();
            }
            KeyCode::Char('g') => {
                self.selected_index = 0;
            }
            KeyCode::Char('G') => {
                self.selected_index = self.current_resource_count().saturating_sub(1);
            }

            KeyCode::Enter => {
                self.open_details();
            }

            KeyCode::Tab => {
                self.next_resource_type();
            }
            KeyCode::BackTab => {
                self.prev_resource_type();
            }

            KeyCode::Char('n') => {
                self.input_mode = InputMode::NamespaceSelect;
            }

            KeyCode::Char('c') => {
                self.input_mode = InputMode::ContextSelect;
            }

            KeyCode::Char('l') => {
                if self.resource_type == ResourceType::Pods {
                    self.view_logs();
                }
            }

            KeyCode::Char('y') => {
                self.view_yaml();
            }

            KeyCode::Char('d') => {
                if self.resource_type == ResourceType::Pods {
                    if let Some(pod) = self.pods.get(self.selected_index) {
                        self.confirm_action = Some(ConfirmAction::DeletePod(pod.name.clone()));
                        self.input_mode = InputMode::Confirm;
                    }
                }
            }

            KeyCode::Char('s') => {
                if self.resource_type == ResourceType::Deployments {
                    if let Some(deploy) = self.deployments.get(self.selected_index) {
                        self.scale_value = deploy.available;
                        self.input_mode = InputMode::ScaleDialog;
                    }
                }
            }

            KeyCode::Char('R') => {
                if self.resource_type == ResourceType::Deployments {
                    if let Some(deploy) = self.deployments.get(self.selected_index) {
                        self.confirm_action =
                            Some(ConfirmAction::RestartDeployment(deploy.name.clone()));
                        self.input_mode = InputMode::Confirm;
                    }
                }
            }

            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }

            KeyCode::Char('b') => {
                self.show_sidebar = !self.show_sidebar;
            }

            KeyCode::Char('r') => {
                self.refresh().await;
                self.status_message = Some("Refreshed".to_string());
            }

            KeyCode::Esc => {
                self.view = View::Resources;
                self.current_pod = None;
                self.current_deployment = None;
                self.logs = None;
                self.yaml = None;
            }

            _ => {}
        }

        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
        false
    }

    fn handle_namespace_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.active_namespace =
                    (self.active_namespace + 1).min(self.namespaces.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.active_namespace = self.active_namespace.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.status_message = Some(format!(
                    "Switched to namespace: {}",
                    self.namespaces
                        .get(self.active_namespace)
                        .map(|n| n.name.as_str())
                        .unwrap_or("default")
                ));
            }
            _ => {}
        }
        false
    }

    fn handle_context_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.active_context =
                    (self.active_context + 1).min(self.contexts.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.active_context = self.active_context.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.status_message = Some(format!(
                    "Switched to context: {}",
                    self.contexts
                        .get(self.active_context)
                        .map(|c| c.name.as_str())
                        .unwrap_or("default")
                ));
            }
            _ => {}
        }
        false
    }

    async fn handle_scale_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.status_message = Some(format!("Scaled to {} replicas", self.scale_value));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scale_value = self.scale_value.saturating_add(1).min(100);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scale_value = self.scale_value.saturating_sub(1);
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).unwrap() as u32;
                self.scale_value = (self.scale_value * 10 + digit).min(100);
            }
            KeyCode::Backspace => {
                self.scale_value /= 10;
            }
            _ => {}
        }
        false
    }

    async fn handle_confirm_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Some(action) = self.confirm_action.take() {
                    match action {
                        ConfirmAction::DeletePod(name) => {
                            self.status_message = Some(format!("Deleted pod: {}", name));
                        }
                        ConfirmAction::RestartDeployment(name) => {
                            self.status_message =
                                Some(format!("Restarting deployment: {}", name));
                        }
                        ConfirmAction::CordonNode(name) => {
                            self.status_message = Some(format!("Cordoned node: {}", name));
                        }
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.confirm_action = None;
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        false
    }

    fn navigate_down(&mut self) {
        let max = self.current_resource_count().saturating_sub(1);
        if self.selected_index < max {
            self.selected_index += 1;
        }
    }

    fn navigate_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    fn next_resource_type(&mut self) {
        let types = ResourceType::all();
        let current_idx = types.iter().position(|t| *t == self.resource_type).unwrap_or(0);
        self.resource_type = types[(current_idx + 1) % types.len()];
        self.selected_index = 0;
    }

    fn prev_resource_type(&mut self) {
        let types = ResourceType::all();
        let current_idx = types.iter().position(|t| *t == self.resource_type).unwrap_or(0);
        self.resource_type = types[(current_idx + types.len() - 1) % types.len()];
        self.selected_index = 0;
    }

    fn open_details(&mut self) {
        match self.resource_type {
            ResourceType::Pods => {
                if let Some(pod) = self.pods.get(self.selected_index) {
                    self.current_pod = Some(pod.clone());
                    self.view = View::Details;
                }
            }
            ResourceType::Deployments => {
                if let Some(deploy) = self.deployments.get(self.selected_index) {
                    self.current_deployment = Some(deploy.clone());
                    self.view = View::Details;
                }
            }
            _ => {}
        }
    }

    fn view_logs(&mut self) {
        if let Some(pod) = &self.current_pod {
            self.logs = Some(format!(
                "Logs for pod: {}\n\n\
                [2024-01-08 10:00:00] Starting container...\n\
                [2024-01-08 10:00:01] Configuration loaded\n\
                [2024-01-08 10:00:02] Server listening on port 80\n\
                [2024-01-08 10:01:00] GET / 200 0.5ms\n\
                [2024-01-08 10:01:05] GET /health 200 0.2ms\n",
                pod.name
            ));
            self.view = View::Logs;
        }
    }

    fn view_yaml(&mut self) {
        match self.resource_type {
            ResourceType::Pods => {
                if let Some(pod) = self.pods.get(self.selected_index) {
                    self.yaml = Some(format!(
                        "apiVersion: v1\n\
                        kind: Pod\n\
                        metadata:\n\
                        \x20 name: {}\n\
                        \x20 namespace: {}\n\
                        spec:\n\
                        \x20 containers:\n\
                        \x20 - name: {}\n\
                        \x20   image: nginx:1.21\n",
                        pod.name,
                        pod.namespace,
                        pod.containers.first().map(|c| c.name.as_str()).unwrap_or("main")
                    ));
                    self.view = View::Yaml;
                }
            }
            ResourceType::Deployments => {
                if let Some(deploy) = self.deployments.get(self.selected_index) {
                    self.yaml = Some(format!(
                        "apiVersion: apps/v1\n\
                        kind: Deployment\n\
                        metadata:\n\
                        \x20 name: {}\n\
                        \x20 namespace: {}\n\
                        spec:\n\
                        \x20 replicas: {}\n",
                        deploy.name, deploy.namespace, deploy.available
                    ));
                    self.view = View::Yaml;
                }
            }
            _ => {}
        }
    }

    pub fn current_namespace(&self) -> &str {
        self.namespaces
            .get(self.active_namespace)
            .map(|n| n.name.as_str())
            .unwrap_or("default")
    }

    pub fn current_context(&self) -> &str {
        self.contexts
            .get(self.active_context)
            .map(|c| c.name.as_str())
            .unwrap_or("default")
    }

    pub fn status_text(&self) -> String {
        if let Some(msg) = &self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Resources => format!(
                "{} {} | ctx:{} ns:{} | Tab:switch n:namespace c:context q:quit",
                self.current_resource_count(),
                self.resource_type.as_str(),
                self.current_context(),
                self.current_namespace()
            ),
            View::Details => "l:logs y:yaml d:delete s:scale R:restart q:back".to_string(),
            View::Logs => "j/k:scroll q:back".to_string(),
            View::Yaml => "q:back".to_string(),
        }
    }
}
