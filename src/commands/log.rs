use crate::api::{ApiTrait, ApiClient};
use spinners::{Spinner, Spinners};

pub async fn log_time(api: &ApiClient, issue_key: &str, time_spent: &str, comment: Option<String>) {
    let mut spinner = Spinner::new(Spinners::Dots, "Logging time...".to_string());

    match api.log_time(issue_key, time_spent, comment).await {
        Ok(worklog) => {
            spinner.stop_with_message(format!(
                "\nTime logged successfully on {}, {}",
                issue_key, time_spent
            ));
            println!(
                "Run 'tempie delete {}' to delete it",
                worklog.tempo_worklog_id
            );
        }
        Err(e) => {
            spinner.stop_with_message(format!("\nError. Failed to log time: {}", e));
        }
    }
}
