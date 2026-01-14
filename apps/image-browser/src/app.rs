use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct ImageLayer {
    pub id: String,
    pub command: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct ContainerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: u64,
    pub created: String,
    pub layers: Vec<ImageLayer>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode { List, Layers }

pub struct App {
    pub images: Vec<ContainerImage>,
    pub selected: usize,
    pub view_mode: ViewMode,
    pub layer_scroll: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            images: vec![
                ContainerImage {
                    id: "sha256:abc123".into(), repository: "nginx".into(), tag: "latest".into(), size: 142_000_000, created: "2 weeks ago".into(),
                    layers: vec![
                        ImageLayer { id: "sha256:layer1".into(), command: "ADD file:xxx in /".into(), size: 77_000_000 },
                        ImageLayer { id: "sha256:layer2".into(), command: "CMD [\"bash\"]".into(), size: 0 },
                        ImageLayer { id: "sha256:layer3".into(), command: "RUN apt-get update && apt-get install...".into(), size: 50_000_000 },
                        ImageLayer { id: "sha256:layer4".into(), command: "COPY nginx.conf /etc/nginx/".into(), size: 15_000_000 },
                    ],
                },
                ContainerImage {
                    id: "sha256:def456".into(), repository: "postgres".into(), tag: "15".into(), size: 379_000_000, created: "3 weeks ago".into(),
                    layers: vec![
                        ImageLayer { id: "sha256:layerA".into(), command: "ADD file:xxx in /".into(), size: 77_000_000 },
                        ImageLayer { id: "sha256:layerB".into(), command: "RUN apt-get update...".into(), size: 200_000_000 },
                        ImageLayer { id: "sha256:layerC".into(), command: "ENV PGDATA=/var/lib/postgresql/data".into(), size: 0 },
                    ],
                },
                ContainerImage {
                    id: "sha256:ghi789".into(), repository: "redis".into(), tag: "7-alpine".into(), size: 30_000_000, created: "1 month ago".into(),
                    layers: vec![
                        ImageLayer { id: "sha256:layerX".into(), command: "ADD file:xxx in /".into(), size: 7_000_000 },
                        ImageLayer { id: "sha256:layerY".into(), command: "RUN apk add redis".into(), size: 23_000_000 },
                    ],
                },
                ContainerImage {
                    id: "sha256:jkl012".into(), repository: "myapp".into(), tag: "v1.2.3".into(), size: 256_000_000, created: "2 days ago".into(),
                    layers: vec![
                        ImageLayer { id: "sha256:base".into(), command: "FROM node:18".into(), size: 150_000_000 },
                        ImageLayer { id: "sha256:deps".into(), command: "RUN npm install".into(), size: 80_000_000 },
                        ImageLayer { id: "sha256:app".into(), command: "COPY . /app".into(), size: 26_000_000 },
                    ],
                },
            ],
            selected: 0,
            view_mode: ViewMode::List,
            layer_scroll: 0,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.view_mode == ViewMode::Layers {
                    self.view_mode = ViewMode::List;
                } else {
                    return true;
                }
            },
            KeyCode::Char('j') | KeyCode::Down => {
                if self.view_mode == ViewMode::List {
                    if self.selected < self.images.len().saturating_sub(1) { self.selected += 1; }
                } else {
                    if let Some(img) = self.images.get(self.selected) {
                        if self.layer_scroll < img.layers.len().saturating_sub(1) { self.layer_scroll += 1; }
                    }
                }
            },
            KeyCode::Char('k') | KeyCode::Up => {
                if self.view_mode == ViewMode::List {
                    self.selected = self.selected.saturating_sub(1);
                } else {
                    self.layer_scroll = self.layer_scroll.saturating_sub(1);
                }
            },
            KeyCode::Enter | KeyCode::Char('l') => {
                self.view_mode = ViewMode::Layers;
                self.layer_scroll = 0;
            },
            KeyCode::Char('r') => self.status_message = Some("Would remove image...".into()),
            KeyCode::Char('t') => self.status_message = Some("Would tag image...".into()),
            KeyCode::Char('p') => self.status_message = Some("Would push image...".into()),
            KeyCode::Char('h') => self.status_message = Some("Would show history...".into()),
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| {
            match self.view_mode {
                ViewMode::List => "j/k:nav enter:layers r:remove t:tag p:push q:quit".into(),
                ViewMode::Layers => "j/k:scroll esc:back q:quit".into(),
            }
        })
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
