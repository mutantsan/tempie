use crate::api::list_worklogs;
use crate::models::WorklogItem;
use crate::storage::Storage;
use crate::utils::{current_month_name, format_duration, working_seconds_in_current_month};

use tabled::builder::Builder;
use tabled::settings::object::Rows;
use tabled::{
    Table,
    settings::{Alignment, Span},
};

pub async fn list(from_date: &str, to_date: &str) {
    println!("Retrieving worklogs...\n");

    match list_worklogs(from_date, to_date).await {
        Ok(worklogs) => {
            println!("{}", build_table(worklogs));
        }
        Err(e) => eprintln!("Error. Failed to list worklogs: {}", e),
    }
}

fn build_table(worklogs: Vec<WorklogItem>) -> Table {
    let storage = Storage::new();
    let config = storage.get_credentials().unwrap();
    let working_hours = format_duration(working_seconds_in_current_month());
    let mut builder = Builder::default();
    let mut total_time = 0;

    builder.push_record(vec![
        format!("{} N/{} (-0h0m)", current_month_name(), working_hours).as_str(),
    ]);
    builder.push_record(vec!["ID", "Duration", "Description", "Issue URL"]);

    for worklog in worklogs {
        total_time += worklog.time_spent_seconds;

        builder.push_record(vec![
            worklog.tempo_worklog_id.to_string(),
            format_duration(worklog.time_spent_seconds),
            worklog.description,
            format!("{}/browse/{}", config.url, worklog.jira_issue.unwrap().key),
        ]);
    }

    builder.push_record(vec![
        format!("{}/8h", format_duration(total_time)).as_str(),
    ]);

    let mut table = builder.build();

    table.modify(Rows::first(), Span::column(4));
    table.modify(Rows::last(), Span::column(4));

    table.modify(Rows::first(), Alignment::center());
    table.modify(Rows::last(), Alignment::right());

    table
}
