use serde::{Deserialize, Serialize};
use serde_json;
use sled;
use std::sync::OnceLock;

use crate::models::JiraIssue;

static DB_INSTANCE: OnceLock<sled::Db> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct UserCredentials {
    pub url: String,
    pub account_id: String,
    pub tempo_token: String,
    pub jira_token: String,
    pub jira_email: String,
}

pub struct Storage {
    db: sled::Db,
}

impl Storage {
    pub fn new() -> Self {
        let db = DB_INSTANCE
            .get_or_init(|| sled::open("tempie.db").expect("Failed to open sled database"))
            .clone();

        Self { db }
    }

    // Store Jira credentials
    pub fn store_credentials(&self, creds: &UserCredentials) {
        let serialized = serde_json::to_string(creds).unwrap();
        self.db
            .insert("jira_credentials", serialized.as_bytes())
            .unwrap();
        self.db.flush().unwrap();
    }

    // Get Jira credentials
    pub fn get_credentials(&self) -> Option<UserCredentials> {
        self.db
            .get("jira_credentials")
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_slice(&v).ok())
    }

    // Store jira issue info
    pub fn store_jira_issue(&self, key: &str, value: &JiraIssue) {
        self.db
            .insert(key, serde_json::to_string(value).unwrap().as_bytes())
            .unwrap();
        self.db.flush().unwrap();
    }

    // Get jira issue info
    pub fn get_jira_issue(&self, key_or_id: &str) -> Option<JiraIssue> {
        self.db
            .get(key_or_id)
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_slice(&v).ok())
    }
}
