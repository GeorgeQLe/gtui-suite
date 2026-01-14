use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;

const SAMPLE_TEXTS: &[&str] = &[
    "The quick brown fox jumps over the lazy dog.",
    "Pack my box with five dozen liquor jugs.",
    "How vexingly quick daft zebras jump!",
    "The five boxing wizards jump quickly.",
    "Sphinx of black quartz, judge my vow.",
    "Two driven jocks help fax my big quiz.",
    "The jay, pig, fox, zebra and my wolves quack!",
    "Sympathizing would fix Quaker objectives.",
    "A wizard's job is to vex chumps quickly in fog.",
    "Watch Jeopardy!, Alex Trebek's fun TV quiz game.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestState {
    Ready,
    Running,
    Finished,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub wpm: f64,
    pub accuracy: f64,
    pub time_seconds: f64,
    pub correct: usize,
    pub incorrect: usize,
}

pub struct App {
    pub target_text: String,
    pub typed_text: String,
    pub state: TestState,
    pub start_time: Option<Instant>,
    pub results: Vec<TestResult>,
    pub current_result: Option<TestResult>,
    pub errors: Vec<usize>,
    text_index: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            target_text: SAMPLE_TEXTS[0].to_string(),
            typed_text: String::new(),
            state: TestState::Ready,
            start_time: None,
            results: Vec::new(),
            current_result: None,
            errors: Vec::new(),
            text_index: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.state {
            TestState::Ready => self.handle_ready_key(key),
            TestState::Running => self.handle_running_key(key),
            TestState::Finished => self.handle_finished_key(key),
        }
    }

    fn handle_ready_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('n') => self.next_text(),
            KeyCode::Char(c) => {
                self.state = TestState::Running;
                self.start_time = Some(Instant::now());
                self.typed_text.push(c);
                self.check_character(0);
            }
            _ => {}
        }
        false
    }

    fn handle_running_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.reset();
            }
            KeyCode::Backspace => {
                if !self.typed_text.is_empty() {
                    let idx = self.typed_text.len() - 1;
                    self.errors.retain(|&e| e != idx);
                    self.typed_text.pop();
                }
            }
            KeyCode::Char(c) => {
                let idx = self.typed_text.len();
                self.typed_text.push(c);
                self.check_character(idx);

                if self.typed_text.len() >= self.target_text.len() {
                    self.finish();
                }
            }
            _ => {}
        }
        false
    }

    fn handle_finished_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('r') | KeyCode::Enter => self.reset(),
            KeyCode::Char('n') => {
                self.next_text();
                self.reset();
            }
            _ => {}
        }
        false
    }

    fn check_character(&mut self, idx: usize) {
        if let (Some(typed), Some(target)) = (
            self.typed_text.chars().nth(idx),
            self.target_text.chars().nth(idx),
        ) {
            if typed != target {
                self.errors.push(idx);
            }
        }
    }

    fn finish(&mut self) {
        self.state = TestState::Finished;

        let elapsed = self.start_time.map(|t| t.elapsed().as_secs_f64()).unwrap_or(1.0);
        let words = self.target_text.split_whitespace().count() as f64;
        let wpm = (words / elapsed) * 60.0;

        let correct = self.typed_text.len() - self.errors.len();
        let accuracy = (correct as f64 / self.typed_text.len() as f64) * 100.0;

        let result = TestResult {
            wpm,
            accuracy,
            time_seconds: elapsed,
            correct,
            incorrect: self.errors.len(),
        };

        self.current_result = Some(result.clone());
        self.results.push(result);
    }

    fn reset(&mut self) {
        self.typed_text.clear();
        self.errors.clear();
        self.state = TestState::Ready;
        self.start_time = None;
        self.current_result = None;
    }

    fn next_text(&mut self) {
        self.text_index = (self.text_index + 1) % SAMPLE_TEXTS.len();
        self.target_text = SAMPLE_TEXTS[self.text_index].to_string();
        self.reset();
    }

    pub fn tick(&mut self) {
        // Update timer display during running state
    }

    pub fn elapsed_seconds(&self) -> f64 {
        self.start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0)
    }

    pub fn live_wpm(&self) -> f64 {
        if self.state != TestState::Running || self.typed_text.is_empty() {
            return 0.0;
        }

        let elapsed = self.elapsed_seconds();
        if elapsed < 0.5 {
            return 0.0;
        }

        let chars = self.typed_text.len() as f64;
        let words = chars / 5.0; // Standard: 5 chars = 1 word
        (words / elapsed) * 60.0
    }

    pub fn live_accuracy(&self) -> f64 {
        if self.typed_text.is_empty() {
            return 100.0;
        }
        let correct = self.typed_text.len() - self.errors.len();
        (correct as f64 / self.typed_text.len() as f64) * 100.0
    }

    pub fn average_wpm(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.results.iter().map(|r| r.wpm).sum::<f64>() / self.results.len() as f64
    }

    pub fn status_text(&self) -> String {
        match self.state {
            TestState::Ready => "Start typing to begin | n:next-text q:quit".to_string(),
            TestState::Running => "Type the text above | Esc:reset".to_string(),
            TestState::Finished => "r/Enter:retry n:next-text q:quit".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
