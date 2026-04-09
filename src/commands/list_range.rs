use crate::api::{ApiClient, ApiTrait};
use crate::models::WorklogItem;
use crate::utils;

use spinners::{Spinner, Spinners};

pub async fn list_range(api: &ApiClient, date_from: &str, date_to: &str) {
    let mut spinner = Spinner::new(Spinners::Dots, "Retrieving worklogs...".to_string());

    match api.list_worklogs(date_from, date_to).await {
        Ok(worklogs) => {
            spinner.stop_with_message(format!(
                "\n{}",
                build_range_output(worklogs, date_from, date_to, &api.storage)
            ));
        }
        Err(e) => {
            spinner.stop_with_message(format!("\nError. Failed to list worklogs: {}", e));
        }
    }
}

fn build_range_output(
    worklogs: Vec<WorklogItem>,
    date_from: &str,
    date_to: &str,
    storage: &crate::storage::Storage,
) -> String {
    let config = storage.get_credentials().unwrap();
    let mut out = String::new();
    let mut total_time = 0;

    out += &format!(
        "\x1b[1mWorklogs\x1b[0m \x1b[2mfrom {} to {}\x1b[0m\n\n",
        date_from, date_to
    );

    let refs: Vec<&WorklogItem> = worklogs.iter().collect();
    out += &crate::commands::list::format_worklog_entries(&refs, &config.url, &mut total_time);

    out += &format!(
        "  \x1b[1m\x1b[32m{}\x1b[0m total\n",
        utils::format_duration(total_time)
    );

    out
}
