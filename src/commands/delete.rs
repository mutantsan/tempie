use crate::api::delete_worklog;

pub async fn delete_log(worklog_id: &str) {
    match delete_worklog(worklog_id).await {
        Ok(_) => println!("Worklog {} deleted successfully", worklog_id),
        Err(e) => eprintln!("Error. {}", e),
    }
}
