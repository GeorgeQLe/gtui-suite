//! Deterministic fixtures and random generators for testing.

use std::collections::HashMap;

/// Collection of deterministic test fixtures.
pub struct Fixtures;

impl Fixtures {
    /// 10-row sample table with name, age, city columns.
    pub fn sample_table_data() -> Vec<Vec<String>> {
        vec![
            vec!["Alice".to_string(), "28".to_string(), "New York".to_string()],
            vec!["Bob".to_string(), "34".to_string(), "Los Angeles".to_string()],
            vec!["Charlie".to_string(), "22".to_string(), "Chicago".to_string()],
            vec!["Diana".to_string(), "45".to_string(), "Houston".to_string()],
            vec!["Eve".to_string(), "31".to_string(), "Phoenix".to_string()],
            vec!["Frank".to_string(), "29".to_string(), "Philadelphia".to_string()],
            vec!["Grace".to_string(), "38".to_string(), "San Antonio".to_string()],
            vec!["Henry".to_string(), "26".to_string(), "San Diego".to_string()],
            vec!["Ivy".to_string(), "33".to_string(), "Dallas".to_string()],
            vec!["Jack".to_string(), "41".to_string(), "San Jose".to_string()],
        ]
    }

    /// Column headers for sample table.
    pub fn sample_table_headers() -> Vec<String> {
        vec!["Name".to_string(), "Age".to_string(), "City".to_string()]
    }

    /// 3-level sample tree with ~20 nodes.
    pub fn sample_tree() -> SimpleTreeNode {
        SimpleTreeNode::branch("Root", vec![
            SimpleTreeNode::branch("Documents", vec![
                SimpleTreeNode::leaf("resume.pdf"),
                SimpleTreeNode::leaf("cover_letter.docx"),
                SimpleTreeNode::branch("Projects", vec![
                    SimpleTreeNode::leaf("project_a.txt"),
                    SimpleTreeNode::leaf("project_b.txt"),
                ]),
            ]),
            SimpleTreeNode::branch("Pictures", vec![
                SimpleTreeNode::leaf("vacation.jpg"),
                SimpleTreeNode::leaf("family.png"),
                SimpleTreeNode::branch("Screenshots", vec![
                    SimpleTreeNode::leaf("screenshot1.png"),
                    SimpleTreeNode::leaf("screenshot2.png"),
                ]),
            ]),
            SimpleTreeNode::branch("Music", vec![
                SimpleTreeNode::leaf("song1.mp3"),
                SimpleTreeNode::leaf("song2.mp3"),
                SimpleTreeNode::leaf("song3.mp3"),
            ]),
        ])
    }

    /// Contact form with 5 fields.
    pub fn sample_form_fields() -> Vec<FormField> {
        vec![
            FormField {
                name: "name".to_string(),
                label: "Full Name".to_string(),
                field_type: FieldType::Text,
                required: true,
                default_value: String::new(),
            },
            FormField {
                name: "email".to_string(),
                label: "Email Address".to_string(),
                field_type: FieldType::Email,
                required: true,
                default_value: String::new(),
            },
            FormField {
                name: "phone".to_string(),
                label: "Phone Number".to_string(),
                field_type: FieldType::Phone,
                required: false,
                default_value: String::new(),
            },
            FormField {
                name: "message".to_string(),
                label: "Message".to_string(),
                field_type: FieldType::TextArea,
                required: true,
                default_value: String::new(),
            },
            FormField {
                name: "subscribe".to_string(),
                label: "Subscribe to newsletter".to_string(),
                field_type: FieldType::Checkbox,
                required: false,
                default_value: "false".to_string(),
            },
        ]
    }

