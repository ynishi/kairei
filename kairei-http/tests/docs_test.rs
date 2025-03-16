use std::convert::Infallible;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use kairei_http::{routes::api::v1::docs::routes, server::AppState};
use serde_json::json;
use tower::{Service, ServiceExt};

// Create a test app with only the documentation routes for testing
fn create_test_app()
-> impl Service<Request<Body>, Response = axum::response::Response, Error = Infallible> {
    // Create a minimal app state
    let app_state = AppState::default();
    // Create a router with only docs routes, nested under /api/v1 to match our test paths
    Router::new()
        .nest("/api/v1", routes())
        .with_state(app_state)
        .into_service()
}

#[tokio::test]
async fn test_get_all_documentation() {
    // Create a request to get all documentation
    let request = Request::builder()
        .uri("/api/v1/docs/dsl")
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    // Create a new app for this request and send it
    let response = create_test_app().oneshot(request).await.unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_category_documentation() {
    // Create a request to get expression category documentation
    let request = Request::builder()
        .uri("/api/v1/docs/dsl/expression")
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    // Create a new app for this request and send the request
    let response = create_test_app().oneshot(request).await.unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);

    // Test invalid category
    let request = Request::builder()
        .uri("/api/v1/docs/dsl/invalid_category_that_should_not_exist")
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    // Create a new app for this request and send it
    let response = create_test_app().oneshot(request).await.unwrap();

    // Even non-existent categories may return Not Found
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_parser_documentation() {
    // Create a request to get a specific parser's documentation
    let request = Request::builder()
        .uri("/api/v1/docs/dsl/expression/parse_think")
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    // Create a new app for this request and send it
    let response = create_test_app().oneshot(request).await.unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);

    // Test non-existent parser
    let request = Request::builder()
        .uri("/api/v1/docs/dsl/expression/nonexistent_parser")
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    // Create a new app for this request and send it
    let response = create_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_content_negotiation() {
    // Create a request with markdown accept header
    let request = Request::builder()
        .uri("/api/v1/docs/dsl")
        .method("GET")
        .header("Accept", "text/markdown")
        .body(Body::empty())
        .unwrap();

    // Create a new app for this request and send it
    let response = create_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/markdown"));

    // Create a request with format parameter
    let request = Request::builder()
        .uri("/api/v1/docs/dsl?format=markdown")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    // Create a new app for this request and send it
    let response = create_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/markdown"));
}

#[tokio::test]
async fn test_get_documentation_map() {
    // Create a request to get documentation map
    let request = Request::builder()
        .uri("/api/v1/docs/dsl/map")
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    // Create a new app for this request and send it
    let response = create_test_app().oneshot(request).await.unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(response.get("version").is_some());
    assert!(response.get("categories").is_some());
    assert!(response.get("parsers_by_category").is_some());
}

#[tokio::test]
async fn test_export_documentation() {
    // Create a request to export documentation
    let request_body = json!({
        "format": "markdown",
        "include_version": true
    });

    let request = Request::builder()
        .uri("/api/v1/docs/dsl/export")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();

    // Create a new app for this request and send it
    let response = create_test_app().oneshot(request).await.unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert!(response.get("format").is_some());
    assert!(response.get("content").is_some());
    assert!(response.get("version").is_some());

    // Test JSON format
    let request_body = json!({
        "format": "json",
        "include_version": true
    });

    let request = Request::builder()
        .uri("/api/v1/docs/dsl/export")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();

    // Create a new app for this request and send it
    let response = create_test_app().oneshot(request).await.unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);
}
