use kairei_core::system::SystemStatus;
use kairei_http::models::{
    AgentCreationRequest, AgentStatus, CreateSystemRequest, CreateSystemResponse, EventRequest,
    ListSystemsResponse, StartSystemRequest,
};
use reqwest::{Client, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("API error: {status} - {message}")]
    Api { status: StatusCode, message: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type ApiResult<T> = Result<T, ApiError>;

pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: SecretString,
}

impl ApiClient {
    pub fn new(base_url: &str, api_key: &SecretString) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.clone(),
        }
    }

    /// Make a generic API request and return the response as JSON
    async fn request<T, R>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&T>,
    ) -> ApiResult<R>
    where
        T: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);

        let mut request = self
            .client
            .request(method, &url)
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.expose_secret()),
            )
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json::<R>().await?)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ApiError::Api {
                status,
                message: error_text,
            })
        }
    }

    /// Convert any response to a JSON value for display
    pub fn to_json<T: Serialize>(data: &T) -> ApiResult<Value> {
        Ok(serde_json::to_value(data)?)
    }

    // System endpoints

    pub async fn list_systems(&self) -> ApiResult<ListSystemsResponse> {
        self.request(reqwest::Method::GET, "/api/v1/systems", None::<&()>)
            .await
    }

    pub async fn create_system(
        &self,
        name: &str,
        description: Option<&str>,
        config: kairei_core::config::SystemConfig,
    ) -> ApiResult<CreateSystemResponse> {
        let request = CreateSystemRequest {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            config,
        };

        self.request(reqwest::Method::POST, "/api/v1/systems", Some(&request))
            .await
    }

    pub async fn get_system(&self, system_id: &str) -> ApiResult<SystemStatus> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/systems/{}", system_id),
            None::<&()>,
        )
        .await
    }

    pub async fn start_system(&self, system_id: &str, dsl: Option<&str>) -> ApiResult<Value> {
        let request = StartSystemRequest {
            dsl: dsl.map(|s| s.to_string()),
        };

        self.request(
            reqwest::Method::POST,
            &format!("/api/v1/systems/{}/start", system_id),
            Some(&request),
        )
        .await
    }

    pub async fn stop_system(&self, system_id: &str) -> ApiResult<Value> {
        self.request(
            reqwest::Method::POST,
            &format!("/api/v1/systems/{}/stop", system_id),
            None::<&()>,
        )
        .await
    }

    pub async fn delete_system(&self, system_id: &str) -> ApiResult<Value> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/api/v1/systems/{}", system_id),
            None::<&()>,
        )
        .await
    }

    // Agent endpoints

    pub async fn list_agents(&self, system_id: &str) -> ApiResult<Value> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/systems/{}/agents", system_id),
            None::<&()>,
        )
        .await
    }

    pub async fn get_agent(&self, system_id: &str, agent_id: &str) -> ApiResult<AgentStatus> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/systems/{}/agents/{}", system_id, agent_id),
            None::<&()>,
        )
        .await
    }

    pub async fn create_agent(
        &self,
        _system_id: &str,
        _definition: &AgentCreationRequest,
    ) -> ApiResult<Value> {
        todo!()
    }

    pub async fn update_agent(
        &self,
        _system_id: &str,
        _agent_id: &str,
        _definition: &AgentCreationRequest,
    ) -> ApiResult<Value> {
        todo!()
    }

    pub async fn delete_agent(&self, system_id: &str, agent_id: &str) -> ApiResult<Value> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/api/v1/systems/{}/agents/{}", system_id, agent_id),
            None::<&()>,
        )
        .await
    }

    pub async fn start_agent(&self, system_id: &str, agent_id: &str) -> ApiResult<Value> {
        self.request(
            reqwest::Method::POST,
            &format!("/api/v1/systems/{}/agents/{}/start", system_id, agent_id),
            None::<&()>,
        )
        .await
    }

    pub async fn stop_agent(&self, system_id: &str, agent_id: &str) -> ApiResult<Value> {
        self.request(
            reqwest::Method::POST,
            &format!("/api/v1/systems/{}/agents/{}/stop", system_id, agent_id),
            None::<&()>,
        )
        .await
    }

    // Event endpoints

    pub async fn list_events(&self, system_id: &str) -> ApiResult<Value> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/systems/{}/events", system_id),
            None::<&()>,
        )
        .await
    }

    pub async fn emit_event(&self, system_id: &str, event: &EventRequest) -> ApiResult<Value> {
        self.request(
            reqwest::Method::POST,
            &format!("/api/v1/systems/{}/events", system_id),
            Some(event),
        )
        .await
    }

    // Utility functions

    pub async fn health_check(&self) -> ApiResult<Value> {
        self.request(reqwest::Method::GET, "/health", None::<&()>)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use tokio;

    // Helper to create a test API client with the mockito server URL
    fn create_test_client(server: &mockito::Server) -> ApiClient {
        ApiClient::new(
            &server.url(),
            &SecretString::from(Box::from("test-api-key")),
        )
    }

    #[tokio::test]
    async fn test_list_systems() {
        // Create a mock server
        let mut server = mockito::Server::new_async().await;

        let mock_response = r#"
        {
            "system_statuses": {
                "system-1": {
                    "started_at": "2023-01-01T00:00:00Z",
                    "running": true,
                    "uptime": 3600,
                    "agent_count": 5,
                    "running_agent_count": 3,
                    "event_queue_size": 10,
                    "event_subscribers": 2,
                    "event_capacity": 100
                }
            }
        }
        "#;

        // Setup mock
        let _m = server
            .mock("GET", "/api/v1/systems")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        // Test API client
        let client = create_test_client(&server);
        let response = client.list_systems().await.unwrap();

        // Verify response
        assert_eq!(response.system_statuses.len(), 1);
        assert!(response.system_statuses.contains_key("system-1"));
        assert_eq!(response.system_statuses["system-1"].agent_count, 5);
    }

    #[tokio::test]
    async fn test_create_system() {
        // Create a mock server
        let mut server = mockito::Server::new_async().await;

        let mock_response = r#"
        {
            "system_id": "new-system-123",
            "session_id": "session-456"
        }
        "#;

        // Setup mock
        let _m = server
            .mock("POST", "/api/v1/systems")
            .match_header("content-type", "application/json")
            .match_header("authorization", "Bearer test-api-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        // Test API client
        let client = create_test_client(&server);
        let response = client
            .create_system(
                "test-system",
                Some("Test system description"),
                kairei_core::config::SystemConfig::default(),
            )
            .await
            .unwrap();

        // Verify response
        assert_eq!(response.system_id, "new-system-123");
        assert_eq!(response.session_id, "session-456");
    }

    #[tokio::test]
    async fn test_get_system() {
        // Create a mock server
        let mut server = mockito::Server::new_async().await;

        let mock_response = r#"
        {
            "started_at": "2023-01-01T00:00:00Z",
            "running": true,
            "uptime": 3600,
            "agent_count": 5,
            "running_agent_count": 3,
            "event_queue_size": 10,
            "event_subscribers": 2,
            "event_capacity": 100
        }
        "#;

        // Setup mock
        let _m = server
            .mock("GET", "/api/v1/systems/test-system")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        // Test API client
        let client = create_test_client(&server);
        let response = client.get_system("test-system").await.unwrap();

        // Verify response
        assert_eq!(response.agent_count, 5);
        assert_eq!(response.running_agent_count, 3);
    }

    #[tokio::test]
    async fn test_start_system() {
        // Create a mock server
        let mut server = mockito::Server::new_async().await;

        let mock_response = r#"
        {
            "status": "ok"
        }
        "#;

        // Setup mock
        let _m = server
            .mock("POST", "/api/v1/systems/test-system/start")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        // Test API client
        let client = create_test_client(&server);
        let response = client.start_system("test-system", None).await.unwrap();

        // Verify response
        assert_eq!(response["status"], "ok");
    }

    #[tokio::test]
    async fn test_stop_system() {
        // Create a mock server
        let mut server = mockito::Server::new_async().await;

        let mock_response = r#"
        {
            "status": "ok"
        }
        "#;

        // Setup mock
        let _m = server
            .mock("POST", "/api/v1/systems/test-system/stop")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        // Test API client
        let client = create_test_client(&server);
        let response = client.stop_system("test-system").await.unwrap();

        // Verify response
        assert_eq!(response["status"], "ok");
    }

    #[tokio::test]
    async fn test_delete_system() {
        // Create a mock server
        let mut server = mockito::Server::new_async().await;

        let mock_response = r#"
        {
            "status": "ok"
        }
        "#;

        // Setup mock
        let _m = server
            .mock("DELETE", "/api/v1/systems/test-system")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        // Test API client
        let client = create_test_client(&server);
        let response = client.delete_system("test-system").await.unwrap();

        // Verify response
        assert_eq!(response["status"], "ok");
    }

    #[tokio::test]
    async fn test_api_error_handling() {
        // Create a mock server
        let mut server = mockito::Server::new_async().await;

        let error_response = r#"
        {
            "error": "System not found",
            "code": "NOT_FOUND"
        }
        "#;

        // Setup mock
        let _m = server
            .mock("GET", "/api/v1/systems/non-existent")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(error_response)
            .create_async()
            .await;

        // Test API client
        let client = create_test_client(&server);
        let result = client.get_system("non-existent").await;

        // Verify error handling
        assert!(result.is_err());
        if let Err(ApiError::Api { status, message }) = result {
            assert_eq!(status, StatusCode::NOT_FOUND);
            assert!(message.contains("System not found"));
        } else {
            panic!("Expected ApiError::Api, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        // Create a mock server
        let mut server = mockito::Server::new_async().await;

        let mock_response = r#"
        {
            "status": "ok",
            "version": "1.0.0"
        }
        "#;

        // Setup mock
        let _m = server
            .mock("GET", "/health")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        // Test API client
        let client = create_test_client(&server);
        let response = client.health_check().await.unwrap();

        // Verify response
        assert_eq!(response["status"], "ok");
        assert_eq!(response["version"], "1.0.0");
    }

    #[test]
    fn test_to_json() {
        // Create a test structure
        #[derive(Serialize)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        // Convert to JSON
        let json = ApiClient::to_json(&data).unwrap();

        // Verify
        assert_eq!(json["name"], "test");
        assert_eq!(json["value"], 42);
    }
}
