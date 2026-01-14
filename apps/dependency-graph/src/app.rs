use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub deps: Vec<Dependency>,
    pub dev: bool,
    pub optional: bool,
}

pub struct App {
    pub tree: Vec<Dependency>,
    pub flat_list: Vec<(usize, String, String, bool)>, // (depth, name, version, expanded)
    pub selected: usize,
    pub expanded: std::collections::HashSet<String>,
    pub show_dev: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let tree = create_demo_deps();
        let mut app = Self {
            tree,
            flat_list: Vec::new(),
            selected: 0,
            expanded: std::collections::HashSet::new(),
            show_dev: false,
            status_message: None,
        };
        app.rebuild_flat_list();
        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.flat_list.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_expanded();
            }
            KeyCode::Char('e') => {
                self.expand_all();
            }
            KeyCode::Char('c') => {
                self.collapse_all();
            }
            KeyCode::Char('d') => {
                self.show_dev = !self.show_dev;
                self.rebuild_flat_list();
                self.status_message = Some(format!(
                    "Dev dependencies: {}",
                    if self.show_dev { "shown" } else { "hidden" }
                ));
            }
            KeyCode::Char('y') => {
                if let Some((_, name, version, _)) = self.flat_list.get(self.selected) {
                    self.status_message = Some(format!("Copied: {} = \"{}\"", name, version));
                }
            }
            _ => {}
        }
        false
    }

    fn toggle_expanded(&mut self) {
        if let Some((_, name, _, _)) = self.flat_list.get(self.selected) {
            let name = name.clone();
            if self.expanded.contains(&name) {
                self.expanded.remove(&name);
            } else {
                self.expanded.insert(name);
            }
            self.rebuild_flat_list();
        }
    }

    fn expand_all(&mut self) {
        fn collect_names(deps: &[Dependency], names: &mut std::collections::HashSet<String>) {
            for dep in deps {
                names.insert(dep.name.clone());
                collect_names(&dep.deps, names);
            }
        }
        collect_names(&self.tree, &mut self.expanded);
        self.rebuild_flat_list();
    }

    fn collapse_all(&mut self) {
        self.expanded.clear();
        self.rebuild_flat_list();
    }

    fn rebuild_flat_list(&mut self) {
        self.flat_list.clear();
        self.add_to_flat_list(&self.tree.clone(), 0);
        self.selected = self.selected.min(self.flat_list.len().saturating_sub(1));
    }

    fn add_to_flat_list(&mut self, deps: &[Dependency], depth: usize) {
        for dep in deps {
            if dep.dev && !self.show_dev {
                continue;
            }

            let expanded = self.expanded.contains(&dep.name);
            self.flat_list.push((depth, dep.name.clone(), dep.version.clone(), expanded));

            if expanded && !dep.deps.is_empty() {
                self.add_to_flat_list(&dep.deps, depth + 1);
            }
        }
    }

    pub fn total_deps(&self) -> usize {
        fn count(deps: &[Dependency]) -> usize {
            deps.iter().map(|d| 1 + count(&d.deps)).sum()
        }
        count(&self.tree)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav Enter:expand/collapse e:expand-all c:collapse-all d:toggle-dev q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_deps() -> Vec<Dependency> {
    vec![
        Dependency {
            name: "ratatui".to_string(),
            version: "0.29.0".to_string(),
            dev: false,
            optional: false,
            deps: vec![
                Dependency {
                    name: "crossterm".to_string(),
                    version: "0.28.0".to_string(),
                    dev: false,
                    optional: false,
                    deps: vec![
                        Dependency {
                            name: "bitflags".to_string(),
                            version: "2.0".to_string(),
                            dev: false,
                            optional: false,
                            deps: vec![],
                        },
                        Dependency {
                            name: "parking_lot".to_string(),
                            version: "0.12".to_string(),
                            dev: false,
                            optional: false,
                            deps: vec![],
                        },
                    ],
                },
                Dependency {
                    name: "unicode-width".to_string(),
                    version: "0.1".to_string(),
                    dev: false,
                    optional: false,
                    deps: vec![],
                },
            ],
        },
        Dependency {
            name: "tokio".to_string(),
            version: "1.40.0".to_string(),
            dev: false,
            optional: false,
            deps: vec![
                Dependency {
                    name: "mio".to_string(),
                    version: "1.0".to_string(),
                    dev: false,
                    optional: false,
                    deps: vec![],
                },
                Dependency {
                    name: "pin-project-lite".to_string(),
                    version: "0.2".to_string(),
                    dev: false,
                    optional: false,
                    deps: vec![],
                },
            ],
        },
        Dependency {
            name: "serde".to_string(),
            version: "1.0.210".to_string(),
            dev: false,
            optional: false,
            deps: vec![
                Dependency {
                    name: "serde_derive".to_string(),
                    version: "1.0.210".to_string(),
                    dev: false,
                    optional: true,
                    deps: vec![],
                },
            ],
        },
        Dependency {
            name: "insta".to_string(),
            version: "1.40".to_string(),
            dev: true,
            optional: false,
            deps: vec![
                Dependency {
                    name: "similar".to_string(),
                    version: "2.0".to_string(),
                    dev: true,
                    optional: false,
                    deps: vec![],
                },
            ],
        },
    ]
}
