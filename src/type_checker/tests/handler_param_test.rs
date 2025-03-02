//! Test for handler parameter scope handling

use crate::{
    ast::{Expression, HandlerBlock, HandlerDef, Literal, Parameter, Statement, TypeInfo},
    type_checker::{visitor::common::TypeVisitor, TypeCheckResult, TypeChecker, TypeContext},
};

#[test]
fn test_handler_parameters_used_in_block() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    // Register core types
    ctx.scope.insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));
    ctx.scope.insert_type("Int".to_string(), TypeInfo::Simple("Int".to_string()));
    ctx.scope.insert_type("Error".to_string(), TypeInfo::Simple("Error".to_string()));
    ctx.scope.insert_type(
        "return_type".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("String".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Create a handler with parameters
    let handler = HandlerDef {
        event_name: "TestEvent".to_string(),
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
        block: HandlerBlock {
            statements: vec![
                // Use parameter variables in the block
                Statement::Expression(Expression::Variable("param1".to_string())),
                Statement::Expression(Expression::Variable("param2".to_string())),
                Statement::Return(Expression::Ok(Box::new(Expression::Literal(
                    Literal::String("Success".to_string()),
                )))),
            ],
        },
    };

    // This should not throw an undefined variable error
    checker.visit_handler(&handler, &mut ctx)?;

    Ok(())
}