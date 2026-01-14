use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TextStats {
    pub characters: usize,
    pub characters_no_spaces: usize,
    pub words: usize,
    pub sentences: usize,
    pub paragraphs: usize,
    pub lines: usize,
    pub avg_word_length: f64,
    pub avg_sentence_length: f64,
    pub reading_time_mins: f64,
    pub speaking_time_mins: f64,
    pub word_frequency: Vec<(String, usize)>,
}

impl TextStats {
    pub fn analyze(text: &str) -> Self {
        let characters = text.chars().count();
        let characters_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count();

        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();

        let sentences = text
            .chars()
            .filter(|c| *c == '.' || *c == '!' || *c == '?')
            .count()
            .max(1);

        let paragraphs = text
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .count()
            .max(1);

        let lines = text.lines().count().max(1);

        let avg_word_length = if word_count > 0 {
            words.iter().map(|w| w.len()).sum::<usize>() as f64 / word_count as f64
        } else {
            0.0
        };

        let avg_sentence_length = if sentences > 0 {
            word_count as f64 / sentences as f64
        } else {
            0.0
        };

        // Average reading speed: 200-250 words per minute
        let reading_time_mins = word_count as f64 / 225.0;

        // Average speaking speed: 125-150 words per minute
        let speaking_time_mins = word_count as f64 / 137.5;

        // Word frequency
        let mut freq_map: HashMap<String, usize> = HashMap::new();
        for word in &words {
            let normalized = word.to_lowercase()
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>();
            if !normalized.is_empty() {
                *freq_map.entry(normalized).or_insert(0) += 1;
            }
        }

        let mut word_frequency: Vec<(String, usize)> = freq_map.into_iter().collect();
        word_frequency.sort_by(|a, b| b.1.cmp(&a.1));
        word_frequency.truncate(20);

        Self {
            characters,
            characters_no_spaces,
            words: word_count,
            sentences,
            paragraphs,
            lines,
            avg_word_length,
            avg_sentence_length,
            reading_time_mins,
            speaking_time_mins,
            word_frequency,
        }
    }
}

pub struct App {
    pub text: String,
    pub stats: TextStats,
    pub selected_word: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let default_text = "The quick brown fox jumps over the lazy dog. This is a sample text for demonstration purposes. You can type or paste your own text to analyze it.\n\nThis is the second paragraph. It contains multiple sentences! How many words are there? Let's find out.\n\nType your text here to see detailed statistics including word count, character count, reading time, and word frequency analysis.";

        let stats = TextStats::analyze(default_text);

        Self {
            text: default_text.to_string(),
            stats,
            selected_word: 0,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Backspace => {
                self.text.pop();
                self.update_stats();
            }
            KeyCode::Enter => {
                self.text.push('\n');
                self.update_stats();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_word < self.stats.word_frequency.len().saturating_sub(1) {
                    self.selected_word += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_word = self.selected_word.saturating_sub(1);
            }
            KeyCode::Char(c) if key.modifiers.is_empty() => {
                self.text.push(c);
                self.update_stats();
            }
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.text.clear();
                self.update_stats();
                self.status_message = Some("Text cleared".to_string());
            }
            _ => {}
        }
        false
    }

    fn update_stats(&mut self) {
        self.stats = TextStats::analyze(&self.text);
        self.selected_word = self.selected_word.min(
            self.stats.word_frequency.len().saturating_sub(1)
        );
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "Type text to analyze | j/k:scroll-freq Ctrl+L:clear q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
