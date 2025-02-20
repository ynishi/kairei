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
        unimplemented!()
    }
    fn capability(&self) -> crate::provider::plugin::CapabilityType {
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
    let visitor = PluginTypeVisitor::new();

    // Test valid types
    assert!(visitor
        .validate_value_type(&Value::String("test".to_string()))
        .is_ok());
    assert!(visitor.validate_value_type(&Value::Integer(42)).is_ok());
    assert!(visitor.validate_value_type(&Value::Float(3.14)).is_ok());
    assert!(visitor.validate_value_type(&Value::Boolean(true)).is_ok());

    // Test valid nested types
    assert!(visitor
        .validate_value_type(&Value::List(vec![
            Value::String("test".to_string()),
            Value::Integer(42),
        ]))
        .is_ok());

    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::String("value".to_string()));
    assert!(visitor.validate_value_type(&Value::Map(map)).is_ok());

    // Test invalid type
    let custom_value = Value::Custom("invalid".to_string());
    assert!(visitor.validate_value_type(&custom_value).is_err());
}
