//! Tokio-based async test harness.

use crate::input::InputSequence;
use crate::terminal::TestTerminal;
use crate::{AsyncHarness, TestError, TestResult};
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Async test harness for Tokio runtime.
pub struct TokioTestHarness {
    terminal: TestTerminal,
    default_timeout: Duration,
}

impl TokioTestHarness {
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
        sleep(Duration::from_millis(16)).await; // ~60fps
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
                    // In a real app, this would be fed to the event loop
                    // For testing, we just record that it happened
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

impl AsyncHarness for TokioTestHarness {
    fn terminal(&self) -> &TestTerminal {
        &self.terminal
    }

    fn terminal_mut(&mut self) -> &mut TestTerminal {
        &mut self.terminal
    }
}

/// Builder for creating test scenarios.
pub struct ScenarioBuilder {
    harness: TokioTestHarness,
    steps: Vec<ScenarioStep>,
}

enum ScenarioStep {
    Input(InputSequence),
    Wait(Duration),
    AssertContains(String),
    AssertNotContains(String),
    Custom(Box<dyn FnOnce(&mut TokioTestHarness) -> TestResult<()> + Send>),
}

impl ScenarioBuilder {
    /// Create a new scenario builder.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            harness: TokioTestHarness::new(width, height),
            steps: Vec::new(),
        }
    }

    /// Add an input step.
    pub fn input(mut self, input: InputSequence) -> Self {
        self.steps.push(ScenarioStep::Input(input));
        self
    }

    /// Add a wait step.
    pub fn wait(mut self, duration: Duration) -> Self {
        self.steps.push(ScenarioStep::Wait(duration));
        self
    }

    /// Add an assertion that text is present.
    pub fn assert_contains(mut self, text: impl Into<String>) -> Self {
        self.steps.push(ScenarioStep::AssertContains(text.into()));
        self
    }

    /// Add an assertion that text is not present.
    pub fn assert_not_contains(mut self, text: impl Into<String>) -> Self {
        self.steps.push(ScenarioStep::AssertNotContains(text.into()));
        self
    }

    /// Add a custom step.
    pub fn custom<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut TokioTestHarness) -> TestResult<()> + Send + 'static,
    {
        self.steps.push(ScenarioStep::Custom(Box::new(f)));
        self
    }

    /// Run the scenario.
    pub async fn run(mut self) -> TestResult<()> {
        for step in self.steps {
            match step {
                ScenarioStep::Input(input) => {
                    self.harness.send_input(&input).await;
                }
                ScenarioStep::Wait(duration) => {
                    self.harness.wait(duration).await;
                }
                ScenarioStep::AssertContains(text) => {
                    self.harness.assert_contains(&text).await?;
                }
                ScenarioStep::AssertNotContains(text) => {
                    self.harness.assert_not_contains(&text).await?;
                }
                ScenarioStep::Custom(f) => {
                    f(&mut self.harness)?;
                }
            }
        }
        Ok(())
    }

    /// Get the harness for manual control.
    pub fn into_harness(self) -> TokioTestHarness {
        self.harness
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_harness_creation() {
        let harness = TokioTestHarness::new(80, 24);
        assert_eq!(harness.terminal().width(), 80);
        assert_eq!(harness.terminal().height(), 24);
    }

    #[tokio::test]
    async fn test_wait_for_render() {
        let mut harness = TokioTestHarness::new(80, 24);
        harness.wait_for_render().await;
        // Should complete without error
    }

    #[tokio::test]
    async fn test_timeout() {
        let mut harness = TokioTestHarness::new(80, 24)
            .with_timeout(Duration::from_millis(100));

        // This should timeout since the condition is never true
        let result = harness
            .run_until(|_| false, Duration::from_millis(50))
            .await;

        assert!(matches!(result, Err(TestError::Timeout)));
    }

    #[tokio::test]
    async fn test_input_sequence() {
        let mut harness = TokioTestHarness::new(80, 24);

        let mut input = InputSequence::new();
        input.text("hello").enter();

        harness.send_input(&input).await;
        // Should complete without error
    }

    #[tokio::test]
    async fn test_scenario_builder() {
        let scenario = ScenarioBuilder::new(80, 24)
            .wait(Duration::from_millis(10))
            .custom(|_harness| Ok(()));

        scenario.run().await.unwrap();
    }
}
