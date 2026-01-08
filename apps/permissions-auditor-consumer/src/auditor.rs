use chrono::{DateTime, Utc};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Finding {
    pub id: Uuid,
    pub path: PathBuf,
    pub finding_type: FindingType,
    pub severity: Severity,
    pub current_permissions: String,
    pub recommended_permissions: Option<String>,
    pub description: String,
    pub fix_command: Option<String>,
    pub found_at: DateTime<Utc>,
    pub ignored: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingType {
    WorldWritable,
    SuidBinary,
    SgidBinary,
    WeakSshPermissions,
    WeakGpgPermissions,
    OwnershipIssue,
    SensitiveFileExposed,
}

impl FindingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingType::WorldWritable => "World Writable",
            FindingType::SuidBinary => "SUID Binary",
            FindingType::SgidBinary => "SGID Binary",
            FindingType::WeakSshPermissions => "Weak SSH Perms",
            FindingType::WeakGpgPermissions => "Weak GPG Perms",
            FindingType::OwnershipIssue => "Ownership Issue",
            FindingType::SensitiveFileExposed => "Sensitive File",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "Low",
            Severity::Medium => "Medium",
            Severity::High => "High",
            Severity::Critical => "Critical",
        }
    }
}

pub struct Auditor {
    config: Config,
}

impl Auditor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn scan(&self) -> Vec<Finding> {
        let mut findings = Vec::new();

        for path_str in &self.config.scan.paths {
            let path = PathBuf::from(shellexpand::tilde(path_str).to_string());
            if path.exists() {
                self.scan_directory(&path, 0, &mut findings);
            }
        }

