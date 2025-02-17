use super::*;
use crate::ast::{LifecycleDef, MicroAgentDef, Policy, PolicyId, PolicyScope, Root, StateDef, StateVarDef, TypeInfo};
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
        lifecycle: Some(LifecycleDef {
            on_init: None,
            on_destroy: None,
        }),
        policies: vec![Policy {
            text: "default".to_string(),
            scope: PolicyScope::Agent("test_agent".to_string()),
            internal_id: PolicyId::new(),
        }],
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

    // Register built-in types in the root scope
    checker.context.scope.enter_scope(); // Enter root scope
    for builtin_type in &["Int", "String", "Float", "Boolean", "Duration"] {
        checker.context.scope.insert_type(
            builtin_type.to_string(),
            TypeInfo::Simple(builtin_type.to_string()),
        );
    }
    // Don't exit scope since we want these types available for validation

    // Create and add the valid agent to root
    let valid_agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: Some(state),
        answer: None,
        observe: None,
        react: None,
        lifecycle: Some(LifecycleDef {
            on_init: None,
            on_destroy: None,
        }),
        policies: vec![Policy {
            text: "default".to_string(),
            scope: PolicyScope::Agent("test_agent".to_string()),
            internal_id: PolicyId::new(),
        }],
    };

    // Add agent to root and check types
    root.micro_agent_defs.push(valid_agent);
    
    // Print scope state before type checking
    println!("Scope contains Int type: {}", checker.context.scope.contains_type("Int"));
    println!("Current scope depth: {}", checker.context.scope.depth());
    
    // Print agent state for debugging
    let agent_state = valid_agent.state.clone();
    println!("Agent state: {:?}", agent_state);
    
    // Add agent to root and check types
    root.micro_agent_defs.push(valid_agent);
    
    let result = checker.check_types(&mut root);
    
    // If there are errors, print them for debugging
    if result.is_err() {
        println!("Type check errors: {:?}", checker.collect_errors());
        println!("Result error: {:?}", result);
    } else {
        println!("Type check succeeded");
    }
    
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
