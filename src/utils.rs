use crate::storage::Storage;
use chrono::{Datelike, Local, NaiveDate, Weekday};
use humantime::parse_duration;

const WORKING_HOURS_PER_DAY: i32 = 8;
const SECONDS_PER_HOUR: i32 = 3600;

// Parse a duration from a string. hours:minutes -> seconds
pub fn parse_duration_from_string(duration_str: &str) -> i32 {
    if duration_str.is_empty() {
        return 0;
    }

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
pub fn working_seconds_in_month(date: &str) -> i32 {
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
    let (year, month) = (date.year(), date.month());

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

// Get the first day of the month in ISO 8601 format
pub fn get_first_day_of_month(date: &str) -> String {
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
    date.format("%Y-%m-01").to_string()
}

// Get the last day of the month in ISO 8601 format
pub fn get_last_day_of_month(date: &str) -> String {
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
    let (year, month) = (date.year(), date.month());

    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or(NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
        .pred_opt()
        .unwrap()
        .format("%Y-%m-%d")
        .to_string()
}

// Check if a day is a weekend
pub fn is_weekend(weekday: Weekday) -> bool {
    weekday == Weekday::Sat || weekday == Weekday::Sun
}

// Get the month name from an ISO 8601 date string, e.g "2025-04-01" -> "April"
pub fn get_month_name(date: &str) -> String {
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
    date.format("%B").to_string()
}

// Get today's date in ISO 8601 format
pub fn today_as_iso8601() -> String {
    let today = Local::now().format("%Y-%m-%d").to_string();
    today
}

// Get the day name from an ISO 8601 date string, e.g "2025-04-06" -> "Sunday"
pub fn get_day_name_from_iso8601(date_string: &str) -> String {
    let date = NaiveDate::parse_from_str(date_string, "%Y-%m-%d").unwrap();
    date.format("%A").to_string()
}

// Ensure credentials exist and exit if they don't
pub fn ensure_credentials_exist(storage: &Storage) -> Result<(), String> {
    let config = storage.get_credentials();

    if config.is_none() {
        return Err("Credentials are not set up. Please run `tempie setup` first.".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserCredentials;

    fn test_date_string_format(date_string: &str) -> Result<(), String> {
        let parts: Vec<&str> = date_string.split('-').collect();

        if parts.len() != 3 {
            return Err("Invalid date format: expected YYYY-MM-DD".to_string());
        }

        let [year, month, day] = parts.as_slice() else {
            return Err("Invalid date format: expected YYYY-MM-DD".to_string());
        };

        if year.is_empty() || month.is_empty() || day.is_empty() {
            return Err("Invalid date format: empty components".to_string());
        }

        Ok(())
    }

    #[test]
    fn test_parse_duration_from_string() {
        // Test hours
        assert_eq!(parse_duration_from_string("1h"), 3600);
        assert_eq!(parse_duration_from_string("2h"), 7200);

        // Test hours
        assert_eq!(parse_duration_from_string("1h"), 3600);
        assert_eq!(parse_duration_from_string("2h"), 7200);

        // Test minutes
        assert_eq!(parse_duration_from_string("30m"), 1800);
        assert_eq!(parse_duration_from_string("45m"), 2700);

        // Test hours and minutes
        assert_eq!(parse_duration_from_string("1h30m"), 5400);
        assert_eq!(parse_duration_from_string("2h45m"), 9900);

        // Test days
        assert_eq!(parse_duration_from_string("1d"), 86400);
        assert_eq!(parse_duration_from_string("2d"), 172800);

        // Test edge cases
        assert_eq!(parse_duration_from_string("0h"), 0);
        assert_eq!(parse_duration_from_string("0m"), 0);
        assert_eq!(parse_duration_from_string("55s"), 55);
        assert_eq!(parse_duration_from_string(""), 0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(900), "15m");
        assert_eq!(format_duration(1800), "30m");
        assert_eq!(format_duration(3600), "1h");
        assert_eq!(format_duration(5400), "1h30m");
        assert_eq!(format_duration(86400), "24h");
        assert_eq!(format_duration(155555), "43h12m");

        // Test edge cases
        assert_eq!(format_duration(0), "0h");
        assert_eq!(format_duration(1), "0h");
        assert_eq!(format_duration(61), "1m");
    }

    #[test]
    fn test_get_month_name() {
        assert_eq!(get_month_name("2025-04-01"), "April");
        assert_eq!(get_month_name("2025-05-01"), "May");
        assert_eq!(get_month_name("2025-06-30"), "June");
    }

    #[test]
    fn test_today_as_iso8601() {
        let today = today_as_iso8601();
        assert!(test_date_string_format(&today).is_ok());
    }

    #[test]
    fn test_ensure_credentials_exist() {
        let test_db_path = "test_ensure_credentials_exist";

        let _ = std::fs::remove_dir_all(test_db_path);
        let storage = Storage::with_path(test_db_path);

        assert!(ensure_credentials_exist(&storage).is_err());

        storage.store_credentials(UserCredentials {
            url: "https://test.com".to_string(),
            account_id: "test".to_string(),
            tempo_token: "test".to_string(),
            jira_token: "test".to_string(),
            jira_email: "test@test.com".to_string(),
        });

        assert!(ensure_credentials_exist(&storage).is_ok());

        let _ = std::fs::remove_dir_all(test_db_path);
    }

    #[test]
    fn test_working_seconds_in_month() {
        let working_seconds = working_seconds_in_month("2025-04-01");
        assert_eq!(working_seconds, 633600);
    }

    #[test]
    fn test_get_first_day_of_month() {
        let first_day = get_first_day_of_month("2025-04-22");
        assert_eq!(first_day, "2025-04-01");
    }

    #[test]
    fn test_get_last_day_of_month() {
        let last_day = get_last_day_of_month("2025-04-22");
        assert_eq!(last_day, "2025-04-30");
    }

    #[test]
    fn test_get_day_name_from_iso8601() {
        assert_eq!(get_day_name_from_iso8601("2025-04-06"), "Sunday");
        assert_eq!(get_day_name_from_iso8601("2025-04-07"), "Monday");
        assert_eq!(get_day_name_from_iso8601("2025-04-09"), "Wednesday");
        assert_eq!(get_day_name_from_iso8601("2025-04-11"), "Friday");
    }
}
