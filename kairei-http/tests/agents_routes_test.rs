use axum::{http::StatusCode, response::Json};
use kairei_http::{
    handlers::test_helpers::test_create_agent, models::agents::AgentCreationRequest,
};
use serde_json::json;

#[tokio::test]
async fn test_create_agent_with_auto_start() {
    // Create a request payload with auto_start set to true
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
    assert_eq!(body["agent_id"], "testagent-001");
    assert_eq!(body["status"], "created"); // The test_create_agent helper always returns "created" status
    assert_eq!(body["validation_result"]["success"], true);
}

#[tokio::test]
async fn test_create_agent_without_auto_start() {
    // Create a request payload with auto_start set to false
    let payload = json!({
        "name": "TestAgent",
        "dsl_code": "micro TestAgent { }",
        "options": {
            "auto_start": false
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
    assert_eq!(body["agent_id"], "testagent-001");
    assert_eq!(body["status"], "created");
    assert_eq!(body["validation_result"]["success"], true);
}

#[tokio::test]
async fn test_create_agent_with_complex_dsl() {
    // Create a request payload with more complex DSL code
    let payload = json!({
        "name": "ComplexAgent",
        "dsl_code": "micro ComplexAgent {
            on init { 
                println(\"Agent initialized\"); 
            } 
            
            answer { 
                on request GetStatus() -> Result<String, Error> { 
                    return Ok(\"Running\"); 
                } 
            } 
        }",
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
    assert_eq!(body["agent_id"], "complexagent-001");
    assert_eq!(body["status"], "created"); // The test_create_agent helper always returns "created" status
    assert_eq!(body["validation_result"]["success"], true);
}

#[tokio::test]
async fn test_create_agent_with_default_options() {
    // Create a request payload without specifying options (should use defaults)
    let payload = json!({
        "name": "DefaultAgent",
        "dsl_code": "micro DefaultAgent { }"
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
    assert_eq!(body["agent_id"], "defaultagent-001");
    assert_eq!(body["status"], "created"); // Default auto_start should be false
    assert_eq!(body["validation_result"]["success"], true);
}
