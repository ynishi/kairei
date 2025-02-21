use crate::{
    ast,
    type_checker::{TypeCheckResult, TypeChecker, TypeContext},
};

mod error_tests;
mod expression_tests;
mod handler_tests;
mod micro_agent_tests;
mod plugin_tests;
mod scope_tests;

#[test]
fn test_type_checker_initialization() {
    let mut checker = TypeChecker::new();
    assert!(checker.collect_errors().is_empty());
}

#[test]
fn test_type_checker_error_collection() {
    let mut checker = TypeChecker::new();
    let mut root = ast::Root::new(None, vec![]);

    // Add an invalid micro agent to trigger errors
    // Create an agent with invalid state type to trigger errors
    let mut variables = std::collections::HashMap::new();
    variables.insert(
        "invalid".to_string(),
        ast::StateVarDef {
            name: "invalid".to_string(),
            type_info: ast::TypeInfo::Simple("NonExistentType".to_string()),
            initial_value: None,
        },
    );

    let invalid_agent = ast::MicroAgentDef {
        name: "test_agent".to_string(),
        state: Some(ast::StateDef { variables }),
        answer: None,
        observe: None,
        react: None,
        lifecycle: Some(ast::LifecycleDef {
            on_init: None,
            on_destroy: None,
        }),
        policies: vec![ast::Policy {
            text: "default".to_string(),
            scope: ast::PolicyScope::Agent("test_agent".to_string()),
            internal_id: ast::PolicyId::new(),
        }],
    };
    root.micro_agent_defs.push(invalid_agent);

    let result = checker.check_types(&mut root);
    assert!(result.is_err()); // Should fail with undefined type error
}

#[test]
fn test_type_checker_with_valid_state() {
    let mut checker = TypeChecker::new();
    let mut root = ast::Root::new(None, vec![]);

    // Create a valid micro agent with state
    // Create a valid state with a built-in type
    let mut variables = std::collections::HashMap::new();
    variables.insert(
        "counter".to_string(),
        ast::StateVarDef {
            name: "counter".to_string(),
            type_info: ast::TypeInfo::Simple("Int".to_string()),
            initial_value: None,
        },
    );
    let state = ast::StateDef { variables };

    // Create and add the valid agent to root
    let valid_agent = ast::MicroAgentDef {
        name: "test_agent".to_string(),
        state: Some(state),
        answer: None,
        observe: None,
        react: None,
        lifecycle: Some(ast::LifecycleDef {
            on_init: None,
            on_destroy: None,
        }),
        policies: vec![ast::Policy {
            text: "default".to_string(),
            scope: ast::PolicyScope::Agent("test_agent".to_string()),
            internal_id: ast::PolicyId::new(),
        }],
    };

    // Add agent to root and check types
    root.micro_agent_defs.push(valid_agent);

    let result = checker.check_types(&mut root);
    assert!(result.is_ok());
    assert!(checker.collect_errors().is_empty());
}

#[test]
fn test_type_context() {
    let mut context = TypeContext::new();
    assert!(!context.has_errors());

    context.add_error(crate::type_checker::TypeCheckError::UndefinedType(
        "Test".to_string(),
    ));
    assert!(context.has_errors());

    let errors = context.take_errors();
    assert_eq!(errors.len(), 1);
    assert!(!context.has_errors());

    context.clear();
    assert!(context.take_errors().is_empty());
}
