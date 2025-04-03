use serde::{Deserialize, Serialize};
use serde_json;
use sled;

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraCredentials {
    pub url: String,
    pub account_id: String,
    pub api_token: String,
}

pub struct Storage {
    db: sled::Db,
}

impl Storage {
    pub fn new() -> Self {
        let db = sled::open("tempie.db").expect("Failed to open sled database");
        Storage { db }
    }

    // Store Jira credentials
    pub fn store_credentials(&self, creds: &JiraCredentials) {
        let serialized = serde_json::to_string(creds).unwrap();
        self.db
            .insert("jira_credentials", serialized.as_bytes())
            .unwrap();
        self.db.flush().unwrap();
    }

    pub fn get_credentials(&self) -> Option<JiraCredentials> {
        self.db
            .get("jira_credentials")
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_slice(&v).ok())
    }

    // // Store arbitrary key-value data (for future use)
    // pub fn store_data(&self, key: &str, value: &str) {
    //     self.db.insert(key, value.as_bytes()).unwrap();
    //     self.db.flush().unwrap();
    // }

    // pub fn get_data(&self, key: &str) -> Option<String> {
    //     self.db
    //         .get(key)
    //         .ok()
    //         .flatten()
    //         .and_then(|v| String::from_utf8(v.to_vec()).ok())
    // }
}
