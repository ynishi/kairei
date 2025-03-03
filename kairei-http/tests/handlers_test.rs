use axum::{extract::Path, http::StatusCode, response::Json};
use kairei_http::{
    handlers::test_helpers::{
        test_create_agent, test_get_agent_details, test_get_system_info, test_send_agent_request,
        test_send_event,
    },
    models::agents::AgentCreationRequest,
    models::events::{AgentRequestPayload, EventRequest},
};
use serde_json::json;

#[tokio::test]
async fn test_get_system_info_handler() {
    // Call the test handler directly
    let response = test_get_system_info().await;

    // Convert to a standard response for testing
    let response_body = serde_json::to_string(&response.0).unwrap();

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

    // Call the test handler directly
    let response = test_create_agent(Json(
        serde_json::from_value::<AgentCreationRequest>(payload).unwrap(),
    ))
    .await;

    // Check the response status
    assert_eq!(response.0, StatusCode::CREATED);

    // Convert to a standard response for testing
    let response_body = serde_json::to_string(&response.1.0).unwrap();

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
    // Call the test handler directly with a valid agent ID
    let response = test_get_agent_details(Path("test-agent-001".to_string())).await;

    // Check that the response is Ok
    assert!(response.is_ok());

    // Get the response body
    let response_body = serde_json::to_string(&response.unwrap().0).unwrap();

    // Parse the response body
    let body: serde_json::Value = serde_json::from_str(&response_body).unwrap();

    // Verify the response structure
    assert!(body.get("agent_id").is_some());
    assert!(body.get("name").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("created_at").is_some());
    assert!(body.get("statistics").is_some());

    // Test with a non-existent agent ID
    let response = test_get_agent_details(Path("not-found-agent".to_string())).await;

    // Check that the response is an error with NOT_FOUND status
    assert!(response.is_err());
    assert_eq!(response.unwrap_err(), StatusCode::NOT_FOUND);
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

    // Call the test handler directly
    let response = test_send_event(Json(
        serde_json::from_value::<EventRequest>(payload).unwrap(),
    ))
    .await;

    // Convert to a standard response for testing
    let response_body = serde_json::to_string(&response.0).unwrap();

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

    // Call the test handler directly with a valid agent ID
    let response = test_send_agent_request(
        Path("weather-agent-001".to_string()),
        Json(serde_json::from_value::<AgentRequestPayload>(payload.clone()).unwrap()),
    )
    .await;

    // Check that the response is Ok
    assert!(response.is_ok());

    // Get the response body
    let response_body = serde_json::to_string(&response.unwrap().0).unwrap();

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
    let response = test_send_agent_request(
        Path("not-found-agent".to_string()),
        Json(serde_json::from_value::<AgentRequestPayload>(payload).unwrap()),
    )
    .await;

    // Check that the response is an error with NOT_FOUND status
    assert!(response.is_err());
    assert_eq!(response.unwrap_err(), StatusCode::NOT_FOUND);
}
