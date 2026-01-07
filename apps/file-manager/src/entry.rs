use chrono::{DateTime, Local};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Represents a file or directory entry
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// File name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// Entry type
    pub entry_type: EntryType,
    /// File size in bytes
    pub size: u64,
    /// Last modified time
    pub modified: Option<DateTime<Local>>,
    /// Unix permissions
    pub permissions: u32,
    /// Is hidden file
    pub is_hidden: bool,
    /// Is symbolic link
    pub is_symlink: bool,
    /// Symlink target (if symlink)
    pub symlink_target: Option<PathBuf>,
}

/// Type of file entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Directory,
    File,
    Symlink,
}

impl FileEntry {
    /// Create entry from a path
    pub fn from_path(path: &Path) -> Option<Self> {
        let metadata = fs::symlink_metadata(path).ok()?;
        let name = path.file_name()?.to_string_lossy().to_string();

        let is_symlink = metadata.file_type().is_symlink();
        let symlink_target = if is_symlink {
            fs::read_link(path).ok()
        } else {
            None
        };

        // Get real metadata for symlinks
        let real_metadata = if is_symlink {
            fs::metadata(path).ok()
        } else {
            Some(metadata.clone())
        };

        let entry_type = if metadata.is_dir() || real_metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false) {
            EntryType::Directory
        } else if is_symlink {
            EntryType::Symlink
        } else {
            EntryType::File
        };

        let size = if entry_type == EntryType::Directory {
            0
        } else {
            real_metadata.as_ref().map(|m| m.len()).unwrap_or(0)
        };

        let modified = real_metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(DateTime::from);

        let permissions = metadata.permissions().mode();
        let is_hidden = name.starts_with('.');

        Some(Self {
            name,
            path: path.to_path_buf(),
            entry_type,
            size,
            modified,
            permissions,
            is_hidden,
            is_symlink,
            symlink_target,
        })
    }

    /// Create parent directory entry
    pub fn parent(path: &Path) -> Option<Self> {
        let parent = path.parent()?;
        Some(Self {
            name: "..".to_string(),
            path: parent.to_path_buf(),
            entry_type: EntryType::Directory,
            size: 0,
            modified: None,
            permissions: 0o755,
            is_hidden: false,
            is_symlink: false,
            symlink_target: None,
        })
    }

    /// Get icon for entry type
    pub fn icon(&self) -> &'static str {
        if self.name == ".." {
            return "󰁝 ";
        }

        match self.entry_type {
            EntryType::Directory => " ",
            EntryType::Symlink => " ",
            EntryType::File => self.file_icon(),
        }
    }

    fn file_icon(&self) -> &'static str {
        let ext = self.path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            // Documents
            "txt" | "md" | "rst" => " ",
            "pdf" => " ",
            "doc" | "docx" => " ",
            "xls" | "xlsx" => " ",
            "ppt" | "pptx" => " ",

            // Code
            "rs" => " ",
            "py" => " ",
            "js" | "ts" | "jsx" | "tsx" => " ",
            "html" | "htm" => " ",
            "css" | "scss" | "sass" => " ",
            "json" => " ",
            "toml" | "yaml" | "yml" => " ",
            "sh" | "bash" | "zsh" => " ",
            "c" | "cpp" | "h" | "hpp" => " ",
            "go" => " ",
            "java" => " ",
            "rb" => " ",
            "php" => " ",

            // Images
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" => " ",

            // Audio/Video
            "mp3" | "wav" | "flac" | "ogg" | "m4a" => " ",
            "mp4" | "mkv" | "avi" | "mov" | "webm" => " ",

            // Archives
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => " ",

            // Config
            "conf" | "cfg" | "ini" => " ",
            "lock" => " ",

            // Git
            "git" | "gitignore" => " ",

            // Executables
            "exe" | "bin" | "app" => " ",

            // Database
            "db" | "sqlite" | "sqlite3" => " ",

            _ => " ",
        }
    }

    /// Format size for display
    pub fn format_size(&self) -> String {
        if self.entry_type == EntryType::Directory {
            return "<DIR>".to_string();
        }

        format_bytes(self.size)
    }

    /// Format permissions as rwx string
    pub fn format_permissions(&self) -> String {
        let mode = self.permissions;

        let mut result = String::with_capacity(10);

        // File type
        result.push(match self.entry_type {
            EntryType::Directory => 'd',
            EntryType::Symlink => 'l',
            EntryType::File => '-',
        });

        // Owner permissions
        result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
        result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
        result.push(if mode & 0o100 != 0 { 'x' } else { '-' });

        // Group permissions
        result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
        result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
        result.push(if mode & 0o010 != 0 { 'x' } else { '-' });

        // Other permissions
        result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
        result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
        result.push(if mode & 0o001 != 0 { 'x' } else { '-' });

        result
    }

    /// Format modified time
    pub fn format_modified(&self) -> String {
        match self.modified {
            Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
            None => String::new(),
        }
    }
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1}T", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// Sort method for entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMethod {
    #[default]
    Name,
    Size,
    Modified,
    Type,
}

impl SortMethod {
    pub fn label(&self) -> &'static str {
        match self {
            SortMethod::Name => "Name",
            SortMethod::Size => "Size",
            SortMethod::Modified => "Date",
            SortMethod::Type => "Type",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            SortMethod::Name => SortMethod::Size,
            SortMethod::Size => SortMethod::Modified,
            SortMethod::Modified => SortMethod::Type,
            SortMethod::Type => SortMethod::Name,
        }
    }
}
