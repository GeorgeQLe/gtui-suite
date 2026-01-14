use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Work,
    ShortBreak,
    LongBreak,
}

impl SessionType {
    pub fn label(&self) -> &'static str {
        match self {
            SessionType::Work => "Work",
            SessionType::ShortBreak => "Short Break",
            SessionType::LongBreak => "Long Break",
        }
    }
}

pub struct App {
    pub state: TimerState,
    pub session_type: SessionType,
    pub remaining: Duration,
    pub elapsed: Duration,
    pub last_tick: Option<Instant>,

    // Settings
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub sessions_until_long_break: u32,

    // Statistics
    pub completed_pomodoros: u32,
    pub total_work_time: Duration,
    pub current_streak: u32,
    pub sessions_in_cycle: u32,

    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let work_duration = Duration::from_secs(25 * 60);
        Self {
            state: TimerState::Idle,
            session_type: SessionType::Work,
            remaining: work_duration,
            elapsed: Duration::ZERO,
            last_tick: None,

            work_duration,
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,

            completed_pomodoros: 0,
            total_work_time: Duration::ZERO,
            current_streak: 0,
            sessions_in_cycle: 0,

            status_message: None,
        }
    }

    pub fn tick(&mut self) {
        if self.state != TimerState::Running {
            return;
        }

        let now = Instant::now();
        if let Some(last) = self.last_tick {
            let delta = now - last;
            self.elapsed += delta;

            if self.remaining > delta {
                self.remaining -= delta;
            } else {
                self.remaining = Duration::ZERO;
                self.finish_session();
            }
        }
        self.last_tick = Some(now);
    }

    fn finish_session(&mut self) {
        self.state = TimerState::Finished;

        if self.session_type == SessionType::Work {
            self.completed_pomodoros += 1;
            self.current_streak += 1;
            self.sessions_in_cycle += 1;
            self.total_work_time += self.work_duration;

            // Determine next session type
            if self.sessions_in_cycle >= self.sessions_until_long_break {
                self.session_type = SessionType::LongBreak;
                self.sessions_in_cycle = 0;
            } else {
                self.session_type = SessionType::ShortBreak;
            }
        } else {
            self.session_type = SessionType::Work;
        }

        self.remaining = self.duration_for_session();
        self.status_message = Some("Session complete! Press Space to start next.".to_string());
    }

    fn duration_for_session(&self) -> Duration {
        match self.session_type {
            SessionType::Work => self.work_duration,
            SessionType::ShortBreak => self.short_break_duration,
            SessionType::LongBreak => self.long_break_duration,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char(' ') => {
                self.toggle_timer();
            }
            KeyCode::Char('r') => {
                self.reset_session();
            }
            KeyCode::Char('s') => {
                self.skip_session();
            }
            KeyCode::Char('w') => {
                self.start_work_session();
            }
            KeyCode::Char('b') => {
                self.start_break_session();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.adjust_work_duration(60);
            }
            KeyCode::Char('-') => {
                self.adjust_work_duration(-60);
            }
            KeyCode::Char('R') => {
                self.reset_statistics();
            }
            _ => {}
        }
        false
    }

    fn toggle_timer(&mut self) {
        match self.state {
            TimerState::Idle | TimerState::Paused | TimerState::Finished => {
                self.state = TimerState::Running;
                self.last_tick = Some(Instant::now());
                self.status_message = None;
            }
            TimerState::Running => {
                self.state = TimerState::Paused;
                self.last_tick = None;
            }
        }
    }

    fn reset_session(&mut self) {
        self.state = TimerState::Idle;
        self.remaining = self.duration_for_session();
        self.elapsed = Duration::ZERO;
        self.last_tick = None;
    }

    fn skip_session(&mut self) {
        if self.session_type == SessionType::Work {
            self.session_type = SessionType::ShortBreak;
        } else {
            self.session_type = SessionType::Work;
        }
        self.remaining = self.duration_for_session();
        self.elapsed = Duration::ZERO;
        self.state = TimerState::Idle;
        self.last_tick = None;
    }

    fn start_work_session(&mut self) {
        self.session_type = SessionType::Work;
        self.remaining = self.work_duration;
        self.elapsed = Duration::ZERO;
        self.state = TimerState::Idle;
        self.last_tick = None;
    }

    fn start_break_session(&mut self) {
        self.session_type = SessionType::ShortBreak;
        self.remaining = self.short_break_duration;
        self.elapsed = Duration::ZERO;
        self.state = TimerState::Idle;
        self.last_tick = None;
    }

    fn adjust_work_duration(&mut self, seconds: i64) {
        let current = self.work_duration.as_secs() as i64;
        let new_duration = (current + seconds).max(60).min(60 * 60) as u64;
        self.work_duration = Duration::from_secs(new_duration);

        if self.session_type == SessionType::Work && self.state == TimerState::Idle {
            self.remaining = self.work_duration;
        }
    }

    fn reset_statistics(&mut self) {
        self.completed_pomodoros = 0;
        self.total_work_time = Duration::ZERO;
        self.current_streak = 0;
        self.sessions_in_cycle = 0;
        self.status_message = Some("Statistics reset!".to_string());
    }

    pub fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn progress(&self) -> f64 {
        let total = self.duration_for_session().as_secs_f64();
        let elapsed = (total - self.remaining.as_secs_f64()).max(0.0);
        (elapsed / total).min(1.0)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.state {
            TimerState::Idle => "Space:start r:reset s:skip w:work b:break +/-:adjust".to_string(),
            TimerState::Running => "Space:pause r:reset s:skip".to_string(),
            TimerState::Paused => "Space:resume r:reset s:skip".to_string(),
            TimerState::Finished => "Space:start next r:reset s:skip".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
