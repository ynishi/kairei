use crate::{
    ErrorHandlerBlock,
    ast::{Expression, HandlerBlock, HandlerDef, Literal, Parameter, Statement, TypeInfo},
    type_checker::{TypeCheckResult, TypeChecker, TypeContext, visitor::common::TypeVisitor},
};

#[test]
fn test_empty_handler() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    let handler = HandlerDef {
        event_name: "test_event".to_string(),
        parameters: vec![],
        block: HandlerBlock { statements: vec![] },
    };

    checker.visit_handler(&handler, &mut ctx)?;
    Ok(())
}

#[test]
fn test_handler_with_statements() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    // Add return type to context
    ctx.scope.insert_type(
        "return_type".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("Int".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Register Int type
    ctx.scope
        .insert_type("Int".to_string(), TypeInfo::Simple("Int".to_string()));
    ctx.scope
        .insert_type("Error".to_string(), TypeInfo::Simple("Error".to_string()));

    let handler = HandlerDef {
        event_name: "test_event".to_string(),
        parameters: vec![],
        block: HandlerBlock {
            statements: vec![
                Statement::Block(vec![]),
                Statement::Return(Expression::Ok(Box::new(Expression::Literal(
                    Literal::Integer(42),
                )))),
            ],
        },
    };

    checker.visit_handler(&handler, &mut ctx)?;
    Ok(())
}

#[test]
fn test_handler_with_error_handling() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    // Add return type to context
    ctx.scope.insert_type(
        "return_type".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("String".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Register basic types
    ctx.scope
        .insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));
    ctx.scope
        .insert_type("Error".to_string(), TypeInfo::Simple("Error".to_string()));

    let handler = HandlerDef {
        event_name: "test_event".to_string(),
        parameters: vec![],
        block: HandlerBlock {
            statements: vec![Statement::WithError {
                statement: Box::new(Statement::Block(vec![])),
                error_handler_block: ErrorHandlerBlock {
                    error_binding: Some("err".to_string()),
                    error_handler_statements: vec![],
                    control: None,
                },
            }],
        },
    };

    checker.visit_handler(&handler, &mut ctx)?;
    Ok(())
}

#[test]
fn test_handler_with_conditional() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    // Add return type to context
    ctx.scope.insert_type(
        "return_type".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("String".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Register basic types
    ctx.scope
        .insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));
    ctx.scope
        .insert_type("Error".to_string(), TypeInfo::Simple("Error".to_string()));
    ctx.scope.insert_type(
        "Boolean".to_string(),
        TypeInfo::Simple("Boolean".to_string()),
    );

    let handler = HandlerDef {
        event_name: "test_event".to_string(),
        parameters: vec![],
        block: HandlerBlock {
            statements: vec![Statement::If {
                condition: Expression::Literal(Literal::Boolean(true)),
                then_block: vec![],
                else_block: None,
            }],
        },
    };

    checker.visit_handler(&handler, &mut ctx)?;
    Ok(())
}

#[test]
fn test_handler_with_parameters() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    let handler = HandlerDef {
        event_name: "test_event".to_string(),
        parameters: vec![
            Parameter {
                name: "param1".to_string(),
                type_info: TypeInfo::Simple("String".to_string()),
            },
            Parameter {
                name: "param2".to_string(),
                type_info: TypeInfo::Simple("Int".to_string()),
            },
        ],
        block: HandlerBlock { statements: vec![] },
    };

    checker.visit_handler(&handler, &mut ctx)?;
    Ok(())
}
