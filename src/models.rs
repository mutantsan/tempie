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
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "startDate", default)]
    pub start_date: String,
    pub issue: TempoIssue,
    #[serde(skip)]
    pub jira_issue: Option<JiraIssue>,
}

impl WorklogItem {
    // The day a worklog belongs to. Prefer Tempo's startDate (the date the work
    // is logged against), fall back to the createdAt date. These diverge when a
    // worklog is created close to midnight: createdAt is UTC, so logging at
    // 00:13 local time can land on the previous calendar day.
    pub fn work_date(&self) -> String {
        if !self.start_date.is_empty() {
            self.start_date.clone()
        } else {
            self.created_at
                .split('T')
                .next()
                .unwrap_or_default()
                .to_string()
        }
    }
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
