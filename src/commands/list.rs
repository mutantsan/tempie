use crate::api::{ApiClient, ApiTrait};
use crate::models::WorklogItem;
use crate::storage::Storage;
use crate::utils;

use spinners::{Spinner, Spinners};
use tabled::builder::Builder;
use tabled::settings::object::Rows;
use tabled::{
    Table,
    settings::{Alignment, Span},
};

pub async fn list(api: &ApiClient, from_date: &str, to_date: &str) {
    let mut spinner = Spinner::new(Spinners::Dots, "Retrieving worklogs...".to_string());

    match api.list_worklogs(from_date, to_date).await {
        Ok(worklogs) => {
            spinner.stop_with_message(format!("\n{}", build_table(worklogs, &api.storage)));
        }
        Err(e) => {
            spinner.stop_with_message(format!("\nError. Failed to list worklogs: {}", e));
        }
    }
}

fn build_table(worklogs: Vec<WorklogItem>, storage: &Storage) -> Table {
    let config = storage.get_credentials().unwrap();
    let working_hours = utils::format_duration(utils::working_seconds_in_current_month());
    let mut builder = Builder::default();
    let mut total_time = 0;

    builder.push_record(vec![
        format!(
            "{} N/{} (-0h0m)",
            utils::current_month_name(),
            working_hours
        )
        .as_str(),
    ]);
    builder.push_record(vec!["ID", "Duration", "Description", "Issue URL"]);

    for worklog in worklogs {
        total_time += worklog.time_spent_seconds;

        builder.push_record(vec![
            worklog.tempo_worklog_id.to_string(),
            utils::format_duration(worklog.time_spent_seconds),
            worklog.description,
            format!("{}/browse/{}", config.url, worklog.jira_issue.unwrap().key),
        ]);
    }

    builder.push_record(vec![
        format!("{}/8h", utils::format_duration(total_time)).as_str(),
    ]);

    let mut table = builder.build();

    table.modify(Rows::first(), Span::column(4));
    table.modify(Rows::last(), Span::column(4));

    table.modify(Rows::first(), Alignment::center());
    table.modify(Rows::last(), Alignment::right());

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::{JiraIssue, TempoIssue, UserCredentials, WorklogItem};

    const TEST_DB_PATH: &str = "test_build_table";

    fn init_test_db() -> Storage {
        cleanup_test_db();

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

    fn cleanup_test_db() {
        let _ = std::fs::remove_dir_all(TEST_DB_PATH);
    }

    #[tokio::test]
    async fn test_build_table() {
        let worklogs = vec![WorklogItem {
            tempo_worklog_id: 99,
            time_spent_seconds: 3600,
            description: "Test comment".to_string(),
            issue: TempoIssue { id: 123 },
            jira_issue: Some(JiraIssue {
                id: "123".to_string(),
                key: "TEST-123".to_string(),
            }),
        }];

        let storage = init_test_db();

        let table = build_table(worklogs, &storage);
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

        cleanup_test_db();
    }
}
