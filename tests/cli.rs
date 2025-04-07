use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("tempie").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: tempie <COMMAND>"))
        .stdout(predicate::str::contains("setup              Configure Jira credentials"))
        .stdout(predicate::str::contains("list               List worklogs"))
        .stdout(predicate::str::contains("log                Log time"))
        .stdout(predicate::str::contains("delete             Delete worklog(s)"))
        .stdout(predicate::str::contains("clean-jira-issues  Clean jira issues from database"))
        .stdout(predicate::str::contains("help               Print this message or the help of the given subcommand(s)"));
}