    /// 100 sample log lines with timestamps.
    pub fn sample_logs() -> Vec<LogEntry> {
        let levels = ["INFO", "DEBUG", "WARN", "ERROR"];
        let messages = [
            "Server started successfully",
            "Processing request",
            "Database query executed",
            "Cache hit",
            "Cache miss",
            "Connection established",
            "Request completed",
            "Configuration loaded",
            "Task scheduled",
            "Background job completed",
        ];

        let mut logs = Vec::with_capacity(100);
        for i in 0..100 {
            let level = levels[i % levels.len()];
            let message = messages[i % messages.len()];
            logs.push(LogEntry {
                timestamp: format!("2024-01-15 10:{:02}:{:02}", i / 60, i % 60),
                level: level.to_string(),
                message: format!("{} (request #{})", message, i),
                source: format!("module{}", i % 5),
            });
        }
        logs
    }

    /// Table with specified dimensions (deterministic content).
    pub fn table_data(rows: usize, cols: usize) -> Vec<Vec<String>> {
        let mut data = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut row_data = Vec::with_capacity(cols);
            for col in 0..cols {
                row_data.push(format!("R{}C{}", row, col));
            }
            data.push(row_data);
        }
        data
    }

    /// Tree with specified depth and branching factor.
    pub fn tree(depth: usize, branching: usize) -> SimpleTreeNode {
        fn build(current_depth: usize, max_depth: usize, branching: usize, path: &str) -> SimpleTreeNode {
            if current_depth >= max_depth {
                SimpleTreeNode::leaf(&format!("leaf_{}", path))
            } else {
                let children: Vec<_> = (0..branching)
                    .map(|i| {
                        let child_path = format!("{}_{}", path, i);
                        build(current_depth + 1, max_depth, branching, &child_path)
                    })
                    .collect();
                SimpleTreeNode::branch(&format!("node_{}", path), children)
            }
        }

        build(0, depth, branching, "0")
    }

    /// Sample key-value configuration.
    pub fn sample_config() -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("theme".to_string(), "dark".to_string());
        config.insert("font_size".to_string(), "14".to_string());
        config.insert("line_numbers".to_string(), "true".to_string());
        config.insert("word_wrap".to_string(), "false".to_string());
        config.insert("tab_size".to_string(), "4".to_string());
        config.insert("auto_save".to_string(), "true".to_string());
        config.insert("auto_save_interval".to_string(), "60".to_string());
        config
    }

    /// Sample command list for command palette testing.
    pub fn sample_commands() -> Vec<CommandInfo> {
        vec![
            CommandInfo {
                id: "file.new".to_string(),
                label: "New File".to_string(),
                description: Some("Create a new file".to_string()),
                shortcut: Some("Ctrl+N".to_string()),
                category: "File".to_string(),
            },
            CommandInfo {
                id: "file.open".to_string(),
                label: "Open File".to_string(),
                description: Some("Open an existing file".to_string()),
                shortcut: Some("Ctrl+O".to_string()),
                category: "File".to_string(),
            },
            CommandInfo {
                id: "file.save".to_string(),
                label: "Save File".to_string(),
                description: Some("Save the current file".to_string()),
                shortcut: Some("Ctrl+S".to_string()),
                category: "File".to_string(),
            },
            CommandInfo {
                id: "edit.undo".to_string(),
                label: "Undo".to_string(),
                description: Some("Undo last action".to_string()),
                shortcut: Some("Ctrl+Z".to_string()),
                category: "Edit".to_string(),
            },
            CommandInfo {
                id: "edit.redo".to_string(),
                label: "Redo".to_string(),
                description: Some("Redo last undone action".to_string()),
                shortcut: Some("Ctrl+Y".to_string()),
                category: "Edit".to_string(),
            },
            CommandInfo {
                id: "view.theme".to_string(),
                label: "Change Theme".to_string(),
                description: Some("Switch color theme".to_string()),
                shortcut: None,
                category: "View".to_string(),
            },
            CommandInfo {
                id: "view.zoom_in".to_string(),
                label: "Zoom In".to_string(),
                description: Some("Increase font size".to_string()),
                shortcut: Some("Ctrl++".to_string()),
                category: "View".to_string(),
            },
            CommandInfo {
                id: "view.zoom_out".to_string(),
                label: "Zoom Out".to_string(),
                description: Some("Decrease font size".to_string()),
                shortcut: Some("Ctrl+-".to_string()),
                category: "View".to_string(),
            },
            CommandInfo {
                id: "help.docs".to_string(),
                label: "Documentation".to_string(),
                description: Some("Open documentation".to_string()),
                shortcut: Some("F1".to_string()),
                category: "Help".to_string(),
            },
            CommandInfo {
                id: "help.about".to_string(),
                label: "About".to_string(),
                description: Some("Show application info".to_string()),
                shortcut: None,
                category: "Help".to_string(),
            },
        ]
    }

    /// Long text for text area / editor testing.
    pub fn sample_long_text() -> String {
        r#"Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.
Duis aute irure dolor in reprehenderit in voluptate velit esse.
Excepteur sint occaecat cupidatat non proident, sunt in culpa.
Qui officia deserunt mollit anim id est laborum.

Curabitur pretium tincidunt lacus. Nulla gravida orci a odio.
Nullam varius, turpis et commodo pharetra, est eros bibendum elit.
Proin neque massa, cursus ut, gravida ut, lobortis eget, lacus.
Sed diam. Praesent fermentum tempor tellus.

Nullam tempus. Mauris ac felis vel velit tristique imperdiet.
Donec at pede. Etiam vel neque nec dui dignissim bibendum.
Vivamus id enim. Phasellus neque orci, porta a, aliquet quis.
Semper quis, nisi. Suspendisse mauris. Fusce accumsan mollis eros.
Pellentesque a diam sit amet mi ullamcorper vehicula."#
            .to_string()
    }
}

