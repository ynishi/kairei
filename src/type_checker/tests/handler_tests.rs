use super::*;
use crate::ast::{Expression, HandlerBlock, HandlerDef, Literal, Parameter, Statement, TypeInfo};

#[test]
fn test_handler_with_parameters() {
    let mut ctx = TypeContext::new();
    ctx.scope
        .insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));

    let visitor = DefaultTypeVisitor;
    let handler = HandlerDef {
        event_name: "test_event".to_string(),
        parameters: vec![Parameter {
            name: "param1".to_string(),
            type_info: TypeInfo::Simple("String".to_string()),
        }],
        block: HandlerBlock { statements: vec![] },
    };
    assert!(visitor.visit_handler(&handler, &mut ctx).is_ok());
}

#[test]
fn test_handler_with_invalid_parameter_type() {
    let mut ctx = TypeContext::new();
    let visitor = DefaultTypeVisitor;

    let handler = HandlerDef {
        event_name: "test_event".to_string(),
        parameters: vec![Parameter {
            name: "param1".to_string(),
            type_info: TypeInfo::Simple("NonExistentType".to_string()),
        }],
        block: HandlerBlock { statements: vec![] },
    };
    assert!(visitor.visit_handler(&handler, &mut ctx).is_err());
}

#[test]
fn test_handler_with_statements() {
    let mut ctx = TypeContext::new();
    ctx.scope
        .insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));

    let visitor = DefaultTypeVisitor;
    let handler = HandlerDef {
        event_name: "test_event".to_string(),
        parameters: vec![],
        block: HandlerBlock {
            statements: vec![Statement::Expression(Expression::Literal(Literal::String(
                "test".to_string(),
            )))],
        },
    };
    assert!(visitor.visit_handler(&handler, &mut ctx).is_ok());
}

#[test]
fn test_handler_with_parameter_scope() {
    let mut ctx = TypeContext::new();
    ctx.scope
        .insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));

    let visitor = DefaultTypeVisitor;
    let handler = HandlerDef {
        event_name: "test_event".to_string(),
        parameters: vec![Parameter {
            name: "param1".to_string(),
            type_info: TypeInfo::Simple("String".to_string()),
        }],
        block: HandlerBlock {
            statements: vec![Statement::Expression(Expression::Variable(
                "param1".to_string(),
            ))],
        },
    };
    assert!(visitor.visit_handler(&handler, &mut ctx).is_ok());
}

#[test]
fn test_handler_with_result_parameter() {
    let mut ctx = TypeContext::new();
    ctx.scope
        .insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));

    let visitor = DefaultTypeVisitor;
    let handler = HandlerDef {
        event_name: "test_event".to_string(),
        parameters: vec![Parameter {
            name: "param1".to_string(),
            type_info: TypeInfo::Result {
                ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                err_type: Box::new(TypeInfo::Simple("String".to_string())),
            },
        }],
        block: HandlerBlock { statements: vec![] },
    };
    // Result types should be allowed in handler parameters
    assert!(visitor.visit_handler(&handler, &mut ctx).is_ok());
}
