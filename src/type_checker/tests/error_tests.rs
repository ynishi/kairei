use super::*;
use crate::{
    ast::TypeInfo,
    type_checker::{error::Location, TypeCheckError},
};

#[test]
fn test_type_check_error_display() {
    let location = Location {
        line: 1,
        column: 1,
        file: "test.kr".to_string(),
    };

    let error = TypeCheckError::TypeMismatch {
        expected: TypeInfo::Simple("String".to_string()),
        found: TypeInfo::Simple("Int".to_string()),
        location,
    };

    assert!(error.to_string().contains("Type mismatch"));
    assert!(error.to_string().contains("String"));
    assert!(error.to_string().contains("Int"));
}

#[test]
fn test_type_check_error_undefined_type() {
    let error = TypeCheckError::UndefinedType("CustomType".to_string());
    assert!(error.to_string().contains("Undefined type"));
    assert!(error.to_string().contains("CustomType"));
}

#[test]
fn test_type_check_result() {
    let result: TypeCheckResult<()> = Ok(());
    assert!(result.is_ok());

    let error_result: TypeCheckResult<()> = Err(TypeCheckError::UndefinedType("Test".to_string()));
    assert!(error_result.is_err());
}

#[test]
fn test_plugin_type_error() {
    let error = TypeCheckError::PluginTypeError {
        message: "Invalid plugin configuration".to_string(),
    };
    assert!(error.to_string().contains("Invalid plugin configuration"));
}

#[test]
fn test_invalid_state_variable() {
    let error = TypeCheckError::InvalidStateVariable {
        message: "Missing required field".to_string(),
    };
    assert!(error.to_string().contains("Missing required field"));
}

#[test]
fn test_invalid_handler_signature() {
    let error = TypeCheckError::InvalidHandlerSignature {
        message: "Invalid parameter type".to_string(),
    };
    assert!(error.to_string().contains("Invalid parameter type"));
}

#[test]
fn test_invalid_think_block() {
    let error = TypeCheckError::InvalidThinkBlock {
        message: "Invalid think block configuration".to_string(),
    };
    assert!(error
        .to_string()
        .contains("Invalid think block configuration"));
}

#[test]
fn test_location_display() {
    let location = Location {
        line: 42,
        column: 10,
        file: "test.kr".to_string(),
    };
    assert_eq!(location.to_string(), "test.kr:42:10");
}

#[test]
fn test_type_check_error_debug() {
    let error = TypeCheckError::TypeMismatch {
        expected: TypeInfo::Simple("String".to_string()),
        found: TypeInfo::Simple("Int".to_string()),
        location: Location {
            line: 1,
            column: 1,
            file: "test.kr".to_string(),
        },
    };
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("TypeMismatch"));
    assert!(debug_str.contains("String"));
    assert!(debug_str.contains("Int"));
}
