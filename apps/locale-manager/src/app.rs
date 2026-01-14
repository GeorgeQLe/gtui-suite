use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Locale {
    pub code: String,
    pub name: String,
    pub charset: String,
    pub enabled: bool,
}

pub struct App {
    pub locales: Vec<Locale>,
    pub selected: usize,
    pub current_locale: String,
    pub modified: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            locales: vec![
                Locale { code: "en_US.UTF-8".into(), name: "English (US)".into(), charset: "UTF-8".into(), enabled: true },
                Locale { code: "en_GB.UTF-8".into(), name: "English (UK)".into(), charset: "UTF-8".into(), enabled: false },
                Locale { code: "de_DE.UTF-8".into(), name: "German".into(), charset: "UTF-8".into(), enabled: false },
                Locale { code: "fr_FR.UTF-8".into(), name: "French".into(), charset: "UTF-8".into(), enabled: false },
                Locale { code: "es_ES.UTF-8".into(), name: "Spanish".into(), charset: "UTF-8".into(), enabled: false },
                Locale { code: "ja_JP.UTF-8".into(), name: "Japanese".into(), charset: "UTF-8".into(), enabled: false },
                Locale { code: "zh_CN.UTF-8".into(), name: "Chinese (Simplified)".into(), charset: "UTF-8".into(), enabled: false },
                Locale { code: "ru_RU.UTF-8".into(), name: "Russian".into(), charset: "UTF-8".into(), enabled: false },
            ],
            selected: 0,
            current_locale: "en_US.UTF-8".into(),
            modified: false,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.locales.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char(' ') | KeyCode::Enter => {
                if let Some(locale) = self.locales.get_mut(self.selected) {
                    locale.enabled = !locale.enabled;
                    self.modified = true;
                }
            },
            KeyCode::Char('d') => {
                if let Some(locale) = self.locales.get(self.selected) {
                    self.current_locale = locale.code.clone();
                    self.status_message = Some(format!("Set default locale: {}", locale.code));
                    self.modified = true;
                }
            },
            KeyCode::Char('g') => self.status_message = Some("Would generate locales...".into()),
            KeyCode::Char('s') => { self.modified = false; self.status_message = Some("Saved locale configuration".into()); },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| format!("{}j/k:nav space:toggle d:set-default g:generate s:save q:quit", if self.modified { "[*] " } else { "" }))
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
