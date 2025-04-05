use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserCredentials {
    pub url: String,
    pub account_id: String,
    pub tempo_token: String,
    pub jira_token: String,
    pub jira_email: String,
}

#[derive(Debug, Deserialize)]
pub struct UserWorklogsResponse {
    pub results: Vec<WorklogItem>,
}

#[derive(Debug, Deserialize)]
pub struct WorklogItem {
    #[serde(rename = "tempoWorklogId")]
    pub tempo_worklog_id: i64,
    #[serde(rename = "timeSpentSeconds")]
    pub time_spent_seconds: i32,
    pub description: String,
    pub issue: TempoIssue,
    #[serde(skip)]
    pub jira_issue: Option<JiraIssue>,
}

#[derive(Debug, Deserialize)]
pub struct TempoIssue {
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
}
