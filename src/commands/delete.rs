use crate::api::{ApiTrait, ApiClient};
use spinners::{Spinner, Spinners};

pub async fn delete_log(api: &ApiClient, worklog_id: &str) {
    let mut spinner = Spinner::new(Spinners::Dots, "Deleting worklog...".to_string());

    match api.delete_worklog(worklog_id).await {
        Ok(_) => spinner.stop_with_message(format!("Worklog {} deleted successfully", worklog_id)),
        Err(e) => spinner.stop_with_message(format!("Error. {}", e)),
    }
}
