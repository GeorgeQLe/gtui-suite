//! Application state and logic.

use crate::config::Config;
use crate::db::{Database, DbResult};
use crate::models::{Habit, HabitEntry, HabitId, HabitStats, Metric, Schedule};
use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Application state.
pub struct App {
    /// Database connection.
    pub db: Database,
    /// Configuration.
    pub config: Config,
    /// Current view.
    pub view: View,
    /// Currently selected date.
    pub selected_date: NaiveDate,
    /// Habits for current view.
    pub habits: Vec<Habit>,
    /// Entries for selected date.
    pub entries: HashMap<HabitId, HabitEntry>,
    /// Selected habit index.
    pub selected_index: usize,
    /// Whether in editing mode.
    pub editing: bool,
    /// Input buffer for editing.
    pub input_buffer: String,
    /// Editing field.
    pub editing_field: EditField,
    /// Statistics cache.
    pub stats_cache: HashMap<HabitId, HabitStats>,
    /// Calendar heatmap data.
    pub heatmap: Vec<(NaiveDate, f32)>,
    /// Message to display.
    pub message: Option<(String, MessageType)>,
    /// Show help popup.
    pub show_help: bool,
    /// Confirmation dialog.
    pub confirm_dialog: Option<ConfirmDialog>,
}

/// Current view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Daily habit list.
    Daily,
    /// Calendar heatmap.
    Calendar,
    /// Streak view.
    Streaks,
    /// Statistics.
    Stats,
}

/// Editing field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    None,
    HabitName,
    HabitDescription,
    Value,
    Notes,
}

/// Message type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

/// Confirmation dialog.
#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub action: ConfirmAction,
}

/// Confirm action type.
#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteHabit(HabitId),
}

impl App {
    /// Create new application.
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();

        let db_path = Config::db_path().unwrap_or_else(|| "habits.db".into());
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::open(&db_path)?;

        let today = Utc::now().date_naive();
        let mut app = Self {
            db,
            config,
            view: View::Daily,
            selected_date: today,
            habits: Vec::new(),
            entries: HashMap::new(),
            selected_index: 0,
            editing: false,
            input_buffer: String::new(),
            editing_field: EditField::None,
            stats_cache: HashMap::new(),
            heatmap: Vec::new(),
            message: None,
            show_help: false,
            confirm_dialog: None,
        };

