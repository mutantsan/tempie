use std::fs;
use tempie::models::UserCredentials;
use tempie::storage::Storage;

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
    let test_db_path = "test_storage_credentials.db";
    let storage = Storage::with_path(test_db_path);
    let test_creds = create_test_credentials();

    // Test that initially there are no credentials
    assert!(storage.get_credentials().is_none());

    let test_creds = storage.store_credentials(test_creds);

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
    let storage = Storage::with_path(test_db_path);
    let test_issue = tempie::models::JiraIssue {
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

    cleanup_test_db(test_db_path);
}

#[test]
fn test_storage_overwrite() {
    let test_db_path = "test_storage_overwrite";
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

    cleanup_test_db(test_db_path);
}
