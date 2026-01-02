//! Testing utilities for TUI applications.
//!
//! This crate provides comprehensive testing tools for TUI apps including:
//! - Snapshot testing with structured buffer comparisons
//! - Input simulation for keyboard and mouse events
//! - Async test harnesses for multiple runtimes
//! - Property-based testing generators
//! - Deterministic fixtures and random data generators

pub mod ci;
pub mod diff;
pub mod fixtures;
pub mod golden;
pub mod input;
pub mod snapshot;
pub mod terminal;
pub mod widget_tests;

#[cfg(feature = "tokio-harness")]
pub mod tokio_harness;

#[cfg(feature = "async-std-harness")]
pub mod async_std_harness;

#[cfg(feature = "smol-harness")]
pub mod smol_harness;

#[cfg(feature = "proptest-support")]
pub mod generators;

// Re-exports
pub use ci::{is_ci, should_update_snapshots, CiConfig};
pub use diff::{CellChanges, CellDiff, SnapshotDiff};
pub use fixtures::Fixtures;
pub use golden::{CleanupReport, CoverageReport, GoldenFiles};
pub use input::InputSequence;
pub use snapshot::{BufferSnapshot, CapturedFrame, CellSnapshot, SnapshotTest};
pub use terminal::TestTerminal;

#[cfg(feature = "tokio-harness")]
pub use tokio_harness::TokioTestHarness;

#[cfg(feature = "async-std-harness")]
pub use async_std_harness::AsyncStdTestHarness;

#[cfg(feature = "smol-harness")]
pub use smol_harness::SmolTestHarness;

/// Environment variable to update snapshots.
pub const UPDATE_SNAPSHOTS: &str = "UPDATE_SNAPSHOTS";

/// Current snapshot format version.
pub const SNAPSHOT_FORMAT_VERSION: u32 = 1;

/// Error types for testing operations.
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Snapshot mismatch: {0}")]
    SnapshotMismatch(String),

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Timeout waiting for condition")]
    Timeout,

    #[error("Terminal error: {0}")]
    Terminal(String),
}

/// Result type for testing operations.
pub type TestResult<T> = Result<T, TestError>;

/// Common trait for async test harnesses.
pub trait AsyncHarness {
    /// Get a reference to the test terminal.
    fn terminal(&self) -> &TestTerminal;

    /// Get a mutable reference to the test terminal.
    fn terminal_mut(&mut self) -> &mut TestTerminal;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(UPDATE_SNAPSHOTS, "UPDATE_SNAPSHOTS");
        assert_eq!(SNAPSHOT_FORMAT_VERSION, 1);
    }
}
