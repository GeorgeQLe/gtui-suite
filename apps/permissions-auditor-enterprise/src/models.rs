use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceFramework {
    CisBenchmark,
    Stig,
    Custom,
}

impl ComplianceFramework {
    pub fn as_str(&self) -> &str {
        match self {
            ComplianceFramework::CisBenchmark => "CIS Benchmark",
            ComplianceFramework::Stig => "DISA STIG",
            ComplianceFramework::Custom => "Custom",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            ComplianceFramework::CisBenchmark => "🏛️",
            ComplianceFramework::Stig => "🎖️",
            ComplianceFramework::Custom => "📋",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &str {
        match self {
            Severity::Critical => "Critical",
            Severity::High => "High",
            Severity::Medium => "Medium",
            Severity::Low => "Low",
            Severity::Info => "Info",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🟢",
            Severity::Info => "🔵",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingStatus {
    Pass,
    Fail,
    NotApplicable,
    Error,
}

impl FindingStatus {
    pub fn as_str(&self) -> &str {
        match self {
            FindingStatus::Pass => "Pass",
            FindingStatus::Fail => "Fail",
            FindingStatus::NotApplicable => "N/A",
            FindingStatus::Error => "Error",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            FindingStatus::Pass => "✓",
            FindingStatus::Fail => "✗",
            FindingStatus::NotApplicable => "—",
            FindingStatus::Error => "!",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub id: String,
    pub framework: ComplianceFramework,
    pub section: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub remediation: String,
    pub references: Vec<String>,
}

impl ComplianceCheck {
    pub fn new(id: &str, framework: ComplianceFramework, section: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            framework,
            section: section.to_string(),
            title: title.to_string(),
            description: String::new(),
            severity: Severity::Medium,
            remediation: String::new(),
            references: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub check: ComplianceCheck,
    pub status: FindingStatus,
    pub details: String,
    pub evidence: Vec<String>,
    pub remediation_steps: Vec<String>,
}

impl Finding {
    pub fn new(check: ComplianceCheck, status: FindingStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            check,
            status,
            details: String::new(),
            evidence: Vec::new(),
            remediation_steps: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    pub id: Uuid,
    pub name: String,
    pub hostname: String,
    pub ip_address: Option<String>,
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub last_scan: Option<DateTime<Utc>>,
    pub compliance_score: Option<f64>,
    pub status: SystemStatus,
}

impl System {
    pub fn new(name: &str, hostname: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            hostname: hostname.to_string(),
            ip_address: None,
            os: None,
            os_version: None,
            last_scan: None,
            compliance_score: None,
            status: SystemStatus::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemStatus {
    Online,
    Offline,
    Scanning,
    Error,
    Unknown,
}

impl SystemStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SystemStatus::Online => "Online",
            SystemStatus::Offline => "Offline",
            SystemStatus::Scanning => "Scanning",
            SystemStatus::Error => "Error",
            SystemStatus::Unknown => "Unknown",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            SystemStatus::Online => "●",
            SystemStatus::Offline => "○",
            SystemStatus::Scanning => "◐",
            SystemStatus::Error => "!",
            SystemStatus::Unknown => "?",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemScan {
    pub id: Uuid,
    pub system_id: Uuid,
    pub system_name: String,
    pub scan_time: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub findings: Vec<Finding>,
    pub compliance_score: f64,
    pub checks_passed: usize,
    pub checks_failed: usize,
    pub checks_na: usize,
}

impl SystemScan {
    pub fn new(system: &System) -> Self {
        Self {
            id: Uuid::new_v4(),
            system_id: system.id,
            system_name: system.name.clone(),
            scan_time: Utc::now(),
            completed_at: None,
            findings: Vec::new(),
            compliance_score: 0.0,
            checks_passed: 0,
            checks_failed: 0,
            checks_na: 0,
        }
    }

    pub fn calculate_score(&mut self) {
        let total = self.findings.len();
        if total == 0 {
            self.compliance_score = 100.0;
            return;
        }

        self.checks_passed = self.findings.iter().filter(|f| f.status == FindingStatus::Pass).count();
        self.checks_failed = self.findings.iter().filter(|f| f.status == FindingStatus::Fail).count();
        self.checks_na = self.findings.iter().filter(|f| f.status == FindingStatus::NotApplicable).count();

        let applicable = total - self.checks_na;
        if applicable == 0 {
            self.compliance_score = 100.0;
        } else {
            self.compliance_score = (self.checks_passed as f64 / applicable as f64) * 100.0;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub path: String,
    pub capabilities: Vec<String>,
    pub expected: bool,
}

impl CapabilityInfo {
    pub fn new(path: &str, capabilities: Vec<String>) -> Self {
        Self {
            path: path.to_string(),
            capabilities,
            expected: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub privileged: bool,
    pub mounts: Vec<ContainerMount>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMount {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
}

#[derive(Debug, Clone)]
pub struct RemediationAction {
    pub id: Uuid,
    pub finding_id: Uuid,
    pub description: String,
    pub command: Option<String>,
    pub status: RemediationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemediationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

impl RemediationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            RemediationStatus::Pending => "Pending",
            RemediationStatus::InProgress => "In Progress",
            RemediationStatus::Completed => "Completed",
            RemediationStatus::Failed => "Failed",
            RemediationStatus::Skipped => "Skipped",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComplianceStats {
    pub total_systems: usize,
    pub systems_scanned: usize,
    pub average_score: f64,
    pub critical_findings: usize,
    pub high_findings: usize,
    pub frameworks_enabled: Vec<ComplianceFramework>,
}

impl ComplianceStats {
    pub fn new() -> Self {
        Self {
            total_systems: 0,
            systems_scanned: 0,
            average_score: 0.0,
            critical_findings: 0,
            high_findings: 0,
            frameworks_enabled: vec![ComplianceFramework::CisBenchmark],
        }
    }

    pub fn update(&mut self, systems: &[System], scans: &[SystemScan]) {
        self.total_systems = systems.len();
        self.systems_scanned = systems.iter().filter(|s| s.last_scan.is_some()).count();

        let scores: Vec<f64> = systems.iter().filter_map(|s| s.compliance_score).collect();
        if !scores.is_empty() {
            self.average_score = scores.iter().sum::<f64>() / scores.len() as f64;
        }

        self.critical_findings = scans
            .iter()
            .flat_map(|s| &s.findings)
            .filter(|f| f.check.severity == Severity::Critical && f.status == FindingStatus::Fail)
            .count();

        self.high_findings = scans
            .iter()
            .flat_map(|s| &s.findings)
            .filter(|f| f.check.severity == Severity::High && f.status == FindingStatus::Fail)
            .count();
    }
}

impl Default for ComplianceStats {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_cis_checks() -> Vec<ComplianceCheck> {
    vec![
        {
            let mut c = ComplianceCheck::new(
                "1.1.1",
                ComplianceFramework::CisBenchmark,
                "1.1",
                "Ensure mounting of cramfs filesystems is disabled",
            );
            c.description = "The cramfs filesystem type is a compressed read-only Linux filesystem.".to_string();
            c.severity = Severity::Low;
            c.remediation = "Add 'install cramfs /bin/true' to /etc/modprobe.d/cramfs.conf".to_string();
            c
        },
        {
            let mut c = ComplianceCheck::new(
                "1.4.1",
                ComplianceFramework::CisBenchmark,
                "1.4",
                "Ensure permissions on bootloader config are configured",
            );
            c.description = "The grub configuration file contains information on boot settings and passwords.".to_string();
            c.severity = Severity::High;
            c.remediation = "Run: chmod og-rwx /boot/grub/grub.cfg".to_string();
            c
        },
        {
            let mut c = ComplianceCheck::new(
                "4.1.1",
                ComplianceFramework::CisBenchmark,
                "4.1",
                "Ensure auditing is enabled",
            );
            c.description = "Enable the auditd daemon to record system events.".to_string();
            c.severity = Severity::Critical;
            c.remediation = "Run: systemctl enable auditd && systemctl start auditd".to_string();
            c
        },
        {
            let mut c = ComplianceCheck::new(
                "5.2.1",
                ComplianceFramework::CisBenchmark,
                "5.2",
                "Ensure permissions on /etc/ssh/sshd_config are configured",
            );
            c.description = "The /etc/ssh/sshd_config file contains configuration for the OpenSSH daemon.".to_string();
            c.severity = Severity::Medium;
            c.remediation = "Run: chmod og-rwx /etc/ssh/sshd_config".to_string();
            c
        },
        {
            let mut c = ComplianceCheck::new(
                "6.1.1",
                ComplianceFramework::CisBenchmark,
                "6.1",
                "Audit system file permissions",
            );
            c.description = "Verify permissions on system files match vendor recommendations.".to_string();
            c.severity = Severity::Medium;
            c.remediation = "Review and correct permissions using the package manager verification.".to_string();
            c
        },
    ]
}
