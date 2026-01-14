use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslHost {
    pub hostname: String,
    pub port: u16,
    pub certificate: Option<CertificateInfo>,
    pub last_checked: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl SslHost {
    pub fn new(hostname: &str, port: u16) -> Self {
        Self {
            hostname: hostname.to_string(),
            port,
            certificate: None,
            last_checked: None,
            error: None,
        }
    }

    pub fn status(&self) -> HostStatus {
        if self.error.is_some() {
            return HostStatus::Error;
        }

        if let Some(ref cert) = self.certificate {
            let now = Utc::now();
            if cert.not_after < now {
                return HostStatus::Expired;
            }

            let days_until_expiry = (cert.not_after - now).num_days();
            if days_until_expiry <= 7 {
                return HostStatus::Critical;
            }
            if days_until_expiry <= 30 {
                return HostStatus::Warning;
            }
            return HostStatus::Valid;
        }

        HostStatus::Unknown
    }

    pub fn days_until_expiry(&self) -> Option<i64> {
        self.certificate.as_ref().map(|cert| {
            (cert.not_after - Utc::now()).num_days()
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostStatus {
    Unknown,
    Valid,
    Warning,
    Critical,
    Expired,
    Error,
}

impl HostStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            HostStatus::Unknown => "?",
            HostStatus::Valid => "✓",
            HostStatus::Warning => "⚠",
            HostStatus::Critical => "!",
            HostStatus::Expired => "✗",
            HostStatus::Error => "⚡",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            HostStatus::Unknown => "Unknown",
            HostStatus::Valid => "Valid",
            HostStatus::Warning => "Warning",
            HostStatus::Critical => "Critical",
            HostStatus::Expired => "Expired",
            HostStatus::Error => "Error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub serial_number: String,
    pub signature_algorithm: String,
    pub san: Vec<String>,
    pub is_self_signed: bool,
    pub key_usage: Vec<String>,
    pub fingerprint_sha256: String,
}

impl CertificateInfo {
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        now >= self.not_before && now <= self.not_after
    }

    pub fn validity_days(&self) -> i64 {
        (self.not_after - self.not_before).num_days()
    }
}

pub fn create_demo_hosts() -> Vec<SslHost> {
    let now = Utc::now();

    vec![
        {
            let mut host = SslHost::new("example.com", 443);
            host.certificate = Some(CertificateInfo {
                subject: "CN=example.com".to_string(),
                issuer: "CN=DigiCert SHA2 Extended Validation Server CA".to_string(),
                not_before: now - chrono::Duration::days(180),
                not_after: now + chrono::Duration::days(185),
                serial_number: "0A:1B:2C:3D:4E:5F".to_string(),
                signature_algorithm: "SHA256withRSA".to_string(),
                san: vec!["example.com".to_string(), "www.example.com".to_string()],
                is_self_signed: false,
                key_usage: vec!["Digital Signature".to_string(), "Key Encipherment".to_string()],
                fingerprint_sha256: "AB:CD:EF:12:34:56:78:90".to_string(),
            });
            host.last_checked = Some(now);
            host
        },
        {
            let mut host = SslHost::new("expiring-soon.test", 443);
            host.certificate = Some(CertificateInfo {
                subject: "CN=expiring-soon.test".to_string(),
                issuer: "CN=Let's Encrypt Authority X3".to_string(),
                not_before: now - chrono::Duration::days(85),
                not_after: now + chrono::Duration::days(5),
                serial_number: "11:22:33:44:55:66".to_string(),
                signature_algorithm: "SHA256withRSA".to_string(),
                san: vec!["expiring-soon.test".to_string()],
                is_self_signed: false,
                key_usage: vec!["Digital Signature".to_string()],
                fingerprint_sha256: "11:22:33:44:55:66:77:88".to_string(),
            });
            host.last_checked = Some(now);
            host
        },
        {
            let mut host = SslHost::new("expired.test", 443);
            host.certificate = Some(CertificateInfo {
                subject: "CN=expired.test".to_string(),
                issuer: "CN=Some CA".to_string(),
                not_before: now - chrono::Duration::days(400),
                not_after: now - chrono::Duration::days(35),
                serial_number: "AA:BB:CC:DD:EE:FF".to_string(),
                signature_algorithm: "SHA256withRSA".to_string(),
                san: vec!["expired.test".to_string()],
                is_self_signed: false,
                key_usage: vec!["Digital Signature".to_string()],
                fingerprint_sha256: "AA:BB:CC:DD:EE:FF:00:11".to_string(),
            });
            host.last_checked = Some(now);
            host
        },
        {
            let mut host = SslHost::new("self-signed.local", 8443);
            host.certificate = Some(CertificateInfo {
                subject: "CN=self-signed.local".to_string(),
                issuer: "CN=self-signed.local".to_string(),
                not_before: now - chrono::Duration::days(30),
                not_after: now + chrono::Duration::days(335),
                serial_number: "01:02:03:04:05:06".to_string(),
                signature_algorithm: "SHA256withRSA".to_string(),
                san: vec!["self-signed.local".to_string(), "localhost".to_string()],
                is_self_signed: true,
                key_usage: vec!["Digital Signature".to_string(), "Key Encipherment".to_string()],
                fingerprint_sha256: "01:02:03:04:05:06:07:08".to_string(),
            });
            host.last_checked = Some(now);
            host
        },
        {
            let mut host = SslHost::new("unreachable.test", 443);
            host.error = Some("Connection refused".to_string());
            host.last_checked = Some(now);
            host
        },
        {
            let mut host = SslHost::new("warning.test", 443);
            host.certificate = Some(CertificateInfo {
                subject: "CN=warning.test".to_string(),
                issuer: "CN=GlobalSign".to_string(),
                not_before: now - chrono::Duration::days(60),
                not_after: now + chrono::Duration::days(20),
                serial_number: "FF:EE:DD:CC:BB:AA".to_string(),
                signature_algorithm: "SHA256withRSA".to_string(),
                san: vec!["warning.test".to_string(), "*.warning.test".to_string()],
                is_self_signed: false,
                key_usage: vec!["Digital Signature".to_string()],
                fingerprint_sha256: "FF:EE:DD:CC:BB:AA:99:88".to_string(),
            });
            host.last_checked = Some(now);
            host
        },
    ]
}
