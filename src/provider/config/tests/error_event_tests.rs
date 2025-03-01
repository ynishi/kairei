//! Tests for provider error event handling.

use crate::event::event_bus::{ErrorSeverity, EventBus, Value};
use crate::provider::config::errors::{ProviderConfigError, SchemaError, ValidationError};
use crate::provider::config::validator::{ErrorCollector, ProviderConfigValidator};
use std::collections::HashMap;
use std::sync::Arc;

struct TestValidator;

impl ProviderConfigValidator for TestValidator {
    fn validate_schema(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        Err(ProviderConfigError::Schema(SchemaError::missing_field(
            "test_field",
        )))
    }

    fn validate_provider_specific(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        Ok(())
    }

    fn validate_capabilities(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        Ok(())
    }

    fn validate_dependencies(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        Ok(())
    }

    fn validate_schema_warnings(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        vec![ProviderConfigError::Validation(
            ValidationError::invalid_value("warning_field", "This is a warning"),
        )]
    }
}

#[tokio::test]
async fn test_error_event_publishing() {
    // Create event bus
    let event_bus = Arc::new(EventBus::new(10));
    let (_, mut error_rx) = event_bus.subscribe();

    // Create error collector with event bus
    let mut collector = ErrorCollector::new_with_event_bus(event_bus.clone(), "test_provider");

    // Add an error
    let error = ProviderConfigError::Schema(SchemaError::missing_field("test_field"));
    collector.add_error(error);

    // Receive error event
    let error_event = error_rx.recv().await.unwrap();

    // Verify error event properties
    assert_eq!(error_event.error_type, "provider_config_schema_error");
    assert_eq!(error_event.severity, ErrorSeverity::Error);

    // Verify parameters
    let provider_id = error_event.parameters.get("provider_id").unwrap();
    if let Value::String(id) = provider_id {
        assert_eq!(id, "test_provider");
    } else {
        panic!("provider_id is not a string");
    }

    let field = error_event.parameters.get("field").unwrap();
    if let Value::String(field_name) = field {
        assert_eq!(field_name, "test_field");
    } else {
        panic!("field is not a string");
    }
}

#[tokio::test]
async fn test_warning_event_publishing() {
    // Create event bus
    let event_bus = Arc::new(EventBus::new(10));
    let (_, mut error_rx) = event_bus.subscribe();

    // Create error collector with event bus
    let mut collector = ErrorCollector::new_with_event_bus(event_bus.clone(), "test_provider");

    // Add a warning
    let warning = ProviderConfigError::Validation(ValidationError::invalid_value(
        "warning_field",
        "This is a warning",
    ));
    collector.add_warning(warning);

    // Receive warning event
    let warning_event = error_rx.recv().await.unwrap();

    // Verify warning event properties
    assert_eq!(warning_event.error_type, "provider_config_validation_error");
    assert_eq!(warning_event.severity, ErrorSeverity::Error);

    // Verify parameters
    let provider_id = warning_event.parameters.get("provider_id").unwrap();
    if let Value::String(id) = provider_id {
        assert_eq!(id, "test_provider");
    } else {
        panic!("provider_id is not a string");
    }

    let field = warning_event.parameters.get("field").unwrap();
    if let Value::String(field_name) = field {
        assert_eq!(field_name, "warning_field");
    } else {
        panic!("field is not a string");
    }

    let context = warning_event.parameters.get("context").unwrap();
    if let Value::String(ctx) = context {
        assert_eq!(ctx, "Warning during provider config validation");
    } else {
        panic!("context is not a string");
    }
}

#[tokio::test]
async fn test_collecting_validator_with_events() {
    // Create event bus
    let event_bus = Arc::new(EventBus::new(10));
    let (_, mut error_rx) = event_bus.subscribe();

    // Create validator
    let validator = TestValidator;

    // Create config
    let config = HashMap::new();

    // Create error collector with event bus
    let mut collector = ErrorCollector::new_with_event_bus(event_bus.clone(), "test_provider");

    // Validate schema
    if let Err(error) = validator.validate_schema(&config) {
        collector.add_error(error);
    }

    // Collect warnings
    for warning in validator.validate_schema_warnings(&config) {
        collector.add_warning(warning);
    }

    // Receive error event
    let error_event = error_rx.recv().await.unwrap();
    assert_eq!(error_event.error_type, "provider_config_schema_error");

    // Receive warning event
    let warning_event = error_rx.recv().await.unwrap();
    assert_eq!(warning_event.error_type, "provider_config_validation_error");
}
