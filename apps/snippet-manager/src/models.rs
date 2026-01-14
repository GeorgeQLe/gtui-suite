use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub code: String,
    pub language: Language,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub use_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Snippet {
    pub fn new(title: &str, code: &str, language: Language) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: String::new(),
            code: code.to_string(),
            language,
            tags: Vec::new(),
            favorite: false,
            use_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn line_count(&self) -> usize {
        self.code.lines().count()
    }

    pub fn preview(&self, max_lines: usize) -> String {
        self.code
            .lines()
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    CSharp,
    Cpp,
    C,
    Ruby,
    Shell,
    Sql,
    Html,
    Css,
    Json,
    Yaml,
    Toml,
    Markdown,
    Other,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::CSharp => "C#",
            Language::Cpp => "C++",
            Language::C => "C",
            Language::Ruby => "Ruby",
            Language::Shell => "Shell",
            Language::Sql => "SQL",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Json => "JSON",
            Language::Yaml => "YAML",
            Language::Toml => "TOML",
            Language::Markdown => "Markdown",
            Language::Other => "Other",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Python => "py",
            Language::JavaScript => "js",
            Language::TypeScript => "ts",
            Language::Go => "go",
            Language::Java => "java",
            Language::CSharp => "cs",
            Language::Cpp => "cpp",
            Language::C => "c",
            Language::Ruby => "rb",
            Language::Shell => "sh",
            Language::Sql => "sql",
            Language::Html => "html",
            Language::Css => "css",
            Language::Json => "json",
            Language::Yaml => "yaml",
            Language::Toml => "toml",
            Language::Markdown => "md",
            Language::Other => "txt",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Language::Rust => "🦀",
            Language::Python => "🐍",
            Language::JavaScript => "📜",
            Language::TypeScript => "📘",
            Language::Go => "🐹",
            Language::Java => "☕",
            Language::CSharp => "🎯",
            Language::Cpp => "⚡",
            Language::C => "©️",
            Language::Ruby => "💎",
            Language::Shell => "🐚",
            Language::Sql => "🗄️",
            Language::Html => "🌐",
            Language::Css => "🎨",
            Language::Json => "📋",
            Language::Yaml => "📄",
            Language::Toml => "⚙️",
            Language::Markdown => "📝",
            Language::Other => "📄",
        }
    }

    pub fn all() -> Vec<Language> {
        vec![
            Language::Rust,
            Language::Python,
            Language::JavaScript,
            Language::TypeScript,
            Language::Go,
            Language::Java,
            Language::CSharp,
            Language::Cpp,
            Language::C,
            Language::Ruby,
            Language::Shell,
            Language::Sql,
            Language::Html,
            Language::Css,
            Language::Json,
            Language::Yaml,
            Language::Toml,
            Language::Markdown,
            Language::Other,
        ]
    }

    pub fn detect(code: &str) -> Language {
        let first_line = code.lines().next().unwrap_or("");

        // Check shebang
        if first_line.starts_with("#!") {
            if first_line.contains("python") {
                return Language::Python;
            } else if first_line.contains("node") {
                return Language::JavaScript;
            } else if first_line.contains("ruby") {
                return Language::Ruby;
            } else if first_line.contains("bash") || first_line.contains("sh") {
                return Language::Shell;
            }
        }

        // Check patterns
        if code.contains("fn main()") || code.contains("impl ") || code.contains("pub fn") {
            return Language::Rust;
        }
        if code.contains("def ") && code.contains(":") {
            return Language::Python;
        }
        if code.contains("function ") || code.contains("const ") && code.contains("=>") {
            return Language::JavaScript;
        }
        if code.contains("interface ") || code.contains(": string") || code.contains(": number") {
            return Language::TypeScript;
        }
        if code.contains("func ") && code.contains("package ") {
            return Language::Go;
        }
        if code.contains("public class ") || code.contains("private void ") {
            return Language::Java;
        }
        if code.starts_with("SELECT ") || code.starts_with("INSERT ") || code.starts_with("CREATE TABLE") {
            return Language::Sql;
        }
        if code.starts_with("<!DOCTYPE") || code.starts_with("<html") {
            return Language::Html;
        }
        if code.starts_with("{") && code.contains("\":") {
            return Language::Json;
        }
        if code.contains("---") || (code.contains(":") && !code.contains("{")) {
            return Language::Yaml;
        }
        if code.contains("[package]") || code.contains("[dependencies]") {
            return Language::Toml;
        }

        Language::Other
    }
}

#[derive(Debug, Clone)]
pub struct SnippetStats {
    pub total: usize,
    pub favorites: usize,
    pub by_language: Vec<(Language, usize)>,
    pub popular_tags: Vec<(String, usize)>,
}

impl SnippetStats {
    pub fn compute(snippets: &[Snippet]) -> Self {
        use std::collections::HashMap;

        let mut by_language: HashMap<Language, usize> = HashMap::new();
        let mut tags: HashMap<String, usize> = HashMap::new();

        for snippet in snippets {
            *by_language.entry(snippet.language).or_insert(0) += 1;
            for tag in &snippet.tags {
                *tags.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        let mut lang_vec: Vec<_> = by_language.into_iter().collect();
        lang_vec.sort_by(|a, b| b.1.cmp(&a.1));

        let mut tag_vec: Vec<_> = tags.into_iter().collect();
        tag_vec.sort_by(|a, b| b.1.cmp(&a.1));
        tag_vec.truncate(10);

        Self {
            total: snippets.len(),
            favorites: snippets.iter().filter(|s| s.favorite).count(),
            by_language: lang_vec,
            popular_tags: tag_vec,
        }
    }
}
