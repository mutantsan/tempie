use crate::storage::Storage;
use chrono::{Datelike, Local, NaiveDate, Weekday};
use humantime::parse_duration;

const WORKING_HOURS_PER_DAY: i32 = 8;
const SECONDS_PER_HOUR: i32 = 3600;

// Parse a duration from a string. hours:minutes -> seconds
pub fn parse_duration_from_string(duration_str: &str) -> i32 {
    parse_duration(duration_str).unwrap().as_secs() as i32
}

// Format a duration in hours and minutes. seconds -> hours:minutes
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

    if hours == 0 {
        return format!("{}m", minutes);
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

// Check if a day is a weekend
fn is_weekend(weekday: Weekday) -> bool {
    weekday == Weekday::Sat || weekday == Weekday::Sun
}

// Get the current month name
pub fn current_month_name() -> String {
    let today = Local::now();
    today.format("%B").to_string()
}

// Get today's date in ISO 8601 format
pub fn today_as_iso8601() -> String {
    let today = Local::now().format("%Y-%m-%d").to_string();
    today
}

// Ensure credentials exist and exit if they don't
pub fn ensure_credentials_exist() -> () {
    let storage = Storage::new();
    let config = storage.get_credentials();

    if !config.is_some() {
        eprintln!("Credentials are not set up. Please run `tempie setup` first.");
        std::process::exit(1);
    }
}
