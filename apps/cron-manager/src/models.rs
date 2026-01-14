use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: Uuid,
    pub expression: CronExpression,
    pub command: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
}

impl CronJob {
    pub fn new(expression: CronExpression, command: &str) -> Self {
        let mut job = Self {
            id: Uuid::new_v4(),
            expression,
            command: command.to_string(),
            description: None,
            enabled: true,
            last_run: None,
            next_run: None,
        };
        job.next_run = job.calculate_next_run();
        job
    }

    pub fn to_crontab_line(&self) -> String {
        format!("{} {}", self.expression.to_string(), self.command)
    }

    pub fn calculate_next_run(&self) -> Option<DateTime<Utc>> {
        // Simplified - in real implementation would calculate actual next run
        Some(Utc::now() + chrono::Duration::hours(1))
    }

    pub fn human_description(&self) -> String {
        self.expression.to_human()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronExpression {
    pub minute: CronField,
    pub hour: CronField,
    pub day_of_month: CronField,
    pub month: CronField,
    pub day_of_week: CronField,
}

impl CronExpression {
    pub fn parse(expr: &str) -> Option<Self> {
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() != 5 {
            return None;
        }

        Some(Self {
            minute: CronField::parse(parts[0])?,
            hour: CronField::parse(parts[1])?,
            day_of_month: CronField::parse(parts[2])?,
            month: CronField::parse(parts[3])?,
            day_of_week: CronField::parse(parts[4])?,
        })
    }

    pub fn to_human(&self) -> String {
        let minute_desc = self.minute.describe("minute");
        let hour_desc = self.hour.describe("hour");
        let dom_desc = self.day_of_month.describe("day of month");
        let month_desc = self.month.describe("month");
        let dow_desc = self.day_of_week.describe("day of week");

        // Common patterns
        if self.is_every_minute() {
            return "Every minute".to_string();
        }

        if self.is_hourly() {
            return format!("Every hour at minute {}", self.minute.single_value().unwrap_or(0));
        }

        if self.is_daily() {
            return format!(
                "Every day at {:02}:{:02}",
                self.hour.single_value().unwrap_or(0),
                self.minute.single_value().unwrap_or(0)
            );
        }

        if self.is_weekly() {
            let day = match self.day_of_week.single_value() {
                Some(0) | Some(7) => "Sunday",
                Some(1) => "Monday",
                Some(2) => "Tuesday",
                Some(3) => "Wednesday",
                Some(4) => "Thursday",
                Some(5) => "Friday",
                Some(6) => "Saturday",
                _ => "day",
            };
            return format!(
                "Every {} at {:02}:{:02}",
                day,
                self.hour.single_value().unwrap_or(0),
                self.minute.single_value().unwrap_or(0)
            );
        }

        if self.is_monthly() {
            return format!(
                "On day {} of every month at {:02}:{:02}",
                self.day_of_month.single_value().unwrap_or(1),
                self.hour.single_value().unwrap_or(0),
                self.minute.single_value().unwrap_or(0)
            );
        }

        // Fallback to detailed description
        format!(
            "At {} {}, {}, {}, {}",
            minute_desc, hour_desc, dom_desc, month_desc, dow_desc
        )
    }

    fn is_every_minute(&self) -> bool {
        self.minute.is_any()
            && self.hour.is_any()
            && self.day_of_month.is_any()
            && self.month.is_any()
            && self.day_of_week.is_any()
    }

    fn is_hourly(&self) -> bool {
        self.minute.is_single()
            && self.hour.is_any()
            && self.day_of_month.is_any()
            && self.month.is_any()
            && self.day_of_week.is_any()
    }

    fn is_daily(&self) -> bool {
        self.minute.is_single()
            && self.hour.is_single()
            && self.day_of_month.is_any()
            && self.month.is_any()
            && self.day_of_week.is_any()
    }

    fn is_weekly(&self) -> bool {
        self.minute.is_single()
            && self.hour.is_single()
            && self.day_of_month.is_any()
            && self.month.is_any()
            && self.day_of_week.is_single()
    }

    fn is_monthly(&self) -> bool {
        self.minute.is_single()
            && self.hour.is_single()
            && self.day_of_month.is_single()
            && self.month.is_any()
            && self.day_of_week.is_any()
    }
}

impl std::fmt::Display for CronExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.minute, self.hour, self.day_of_month, self.month, self.day_of_week
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CronField {
    Any,
    Value(u32),
    Range(u32, u32),
    Step(u32),
    List(Vec<u32>),
    RangeStep(u32, u32, u32),
}

impl CronField {
    pub fn parse(s: &str) -> Option<Self> {
        if s == "*" {
            return Some(CronField::Any);
        }

        // Step: */5
        if let Some(step) = s.strip_prefix("*/") {
            if let Ok(n) = step.parse() {
                return Some(CronField::Step(n));
            }
        }

        // Range with step: 1-10/2
        if s.contains('-') && s.contains('/') {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                let range_parts: Vec<&str> = parts[0].split('-').collect();
                if range_parts.len() == 2 {
                    if let (Ok(start), Ok(end), Ok(step)) = (
                        range_parts[0].parse(),
                        range_parts[1].parse(),
                        parts[1].parse(),
                    ) {
                        return Some(CronField::RangeStep(start, end, step));
                    }
                }
            }
        }

        // Range: 1-10
        if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (parts[0].parse(), parts[1].parse()) {
                    return Some(CronField::Range(start, end));
                }
            }
        }

