use super::*;
use crate::{
    ast::{self, Expression, Literal, TypeInfo},
    StateAccessPath,
};

#[test]
fn test_literal_expressions() {
    let mut ctx = TypeContext::new();
    let visitor = DefaultTypeVisitor;

    // Test integer literal
    let int_expr = Expression::Literal(Literal::Integer(42));
    assert!(visitor.visit_expression(&int_expr, &mut ctx).is_ok());
    assert_eq!(
        visitor.infer_type(&int_expr, &mut ctx).unwrap(),
        TypeInfo::Simple("Int".to_string())
    );

    // Test string literal
    let str_expr = Expression::Literal(Literal::String("test".to_string()));
    assert!(visitor.visit_expression(&str_expr, &mut ctx).is_ok());
    assert_eq!(
        visitor.infer_type(&str_expr, &mut ctx).unwrap(),
        TypeInfo::Simple("String".to_string())
    );

    // Test float literal
    let float_expr = Expression::Literal(Literal::Float(3.14));
    assert!(visitor.visit_expression(&float_expr, &mut ctx).is_ok());
    assert_eq!(
        visitor.infer_type(&float_expr, &mut ctx).unwrap(),
        TypeInfo::Simple("Float".to_string())
    );

    // Test boolean literal
    let bool_expr = Expression::Literal(Literal::Boolean(true));
    assert!(visitor.visit_expression(&bool_expr, &mut ctx).is_ok());
    assert_eq!(
        visitor.infer_type(&bool_expr, &mut ctx).unwrap(),
        TypeInfo::Simple("Boolean".to_string())
    );
}

#[test]
fn test_variable_expressions() {
    let mut ctx = TypeContext::new();
    let visitor = DefaultTypeVisitor;

    // Add variable to scope
    ctx.scope.insert_type(
        "test_var".to_string(),
        TypeInfo::Simple("String".to_string()),
    );

    // Test variable access
    let var_expr = Expression::Variable("test_var".to_string());
    assert!(visitor.visit_expression(&var_expr, &mut ctx).is_ok());
    assert_eq!(
        visitor.infer_type(&var_expr, &mut ctx).unwrap(),
        TypeInfo::Simple("String".to_string())
    );

    // Test undefined variable
    let undef_expr = Expression::Variable("undefined".to_string());
    assert!(visitor.visit_expression(&undef_expr, &mut ctx).is_err());
}

#[test]
fn test_state_access_expressions() {
    let mut ctx = TypeContext::new();
    let visitor = DefaultTypeVisitor;

    // Add state variable to scope
    ctx.scope.insert_type(
        "state.counter".to_string(),
        TypeInfo::Simple("Int".to_string()),
    );

    // Test state access
    let state_expr = Expression::StateAccess(StateAccessPath(vec![
        "state".to_string(),
        "counter".to_string(),
    ]));
    assert!(visitor.visit_expression(&state_expr, &mut ctx).is_ok());
    assert_eq!(
        visitor.infer_type(&state_expr, &mut ctx).unwrap(),
        TypeInfo::Simple("Int".to_string())
    );

    // Test invalid state access
    let invalid_expr = Expression::StateAccess(StateAccessPath(vec![
        "state".to_string(),
        "invalid".to_string(),
    ]));
    assert!(visitor.visit_expression(&invalid_expr, &mut ctx).is_err());
}

#[test]
fn test_think_block_expressions() {
    let mut ctx = TypeContext::new();
    ctx.scope.insert_type(
        "location".to_string(),
        TypeInfo::Simple("String".to_string()),
    );

    let visitor = DefaultTypeVisitor;

    let valid_think_expr = ast::Expression::Think {
        args: vec![
            ast::Argument::Positional(ast::Expression::Literal(ast::Literal::String(
                "Find suitable hotels matching criteria".to_string(),
            ))),
            ast::Argument::Positional(ast::Expression::Variable("location".to_string())),
        ],
        with_block: None,
    };
    println!(
        "{:?}",
        visitor.visit_expression(&valid_think_expr, &mut ctx)
    );
    assert!(visitor
        .visit_expression(&valid_think_expr, &mut ctx)
        .is_ok());
}
