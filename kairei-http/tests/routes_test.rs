use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kairei_core::{
    config::{ProviderConfig, ProviderConfigs, SystemConfig},
    provider::provider::ProviderType,
    system::SystemStatus,
};
use kairei_http::{
    auth::auth_middleware,
    handlers::test_helpers::create_test_state,
    models::{CreateSystemRequest, CreateSystemResponse, ListSystemsResponse},
    routes,
};
use serde_json::json;
use tower::ServiceExt;

fn create_test_system_config() -> SystemConfig {
    SystemConfig {
        provider_configs: {
            let mut providers = HashMap::new();
            providers.insert(
                "default_provider".to_string(),
                ProviderConfig {
                    provider_type: ProviderType::SimpleExpert,
                    ..Default::default()
                },
            );
            ProviderConfigs {
                providers,
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

#[tokio::test]
async fn test_system_route() {
    let app_state: kairei_http::server::AppState = create_test_state();

    // Create the router with a test state
    let app = routes::create_api_router()
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(app_state.auth_store.clone()),
            auth_middleware,
        ))
        .into_service();

    // Create a request to the create system
    let request_body = CreateSystemRequest {
        name: "TestSystem".to_string(),
        config: create_test_system_config(),
        ..Default::default()
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

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();
    let resp: CreateSystemResponse = serde_json::from_slice(&body).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Verify the response structure
    let system_id = resp.system_id.clone();
    assert!(!system_id.is_empty());

    // Create a request to list systems
    let request = Request::builder()
        .uri("/api/v1/systems")
        .method("GET")
        .header("X-API-Key", "admin-key")
        .body("".to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();
    let resp: ListSystemsResponse = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(resp.system_statuses.get(&system_id).unwrap().running);

    // Get system
    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}", system_id))
        .method("GET")
        .header("X-API-Key", "admin-key")
        .body("".to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    let resp: SystemStatus = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(resp.running);

    // Start system
    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}/start", system_id))
        .method("POST")
        .header("X-API-Key", "admin-key")
        .body("".to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    assert!(body.is_empty());

    // Verify the response structure
    assert!(resp.running);

    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}/shutdown", system_id))
        .method("POST")
        .header("X-API-Key", "admin-key")
        .body("".to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    assert!(body.is_empty());

    // Remove system
    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}", system_id))
        .method("DELETE")
        .header("X-API-Key", "admin-key")
        .body("".to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_agent_route() {
    // Create the router with a test state
    let app_state: kairei_http::server::AppState = create_test_state();
    let app = routes::create_api_router()
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(app_state.auth_store.clone()),
            auth_middleware,
        ))
        .into_service();

    // Create a request to create an agent
    let request_body = json!({
        "name": "TestAgent",
        "dsl_code": "micro TestAgent { }",
        "options": {
            "auto_start": true
        }
    });

    let request = Request::builder()
        .uri("/api/v1/users/admin/agents")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-API-Key", "admin-key")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    // Process the request
    let response = app.oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::CREATED);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(body.get("agent_id").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("validation_result").is_some());
}

#[tokio::test]
async fn test_get_agent_details_route() {
    // Create the router with a test state
    let app_state: kairei_http::server::AppState = create_test_state();
    let app = routes::create_api_router()
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(app_state.auth_store.clone()),
            auth_middleware,
        ))
        .into_service();

    // Create a request to get agent details
    let request = Request::builder()
        .uri("/api/v1/agents/test-agent-001")
        .method("GET")
        .header("X-API-Key", "admin-key")
        .body(Body::empty())
        .unwrap();

    // Process the request
    let response = app.oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(body.get("agent_id").is_some());
    assert!(body.get("name").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("created_at").is_some());
    assert!(body.get("statistics").is_some());
}

#[tokio::test]
async fn test_send_event_route() {
    // Create the router with a test state
    let app_state: kairei_http::server::AppState = create_test_state();
    let app = routes::create_api_router()
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(app_state.auth_store.clone()),
            auth_middleware,
        ))
        .into_service();

    // Create a request to send an event
    let request_body = json!({
        "event_type": "WeatherUpdate",
        "payload": {
            "location": "Tokyo",
            "temperature": 25.5,
            "conditions": "Sunny"
        },
        "target_agents": ["weather-agent-001"]
    });

    let request = Request::builder()
        .uri("/api/v1/events")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-API-Key", "admin-key")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    // Process the request
    let response = app.oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(body.get("event_id").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("delivered_to").is_some());
}

#[tokio::test]
async fn test_send_agent_request_route() {
    // Create the router with a test state
    let app_state: kairei_http::server::AppState = create_test_state();
    let app = routes::create_api_router()
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(app_state.auth_store.clone()),
            auth_middleware,
        ))
        .into_service();

    // Create a request to send a request to an agent
    let request_body = json!({
        "request_type": "GetWeather",
        "parameters": {
            "location": "Tokyo"
        }
    });

    let request = Request::builder()
        .uri("/api/v1/events/agents/weather-agent-001/request")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-API-Key", "admin-key")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    // Process the request
    let response = app.oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(body.get("request_id").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("result").is_some());
}
