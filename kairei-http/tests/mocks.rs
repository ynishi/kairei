//! Mock implementations for testing
//!
//! This module provides mock implementations of the kairei-core API traits for testing.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use kairei_core::{
    agent_registry::AgentError,
    api::{
        agent::AgentApi,
        event::EventApi,
        models::{
            AgentCreationRequest, AgentCreationResponse, AgentStatusDto, SystemStatusDto,
            ValidationResult,
        },
        state::StateApi,
        system::SystemApi,
    },
    eval::expression,
    event::event_bus::{Event, EventReceiver, Value},
    event::event_registry::EventType,
    system::{SystemError, SystemResult},
};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use kairei_http::integration::KaireiSystem;

/// Mock System API implementation
pub struct MockSystemApi;

#[async_trait]
impl SystemApi for MockSystemApi {
    async fn get_system_status(&self) -> SystemResult<SystemStatusDto> {
        Ok(SystemStatusDto {
            status: "running".to_string(),
            uptime_seconds: 3600,
            version: "0.1.0".to_string(),
            started_at: "2025-03-03T12:00:00Z".to_string(),
            agent_count: 5,
            running_agent_count: 3,
            event_queue_size: 10,
            event_subscribers: 2,
        })
    }

    async fn start(&self) -> SystemResult<()> {
        Ok(())
    }

    async fn shutdown(&self) -> SystemResult<()> {
        Ok(())
    }

    async fn emergency_shutdown(&self) -> SystemResult<()> {
        Ok(())
    }
}

/// Mock Agent API implementation
pub struct MockAgentApi;

#[async_trait]
impl AgentApi for MockAgentApi {
    async fn register_agent_from_dsl(
        &self,
        request: AgentCreationRequest,
    ) -> Result<AgentCreationResponse, SystemError> {
        Ok(AgentCreationResponse {
            agent_id: request.name.clone(),
            status: if request.options.auto_start {
                "running".to_string()
            } else {
                "created".to_string()
            },
            validation_result: ValidationResult {
                success: true,
                warnings: vec![],
            },
        })
    }

    async fn start_agent(&self, _agent_name: &str) -> SystemResult<()> {
        Ok(())
    }

    async fn stop_agent(&self, _agent_name: &str) -> SystemResult<()> {
        Ok(())
    }

    async fn restart_agent(&self, _agent_name: &str) -> SystemResult<()> {
        Ok(())
    }

    async fn get_agent_status(&self, agent_name: &str) -> Result<AgentStatusDto, SystemError> {
        if agent_name.contains("not-found") {
            return Err(SystemError::Agent(AgentError::AgentNotFound {
                agent_id: agent_name.to_string(),
            }));
        }

        Ok(AgentStatusDto {
            name: "WeatherAgent".to_string(),
            state: "running".to_string(),
            last_updated: "2025-03-03T12:00:00Z".to_string(),
        })
    }

    async fn scale_up(
        &self,
        _name: &str,
        _count: usize,
        _metadata: HashMap<String, Value>,
    ) -> Result<Vec<String>, SystemError> {
        Ok(vec!["agent-001".to_string(), "agent-002".to_string()])
    }

    async fn scale_down(
        &self,
        _name: &str,
        _count: usize,
        _metadata: HashMap<String, Value>,
    ) -> SystemResult<()> {
        Ok(())
    }
}

/// Mock Event API implementation
pub struct MockEventApi;

#[async_trait]
impl EventApi for MockEventApi {
    async fn send_event(&self, _event: Event) -> SystemResult<()> {
        Ok(())
    }

    async fn send_request(&self, _event: Event) -> SystemResult<Value> {
        Ok(Value::Map(HashMap::new()))
    }

    async fn subscribe_events(&self, _event_types: Vec<EventType>) -> SystemResult<EventReceiver> {
        unimplemented!("Not needed for tests")
    }

    async fn send_typed_event(
        &self,
        _event_type: String,
        _payload: JsonValue,
        _target_agents: Vec<String>,
    ) -> SystemResult<String> {
        Ok(format!(
            "evt-{}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        ))
    }

    async fn send_agent_request(
        &self,
        agent_id: &str,
        request_type: String,
        parameters: JsonValue,
    ) -> SystemResult<JsonValue> {
        if agent_id.contains("not-found") {
            return Err(SystemError::Agent(AgentError::AgentNotFound {
                agent_id: agent_id.to_string(),
            }));
        }

        // Mock response for GetWeather request
        if request_type == "GetWeather" {
            let location = parameters
                .get("location")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            Ok(serde_json::json!({
                "temperature": 25.5,
                "conditions": "Sunny",
                "humidity": 60,
                "location": location
            }))
        } else {
            Ok(serde_json::json!({
                "message": "Request processed successfully"
            }))
        }
    }
}

/// Mock State API implementation
pub struct MockStateApi;

#[async_trait]
impl StateApi for MockStateApi {
    async fn get_agent_state(
        &self,
        _agent_name: &str,
        _key: &str,
    ) -> Result<expression::Value, SystemError> {
        Ok(expression::Value::String("test-value".to_string()))
    }
}

/// Create a mock KaireiSystem for testing
pub fn create_mock_kairei_system() -> Arc<KaireiSystem> {
    Arc::new(KaireiSystem {
        system_api: Arc::new(MockSystemApi),
        agent_api: Arc::new(MockAgentApi),
        event_api: Arc::new(MockEventApi),
        state_api: Arc::new(MockStateApi),
    })
}
