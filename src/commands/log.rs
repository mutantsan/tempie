use crate::api::log_time as log_time_api;
use spinners::{Spinner, Spinners};

pub async fn log_time(issue_key: &str, time_spent: &str, comment: Option<String>) {
    let mut spinner = Spinner::new(Spinners::Dots, "Logging time...".to_string());

    match log_time_api(issue_key, time_spent, comment).await {
        Ok(worklog) => {
            spinner.stop_with_message(format!(
                "\nTime logged successfully on {}, {}",
                issue_key, time_spent
            ));
            println!(
                "Run `tempie delete {}` to delete it",
                worklog.tempo_worklog_id
            );
        }
        Err(e) => {
            spinner.stop_with_message(format!("\nError. Failed to log time: {}", e));
        }
    }
}
