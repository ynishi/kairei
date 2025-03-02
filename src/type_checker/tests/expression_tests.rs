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

#[test]
fn test_function_call_with_variable_arguments() -> TypeCheckResult<()> {
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

    // First, create a variable with a value
    let var_name = "test_var".to_string();
    let var_value = Expression::Literal(Literal::Integer(42));
    let var_stmt = Statement::Assignment {
        target: vec![Expression::Variable(var_name.clone())],
        value: var_value,
    };

    // Visit the statement to create the variable
    visitor.visit_statement(&var_stmt, &mut ctx)?;

    // Now create a function call using the variable as an argument
    let func_call = Expression::FunctionCall {
        function: "test_func".to_string(),
        arguments: vec![Expression::Variable(var_name.clone())],
    };

    // Infer the type of the function call
    let result_type = visitor.infer_type(&func_call, &ctx)?;

    // Check that the result type is correct
    assert!(matches!(result_type, TypeInfo::Result { .. }));

    Ok(())
}

#[test]
fn test_function_call_with_nested_function_call() -> TypeCheckResult<()> {
    use crate::type_checker::visitor::default::DefaultVisitor;

    let visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Register two test functions
    ctx.scope.insert_type(
        "inner_func".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("Int".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Make outer_func accept a Result type as its argument
    ctx.scope.insert_type(
        "outer_func".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("Int".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Create a nested function call
    let inner_call = Expression::FunctionCall {
        function: "inner_func".to_string(),
        arguments: vec![Expression::Literal(Literal::Integer(42))],
    };

    let outer_call = Expression::FunctionCall {
        function: "outer_func".to_string(),
        arguments: vec![inner_call],
    };

    // Infer the type of the outer function call
    let result_type = visitor.infer_type(&outer_call, &ctx)?;

    // Check that the result type is correct
    assert!(matches!(result_type, TypeInfo::Result { .. }));

    Ok(())
}

#[test]
fn test_function_call_with_binary_op_argument() -> TypeCheckResult<()> {
    use crate::type_checker::visitor::default::DefaultVisitor;

    let visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Register a test function
    ctx.scope.insert_type(
        "test_func".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("Int".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Create a binary operation
    let binary_op = Expression::BinaryOp {
        left: Box::new(Expression::Literal(Literal::Integer(1))),
        right: Box::new(Expression::Literal(Literal::Integer(2))),
        op: BinaryOperator::Add,
    };

    // Create a function call with the binary operation as an argument
    let func_call = Expression::FunctionCall {
        function: "test_func".to_string(),
        arguments: vec![binary_op],
    };

    // Infer the type of the function call
    let result_type = visitor.infer_type(&func_call, &ctx)?;

    // Check that the result type is correct
    assert!(matches!(result_type, TypeInfo::Result { .. }));

    Ok(())
}

#[test]
fn test_function_call_with_complex_parameter_types() -> TypeCheckResult<()> {
    use crate::type_checker::visitor::default::DefaultVisitor;
    use std::collections::HashMap;

    let visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Register a test function with a complex parameter type (Map)
    // Function takes a Map<String, Int> parameter and returns a Result<String, Error>
    ctx.scope.insert_type(
        "map_func".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("String".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Register the parameter types for map_func
    ctx.scope.insert_type(
        "map_func.params".to_string(),
        TypeInfo::Array(Box::new(TypeInfo::Map(
            Box::new(TypeInfo::Simple("String".to_string())),
            Box::new(TypeInfo::Simple("Int".to_string())),
        ))),
    );

    // Create a map literal
    let mut map = HashMap::new();
    map.insert("key".to_string(), Literal::Integer(42));
    let map_literal = Expression::Literal(Literal::Map(map));

    // Create a function call with the map as an argument
    let func_call = Expression::FunctionCall {
        function: "map_func".to_string(),
        arguments: vec![map_literal],
    };

    // Infer the type of the function call
    let result_type = visitor.infer_type(&func_call, &ctx)?;

    // Check that the result type is correct
    assert!(matches!(result_type, TypeInfo::Result { .. }));

    Ok(())
}

#[test]
fn test_function_call_with_generic_return_type() -> TypeCheckResult<()> {
    use crate::type_checker::visitor::default::DefaultVisitor;

    let visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Register a test function with a generic return type
    ctx.scope.insert_type(
        "generic_func".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Array(Box::new(TypeInfo::Simple(
                "Int".to_string(),
            )))),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Create a function call with an argument to match the expected parameter count
    let func_call = Expression::FunctionCall {
        function: "generic_func".to_string(),
        arguments: vec![Expression::Literal(Literal::Integer(42))],
    };

    // Infer the type of the function call
    let result_type = visitor.infer_type(&func_call, &ctx)?;

    // Check that the result type is correct
    assert!(matches!(
        result_type,
        TypeInfo::Result {
            ok_type,
            ..
        } if matches!(*ok_type, TypeInfo::Array(..))
    ));

    Ok(())
}

#[test]
fn test_nested_ok_err_expressions() -> TypeCheckResult<()> {
    use crate::type_checker::visitor::default::DefaultVisitor;

    let visitor = DefaultVisitor::new();
    let ctx = TypeContext::new();

    // Test Ok(Ok(expr))
    let inner_ok = Expression::Ok(Box::new(Expression::Literal(Literal::Integer(42))));
    let nested_ok = Expression::Ok(Box::new(inner_ok));
    let result_type = visitor.infer_type(&nested_ok, &ctx)?;

    // The result should be a Result type with a Result as the ok_type
    assert!(matches!(
        result_type,
        TypeInfo::Result {
            ok_type,
            ..
        } if matches!(*ok_type, TypeInfo::Result { .. })
    ));

    // Test Ok(Err(expr))
    let inner_err = Expression::Err(Box::new(Expression::Literal(Literal::String(
        "error".to_string(),
    ))));
    let nested_ok_err = Expression::Ok(Box::new(inner_err));
    let result_type = visitor.infer_type(&nested_ok_err, &ctx)?;

    // The result should be a Result type with a Result as the ok_type
    assert!(matches!(
        result_type,
        TypeInfo::Result {
            ok_type,
            ..
        } if matches!(*ok_type, TypeInfo::Result { .. })
    ));

    // Test Err(Ok(expr))
    let inner_ok = Expression::Ok(Box::new(Expression::Literal(Literal::Integer(42))));
    let nested_err_ok = Expression::Err(Box::new(inner_ok));
    let result_type = visitor.infer_type(&nested_err_ok, &ctx)?;

    // The result should be a Result type with a Result as the err_type
    assert!(matches!(
        result_type,
        TypeInfo::Result {
            err_type,
            ..
        } if matches!(*err_type, TypeInfo::Result { .. })
    ));

    // Test Err(Err(expr))
    let inner_err = Expression::Err(Box::new(Expression::Literal(Literal::String(
        "inner error".to_string(),
    ))));
    let nested_err_err = Expression::Err(Box::new(inner_err));
    let result_type = visitor.infer_type(&nested_err_err, &ctx)?;

    // The result should be a Result type with a Result as the err_type
    assert!(matches!(
        result_type,
        TypeInfo::Result {
            err_type,
            ..
        } if matches!(*err_type, TypeInfo::Result { .. })
    ));

    // Test deeply nested Ok expressions: Ok(Ok(Ok(expr)))
    let inner_ok = Expression::Ok(Box::new(Expression::Literal(Literal::Integer(42))));
    let middle_ok = Expression::Ok(Box::new(inner_ok));
    let outer_ok = Expression::Ok(Box::new(middle_ok));
    let result_type = visitor.infer_type(&outer_ok, &ctx)?;

    // The result should be a Result type with a nested Result structure
    if let TypeInfo::Result { ok_type, .. } = &result_type {
        if let TypeInfo::Result {
            ok_type: inner_ok, ..
        } = &**ok_type
        {
            if let TypeInfo::Result { .. } = &**inner_ok {
                // Test passes - we have a Result with a Result with a Result
            } else {
                panic!("Expected a Result with a Result with a Result, but got a Result with a Result with something else");
            }
        } else {
            panic!("Expected a Result with a Result, but got a Result with something else");
        }
    } else {
        panic!("Expected a Result, but got something else");
    }

    Ok(())
}

#[test]
fn test_ok_err_assignment() -> TypeCheckResult<()> {
    use crate::ast::Statement;
    use crate::type_checker::visitor::default::DefaultVisitor;

    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Create an assignment with an Ok expression
    let name = "ok_var".to_string();
    let value = Expression::Ok(Box::new(Expression::Literal(Literal::Integer(42))));
    let stmt = Statement::Assignment {
        target: vec![Expression::Variable(name.clone())],
        value,
    };

    // Visit the statement to trigger type inference
    visitor.visit_statement(&stmt, &mut ctx)?;

    // Now the variable should have a Result type in the scope
    let var_type = ctx.scope.get_type(&name).unwrap();
    if let TypeInfo::Result { ok_type, err_type } = var_type {
        if let TypeInfo::Simple(s) = &*ok_type {
            assert_eq!(s, "Int");
        } else {
            panic!("Expected ok_type to be Int, but got something else");
        }
        if let TypeInfo::Simple(s) = &*err_type {
            assert_eq!(s, "Error");
        } else {
            panic!("Expected err_type to be Error, but got something else");
        }
    } else {
        panic!("Expected a Result type, but got something else");
    }

    // Create an assignment with an Err expression
    let name = "err_var".to_string();
    let value = Expression::Err(Box::new(Expression::Literal(Literal::String(
        "error".to_string(),
    ))));
    let stmt = Statement::Assignment {
        target: vec![Expression::Variable(name.clone())],
        value,
    };

    // Visit the statement to trigger type inference
    visitor.visit_statement(&stmt, &mut ctx)?;

    // Now the variable should have a Result type in the scope
    let var_type = ctx.scope.get_type(&name).unwrap();
    if let TypeInfo::Result { ok_type, err_type } = var_type {
        if let TypeInfo::Simple(s) = &*ok_type {
            assert_eq!(s, "Any");
        } else {
            panic!("Expected ok_type to be Any, but got something else");
        }
        if let TypeInfo::Simple(s) = &*err_type {
            assert_eq!(s, "String");
        } else {
            panic!("Expected err_type to be String, but got something else");
        }
    } else {
        panic!("Expected a Result type, but got something else");
    }

    // Create an assignment with a nested Ok expression
    let name = "nested_ok_var".to_string();
    let inner_ok = Expression::Ok(Box::new(Expression::Literal(Literal::Integer(42))));
    let value = Expression::Ok(Box::new(inner_ok));
    let stmt = Statement::Assignment {
        target: vec![Expression::Variable(name.clone())],
        value,
    };

    // Visit the statement to trigger type inference
    visitor.visit_statement(&stmt, &mut ctx)?;

    // Now the variable should have a nested Result type in the scope
    let var_type = ctx.scope.get_type(&name).unwrap();
    if let TypeInfo::Result { ok_type, .. } = var_type {
        if let TypeInfo::Result { .. } = &*ok_type {
            // Test passes - we have a Result with a Result
        } else {
            panic!("Expected a Result with a Result, but got a Result with something else");
        }
    } else {
        panic!("Expected a Result, but got something else");
    }

    Ok(())
}

#[test]
fn test_ok_err_type_mismatch() -> TypeCheckResult<()> {
    use crate::ast::Statement;
    use crate::type_checker::visitor::default::DefaultVisitor;
    use crate::type_checker::TypeCheckError;

    // Test assignment with type mismatch
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // First, create a variable with a specific Result type
    ctx.scope.insert_type(
        "typed_result".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("String".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Try to assign an Ok expression with the wrong type
    let stmt = Statement::Assignment {
        target: vec![Expression::Variable("typed_result".to_string())],
        value: Expression::Ok(Box::new(Expression::Literal(Literal::Integer(42)))),
    };

    // Visit the statement should result in a type mismatch error
    let result = visitor.visit_statement(&stmt, &mut ctx);
    assert!(matches!(result, Err(TypeCheckError::TypeMismatch { .. })));

    // Test nested Ok expressions with type mismatch
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Register a variable with a specific nested Result type
    ctx.scope.insert_type(
        "nested_result".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Result {
                ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                err_type: Box::new(TypeInfo::Simple("Error".to_string())),
            }),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Create a nested Ok expression with the wrong inner type
    let inner_ok = Expression::Ok(Box::new(Expression::Literal(Literal::Integer(42))));
    let nested_ok = Expression::Ok(Box::new(inner_ok));

    // Try to assign the nested Ok expression to the variable
    let stmt = Statement::Assignment {
        target: vec![Expression::Variable("nested_result".to_string())],
        value: nested_ok,
    };

    // Visit the statement should result in a type mismatch error
    let result = visitor.visit_statement(&stmt, &mut ctx);
    assert!(matches!(result, Err(TypeCheckError::TypeMismatch { .. })));

    Ok(())
}
