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

    let url = format!("{}/rest/api/3/issue/{}", config.url, issue_or_key);

    let client = Client::new()
        .get(&url)
        .basic_auth(config.jira_email, Some(config.jira_token));

    let response = client.send().await.expect("JIRA request failed");
    let raw_response = response.text().await.expect("Failed to get response text");

    let json_data: JiraIssue = serde_json::from_str(&raw_response)
        .map_err(|e| format!("Unable to parse Jira issue from response: {}", e))?;

    let issue = JiraIssue {
        key: json_data.key,
        id: json_data.id,
    };

    storage.store_jira_issue(&issue.key.as_str(), &issue);
    storage.store_jira_issue(&issue.id.as_str(), &issue);

    Ok(issue)
}
