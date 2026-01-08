use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Host {
    pub ip: IpAddr,
    pub hostname: Option<String>,
    pub ports: Vec<Port>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub number: u16,
    pub state: PortState,
    pub service: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    Open,
    Closed,
    Filtered,
}

#[derive(Debug, Clone)]
pub enum ScanResult {
    PortScanned { ip: IpAddr, port: u16, state: PortState },
    HostCompleted { ip: IpAddr },
    ScanCompleted,
    Progress { scanned: usize, total: usize },
}

pub struct Scanner {
    timeout_ms: u64,
    max_concurrent: usize,
}

impl Scanner {
    pub fn new(timeout_ms: u64, max_concurrent: usize) -> Self {
        Self {
            timeout_ms,
            max_concurrent,
        }
    }

    pub fn scan_ports(
        &self,
        target: IpAddr,
        ports: Vec<u16>,
    ) -> mpsc::Receiver<ScanResult> {
        let (tx, rx) = mpsc::channel(1000);
        let timeout_ms = self.timeout_ms;
        let max_concurrent = self.max_concurrent;

        tokio::spawn(async move {
            let total = ports.len();
            let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));
            let mut handles = Vec::new();
            let scanned = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

            for port in ports {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let tx = tx.clone();
                let scanned = scanned.clone();

                let handle = tokio::spawn(async move {
                    let state = scan_tcp_port(target, port, timeout_ms).await;
                    let _ = tx.send(ScanResult::PortScanned {
                        ip: target,
                        port,
                        state,
                    }).await;

                    let count = scanned.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    let _ = tx.send(ScanResult::Progress {
                        scanned: count,
                        total,
                    }).await;

                    drop(permit);
                });

                handles.push(handle);
            }

            for handle in handles {
                let _ = handle.await;
            }

            let _ = tx.send(ScanResult::HostCompleted { ip: target }).await;
            let _ = tx.send(ScanResult::ScanCompleted).await;
        });

        rx
    }
}

async fn scan_tcp_port(ip: IpAddr, port: u16, timeout_ms: u64) -> PortState {
    let addr = SocketAddr::new(ip, port);
    let connect_timeout = Duration::from_millis(timeout_ms);

    match timeout(connect_timeout, TcpStream::connect(addr)).await {
        Ok(Ok(_)) => PortState::Open,
        Ok(Err(e)) => {
            // Connection refused means port is closed
            if e.kind() == std::io::ErrorKind::ConnectionRefused {
                PortState::Closed
            } else {
                PortState::Filtered
            }
        }
        Err(_) => PortState::Filtered, // Timeout
    }
}

pub fn get_service_name(port: u16) -> Option<&'static str> {
    match port {
        20 => Some("ftp-data"),
        21 => Some("ftp"),
        22 => Some("ssh"),
        23 => Some("telnet"),
        25 => Some("smtp"),
        53 => Some("dns"),
        80 => Some("http"),
        110 => Some("pop3"),
        111 => Some("rpcbind"),
        135 => Some("msrpc"),
        139 => Some("netbios-ssn"),
        143 => Some("imap"),
        443 => Some("https"),
        445 => Some("microsoft-ds"),
        993 => Some("imaps"),
        995 => Some("pop3s"),
        1433 => Some("mssql"),
        1521 => Some("oracle"),
        3306 => Some("mysql"),
        3389 => Some("rdp"),
        5432 => Some("postgresql"),
        5900 => Some("vnc"),
        6379 => Some("redis"),
        8080 => Some("http-proxy"),
        8443 => Some("https-alt"),
        27017 => Some("mongodb"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_names() {
        assert_eq!(get_service_name(22), Some("ssh"));
        assert_eq!(get_service_name(80), Some("http"));
        assert_eq!(get_service_name(443), Some("https"));
        assert_eq!(get_service_name(99999), None);
    }

    #[test]
    fn test_port_state_eq() {
        assert_eq!(PortState::Open, PortState::Open);
        assert_ne!(PortState::Open, PortState::Closed);
    }
}
