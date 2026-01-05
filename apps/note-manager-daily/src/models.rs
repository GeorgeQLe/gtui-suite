//! Data models for daily notes.

use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use serde::{Deserialize, Serialize};

pub type EntryId = i64;

/// A daily note entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyEntry {
    pub id: EntryId,
    pub date: NaiveDate,
    pub content: String,
    pub word_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DailyEntry {
    pub fn new(date: NaiveDate) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            date,
            content: String::new(),
            word_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_word_count(&mut self) {
        self.word_count = self.content.split_whitespace().count();
    }

    pub fn preview(&self, max_lines: usize) -> String {
        self.content
            .lines()
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn formatted_date(&self) -> String {
        self.date.format("%A, %B %d, %Y").to_string()
    }
}

/// Calendar data for a month.
#[derive(Debug, Clone)]
pub struct MonthCalendar {
    pub year: i32,
    pub month: u32,
    pub days: Vec<CalendarDay>,
}

impl MonthCalendar {
    pub fn new(year: i32, month: u32) -> Self {
        let _first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let days_in_month = days_in_month(year, month);

        let days = (1..=days_in_month)
            .map(|day| {
                let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
                CalendarDay {
                    date,
                    has_entry: false,
                    is_today: date == Utc::now().date_naive(),
                    word_count: 0,
                }
            })
            .collect();

        Self { year, month, days }
    }

    pub fn month_name(&self) -> &'static str {
        match self.month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        }
    }

    pub fn first_weekday(&self) -> Weekday {
        NaiveDate::from_ymd_opt(self.year, self.month, 1)
            .unwrap()
            .weekday()
    }
}

/// A day in the calendar.
#[derive(Debug, Clone)]
pub struct CalendarDay {
    pub date: NaiveDate,
    pub has_entry: bool,
    pub is_today: bool,
    pub word_count: usize,
}

/// Statistics for the journal.
#[derive(Debug, Clone, Default)]
pub struct JournalStats {
    pub total_entries: usize,
    pub total_words: usize,
    pub current_streak: usize,
    pub longest_streak: usize,
    pub avg_words_per_entry: usize,
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) { 29 } else { 28 }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
