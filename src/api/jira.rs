use crate::models::*;
use crate::storage::Storage;
use reqwest::Client;

// Get Jira issue from Jira API and store it in the database by its id and key
pub async fn get_jira_issue(issue_or_key: &str) -> Result<JiraIssue, String> {
    let storage = Storage::new();
    let config = storage.get_credentials().unwrap();

    if let Some(jira_issue) = storage.get_jira_issue(issue_or_key) {
        return Ok(jira_issue);
    }

    let url = format!("{}/rest/api/3/project/{}", config.url, issue_or_key);

    println!("Getting issue from JIRA: {}", url);

    let client = Client::new()
        .get(&url)
        .basic_auth(config.jira_email, Some(config.jira_token));

    let response = client.send().await.expect("JIRA request failed");
    let raw_response = response.text().await.expect("Failed to get response text");

    let json_data: JiraIssue = serde_json::from_str(&raw_response)
        .map_err(|e| format!("Unable to parse Jira issue from response: {}", e))?;

    storage.store_jira_issue(json_data.key.as_str(), &json_data);
    storage.store_jira_issue(json_data.id.as_str(), &json_data);

    Ok(json_data)
}
