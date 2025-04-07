use crate::storage::Storage;
use spinners::{Spinner, Spinners};

pub async fn clean_jira_issues(storage: &Storage) {
    let mut spinner = Spinner::new(Spinners::Dots, "Cleaning database...".to_string());

    storage.delete_jira_issues();

    spinner.stop_with_message("Database cleaned successfully".to_string());
}