        // List: 1,5,10
        if s.contains(',') {
            let values: Result<Vec<u32>, _> = s.split(',').map(|v| v.parse()).collect();
            if let Ok(v) = values {
                return Some(CronField::List(v));
            }
        }

        // Single value
        if let Ok(n) = s.parse() {
            return Some(CronField::Value(n));
        }

        None
    }

    pub fn is_any(&self) -> bool {
        matches!(self, CronField::Any)
    }

    pub fn is_single(&self) -> bool {
        matches!(self, CronField::Value(_))
    }

    pub fn single_value(&self) -> Option<u32> {
        match self {
            CronField::Value(v) => Some(*v),
            _ => None,
        }
    }

    pub fn describe(&self, unit: &str) -> String {
        match self {
            CronField::Any => format!("every {}", unit),
            CronField::Value(v) => format!("{} {}", v, unit),
            CronField::Range(start, end) => format!("{} {}-{}", unit, start, end),
            CronField::Step(step) => format!("every {} {}s", step, unit),
            CronField::List(values) => {
                let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                format!("{} {}", unit, vals.join(","))
            }
            CronField::RangeStep(start, end, step) => {
                format!("every {} {}s from {} to {}", step, unit, start, end)
            }
        }
    }
}

impl std::fmt::Display for CronField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CronField::Any => write!(f, "*"),
            CronField::Value(v) => write!(f, "{}", v),
            CronField::Range(start, end) => write!(f, "{}-{}", start, end),
            CronField::Step(step) => write!(f, "*/{}", step),
            CronField::List(values) => {
                let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                write!(f, "{}", vals.join(","))
            }
            CronField::RangeStep(start, end, step) => write!(f, "{}-{}/{}", start, end, step),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronPreset {
    EveryMinute,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Reboot,
}

impl CronPreset {
    pub fn as_str(&self) -> &'static str {
        match self {
            CronPreset::EveryMinute => "Every minute",
            CronPreset::Hourly => "Hourly",
            CronPreset::Daily => "Daily (midnight)",
            CronPreset::Weekly => "Weekly (Sunday midnight)",
            CronPreset::Monthly => "Monthly (1st at midnight)",
            CronPreset::Yearly => "Yearly (Jan 1st)",
            CronPreset::Reboot => "At reboot",
        }
    }

    pub fn expression(&self) -> &'static str {
        match self {
            CronPreset::EveryMinute => "* * * * *",
            CronPreset::Hourly => "0 * * * *",
            CronPreset::Daily => "0 0 * * *",
            CronPreset::Weekly => "0 0 * * 0",
            CronPreset::Monthly => "0 0 1 * *",
            CronPreset::Yearly => "0 0 1 1 *",
            CronPreset::Reboot => "@reboot",
        }
    }

    pub fn all() -> Vec<CronPreset> {
        vec![
            CronPreset::EveryMinute,
            CronPreset::Hourly,
            CronPreset::Daily,
            CronPreset::Weekly,
            CronPreset::Monthly,
            CronPreset::Yearly,
        ]
    }
}
