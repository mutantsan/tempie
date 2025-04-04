use chrono::{Datelike, Local, NaiveDate, Weekday};
use humantime::parse_duration;
use std::time::Duration;

const WORKING_HOURS_PER_DAY: i32 = 8;
const SECONDS_PER_HOUR: i32 = 3600;

pub fn parse_duration_from_string(duration_str: &str) -> Duration {
    parse_duration(duration_str).unwrap()
}

pub fn format_duration(seconds: i32) -> String {
    let total_minutes = seconds / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    if hours == 0 && minutes == 0 {
        return "0h".to_string();
    }

    if minutes == 0 {
        return format!("{}h", hours);
    }

    format!("{}h{}m", hours, minutes)
}

// Get how many working hours in a current month
pub fn working_seconds_in_current_month() -> i32 {
    let today = Local::now().date_naive();
    let (year, month) = (today.year(), today.month());

    // Get the last day of the current month
    let last_day = NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|date| date.with_month(month + 1))
        .or_else(|| NaiveDate::from_ymd_opt(year + 1, 1, 1))
        .and_then(|date| date.pred_opt())
        .map(|date| date.day())
        .expect("Failed to get last day of month");

    // Calculate working days (excluding weekends) and multiply by 8 hours
    let working_days = (1..=last_day)
        .filter_map(|day| NaiveDate::from_ymd_opt(year, month, day))
        .filter(|date| !is_weekend(date.weekday()))
        .count() as i32;

    working_days * WORKING_HOURS_PER_DAY * SECONDS_PER_HOUR
}

fn is_weekend(weekday: Weekday) -> bool {
    weekday == Weekday::Sat || weekday == Weekday::Sun
}

pub fn current_month_name() -> String {
    let today = Local::now();
    today.format("%B").to_string()
}
