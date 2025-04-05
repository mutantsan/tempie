use tempie::models::UserCredentials;
use tempie::storage::Storage;
use tempie::utils::*;

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
fn test_current_month_name() {
    let current_month = chrono::Local::now().format("%B").to_string();
    assert_eq!(current_month_name(), current_month);
}

#[test]
fn test_today_as_iso8601() {
    let today = today_as_iso8601();
    let parts: Vec<&str> = today.split("-").collect();

    match parts.as_slice() {
        [year, month, day] => {
            assert!(*year != "");
            assert!(*month != "");
            assert!(*day != "");
        }
        _ => {
            panic!("Impossible to parse")
        }
    }
}

#[test]
fn test_is_weekend() {
    assert_eq!(is_weekend(chrono::Weekday::Sat), true);
    assert_eq!(is_weekend(chrono::Weekday::Sun), true);
    assert_eq!(is_weekend(chrono::Weekday::Mon), false);
}

#[test]
fn test_ensure_credentials_exist() {
    let test_db_path = "test_tempie.db";

    let _ = std::fs::remove_dir_all(test_db_path);

    assert!(ensure_credentials_exist(Some(test_db_path)).is_err());

    let storage = Storage::with_path(test_db_path);

    storage.store_credentials(UserCredentials {
        url: "https://test.com".to_string(),
        account_id: "test".to_string(),
        tempo_token: "test".to_string(),
        jira_token: "test".to_string(),
        jira_email: "test@test.com".to_string(),
    });

    assert!(ensure_credentials_exist(Some(test_db_path)).is_ok());

    drop(storage);
    let _ = std::fs::remove_dir_all(test_db_path);
}

#[test]
fn test_working_seconds_in_current_month() {
    let working_seconds = working_seconds_in_current_month();
    assert!(working_seconds > 518400);
    assert!(working_seconds < 662400);
}
