use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone)]
pub struct S3Bucket {
    pub name: String,
    pub region: String,
    pub created: String,
    pub objects: u64,
    pub size: String,
}

#[derive(Clone)]
pub struct S3Object {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
    pub storage_class: String,
    pub is_folder: bool,
}

pub enum View {
    Buckets,
    Objects,
}

pub struct App {
    pub buckets: Vec<S3Bucket>,
    pub objects: Vec<S3Object>,
    pub selected_bucket: usize,
    pub selected_object: usize,
    pub current_view: View,
    pub current_path: Vec<String>,
    pub show_help: bool,
    pub show_properties: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            buckets: vec![
                S3Bucket {
                    name: "production-assets".to_string(),
                    region: "us-east-1".to_string(),
                    created: "2023-01-15".to_string(),
                    objects: 15420,
                    size: "2.3 TB".to_string(),
                },
                S3Bucket {
                    name: "backup-data".to_string(),
                    region: "us-west-2".to_string(),
                    created: "2022-06-20".to_string(),
                    objects: 8932,
                    size: "850 GB".to_string(),
                },
                S3Bucket {
                    name: "logs-archive".to_string(),
                    region: "eu-west-1".to_string(),
                    created: "2023-03-10".to_string(),
                    objects: 245000,
                    size: "5.6 TB".to_string(),
                },
                S3Bucket {
                    name: "staging-uploads".to_string(),
                    region: "us-east-1".to_string(),
                    created: "2023-08-05".to_string(),
                    objects: 1230,
                    size: "45 GB".to_string(),
                },
                S3Bucket {
                    name: "ml-training-data".to_string(),
                    region: "us-east-2".to_string(),
                    created: "2023-11-01".to_string(),
                    objects: 5600,
                    size: "1.2 TB".to_string(),
                },
            ],
            objects: Vec::new(),
            selected_bucket: 0,
            selected_object: 0,
            current_view: View::Buckets,
            current_path: Vec::new(),
            show_help: false,
            show_properties: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            self.show_help = false;
            return false;
        }

        if self.show_properties {
            if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                self.show_properties = false;
            }
            return false;
        }

        match &self.current_view {
            View::Buckets => self.handle_buckets_key(key),
            View::Objects => self.handle_objects_key(key),
        }
    }

    fn handle_buckets_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_bucket < self.buckets.len().saturating_sub(1) {
                    self.selected_bucket += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_bucket = self.selected_bucket.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                self.enter_bucket();
            }
            KeyCode::Char('i') => {
                self.show_properties = true;
            }
            _ => {}
        }
        false
    }

    fn handle_objects_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_object < self.objects.len().saturating_sub(1) {
                    self.selected_object += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_object = self.selected_object.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                if !self.objects.is_empty() && self.objects[self.selected_object].is_folder {
                    self.enter_folder();
                }
            }
            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => {
                self.go_back();
            }
            KeyCode::Esc => {
                if self.current_path.is_empty() {
                    self.current_view = View::Buckets;
                    self.objects.clear();
                } else {
                    self.go_back();
                }
            }
            KeyCode::Char('d') => {
                // Download (demo)
            }
            KeyCode::Char('D') => {
                // Delete (demo)
            }
            KeyCode::Char('i') => {
                self.show_properties = true;
            }
            _ => {}
        }
        false
    }

    fn enter_bucket(&mut self) {
        self.current_view = View::Objects;
        self.current_path.clear();
        self.selected_object = 0;
        self.load_objects();
    }

    fn enter_folder(&mut self) {
        let folder = &self.objects[self.selected_object];
        self.current_path.push(folder.key.trim_end_matches('/').to_string());
        self.selected_object = 0;
        self.load_objects();
    }

    fn go_back(&mut self) {
        if self.current_path.is_empty() {
            self.current_view = View::Buckets;
            self.objects.clear();
        } else {
            self.current_path.pop();
            self.selected_object = 0;
            self.load_objects();
        }
    }

    fn load_objects(&mut self) {
        // Demo data based on current bucket and path
        let bucket = &self.buckets[self.selected_bucket];

        self.objects = match (bucket.name.as_str(), self.current_path.len()) {
            ("production-assets", 0) => vec![
                S3Object {
                    key: "images/".to_string(),
                    size: 0,
                    last_modified: "-".to_string(),
                    storage_class: "FOLDER".to_string(),
                    is_folder: true,
                },
                S3Object {
                    key: "videos/".to_string(),
                    size: 0,
                    last_modified: "-".to_string(),
                    storage_class: "FOLDER".to_string(),
                    is_folder: true,
                },
                S3Object {
                    key: "documents/".to_string(),
                    size: 0,
                    last_modified: "-".to_string(),
                    storage_class: "FOLDER".to_string(),
                    is_folder: true,
                },
                S3Object {
                    key: "index.html".to_string(),
                    size: 4520,
                    last_modified: "2024-01-15 10:30".to_string(),
                    storage_class: "STANDARD".to_string(),
                    is_folder: false,
                },
                S3Object {
                    key: "config.json".to_string(),
                    size: 1024,
                    last_modified: "2024-01-14 09:15".to_string(),
                    storage_class: "STANDARD".to_string(),
                    is_folder: false,
                },
            ],
            ("production-assets", 1) => vec![
                S3Object {
                    key: "logo.png".to_string(),
                    size: 25600,
                    last_modified: "2024-01-10 14:20".to_string(),
                    storage_class: "STANDARD".to_string(),
                    is_folder: false,
                },
                S3Object {
                    key: "banner.jpg".to_string(),
                    size: 156000,
                    last_modified: "2024-01-12 08:45".to_string(),
                    storage_class: "STANDARD".to_string(),
                    is_folder: false,
                },
                S3Object {
                    key: "icon.svg".to_string(),
                    size: 2048,
                    last_modified: "2024-01-08 16:00".to_string(),
                    storage_class: "STANDARD".to_string(),
                    is_folder: false,
                },
            ],
            _ => vec![
                S3Object {
                    key: "data/".to_string(),
                    size: 0,
                    last_modified: "-".to_string(),
                    storage_class: "FOLDER".to_string(),
                    is_folder: true,
                },
                S3Object {
                    key: "file1.txt".to_string(),
                    size: 1024,
                    last_modified: "2024-01-15 12:00".to_string(),
                    storage_class: "STANDARD".to_string(),
                    is_folder: false,
                },
                S3Object {
                    key: "file2.json".to_string(),
                    size: 2048,
                    last_modified: "2024-01-14 11:30".to_string(),
                    storage_class: "STANDARD".to_string(),
                    is_folder: false,
                },
            ],
        };
    }

    pub fn current_bucket_name(&self) -> &str {
        &self.buckets[self.selected_bucket].name
    }

    pub fn current_path_display(&self) -> String {
        if self.current_path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}/", self.current_path.join("/"))
        }
    }
}

pub fn format_size(size: u64) -> String {
    if size == 0 {
        "-".to_string()
    } else if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
