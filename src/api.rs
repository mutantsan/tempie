use crate::storage::Storage;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct WorklogItem {
    pub description: Option<String>,
    pub issue: HashMap<String, String>,
    pub tempo_worklog_id: i64,
    pub time_spent_seconds: i32,
}

#[derive(Debug, Deserialize)]
pub struct UserWorklogsResponse {
    // pub metadata: HashMap<String, String>,
    pub results: Vec<WorklogItem>,
    // pub self_url: String,
}

pub async fn log_time(issue_key: &str, time_spent: &str, comment: Option<String>) -> Result<(), String> {
    let storage = Storage::new();
    let config = storage.get_credentials().unwrap();

    let url = format!("{}/rest/api/3/issue/{}/worklog", config.url, issue_key);
    let client = Client::new();

    let response = client
        .post(&url)
        .json(&serde_json::json!({ "timeSpent": time_spent, "comment": comment.unwrap_or_default() }))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status() == StatusCode::OK => {
            println!(
                "✅ Time logged successfully on {}: {}",
                issue_key, time_spent
            );
            Ok(())
        }
        Ok(resp) => Err(format!(
            "❌ Failed to log time: {} - {}",
            url,
            resp.status()
        )),
        Err(e) => Err(format!("❌ Request error: {}", e)),
    }
}

pub async fn list_worklogs(
    from_date: &str,
    to_date: &str,
) -> Result<Vec<HashMap<String, String>>, String> {
    let storage = Storage::new();
    let config = storage.get_credentials().unwrap();

    let url = format!("https://api.tempo.io/4/worklogs/user/{}", config.account_id);
    let client = Client::new();

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.api_token))
        .query(&[("from", from_date), ("to", to_date)])
        .send()
        .await;

    match response {
        Ok(resp) if resp.status() == StatusCode::OK => {
            let data: UserWorklogsResponse = resp.json().await.map_err(|e| e.to_string())?;
            let mut result = Vec::new();

            for log in data.results {
                let jira_issue = get_jira_issue(&log.issue["self"]).await?;

                let mut worklog = HashMap::new();
                worklog.insert("id".to_string(), log.tempo_worklog_id.to_string());
                worklog.insert("duration".to_string(), log.time_spent_seconds.to_string());
                worklog.insert(
                    "description".to_string(),
                    log.description.unwrap_or_default(),
                );
                worklog.insert(
                    "task_url".to_string(),
                    format!("{}/browse/{}", config.url, jira_issue["key"]),
                );

                result.push(worklog);
            }
            Ok(result)
        }
        Ok(resp) => Err(format!(
            "❌ Failed to fetch worklogs: {} - {}",
            url,
            resp.status()
        )),
        Err(e) => Err(format!("❌ Request error: {}", e)),
    }
}

pub async fn get_jira_issue(url: &str) -> Result<HashMap<String, String>, String> {
    let storage = Storage::new();
    let config = storage.get_credentials().unwrap();

    let client = Client::new();
    let response = client
        .get(url)
        .basic_auth(&config.account_id, Some(&config.api_token))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status() == StatusCode::OK => {
            let data: HashMap<String, String> = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data)
        }
        Ok(_) => Err(format!("❌ Failed to fetch issue: {}", url)),
        Err(e) => Err(format!("❌ Request error: {}", e)),
    }
}
