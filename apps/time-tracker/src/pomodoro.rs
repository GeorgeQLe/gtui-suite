//! Pomodoro timer functionality.

use crate::config::PomodoroConfig;
use chrono::{DateTime, Duration, Utc};

/// Pomodoro session type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Work,
    ShortBreak,
    LongBreak,
}

impl SessionType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Work => "Work",
            Self::ShortBreak => "Short Break",
            Self::LongBreak => "Long Break",
        }
    }

    pub fn is_break(&self) -> bool {
        matches!(self, Self::ShortBreak | Self::LongBreak)
    }
}

/// Pomodoro timer state.
#[derive(Debug, Clone)]
pub struct PomodoroTimer {
    /// Configuration.
    config: PomodoroConfig,
    /// Current session type.
    session_type: SessionType,
    /// When session started (None if paused).
    started_at: Option<DateTime<Utc>>,
    /// Time remaining when paused.
    paused_remaining: Option<Duration>,
    /// Completed pomodoros in current cycle.
    completed_pomodoros: u32,
    /// Total completed pomodoros today.
    total_pomodoros: u32,
    /// Whether timer is active.
    active: bool,
}

impl PomodoroTimer {
    /// Create new timer with config.
    pub fn new(config: PomodoroConfig) -> Self {
        Self {
            config,
            session_type: SessionType::Work,
            started_at: None,
            paused_remaining: None,
            completed_pomodoros: 0,
            total_pomodoros: 0,
            active: false,
        }
    }

    /// Start or resume timer.
    pub fn start(&mut self) {
        if !self.active {
            self.active = true;
            self.started_at = Some(Utc::now());
        }
    }

    /// Pause timer.
    pub fn pause(&mut self) {
        if self.active {
            self.paused_remaining = Some(self.remaining());
            self.started_at = None;
            self.active = false;
        }
    }

    /// Toggle timer.
    pub fn toggle(&mut self) {
        if self.active {
            self.pause();
        } else {
            self.start();
        }
    }

    /// Reset current session.
    pub fn reset(&mut self) {
        self.started_at = None;
        self.paused_remaining = None;
        self.active = false;
    }

    /// Skip to next session.
    pub fn skip(&mut self) {
        self.complete_session();
    }

    /// Complete current session and move to next.
    fn complete_session(&mut self) {
        match self.session_type {
            SessionType::Work => {
                self.completed_pomodoros += 1;
                self.total_pomodoros += 1;

                if self.completed_pomodoros >= self.config.pomodoros_before_long {
                    self.session_type = SessionType::LongBreak;
                    self.completed_pomodoros = 0;
                } else {
                    self.session_type = SessionType::ShortBreak;
                }

                if self.config.auto_start_breaks {
                    self.start();
                } else {
                    self.reset();
                }
            }
            SessionType::ShortBreak | SessionType::LongBreak => {
                self.session_type = SessionType::Work;

                if self.config.auto_start_work {
                    self.start();
                } else {
                    self.reset();
                }
            }
        }
    }

    /// Get session duration.
    pub fn session_duration(&self) -> Duration {
        let mins = match self.session_type {
            SessionType::Work => self.config.work_mins,
            SessionType::ShortBreak => self.config.short_break_mins,
            SessionType::LongBreak => self.config.long_break_mins,
        };
        Duration::minutes(mins as i64)
    }

    /// Get remaining time.
    pub fn remaining(&self) -> Duration {
        if let Some(remaining) = self.paused_remaining {
            return remaining;
        }

        let Some(started) = self.started_at else {
            return self.session_duration();
        };

        let elapsed = Utc::now().signed_duration_since(started);
        let remaining = self.session_duration() - elapsed;

        if remaining < Duration::zero() {
            Duration::zero()
        } else {
            remaining
        }
    }

    /// Get elapsed time.
    pub fn elapsed(&self) -> Duration {
        self.session_duration() - self.remaining()
    }

    /// Get progress (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        let total = self.session_duration().num_seconds() as f64;
        let elapsed = self.elapsed().num_seconds() as f64;
        (elapsed / total).min(1.0)
    }

    /// Check if session is complete.
    pub fn is_complete(&self) -> bool {
        self.remaining() <= Duration::zero()
    }

    /// Tick the timer (call each frame).
    pub fn tick(&mut self) -> bool {
        if self.active && self.is_complete() {
            self.complete_session();
            return true; // Session completed
        }
        false
    }

    /// Get current session type.
    pub fn session_type(&self) -> SessionType {
        self.session_type
    }

    /// Check if active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get completed pomodoros.
    pub fn completed_pomodoros(&self) -> u32 {
        self.completed_pomodoros
    }

    /// Get total pomodoros today.
    pub fn total_pomodoros(&self) -> u32 {
        self.total_pomodoros
    }

    /// Format remaining time.
    pub fn format_remaining(&self) -> String {
        let remaining = self.remaining();
        let mins = remaining.num_minutes();
        let secs = remaining.num_seconds() % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Get pomodoros until long break.
    pub fn until_long_break(&self) -> u32 {
        self.config.pomodoros_before_long - self.completed_pomodoros
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = PomodoroTimer::new(PomodoroConfig::default());
        assert!(!timer.is_active());
        assert_eq!(timer.session_type(), SessionType::Work);
    }

    #[test]
    fn test_timer_toggle() {
        let mut timer = PomodoroTimer::new(PomodoroConfig::default());

        timer.toggle();
        assert!(timer.is_active());

        timer.toggle();
        assert!(!timer.is_active());
    }

    #[test]
    fn test_session_duration() {
        let config = PomodoroConfig {
            work_mins: 25,
            short_break_mins: 5,
            long_break_mins: 15,
            pomodoros_before_long: 4,
            auto_start_breaks: false,
            auto_start_work: false,
        };
        let timer = PomodoroTimer::new(config);

        assert_eq!(timer.session_duration(), Duration::minutes(25));
    }
}
