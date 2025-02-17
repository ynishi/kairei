use super::*;
use crate::ast::{MicroAgentDef, Root, StateDef, TypeInfo};
use std::collections::HashMap;

#[test]
fn test_type_checker_initialization() {
    let checker = DefaultTypeChecker::new();
    assert!(checker.collect_errors().is_empty());
}

#[test]
fn test_type_checker_error_collection() {
    let mut checker = DefaultTypeChecker::new();
    let mut root = Root::default();
    
    // Add an invalid micro agent to trigger errors
    let invalid_agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: None,
        answer: None,
        observe: None,
        react: None,
    };
    root.micro_agent_defs.push(invalid_agent);
    
    let result = checker.check_types(&mut root);
    assert!(result.is_ok()); // Should not panic
    assert!(!checker.collect_errors().is_empty()); // Should have collected errors
}

#[test]
fn test_type_checker_with_valid_state() {
    let mut checker = DefaultTypeChecker::new();
    let mut root = Root::default();
    
    // Create a valid micro agent with state
    let mut state_vars = HashMap::new();
    state_vars.insert(
        "counter".to_string(),
        StateDef {
            type_info: TypeInfo::Simple("Int".to_string()),
            initial_value: None,
        },
    );
    
    let valid_agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: Some(state_vars),
        answer: None,
        observe: None,
        react: None,
    };
    
    root.micro_agent_defs.push(valid_agent);
    
    let result = checker.check_types(&mut root);
    assert!(result.is_ok());
    assert!(checker.collect_errors().is_empty());
}

#[test]
fn test_type_context() {
    let mut context = TypeContext::new();
    assert!(!context.has_errors());
    
    context.add_error(TypeCheckError::UndefinedType("Test".to_string()));
    assert!(context.has_errors());
    
    let errors = context.take_errors();
    assert_eq!(errors.len(), 1);
    assert!(!context.has_errors());
    
    context.clear();
    assert!(context.take_errors().is_empty());
}
