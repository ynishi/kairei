use crate::{
    ast::{BinaryOperator, Expression, Literal, TypeInfo},
    type_checker::{visitor::common::TypeVisitor, TypeCheckResult, TypeChecker, TypeContext},
};

#[test]
fn test_literal_expressions() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    let expr = Expression::Literal(Literal::Integer(42));
    checker.visit_expression(&expr, &mut ctx)?;

    Ok(())
}

#[test]
fn test_binary_expressions() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Literal(Literal::Integer(1))),
        right: Box::new(Expression::Literal(Literal::Integer(2))),
        op: BinaryOperator::Add,
    };
    checker.visit_expression(&expr, &mut ctx)?;

    Ok(())
}

#[test]
fn test_variable_expressions() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    // Register a variable type in the context
    ctx.scope
        .insert_type("x".to_string(), TypeInfo::Simple("Int".to_string()));

    let expr = Expression::Variable("x".to_string());
    checker.visit_expression(&expr, &mut ctx)?;

    Ok(())
}

#[test]
fn test_function_call_expressions() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    // Register function type in the context
    ctx.scope.insert_type(
        "test_func".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("Int".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    let expr = Expression::FunctionCall {
        function: "test_func".to_string(),
        arguments: vec![Expression::Literal(Literal::Integer(42))],
    };
    checker.visit_expression(&expr, &mut ctx)?;

    Ok(())
}

#[test]
fn test_result_expressions() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    let ok_expr = Expression::Ok(Box::new(Expression::Literal(Literal::Integer(42))));
    checker.visit_expression(&ok_expr, &mut ctx)?;

    let err_expr = Expression::Err(Box::new(Expression::Literal(Literal::String(
        "error".to_string(),
    ))));
    checker.visit_expression(&err_expr, &mut ctx)?;

    Ok(())
}
