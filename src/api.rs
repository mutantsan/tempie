use crate::models::*;
use crate::storage::Storage;
use crate::utils::parse_duration_from_string;
use chrono::Local;
use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use std::collections::HashSet;

const TEMPO_BASE_URL: &str = "https://api.tempo.io/4";
const CONCURRENT_REQUESTS: usize = 5;

#[async_trait::async_trait]
pub trait ApiTrait {
    async fn log_time(
        &self,
        issue_key: &str,
        time_spent: &str,
        comment: Option<String>,
    ) -> Result<WorklogItem, String>;
    async fn list_worklogs(&self, from: &str, to: &str) -> Result<Vec<WorklogItem>, String>;
    async fn delete_worklogs(&self, ids: &Vec<String>) -> Result<(), String>;
    async fn get_jira_issue(&self, issue_or_key: &str) -> Result<JiraIssue, String>;
}

pub struct ApiClient {
    client: Client,
    pub storage: Storage,
    pub config: UserCredentials,
}

impl ApiClient {
    pub fn new(storage: Storage) -> Self {
        let config = storage.get_credentials().unwrap();

        Self {
            client: Client::new(),
            storage,
            config,
        }
    }

    // Prefetch Jira issues concurrently
    async fn prefetch_jira_issues_concurrently(
        &self,
        worklogs: &Vec<WorklogItem>,
    ) -> Vec<JiraIssue> {
        let mut issues = HashSet::new();

        for worklog in worklogs {
            issues.insert(worklog.issue.id.to_string());
        }

        stream::iter(issues.iter().cloned())
            .map(|issue_id| async move {
                match self.get_jira_issue(&issue_id).await {
                    Ok(issue) => Some(issue),
                    Err(e) => {
                        eprintln!("Failed to fetch issue {}: {}", issue_id, e);
                        None
                    }
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS)
            .filter_map(async move |res| res) // Only keep successful results
            .collect()
            .await
    }
}

#[async_trait::async_trait]
impl ApiTrait for ApiClient {
    async fn log_time(
        &self,
        issue_key: &str,
        time_spent: &str,
        comment: Option<String>,
    ) -> Result<WorklogItem, String> {
        let issue = self.get_jira_issue(&issue_key).await;

        let response = self
            .client
            .post(format!("{}/worklogs/", TEMPO_BASE_URL,))
            .bearer_auth(&self.config.tempo_token)
            .json(&serde_json::json!({
                "authorAccountId": self.config.account_id,
                "issueId": issue.map_err(|e| format!("{}", e))?.id,
                "description": comment.unwrap_or_default(),
                "startDate": Local::now().format("%Y-%m-%d").to_string(),
                "timeSpentSeconds": parse_duration_from_string(time_spent)
            }))
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        let status = response.status();
        if !status.is_success() {
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

    async fn list_worklogs(
        &self,
        from_date: &str,
        to_date: &str,
    ) -> Result<Vec<WorklogItem>, String> {
        let mut worklogs: Vec<WorklogItem> = Vec::new();

        let mut offset = 0;
        let limit = 50;

        loop {
            let response = self
                .client
                .get(format!(
                    "{}/worklogs/user/{}",
                    TEMPO_BASE_URL, self.config.account_id
                ))
                .bearer_auth(&self.config.tempo_token)
                .query(&[
                    ("from", from_date),
                    ("to", to_date),
                    ("offset", offset.to_string().as_str()),
                ])
                .send()
                .await
                .map_err(|e| format!("Request error: {}", e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(format!("Failed to fetch worklogs: {}", status));
            }

            let mut json_data: UserWorklogsResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            if json_data.results.is_empty() {
                break;
            }

            let _ = self
                .prefetch_jira_issues_concurrently(&json_data.results)
                .await;

            for worklog in json_data.results.iter_mut() {
                worklog.jira_issue =
                    Some(self.get_jira_issue(&worklog.issue.id.to_string()).await?);
            }

            worklogs.extend(json_data.results);

            offset += limit;
        }

        Ok(worklogs)
    }

    // Delete a worklog by its ID
    async fn delete_worklogs(&self, worklog_ids: &Vec<String>) -> Result<(), String> {
        for worklog_id in worklog_ids {
            let response = self
                .client
                .delete(&format!("{}/worklogs/{}", TEMPO_BASE_URL, worklog_id))
                .bearer_auth(&self.config.tempo_token)
                .json(&serde_json::json!({
                    "id": worklog_id
                }))
                .send()
                .await
                .map_err(|e| format!("Request error: {}", e))?;

            if response.status() != StatusCode::NO_CONTENT {
                return Err(format!(
                    "Failed to delete worklog {}: {}",
                    worklog_id,
                    response.status()
                ));
            }
        }

        Ok(())
    }

    // Get Jira issue from Jira API and store it in the database by its id and key
    async fn get_jira_issue(&self, issue_or_key: &str) -> Result<JiraIssue, String> {
        if let Some(jira_issue) = self.storage.get_jira_issue(issue_or_key) {
            return Ok(jira_issue);
        }

        let url = format!("{}/rest/api/3/issue/{}", self.config.url, issue_or_key);

        let client = self
            .client
            .get(&url)
            .basic_auth(&self.config.jira_email, Some(&self.config.jira_token));

        let response = client.send().await.expect("JIRA request failed");
        let status = response.status();

        if !status.is_success() {
            let error_message = response
                .text()
                .await
                .ok()
                .and_then(|body| {
                    serde_json::from_str::<serde_json::Value>(&body)
                        .ok()
                        .and_then(|json| {
                            json["errorMessages"]
                                .as_array()
                                .and_then(|msgs| msgs.first())
                                .and_then(|msg| msg.as_str())
                                .map(String::from)
                        })
                })
                .unwrap_or_else(|| "Failed to read error response".to_string());

            return Err(format!("{} {}", error_message, status));
        }

        let raw_response = response.text().await.expect("Failed to get response text");

        let json_data: JiraIssue = serde_json::from_str(&raw_response)
            .map_err(|e| format!("Unable to retrieve Jira issue: {}", e))?;

        let issue = JiraIssue {
            key: json_data.key,
            id: json_data.id,
        };

        self.storage.store_jira_issue(&issue);

        Ok(issue)
    }
}
