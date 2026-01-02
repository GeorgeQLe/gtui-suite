//! async-std based async test harness.

use crate::input::InputSequence;
use crate::terminal::TestTerminal;
use crate::{AsyncHarness, TestError, TestResult};
use async_std::future::timeout;
use async_std::task::sleep;
use std::time::Duration;

/// Async test harness for async-std runtime.
pub struct AsyncStdTestHarness {
    terminal: TestTerminal,
    default_timeout: Duration,
}

impl AsyncStdTestHarness {
    /// Create a new test harness.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            terminal: TestTerminal::new(width, height),
            default_timeout: Duration::from_secs(5),
        }
    }

    /// Set the default timeout for async operations.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Run until a condition is met or timeout occurs.
    pub async fn run_until<F>(&mut self, condition: F, timeout_duration: Duration) -> TestResult<()>
    where
        F: Fn(&TestTerminal) -> bool,
    {
        let check_interval = Duration::from_millis(10);

        timeout(timeout_duration, async {
            loop {
                if condition(&self.terminal) {
                    return Ok(());
                }
                sleep(check_interval).await;
            }
        })
        .await
        .map_err(|_| TestError::Timeout)?
    }

    /// Wait for the terminal to contain specific text.
    pub async fn wait_for_text(&mut self, text: &str) -> TestResult<()> {
        let text = text.to_string();
        self.run_until(|term| term.to_string().contains(&text), self.default_timeout)
            .await
    }

    /// Wait for the terminal to not contain specific text.
    pub async fn wait_for_no_text(&mut self, text: &str) -> TestResult<()> {
        let text = text.to_string();
        self.run_until(|term| !term.to_string().contains(&text), self.default_timeout)
            .await
    }

    /// Wait for a render cycle.
    pub async fn wait_for_render(&mut self) {
        sleep(Duration::from_millis(16)).await;
    }

    /// Wait for a specific duration.
    pub async fn wait(&mut self, duration: Duration) {
        sleep(duration).await;
    }

    /// Process an input sequence.
    pub async fn send_input(&mut self, input: &InputSequence) {
        use crate::input::SequenceItem;

        for item in input.iter() {
            match item {
                SequenceItem::Event(event) => {
                    let _ = event;
                }
                SequenceItem::Delay(duration) => {
                    if input.uses_real_timing() {
                        sleep(duration).await;
                    }
                }
            }
        }
    }

    /// Process input and wait for a condition.
    pub async fn send_input_and_wait<F>(
        &mut self,
        input: &InputSequence,
        condition: F,
    ) -> TestResult<()>
    where
        F: Fn(&TestTerminal) -> bool,
    {
        self.send_input(input).await;
        self.run_until(condition, self.default_timeout).await
    }

    /// Assert terminal contains text with retry.
    pub async fn assert_contains(&mut self, text: &str) -> TestResult<()> {
        self.wait_for_text(text).await
    }

    /// Assert terminal does not contain text.
    pub async fn assert_not_contains(&mut self, text: &str) -> TestResult<()> {
        self.wait_for_no_text(text).await
    }

    /// Resize the terminal.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.terminal.resize(width, height);
    }

    /// Clear the terminal.
    pub fn clear(&mut self) {
        self.terminal.clear();
    }
}

impl AsyncHarness for AsyncStdTestHarness {
    fn terminal(&self) -> &TestTerminal {
        &self.terminal
    }

    fn terminal_mut(&mut self) -> &mut TestTerminal {
        &mut self.terminal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn test_harness_creation() {
        let harness = AsyncStdTestHarness::new(80, 24);
        assert_eq!(harness.terminal().width(), 80);
        assert_eq!(harness.terminal().height(), 24);
    }

    #[async_std::test]
    async fn test_wait_for_render() {
        let mut harness = AsyncStdTestHarness::new(80, 24);
        harness.wait_for_render().await;
    }

    #[async_std::test]
    async fn test_timeout() {
        let mut harness = AsyncStdTestHarness::new(80, 24)
            .with_timeout(Duration::from_millis(100));

        let result = harness
            .run_until(|_| false, Duration::from_millis(50))
            .await;

        assert!(matches!(result, Err(TestError::Timeout)));
    }
}
