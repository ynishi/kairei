use super::*;
use crate::ast::{LifecycleDef, MicroAgentDef, Policy, Root, StateDef, StateVarDef, TypeInfo};
use std::collections::HashMap;

#[test]
fn test_type_checker_initialization() {
    let checker = DefaultTypeChecker::new();
    assert!(checker.collect_errors().is_empty());
}

#[test]
fn test_type_checker_error_collection() {
    let mut checker = DefaultTypeChecker::new();
    let mut root = Root::new(None, vec![]);

    // Add an invalid micro agent to trigger errors
    // Create an agent with invalid state type to trigger errors
    let mut variables = HashMap::new();
    variables.insert(
        "invalid".to_string(),
        StateVarDef {
            name: "invalid".to_string(),
            type_info: TypeInfo::Simple("NonExistentType".to_string()),
            initial_value: None,
        },
    );
    
    let invalid_agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: Some(StateDef { variables }),
        answer: None,
        observe: None,
        react: None,
        lifecycle: None,
        policies: vec![],
    };
    root.micro_agent_defs.push(invalid_agent);

    let result = checker.check_types(&mut root);
    assert!(result.is_err()); // Should fail with undefined type error
}

#[test]
fn test_type_checker_with_valid_state() {
    let mut checker = DefaultTypeChecker::new();
    let mut root = Root::new(None, vec![]);

    // Create a valid micro agent with state
    // Register built-in types
    for builtin_type in &["String", "Int", "Float", "Boolean", "Duration"] {
        checker.context.scope.insert_type(
            builtin_type.to_string(),
            TypeInfo::Simple(builtin_type.to_string()),
        );
    }

    // Create a valid state with a built-in type
    let mut variables = HashMap::new();
    variables.insert(
        "counter".to_string(),
        StateVarDef {
            name: "counter".to_string(),
            type_info: TypeInfo::Simple("Int".to_string()),
            initial_value: None,
        },
    );
    let state = StateDef { variables };

    let valid_agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: Some(state),
        answer: None,
        observe: None,
        react: None,
        lifecycle: None,
        policies: vec![],
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
