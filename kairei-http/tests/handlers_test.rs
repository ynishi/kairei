use axum::http::StatusCode;
use kairei_http::{
    handlers::test_helpers::{create_test_state, create_test_user_with_api_key},
    models::agents::{AgentCreationOptions, AgentCreationRequest},
};

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
async fn test_create_agent_handler_integration() {
    // Create a request payload
    let payload = AgentCreationRequest {
        name: "TestAgent".to_string(),
        dsl_code: "micro TestAgent { }".to_string(),
        options: AgentCreationOptions { auto_start: true },
    };

    // Call the test helper function
    let (status, json_response) =
        kairei_http::handlers::test_helpers::test_create_agent(axum::response::Json(payload)).await;

    // Verify the status code
    assert_eq!(status, StatusCode::CREATED);

    // Verify the response structure
    assert_eq!(json_response.0.agent_id, "testagent-001");
    assert_eq!(format!("{:?}", json_response.0.status), "Created");
    assert!(json_response.0.validation_result.success);
}
