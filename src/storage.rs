use serde_json;
use sled;
use std::sync::OnceLock;

use crate::models::{JiraIssue, UserCredentials};

static DB_INSTANCE: OnceLock<sled::Result<sled::Db>> = OnceLock::new();
pub const DEFAULT_DATABASE_PATH: &str = "tempie.db";

pub struct Storage {
    db: sled::Db,
}

impl Storage {
    pub fn new() -> Self {
        Self::with_path(DEFAULT_DATABASE_PATH)
    }

    pub fn with_path(path: &str) -> Self {
        // we need lock, because we often close/open connection to the database
        let db = DB_INSTANCE.get_or_init(|| sled::open(path));

        if let Ok(db) = db {
            Self { db: db.clone() }
        } else {
            panic!("The database is busy. You should wait for the previous operation to complete.");
        }
    }

    // Store Jira credentials
    pub fn store_credentials(&self, creds: UserCredentials) -> UserCredentials {
        let serialized = serde_json::to_string(&creds).unwrap();
        self.db
            .insert("jira_credentials", serialized.as_bytes())
            .unwrap();
        self.db.flush().unwrap();

        creds
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
    pub fn store_jira_issue(&self, issue: &JiraIssue) {
        self.db
            .insert(
                issue.id.as_str(),
                serde_json::to_string(issue).unwrap().as_bytes(),
            )
            .unwrap();

        self.db
            .insert(
                issue.key.as_str(),
                serde_json::to_string(issue).unwrap().as_bytes(),
            )
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
