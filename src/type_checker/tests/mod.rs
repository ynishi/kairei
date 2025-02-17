use super::*;
use crate::ast::{MicroAgentDef, Root};

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
