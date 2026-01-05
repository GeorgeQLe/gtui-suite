//! Golden file management for snapshot testing.

use crate::TestResult;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Manager for golden snapshot files.
pub struct GoldenFiles {
    /// Base directory for snapshots.
    base_path: PathBuf,
    /// Extension for snapshot files.
    extension: String,
}

impl GoldenFiles {
    /// Create a new golden files manager.
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            extension: "snap".to_string(),
        }
    }

    /// Set a custom file extension.
    pub fn with_extension(mut self, ext: impl Into<String>) -> Self {
        self.extension = ext.into();
        self
    }

    /// Get the base path.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Get the path for a snapshot.
    pub fn snapshot_path(&self, name: &str) -> PathBuf {
        self.base_path.join(format!("{}.{}", name, self.extension))
    }

    /// Check if a snapshot exists.
    pub fn exists(&self, name: &str) -> bool {
        self.snapshot_path(name).exists()
    }

    /// List all snapshots in the directory.
    pub fn list_snapshots(&self) -> TestResult<Vec<String>> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();

        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    if path.extension().and_then(|e| e.to_str()) == Some(&self.extension) {
                        snapshots.push(name.to_string());
                    }
                }
            }
        }

        snapshots.sort();
        Ok(snapshots)
    }

    /// Delete a snapshot.
    pub fn delete(&self, name: &str) -> TestResult<()> {
        let path = self.snapshot_path(name);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Clean up orphaned snapshots.
    ///
    /// Takes a set of expected snapshot names and removes any files not in the set.
    pub fn cleanup(&self, expected: &HashSet<String>) -> TestResult<CleanupReport> {
        let mut report = CleanupReport {
            removed: Vec::new(),
            kept: Vec::new(),
        };

        for name in self.list_snapshots()? {
            let path = self.snapshot_path(&name);
            if expected.contains(&name) {
                report.kept.push(path);
            } else {
                std::fs::remove_file(&path)?;
                report.removed.push(path);
            }
        }

        Ok(report)
    }

    /// Verify snapshot coverage.
    ///
    /// Checks that all expected snapshots exist and reports any missing or extra files.
    pub fn verify_coverage(&self, expected: &HashSet<String>) -> TestResult<CoverageReport> {
        let existing: HashSet<String> = self.list_snapshots()?.into_iter().collect();

        let missing: Vec<String> = expected.difference(&existing).cloned().collect();
        let extra: Vec<String> = existing.difference(expected).cloned().collect();

        Ok(CoverageReport {
            total_expected: expected.len(),
            total_existing: existing.len(),
            missing,
            extra,
        })
    }

    /// Create the snapshots directory if it doesn't exist.
    pub fn ensure_dir(&self) -> TestResult<()> {
        if !self.base_path.exists() {
            std::fs::create_dir_all(&self.base_path)?;
        }
        Ok(())
    }

    /// Get total size of all snapshot files.
    pub fn total_size(&self) -> TestResult<u64> {
        if !self.base_path.exists() {
            return Ok(0);
        }

        let mut total = 0u64;
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.path().is_file() {
                total += entry.metadata()?.len();
            }
        }
        Ok(total)
    }
}

impl Default for GoldenFiles {
    fn default() -> Self {
        Self::new("tests/snapshots")
    }
}

/// Report from cleaning up orphaned snapshots.
#[derive(Debug, Clone)]
pub struct CleanupReport {
    /// Files that were removed.
    pub removed: Vec<PathBuf>,
    /// Files that were kept.
    pub kept: Vec<PathBuf>,
}

impl CleanupReport {
    /// Check if any files were removed.
    pub fn had_orphans(&self) -> bool {
        !self.removed.is_empty()
    }

    /// Get the number of files removed.
    pub fn removed_count(&self) -> usize {
        self.removed.len()
    }

    /// Get the number of files kept.
    pub fn kept_count(&self) -> usize {
        self.kept.len()
    }
}

impl std::fmt::Display for CleanupReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cleanup Report:")?;
        writeln!(f, "  Kept: {} files", self.kept.len())?;
        writeln!(f, "  Removed: {} files", self.removed.len())?;

        if !self.removed.is_empty() {
            writeln!(f, "\nRemoved files:")?;
            for path in &self.removed {
                writeln!(f, "  - {}", path.display())?;
            }
        }

        Ok(())
    }
}

