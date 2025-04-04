use crate::api::jira::get_jira_issue;
use crate::models::*;
use crate::storage::Storage;
use reqwest::{Client, StatusCode};
const TEMPO_BASE_URL: &str = "https://api.tempo.io/4";

pub async fn log_time(
    issue_key: &str,
    time_spent: &str,
    comment: Option<String>,
) -> Result<(), String> {
    let storage = Storage::new();
    let config = storage.get_credentials().unwrap();

    let issue = get_jira_issue(&issue_key).await;

    let url = format!(
        "{}/rest/api/3/issue/{}/worklog",
        config.url,
        issue.unwrap().id
    );
    let client = Client::new();

    let response = client
        .post(&url)
        .json(
            &serde_json::json!({ "timeSpent": time_spent, "comment": comment.unwrap_or_default() }),
        )
        .send()
        .await;

    match response {
        Ok(resp) if resp.status() == StatusCode::CREATED => {
            println!("Time logged successfully on {}: {}", issue_key, time_spent);
            Ok(())
        }
        Ok(resp) => Err(format!("Failed to log time: {} - {}", url, resp.status())),
        Err(e) => Err(format!("Request error: {}", e)),
    }
}

pub async fn list_worklogs(from_date: &str, to_date: &str) -> Result<Vec<WorklogItem>, String> {
    let storage = Storage::new();
    let config = storage.get_credentials().unwrap();
    let client = Client::new();

    let response = client
        .get(&format!(
            "{}/worklogs/user/{}",
            TEMPO_BASE_URL, config.account_id
        ))
        .bearer_auth(&config.tempo_token)
        .query(&[("from", from_date), ("to", to_date)])
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch worklogs: {}", response.status()));
    }

    let mut json_data: UserWorklogsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    for worklog in json_data.results.iter_mut() {
        worklog.jira_issue = Some(get_jira_issue(&worklog.issue.id.to_string()).await?);
    }

    Ok(json_data.results)
}
