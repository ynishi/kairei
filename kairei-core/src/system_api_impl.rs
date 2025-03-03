//! Implementation of API traits for the System struct
//!
//! This module implements the API traits defined in the api module for the System struct.

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
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
    system::{System, SystemError, SystemResult},
};

#[async_trait]
impl SystemApi for System {
    async fn get_system_status(&self) -> SystemResult<SystemStatusDto> {
        let status = self.get_system_status().await?;
        Ok(SystemStatusDto::from(status))
    }

    async fn start(&self) -> SystemResult<()> {
        self.start().await
    }

    async fn shutdown(&self) -> SystemResult<()> {
        self.shutdown().await
    }

    async fn emergency_shutdown(&self) -> SystemResult<()> {
        self.emergency_shutdown().await
    }
}

#[async_trait]
impl AgentApi for System {
    async fn register_agent_from_dsl(
        &self,
        request: AgentCreationRequest,
    ) -> Result<AgentCreationResponse, SystemError> {
        // Parse DSL to AST
        let ast_registry = self.ast_registry().read().await;
        let _root_ast = match ast_registry.create_ast_from_dsl(&request.dsl_code).await {
            Ok(ast) => ast,
            Err(err) => {
                return Ok(AgentCreationResponse {
                    agent_id: request.name.clone(),
                    status: "failed".to_string(),
                    validation_result: ValidationResult {
                        success: false,
                        warnings: vec![format!("AST creation failed: {}", err)],
                    },
                });
            }
        };

        // Extract the agent definition from the root AST
        // This is a simplification - in a real implementation, we would need to
        // properly extract the MicroAgentDef from the Root AST
        drop(ast_registry);

        // For now, we'll just use the name as the agent ID
        // In a real implementation, we would extract the MicroAgentDef from the Root AST
        // and register it properly

        // TODO: Implement proper agent registration from AST
        // This is a placeholder for the actual implementation
        // if let Err(err) = self.register_agent_ast(&request.name, &agent_def).await {
        //     return Ok(AgentCreationResponse {
        //         agent_id: request.name.clone(),
        //         status: "failed".to_string(),
        //         validation_result: ValidationResult {
        //             success: false,
        //             warnings: vec![format!("AST registration failed: {}", err)],
        //         },
        //     });
        // }

        // Register the agent
        // TODO: Implement proper agent registration
        // This is a placeholder for the actual implementation
        // if let Err(err) = self.register_agent(&request.name).await {
        //     return Ok(AgentCreationResponse {
        //         agent_id: request.name.clone(),
        //         status: "failed".to_string(),
        //         validation_result: ValidationResult {
        //             success: false,
        //             warnings: vec![format!("Agent registration failed: {}", err)],
        //         },
        //     });
        // }

        // Start the agent if auto_start is true
        let mut status = "created".to_string();
        if request.options.auto_start {
            // TODO: Implement proper agent starting
            // This is a placeholder for the actual implementation
            // if let Err(err) = self.start_agent(&request.name).await {
            //     return Ok(AgentCreationResponse {
            //         agent_id: request.name.clone(),
            //         status: "created_but_start_failed".to_string(),
            //         validation_result: ValidationResult {
            //             success: true,
            //             warnings: vec![format!("Agent created but failed to start: {}", err)],
            //         },
            //     });
            // }
            status = "running".to_string();
        }

        Ok(AgentCreationResponse {
            agent_id: request.name.clone(),
            status,
            validation_result: ValidationResult {
                success: true,
                warnings: vec![],
            },
        })
    }

    async fn start_agent(&self, agent_name: &str) -> SystemResult<()> {
        self.start_agent(agent_name).await
    }

    async fn stop_agent(&self, agent_name: &str) -> SystemResult<()> {
        self.stop_agent(agent_name).await
    }

    async fn restart_agent(&self, agent_name: &str) -> SystemResult<()> {
        self.restart_agent(agent_name).await
    }

    async fn get_agent_status(&self, agent_name: &str) -> Result<AgentStatusDto, SystemError> {
        let status = self.get_agent_status(agent_name).await?;
        Ok(AgentStatusDto::from(status))
    }

    async fn scale_up(
        &self,
        name: &str,
        count: usize,
        metadata: HashMap<String, Value>,
    ) -> Result<Vec<String>, SystemError> {
        self.scale_up(name, count, metadata).await
    }

    async fn scale_down(
        &self,
        name: &str,
        count: usize,
        metadata: HashMap<String, Value>,
    ) -> SystemResult<()> {
        self.scale_down(name, count, metadata).await
    }
}

#[async_trait]
impl EventApi for System {
    async fn send_event(&self, event: Event) -> SystemResult<()> {
        self.send_event(event).await
    }

    async fn send_request(&self, event: Event) -> SystemResult<Value> {
        self.send_request(event).await
    }

    async fn subscribe_events(&self, event_types: Vec<EventType>) -> SystemResult<EventReceiver> {
        self.subscribe_events(event_types).await
    }

