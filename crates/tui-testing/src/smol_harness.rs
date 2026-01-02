//! smol-based async test harness.

use crate::input::InputSequence;
use crate::terminal::TestTerminal;
use crate::{AsyncHarness, TestError, TestResult};
use smol::Timer;
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

/// Async test harness for smol runtime.
pub struct SmolTestHarness {
    terminal: TestTerminal,
    default_timeout: Duration,
}

impl SmolTestHarness {
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
        let start = Instant::now();

        loop {
            if condition(&self.terminal) {
                return Ok(());
            }

            if start.elapsed() >= timeout_duration {
                return Err(TestError::Timeout);
            }

            Timer::after(check_interval).await;
        }
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
        Timer::after(Duration::from_millis(16)).await;
    }

    /// Wait for a specific duration.
    pub async fn wait(&mut self, duration: Duration) {
        Timer::after(duration).await;
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
                        Timer::after(duration).await;
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

    /// Run a future on the smol executor.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        smol::block_on(future)
    }
}

impl AsyncHarness for SmolTestHarness {
    fn terminal(&self) -> &TestTerminal {
        &self.terminal
    }

    fn terminal_mut(&mut self) -> &mut TestTerminal {
        &mut self.terminal
    }
}

/// Run a test with the smol runtime.
pub fn run_smol_test<F, Fut>(test_fn: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    smol::block_on(test_fn());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = SmolTestHarness::new(80, 24);
        assert_eq!(harness.terminal().width(), 80);
        assert_eq!(harness.terminal().height(), 24);
    }

    #[test]
    fn test_wait_for_render() {
        smol::block_on(async {
            let mut harness = SmolTestHarness::new(80, 24);
            harness.wait_for_render().await;
        });
    }

    #[test]
    fn test_timeout() {
        smol::block_on(async {
            let mut harness = SmolTestHarness::new(80, 24)
                .with_timeout(Duration::from_millis(100));

            let result = harness
                .run_until(|_| false, Duration::from_millis(50))
                .await;

            assert!(matches!(result, Err(TestError::Timeout)));
        });
    }

    #[test]
    fn test_run_smol_test() {
        run_smol_test(|| async {
            let harness = SmolTestHarness::new(80, 24);
            assert_eq!(harness.terminal().width(), 80);
        });
    }
}
