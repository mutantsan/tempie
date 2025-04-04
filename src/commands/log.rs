use crate::api::log_time as log_time_api;

pub async fn log_time(issue_key: &str, time_spent: &str, comment: Option<String>) {
    match log_time_api(issue_key, time_spent, comment).await {
        Ok(_) => println!("Time logged successfully on {}", issue_key),
        Err(e) => eprintln!("Error. Failed to log time: {}", e),
    }
}