/// Report from verifying snapshot coverage.
#[derive(Debug, Clone)]
pub struct CoverageReport {
    /// Total expected snapshots.
    pub total_expected: usize,
    /// Total existing snapshots.
    pub total_existing: usize,
    /// Missing snapshots (expected but not found).
    pub missing: Vec<String>,
    /// Extra snapshots (found but not expected).
    pub extra: Vec<String>,
}

impl CoverageReport {
    /// Check if coverage is complete.
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.extra.is_empty()
    }

    /// Check if there are missing snapshots.
    pub fn has_missing(&self) -> bool {
        !self.missing.is_empty()
    }

    /// Check if there are extra snapshots.
    pub fn has_extra(&self) -> bool {
        !self.extra.is_empty()
    }

    /// Get coverage percentage.
    pub fn coverage_percent(&self) -> f64 {
        if self.total_expected == 0 {
            return 100.0;
        }
        let found = self.total_expected - self.missing.len();
        (found as f64 / self.total_expected as f64) * 100.0
    }
}

impl std::fmt::Display for CoverageReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Coverage Report:")?;
        writeln!(f, "  Expected: {} snapshots", self.total_expected)?;
        writeln!(f, "  Existing: {} snapshots", self.total_existing)?;
        writeln!(f, "  Coverage: {:.1}%", self.coverage_percent())?;

        if !self.missing.is_empty() {
            writeln!(f, "\nMissing snapshots:")?;
            for name in &self.missing {
                writeln!(f, "  - {}", name)?;
            }
        }

        if !self.extra.is_empty() {
            writeln!(f, "\nExtra snapshots:")?;
            for name in &self.extra {
                writeln!(f, "  - {}", name)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_golden_files_creation() {
        let gf = GoldenFiles::new("/tmp/test_snapshots");
        assert_eq!(gf.base_path(), Path::new("/tmp/test_snapshots"));
    }

    #[test]
    fn test_snapshot_path() {
        let gf = GoldenFiles::new("/tmp/test");
        assert_eq!(
            gf.snapshot_path("my_test"),
            PathBuf::from("/tmp/test/my_test.snap")
        );
    }

    #[test]
    fn test_custom_extension() {
        let gf = GoldenFiles::new("/tmp/test").with_extension("json");
        assert_eq!(
            gf.snapshot_path("my_test"),
            PathBuf::from("/tmp/test/my_test.json")
        );
    }

    #[test]
    fn test_list_snapshots() {
        let dir = tempdir().unwrap();
        let gf = GoldenFiles::new(dir.path());

        // Create some test files
        fs::write(gf.snapshot_path("test1"), b"content").unwrap();
        fs::write(gf.snapshot_path("test2"), b"content").unwrap();
        fs::write(dir.path().join("other.txt"), b"not a snapshot").unwrap();

        let snapshots = gf.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 2);
        assert!(snapshots.contains(&"test1".to_string()));
        assert!(snapshots.contains(&"test2".to_string()));
    }

    #[test]
    fn test_cleanup() {
        let dir = tempdir().unwrap();
        let gf = GoldenFiles::new(dir.path());

        // Create files
        fs::write(gf.snapshot_path("keep1"), b"content").unwrap();
        fs::write(gf.snapshot_path("keep2"), b"content").unwrap();
        fs::write(gf.snapshot_path("orphan"), b"content").unwrap();

        let expected: HashSet<String> = ["keep1", "keep2"].iter().map(|s| s.to_string()).collect();
        let report = gf.cleanup(&expected).unwrap();

        assert_eq!(report.kept_count(), 2);
        assert_eq!(report.removed_count(), 1);
        assert!(!gf.exists("orphan"));
    }

    #[test]
    fn test_coverage() {
        let dir = tempdir().unwrap();
        let gf = GoldenFiles::new(dir.path());

        // Create some files
        fs::write(gf.snapshot_path("test1"), b"content").unwrap();
        fs::write(gf.snapshot_path("extra"), b"content").unwrap();

        let expected: HashSet<String> = ["test1", "missing"].iter().map(|s| s.to_string()).collect();
        let report = gf.verify_coverage(&expected).unwrap();

        assert!(report.has_missing());
        assert!(report.has_extra());
        assert_eq!(report.missing, vec!["missing"]);
        assert_eq!(report.extra, vec!["extra"]);
    }

    #[test]
    fn test_ensure_dir() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("nested").join("path");
        let gf = GoldenFiles::new(&nested);

        assert!(!nested.exists());
        gf.ensure_dir().unwrap();
        assert!(nested.exists());
    }
}
