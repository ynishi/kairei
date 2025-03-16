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
                "uptime": 3600,
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
            "uptime": 3600,
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

#[tokio::test]
async fn test_doc_map_command() {
    // Mock API response
    let mock_response = serde_json::json!({
        "version": "1.0.0",
        "categories": ["Expression", "Statement", "Handler"],
        "parsers_by_category": {
            "Expression": ["parse_binary_expression", "parse_think"],
            "Statement": ["parse_if_statement"],
            "Handler": ["parse_observe_handler"]
        }
    })
    .to_string();

    // Setup mock server
    let mut server = mockito::Server::new_async().await;

    let _m = server
        .mock("GET", "/api/v1/docs/dsl/map")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&mock_response)
        .create_async()
        .await;

    // Run doc map command
    let assert = kairei_cmd()
        .arg("-u")
        .arg(server.url())
        .arg("-k")
        .arg("test-key")
        .arg("doc")
        .arg("map")
        .assert();

    // Should run successfully
    assert
        .success()
        .stdout(predicate::str::contains("Documentation Map"))
        .stdout(predicate::str::contains("Expression"))
        .stdout(predicate::str::contains("parse_binary_expression"));
}

#[tokio::test]
async fn test_doc_export_command() {
    // Mock API response
    let mock_response = serde_json::json!({
        "format": "markdown",
        "content": "# Documentation",
        "version": "1.0.0"
    })
    .to_string();

    // Setup mock server
    let mut server = mockito::Server::new_async().await;

    let _m = server
        .mock("POST", "/api/v1/docs/dsl/export")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&mock_response)
        .create_async()
        .await;

    // Create a temporary file path
    let temp_dir = std::env::temp_dir();
    let output_file = temp_dir.join("test_doc_export.md");
    let output_path = output_file.to_str().unwrap();

    // Run doc export command
    let assert = kairei_cmd()
        .arg("-u")
        .arg(server.url())
        .arg("-k")
        .arg("test-key")
        .arg("doc")
        .arg("export")
        .arg("--format")
        .arg("markdown")
        .arg("--output-file")
        .arg(output_path)
        .assert();

    // Should run successfully
    assert
        .success()
        .stdout(predicate::str::contains("Documentation exported to"));

    // Clean up the temporary file
    if output_file.exists() {
        std::fs::remove_file(output_file).unwrap();
    }
}
