use crate::storage::{JiraCredentials, Storage};
use dialoguer::{Input, Password};

const TEMPO_API_INTEGRATION_URL: &str =
    "/plugins/servlet/ac/io.tempo.jira/tempo-app#!/configuration/api-integration";

pub fn setup() {
    let storage = Storage::new();

    if storage.get_credentials().is_some() {
        println!("Jira credentials already saved");
        return;
    }

    println!("\nStep 1/2:");
    println!("Enter your Jira profile URL to fetch your `account id` and `domain name`:");
    println!("1. Navigate to the top-right corner and click your avatar");
    println!("2. Select \"👤 Profile\" from the dropdown menu");
    println!("3. Copy the URL from your browser's address bar and paste it below:\n");

    let profile_url: String = Input::new()
        .with_prompt("Enter Jira URL")
        .interact_text()
        .unwrap();

    let jira_url = profile_url.split("/jira/").next().unwrap().to_string();
    let account_id = profile_url.split("/").nth(4).unwrap();

    println!("\nStep 2/2:");
    println!("That's almost everything! Enter your tempo token. You can generate it here:");
    println!("{}{}\n", jira_url, TEMPO_API_INTEGRATION_URL);

    if webbrowser::open(&format!("{jira_url}{TEMPO_API_INTEGRATION_URL}")).is_ok() {
        println!("The link should have opened automatically in your browser\n");
    }

    let api_token: String = Password::new()
        .with_prompt("Enter your Jira API token")
        .interact()
        .unwrap();

    storage.store_credentials(&JiraCredentials {
        url: jira_url.clone(),
        account_id: account_id.to_string(),
        api_token,
    });

    println!("✅ Jira credentials saved successfully!");
}
