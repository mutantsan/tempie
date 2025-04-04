mod api;
mod commands;
mod models;
mod storage;
mod utils;

use crate::commands::{delete_log, list, log_time, setup};
use chrono::Local;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure Jira credentials
    Setup,
    /// List worklogs
    List {
        #[arg(default_value_t = today_as_iso8601())]
        from_date: String,
        #[arg(default_value_t = today_as_iso8601())]
        to_date: String,
    },
    /// Log time
    Log {
        issue_key: String,
        time_spent: String,
        comment: Option<String>,
    },
    /// Delete worklog
    Delete { worklog_id: String },
}

fn today_as_iso8601() -> String {
    let today = Local::now().format("%Y-%m-%d").to_string();
    today
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => setup(),
        Commands::List { from_date, to_date } => list(&from_date, &to_date).await,
        Commands::Log {
            issue_key,
            time_spent,
            comment,
        } => log_time(&issue_key, &time_spent, comment).await,
        Commands::Delete { worklog_id } => delete_log(&worklog_id).await,
    }
}
