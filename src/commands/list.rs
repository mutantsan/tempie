use crate::api::{ApiClient, ApiTrait};
use crate::models::{UserCredentials, WorklogItem};
use crate::storage::Storage;
use crate::utils;

use spinners::{Spinner, Spinners};
use tabled::{
    builder::Builder,
    settings::object::Rows,
    settings::style::BorderSpanCorrection,
    settings::{Alignment, Color, Span, Style},
    Table,
};

pub async fn list(api: &ApiClient, date: &str) {
    let mut spinner = Spinner::new(Spinners::Dots, "Retrieving worklogs...".to_string());
    let first_day = utils::get_first_day_of_month(date);
    let last_day = utils::get_last_day_of_month(date);

    match api.list_worklogs(&first_day, &last_day).await {
        Ok(worklogs) => {
            spinner.stop_with_message(format!(
                "\n{}",
                build_list_table(worklogs, &date, &api.storage)
            ));
        }
        Err(e) => {
            spinner.stop_with_message(format!("\nError. Failed to list worklogs: {}", e));
        }
    }
}

fn build_list_table(worklogs: Vec<WorklogItem>, date: &str, storage: &Storage) -> Table {
    let config = storage.get_credentials().unwrap();
    let mut builder = Builder::default();
    let mut total_time = 0;

    add_header_rows(
        &mut builder,
        calculate_total_time(&worklogs),
        utils::working_seconds_in_month(date),
        date,
    );

    add_column_headers(&mut builder);

    add_list_worklog_rows(
        &mut builder,
        &filter_out_worklogs_by_date(&worklogs, date),
        &config,
        &mut total_time,
    );
    add_list_footer_row(&mut builder, total_time);

    let mut table = builder.build();
    apply_list_table_formatting(&mut table);

    table
}

fn add_header_rows(builder: &mut Builder, worked_seconds: i32, working_seconds: i32, date: &str) {
    builder.push_record(vec![format!(
        "{} {}/{} (-{})",
        utils::get_month_name(date),
        utils::format_duration(worked_seconds),
        utils::format_duration(working_seconds),
        utils::format_duration(working_seconds - worked_seconds)
    )
    .as_str()]);

    builder.push_record(vec![format!(
        "{} ({})",
        utils::get_day_name_from_iso8601(date),
        date
    )
    .as_str()]);
}

pub fn add_column_headers(builder: &mut Builder) {
    builder.push_record(vec![
        "ID",
        "Duration",
        "Created At",
        "Description",
        "Issue URL",
    ]);
}

pub fn add_list_worklog_rows(
    builder: &mut Builder,
    worklogs: &Vec<&WorklogItem>,
    config: &UserCredentials,
    total_time: &mut i32,
) {
    for worklog in worklogs {
        *total_time += worklog.time_spent_seconds;

        builder.push_record(vec![
            worklog.tempo_worklog_id.to_string(),
            utils::format_duration(worklog.time_spent_seconds),
            chrono::DateTime::parse_from_rfc3339(&worklog.created_at)
                .unwrap()
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            truncate_string(&worklog.description, 100),
            format!(
                "{}/browse/{}",
                config.url,
                worklog.jira_issue.as_ref().unwrap().key
            ),
        ]);
    }
}

fn truncate_string(string: &str, max_length: usize) -> String {
    if string.len() > max_length {
        format!("{}...", &string[..max_length])
    } else {
        string.to_string()
    }
}

fn add_list_footer_row(builder: &mut Builder, total_time: i32) {
    builder.push_record(vec![
        format!("{}/8h", utils::format_duration(total_time)).as_str()
    ]);
}

fn apply_list_table_formatting(table: &mut Table) {
    table.modify(Rows::first(), Span::column(5));
    table.modify(Rows::single(1), Span::column(5));
    table.modify(Rows::last(), Span::column(5));

    table.modify(Rows::first(), Alignment::center());
    table.modify(Rows::single(1), Alignment::center());
    table.modify(Rows::last(), Alignment::right());

    apply_common_formatting(table)
}

pub fn apply_common_formatting(table: &mut Table) {
    table.with(Style::modern());
    table.with(BorderSpanCorrection);

    table.modify(Rows::first(), Color::BG_BLACK | Color::FG_WHITE);
    table.modify(Rows::last(), Color::BG_BLACK | Color::FG_WHITE);
}

// Calculate the total time spent in seconds this month
fn calculate_total_time(worklogs: &Vec<WorklogItem>) -> i32 {
    worklogs
        .iter()
        .map(|worklog| worklog.time_spent_seconds)
        .sum()
}

// Filter out worklogs by date provided by the user
fn filter_out_worklogs_by_date<'a>(
    worklogs: &'a Vec<WorklogItem>,
    date: &str,
) -> Vec<&'a WorklogItem> {
    worklogs
        .iter()
        .filter(|worklog| {
            let worklog_date = worklog.created_at.split('T').next().unwrap();
            worklog_date >= date && worklog_date <= date
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::{JiraIssue, TempoIssue, UserCredentials, WorklogItem};

    const TEST_DB_PATH: &str = "test_build_list_table";

    fn init_test_db(path: &str) -> Storage {
        cleanup_test_db(path);

        let storage = Storage::with_path(TEST_DB_PATH);
        storage.store_credentials(UserCredentials {
            url: "https://test.atlassian.net".to_string(),
            account_id: "test123".to_string(),
            tempo_token: "test-tempo-token".to_string(),
            jira_token: "test-jira-token".to_string(),
            jira_email: "test@example.com".to_string(),
        });

        storage
    }

    fn cleanup_test_db(path: &str) {
        let _ = std::fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn test_build_list_table() {
        let test_db_path = "test_storage_overwrite";
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

        let storage = init_test_db(test_db_path);

        let table = build_list_table(worklogs, &"2025-04-01".to_string(), &storage);
        let table_str = table.to_string();

        assert!(table_str.contains("ID"));
        assert!(table_str.contains("Duration"));
        assert!(table_str.contains("Description"));
        assert!(table_str.contains("Issue URL"));

        assert!(table_str.contains("99"));
        assert!(table_str.contains("1h"));
        assert!(table_str.contains("123"));
        assert!(table_str.contains("Test comment"));
        assert!(table_str.contains("https://test.atlassian.net/browse/TEST-123"));

        assert!(table_str.contains("1h/8h"));

        cleanup_test_db(test_db_path);
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

        let filtered_worklogs = filter_out_worklogs_by_date(&worklogs, &"2025-04-01".to_string());
        assert_eq!(filtered_worklogs.len(), 1);
        assert_eq!(filtered_worklogs[0].tempo_worklog_id, 99);

        let filtered_worklogs = filter_out_worklogs_by_date(&worklogs, &"2025-04-02".to_string());
        assert_eq!(filtered_worklogs.len(), 0);
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

        let total_time = calculate_total_time(&worklogs);
        assert_eq!(total_time, 10800);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("Hello, world!", 10), "Hello, wor...");
        assert_eq!(truncate_string("Hello, world!", 15), "Hello, world!");
    }
}
