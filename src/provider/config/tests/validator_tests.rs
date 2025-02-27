//! Tests for the provider configuration validation framework.

use crate::provider::config::{
    errors::{ErrorSeverity, ProviderConfigError, ValidationError},
    CollectingValidator, EvaluatorValidator, ProviderConfigValidator, TypeCheckerValidator,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_type_checker_validator() {
    let validator = TypeCheckerValidator::default();

    // Valid memory config
    let config: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "type": "memory",
        "ttl": 3600
    }))
    .unwrap();

    assert!(validator.validate(&config).is_ok());

    // Invalid memory config (missing type)
    let config: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "ttl": 3600
    }))
    .unwrap();

    assert!(validator.validate(&config).is_err());
}

#[test]
fn test_evaluator_validator() {
    let validator = EvaluatorValidator::default();

    // Valid memory config
    let config: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "type": "memory",
        "ttl": 3600,
        "capabilities": {
            "memory": true
        }
    }))
    .unwrap();

    assert!(validator.validate(&config).is_ok());

    // Invalid memory config (ttl = 0)
    let config: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "type": "memory",
        "ttl": 0,
        "capabilities": {
            "memory": true
        }
    }))
    .unwrap();

    assert!(validator.validate_provider_specific(&config).is_err());
}

#[test]
fn test_collecting_validator() {
    let validator = TypeCheckerValidator::default();

    // Config with multiple errors
    let config: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        // Missing type
        "chunk_size": "not a number", // Wrong type
        "max_tokens": 0 // Invalid value
    }))
    .unwrap();

    let collector = validator.validate_collecting(&config);

    assert!(collector.has_errors());
    assert!(!collector.errors.is_empty());
}

#[test]
fn test_validator_integration() {
    // Test that both validators can be used together
    let type_checker = TypeCheckerValidator::default();
    let evaluator = EvaluatorValidator::default();

    let config: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "type": "rag",
        "chunk_size": 512,
        "max_tokens": 1000,
        "capabilities": {
            "rag": true
        }
    }))
    .unwrap();

    // Both validators should pass
    assert!(type_checker.validate(&config).is_ok());
    assert!(evaluator.validate(&config).is_ok());

    // Test with invalid config
    let config: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "type": "rag",
        "chunk_size": 0, // Invalid
        "max_tokens": 1000,
        "capabilities": {
            "rag": true
        }
    }))
    .unwrap();

    // Type checker should pass but evaluator should fail
    assert!(type_checker.validate(&config).is_ok());
    assert!(evaluator.validate_provider_specific(&config).is_err());
}

#[test]
fn test_collecting_validator_with_warnings() {
    let validator = TypeCheckerValidator::default();

    // Config with warnings but no errors
    let config: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "type": "memory",
        "ttl": 3600,
        "legacy_mode": true // This should trigger a warning
    }))
    .unwrap();

    let collector = validator.validate_collecting(&config);

    assert!(!collector.has_errors());
    assert!(collector.has_warnings());
    assert!(!collector.warnings.is_empty());

    // Check that the warning is about the legacy_mode field
    let warning = &collector.warnings[0];
    match warning {
        ProviderConfigError::Validation(ValidationError::InvalidValue { context, .. }) => {
            assert_eq!(context.location.field, Some("legacy_mode".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Warning);
        }
        _ => panic!("Expected ValidationError::InvalidValue"),
    }
}

#[test]
fn test_evaluator_validator_warnings() {
    let validator = EvaluatorValidator::default();

    // Config with warnings but no errors
    let config: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "type": "memory",
        "ttl": 30, // This should trigger a warning (too low)
        "capabilities": {
            "memory": true
        }
    }))
    .unwrap();

    let collector = validator.validate_collecting(&config);

    assert!(!collector.has_errors());
    assert!(collector.has_warnings());
    assert!(!collector.warnings.is_empty());

    // Check that the warning is about the ttl field
    let warning = &collector.warnings[0];
    match warning {
        ProviderConfigError::Validation(ValidationError::InvalidValue { context, .. }) => {
            assert_eq!(context.location.field, Some("ttl".to_string()));
            assert_eq!(context.severity, ErrorSeverity::Warning);
        }
        _ => panic!("Expected ValidationError::InvalidValue"),
    }
}
