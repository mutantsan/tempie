use crate::api::{ApiClient, ApiTrait};
use crate::models::WorklogItem;
use crate::storage::Storage;
use crate::utils;

use spinners::{Spinner, Spinners};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const FG_GREEN: &str = "\x1b[32m";
const FG_YELLOW: &str = "\x1b[33m";
const FG_CYAN: &str = "\x1b[36m";

pub async fn list(api: &ApiClient, date: &str) {
    let mut spinner = Spinner::new(Spinners::Dots, "Retrieving worklogs...".to_string());
    let first_day = utils::get_first_day_of_month(date);
    let last_day = utils::get_last_day_of_month(date);

    match api.list_worklogs(&first_day, &last_day).await {
        Ok(worklogs) => {
            spinner.stop_with_message(format!(
                "\n{}",
                build_list_output(worklogs, date, &api.storage)
            ));
        }
        Err(e) => {
            spinner.stop_with_message(format!("\nError. Failed to list worklogs: {}", e));
        }
    }
}

fn build_list_output(worklogs: Vec<WorklogItem>, date: &str, storage: &Storage) -> String {
    let config = storage.get_credentials().unwrap();
    let mut out = String::new();

    let worked = calculate_total_time(&worklogs);
    let capacity = utils::working_seconds_in_month(date);
    out += &format!(
        "{}{}{} {}/{} {DIM}(-{}){RESET}\n",
        BOLD,
        utils::get_month_name(date),
        RESET,
        utils::format_duration(worked),
        utils::format_duration(capacity),
        utils::format_duration(capacity - worked),
    );
    out += &format!(
        "{DIM}{} ({}){RESET}\n",
        utils::get_day_name_from_iso8601(date),
        date
    );

    let day_worklogs = filter_out_worklogs_by_date(&worklogs, date);
    let mut daily_total = 0;

    if !day_worklogs.is_empty() {
        out += "\n";
        out += &format_worklog_entries(&day_worklogs, &config.url, &mut daily_total);
    }

    out += &format!(
        "  {BOLD}{FG_GREEN}{}{RESET}/8h\n",
        utils::format_duration(daily_total)
    );

    out
}

pub fn format_worklog_entries(
    worklogs: &[&WorklogItem],
    jira_base_url: &str,
    total_time: &mut i32,
) -> String {
    let mut out = String::new();

    for worklog in worklogs {
        *total_time += worklog.time_spent_seconds;

        let key = &worklog.jira_issue.as_ref().unwrap().key;
        let url = format!("{}/browse/{}", jira_base_url, key);
        let hyperlink = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, key);
        let time = chrono::DateTime::parse_from_rfc3339(&worklog.created_at)
            .unwrap()
            .with_timezone(&chrono::Local)
            .format("%m-%d %H:%M")
            .to_string();

        out += &format!(
            "  {BOLD}{FG_YELLOW}#{}{RESET}  {FG_GREEN}{}{RESET}  {DIM}{}{RESET}  {FG_CYAN}{}{RESET}\n",
            worklog.tempo_worklog_id,
            utils::format_duration(worklog.time_spent_seconds),
            time,
            hyperlink,
        );
        out += &format!("  {DIM}{}{RESET}\n", truncate_string(&worklog.description, 200));
        out += &format!("\n{DIM}  {}{RESET}\n\n", "─".repeat(40));
    }

    out
}

fn truncate_string(string: &str, max_length: usize) -> String {
    if string.len() > max_length {
        format!("{}...", &string[..max_length])
    } else {
        string.to_string()
    }
}

// Calculate the total time spent in seconds this month
fn calculate_total_time(worklogs: &[WorklogItem]) -> i32 {
    worklogs.iter().map(|w| w.time_spent_seconds).sum()
}

// Filter out worklogs by date provided by the user
fn filter_out_worklogs_by_date<'a>(
    worklogs: &'a [WorklogItem],
    date: &str,
) -> Vec<&'a WorklogItem> {
    worklogs
        .iter()
        .filter(|w| {
            let worklog_date = w.created_at.split('T').next().unwrap();
            worklog_date == date
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::{JiraIssue, TempoIssue, UserCredentials, WorklogItem};
    use crate::storage::Storage;

    fn make_test_storage(path: &str) -> Storage {
        let _ = std::fs::remove_dir_all(path);
        let storage = Storage::with_path(path);
        storage.store_credentials(UserCredentials {
            url: "https://test.atlassian.net".to_string(),
            account_id: "test123".to_string(),
            tempo_token: "test-tempo-token".to_string(),
            jira_token: "test-jira-token".to_string(),
            jira_email: "test@example.com".to_string(),
        });
        storage
    }

    #[tokio::test]
    async fn test_build_list_output() {
        let test_db_path = "test_build_list_output";
        let storage = make_test_storage(test_db_path);
        let worklogs = vec![WorklogItem {
            tempo_worklog_id: 99,
            time_spent_seconds: 3600,
            description: "Test comment".to_string(),
            created_at: "2025-04-01T00:00:00Z".to_string(),
            issue: TempoIssue { id: 123 },
            jira_issue: Some(JiraIssue {
                id: "123".to_string(),
                key: "TEST-123".to_string(),
            }),
        }];

        let output = build_list_output(worklogs, "2025-04-01", &storage);

        assert!(output.contains("April"));
        assert!(output.contains("1h"));
        assert!(output.contains("99"));
        assert!(output.contains("Test comment"));
        assert!(output.contains("TEST-123"));
        assert!(output.contains("https://test.atlassian.net/browse/TEST-123"));
        assert!(output.contains("/8h"));

        let _ = std::fs::remove_dir_all(test_db_path);
    }

    #[tokio::test]
    async fn test_filter_out_worklogs_by_date() {
        let worklogs = vec![WorklogItem {
            tempo_worklog_id: 99,
            time_spent_seconds: 3600,
            description: "Test comment".to_string(),
            created_at: "2025-04-01T00:00:00Z".to_string(),
            issue: TempoIssue { id: 123 },
            jira_issue: Some(JiraIssue {
                id: "123".to_string(),
                key: "TEST-123".to_string(),
            }),
        }];

        let filtered = filter_out_worklogs_by_date(&worklogs, "2025-04-01");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].tempo_worklog_id, 99);

        let filtered = filter_out_worklogs_by_date(&worklogs, "2025-04-02");
        assert_eq!(filtered.len(), 0);
    }

    #[tokio::test]
    async fn test_calculate_total_time() {
        let worklogs = vec![
            WorklogItem {
                tempo_worklog_id: 99,
                time_spent_seconds: 3600,
                description: "Test comment".to_string(),
                created_at: "2025-04-01T00:00:00Z".to_string(),
                issue: TempoIssue { id: 123 },
                jira_issue: Some(JiraIssue {
                    id: "123".to_string(),
                    key: "TEST-123".to_string(),
                }),
            },
            WorklogItem {
                tempo_worklog_id: 100,
                time_spent_seconds: 7200,
                description: "Test comment 2".to_string(),
                created_at: "2025-04-01T00:00:00Z".to_string(),
                issue: TempoIssue { id: 123 },
                jira_issue: Some(JiraIssue {
                    id: "123".to_string(),
                    key: "TEST-123".to_string(),
                }),
            },
        ];

        assert_eq!(calculate_total_time(&worklogs), 10800);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("Hello, world!", 10), "Hello, wor...");
        assert_eq!(truncate_string("Hello, world!", 15), "Hello, world!");
    }
}
