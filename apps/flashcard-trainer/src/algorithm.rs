//! Spaced repetition algorithms.

use crate::models::{CardSchedule, CardState, Response, ReviewSchedule};
use chrono::{Duration, Utc};

/// Trait for spaced repetition algorithms.
pub trait SrsAlgorithm: Send + Sync {
    /// Algorithm name.
    fn name(&self) -> &str;

    /// Calculate next review schedule.
    fn calculate_next_review(&self, schedule: &CardSchedule, response: Response) -> ReviewSchedule;

    /// Get initial schedule for new cards.
    fn initial_schedule(&self) -> ReviewSchedule;
}

/// SM-2 algorithm (SuperMemo/Anki classic).
pub struct Sm2 {
    /// Initial ease factor.
    pub initial_ease: f64,
    /// Easy bonus multiplier.
    pub easy_bonus: f64,
    /// Hard interval multiplier.
    pub hard_multiplier: f64,
    /// Minimum ease factor.
    pub min_ease: f64,
}

impl Default for Sm2 {
    fn default() -> Self {
        Self {
            initial_ease: 2.5,
            easy_bonus: 1.3,
            hard_multiplier: 1.2,
            min_ease: 1.3,
        }
    }
}

impl SrsAlgorithm for Sm2 {
    fn name(&self) -> &str {
        "SM-2"
    }

    fn calculate_next_review(&self, schedule: &CardSchedule, response: Response) -> ReviewSchedule {
        let mut ease = schedule.ease_factor;
        let mut interval = schedule.interval;
        let mut state = schedule.state;

        match response {
            Response::Again => {
                // Reset to learning
                ease = (ease - 0.2).max(self.min_ease);
                interval = 1;
                state = CardState::Relearning;
            }
            Response::Hard => {
                ease = (ease - 0.15).max(self.min_ease);
                interval = ((interval as f64 * self.hard_multiplier) as i64).max(1);
                state = CardState::Review;
            }
            Response::Good => {
                if matches!(state, CardState::New | CardState::Learning) {
                    interval = 1;
                } else {
                    interval = (interval as f64 * ease) as i64;
                }
                state = CardState::Review;
            }
            Response::Easy => {
                ease += 0.15;
                interval = ((interval as f64 * ease * self.easy_bonus) as i64).max(4);
                state = CardState::Review;
            }
        }

        let next_review = Utc::now() + Duration::days(interval);

        ReviewSchedule {
            next_review,
            interval: Duration::days(interval),
            ease_factor: ease,
            state,
        }
    }

    fn initial_schedule(&self) -> ReviewSchedule {
        ReviewSchedule {
            next_review: Utc::now(),
            interval: Duration::days(0),
            ease_factor: self.initial_ease,
            state: CardState::New,
        }
    }
}

/// Leitner Box system.
pub struct Leitner {
    /// Intervals for each box (in days).
    pub box_intervals: Vec<i64>,
}

impl Default for Leitner {
    fn default() -> Self {
        Self {
            box_intervals: vec![1, 2, 4, 7, 14, 30, 60],
        }
    }
}

impl SrsAlgorithm for Leitner {
    fn name(&self) -> &str {
        "Leitner"
    }

    fn calculate_next_review(&self, schedule: &CardSchedule, response: Response) -> ReviewSchedule {
        // Use interval as box index
        let current_box = schedule.interval as usize;

        let (new_box, state) = match response {
            Response::Again => (0, CardState::Relearning),
            Response::Hard => (current_box.saturating_sub(1), CardState::Review),
            Response::Good | Response::Easy => {
                let next = (current_box + 1).min(self.box_intervals.len() - 1);
                (next, CardState::Review)
            }
        };

        let interval = self.box_intervals.get(new_box).copied().unwrap_or(1);
        let next_review = Utc::now() + Duration::days(interval);

        ReviewSchedule {
            next_review,
            interval: Duration::days(new_box as i64), // Store box index
            ease_factor: 2.5, // Leitner doesn't use ease
            state,
        }
    }

    fn initial_schedule(&self) -> ReviewSchedule {
        ReviewSchedule {
            next_review: Utc::now(),
            interval: Duration::days(0), // Box 0
            ease_factor: 2.5,
            state: CardState::New,
        }
    }
}

/// Get algorithm by name.
pub fn get_algorithm(name: &str) -> Box<dyn SrsAlgorithm> {
    match name.to_lowercase().as_str() {
        "leitner" => Box::new(Leitner::default()),
        _ => Box::new(Sm2::default()), // Default to SM-2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm2_good_response() {
        let algo = Sm2::default();
        let schedule = CardSchedule {
            card_id: uuid::Uuid::new_v4(),
            due: Utc::now(),
            interval: 1,
            ease_factor: 2.5,
            review_count: 1,
            lapses: 0,
            state: CardState::Review,
        };

        let result = algo.calculate_next_review(&schedule, Response::Good);
        assert!(result.interval.num_days() >= 2);
        assert_eq!(result.state, CardState::Review);
    }

    #[test]
    fn test_sm2_again_response() {
        let algo = Sm2::default();
        let schedule = CardSchedule {
            card_id: uuid::Uuid::new_v4(),
            due: Utc::now(),
            interval: 10,
            ease_factor: 2.5,
            review_count: 5,
            lapses: 0,
            state: CardState::Review,
        };

        let result = algo.calculate_next_review(&schedule, Response::Again);
        assert_eq!(result.interval.num_days(), 1);
        assert_eq!(result.state, CardState::Relearning);
    }

    #[test]
    fn test_leitner_progression() {
        let algo = Leitner::default();
        let schedule = CardSchedule {
            card_id: uuid::Uuid::new_v4(),
            due: Utc::now(),
            interval: 0, // Box 0
            ease_factor: 2.5,
            review_count: 0,
            lapses: 0,
            state: CardState::New,
        };

        let result = algo.calculate_next_review(&schedule, Response::Good);
        assert_eq!(result.interval.num_days(), 1); // Box 1
    }
}
