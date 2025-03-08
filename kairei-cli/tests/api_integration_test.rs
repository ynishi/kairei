// API integration tests for the Kairei CLI
// Tests CLI commands that interact with the API endpoints

use assert_cmd::Command;
use predicates::prelude::*;

fn kairei_cmd() -> Command {
    Command::cargo_bin("kairei").unwrap()
}

#[tokio::test]
async fn test_system_list_command() {
    // Mock API response
    let mock_response = r#"
    {
        "system_statuses": {
            "system-1": {
                "started_at": "2023-01-01T00:00:00Z",
                "running": true,
                "uptime": {"secs": 3600, "nanos": 0},
                "agent_count": 5,
                "running_agent_count": 3,
                "event_queue_size": 10,
                "event_subscribers": 2,
                "event_capacity": 100
            }
        }
    }
    "#;

    let mut server = mockito::Server::new_async().await;

    // Setup mock server
    let _m = server
        .mock("GET", "/api/v1/systems")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    // Run system list command
    let assert = kairei_cmd()
        .arg("-u")
        .arg(server.url())
        .arg("-k")
        .arg("test-key")
        .arg("system")
        .arg("list")
        .assert();

    // Should run successfully
    assert
        .success()
        .stdout(predicate::str::contains("system-1"))
        .stdout(predicate::str::contains("agent_count"));
}

#[tokio::test]
async fn test_system_get_command() {
    // Mock API response
    let mock_response = r#"
        {
            "started_at": "2023-01-01T00:00:00Z",
            "running": true,
            "uptime": {"secs": 3600, "nanos": 0},
            "agent_count": 5,
            "running_agent_count": 3,
            "event_queue_size": 10,
            "event_subscribers": 2,
            "event_capacity": 100
        }
    "#;

    let mut server = mockito::Server::new_async().await;

    // Setup mock server
    let _mock = server
        .mock("GET", "/api/v1/systems/test-system")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    // Run system get command
    let assert = kairei_cmd()
        .arg("-u")
        .arg(server.url())
        .arg("-k")
        .arg("test-key")
        .arg("system")
        .arg("get")
        .arg("test-system")
        .assert();

    // Should run successfully
    assert
        .success()
        .stdout(predicate::str::contains("started_at"))
        .stdout(predicate::str::contains("running"))
        .stdout(predicate::str::contains("uptime"));
}

#[tokio::test]
async fn test_error_handling() {
    // Mock API error response
    let mock_response = r#"
    {
        "error": "System not found",
        "code": "NOT_FOUND"
    }
    "#;

    // Setup mock server
    let mut server = mockito::Server::new_async().await;

    let _m = server
        .mock("GET", "/api/v1/systems/non-existent")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    // Run system get command with non-existent ID
    let assert = kairei_cmd()
        .arg("-u")
        .arg(server.url())
        .arg("-k")
        .arg("test-key")
        .arg("system")
        .arg("get")
        .arg("non-existent")
        .assert();

    // Should fail with an error message
    assert
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("API error:"))
        .stderr(predicate::str::contains("System not found"));
}
