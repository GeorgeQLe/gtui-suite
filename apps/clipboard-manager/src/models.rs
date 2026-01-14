use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: Uuid,
    pub content: String,
    pub content_type: ContentType,
    pub category: Option<String>,
    pub pinned: bool,
    pub favorite: bool,
    pub use_count: u32,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub source_app: Option<String>,
}

impl ClipboardEntry {
    pub fn new(content: String) -> Self {
        let now = Utc::now();
        let content_type = ContentType::detect(&content);
        Self {
            id: Uuid::new_v4(),
            content,
            content_type,
            category: None,
            pinned: false,
            favorite: false,
            use_count: 0,
            created_at: now,
            last_used: now,
            source_app: None,
        }
    }

    pub fn preview(&self, max_len: usize) -> String {
        let first_line = self.content.lines().next().unwrap_or("");
        if first_line.len() > max_len {
            format!("{}...", &first_line[..max_len.saturating_sub(3)])
        } else if self.content.lines().count() > 1 {
            format!("{}...", first_line)
        } else {
            first_line.to_string()
        }
    }

    pub fn line_count(&self) -> usize {
        self.content.lines().count()
    }

    pub fn char_count(&self) -> usize {
        self.content.chars().count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Code,
    Url,
    Email,
    Path,
    Json,
    Xml,
    Command,
    Password,
}

impl ContentType {
    pub fn detect(content: &str) -> Self {
        let trimmed = content.trim();

        // Check for URL
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return ContentType::Url;
        }

        // Check for email
        if trimmed.contains('@') && trimmed.contains('.') && !trimmed.contains(' ') {
            return ContentType::Email;
        }

        // Check for file path
        if trimmed.starts_with('/') || trimmed.starts_with("~/") || trimmed.starts_with("./") {
            return ContentType::Path;
        }

        // Check for Windows path
        if trimmed.len() > 2 && trimmed.chars().nth(1) == Some(':') {
            return ContentType::Path;
        }

        // Check for JSON
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
                return ContentType::Json;
            }
        }

        // Check for XML/HTML
        if trimmed.starts_with('<') && trimmed.ends_with('>') {
            return ContentType::Xml;
        }

        // Check for shell command patterns
        if trimmed.starts_with("$ ")
            || trimmed.starts_with("# ")
            || trimmed.starts_with("sudo ")
            || trimmed.starts_with("git ")
            || trimmed.starts_with("docker ")
            || trimmed.starts_with("npm ")
            || trimmed.starts_with("cargo ")
        {
            return ContentType::Command;
        }

        // Check for code patterns
        if trimmed.contains("fn ")
            || trimmed.contains("function ")
            || trimmed.contains("def ")
            || trimmed.contains("class ")
            || trimmed.contains("const ")
            || trimmed.contains("let ")
            || trimmed.contains("import ")
            || trimmed.contains("pub ")
        {
            return ContentType::Code;
        }

        ContentType::Text
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "Text",
            ContentType::Code => "Code",
            ContentType::Url => "URL",
            ContentType::Email => "Email",
            ContentType::Path => "Path",
            ContentType::Json => "JSON",
            ContentType::Xml => "XML",
            ContentType::Command => "Command",
            ContentType::Password => "Password",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ContentType::Text => "📝",
            ContentType::Code => "💻",
            ContentType::Url => "🔗",
            ContentType::Email => "📧",
            ContentType::Path => "📁",
            ContentType::Json => "🔧",
            ContentType::Xml => "📄",
            ContentType::Command => "⌨️",
            ContentType::Password => "🔒",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Category {
    pub name: String,
    pub color: CategoryColor,
    pub count: usize,
}

impl Category {
    pub fn new(name: &str, color: CategoryColor) -> Self {
        Self {
            name: name.to_string(),
            color,
            count: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryColor {
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
}

impl CategoryColor {
    pub fn to_ratatui_color(&self) -> ratatui::style::Color {
        match self {
            CategoryColor::Red => ratatui::style::Color::Red,
            CategoryColor::Green => ratatui::style::Color::Green,
            CategoryColor::Blue => ratatui::style::Color::Blue,
            CategoryColor::Yellow => ratatui::style::Color::Yellow,
            CategoryColor::Magenta => ratatui::style::Color::Magenta,
            CategoryColor::Cyan => ratatui::style::Color::Cyan,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClipboardStats {
    pub total_entries: usize,
    pub pinned_count: usize,
    pub favorite_count: usize,
    pub total_chars: usize,
    pub categories: Vec<(String, usize)>,
}

impl ClipboardStats {
    pub fn compute(entries: &[ClipboardEntry]) -> Self {
        let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for entry in entries {
            if let Some(ref cat) = entry.category {
                *categories.entry(cat.clone()).or_insert(0) += 1;
            }
        }

        Self {
            total_entries: entries.len(),
            pinned_count: entries.iter().filter(|e| e.pinned).count(),
            favorite_count: entries.iter().filter(|e| e.favorite).count(),
            total_chars: entries.iter().map(|e| e.char_count()).sum(),
            categories: categories.into_iter().collect(),
        }
    }
}