    async fn send_typed_event(
        &self,
        event_type: String,
        payload: JsonValue,
        _target_agents: Vec<String>,
    ) -> SystemResult<String> {
        let event_id = Uuid::new_v4().to_string();

        // Create event type
        let event_type = EventType::Custom(event_type);

        // Create event
        let mut event = Event {
            event_type,
            parameters: HashMap::new(),
            // Default values for other fields
        };

        // Convert JSON payload to parameters
        if let JsonValue::Object(obj) = payload.clone() {
            for (k, v) in obj {
                match v {
                    JsonValue::String(s) => {
                        event.parameters.insert(k, Value::String(s));
                    }
                    JsonValue::Number(n) => {
                        if n.is_i64() {
                            event
                                .parameters
                                .insert(k, Value::Integer(n.as_i64().unwrap()));
                        } else if n.is_f64() {
                            event
                                .parameters
                                .insert(k, Value::Float(n.as_f64().unwrap()));
                        }
                    }
                    JsonValue::Bool(b) => {
                        event.parameters.insert(k, Value::Boolean(b));
                    }
                    _ => {
                        event
                            .parameters
                            .insert(k, Value::String(format!("{:?}", v)));
                    }
                }
            }
        } else {
            // If not an object, store as string
            event.parameters.insert(
                "raw_json".to_string(),
                Value::String(format!("{:?}", payload)),
            );
        }

        // Send the event
        self.send_event(event).await?;

        Ok(event_id)
    }

    async fn send_agent_request(
        &self,
        agent_id: &str,
        request_type: String,
        parameters: JsonValue,
    ) -> SystemResult<JsonValue> {
        // Create request event
        let request_id = Uuid::new_v4().to_string();
        let event_type = EventType::Request {
            requester: "http-api".to_string(),
            responder: agent_id.to_string(),
            request_id: request_id.clone(),
            request_type: request_type.clone(),
        };

        // Create event
        let mut event = Event {
            event_type,
            parameters: HashMap::new(),
            // Default values for other fields
        };

        // Add request type and ID to parameters
        event
            .parameters
            .insert("request_type".to_string(), Value::String(request_type));
        event
            .parameters
            .insert("request_id".to_string(), Value::String(request_id.clone()));

        // Convert JSON parameters to event parameters
        if let JsonValue::Object(obj) = parameters.clone() {
            for (k, v) in obj {
                match v {
                    JsonValue::String(s) => {
                        event.parameters.insert(k, Value::String(s));
                    }
                    JsonValue::Number(n) => {
                        if n.is_i64() {
                            event
                                .parameters
                                .insert(k, Value::Integer(n.as_i64().unwrap()));
                        } else if n.is_f64() {
                            event
                                .parameters
                                .insert(k, Value::Float(n.as_f64().unwrap()));
                        }
                    }
                    JsonValue::Bool(b) => {
                        event.parameters.insert(k, Value::Boolean(b));
                    }
                    _ => {
                        event
                            .parameters
                            .insert(k, Value::String(format!("{:?}", v)));
                    }
                }
            }
        }

        // Send the request and get response
        let response = self.send_request(event).await?;

        // Convert Value to JSON
        let json_response = match response {
            Value::Map(map) => {
                let mut json_map = serde_json::Map::new();
                for (k, v) in map {
                    let json_value = match v {
                        Value::String(s) => JsonValue::String(s),
                        Value::Integer(i) => JsonValue::Number(i.into()),
                        Value::Float(f) => {
                            if let Some(num) = serde_json::Number::from_f64(f) {
                                JsonValue::Number(num)
                            } else {
                                JsonValue::Null
                            }
                        }
                        Value::Boolean(b) => JsonValue::Bool(b),
                        Value::Map(nested_map) => {
                            // Simple conversion for nested maps
                            let mut nested_json = serde_json::Map::new();
                            for (nk, nv) in nested_map {
                                nested_json.insert(nk, JsonValue::String(format!("{:?}", nv)));
                            }
                            JsonValue::Object(nested_json)
                        }
                        _ => JsonValue::String(format!("{:?}", v)),
                    };
                    json_map.insert(k, json_value);
                }
                JsonValue::Object(json_map)
            }
            Value::String(s) => JsonValue::String(s),
            Value::Integer(i) => JsonValue::Number(i.into()),
            Value::Float(f) => {
                if let Some(num) = serde_json::Number::from_f64(f) {
                    JsonValue::Number(num)
                } else {
                    JsonValue::Null
                }
            }
            Value::Boolean(b) => JsonValue::Bool(b),
            _ => JsonValue::String(format!("{:?}", response)),
        };

        Ok(json_response)
    }
}

#[async_trait]
impl StateApi for System {
    async fn get_agent_state(
        &self,
        agent_name: &str,
        key: &str,
    ) -> Result<expression::Value, SystemError> {
        self.get_agent_state(agent_name, key).await
    }
}
