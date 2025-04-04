use crate::storage::{UserCredentials, Storage};
use dialoguer::{Input, Password};

const JIRA_API_TOKENS_URL: &str = "https://id.atlassian.com/manage-profile/security/api-tokens";
const TEMPO_API_INTEGRATION_URL: &str =
    "/plugins/servlet/ac/io.tempo.jira/tempo-app#!/configuration/api-integration";

pub fn setup() {
    let storage = Storage::new();

    if storage.get_credentials().is_some() {
        println!("\nJira credentials already saved!");

        let overwrite_prompt: String = Input::new()
            .with_prompt("Do you want to overwrite credentials? (y/N)")
            .default("n".to_string())
            .interact_text()
            .unwrap();

        if overwrite_prompt.to_lowercase() == "y" {
            println!("Overwriting credentials...");
        } else {
            return;
        }
    }

    println!("\nStep 1/4:");
    println!("Enter your Jira profile URL to fetch your `account id` and Jira `domain name`:");
    println!("1. Navigate to the top-right corner and click your avatar");
    println!("2. Select \"👤 Profile\" from the dropdown menu");
    println!("3. Copy the URL from your browser's address bar and paste it below:\n");

    let profile_url: String = Input::new()
        .with_prompt("Enter Jira URL")
        .interact_text()
        .unwrap();

    let jira_url = profile_url.split("/jira/").next().unwrap().to_string();

    println!("\nStep 2/4:");
    println!("Enter your tempo token. You can generate it here:");
    println!("{}{}\n", jira_url, TEMPO_API_INTEGRATION_URL);

    if webbrowser::open(&format!("{jira_url}{TEMPO_API_INTEGRATION_URL}")).is_ok() {
        println!("The link should have opened automatically in your browser\n");
    }

    let tempo_token: String = Password::new()
        .with_prompt("Enter your Tempo API token")
        .interact()
        .unwrap();

    println!("\nStep 3/4:");
    println!("Enter your Jira API token:");
    println!("You can generate it here: {}\n", JIRA_API_TOKENS_URL);

    if webbrowser::open(JIRA_API_TOKENS_URL).is_ok() {
        println!("The link should have opened automatically in your browser\n");
    }

    let jira_token: String = Password::new()
        .with_prompt("Enter your Jira API token")
        .interact()
        .unwrap();

    println!("\nStep 4/4:");
    println!("This is the last step! Enter your Jira email:");

    let jira_email: String = Input::new()
        .with_prompt("Enter your Jira email")
        .interact_text()
        .unwrap();

    storage.store_credentials(&UserCredentials {
        url: jira_url.clone(),
        account_id: extract_account_id(&profile_url)
            .unwrap_or_else(|| {
                eprintln!("\nError: Could not extract account ID from the provided URL.");
                eprintln!("URL: {}", profile_url);
                eprintln!("Please make sure you've copied the correct profile URL.");
                eprintln!("The URL should be from your Jira profile page.");
                std::process::exit(1);
            })
            .to_string(),
        tempo_token,
        jira_token,
        jira_email,
    });

    println!("\nJira credentials saved successfully!");
}

fn extract_account_id(url: &str) -> Option<&str> {
    url.rsplit_once("/people/")?.1.split('/').next()
}
