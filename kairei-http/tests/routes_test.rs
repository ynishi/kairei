use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kairei_http::routes;
use serde_json::json;
// Arc is used in the create_mock_kairei_system function
use tower::ServiceExt;

// Import the create_mock_kairei_system function directly
#[path = "mocks.rs"]
mod mocks;
use mocks::create_mock_kairei_system;

#[tokio::test]
async fn test_system_info_route() {
    // Create a mock KaireiSystem
    let kairei_system = create_mock_kairei_system();

    // Create the router
    let app = routes::create_api_router(kairei_system);

    // Create a request to the system info endpoint
    let request = Request::builder()
        .uri("/api/v1/system/info")
        .method("GET")
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
    assert!(body.get("version").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("capabilities").is_some());
    assert!(body.get("statistics").is_some());
}

#[tokio::test]
async fn test_create_agent_route() {
    // Create a mock KaireiSystem
    let kairei_system = create_mock_kairei_system();

    // Create the router
    let app = routes::create_api_router(kairei_system);

    // Create a request to create an agent
    let request_body = json!({
        "name": "TestAgent",
        "dsl_code": "micro TestAgent { }",
        "options": {
            "auto_start": true
        }
    });

    let request = Request::builder()
        .uri("/api/v1/agents")
        .method("POST")
        .header("Content-Type", "application/json")
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
    // Create a mock KaireiSystem
    let kairei_system = create_mock_kairei_system();

    // Create the router
    let app = routes::create_api_router(kairei_system);

    // Create a request to get agent details
    let request = Request::builder()
        .uri("/api/v1/agents/test-agent-001")
        .method("GET")
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
    // Create a mock KaireiSystem
    let kairei_system = create_mock_kairei_system();

    // Create the router
    let app = routes::create_api_router(kairei_system);

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
    // Create a mock KaireiSystem
    let kairei_system = create_mock_kairei_system();

    // Create the router
    let app = routes::create_api_router(kairei_system);

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