        app.refresh()?;
        Ok(app)
    }

    /// Refresh data from database.
    pub fn refresh(&mut self) -> DbResult<()> {
        self.habits = match self.view {
            View::Daily => self.db.get_habits_due_on(self.selected_date)?,
            _ => self.db.list_habits(false)?,
        };

        // Load entries for selected date
        self.entries.clear();
        let entries = self.db.get_entries_for_date(self.selected_date)?;
        for entry in entries {
            self.entries.insert(entry.habit_id, entry);
        }

        // Refresh stats cache
        self.stats_cache.clear();
        for habit in &self.habits {
            if let Ok(stats) = self.db.calculate_stats(habit) {
                self.stats_cache.insert(habit.id, stats);
            }
        }

        // Refresh heatmap if in calendar view
        if self.view == View::Calendar {
            self.refresh_heatmap()?;
        }

        // Ensure selected index is valid
        if self.selected_index >= self.habits.len() && !self.habits.is_empty() {
            self.selected_index = self.habits.len() - 1;
        }

        Ok(())
    }

    /// Refresh heatmap data.
    fn refresh_heatmap(&mut self) -> DbResult<()> {
        self.heatmap.clear();

        let today = Utc::now().date_naive();
        let start = today - Duration::days(365);

        let all_habits = self.db.list_habits(false)?;

        let mut date = start;
        while date <= today {
            let habits_due: Vec<_> = all_habits.iter().filter(|h| h.is_due_on(date)).collect();

            if !habits_due.is_empty() {
                let entries = self.db.get_entries_for_date(date)?;
                let completed = entries.iter().filter(|e| e.completed).count();
                let rate = completed as f32 / habits_due.len() as f32;
                self.heatmap.push((date, rate));
            }

            date = date.succ_opt().unwrap();
        }

        Ok(())
    }

    /// Check if in editing mode.
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// Get selected habit.
    pub fn selected_habit(&self) -> Option<&Habit> {
        self.habits.get(self.selected_index)
    }

    /// Get entry for selected habit.
    pub fn selected_entry(&self) -> Option<&HabitEntry> {
        self.selected_habit()
            .and_then(|h| self.entries.get(&h.id))
    }

    /// Handle key input.
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Handle confirmation dialog
        if let Some(dialog) = &self.confirm_dialog.clone() {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.execute_confirm(dialog.action.clone());
                    self.confirm_dialog = None;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirm_dialog = None;
                }
                _ => {}
            }
            return;
        }

        // Handle help popup
        if self.show_help {
            self.show_help = false;
            return;
        }

        // Clear message on any key
        self.message = None;

        // Handle editing mode
        if self.editing {
            self.handle_edit_key(key);
            return;
        }

        match key.code {
            // Navigation
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('g') if key.modifiers.is_empty() => self.selected_index = 0,
            KeyCode::Char('G') => {
                if !self.habits.is_empty() {
                    self.selected_index = self.habits.len() - 1;
                }
            }

            // Date navigation
            KeyCode::Char('h') | KeyCode::Left => self.change_date(-1),
            KeyCode::Char('l') | KeyCode::Right => self.change_date(1),
            KeyCode::Char('t') => {
                self.selected_date = Utc::now().date_naive();
                let _ = self.refresh();
            }

            // Toggle completion
            KeyCode::Char(' ') | KeyCode::Enter => self.toggle_completion(),

            // Views
            KeyCode::Char('c') => {
                self.view = View::Calendar;
                let _ = self.refresh();
            }
            KeyCode::Char('s') => {
                self.view = View::Stats;
                let _ = self.refresh();
            }
            KeyCode::Char('r') => {
                self.view = View::Streaks;
                let _ = self.refresh();
            }
            KeyCode::Char('1') => {
                self.view = View::Daily;
                let _ = self.refresh();
            }

            // Actions
            KeyCode::Char('a') => self.start_add_habit(),
            KeyCode::Char('e') => self.start_edit_habit(),
            KeyCode::Char('d') => self.confirm_delete_habit(),
            KeyCode::Char('n') => self.start_add_note(),

            // Help
            KeyCode::Char('?') => self.show_help = true,

            _ => {}
        }
    }

    /// Handle editing keys.
    fn handle_edit_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.editing = false;
                self.input_buffer.clear();
                self.editing_field = EditField::None;
            }
            KeyCode::Enter => self.finish_editing(),
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }

    /// Move selection by delta.
    fn move_selection(&mut self, delta: i32) {
        if self.habits.is_empty() {
            return;
        }

        let new_index = self.selected_index as i32 + delta;
        self.selected_index = new_index.clamp(0, self.habits.len() as i32 - 1) as usize;
    }

    /// Change selected date.
    fn change_date(&mut self, delta: i64) {
        self.selected_date = self.selected_date + Duration::days(delta);
        let _ = self.refresh();
    }

    /// Toggle completion for selected habit.
    fn toggle_completion(&mut self) {
        let Some(habit) = self.selected_habit().cloned() else {
            return;
        };

        if !habit.is_binary() {
            // For quantitative habits, start value editing
            self.editing = true;
            self.editing_field = EditField::Value;
            self.input_buffer = self
                .selected_entry()
                .and_then(|e| e.value)
                .map(|v| v.to_string())
                .unwrap_or_default();
            return;
        }

        // Toggle binary habit
        let completed = !self.entries.get(&habit.id).map_or(false, |e| e.completed);

        let entry = HabitEntry::new_binary(habit.id, self.selected_date, completed);
        if self.db.upsert_entry(&entry).is_ok() {
            self.entries.insert(habit.id, entry);
            self.message = Some((
                format!("{} {}", habit.name, if completed { "completed" } else { "uncompleted" }),
                MessageType::Success,
            ));
        }
    }

    /// Start adding a new habit.
    fn start_add_habit(&mut self) {
        self.editing = true;
        self.editing_field = EditField::HabitName;
        self.input_buffer.clear();
    }

    /// Start editing selected habit.
    fn start_edit_habit(&mut self) {
        if let Some(habit) = self.selected_habit() {
            self.editing = true;
            self.editing_field = EditField::HabitName;
            self.input_buffer = habit.name.clone();
        }
    }

    /// Confirm delete habit.
    fn confirm_delete_habit(&mut self) {
        if let Some(habit) = self.selected_habit() {
            self.confirm_dialog = Some(ConfirmDialog {
                title: "Delete Habit".to_string(),
                message: format!("Delete '{}'? This cannot be undone. (y/n)", habit.name),
                action: ConfirmAction::DeleteHabit(habit.id),
            });
        }
    }

    /// Execute confirmed action.
    fn execute_confirm(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::DeleteHabit(id) => {
                if self.db.delete_habit(id).is_ok() {
                    self.message = Some(("Habit deleted".to_string(), MessageType::Success));
                    let _ = self.refresh();
                }
            }
        }
    }

    /// Start adding note to entry.
    fn start_add_note(&mut self) {
        if self.selected_habit().is_some() {
            self.editing = true;
            self.editing_field = EditField::Notes;
            self.input_buffer = self
                .selected_entry()
                .and_then(|e| e.notes.clone())
                .unwrap_or_default();
        }
    }

    /// Finish editing and save.
    fn finish_editing(&mut self) {
        match self.editing_field {
            EditField::HabitName => {
                if !self.input_buffer.is_empty() {
                    if self.selected_habit().is_some() {
                        // Edit existing
                        if let Some(habit) = self.habits.get_mut(self.selected_index) {
                            habit.name = self.input_buffer.clone();
                            let _ = self.db.update_habit(habit);
                        }
                    } else {
                        // Add new
                        let habit = Habit::new_binary(&self.input_buffer);
                        if self.db.insert_habit(&habit).is_ok() {
                            self.message =
                                Some(("Habit created".to_string(), MessageType::Success));
                        }
                    }
                    let _ = self.refresh();
                }
            }
            EditField::Value => {
                if let Some(habit) = self.selected_habit().cloned() {
                    if let Ok(value) = self.input_buffer.parse::<f64>() {
                        let goal = habit.goal().unwrap_or(1.0);
                        let entry =
                            HabitEntry::new_quantity(habit.id, self.selected_date, value, goal);
                        if self.db.upsert_entry(&entry).is_ok() {
                            self.entries.insert(habit.id, entry);
                        }
                    }
                }
            }
            EditField::Notes => {
                if let Some(habit) = self.selected_habit().cloned() {
                    let notes = if self.input_buffer.is_empty() {
                        None
                    } else {
                        Some(self.input_buffer.clone())
                    };

                    if let Some(entry) = self.entries.get_mut(&habit.id) {
                        entry.notes = notes;
                        let _ = self.db.upsert_entry(entry);
                    } else {
                        let mut entry =
                            HabitEntry::new_binary(habit.id, self.selected_date, false);
                        entry.notes = notes;
                        let _ = self.db.upsert_entry(&entry);
                        self.entries.insert(habit.id, entry);
                    }
                }
            }
            _ => {}
        }

        self.editing = false;
        self.input_buffer.clear();
        self.editing_field = EditField::None;
    }

    /// Get completion rate for today.
    pub fn today_completion_rate(&self) -> f32 {
        if self.habits.is_empty() {
            return 0.0;
        }

        let completed = self
            .habits
            .iter()
            .filter(|h| self.entries.get(&h.id).map_or(false, |e| e.completed))
            .count();

        completed as f32 / self.habits.len() as f32
    }

    /// Get view title.
    pub fn view_title(&self) -> &str {
        match self.view {
            View::Daily => "Daily Habits",
            View::Calendar => "Calendar",
            View::Streaks => "Streaks",
            View::Stats => "Statistics",
        }
    }
}
