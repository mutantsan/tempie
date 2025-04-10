use crate::api::{ApiClient, ApiTrait};
use crate::models::WorklogItem;
use crate::storage::Storage;
use crate::utils;

use spinners::{Spinner, Spinners};
use tabled::{
    builder::Builder,
    settings::object::Rows,
    settings::{Alignment, Span},
    Table,
};

pub async fn list_range(api: &ApiClient, date_from: &str, date_to: &str) {
    let mut spinner = Spinner::new(Spinners::Dots, "Retrieving worklogs...".to_string());

    match api.list_worklogs(&date_from, &date_to).await {
        Ok(worklogs) => {
            spinner.stop_with_message(format!(
                "\n{}",
                build_range_table(worklogs, &date_from, &date_to, &api.storage)
            ));
        }
        Err(e) => {
            spinner.stop_with_message(format!("\nError. Failed to list worklogs: {}", e));
        }
    }
}

fn build_range_table(
    worklogs: Vec<WorklogItem>,
    date_from: &str,
    date_to: &str,
    storage: &Storage,
) -> Table {
    let config = storage.get_credentials().unwrap();
    let mut builder = Builder::default();
    let mut total_time = 0;

    builder.push_record(vec![
        format!("Worklogs from {} to {}", date_from, date_to).as_str()
    ]);

    crate::commands::list::add_column_headers(&mut builder);

    crate::commands::list::add_list_worklog_rows(
        &mut builder,
        &worklogs.iter().collect::<Vec<_>>(),
        &config,
        &mut total_time,
    );

    builder.push_record(vec![
        format!("{}", utils::format_duration(total_time)).as_str()
    ]);

    let mut table = builder.build();
    apply_range_table_formatting(&mut table);

    table
}

fn apply_range_table_formatting(table: &mut Table) {
    table.modify(Rows::first(), Span::column(5));
    table.modify(Rows::last(), Span::column(5));

    table.modify(Rows::first(), Alignment::center());
    table.modify(Rows::last(), Alignment::right());

    crate::commands::list::apply_common_formatting(table)
}
