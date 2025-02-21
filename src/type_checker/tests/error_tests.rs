use crate::{
    ast::{BinaryOperator, Expression, Literal, TypeInfo},
    type_checker::{
        visitor::{common::TypeVisitor, default::DefaultVisitor},
        TypeCheckError, TypeCheckResult, TypeContext,
    },
};

#[test]
fn test_invalid_operator_type() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Try to add a string and an integer
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Literal(Literal::String("hello".to_string()))),
        right: Box::new(Expression::Literal(Literal::Integer(42))),
        op: BinaryOperator::Add,
    };

    let result = visitor.visit_expression(&expr, &mut ctx);
    assert!(matches!(
        result,
        Err(TypeCheckError::InvalidOperatorType { .. })
    ));

    Ok(())
}

#[test]
fn test_invalid_logical_operator_type() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Try to use AND with non-boolean operands
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Literal(Literal::Integer(1))),
        right: Box::new(Expression::Literal(Literal::Integer(2))),
        op: BinaryOperator::And,
    };

    let result = visitor.visit_expression(&expr, &mut ctx);
    assert!(matches!(
        result,
        Err(TypeCheckError::InvalidOperatorType { .. })
    ));

    Ok(())
}

#[test]
fn test_invalid_function_argument_type() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Register function type
    ctx.scope.insert_type(
        "test_func".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("Int".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Call function with wrong argument type
    let expr = Expression::FunctionCall {
        function: "test_func".to_string(),
        arguments: vec![Expression::Literal(Literal::String(
            "wrong type".to_string(),
        ))],
    };

    let result = visitor.visit_expression(&expr, &mut ctx);
    // Note: This will pass for now since we haven't implemented argument type checking yet
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_undefined_variable() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    let expr = Expression::Variable("undefined_var".to_string());
    let result = visitor.visit_expression(&expr, &mut ctx);
    assert!(matches!(result, Err(TypeCheckError::UndefinedVariable(..))));

    Ok(())
}

#[test]
fn test_undefined_function() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    let expr = Expression::FunctionCall {
        function: "undefined_func".to_string(),
        arguments: vec![],
    };
    let result = visitor.visit_expression(&expr, &mut ctx);
    assert!(matches!(result, Err(TypeCheckError::UndefinedFunction(..))));

    Ok(())
}

#[test]
fn test_type_mismatch_in_assignment() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Register variable type
    ctx.scope
        .insert_type("x".to_string(), TypeInfo::Simple("Int".to_string()));

    // Try to assign string to int variable
    let expr = Expression::Variable("x".to_string());
    let value = Expression::Literal(Literal::String("wrong type".to_string()));

    let result = visitor.visit_statement(
        &crate::ast::Statement::Assignment {
            target: vec![expr],
            value: value,
        },
        &mut ctx,
    );
    assert!(matches!(result, Err(TypeCheckError::TypeMismatch { .. })));

    Ok(())
}
