use crate::api::{ApiClient, ApiTrait};
use crate::models::WorklogItem;
use crate::utils;

use chrono::{Datelike, Local, NaiveDate};
use spinners::{Spinner, Spinners};
use std::collections::HashMap;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const FG_GREEN: &str = "\x1b[32m";
const FG_YELLOW: &str = "\x1b[33m";
const FG_RED: &str = "\x1b[31m";

pub async fn month(api: &ApiClient, date: &str) {
    let mut spinner = Spinner::new(Spinners::Dots, "Retrieving worklogs...".to_string());
    let first_day = utils::get_first_day_of_month(date);
    let last_day = utils::get_last_day_of_month(date);

    match api.list_worklogs(&first_day, &last_day).await {
        Ok(worklogs) => {
            spinner.stop_with_message(format!("\n{}", build_month_output(worklogs, date)));
        }
        Err(e) => {
            spinner.stop_with_message(format!("\nError. Failed to list worklogs: {}", e));
        }
    }
}

// The day a worklog belongs to. Prefer Tempo's startDate, fall back to createdAt.
fn worklog_date(worklog: &WorklogItem) -> String {
    if !worklog.start_date.is_empty() {
        worklog.start_date.clone()
    } else {
        worklog
            .created_at
            .split('T')
            .next()
            .unwrap_or_default()
            .to_string()
    }
}

fn build_month_output(worklogs: Vec<WorklogItem>, date: &str) -> String {
    let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
    let (year, month_num) = (parsed.year(), parsed.month());
    let today = Local::now().date_naive();

    let mut logged_by_day: HashMap<String, i32> = HashMap::new();
    for worklog in &worklogs {
        *logged_by_day.entry(worklog_date(worklog)).or_insert(0) += worklog.time_spent_seconds;
    }

    let last_day = NaiveDate::parse_from_str(&utils::get_last_day_of_month(date), "%Y-%m-%d")
        .unwrap()
        .day();

    let mut out = String::new();
    out += &format!("{BOLD}{}{RESET}\n\n", parsed.format("%B %Y"));
    out += &format!(
        "{DIM}  {:<3} {:<10}  {:>8}  {:>8}  {:>8}{RESET}\n",
        "", "Date", "Logged", "Target", "Diff"
    );

    let mut logged_total = 0;

    for day in 1..=last_day {
        let current = NaiveDate::from_ymd_opt(year, month_num, day).unwrap();
        let iso = current.format("%Y-%m-%d").to_string();
        let is_weekend = utils::is_weekend(current.weekday());
        let logged = *logged_by_day.get(&iso).unwrap_or(&0);
        let target = utils::working_seconds_in_day(&iso);

        logged_total += logged;

        let color = if is_weekend || current > today {
            DIM
        } else if logged >= target {
            FG_GREEN
        } else if logged == 0 {
            FG_RED
        } else {
            FG_YELLOW
        };

        let logged_str = if logged == 0 {
            "·".to_string()
        } else {
            utils::format_duration(logged)
        };
        let target_str = if is_weekend {
            "·".to_string()
        } else {
            utils::format_duration(target)
        };
        let diff_str = if is_weekend && logged == 0 {
            String::new()
        } else {
            utils::format_signed(logged - target)
        };
        let marker = if current == today { "›" } else { " " };

        out += &format!(
            "{color}{} {:<3} {:<10}  {:>8}  {:>8}  {:>8}{RESET}\n",
            marker,
            current.format("%a"),
            iso,
            logged_str,
            target_str,
            diff_str,
        );
    }

    let capacity = utils::working_seconds_in_month(date);

    out += &format!("  {}\n", "─".repeat(46));
    out += &format!(
        "  {BOLD}{:<16}{RESET}{:>8}  {:>8}  {}{:>8}{RESET}\n",
        "Total",
        utils::format_duration(logged_total),
        utils::format_duration(capacity),
        if logged_total >= capacity {
            FG_GREEN
        } else {
            FG_YELLOW
        },
        utils::format_signed(logged_total - capacity),
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{JiraIssue, TempoIssue, WorklogItem};

    fn worklog(start_date: &str, seconds: i32) -> WorklogItem {
        WorklogItem {
            tempo_worklog_id: 1,
            time_spent_seconds: seconds,
            description: "Test".to_string(),
            created_at: format!("{}T09:00:00Z", start_date),
            start_date: start_date.to_string(),
            issue: TempoIssue { id: 1 },
            jira_issue: Some(JiraIssue {
                id: "1".to_string(),
                key: "TEST-1".to_string(),
            }),
        }
    }

    #[test]
    fn test_worklog_date_falls_back_to_created_at() {
        let mut item = worklog("2025-04-01", 3600);
        item.start_date = String::new();
        assert_eq!(worklog_date(&item), "2025-04-01");
    }

    #[test]
    fn test_build_month_output() {
        let worklogs = vec![
            worklog("2025-04-01", 3600),
            worklog("2025-04-01", 1800),
            worklog("2025-04-02", 8 * 3600),
        ];

        let output = build_month_output(worklogs, "2025-04-15");

        assert!(output.contains("April 2025"));
        assert!(output.contains("2025-04-01"));
        assert!(output.contains("2025-04-30"));
        // 1h30m logged on the 1st
        assert!(output.contains("1h30m"));
        assert!(output.contains("Total"));
        // April 2025 has 176 working hours
        assert!(output.contains("176h"));
        // Logged total is 1h30m + 8h = 9h30m
        assert!(output.contains("9h30m"));
    }
}
