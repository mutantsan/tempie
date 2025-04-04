use crate::api::jira::get_jira_issue;
use crate::models::*;
use crate::storage::Storage;
use crate::utils::parse_duration_from_string;
use chrono::Local;
use reqwest::{Client, StatusCode};

const TEMPO_BASE_URL: &str = "https://api.tempo.io/4";

pub async fn log_time(
    issue_key: &str,
    time_spent: &str,
    comment: Option<String>,
) -> Result<WorklogItem, String> {
    let storage = Storage::new();
    let config = storage.get_credentials().unwrap();

    let issue = get_jira_issue(&issue_key).await;

    let url = format!("{}/worklogs/", TEMPO_BASE_URL,);
    let client = Client::new();

    let response = client
        .post(&url)
        .bearer_auth(&config.tempo_token)
        .json(&serde_json::json!({
            "authorAccountId": config.account_id,
            "issueId": issue.map_err(|e| format!("Failed to get Jira issue: {}", e))?.id,
            "description": comment.unwrap_or_default(),
            "startDate": Local::now().format("%Y-%m-%d").to_string(),
            "timeSpentSeconds": parse_duration_from_string(time_spent)
        }))
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        return Err(format!(
            "Failed to fetch worklogs: {}, {}",
            status, error_body
        ));
    }

    let json_data: WorklogItem = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(json_data)
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

// Delete a worklog by its ID
pub async fn delete_worklog(worklog_id: &str) -> Result<(), String> {
    let storage = Storage::new();
    let config = storage.get_credentials().unwrap();

    let response = Client::new()
        .delete(&format!("{}/worklogs/{}", TEMPO_BASE_URL, worklog_id))
        .bearer_auth(&config.tempo_token)
        .json(&serde_json::json!({
            "id": worklog_id
        }))
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;

    if response.status() != StatusCode::NO_CONTENT {
        return Err(format!("Failed to delete worklog: {}", response.status()));
    }

    Ok(())
}
