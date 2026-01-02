//! Sandbox configuration for plugin isolation.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Sandbox configuration for plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Memory limit in bytes.
    pub memory_limit: usize,
    /// Instruction/cycle limit.
    pub instruction_limit: u64,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Allowed filesystem paths (glob patterns).
    pub allowed_paths: Vec<String>,
    /// Whether network access is allowed.
    pub allow_network: bool,
    /// Allowed network hosts (if network is enabled).
    pub allowed_hosts: Vec<String>,
    /// Allowed Lua modules.
    pub allowed_lua_modules: HashSet<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            memory_limit: 10 * 1024 * 1024, // 10 MB
            instruction_limit: 1_000_000,
            timeout_ms: 5000,
            allowed_paths: Vec::new(),
            allow_network: false,
            allowed_hosts: Vec::new(),
            allowed_lua_modules: Self::default_lua_modules(),
        }
    }
}

impl SandboxConfig {
    /// Create a new sandbox config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a permissive sandbox (for trusted plugins).
    pub fn permissive() -> Self {
        Self {
            memory_limit: 100 * 1024 * 1024, // 100 MB
            instruction_limit: 10_000_000,
            timeout_ms: 30000,
            allowed_paths: vec!["**".to_string()],
            allow_network: true,
            allowed_hosts: vec!["*".to_string()],
            allowed_lua_modules: Self::all_lua_modules(),
        }
    }

    /// Create a restrictive sandbox (maximum isolation).
    pub fn restrictive() -> Self {
        Self {
            memory_limit: 1024 * 1024, // 1 MB
            instruction_limit: 100_000,
            timeout_ms: 1000,
            allowed_paths: Vec::new(),
            allow_network: false,
            allowed_hosts: Vec::new(),
            allowed_lua_modules: Self::minimal_lua_modules(),
        }
    }

    /// Set memory limit.
    pub fn with_memory_limit(mut self, bytes: usize) -> Self {
        self.memory_limit = bytes;
        self
    }

    /// Set instruction limit.
    pub fn with_instruction_limit(mut self, limit: u64) -> Self {
        self.instruction_limit = limit;
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Allow a filesystem path.
    pub fn allow_path(mut self, path: impl Into<String>) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// Enable network access.
    pub fn with_network(mut self) -> Self {
        self.allow_network = true;
        self
    }

    /// Allow a network host.
    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.push(host.into());
        self
    }

    /// Allow a Lua module.
    pub fn allow_lua_module(mut self, module: impl Into<String>) -> Self {
        self.allowed_lua_modules.insert(module.into());
        self
    }

    /// Check if a path is allowed.
    pub fn is_path_allowed(&self, path: &PathBuf) -> bool {
        if self.allowed_paths.is_empty() {
            return false;
        }

        let path_str = path.to_string_lossy();

        for pattern in &self.allowed_paths {
            if pattern == "**" {
                return true;
            }

            if Self::glob_match(pattern, &path_str) {
                return true;
            }
        }

        false
    }

    /// Check if a host is allowed.
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if !self.allow_network {
            return false;
        }

        if self.allowed_hosts.is_empty() || self.allowed_hosts.contains(&"*".to_string()) {
            return true;
        }

        self.allowed_hosts.iter().any(|h| h == host || h == "*")
    }

    /// Check if a Lua module is allowed.
    pub fn is_lua_module_allowed(&self, module: &str) -> bool {
        self.allowed_lua_modules.contains(module) || self.allowed_lua_modules.contains("*")
    }

    /// Default safe Lua modules.
    fn default_lua_modules() -> HashSet<String> {
        [
            "string",
            "table",
            "math",
            "utf8",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Minimal Lua modules.
    fn minimal_lua_modules() -> HashSet<String> {
        ["string", "math"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// All standard Lua modules.
    fn all_lua_modules() -> HashSet<String> {
        [
            "string",
            "table",
            "math",
            "utf8",
            "os",
            "io",
            "package",
            "coroutine",
            "debug",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Simple glob matching.
    fn glob_match(pattern: &str, text: &str) -> bool {
        if pattern == "*" || pattern == "**" {
            return true;
        }

        if pattern.starts_with("*") && pattern.ends_with("*") {
            let inner = &pattern[1..pattern.len() - 1];
            return text.contains(inner);
        }

        if pattern.starts_with("*") {
            let suffix = &pattern[1..];
            return text.ends_with(suffix);
        }

        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len() - 1];
            return text.starts_with(prefix);
        }

        pattern == text
    }
}

/// Sandbox violation record.
#[derive(Debug, Clone)]
pub struct SandboxViolation {
    /// Type of violation.
    pub violation_type: ViolationType,
    /// Description.
    pub description: String,
    /// Timestamp.
    pub timestamp: std::time::SystemTime,
}

/// Types of sandbox violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    /// Memory limit exceeded.
    MemoryLimit,
    /// Instruction limit exceeded.
    InstructionLimit,
    /// Timeout.
    Timeout,
    /// Unauthorized filesystem access.
    FileAccess,
    /// Unauthorized network access.
    NetworkAccess,
    /// Unauthorized module access.
    ModuleAccess,
}

impl std::fmt::Display for ViolationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryLimit => write!(f, "memory_limit"),
            Self::InstructionLimit => write!(f, "instruction_limit"),
            Self::Timeout => write!(f, "timeout"),
            Self::FileAccess => write!(f, "file_access"),
            Self::NetworkAccess => write!(f, "network_access"),
            Self::ModuleAccess => write!(f, "module_access"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SandboxConfig::default();
        assert_eq!(config.memory_limit, 10 * 1024 * 1024);
        assert!(!config.allow_network);
    }

    #[test]
    fn test_permissive() {
        let config = SandboxConfig::permissive();
        assert!(config.allow_network);
        assert!(config.is_host_allowed("example.com"));
    }

    #[test]
    fn test_restrictive() {
        let config = SandboxConfig::restrictive();
        assert!(!config.allow_network);
        assert_eq!(config.memory_limit, 1024 * 1024);
    }

    #[test]
    fn test_path_allowed() {
        let config = SandboxConfig::default()
            .allow_path("/home/user/.config/*");

        assert!(config.is_path_allowed(&PathBuf::from("/home/user/.config/app")));
        assert!(!config.is_path_allowed(&PathBuf::from("/etc/passwd")));
    }

    #[test]
    fn test_lua_module_allowed() {
        let config = SandboxConfig::default();
        assert!(config.is_lua_module_allowed("string"));
        assert!(config.is_lua_module_allowed("math"));
        assert!(!config.is_lua_module_allowed("os"));
        assert!(!config.is_lua_module_allowed("io"));
    }

    #[test]
    fn test_glob_match() {
        assert!(SandboxConfig::glob_match("*", "anything"));
        assert!(SandboxConfig::glob_match("*.txt", "file.txt"));
        assert!(SandboxConfig::glob_match("/home/*", "/home/user"));
        assert!(SandboxConfig::glob_match("*config*", "/path/config/file"));
    }
}
