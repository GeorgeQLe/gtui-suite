use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConnectionState {
    Established,
    Listen,
    TimeWait,
    CloseWait,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    Closing,
    LastAck,
    Unknown(String),
}

impl ConnectionState {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ESTABLISHED" | "ESTAB" => ConnectionState::Established,
            "LISTEN" => ConnectionState::Listen,
            "TIME_WAIT" | "TIME-WAIT" => ConnectionState::TimeWait,
            "CLOSE_WAIT" | "CLOSE-WAIT" => ConnectionState::CloseWait,
            "SYN_SENT" | "SYN-SENT" => ConnectionState::SynSent,
            "SYN_RECV" | "SYN-RECV" => ConnectionState::SynRecv,
            "FIN_WAIT1" | "FIN-WAIT-1" => ConnectionState::FinWait1,
            "FIN_WAIT2" | "FIN-WAIT-2" => ConnectionState::FinWait2,
            "CLOSING" => ConnectionState::Closing,
            "LAST_ACK" | "LAST-ACK" => ConnectionState::LastAck,
            _ => ConnectionState::Unknown(s.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ConnectionState::Established => "ESTABLISHED",
            ConnectionState::Listen => "LISTEN",
            ConnectionState::TimeWait => "TIME_WAIT",
            ConnectionState::CloseWait => "CLOSE_WAIT",
            ConnectionState::SynSent => "SYN_SENT",
            ConnectionState::SynRecv => "SYN_RECV",
            ConnectionState::FinWait1 => "FIN_WAIT1",
            ConnectionState::FinWait2 => "FIN_WAIT2",
            ConnectionState::Closing => "CLOSING",
            ConnectionState::LastAck => "LAST_ACK",
            ConnectionState::Unknown(s) => s,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub protocol: Protocol,
    pub local_addr: String,
    pub remote_addr: String,
    pub state: ConnectionState,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
}

pub fn get_connections() -> Vec<Connection> {
    let mut connections = Vec::new();

    // Try ss first (more modern), fall back to netstat
    if let Ok(output) = Command::new("ss")
        .args(["-tunapH"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(conn) = parse_ss_line(line) {
                    connections.push(conn);
                }
            }
            return connections;
        }
    }

    // Fallback to netstat
    if let Ok(output) = Command::new("netstat")
        .args(["-tunap"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(2) {
                if let Some(conn) = parse_netstat_line(line) {
                    connections.push(conn);
                }
            }
        }
    }

    connections
}

fn parse_ss_line(line: &str) -> Option<Connection> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }

    let protocol = match parts[0].to_lowercase().as_str() {
        "tcp" => Protocol::Tcp,
        "udp" => Protocol::Udp,
        _ => return None,
    };

    let state = if protocol == Protocol::Udp {
        ConnectionState::Unknown("UNCONN".to_string())
    } else {
        ConnectionState::from_str(parts[1])
    };

    let (local_idx, remote_idx) = if protocol == Protocol::Udp {
        (4, 5)
    } else {
        (4, 5)
    };

    let local_addr = parts.get(local_idx).map(|s| s.to_string()).unwrap_or_default();
    let remote_addr = parts.get(remote_idx).map(|s| s.to_string()).unwrap_or_default();

    // Parse PID/process from last column
    let (pid, process_name) = if let Some(last) = parts.last() {
        parse_pid_process(last)
    } else {
        (None, None)
    };

    Some(Connection {
        protocol,
        local_addr,
        remote_addr,
        state,
        pid,
        process_name,
    })
}

fn parse_netstat_line(line: &str) -> Option<Connection> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let protocol = match parts[0].to_lowercase().as_str() {
        "tcp" | "tcp6" => Protocol::Tcp,
        "udp" | "udp6" => Protocol::Udp,
        _ => return None,
    };

    let local_addr = parts.get(3).map(|s| s.to_string()).unwrap_or_default();
    let remote_addr = parts.get(4).map(|s| s.to_string()).unwrap_or_default();

    let state = if parts.len() > 5 && protocol == Protocol::Tcp {
        ConnectionState::from_str(parts[5])
    } else {
        ConnectionState::Unknown("-".to_string())
    };

    let (pid, process_name) = if let Some(last) = parts.last() {
        parse_pid_process(last)
    } else {
        (None, None)
    };

    Some(Connection {
        protocol,
        local_addr,
        remote_addr,
        state,
        pid,
        process_name,
    })
}

fn parse_pid_process(s: &str) -> (Option<u32>, Option<String>) {
    // Format is usually "pid/program" or just "-"
    if s == "-" || s.is_empty() {
        return (None, None);
    }

    // Handle formats like "users:((\"sshd\",pid=1234,fd=3))"
    if s.contains("pid=") {
        if let Some(pid_start) = s.find("pid=") {
            let pid_str: String = s[pid_start + 4..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            let pid = pid_str.parse().ok();

            // Try to extract process name
            let name = if let Some(start) = s.find("((\"") {
                let rest = &s[start + 3..];
                rest.split('"').next().map(|s| s.to_string())
            } else {
                None
            };

            return (pid, name);
        }
    }

    // Simple "pid/name" format
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() >= 2 {
        let pid = parts[0].parse().ok();
        let name = Some(parts[1].to_string());
        (pid, name)
    } else if let Ok(pid) = s.parse::<u32>() {
        (Some(pid), None)
    } else {
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol() {
        assert_eq!(Protocol::Tcp.as_str(), "TCP");
        assert_eq!(Protocol::Udp.as_str(), "UDP");
    }

    #[test]
    fn test_connection_state() {
        assert_eq!(ConnectionState::from_str("ESTABLISHED"), ConnectionState::Established);
        assert_eq!(ConnectionState::from_str("LISTEN"), ConnectionState::Listen);
        assert_eq!(ConnectionState::from_str("TIME_WAIT"), ConnectionState::TimeWait);
    }

    #[test]
    fn test_parse_pid_process() {
        assert_eq!(parse_pid_process("-"), (None, None));
        assert_eq!(parse_pid_process("1234/nginx"), (Some(1234), Some("nginx".to_string())));
    }
}
