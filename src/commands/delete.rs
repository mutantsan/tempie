use crate::api::{ApiClient, ApiTrait};
use spinners::{Spinner, Spinners};

pub async fn delete_log(api: &ApiClient, worklog_ids: &Vec<String>) {
    let mut spinner = Spinner::new(Spinners::Dots, "Deleting worklog...".to_string());

    match api.delete_worklogs(worklog_ids).await {
        Ok(_) => spinner.stop_with_message(format!(
            "Worklog(s) deleted successfully: {}",
            worklog_ids.join(", ")
        )),
        Err(e) => spinner.stop_with_message(format!("Error. {}", e)),
    }
}
