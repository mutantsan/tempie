mod api;
mod commands;
mod models;
mod storage;
mod utils;

use crate::commands::{delete_log, list, log_time, setup};
use crate::utils::{ensure_credentials_exist, today_as_iso8601};
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
        #[arg(default_value_t = today_as_iso8601(), help = "The start date to list worklogs from (format: YYYY-MM-DD)")]
        from_date: String,
        #[arg(default_value_t = today_as_iso8601(), help = "The end date to list worklogs to (format: YYYY-MM-DD)")]
        to_date: String,
    },
    /// Log time
    Log {
        #[arg(help = "The Jira issue key to log time against (e.g., XXX-123)")]
        issue_key: String,
        #[arg(help = "The time spent to log (e.g., 1h30m)")]
        time_spent: String,
        #[arg(help = "The comment to add to the worklog. Optional.")]
        comment: Option<String>,
    },
    /// Delete worklog
    Delete {
        #[arg(help = "Worklog ID to delete")]
        worklog_id: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let api = api::ApiClient::new();

    match cli.command {
        Commands::Setup => setup(&api),
        Commands::List { from_date, to_date } => {
            if let Err(err) = ensure_credentials_exist(&api.storage) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
            list(&api, &from_date, &to_date).await
        }
        Commands::Log {
            issue_key,
            time_spent,
            comment,
        } => {
            if let Err(err) = ensure_credentials_exist(&api.storage) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
            log_time(&api, &issue_key, &time_spent, comment).await
        }
        Commands::Delete { worklog_id } => {
            if let Err(err) = ensure_credentials_exist(&api.storage) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
            delete_log(&api, &worklog_id).await
        }
    }
}
