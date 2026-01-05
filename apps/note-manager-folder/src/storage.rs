//! File-system based storage for notes.

use crate::models::{Folder, Node, NodeId, Note, SearchResult, TreeItem};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Storage {
    root: PathBuf,
    nodes: HashMap<NodeId, Node>,
    root_id: NodeId,
}

impl Storage {
    pub fn open(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)?;

        let mut storage = Self {
            root: root.clone(),
            nodes: HashMap::new(),
            root_id: String::new(),
        };

        storage.scan_directory()?;
        Ok(storage)
    }

    fn scan_directory(&mut self) -> Result<()> {
        self.nodes.clear();

        // Create root folder
        let root_folder = Folder::new("Notes", self.root.clone(), None);
        self.root_id = root_folder.id.clone();
        self.nodes.insert(root_folder.id.clone(), Node::Folder(root_folder));

        // Recursively scan
        self.scan_dir(&self.root.clone(), self.root_id.clone())?;
        Ok(())
    }

    fn scan_dir(&mut self, dir: &Path, parent_id: NodeId) -> Result<Vec<NodeId>> {
        let mut children = Vec::new();

        let entries = fs::read_dir(dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Skip hidden files
            if name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                let mut folder = Folder::new(&name, path.clone(), Some(parent_id.clone()));
                let folder_id = folder.id.clone();

                // Recursively scan subdirectories
                folder.children = self.scan_dir(&path, folder_id.clone())?;

                children.push(folder_id.clone());
                self.nodes.insert(folder_id, Node::Folder(folder));
            } else if path.extension().map_or(false, |e| e == "md") {
                let title = name.trim_end_matches(".md").to_string();
                let mut note = Note::new(&title, path.clone(), parent_id.clone());

                // Read content
                if let Ok(content) = fs::read_to_string(&path) {
                    note.content = content;
                }

                children.push(note.id.clone());
                self.nodes.insert(note.id.clone(), Node::Note(note));
            }
        }

        // Update parent's children
        if let Some(Node::Folder(folder)) = self.nodes.get_mut(&parent_id) {
            folder.children = children.clone();
        }

        Ok(children)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.scan_directory()
    }

    pub fn root_id(&self) -> &NodeId {
        &self.root_id
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn get_note(&self, id: &NodeId) -> Option<&Note> {
        match self.nodes.get(id) {
            Some(Node::Note(note)) => Some(note),
            _ => None,
        }
    }

    pub fn get_folder(&self, id: &NodeId) -> Option<&Folder> {
        match self.nodes.get(id) {
            Some(Node::Folder(folder)) => Some(folder),
            _ => None,
        }
    }

    pub fn toggle_folder(&mut self, id: &NodeId) {
        if let Some(Node::Folder(folder)) = self.nodes.get_mut(id) {
            folder.expanded = !folder.expanded;
        }
    }

    pub fn expand_folder(&mut self, id: &NodeId) {
        if let Some(Node::Folder(folder)) = self.nodes.get_mut(id) {
            folder.expanded = true;
        }
    }

    pub fn build_tree(&self) -> Vec<TreeItem> {
        let mut items = Vec::new();
        self.build_tree_recursive(&self.root_id, 0, &mut items);
        items
    }

    fn build_tree_recursive(&self, id: &NodeId, depth: usize, items: &mut Vec<TreeItem>) {
        if let Some(node) = self.nodes.get(id) {
            match node {
                Node::Folder(folder) => {
                    // Skip root in display but process children
                    if depth > 0 {
                        items.push(TreeItem {
                            id: folder.id.clone(),
                            name: folder.name.clone(),
                            is_folder: true,
                            depth,
                            expanded: folder.expanded,
                            has_children: !folder.children.is_empty(),
                        });
                    }

                    if folder.expanded || depth == 0 {
                        for child_id in &folder.children {
                            self.build_tree_recursive(child_id, depth + 1, items);
                        }
                    }
                }
                Node::Note(note) => {
                    items.push(TreeItem {
                        id: note.id.clone(),
                        name: note.title.clone(),
                        is_folder: false,
                        depth,
                        expanded: false,
                        has_children: false,
                    });
                }
            }
        }
    }

    pub fn create_folder(&mut self, name: &str, parent_id: &NodeId) -> Result<NodeId> {
        let parent_path = self.get_folder(parent_id)
            .map(|f| f.path.clone())
            .context("Parent folder not found")?;

        let folder_path = parent_path.join(name);
        fs::create_dir(&folder_path)?;

        let folder = Folder::new(name, folder_path, Some(parent_id.clone()));
        let folder_id = folder.id.clone();

        // Update parent
        if let Some(Node::Folder(parent)) = self.nodes.get_mut(parent_id) {
            parent.children.push(folder_id.clone());
        }

        self.nodes.insert(folder_id.clone(), Node::Folder(folder));
        Ok(folder_id)
    }

    pub fn create_note(&mut self, title: &str, parent_id: &NodeId) -> Result<NodeId> {
        let parent_path = self.get_folder(parent_id)
            .map(|f| f.path.clone())
            .context("Parent folder not found")?;

        let filename = format!("{}.md", title);
        let note_path = parent_path.join(&filename);

        let note = Note::new(title, note_path.clone(), parent_id.clone());
        let note_id = note.id.clone();

        // Create the file
        fs::write(&note_path, "")?;

        // Update parent
        if let Some(Node::Folder(parent)) = self.nodes.get_mut(parent_id) {
            parent.children.push(note_id.clone());
        }

        self.nodes.insert(note_id.clone(), Node::Note(note));
        Ok(note_id)
    }

    pub fn save_note(&mut self, id: &NodeId, content: &str) -> Result<()> {
        if let Some(Node::Note(note)) = self.nodes.get_mut(id) {
            note.content = content.to_string();
            note.updated_at = chrono::Utc::now();
            fs::write(&note.path, content)?;
        }
        Ok(())
    }

    pub fn delete_node(&mut self, id: &NodeId) -> Result<()> {
        if let Some(node) = self.nodes.get(id).cloned() {
            let path = node.path().clone();

            // Remove from parent's children
            let parent_id = match &node {
                Node::Folder(f) => f.parent_id.clone(),
                Node::Note(n) => Some(n.parent_id.clone()),
            };

            if let Some(pid) = parent_id {
                if let Some(Node::Folder(parent)) = self.nodes.get_mut(&pid) {
                    parent.children.retain(|c| c != id);
                }
            }

            // Delete from filesystem
            if node.is_folder() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }

            self.nodes.remove(id);
        }
        Ok(())
    }

    pub fn rename_node(&mut self, id: &NodeId, new_name: &str) -> Result<()> {
        if let Some(node) = self.nodes.get_mut(id) {
            let old_path = node.path().clone();
            let new_path = old_path.parent()
                .context("No parent directory")?
                .join(if node.is_folder() {
                    new_name.to_string()
                } else {
                    format!("{}.md", new_name)
                });

            fs::rename(&old_path, &new_path)?;

            match node {
                Node::Folder(f) => {
                    f.name = new_name.to_string();
                    f.path = new_path;
                }
                Node::Note(n) => {
                    n.title = new_name.to_string();
                    n.path = new_path;
                    n.updated_at = chrono::Utc::now();
                }
            }
        }
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for node in self.nodes.values() {
            if let Node::Note(note) = node {
                let title_matches = note.title.to_lowercase().contains(&query_lower);
                let content_lower = note.content.to_lowercase();
                let content_matches = content_lower.contains(&query_lower);

                if title_matches || content_matches {
                    let match_count = content_lower.matches(&query_lower).count()
                        + if title_matches { 1 } else { 0 };

                    let snippet = if content_matches {
                        if let Some(pos) = content_lower.find(&query_lower) {
                            let start = pos.saturating_sub(30);
                            let end = (pos + query.len() + 30).min(note.content.len());
                            format!("...{}...", &note.content[start..end])
                        } else {
                            note.preview(1)
                        }
                    } else {
                        note.preview(1)
                    };

                    results.push(SearchResult {
                        note_id: note.id.clone(),
                        title: note.title.clone(),
                        path: note.path.clone(),
                        snippet,
                        match_count,
                    });
                }
            }
        }

        results.sort_by(|a, b| b.match_count.cmp(&a.match_count));
        results
    }

    pub fn note_count(&self) -> usize {
        self.nodes.values().filter(|n| matches!(n, Node::Note(_))).count()
    }

    pub fn folder_count(&self) -> usize {
        self.nodes.values().filter(|n| matches!(n, Node::Folder(_))).count()
    }
}
