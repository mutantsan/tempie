[1mdiff --git a/.gitignore b/.gitignore[m
[1mindex 2f710ac..de358ff 100644[m
[1m--- a/.gitignore[m
[1m+++ b/.gitignore[m
[36m@@ -1,3 +1,2 @@[m
 /target[m
[31m-tempie.db[m
 .vscode/[m
[1mdiff --git a/Cargo.lock b/Cargo.lock[m
[1mindex ebe7ef9..4d417fe 100644[m
[1m--- a/Cargo.lock[m
[1m+++ b/Cargo.lock[m
[36m@@ -2166,6 +2166,7 @@[m [mdependencies = [[m
  "tabled",[m
  "tokio",[m
  "webbrowser",[m
[32m+[m[32m "xdg-home",[m
 ][m
 [m
 [[package]][m
[36m@@ -2967,6 +2968,16 @@[m [mversion = "0.5.5"[m
 source = "registry+https://github.com/rust-lang/crates.io-index"[m
 checksum = "1e9df38ee2d2c3c5948ea468a8406ff0db0b29ae1ffde1bcf20ef305bcc95c51"[m
 [m
[32m+[m[32m[[package]][m
[32m+[m[32mname = "xdg-home"[m
[32m+[m[32mversion = "1.3.0"[m
[32m+[m[32msource = "registry+https://github.com/rust-lang/crates.io-index"[m
[32m+[m[32mchecksum = "ec1cdab258fb55c0da61328dc52c8764709b249011b2cad0454c72f0bf10a1f6"[m
[32m+[m[32mdependencies = [[m
[32m+[m[32m "libc",[m
[32m+[m[32m "windows-sys 0.59.0",[m
[32m+[m[32m][m
[32m+[m
 [[package]][m
 name = "yaml-rust2"[m
 version = "0.10.1"[m
[1mdiff --git a/Cargo.toml b/Cargo.toml[m
[1mindex 590d6e8..cae4d29 100644[m
[1m--- a/Cargo.toml[m
[1m+++ b/Cargo.toml[m
[36m@@ -27,6 +27,7 @@[m [mspinners = "4.1.1"[m
 tabled = {version = "0.18.0", features = ["ansi"]}[m
 tokio = {version = "1", features = ["full"]}[m
 webbrowser = "1.0"[m
[32m+[m[32mxdg-home = "1.3.0"[m
 [m
 [[bin]][m
 name = "tempie"[m
[1mdiff --git a/src/commands/list.rs b/src/commands/list.rs[m
[1mindex 5160926..8112cb3 100644[m
[1m--- a/src/commands/list.rs[m
[1m+++ b/src/commands/list.rs[m
[36m@@ -9,8 +9,7 @@[m [muse tabled::{[m
     builder::Builder,[m
     settings::object::Rows,[m
     settings::style::BorderSpanCorrection,[m
[31m-    settings::{Alignment, Span},[m
[31m-    settings::{Color, Style},[m
[32m+[m[32m    settings::{Alignment, Span, Color, Style},[m
 };[m
 [m
 pub async fn list(api: &ApiClient, from_date: &str, to_date: &str) {[m
[1mdiff --git a/src/storage.rs b/src/storage.rs[m
[1mindex 6f0621f..46f6205 100644[m
[1m--- a/src/storage.rs[m
[1m+++ b/src/storage.rs[m
[36m@@ -1,7 +1,8 @@[m
[32m+[m[32muse crate::models::{JiraIssue, UserCredentials};[m
 use serde_json;[m
 use sled;[m
[31m-[m
[31m-use crate::models::{JiraIssue, UserCredentials};[m
[32m+[m[32muse std::path::PathBuf;[m
[32m+[m[32muse xdg_home::home_dir;[m
 [m
 pub struct Storage {[m
     db: sled::Db,[m
[36m@@ -9,7 +10,7 @@[m [mpub struct Storage {[m
 [m
 impl Storage {[m
     pub fn new() -> Self {[m
[31m-        Self::with_path("tempie.db")[m
[32m+[m[32m        Self::with_path(Self::get_db_path("tempie.db").to_str().unwrap())[m
     }[m
 [m
     pub fn with_path(path: &str) -> Self {[m
[36m@@ -23,6 +24,18 @@[m [mimpl Storage {[m
         Self { db }[m
     }[m
 [m
[32m+[m[32m    pub fn get_db_path(db_name: &str) -> PathBuf {[m
[32m+[m[32m        let home = home_dir().unwrap();[m
[32m+[m
[32m+[m[32m        let tempie_dir = home.join(".tempie");[m
[32m+[m
[32m+[m[32m        if !tempie_dir.exists() {[m
[32m+[m[32m            std::fs::create_dir_all(&tempie_dir).expect("Could not create .tempie directory");[m
[32m+[m[32m        }[m
[32m+[m
[32m+[m[32m        tempie_dir.join(db_name)[m
[32m+[m[32m    }[m
[32m+[m
     // Store Jira credentials[m
     pub fn store_credentials(&self, creds: UserCredentials) -> UserCredentials {[m
         let serialized = serde_json::to_string(&creds).unwrap();[m
