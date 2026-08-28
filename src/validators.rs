pub fn validate_iso8601_date(s: &str) -> Result<String, String> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|_| s.to_string())
        .map_err(|_| {
            format!(
                "Invalid date format or wrong date: '{}'. Expected YYYY-MM-DD",
                s
            )
        })
}

// Accept either YYYY-MM or YYYY-MM-DD and normalise it to YYYY-MM-DD
// (the first day of that month). Used by commands that only care about the month.
pub fn validate_month(s: &str) -> Result<String, String> {
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(date.format("%Y-%m-%d").to_string());
    }

    if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{}-01", s), "%Y-%m-%d") {
        return Ok(date.format("%Y-%m-%d").to_string());
    }

    Err(format!(
        "Invalid month format or wrong date: '{}'. Expected YYYY-MM or YYYY-MM-DD",
        s
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_month() {
        assert_eq!(validate_month("2025-04"), Ok("2025-04-01".to_string()));
        assert_eq!(validate_month("2025-04-22"), Ok("2025-04-22".to_string()));
        assert!(validate_month("2025-13").is_err());
        assert!(validate_month("2025-04-35").is_err());
        assert!(validate_month("nonsense").is_err());
    }

    #[test]
    fn test_validate_iso8601_date() {
        assert_eq!(
            validate_iso8601_date("2025-04-06"),
            Ok("2025-04-06".to_string())
        );
        assert_eq!(
            validate_iso8601_date("2025-04-35"),
            Err("Invalid date format or wrong date: '2025-04-35'. Expected YYYY-MM-DD".to_string())
        );
    }
}
