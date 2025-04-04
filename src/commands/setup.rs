use crate::storage::{Storage, UserCredentials};
use dialoguer::{Input, Password};

const JIRA_API_TOKENS_URL: &str = "https://id.atlassian.com/manage-profile/security/api-tokens";
const TEMPO_API_INTEGRATION_URL: &str =
    "/plugins/servlet/ac/io.tempo.jira/tempo-app#!/configuration/api-integration";

pub fn setup() {
    let storage = Storage::new();

    if !should_overwrite_credentials(&storage) {
        return;
    }

    let (jira_url, account_id) = get_jira_credentials();
    let tempo_token = get_tempo_token(&jira_url);
    let jira_token = get_jira_token();
    let jira_email = get_jira_email();

    storage.store_credentials(&UserCredentials {
        url: jira_url.clone(),
        account_id,
        tempo_token,
        jira_token,
        jira_email,
    });

    println!("\nJira credentials saved successfully!");
}

fn should_overwrite_credentials(storage: &Storage) -> bool {
    if storage.get_credentials().is_none() {
        return true;
    }

    println!("\nJira credentials already saved!");

    let overwrite_prompt: String = Input::new()
        .with_prompt("Do you want to overwrite credentials? (y/N)")
        .default("n".to_string())
        .interact_text()
        .unwrap();

    if overwrite_prompt.to_lowercase() == "y" {
        println!("Overwriting credentials...");
        true
    } else {
        false
    }
}

fn get_jira_credentials() -> (String, String) {
    println!("\nStep 1/4:");
    println!("Enter your Jira profile URL to fetch your `account id` and Jira `domain name`:");
    println!("1. Navigate to the top-right corner and click your avatar");
    println!("2. Select \"👤 Profile\" from the dropdown menu");
    println!("3. Copy the URL from your browser's address bar and paste it below:\n");

    let profile_url: String = Input::new()
        .with_prompt("Enter Jira URL")
        .interact_text()
        .unwrap();

    let parts: Vec<&str> = profile_url.split("/jira/people/").collect();

    match parts.as_slice() {
        [url, id] => {
            let jira_url = url.to_string();
            let account_id = id.to_string();

            (jira_url, account_id)
        }
        _ => {
            eprintln!(
                "\nInvalid Jira URL. Please make sure you've copied the correct profile URL."
            );
            eprintln!("Example: https://xxx.jira.com/jira/people/1b2c3d4e5f6g7h8i9j0k");
            std::process::exit(1);
        }
    }
}

fn get_tempo_token(jira_url: &str) -> String {
    println!("\nStep 2/4:");
    println!("Enter your tempo token. You can generate it here:");
    println!("{}{}\n", jira_url, TEMPO_API_INTEGRATION_URL);

    if webbrowser::open(&format!("{jira_url}{TEMPO_API_INTEGRATION_URL}")).is_ok() {
        println!("The link should have opened automatically in your browser\n");
    }

    Password::new()
        .with_prompt("Enter your Tempo API token")
        .interact()
        .unwrap()
}

fn get_jira_token() -> String {
    println!("\nStep 3/4:");
    println!("Enter your Jira API token:");
    println!("You can generate it here: {}\n", JIRA_API_TOKENS_URL);

    if webbrowser::open(JIRA_API_TOKENS_URL).is_ok() {
        println!("The link should have opened automatically in your browser\n");
    }

    Password::new()
        .with_prompt("Enter your Jira API token")
        .interact()
        .unwrap()
}

fn get_jira_email() -> String {
    println!("\nStep 4/4:");
    println!("This is the last step! Enter your Jira email:");

    Input::new()
        .with_prompt("Enter your Jira email")
        .interact_text()
        .unwrap()
}
