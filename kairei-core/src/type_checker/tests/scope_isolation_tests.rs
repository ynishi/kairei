use crate::{
    ast::{Expression, HandlerBlock, HandlerDef, Parameter, Statement, TypeInfo},
    type_checker::{
        TypeCheckResult, TypeContext, visitor::DefaultVisitor, visitor::common::TypeVisitor,
    },
};

#[test]
fn test_scope_isolation_between_handlers() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::default();
    let mut ctx = TypeContext::new();

    // Register core types
    ctx.scope
        .insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));
    ctx.scope
        .insert_type("Int".to_string(), TypeInfo::Simple("Int".to_string()));
    ctx.scope
        .insert_type("Error".to_string(), TypeInfo::Simple("Error".to_string()));
    ctx.scope.insert_type(
        "return_type".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("String".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Create first handler with a parameter
    let handler1 = HandlerDef {
        event_name: "FirstEvent".to_string(),
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

    // Create second handler with a different parameter
    let handler2 = HandlerDef {
        event_name: "SecondEvent".to_string(),
        parameters: vec![Parameter {
            name: "param2".to_string(),
            type_info: TypeInfo::Simple("Int".to_string()),
        }],
        block: HandlerBlock {
            statements: vec![Statement::Expression(Expression::Variable(
                "param2".to_string(),
            ))],
        },
    };

    // Check first handler
    visitor.visit_handler(&handler1, &mut ctx)?;

    // param1 should not be in scope after visiting handler1
    assert!(!ctx.scope.contains_type("param1"));

    // Check second handler
    visitor.visit_handler(&handler2, &mut ctx)?;

    // param2 should not be in scope after visiting handler2
    assert!(!ctx.scope.contains_type("param2"));

    Ok(())
}

#[test]
fn test_scope_isolation_in_blocks() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::default();
    let mut ctx = TypeContext::new();

    // Register core types
    ctx.scope
        .insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));
    ctx.scope
        .insert_type("Int".to_string(), TypeInfo::Simple("Int".to_string()));

    // Create a statement with nested blocks
    let statement = Statement::Block(vec![
        // Outer block with variable 'outer'
        Statement::Assignment {
            target: vec![Expression::Variable("outer".to_string())],
            value: Expression::Variable("String".to_string()),
        },
        // Inner block with variable 'inner'
        Statement::Block(vec![
            Statement::Assignment {
                target: vec![Expression::Variable("inner".to_string())],
                value: Expression::Variable("Int".to_string()),
            },
            // Access both variables
            Statement::Expression(Expression::Variable("outer".to_string())),
            Statement::Expression(Expression::Variable("inner".to_string())),
        ]),
        // After inner block, 'inner' should not be accessible
        Statement::Expression(Expression::Variable("outer".to_string())),
    ]);

    // Visit the statement
    visitor.visit_statement(&statement, &mut ctx)?;

    // After visiting, neither variable should be in scope
    assert!(!ctx.scope.contains_type("outer"));
    assert!(!ctx.scope.contains_type("inner"));

    Ok(())
}

#[test]
fn test_scope_isolation_in_conditionals() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::default();
    let mut ctx = TypeContext::new();

    // Register core types
    ctx.scope
        .insert_type("String".to_string(), TypeInfo::Simple("String".to_string()));
    ctx.scope
        .insert_type("Int".to_string(), TypeInfo::Simple("Int".to_string()));
    ctx.scope.insert_type(
        "Boolean".to_string(),
        TypeInfo::Simple("Boolean".to_string()),
    );

    // Create a conditional statement
    let statement = Statement::If {
        condition: Expression::Variable("Boolean".to_string()),
        then_block: vec![
            // Then block with variable 'then_var'
            Statement::Assignment {
                target: vec![Expression::Variable("then_var".to_string())],
                value: Expression::Variable("String".to_string()),
            },
        ],
        else_block: Some(vec![
            // Else block with variable 'else_var'
            Statement::Assignment {
                target: vec![Expression::Variable("else_var".to_string())],
                value: Expression::Variable("Int".to_string()),
            },
        ]),
    };

    // Visit the statement
    visitor.visit_statement(&statement, &mut ctx)?;

    // After visiting, neither variable should be in scope
    assert!(!ctx.scope.contains_type("then_var"));
    assert!(!ctx.scope.contains_type("else_var"));

    Ok(())
}

#[test]
fn test_scope_checkpoint_restoration() -> TypeCheckResult<()> {
    let mut ctx = TypeContext::new();

    // Create initial scope with a type
    ctx.scope.insert_type(
        "base_type".to_string(),
        TypeInfo::Simple("String".to_string()),
    );

    // Create a checkpoint
    let checkpoint = ctx.create_scope_checkpoint();

    // Enter a new scope and add types
    ctx.scope.enter_scope();
    ctx.scope
        .insert_type("temp_type".to_string(), TypeInfo::Simple("Int".to_string()));

    // Verify both types are accessible
    assert!(ctx.scope.contains_type("base_type"));
    assert!(ctx.scope.contains_type("temp_type"));

    // Restore the checkpoint
    ctx.restore_scope_checkpoint(checkpoint);

    // Verify only the base type is accessible
    assert!(ctx.scope.contains_type("base_type"));
    assert!(!ctx.scope.contains_type("temp_type"));

    Ok(())
}
