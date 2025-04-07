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

#[cfg(test)]
mod tests {
    use super::*;

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
