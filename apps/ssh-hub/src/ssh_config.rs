use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::host::HostProfile;

/// Parse SSH config file and extract host definitions
pub fn parse_ssh_config() -> Result<Vec<HostProfile>> {
    let config_path = get_ssh_config_path();

    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&config_path)?;
    parse_config_content(&content)
}

fn get_ssh_config_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".ssh").join("config"))
        .unwrap_or_else(|| PathBuf::from("/etc/ssh/ssh_config"))
}

fn parse_config_content(content: &str) -> Result<Vec<HostProfile>> {
    let mut hosts = Vec::new();
    let mut current_host: Option<HashMap<String, String>> = None;
    let mut current_name: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key-value pair
        let parts: Vec<&str> = line.splitn(2, |c: char| c.is_whitespace() || c == '=').collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0].trim().to_lowercase();
        let value = parts[1].trim().trim_matches('"').to_string();

        if key == "host" {
            // Save previous host if exists
            if let (Some(name), Some(props)) = (current_name.take(), current_host.take()) {
                if let Some(host) = build_host_profile(&name, &props) {
                    hosts.push(host);
                }
            }

            // Start new host (skip wildcards)
            if !value.contains('*') && !value.contains('?') {
                current_name = Some(value);
                current_host = Some(HashMap::new());
            }
        } else if let Some(ref mut props) = current_host {
            props.insert(key, value);
        }
    }

    // Don't forget the last host
    if let (Some(name), Some(props)) = (current_name, current_host) {
        if let Some(host) = build_host_profile(&name, &props) {
            hosts.push(host);
        }
    }

    Ok(hosts)
}

fn build_host_profile(name: &str, props: &HashMap<String, String>) -> Option<HostProfile> {
    let hostname = props.get("hostname").cloned().unwrap_or_else(|| name.to_string());

    let mut host = HostProfile::new(name.to_string(), hostname);

    if let Some(user) = props.get("user") {
        host.user = Some(user.clone());
    }

    if let Some(port) = props.get("port") {
        host.port = port.parse().ok();
    }

    if let Some(identity) = props.get("identityfile") {
        let path = expand_tilde(identity);
        host.identity_file = Some(PathBuf::from(path));
    }

    if let Some(proxy) = props.get("proxyjump") {
        host.proxy_jump = Some(proxy.clone());
    }

    Some(host)
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &path[1..]);
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let config = r#"
Host myserver
    HostName 192.168.1.100
    User admin
    Port 2222

Host webserver
    HostName web.example.com
    User deploy
    IdentityFile ~/.ssh/deploy_key
"#;

        let hosts = parse_config_content(config).unwrap();
        assert_eq!(hosts.len(), 2);

        assert_eq!(hosts[0].name, "myserver");
        assert_eq!(hosts[0].host, "192.168.1.100");
        assert_eq!(hosts[0].user, Some("admin".to_string()));
        assert_eq!(hosts[0].port, Some(2222));

        assert_eq!(hosts[1].name, "webserver");
        assert_eq!(hosts[1].host, "web.example.com");
        assert_eq!(hosts[1].user, Some("deploy".to_string()));
    }

    #[test]
    fn test_skip_wildcards() {
        let config = r#"
Host *
    ServerAliveInterval 60

Host myserver
    HostName 192.168.1.100
"#;

        let hosts = parse_config_content(config).unwrap();
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].name, "myserver");
    }
}
