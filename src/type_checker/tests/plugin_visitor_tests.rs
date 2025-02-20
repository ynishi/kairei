use super::*;
use crate::{
    config::{PluginConfig, ProviderConfig},
    eval::expression::Value,
    provider::{
        capability,
        llm::LLMResponse,
        plugin::{self, ProviderPlugin},
        provider::Section,
        request::{ProviderRequest, ProviderResponse, RequestInput, ExecutionState, ResponseMetadata},
        types::ProviderResult,
    },
    type_checker::visitor::PluginTypeVisitor,
};
use std::collections::HashMap;
use async_trait::async_trait;

// Mock implementation for testing
struct MockPlugin;
#[async_trait]
impl ProviderPlugin for MockPlugin {
    fn capability(&self) -> capability::CapabilityType {
        capability::CapabilityType::Generate
    }
    fn priority(&self) -> i32 {
        0
    }
    async fn generate_section<'a>(&self, _context: &PluginContext<'a>) -> ProviderResult<Section> {
        unimplemented!()
    }
    async fn process_response<'a>(&self, _context: &PluginContext<'a>, _response: &LLMResponse) -> ProviderResult<()> {
        unimplemented!()
    }
}

#[test]
fn test_plugin_request_validation() {
    let mut ctx = TypeContext::new();
    let visitor = PluginTypeVisitor::new();

    // Test valid request
    let valid_request = ProviderRequest {
        input: RequestInput {
            query: Value::String("test query".to_string()),
            parameters: HashMap::new(),
        },
        config: ProviderConfig::default(),
        state: ExecutionState::default(),
    };
    assert!(visitor
        .validate_plugin_request(&valid_request, &MockPlugin {}, &mut ctx)
        .is_ok());

    // Test invalid query type
    let invalid_request = ProviderRequest {
        input: RequestInput {
            query: Value::List(vec![]), // Unsupported type
            parameters: HashMap::new(),
        },
        config: ProviderConfig::default(),
        state: ExecutionState::default(),
    };
    assert!(visitor
        .validate_plugin_request(&invalid_request, &MockPlugin {}, &mut ctx)
        .is_err());
}

#[test]
fn test_plugin_response_validation() {
    let mut ctx = TypeContext::new();
    let visitor = PluginTypeVisitor::new();

    let response = ProviderResponse {
        output: "test output".to_string(),
        metadata: ResponseMetadata::default(),
    };
    assert!(visitor
        .validate_plugin_response(&response, &mut ctx)
        .is_ok());
}

#[test]
fn test_plugin_config_validation() {
    let mut ctx = TypeContext::new();
    let visitor = PluginTypeVisitor::new();

    let config = PluginConfig::Basic {
        name: "test".to_string(),
        settings: HashMap::new(),
    };
    assert!(visitor.validate_plugin_config(&config, &mut ctx).is_ok());
}

#[test]
fn test_value_type_validation() {
    let mut ctx = TypeContext::new();
    let visitor = PluginTypeVisitor::new();

    // Test valid types through request validation
    let valid_types = vec![
        Value::String("test".to_string()),
        Value::Integer(42),
        Value::Float(3.14),
        Value::Boolean(true),
    ];

    for value in valid_types {
        let request = ProviderRequest {
            input: RequestInput {
                query: value,
                parameters: HashMap::new(),
            },
            config: ProviderConfig::default(),
            state: ExecutionState::default(),
        };
        assert!(visitor
            .validate_plugin_request(&request, &MockPlugin {}, &mut ctx)
            .is_ok());
    }

    // Test valid nested types through request validation
    let mut nested_params = HashMap::new();
    nested_params.insert(
        "list".to_string(),
        Value::List(vec![Value::String("test".to_string()), Value::Integer(42)]),
    );
    nested_params.insert("map".to_string(), {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::String("value".to_string()));
        Value::Map(map)
    });

    let nested_request = ProviderRequest {
        input: RequestInput {
            query: Value::String("test".to_string()),
            parameters: nested_params,
        },
        config: ProviderConfig::default(),
        state: ExecutionState::default(),
    };
    assert!(visitor
        .validate_plugin_request(&nested_request, &MockPlugin {}, &mut ctx)
        .is_ok());

    // Test invalid type through request validation
    let invalid_request = ProviderRequest {
        input: RequestInput {
            query: Value::Null,
            parameters: HashMap::new(),
        },
        config: ProviderConfig::default(),
        state: ExecutionState::default(),
    };
    assert!(visitor
        .validate_plugin_request(&invalid_request, &MockPlugin {}, &mut ctx)
        .is_err());
}
