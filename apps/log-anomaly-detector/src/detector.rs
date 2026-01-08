use regex::Regex;

use crate::models::{LogEntry, PatternRule, Severity};

pub struct Detector {
    rules: Vec<CompiledRule>,
}

struct CompiledRule {
    rule: PatternRule,
    regex: Regex,
}

impl Detector {
    pub fn new(rules: Vec<PatternRule>) -> Self {
        let compiled: Vec<CompiledRule> = rules
            .into_iter()
            .filter_map(|rule| {
                Regex::new(&rule.pattern).ok().map(|regex| CompiledRule { rule, regex })
            })
            .collect();

        Self { rules: compiled }
    }

    pub fn default_rules() -> Vec<PatternRule> {
        vec![
            PatternRule::new(
                "Failed SSH Login",
                r"Failed password for .* from (\d+\.\d+\.\d+\.\d+)",
                Severity::Warning,
                "SSH authentication failure",
            ),
            PatternRule::new(
                "Out of Memory",
                r"Out of memory|OOM|oom-killer",
                Severity::Critical,
                "System out of memory event",
            ),
            PatternRule::new(
                "Disk Full",
                r"No space left on device",
                Severity::Critical,
                "Disk space exhausted",
            ),
            PatternRule::new(
                "Segmentation Fault",
                r"segfault|SIGSEGV|segmentation fault",
                Severity::Error,
                "Application crash - segmentation fault",
            ),
            PatternRule::new(
                "Permission Denied",
                r"Permission denied|EACCES",
                Severity::Warning,
                "Access permission error",
            ),
            PatternRule::new(
                "Connection Refused",
                r"Connection refused|ECONNREFUSED",
                Severity::Warning,
                "Network connection refused",
            ),
            PatternRule::new(
                "Stack Trace",
                r"at .+\(.+:\d+\)|Traceback|Exception|panic",
                Severity::Error,
                "Application error with stack trace",
            ),
            PatternRule::new(
                "Suspicious IP Activity",
                r"multiple failed|brute.?force|attack",
                Severity::Warning,
                "Potential security threat",
            ),
            PatternRule::new(
                "Service Restart",
                r"Stopped|Starting|Restarting|systemd.+started",
                Severity::Info,
                "Service state change",
            ),
            PatternRule::new(
                "High CPU/Memory",
                r"high.?cpu|high.?memory|resource.?limit|throttl",
                Severity::Warning,
                "Resource utilization alert",
            ),
        ]
    }

    pub fn check_line(&self, entry: &LogEntry) -> Vec<(&PatternRule, Vec<String>)> {
        let mut matches = Vec::new();

        for compiled in &self.rules {
            if !compiled.rule.enabled {
                continue;
            }

            if let Some(captures) = compiled.regex.captures(&entry.content) {
                let groups: Vec<String> = captures
                    .iter()
                    .skip(1)
                    .filter_map(|m| m.map(|m| m.as_str().to_string()))
                    .collect();
                matches.push((&compiled.rule, groups));
            }
        }

        matches
    }

    pub fn rules(&self) -> impl Iterator<Item = &PatternRule> {
        self.rules.iter().map(|c| &c.rule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_ssh_failure() {
        let rules = vec![PatternRule::new(
            "Failed SSH Login",
            r"Failed password for .* from (\d+\.\d+\.\d+\.\d+)",
            Severity::Warning,
            "SSH authentication failure",
        )];

        let detector = Detector::new(rules);
        let entry = LogEntry::new(
            "/var/log/auth.log",
            1,
            "Failed password for root from 192.168.1.100 port 22 ssh2",
        );

        let matches = detector.check_line(&entry);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0.name, "Failed SSH Login");
        assert_eq!(matches[0].1, vec!["192.168.1.100"]);
    }

    #[test]
    fn test_detector_no_match() {
        let rules = Detector::default_rules();
        let detector = Detector::new(rules);
        let entry = LogEntry::new("/var/log/syslog", 1, "Normal log message here");

        let matches = detector.check_line(&entry);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_detector_oom() {
        let rules = Detector::default_rules();
        let detector = Detector::new(rules);
        let entry = LogEntry::new("/var/log/syslog", 1, "kernel: Out of memory: Kill process");

        let matches = detector.check_line(&entry);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|(r, _)| r.name == "Out of Memory"));
    }
}
