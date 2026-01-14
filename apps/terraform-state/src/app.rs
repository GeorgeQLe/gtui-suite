use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone)]
pub struct Workspace {
    pub name: String,
    pub backend: String,
    pub last_modified: String,
    pub resources: usize,
}

#[derive(Clone)]
pub struct Resource {
    pub address: String,
    pub resource_type: String,
    pub provider: String,
    pub mode: String,
    pub tainted: bool,
}

pub enum View {
    Workspaces,
    Resources,
    Details,
}

pub struct App {
    pub workspaces: Vec<Workspace>,
    pub resources: Vec<Resource>,
    pub selected_workspace: usize,
    pub selected_resource: usize,
    pub current_view: View,
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            workspaces: vec![
                Workspace {
                    name: "production".to_string(),
                    backend: "s3://terraform-state/prod".to_string(),
                    last_modified: "2024-01-15 10:30:00".to_string(),
                    resources: 45,
                },
                Workspace {
                    name: "staging".to_string(),
                    backend: "s3://terraform-state/staging".to_string(),
                    last_modified: "2024-01-15 09:15:00".to_string(),
                    resources: 32,
                },
                Workspace {
                    name: "development".to_string(),
                    backend: "s3://terraform-state/dev".to_string(),
                    last_modified: "2024-01-14 16:45:00".to_string(),
                    resources: 28,
                },
                Workspace {
                    name: "testing".to_string(),
                    backend: "local://terraform.tfstate".to_string(),
                    last_modified: "2024-01-14 11:20:00".to_string(),
                    resources: 15,
                },
            ],
            resources: Vec::new(),
            selected_workspace: 0,
            selected_resource: 0,
            current_view: View::Workspaces,
            show_help: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            self.show_help = false;
            return false;
        }

        match &self.current_view {
            View::Workspaces => self.handle_workspaces_key(key),
            View::Resources => self.handle_resources_key(key),
            View::Details => self.handle_details_key(key),
        }
    }

    fn handle_workspaces_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_workspace < self.workspaces.len().saturating_sub(1) {
                    self.selected_workspace += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_workspace = self.selected_workspace.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.load_resources();
                self.current_view = View::Resources;
            }
            KeyCode::Char('r') => {
                // Refresh state (demo)
            }
            _ => {}
        }
        false
    }

    fn handle_resources_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_view = View::Workspaces;
                self.resources.clear();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_resource < self.resources.len().saturating_sub(1) {
                    self.selected_resource += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_resource = self.selected_resource.saturating_sub(1);
            }
            KeyCode::Enter => {
                if !self.resources.is_empty() {
                    self.current_view = View::Details;
                }
            }
            KeyCode::Char('t') => {
                // Toggle taint (demo)
                if !self.resources.is_empty() {
                    self.resources[self.selected_resource].tainted =
                        !self.resources[self.selected_resource].tainted;
                }
            }
            KeyCode::Char('d') => {
                // Destroy resource (demo)
            }
            KeyCode::Char('m') => {
                // Move resource (demo)
            }
            _ => {}
        }
        false
    }

    fn handle_details_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Enter => {
                self.current_view = View::Resources;
            }
            _ => {}
        }
        false
    }

    fn load_resources(&mut self) {
        let workspace = &self.workspaces[self.selected_workspace];
        self.selected_resource = 0;

        self.resources = match workspace.name.as_str() {
            "production" => vec![
                Resource {
                    address: "aws_vpc.main".to_string(),
                    resource_type: "aws_vpc".to_string(),
                    provider: "provider[\"registry.terraform.io/hashicorp/aws\"]".to_string(),
                    mode: "managed".to_string(),
                    tainted: false,
                },
                Resource {
                    address: "aws_subnet.public[0]".to_string(),
                    resource_type: "aws_subnet".to_string(),
                    provider: "provider[\"registry.terraform.io/hashicorp/aws\"]".to_string(),
                    mode: "managed".to_string(),
                    tainted: false,
                },
                Resource {
                    address: "aws_subnet.public[1]".to_string(),
                    resource_type: "aws_subnet".to_string(),
                    provider: "provider[\"registry.terraform.io/hashicorp/aws\"]".to_string(),
                    mode: "managed".to_string(),
                    tainted: false,
                },
                Resource {
                    address: "aws_instance.web[0]".to_string(),
                    resource_type: "aws_instance".to_string(),
                    provider: "provider[\"registry.terraform.io/hashicorp/aws\"]".to_string(),
                    mode: "managed".to_string(),
                    tainted: true,
                },
                Resource {
                    address: "aws_instance.web[1]".to_string(),
                    resource_type: "aws_instance".to_string(),
                    provider: "provider[\"registry.terraform.io/hashicorp/aws\"]".to_string(),
                    mode: "managed".to_string(),
                    tainted: false,
                },
                Resource {
                    address: "aws_rds_cluster.main".to_string(),
                    resource_type: "aws_rds_cluster".to_string(),
                    provider: "provider[\"registry.terraform.io/hashicorp/aws\"]".to_string(),
                    mode: "managed".to_string(),
                    tainted: false,
                },
                Resource {
                    address: "aws_s3_bucket.assets".to_string(),
                    resource_type: "aws_s3_bucket".to_string(),
                    provider: "provider[\"registry.terraform.io/hashicorp/aws\"]".to_string(),
                    mode: "managed".to_string(),
                    tainted: false,
                },
            ],
            _ => vec![
                Resource {
                    address: "aws_vpc.main".to_string(),
                    resource_type: "aws_vpc".to_string(),
                    provider: "provider[\"registry.terraform.io/hashicorp/aws\"]".to_string(),
                    mode: "managed".to_string(),
                    tainted: false,
                },
                Resource {
                    address: "aws_instance.app".to_string(),
                    resource_type: "aws_instance".to_string(),
                    provider: "provider[\"registry.terraform.io/hashicorp/aws\"]".to_string(),
                    mode: "managed".to_string(),
                    tainted: false,
                },
                Resource {
                    address: "aws_security_group.allow_http".to_string(),
                    resource_type: "aws_security_group".to_string(),
                    provider: "provider[\"registry.terraform.io/hashicorp/aws\"]".to_string(),
                    mode: "managed".to_string(),
                    tainted: false,
                },
            ],
        };
    }

    pub fn current_workspace_name(&self) -> &str {
        &self.workspaces[self.selected_workspace].name
    }
}
