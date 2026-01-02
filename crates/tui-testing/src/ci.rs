//! CI integration utilities.

use std::env;

/// Environment variable to update snapshots.
pub const UPDATE_SNAPSHOTS_VAR: &str = "UPDATE_SNAPSHOTS";

/// Environment variable for CI detection.
pub const CI_VAR: &str = "CI";

/// Check if running in a CI environment.
pub fn is_ci() -> bool {
    env::var(CI_VAR).is_ok()
        || env::var("CONTINUOUS_INTEGRATION").is_ok()
        || env::var("GITHUB_ACTIONS").is_ok()
        || env::var("GITLAB_CI").is_ok()
        || env::var("TRAVIS").is_ok()
        || env::var("CIRCLECI").is_ok()
        || env::var("JENKINS_URL").is_ok()
}

/// Check if snapshots should be updated.
pub fn should_update_snapshots() -> bool {
    env::var(UPDATE_SNAPSHOTS_VAR).is_ok()
}

/// Configuration for CI-specific behavior.
#[derive(Debug, Clone)]
pub struct CiConfig {
    /// Whether to use colored output.
    pub colored_output: bool,
    /// Whether to show verbose diffs.
    pub verbose_diffs: bool,
    /// Whether to fail on first error.
    pub fail_fast: bool,
    /// Maximum number of diff lines to show.
    pub max_diff_lines: usize,
    /// Whether running in CI.
    pub is_ci: bool,
}

impl CiConfig {
    /// Create configuration from environment variables.
    pub fn from_env() -> Self {
        let is_ci = is_ci();

        Self {
            // Disable colors in CI by default (unless explicitly enabled)
            colored_output: !is_ci || env::var("FORCE_COLOR").is_ok(),
            // Show verbose diffs in CI
            verbose_diffs: is_ci || env::var("VERBOSE_DIFFS").is_ok(),
            // Fail fast in CI by default
            fail_fast: is_ci && env::var("NO_FAIL_FAST").is_err(),
            // More diff lines in verbose mode
            max_diff_lines: if is_ci { 100 } else { 20 },
            is_ci,
        }
    }

    /// Create local development configuration.
    pub fn local() -> Self {
        Self {
            colored_output: true,
            verbose_diffs: false,
            fail_fast: false,
            max_diff_lines: 20,
            is_ci: false,
        }
    }

    /// Create CI-specific configuration.
    pub fn ci() -> Self {
        Self {
            colored_output: false,
            verbose_diffs: true,
            fail_fast: true,
            max_diff_lines: 100,
            is_ci: true,
        }
    }

    /// Set colored output.
    pub fn with_colors(mut self, enabled: bool) -> Self {
        self.colored_output = enabled;
        self
    }

    /// Set verbose diffs.
    pub fn with_verbose_diffs(mut self, enabled: bool) -> Self {
        self.verbose_diffs = enabled;
        self
    }

    /// Set fail fast behavior.
    pub fn with_fail_fast(mut self, enabled: bool) -> Self {
        self.fail_fast = enabled;
        self
    }

    /// Set maximum diff lines.
    pub fn with_max_diff_lines(mut self, lines: usize) -> Self {
        self.max_diff_lines = lines;
        self
    }
}

impl Default for CiConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Test result reporter for CI.
#[derive(Debug, Default)]
pub struct TestReporter {
    /// Number of passed tests.
    pub passed: usize,
    /// Number of failed tests.
    pub failed: usize,
    /// Number of skipped tests.
    pub skipped: usize,
    /// Failure messages.
    pub failures: Vec<TestFailure>,
}

/// A test failure record.
#[derive(Debug, Clone)]
pub struct TestFailure {
    /// Test name.
    pub name: String,
    /// Failure message.
    pub message: String,
    /// Optional diff output.
    pub diff: Option<String>,
}

impl TestReporter {
    /// Create a new reporter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a passed test.
    pub fn pass(&mut self, _name: &str) {
        self.passed += 1;
    }

    /// Record a failed test.
    pub fn fail(&mut self, name: &str, message: &str, diff: Option<String>) {
        self.failed += 1;
        self.failures.push(TestFailure {
            name: name.to_string(),
            message: message.to_string(),
            diff,
        });
    }

    /// Record a skipped test.
    pub fn skip(&mut self, _name: &str) {
        self.skipped += 1;
    }

    /// Get total test count.
    pub fn total(&self) -> usize {
        self.passed + self.failed + self.skipped
    }

    /// Check if all tests passed.
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Generate a summary report.
    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!("Test Results:"),
            format!("  Passed:  {}", self.passed),
            format!("  Failed:  {}", self.failed),
            format!("  Skipped: {}", self.skipped),
            format!("  Total:   {}", self.total()),
        ];

        if !self.failures.is_empty() {
            lines.push(String::new());
            lines.push("Failures:".to_string());

            for failure in &self.failures {
                lines.push(format!("  {} - {}", failure.name, failure.message));
                if let Some(diff) = &failure.diff {
                    for line in diff.lines().take(20) {
                        lines.push(format!("    {}", line));
                    }
                }
            }
        }

        lines.join("\n")
    }

    /// Exit with appropriate code.
    pub fn exit(&self) -> ! {
        std::process::exit(if self.all_passed() { 0 } else { 1 })
    }
}

/// Utilities for GitHub Actions.
pub mod github_actions {
    /// Output an error annotation.
    pub fn error(message: &str, file: Option<&str>, line: Option<usize>) {
        let location = match (file, line) {
            (Some(f), Some(l)) => format!(" file={},line={}", f, l),
            (Some(f), None) => format!(" file={}", f),
            _ => String::new(),
        };
        println!("::error{}::{}", location, message);
    }

    /// Output a warning annotation.
    pub fn warning(message: &str, file: Option<&str>, line: Option<usize>) {
        let location = match (file, line) {
            (Some(f), Some(l)) => format!(" file={},line={}", f, l),
            (Some(f), None) => format!(" file={}", f),
            _ => String::new(),
        };
        println!("::warning{}::{}", location, message);
    }

    /// Start a group in the log.
    pub fn group(name: &str) {
        println!("::group::{}", name);
    }

    /// End a group in the log.
    pub fn endgroup() {
        println!("::endgroup::");
    }

    /// Set an output variable.
    pub fn set_output(name: &str, value: &str) {
        println!("::set-output name={}::{}", name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_config_local() {
        let config = CiConfig::local();
        assert!(config.colored_output);
        assert!(!config.verbose_diffs);
        assert!(!config.fail_fast);
        assert!(!config.is_ci);
    }

    #[test]
    fn test_ci_config_ci() {
        let config = CiConfig::ci();
        assert!(!config.colored_output);
        assert!(config.verbose_diffs);
        assert!(config.fail_fast);
        assert!(config.is_ci);
    }

    #[test]
    fn test_reporter() {
        let mut reporter = TestReporter::new();

        reporter.pass("test1");
        reporter.pass("test2");
        reporter.fail("test3", "assertion failed", None);
        reporter.skip("test4");

        assert_eq!(reporter.total(), 4);
        assert_eq!(reporter.passed, 2);
        assert_eq!(reporter.failed, 1);
        assert_eq!(reporter.skipped, 1);
        assert!(!reporter.all_passed());
    }

    #[test]
    fn test_reporter_summary() {
        let mut reporter = TestReporter::new();
        reporter.pass("test1");
        reporter.fail("test2", "error", Some("diff here".to_string()));

        let summary = reporter.summary();
        assert!(summary.contains("Passed:  1"));
        assert!(summary.contains("Failed:  1"));
        assert!(summary.contains("test2 - error"));
    }
}