/// A simple tree node for testing.
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleTreeNode {
    pub name: String,
    pub children: Vec<SimpleTreeNode>,
}

impl SimpleTreeNode {
    /// Create a leaf node.
    pub fn leaf(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: Vec::new(),
        }
    }

    /// Create a branch node.
    pub fn branch(name: &str, children: Vec<SimpleTreeNode>) -> Self {
        Self {
            name: name.to_string(),
            children,
        }
    }

    /// Check if this is a leaf node.
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Count all nodes.
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }

    /// Get the depth of the tree.
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }
}

/// A form field definition.
#[derive(Debug, Clone, PartialEq)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default_value: String,
}

/// Field types for forms.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Text,
    Email,
    Phone,
    TextArea,
    Checkbox,
    Select,
    Number,
    Date,
}

/// A log entry.
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
}

/// Command information for command palette.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandInfo {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub shortcut: Option<String>,
    pub category: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_table_data() {
        let data = Fixtures::sample_table_data();
        assert_eq!(data.len(), 10);
        assert_eq!(data[0].len(), 3);
    }

    #[test]
    fn test_sample_tree() {
        let tree = Fixtures::sample_tree();
        assert_eq!(tree.name, "Root");
        assert_eq!(tree.children.len(), 3);
        assert!(tree.count() >= 15);
    }

    #[test]
    fn test_sample_form_fields() {
        let fields = Fixtures::sample_form_fields();
        assert_eq!(fields.len(), 5);
        assert!(fields.iter().any(|f| f.name == "email"));
    }

    #[test]
    fn test_sample_logs() {
        let logs = Fixtures::sample_logs();
        assert_eq!(logs.len(), 100);
        assert!(logs.iter().any(|l| l.level == "ERROR"));
    }

    #[test]
    fn test_sized_table() {
        let data = Fixtures::table_data(5, 3);
        assert_eq!(data.len(), 5);
        assert_eq!(data[0].len(), 3);
        assert_eq!(data[2][1], "R2C1");
    }

    #[test]
    fn test_sized_tree() {
        let tree = Fixtures::tree(3, 2);
        // depth parameter means branching levels, plus 1 for leaves = 4 total depth
        assert_eq!(tree.depth(), 4);
    }

    #[test]
    fn test_sample_config() {
        let config = Fixtures::sample_config();
        assert!(config.contains_key("theme"));
        assert_eq!(config.get("theme"), Some(&"dark".to_string()));
    }

    #[test]
    fn test_sample_commands() {
        let commands = Fixtures::sample_commands();
        assert!(commands.len() >= 10);
        assert!(commands.iter().any(|c| c.id == "file.save"));
    }
}
