mod api;
mod commands;
mod models;
mod storage;
mod utils;
mod validators;

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
        #[arg(
            default_value_t = utils::today_as_iso8601(),
            help = "The date to list worklogs from (format: YYYY-MM-DD)",
            value_parser = validators::validate_iso8601_date
        )]
        date: String,
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
    /// Delete worklog(s)
    Delete {
        #[arg(help = "Worklog ID(s) to delete", num_args = 1..)]
        worklog_ids: Vec<String>,
    },
    /// Clean jira issues from database
    CleanJiraIssues,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let storage = storage::Storage::new();

    match cli.command {
        Commands::Setup => commands::setup(&storage),
        Commands::CleanJiraIssues => commands::clean_jira_issues(&storage).await,
        _ => {}
    }

    if let Err(err) = utils::ensure_credentials_exist(&storage) {
        eprintln!("{}", err);
        std::process::exit(1);
    }

    let api = api::ApiClient::new(storage);

    match cli.command {
        Commands::Setup => {}
        Commands::CleanJiraIssues => {}
        Commands::List { date } => commands::list(&api, &date).await,
        Commands::Log {
            issue_key,
            time_spent,
            comment,
        } => commands::log_time(&api, &issue_key, &time_spent, comment).await,
        Commands::Delete { worklog_ids } => commands::delete_log(&api, &worklog_ids).await,
    }
}
