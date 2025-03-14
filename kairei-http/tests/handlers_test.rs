use axum::{extract::Path, http::StatusCode, response::Json};
use kairei_http::{
    handlers::test_helpers::{
        create_test_state, create_test_user_with_api_key, test_create_agent, test_get_agent,
        test_get_system, test_send_agent_request, test_send_event, test_suggest_fixes,
        test_validate_dsl,
    },
    models::{
        agents::AgentCreationRequest,
        events::{AgentRequestPayload, EventRequest},
    },
    services::compiler::models::{
        SuggestionRequest, ValidationError, ValidationRequest, ValidationResponse,
    },
};
use serde_json::json;

#[tokio::test]
async fn test_auth_store_default_users() {
    // Create a test state with default users
    let app_state = create_test_state();

    // Verify that the default admin user exists
    let admin_user = app_state.auth_store.get_user_by_api_key("admin-key");
    assert!(admin_user.is_some());
    assert_eq!(admin_user.unwrap().user_id, "admin");

    // Verify that the default regular users exist
    let user1 = app_state.auth_store.get_user_by_api_key("user1-key");
    assert!(user1.is_some());
    assert_eq!(user1.unwrap().user_id, "user1");

    let user2 = app_state.auth_store.get_user_by_api_key("user2-key");
    assert!(user2.is_some());
    assert_eq!(user2.unwrap().user_id, "user2");
}

#[tokio::test]
async fn test_create_custom_user() {
    // Create a test state
    let app_state = create_test_state();

    // Create a custom test user
    create_test_user_with_api_key(&app_state, "test-user", "Test User", false, "test-key");

    // Verify that the custom user exists
    let user = app_state.auth_store.get_user_by_api_key("test-key");
    assert!(user.is_some());
    assert_eq!(user.unwrap().user_id, "test-user");
}

#[tokio::test]
async fn test_get_system_handler() {
    // Call the test handler directly
    let response = test_get_system().await;

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
async fn test_get_agent_handler() {
    // Call the test handler directly with a valid agent ID
    let response = test_get_agent(Path("test-agent-001".to_string())).await;

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
    let response = test_get_agent(Path("not-found-agent".to_string())).await;

    // Check that the response is an error with NOT_FOUND status
    assert!(response.is_err());
    assert_eq!(response.unwrap_err(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_validate_dsl_handler_valid_code() {
    // Create a request payload with valid DSL code
    let payload = json!({
        "code": "micro TestAgent { }"
    });

    // Call the test handler directly
    let response = test_validate_dsl(Json(
        serde_json::from_value::<ValidationRequest>(payload).unwrap(),
    ))
    .await;

    // Convert to a standard response for testing
    let response_body = serde_json::to_string(&response.0).unwrap();

    // Parse the response body
    let _: ValidationResponse = serde_json::from_str(&response_body).unwrap();
}

#[tokio::test]
async fn test_validate_dsl_handler_invalid_code() {
    // Create a request payload with invalid DSL code
    let payload = json!({
        "code": "micro TestAgent { ERROR on init { println(\"Hello, World!\"); } }"
    });

    // Call the test handler directly
    let response = test_validate_dsl(Json(
        serde_json::from_value::<ValidationRequest>(payload).unwrap(),
    ))
    .await;

    // Convert to a standard response for testing
    let response_body = serde_json::to_string(&response.0).unwrap();

    // Parse the response body
    let _: ValidationResponse = serde_json::from_str(&response_body).unwrap();
}

#[tokio::test]
async fn test_validate_dsl_handler_with_warnings() {
    // Create a request payload with code that has warnings
    let payload = json!({
        "code": "micro TestAgent { WARNING on init { println(\"Hello, World!\"); } }"
    });

    // Call the test handler directly
    let response = test_validate_dsl(Json(
        serde_json::from_value::<ValidationRequest>(payload).unwrap(),
    ))
    .await;

    // Convert to a standard response for testing
    let response_body = serde_json::to_string(&response.0).unwrap();

    // Parse the response body
    let _: ValidationResponse = serde_json::from_str(&response_body).unwrap();
}

#[tokio::test]
async fn test_suggest_fixes_handler() {
    // Create a validation error
    let error = ValidationError {
        message: "Parse error: unexpected token".to_string(),
        location: kairei_http::services::compiler::models::ErrorLocation {
            line: 1,
            column: 15,
            start_position: None,
            end_position: None,
            context: "micro TestAgent { ERROR }".to_string(),
            token_text: None,
        },
        error_code: "E1001".to_string(),
        suggestion: "Check syntax for errors".to_string(),
    };

    // Create a request payload
    let payload = json!({
        "code": "micro TestAgent { ERROR }",
        "errors": [error]
    });

    // Call the test handler directly
    let response = test_suggest_fixes(Json(
        serde_json::from_value::<SuggestionRequest>(payload).unwrap(),
    ))
    .await;

    // Convert to a standard response for testing
    let response_body = serde_json::to_string(&response.0).unwrap();

    // Parse the response body
    let body: serde_json::Value = serde_json::from_str(&response_body).unwrap();

    // Verify the response structure
    assert!(body.get("original_code").is_some());
    assert!(body.get("fixed_code").is_some());
    assert!(body.get("explanation").is_some());

    // Verify specific values
    assert_eq!(body["original_code"], "micro TestAgent { ERROR }");
    assert_eq!(body["fixed_code"], "micro TestAgent {  }");
    assert!(
        body["explanation"]
            .as_str()
            .unwrap()
            .contains("Removed syntax errors")
    );
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
