use crate::models::UserCredentials;
use crate::storage::Storage;
use dialoguer::{Input, Password};

const JIRA_API_TOKENS_URL: &str = "https://id.atlassian.com/manage-profile/security/api-tokens";
const TEMPO_API_INTEGRATION_URL: &str =
    "/plugins/servlet/ac/io.tempo.jira/tempo-app#!/configuration/api-integration";

pub fn setup(storage: &Storage) {
    if !should_overwrite_credentials(storage) {
        return;
    }

    let (jira_url, account_id) = get_jira_credentials();
    let tempo_token = get_tempo_token(&jira_url);
    let jira_token = get_jira_token();
    let jira_email = get_jira_email();

    storage.store_credentials(UserCredentials {
        url: jira_url.clone(),
        account_id,
        tempo_token,
        jira_token,
        jira_email,
    });

    println!("\nUser credentials saved successfully!");
    println!("{}", format_credentials_for_display(storage));
}

fn should_overwrite_credentials(storage: &Storage) -> bool {
    if storage.get_credentials().is_none() {
        return true;
    }

    println!("\nUser credentials already saved!");
    println!("{}", format_credentials_for_display(storage));

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

fn format_credentials_for_display(storage: &Storage) -> String {
    let credentials = storage.get_credentials().unwrap();

    let output = format!(
        "\nCurrent credentials:

👤 User Email: {}
🔗 Jira URL: {}
🔑 Jira Token: {}
🔑 Tempo Token: {}\n",
        credentials.jira_email,
        credentials.url,
        mask_token(&credentials.jira_token),
        mask_token(&credentials.tempo_token)
    );

    output
}

/// Masks the middle of a token with `***`, keeping a few characters at the start and end.
fn mask_token(token: &str) -> String {
    let len = token.len();

    match len {
        0..=6 => "*".repeat(len), // Fully mask very short tokens
        _ => {
            let start_len = len / 4;
            let end_len = len / 4;
            let start = &token[..start_len];
            let end = &token[len - end_len..];
            format!("{start}***{end}")
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn cleanup_test_db(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_mask_token() {
        assert_eq!(mask_token("1234567890"), "12***90");
        assert_eq!(mask_token("12345678901234567890"), "12345***67890");
    }

    #[test]
    fn test_format_credentials_for_display() {
        let test_db_path = "test_format_credentials_for_display";
        cleanup_test_db(test_db_path);

        let storage = Storage::with_path(test_db_path);
        storage.store_credentials(UserCredentials {
            url: "https://example.com".to_string(),
            account_id: "1234567890".to_string(),
            tempo_token: "tempo_token_value".to_string(),
            jira_token: "jira_token_value".to_string(),
            jira_email: "example@example.com".to_string(),
        });

        let output = format_credentials_for_display(&storage);

        assert!(output.contains("👤 User Email: example@example.com"));
        assert!(output.contains("🔗 Jira URL: https://example.com"));
        assert!(output.contains("🔑 Jira Token: jira***alue"));
        assert!(output.contains("🔑 Tempo Token: temp***alue"));

        cleanup_test_db(test_db_path);
    }
}
