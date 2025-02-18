use super::*;
use crate::ast::{
    HandlerBlock, HandlerDef, LifecycleDef, MicroAgentDef, Parameter, Policy, PolicyId, PolicyScope,
    StateDef, StateVarDef, TypeInfo,
};
use std::collections::HashMap;

#[test]
fn test_micro_agent_basic() {
    let mut ctx = TypeContext::new();
    let visitor = DefaultTypeVisitor;
    let mut agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: None,
        answer: None,
        observe: None,
        react: None,
        lifecycle: None,
        policies: vec![],
    };
    assert!(visitor.visit_micro_agent(&mut agent, &mut ctx).is_ok());
}

#[test]
fn test_micro_agent_with_state() {
    let mut ctx = TypeContext::new();
    // Register built-in types
    ctx.scope.insert_type("Int".to_string(), TypeInfo::Simple("Int".to_string()));
    ctx.scope.insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));
    
    let visitor = DefaultTypeVisitor;
    
    let mut variables = HashMap::new();
    variables.insert(
        "counter".to_string(),
        StateVarDef {
            name: "counter".to_string(),
            type_info: TypeInfo::Simple("Int".to_string()),
            initial_value: None,
        },
    );
    
    let mut agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: Some(StateDef { variables }),
        answer: None,
        observe: None,
        react: None,
        lifecycle: None,
        policies: vec![],
    };
    
    assert!(visitor.visit_micro_agent(&mut agent, &mut ctx).is_ok());
}

#[test]
fn test_micro_agent_with_lifecycle() {
    let mut ctx = TypeContext::new();
    let visitor = DefaultTypeVisitor;
    
    let init_handler = HandlerBlock { statements: vec![] };
    let destroy_handler = HandlerBlock { statements: vec![] };
    
    let mut agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: None,
        answer: None,
        observe: None,
        react: None,
        lifecycle: Some(LifecycleDef {
            on_init: Some(init_handler),
            on_destroy: Some(destroy_handler),
        }),
        policies: vec![],
    };
    
    assert!(visitor.visit_micro_agent(&mut agent, &mut ctx).is_ok());
}

#[test]
fn test_micro_agent_with_invalid_state() {
    let mut ctx = TypeContext::new();
    let visitor = DefaultTypeVisitor;
    
    let mut variables = HashMap::new();
    variables.insert(
        "invalid".to_string(),
        StateVarDef {
            name: "invalid".to_string(),
            type_info: TypeInfo::Simple("NonExistentType".to_string()),
            initial_value: None,
        },
    );
    
    let mut agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: Some(StateDef { variables }),
        answer: None,
        observe: None,
        react: None,
        lifecycle: None,
        policies: vec![],
    };
    
    assert!(visitor.visit_micro_agent(&mut agent, &mut ctx).is_err());
}
