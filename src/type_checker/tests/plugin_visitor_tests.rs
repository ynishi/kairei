use super::*;
use crate::{
    eval::expression::Value,
    provider::{
        plugin::ProviderPlugin,
        request::{ProviderRequest, ProviderResponse, RequestInput},
    },
    type_checker::visitor::plugin_visitor::PluginTypeVisitor,
};
use std::collections::HashMap;

// Mock implementation for testing
struct MockPlugin;
impl ProviderPlugin for MockPlugin {
    fn execute(&self, _request: &ProviderRequest) -> Result<ProviderResponse, String> {
        Ok(ProviderResponse {
            output: "test".to_string(),
        })
    }
    fn capability(&self) -> crate::provider::plugin::CapabilityType {
        crate::provider::plugin::CapabilityType::General
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
    };
    assert!(visitor
        .validate_plugin_response(&response, &mut ctx)
        .is_ok());
}

#[test]
fn test_plugin_config_validation() {
    let mut ctx = TypeContext::new();
    let visitor = PluginTypeVisitor::new();

    let config = crate::config::PluginConfig {
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
        };
        assert!(visitor
            .validate_plugin_request(&request, &MockPlugin {}, &mut ctx)
            .is_ok());
    }

    // Test valid nested types through request validation
    let mut nested_params = HashMap::new();
    nested_params.insert(
        "list".to_string(),
        Value::List(vec![
            Value::String("test".to_string()),
            Value::Integer(42),
        ]),
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
    };
    assert!(visitor
        .validate_plugin_request(&invalid_request, &MockPlugin {}, &mut ctx)
        .is_err());
}
