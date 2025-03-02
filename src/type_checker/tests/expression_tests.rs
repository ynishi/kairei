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

#[test]
fn test_variable_type_inference() -> TypeCheckResult<()> {
    use crate::ast::Statement;
    use crate::type_checker::visitor::default::DefaultVisitor;

    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Create an assignment statement
    let name = "inferred_var".to_string();
    let value = Expression::Literal(Literal::String("hello".to_string()));
    let stmt = Statement::Assignment {
        target: vec![Expression::Variable(name.clone())],
        value,
    };

    // Visit the statement to trigger type inference
    visitor.visit_statement(&stmt, &mut ctx)?;

    // Now the variable should have a type in the scope
    let var_type = ctx.scope.get_type(&name).unwrap();
    assert!(matches!(var_type, TypeInfo::Simple(s) if s == "String"));

    Ok(())
}

#[test]
fn test_function_call_type_inference() -> TypeCheckResult<()> {
    use crate::ast::Statement;
    use crate::type_checker::visitor::default::DefaultVisitor;

    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Register a test function
    ctx.scope.insert_type(
        "test_func".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("Int".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Create an assignment with a function call
    let name = "func_result".to_string();
    let value = Expression::FunctionCall {
        function: "test_func".to_string(),
        arguments: vec![Expression::Literal(Literal::Integer(42))],
    };
    let stmt = Statement::Assignment {
        target: vec![Expression::Variable(name.clone())],
        value,
    };

    // Visit the statement to trigger type inference
    visitor.visit_statement(&stmt, &mut ctx)?;

    // Now the variable should have a type in the scope
    let var_type = ctx.scope.get_type(&name).unwrap();
    assert!(matches!(var_type, TypeInfo::Result { .. }));

    Ok(())
}

#[test]
fn test_binary_op_type_inference() -> TypeCheckResult<()> {
    use crate::ast::Statement;
    use crate::type_checker::visitor::default::DefaultVisitor;

    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Create an assignment with a binary operation
    let name = "sum".to_string();
    let value = Expression::BinaryOp {
        left: Box::new(Expression::Literal(Literal::Integer(1))),
        right: Box::new(Expression::Literal(Literal::Integer(2))),
        op: BinaryOperator::Add,
    };
    let stmt = Statement::Assignment {
        target: vec![Expression::Variable(name.clone())],
        value,
    };

    // Visit the statement to trigger type inference
    visitor.visit_statement(&stmt, &mut ctx)?;

    // Now the variable should have a type in the scope
    let var_type = ctx.scope.get_type(&name).unwrap();
    assert!(matches!(var_type, TypeInfo::Simple(s) if s == "Int"));

    Ok(())
}