        // Sort by severity (critical first)
        findings.sort_by(|a, b| b.severity.cmp(&a.severity));
        findings
    }

    fn scan_directory(&self, dir: &Path, depth: usize, findings: &mut Vec<Finding>) {
        if depth > self.config.scan.max_depth {
            return;
        }

        // Check if path should be ignored
        if self.should_ignore(dir) {
            return;
        }

        // Check the directory itself
        self.check_path(dir, findings);

        // Scan contents
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return, // Permission denied or other error
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_symlink() && !self.config.scan.follow_symlinks {
                continue;
            }

            self.check_path(&path, findings);

            if path.is_dir() {
                self.scan_directory(&path, depth + 1, findings);
            }
        }
    }

    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check exact paths
        for ignore_path in &self.config.ignore.paths {
            if path_str == *ignore_path {
                return true;
            }
        }

        // Check patterns (simple glob matching)
        for pattern in &self.config.ignore.patterns {
            if simple_glob_match(pattern, &path_str) {
                return true;
            }
        }

        false
    }

    fn check_path(&self, path: &Path, findings: &mut Vec<Finding>) {
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return,
        };

        let mode = metadata.permissions().mode();
        let path_str = path.to_string_lossy();

        // Check for world-writable
        if self.config.checks.world_writable && (mode & 0o002) != 0 {
            findings.push(Finding {
                id: Uuid::new_v4(),
                path: path.to_path_buf(),
                finding_type: FindingType::WorldWritable,
                severity: if metadata.is_dir() { Severity::High } else { Severity::Medium },
                current_permissions: format!("{:o}", mode & 0o7777),
                recommended_permissions: Some(format!("{:o}", mode & 0o7775)),
                description: format!(
                    "{} is world-writable",
                    if metadata.is_dir() { "Directory" } else { "File" }
                ),
                fix_command: Some(format!("chmod o-w \"{}\"", path_str)),
                found_at: Utc::now(),
                ignored: false,
            });
        }

        // Check for SUID
        if self.config.checks.suid_sgid && (mode & 0o4000) != 0 && metadata.is_file() {
            findings.push(Finding {
                id: Uuid::new_v4(),
                path: path.to_path_buf(),
                finding_type: FindingType::SuidBinary,
                severity: Severity::High,
                current_permissions: format!("{:o}", mode & 0o7777),
                recommended_permissions: None,
                description: "SUID binary detected - verify this is expected".to_string(),
                fix_command: Some(format!("chmod u-s \"{}\"", path_str)),
                found_at: Utc::now(),
                ignored: false,
            });
        }

        // Check for SGID
        if self.config.checks.suid_sgid && (mode & 0o2000) != 0 && metadata.is_file() {
            findings.push(Finding {
                id: Uuid::new_v4(),
                path: path.to_path_buf(),
                finding_type: FindingType::SgidBinary,
                severity: Severity::Medium,
                current_permissions: format!("{:o}", mode & 0o7777),
                recommended_permissions: None,
                description: "SGID binary detected - verify this is expected".to_string(),
                fix_command: Some(format!("chmod g-s \"{}\"", path_str)),
                found_at: Utc::now(),
                ignored: false,
            });
        }

        // Check SSH permissions
        if self.config.checks.ssh_permissions && path_str.contains("/.ssh/") {
            self.check_ssh_permissions(path, mode, findings);
        }

        // Check GPG permissions
        if self.config.checks.gpg_permissions && path_str.contains("/.gnupg/") {
            self.check_gpg_permissions(path, mode, findings);
        }
    }

    fn check_ssh_permissions(&self, path: &Path, mode: u32, findings: &mut Vec<Finding>) {
        let path_str = path.to_string_lossy();
        let file_mode = mode & 0o7777;

        // Private keys should be 600
        if path_str.ends_with("id_rsa") || path_str.ends_with("id_ed25519") || path_str.ends_with("id_ecdsa") {
            if file_mode != 0o600 {
                findings.push(Finding {
                    id: Uuid::new_v4(),
                    path: path.to_path_buf(),
                    finding_type: FindingType::WeakSshPermissions,
                    severity: Severity::Critical,
                    current_permissions: format!("{:o}", file_mode),
                    recommended_permissions: Some("600".to_string()),
                    description: "SSH private key has weak permissions".to_string(),
                    fix_command: Some(format!("chmod 600 \"{}\"", path_str)),
                    found_at: Utc::now(),
                    ignored: false,
                });
            }
        }

        // authorized_keys should be 600 or 644
        if path_str.ends_with("authorized_keys") && file_mode != 0o600 && file_mode != 0o644 {
            findings.push(Finding {
                id: Uuid::new_v4(),
                path: path.to_path_buf(),
                finding_type: FindingType::WeakSshPermissions,
                severity: Severity::High,
                current_permissions: format!("{:o}", file_mode),
                recommended_permissions: Some("600".to_string()),
                description: "authorized_keys has weak permissions".to_string(),
                fix_command: Some(format!("chmod 600 \"{}\"", path_str)),
                found_at: Utc::now(),
                ignored: false,
            });
        }

        // .ssh directory should be 700
        if path_str.ends_with(".ssh") && file_mode != 0o700 {
            findings.push(Finding {
                id: Uuid::new_v4(),
                path: path.to_path_buf(),
                finding_type: FindingType::WeakSshPermissions,
                severity: Severity::High,
                current_permissions: format!("{:o}", file_mode),
                recommended_permissions: Some("700".to_string()),
                description: ".ssh directory has weak permissions".to_string(),
                fix_command: Some(format!("chmod 700 \"{}\"", path_str)),
                found_at: Utc::now(),
                ignored: false,
            });
        }
    }

    fn check_gpg_permissions(&self, path: &Path, mode: u32, findings: &mut Vec<Finding>) {
        let path_str = path.to_string_lossy();
        let file_mode = mode & 0o7777;

        // .gnupg directory should be 700
        if path_str.ends_with(".gnupg") && file_mode != 0o700 {
            findings.push(Finding {
                id: Uuid::new_v4(),
                path: path.to_path_buf(),
                finding_type: FindingType::WeakGpgPermissions,
                severity: Severity::High,
                current_permissions: format!("{:o}", file_mode),
                recommended_permissions: Some("700".to_string()),
                description: ".gnupg directory has weak permissions".to_string(),
                fix_command: Some(format!("chmod 700 \"{}\"", path_str)),
                found_at: Utc::now(),
                ignored: false,
            });
        }

        // Private keys should be 600
        if path_str.contains("private-keys") && file_mode != 0o600 {
            findings.push(Finding {
                id: Uuid::new_v4(),
                path: path.to_path_buf(),
                finding_type: FindingType::WeakGpgPermissions,
                severity: Severity::Critical,
                current_permissions: format!("{:o}", file_mode),
                recommended_permissions: Some("600".to_string()),
                description: "GPG private key has weak permissions".to_string(),
                fix_command: Some(format!("chmod 600 \"{}\"", path_str)),
                found_at: Utc::now(),
                ignored: false,
            });
        }
    }
}

fn simple_glob_match(pattern: &str, text: &str) -> bool {
    // Simple glob matching for * wildcards
    if !pattern.contains('*') {
        return pattern == text;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if let Some(found) = text[pos..].find(part) {
            if i == 0 && found != 0 {
                // Pattern doesn't start with *, so must match from beginning
                return false;
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }

    // If pattern doesn't end with *, must match to end
    if !pattern.ends_with('*') && pos != text.len() {
        return false;
    }

    true
}

mod shellexpand {
    pub fn tilde(path: &str) -> std::borrow::Cow<'_, str> {
        if path.starts_with('~') {
            if let Some(home) = std::env::var_os("HOME") {
                return std::borrow::Cow::Owned(path.replacen('~', &home.to_string_lossy(), 1));
            }
        }
        std::borrow::Cow::Borrowed(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_glob_match() {
        assert!(simple_glob_match("*/node_modules/*", "/home/user/project/node_modules/package"));
        assert!(simple_glob_match("*/.git/*", "/home/user/project/.git/config"));
        assert!(!simple_glob_match("*/node_modules/*", "/home/user/project/src/main.rs"));
        assert!(simple_glob_match("*.rs", "main.rs"));
        assert!(!simple_glob_match("*.rs", "main.py"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_finding_type_display() {
        assert_eq!(FindingType::WorldWritable.as_str(), "World Writable");
        assert_eq!(FindingType::SuidBinary.as_str(), "SUID Binary");
    }
}
