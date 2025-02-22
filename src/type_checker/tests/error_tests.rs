use crate::{
    ast::{BinaryOperator, Expression, Literal, TypeInfo},
    type_checker::{
        error::Location,
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
    assert!(matches!(
        result,
        Err(TypeCheckError::InvalidArgumentType { .. })
    ));

    Ok(())
}

#[test]
fn test_undefined_variable() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    let expr = Expression::Variable("undefined_var".to_string());
    let result = visitor.visit_expression(&expr, &mut ctx);
    assert!(matches!(
        result,
        Err(TypeCheckError::UndefinedVariable { .. })
    ));

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
    assert!(matches!(
        result,
        Err(TypeCheckError::UndefinedFunction { name: _, meta: _ })
    ));

    Ok(())
}

#[test]
fn test_undefined_function_with_meta() {
    let location = Location {
        line: 1,
        column: 1,
        file: "test.rs".to_string(),
    };
    let error = TypeCheckError::undefined_function("test_func".to_string(), location.clone());

    if let TypeCheckError::UndefinedFunction { meta, name } = error {
        assert_eq!(meta.location, location);
        assert_eq!(name, "test_func");
        assert!(meta.help.contains("Function 'test_func' is not defined"));
        assert!(meta.suggestion.contains("Check function name for typos"));
    } else {
        panic!("Expected UndefinedFunction error");
    }
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

#[test]
fn test_undefined_type_with_meta() {
    let location = Location {
        line: 1,
        column: 1,
        file: "test.rs".to_string(),
    };
    let error = TypeCheckError::undefined_type("MyType".to_string(), location.clone());

    if let TypeCheckError::UndefinedType { meta, name } = error {
        assert_eq!(meta.location, location);
        assert_eq!(name, "MyType");
        assert!(meta.help.contains("MyType"));
        assert!(meta.suggestion.contains("Check type name"));
    } else {
        panic!("Expected UndefinedType error");
    }
}

#[test]
fn test_undefined_variable_with_meta() {
    let location = Location {
        line: 1,
        column: 1,
        file: "test.rs".to_string(),
    };
    let error = TypeCheckError::undefined_variable("x".to_string(), location.clone());

    if let TypeCheckError::UndefinedVariable { meta, name } = error {
        assert_eq!(meta.location, location);
        assert_eq!(name, "x");
        assert!(meta.help.contains("x"));
        assert!(meta.suggestion.contains("Check variable name"));
    } else {
        panic!("Expected UndefinedVariable error");
    }
}

#[test]
fn test_invalid_state_variable_with_meta() {
    let location = Location {
        line: 1,
        column: 1,
        file: "test.rs".to_string(),
    };
    let error = TypeCheckError::invalid_state_variable(
        "Invalid state variable access".to_string(),
        location.clone(),
    );

    if let TypeCheckError::InvalidStateVariable { meta, message } = error {
        assert_eq!(meta.location, location);
        assert_eq!(message, "Invalid state variable access");
        assert!(meta
            .help
            .contains("Invalid state variable declaration or usage"));
        assert!(meta
            .suggestion
            .contains("Check that the state variable is properly declared"));
    } else {
        panic!("Expected InvalidStateVariable error");
    }
}

#[test]
fn test_invalid_type_arguments_with_meta() {
    let location = Location {
        line: 1,
        column: 1,
        file: "test.rs".to_string(),
    };
    let error = TypeCheckError::invalid_type_arguments(
        "Wrong number of type arguments".to_string(),
        location.clone(),
    );

    if let TypeCheckError::InvalidTypeArguments { meta, message } = error {
        assert_eq!(meta.location, location);
        assert_eq!(message, "Wrong number of type arguments");
        assert!(meta.help.contains("Invalid arguments provided"));
        assert!(meta.suggestion.contains("Check the type arguments"));
    } else {
        panic!("Expected InvalidTypeArguments error");
    }
}
