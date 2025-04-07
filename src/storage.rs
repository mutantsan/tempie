use crate::models::{JiraIssue, UserCredentials};
use serde_json;
use sled;
use std::path::PathBuf;
use xdg_home::home_dir;

pub struct Storage {
    db: sled::Db,
}

impl Storage {
    pub fn new() -> Self {
        Self::with_path(Self::get_db_path("tempie.db").to_str().unwrap())
    }

    pub fn with_path(path: &str) -> Self {
        let db = sled::open(path).unwrap_or_else(|e| {
            if e.to_string().contains("lock file") {
                panic!("Database is already in use. Please wait other command to finish.");
            }
            panic!("Failed to open sled DB: {}", e);
        });

        Self { db }
    }

    pub fn get_db_path(db_name: &str) -> PathBuf {
        let home = home_dir().unwrap();

        let tempie_dir = home.join(".tempie");

        if !tempie_dir.exists() {
            std::fs::create_dir_all(&tempie_dir).expect("Could not create .tempie directory");
        }

        tempie_dir.join(db_name)
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    fn cleanup_test_db(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    fn create_test_credentials() -> UserCredentials {
        UserCredentials {
            url: "https://test.atlassian.net".to_string(),
            account_id: "test123".to_string(),
            tempo_token: "test-tempo-token".to_string(),
            jira_token: "test-jira-token".to_string(),
            jira_email: "test@example.com".to_string(),
        }
    }

    #[test]
    fn test_storage_credentials() {
        let test_db_path = "test_storage_credentials";
        cleanup_test_db(test_db_path);
        let storage = Storage::with_path(test_db_path);

        // Test that initially there are no credentials
        assert!(storage.get_credentials().is_none());

        let test_creds = storage.store_credentials(create_test_credentials());

        let retrieved_creds = storage
            .get_credentials()
            .expect("Failed to get credentials");

        assert_eq!(retrieved_creds.url, test_creds.url);
        assert_eq!(retrieved_creds.account_id, test_creds.account_id);
        assert_eq!(retrieved_creds.tempo_token, test_creds.tempo_token);
        assert_eq!(retrieved_creds.jira_token, test_creds.jira_token);
        assert_eq!(retrieved_creds.jira_email, test_creds.jira_email);

        cleanup_test_db(test_db_path);
    }

    #[test]
    fn test_storage_jira_issue() {
        let test_db_path = "test_storage_jira_issue";
        cleanup_test_db(test_db_path);
        let storage = Storage::with_path(test_db_path);
        let test_issue = JiraIssue {
            id: "12345".to_string(),
            key: "TEST-123".to_string(),
        };

        assert!(storage.get_jira_issue(&test_issue.key).is_none());
        assert!(storage.get_jira_issue(&test_issue.id).is_none());

        storage.store_jira_issue(&test_issue);
        storage.store_jira_issue(&test_issue);

        // Verify stored issue can be retrieved by key
        let retrieved_by_key = storage
            .get_jira_issue(&test_issue.key)
            .expect("Failed to get issue by key");

        assert_eq!(retrieved_by_key.id, test_issue.id);
        assert_eq!(retrieved_by_key.key, test_issue.key);

        // Verify stored issue can be retrieved by ID
        let retrieved_by_id = storage
            .get_jira_issue(&test_issue.id)
            .expect("Failed to get issue by id");

        assert_eq!(retrieved_by_id.id, test_issue.id);
        assert_eq!(retrieved_by_id.key, test_issue.key);

        let _ = fs::remove_dir_all(test_db_path);
    }

    #[test]
    fn test_storage_overwrite() {
        let test_db_path = "test_storage_overwrite";
        cleanup_test_db(test_db_path);
        let storage = Storage::with_path(test_db_path);

        storage.store_credentials(create_test_credentials());

        // Create and store new credentials
        let new_creds = UserCredentials {
            url: "https://new.atlassian.net".to_string(),
            account_id: "new456".to_string(),
            tempo_token: "new-tempo-token".to_string(),
            jira_token: "new-jira-token".to_string(),
            jira_email: "new@example.com".to_string(),
        };
        let new_creds = storage.store_credentials(new_creds);

        // Verify that new credentials overwrote the old ones
        let retrieved_creds = storage
            .get_credentials()
            .expect("Failed to get credentials");

        assert_eq!(retrieved_creds.url, new_creds.url);
        assert_eq!(retrieved_creds.account_id, new_creds.account_id);
        assert_eq!(retrieved_creds.tempo_token, new_creds.tempo_token);
        assert_eq!(retrieved_creds.jira_token, new_creds.jira_token);
        assert_eq!(retrieved_creds.jira_email, new_creds.jira_email);

        let _ = fs::remove_dir_all(test_db_path);
    }
}
