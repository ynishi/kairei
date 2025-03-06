use std::{collections::HashMap, sync::Arc};

use axum::http::{Request, StatusCode};
use kairei_core::{
    config::{ProviderConfig, ProviderConfigs, SystemConfig},
    provider::provider::ProviderType,
    system::SystemStatus,
};
use kairei_http::{
    auth::auth_middleware,
    handlers::test_helpers::create_test_state,
    models::{
        CreateSystemRequest, CreateSystemResponse, EventRequest, GetAgentResponse,
        ListAgentsResponse, ListSystemsResponse, ScaleDownAgentRequest, ScaleUpAgentRequest,
        SendRequestAgentRequest, StartSystemRequest,
    },
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
        .header("Content-Type", "application/json")
        .header("X-API-Key", "admin-key")
        .body(json!(StartSystemRequest { dsl: None }).to_string())
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
        .uri(format!("/api/v1/systems/{}/stop", system_id))
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
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
async fn test_agent_route() {
    // Create the router with a test state
    let app_state: kairei_http::server::AppState = create_test_state();
    let app = routes::create_api_router()
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(app_state.auth_store.clone()),
            auth_middleware,
        ))
        .into_service();

    // setup system
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
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    let resp: CreateSystemResponse = serde_json::from_slice(&body).unwrap();
    let system_id = resp.system_id.clone();

    // start system
    let request_body = json!(StartSystemRequest {
        dsl: Some(
            r#"micro Counter {
            answer {
                on request GetCount() -> Result<Int, Error> {
                    return Ok(1)
                }
            }
        }"#
            .to_string()
        )
    });
    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}/start", system_id))
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body(request_body.to_string())
        .unwrap();

    //  return;
    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    std::thread::sleep(std::time::Duration::from_millis(100));

    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}/agents", system_id))
        .method("GET")
        .header("Content-Type", "application/json")
        .header("X-API-Key", "admin-key")
        .body(json!(request_body).to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    let list_agents_response: ListAgentsResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(list_agents_response.agents.len(), 3);
    let agent_id = list_agents_response
        .agents
        .iter()
        .find(|e| e.agent_id == "Counter")
        .unwrap()
        .agent_id
        .clone();

    // Get agent
    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}/agents/{}", system_id, agent_id))
        .method("GET")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body("".to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    assert!(response.status().is_success());

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    let agent: GetAgentResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(agent.agent_id, agent_id);

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create a request to create an agent
    let request_body = json!(SendRequestAgentRequest {
        request_type: "GetCount".to_string(),
        payload: serde_json::Value::Null
    });

    let request = Request::builder()
        .uri(format!(
            "/api/v1/systems/{}/agents/{}/request",
            system_id, agent_id
        ))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-API-Key", "admin-key")
        .body(request_body.to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    // panic!("request: abc");

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify the response structure
    assert_eq!(
        body,
        json!(
            {
                "value": 1
            }
        )
    );

    // Scale up agent
    let request_body = json!(ScaleUpAgentRequest {
        instances: 1,
        options: Default::default()
    });
    let request = Request::builder()
        .uri(format!(
            "/api/v1/systems/{}/agents/{}/scaleup",
            system_id, agent_id
        ))
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body(request_body.to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request_body = json!(ScaleDownAgentRequest {
        instances: 1,
        ..Default::default()
    });

    // Scale down agent
    let request = Request::builder()
        .uri(format!(
            "/api/v1/systems/{}/agents/{}/scaledown",
            system_id, agent_id
        ))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-API-Key", "admin-key")
        .body(request_body.to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Stop agent
    let request = Request::builder()
        .uri(format!(
            "/api/v1/systems/{}/agents/{}/stop",
            system_id, agent_id
        ))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-API-Key", "admin-key")
        .body("".to_string())
        .unwrap();

    // Process the request
    let response = app.clone().oneshot(request).await.unwrap();
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_event_route() {
    let app_state: kairei_http::server::AppState = create_test_state();

    // Create the router with a test state
    let app = routes::create_api_router()
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(app_state.auth_store.clone()),
            auth_middleware,
        ))
        .into_service();

    // setup system
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
    let body = axum::body::to_bytes(response.into_body(), 1000)
        .await
        .unwrap();

    let resp: CreateSystemResponse = serde_json::from_slice(&body).unwrap();
    let system_id = resp.system_id.clone();

    std::thread::sleep(std::time::Duration::from_millis(100));

    let request = Request::builder()
        .uri(format!("/api/v1/systems/{}/events", system_id))
        .method("GET")
        .header("X-API-Key", "admin-key")
        .body("".to_string())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);

    let request = Request::builder()
        .uri(format!(
            "/api/v1/systems/{}/events/{}/emit",
            system_id, "test_event"
        ))
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body(json!(EventRequest::default()).to_string())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);

    let request = Request::builder()
        .uri(format!(
            "/api/v1/systems/{}/events/{}/subscribe",
            system_id, "test_event"
        ))
        .method("POST")
        .header("X-API-Key", "admin-key")
        .header("Content-Type", "application/json")
        .body("".to_string())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
}
