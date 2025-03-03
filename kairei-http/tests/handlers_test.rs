use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use kairei_http::handlers::{
    agents::{create_agent, get_agent_details},
    events::{send_agent_request, send_event},
    system::get_system_info,
};
use serde_json::json;
// Arc is used in the create_mock_kairei_system function

// Import the create_mock_kairei_system function directly
#[path = "mocks.rs"]
mod mocks;
use mocks::create_mock_kairei_system;

#[tokio::test]
async fn test_get_system_info_handler() {
    // Create a mock KaireiSystem
    let kairei_system = create_mock_kairei_system();

    // Call the handler directly
    let response = get_system_info(State(kairei_system)).await;

    // Get the response
    let response = response.unwrap();

    // Extract the inner value from Json wrapper and convert to a standard response for testing
    let inner_response = response.0;
    let response_body = serde_json::to_string(&inner_response).unwrap();

    // Parse the response body
    let body: serde_json::Value = serde_json::from_str(&response_body).unwrap();

    // Verify the response structure
    assert!(body.get("version").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("capabilities").is_some());
    assert!(body.get("statistics").is_some());

    // Verify specific values
    assert_eq!(body["version"], "0.1.0");
    assert_eq!(body["status"], "running");
    assert!(!body["capabilities"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_agent_handler() {
    // Create a request payload
    let payload = json!({
        "name": "TestAgent",
        "dsl_code": "micro TestAgent { }",
        "options": {
            "auto_start": true
        }
    });

    // Create a mock KaireiSystem
    let kairei_system = create_mock_kairei_system();

    // Call the handler directly
    let response = create_agent(
        State(kairei_system),
        Json(serde_json::from_value(payload).unwrap()),
    )
    .await;

    // Check the response status
    let (status, json_response) = response.unwrap();
    assert_eq!(status, StatusCode::CREATED);

    // Convert to a standard response for testing
    let response_body = serde_json::to_string(&json_response.0).unwrap();

    // Parse the response body
    let body: serde_json::Value = serde_json::from_str(&response_body).unwrap();

    // Verify the response structure
    assert!(body.get("agent_id").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("validation_result").is_some());

    // Verify specific values
    assert_eq!(body["status"], "created");
    assert_eq!(body["validation_result"]["success"], true);
}

#[tokio::test]
async fn test_get_agent_details_handler() {
    // Create a mock KaireiSystem
    let kairei_system = create_mock_kairei_system();

    // Call the handler directly with a valid agent ID
    let response = get_agent_details(
        State(kairei_system.clone()),
        Path("test-agent-001".to_string()),
    )
    .await;

    // Check that the response is Ok
    assert!(response.is_ok());

    // Extract the inner value from Json wrapper
    let inner_response = response.unwrap().0;

    // Get the response body
    let response_body = serde_json::to_string(&inner_response).unwrap();

    // Parse the response body
    let body: serde_json::Value = serde_json::from_str(&response_body).unwrap();

    // Verify the response structure
    assert!(body.get("agent_id").is_some());
    assert!(body.get("name").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("created_at").is_some());
    assert!(body.get("statistics").is_some());

    // Test with a non-existent agent ID
    let response =
        get_agent_details(State(kairei_system), Path("not-found-agent".to_string())).await;

    // Check that the response is an error
    assert!(response.is_err());

    // Get the error and check that it's an AppError that maps to NOT_FOUND status
    let err = response.unwrap_err();
    assert!(err == StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_send_event_handler() {
    // Create a request payload
    let payload = json!({
        "event_type": "WeatherUpdate",
        "payload": {
            "location": "Tokyo",
            "temperature": 25.5,
            "conditions": "Sunny"
        },
        "target_agents": ["weather-agent-001"]
    });

    // Create a mock KaireiSystem
    let kairei_system = create_mock_kairei_system();

    // Call the handler directly
    let response = send_event(
        State(kairei_system),
        Json(serde_json::from_value(payload).unwrap()),
    )
    .await;

    // Get the response
    let response = response.unwrap();

    // Extract the inner value from Json wrapper and convert to a standard response for testing
    let inner_response = response.0;
    let response_body = serde_json::to_string(&inner_response).unwrap();

    // Parse the response body
    let body: serde_json::Value = serde_json::from_str(&response_body).unwrap();

    // Verify the response structure
    assert!(body.get("event_id").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("delivered_to").is_some());

    // Verify specific values
    assert!(body["event_id"].as_str().unwrap().starts_with("evt-"));
    assert_eq!(body["status"], "delivered");
    assert!(body["delivered_to"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn test_send_agent_request_handler() {
    // Create a request payload
    let payload = json!({
        "request_type": "GetWeather",
        "parameters": {
            "location": "Tokyo"
        }
    });

    // Create a mock KaireiSystem
    let kairei_system = create_mock_kairei_system();

    // Call the handler directly with a valid agent ID
    let response = send_agent_request(
        State(kairei_system.clone()),
        Path("weather-agent-001".to_string()),
        Json(serde_json::from_value(payload.clone()).unwrap()),
    )
    .await;

    // Check that the response is Ok
    assert!(response.is_ok());

    // Extract the inner value from Json wrapper
    let inner_response = response.unwrap().0;

    // Get the response body
    let response_body = serde_json::to_string(&inner_response).unwrap();

    // Parse the response body
    let body: serde_json::Value = serde_json::from_str(&response_body).unwrap();

    // Verify the response structure
    assert!(body.get("request_id").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("result").is_some());

    // Verify specific values
    assert!(body["request_id"].as_str().unwrap().starts_with("req-"));
    assert_eq!(body["status"], "completed");
    assert_eq!(body["result"]["location"], "Tokyo");

    // Test with a non-existent agent ID
    let response = send_agent_request(
        State(kairei_system),
        Path("not-found-agent".to_string()),
        Json(serde_json::from_value(payload).unwrap()),
    )
    .await;

    // Check that the response is an error
    assert!(response.is_err());

    // Get the error and check that it's an AppError that maps to NOT_FOUND status
    let err = response.unwrap_err();
    assert!(err == StatusCode::NOT_FOUND);
}
