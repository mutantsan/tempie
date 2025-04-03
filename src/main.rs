mod api;
mod commands;
mod storage;

use crate::commands::{list, log_time, setup};
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
        #[arg(default_value_t = String::from("today"))]
        from_date: String,
        #[arg(default_value_t = String::from("today"))]
        to_date: String,
    },
    /// Log time
    Log {
        issue_key: String,
        time_spent: String,
        comment: Option<String>,
    },
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
    }
}
