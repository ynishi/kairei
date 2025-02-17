use super::*;
use crate::ast::TypeInfo;

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
