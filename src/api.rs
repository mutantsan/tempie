use crate::models::*;
use crate::storage::Storage;
use crate::utils::parse_duration_from_string;
use chrono::Local;
use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use std::collections::{HashMap, HashSet};

const TEMPO_BASE_URL: &str = "https://api.tempo.io/4";
const CONCURRENT_REQUESTS: usize = 5;
// Big enough to fetch a typical month in a single request
const WORKLOGS_PAGE_SIZE: usize = 1000;
const JIRA_SEARCH_BATCH_SIZE: usize = 100;

#[async_trait::async_trait]
pub trait ApiTrait {
    async fn log_time(
        &self,
        issue_key: &str,
        time_spent: &str,
        comment: Option<String>,
    ) -> Result<WorklogItem, String>;
    async fn list_worklogs(&self, from: &str, to: &str) -> Result<Vec<WorklogItem>, String>;
    async fn attach_jira_issues(&self, worklogs: &mut [&mut WorklogItem]) -> Result<(), String>;
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

    // Resolve many Jira issues in one round-trip per batch via JQL. Best-effort:
    // returns whatever it managed to find, the caller falls back to single lookups.
    async fn search_jira_issues_by_id(&self, issue_ids: &[String]) -> Vec<JiraIssue> {
        let mut found = Vec::new();

        for chunk in issue_ids.chunks(JIRA_SEARCH_BATCH_SIZE) {
            let jql = format!("id in ({})", chunk.join(","));

            let response = self
                .client
                .get(format!("{}/rest/api/3/search/jql", self.config.url))
                .basic_auth(&self.config.jira_email, Some(&self.config.jira_token))
                .query(&[
                    ("jql", jql.as_str()),
                    ("fields", "key"),
                    ("maxResults", JIRA_SEARCH_BATCH_SIZE.to_string().as_str()),
                ])
                .send()
                .await;

            if let Ok(response) = response {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<JiraSearchResponse>().await {
                        found.extend(data.issues);
                    }
                }
            }
        }

        found
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
                    ("limit", WORKLOGS_PAGE_SIZE.to_string().as_str()),
                ])
                .send()
                .await
                .map_err(|e| format!("Request error: {}", e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(format!("Failed to fetch worklogs: {}", status));
            }

            let json_data: UserWorklogsResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let page_len = json_data.results.len();
            // Tempo omits `metadata.next` on the last page. Without metadata,
            // fall back to paging until an empty page comes back.
            let has_more = match &json_data.metadata {
                Some(metadata) => metadata.next.is_some(),
                None => page_len > 0,
            };

            worklogs.extend(json_data.results);

            if !has_more || page_len == 0 {
                break;
            }

            // Advance by what was actually returned, in case Tempo caps the limit
            offset += page_len;
        }

        Ok(worklogs)
    }

    // Fill in `jira_issue` for the given worklogs: cached issues first, then one
    // batched Jira search for the rest, then single lookups for anything left.
    async fn attach_jira_issues(&self, worklogs: &mut [&mut WorklogItem]) -> Result<(), String> {
        let mut issues: HashMap<String, JiraIssue> = HashMap::new();
        let mut missing: HashSet<String> = HashSet::new();

        for worklog in worklogs.iter() {
            let issue_id = worklog.issue.id.to_string();

            if issues.contains_key(&issue_id) || missing.contains(&issue_id) {
                continue;
            }

            match self.storage.get_jira_issue(&issue_id) {
                Some(issue) => {
                    issues.insert(issue_id, issue);
                }
                None => {
                    missing.insert(issue_id);
                }
            }
        }

        if !missing.is_empty() {
            let missing: Vec<String> = missing.into_iter().collect();

            let found = self.search_jira_issues_by_id(&missing).await;
            self.storage.store_jira_issues(&found);
            for issue in found {
                issues.insert(issue.id.clone(), issue);
            }

            // Whatever the search didn't return (JQL error, deleted issue, no
            // access) goes through single lookups, which report a proper error.
            let leftover: Vec<String> = missing
                .into_iter()
                .filter(|issue_id| !issues.contains_key(issue_id))
                .collect();

            let fetched: Vec<Result<JiraIssue, String>> = stream::iter(leftover)
                .map(|issue_id| async move {
                    self.get_jira_issue(&issue_id)
                        .await
                        .map_err(|e| format!("Failed to fetch issue {}: {}", issue_id, e))
                })
                .buffer_unordered(CONCURRENT_REQUESTS)
                .collect()
                .await;

            for result in fetched {
                let issue = result?;
                issues.insert(issue.id.clone(), issue);
            }
        }

        for worklog in worklogs.iter_mut() {
            worklog.jira_issue = issues.get(&worklog.issue.id.to_string()).cloned();
        }

        Ok(())
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
