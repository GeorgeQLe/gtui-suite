use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseCategory {
    Permissive,
    Copyleft,
    Proprietary,
    Unknown,
}

impl LicenseCategory {
    pub fn name(&self) -> &'static str {
        match self {
            LicenseCategory::Permissive => "Permissive",
            LicenseCategory::Copyleft => "Copyleft",
            LicenseCategory::Proprietary => "Proprietary",
            LicenseCategory::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LicenseInfo {
    pub package: String,
    pub version: String,
    pub license: String,
    pub category: LicenseCategory,
}

pub struct App {
    pub licenses: Vec<LicenseInfo>,
    pub selected: usize,
    pub filter: Option<LicenseCategory>,
    pub filtered_indices: Vec<usize>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let licenses = create_demo_licenses();
        let filtered_indices: Vec<usize> = (0..licenses.len()).collect();
        Self {
            licenses,
            selected: 0,
            filter: None,
            filtered_indices,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.filtered_indices.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('1') => self.set_filter(Some(LicenseCategory::Permissive)),
            KeyCode::Char('2') => self.set_filter(Some(LicenseCategory::Copyleft)),
            KeyCode::Char('3') => self.set_filter(Some(LicenseCategory::Proprietary)),
            KeyCode::Char('4') => self.set_filter(Some(LicenseCategory::Unknown)),
            KeyCode::Char('0') | KeyCode::Char('a') => self.set_filter(None),
            KeyCode::Char('y') => {
                if let Some(&idx) = self.filtered_indices.get(self.selected) {
                    if let Some(lic) = self.licenses.get(idx) {
                        self.status_message = Some(format!("Copied: {} - {}", lic.package, lic.license));
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn set_filter(&mut self, filter: Option<LicenseCategory>) {
        self.filter = filter;
        self.update_filtered();
        self.status_message = Some(match filter {
            Some(cat) => format!("Filtered: {}", cat.name()),
            None => "Showing all licenses".to_string(),
        });
    }

    fn update_filtered(&mut self) {
        self.filtered_indices = self.licenses
            .iter()
            .enumerate()
            .filter(|(_, lic)| {
                self.filter.map_or(true, |cat| lic.category == cat)
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = self.selected.min(self.filtered_indices.len().saturating_sub(1));
    }

    pub fn category_counts(&self) -> (usize, usize, usize, usize) {
        let permissive = self.licenses.iter().filter(|l| l.category == LicenseCategory::Permissive).count();
        let copyleft = self.licenses.iter().filter(|l| l.category == LicenseCategory::Copyleft).count();
        let proprietary = self.licenses.iter().filter(|l| l.category == LicenseCategory::Proprietary).count();
        let unknown = self.licenses.iter().filter(|l| l.category == LicenseCategory::Unknown).count();
        (permissive, copyleft, proprietary, unknown)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav 1:permissive 2:copyleft 3:proprietary 4:unknown 0:all y:copy q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_licenses() -> Vec<LicenseInfo> {
    vec![
        LicenseInfo {
            package: "ratatui".to_string(),
            version: "0.29.0".to_string(),
            license: "MIT".to_string(),
            category: LicenseCategory::Permissive,
        },
        LicenseInfo {
            package: "tokio".to_string(),
            version: "1.40.0".to_string(),
            license: "MIT".to_string(),
            category: LicenseCategory::Permissive,
        },
        LicenseInfo {
            package: "serde".to_string(),
            version: "1.0.210".to_string(),
            license: "MIT OR Apache-2.0".to_string(),
            category: LicenseCategory::Permissive,
        },
        LicenseInfo {
            package: "crossterm".to_string(),
            version: "0.28.0".to_string(),
            license: "MIT".to_string(),
            category: LicenseCategory::Permissive,
        },
        LicenseInfo {
            package: "chrono".to_string(),
            version: "0.4.38".to_string(),
            license: "MIT OR Apache-2.0".to_string(),
            category: LicenseCategory::Permissive,
        },
        LicenseInfo {
            package: "linux-kernel-module".to_string(),
            version: "0.1.0".to_string(),
            license: "GPL-2.0".to_string(),
            category: LicenseCategory::Copyleft,
        },
        LicenseInfo {
            package: "gnu-readline".to_string(),
            version: "8.2".to_string(),
            license: "GPL-3.0".to_string(),
            category: LicenseCategory::Copyleft,
        },
        LicenseInfo {
            package: "proprietary-lib".to_string(),
            version: "1.0.0".to_string(),
            license: "Commercial".to_string(),
            category: LicenseCategory::Proprietary,
        },
        LicenseInfo {
            package: "mystery-crate".to_string(),
            version: "0.5.0".to_string(),
            license: "UNKNOWN".to_string(),
            category: LicenseCategory::Unknown,
        },
        LicenseInfo {
            package: "anyhow".to_string(),
            version: "1.0.89".to_string(),
            license: "MIT OR Apache-2.0".to_string(),
            category: LicenseCategory::Permissive,
        },
    ]
}
