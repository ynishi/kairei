use axum::{extract::Request, http::StatusCode};
use kairei_http::{
    auth::auth_middleware,
    handlers::test_helpers::create_test_state,
    models::{CreateSystemRequest, CreateSystemResponse, ListSystemsResponse, StartSystemRequest},
    routes,
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_system_route() {
    // Create the router with a test state
    let app_state: kairei_http::server::AppState = create_test_state();
    let config = kairei_http::server::ServerConfig::default();

    let app = routes::create_api_router(&config)
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(app_state.auth_store.clone()),
            auth_middleware,
        ))
        .into_service();

    // Test creating a system
    let request_body = CreateSystemRequest {
        name: "TestSystem".to_string(),
        description: Some("A test system".to_string()),
        config: kairei_core::config::SystemConfig::default(),
    };

    let request = Request::builder()
        .uri("/api/v1/systems")
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body(json!(request_body).to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    let _body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    // Test listing systems
    let request = Request::builder()
        .uri("/api/v1/systems")
        .method("GET")
        .header("X-API-Key", "admin-key")
        .body("".to_string())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let _body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    let resp: ListSystemsResponse = serde_json::from_slice(&_body).unwrap();
    assert!(!resp.system_statuses.is_empty());

    // Get first system ID
    let system_id = resp.system_statuses.keys().next().unwrap();

    // Test getting a system
    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}", system_id))
        .method("GET")
        .header("X-API-Key", "admin-key")
        .body("".to_string())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let _body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    // Test starting a system
    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}/start", system_id))
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body(json!(StartSystemRequest { dsl: None }).to_string())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let _body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    // Test stopping a system
    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}/stop", system_id))
        .method("POST")
        .header("X-API-Key", "admin-key")
        .body("".to_string())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_create_agent_in_system_route() {
    // Create the router with a test state
    let app_state: kairei_http::server::AppState = create_test_state();
    let config = kairei_http::server::ServerConfig::default();

    let app = routes::create_api_router(&config)
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(app_state.auth_store.clone()),
            auth_middleware,
        ))
        .into_service();

    // Setup system
    let request_body = CreateSystemRequest {
        name: "TestSystem".to_string(),
        description: Some("A test system".to_string()),
        config: kairei_core::config::SystemConfig::default(),
    };

    let request = Request::builder()
        .uri("/api/v1/systems")
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body(json!(request_body).to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    let _body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    let resp: CreateSystemResponse = serde_json::from_slice(&_body).unwrap();
    let system_id = resp.system_id.clone();

    // Create agent request
    let agent_request = json!({
        "name": "NewAgent",
        "dsl_code": r#"micro NewAgent {
            answer {
                on request GetValue() -> Result<Int, Error> {
                    return Ok(42)
                }
            }
        }"#,
        "options": {
            "auto_start": true
        }
    });

    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}/agents", system_id))
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body(agent_request.to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::CREATED);

    // Get the response body
    let _body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    let agent_response: serde_json::Value = serde_json::from_slice(&_body).unwrap();

    // Verify the response structure
    assert_eq!(agent_response["agent_id"], "NewAgent");
    // Check status is either "Created" or "Running" based on auto_start option
    assert!(agent_response["status"] == "Created" || agent_response["status"] == "Running");
    assert!(
        agent_response["validation_result"]["success"]
            .as_bool()
            .unwrap()
    );
}

#[tokio::test]
async fn test_create_agent_route_errors() {
    // Create the router with a test state
    let app_state: kairei_http::server::AppState = create_test_state();
    let config = kairei_http::server::ServerConfig::default();

    let app = routes::create_api_router(&config)
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(app_state.auth_store.clone()),
            auth_middleware,
        ))
        .into_service();

    // Setup system
    let request_body = CreateSystemRequest {
        name: "TestSystem".to_string(),
        description: Some("A test system".to_string()),
        config: kairei_core::config::SystemConfig::default(),
    };

    let request = Request::builder()
        .uri("/api/v1/systems")
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body(json!(request_body).to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    let _body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    let resp: CreateSystemResponse = serde_json::from_slice(&_body).unwrap();
    let system_id = resp.system_id.clone();

    // Test unauthorized access
    let agent_request = json!({
        "name": "NewAgent",
        "dsl_code": "micro NewAgent { }",
        "options": {
            "auto_start": false
        }
    });

    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}/agents", system_id))
        .method("POST")
        .header("X-API-Key", "user1-key") // Non-admin user
        .header("Content-Type", "application/json")
        .body(agent_request.to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Test non-existent system
    let request = Request::builder()
        .uri("/api/v1/systems/non-existent-system/agents")
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body(agent_request.to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
