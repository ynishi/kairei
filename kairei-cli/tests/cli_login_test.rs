use assert_cmd::Command;
use predicates::prelude::*;

fn kairei_cmd() -> Command {
    Command::cargo_bin("kairei").unwrap()
}

#[test]
fn test_login_no_args_shows_current_settings() {
    // Running login without arguments should show current settings
    let assert = kairei_cmd().arg("login").assert();

    // Should run successfully
    assert
        .success()
        // Should contain the API URL and API Key headers
        .stdout(predicate::str::contains("Current API settings:"))
        .stdout(predicate::str::contains("API URL:"))
        .stdout(predicate::str::contains("API Key:"));
}

#[test]
fn test_login_key_masking() {
    // Run login command with a known API key
    let assert = kairei_cmd()
        .arg("login")
        .arg("--api-key")
        .arg("abcdefghijklmnopqrstuvwxyz")
        .assert();

    // Should run successfully and mask most of the key
    assert
        .success()
        // The API key should be masked in the output showing only the last 4 chars
        .stdout(predicate::str::contains("**********************wxyz"))
        // But should not show the full key
        .stdout(predicate::str::contains("abcdefghijklmnopqrstuvwxyz").not());
}
