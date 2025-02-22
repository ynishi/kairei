use crate::{
    ast::{
        AnswerDef, ErrorHandlerBlock, EventHandler, EventType, Expression, HandlerBlock, Literal,
        MicroAgentDef, ObserveDef, Parameter, RequestHandler, RequestType, Root, StateDef,
        StateVarDef, Statement, TypeInfo,
    },
    type_checker::{TypeCheckError, TypeCheckResult, TypeChecker},
};

use super::TestPlugin;

#[test]
fn test_complex_ast_type_checking() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();

    // Register state variables in scope
    checker.insert_type("counter".to_string(), TypeInfo::Simple("Int".to_string()));
    checker.insert_type("data".to_string(), TypeInfo::Simple("String".to_string()));

    // Register function in scope with argument type
    checker.insert_type(
        "risky_operation".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Array(Box::new(TypeInfo::Simple(
                "Int".to_string(),
            )))),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Create a complex AST with nested structures
    let mut root = Root {
        micro_agent_defs: vec![MicroAgentDef {
            name: "ComplexAgent".to_string(),
            state: Some(StateDef {
                variables: vec![
                    (
                        "counter".to_string(),
                        StateVarDef {
                            name: "counter".to_string(),
                            type_info: TypeInfo::Simple("Int".to_string()),
                            initial_value: None,
                        },
                    ),
                    (
                        "data".to_string(),
                        StateVarDef {
                            name: "data".to_string(),
                            type_info: TypeInfo::Simple("String".to_string()),
                            initial_value: None,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
            }),
            observe: Some(ObserveDef {
                handlers: vec![EventHandler {
                    event_type: EventType::Custom("increment".to_string()),
                    parameters: vec![],
                    block: HandlerBlock {
                        statements: vec![
                            Statement::Assignment {
                                target: vec![Expression::Variable("counter".to_string())],
                                value: Expression::Literal(Literal::Integer(42)),
                            },
                            Statement::WithError {
                                statement: Box::new(Statement::Expression(
                                    Expression::FunctionCall {
                                        function: "risky_operation".to_string(),
                                        arguments: vec![Expression::Literal(Literal::Integer(1))],
                                    },
                                )),
                                error_handler_block: ErrorHandlerBlock {
                                    error_binding: None,
                                    error_handler_statements: vec![Statement::Return(
                                        Expression::Err(Box::new(Expression::Literal(
                                            Literal::String("Operation failed".to_string()),
                                        ))),
                                    )],
                                    control: None,
                                },
                            },
                        ],
                    },
                }],
            }),
            ..Default::default()
        }],
        world_def: None,
    };

    checker.check_types(&mut root)
}

#[test]
fn test_error_recovery_and_reporting() {
    let mut checker = TypeChecker::new();

    // Register state variable in scope
    checker.insert_type("number".to_string(), TypeInfo::Simple("Int".to_string()));

    // Create an AST with intentional type errors
    let mut root = Root {
        micro_agent_defs: vec![MicroAgentDef {
            name: "ErrorAgent".to_string(),
            state: Some(StateDef {
                variables: vec![(
                    "number".to_string(),
                    StateVarDef {
                        name: "number".to_string(),
                        type_info: TypeInfo::Simple("Int".to_string()),
                        initial_value: None,
                    },
                )]
                .into_iter()
                .collect(),
            }),
            observe: Some(ObserveDef {
                handlers: vec![EventHandler {
                    event_type: EventType::Custom("invalid_assignment".to_string()),
                    parameters: vec![],
                    block: HandlerBlock {
                        statements: vec![Statement::Assignment {
                            target: vec![Expression::Variable("number".to_string())],
                            value: Expression::Literal(Literal::String("not a number".to_string())),
                        }],
                    },
                }],
            }),
            ..Default::default()
        }],
        world_def: None,
    };

    let result = checker.check_types(&mut root);

    // Verify error details
    match result {
        Err(err) => match err {
            TypeCheckError::TypeMismatch {
                ref expected,
                ref found,
                ..
            } => {
                assert_eq!(expected.to_string(), "Int");
                assert_eq!(found.to_string(), "String");
            }
            _ => panic!("Expected TypeMismatch error, got {:?}", err),
        },
        Ok(_) => panic!("Expected type checking to fail"),
    }
}

#[test]
fn test_plugin_integration() -> TypeCheckResult<()> {
    // Create an AST using plugin features
    let mut root = Root {
        micro_agent_defs: vec![MicroAgentDef {
            name: "PluginAgent".to_string(),
            ..Default::default()
        }],
        world_def: None,
    };

    let mut checker = TypeChecker::new();
    checker.register_plugin(Box::new(TestPlugin));
    checker.check_types(&mut root)
}

#[test]
fn test_event_handler_type_checking() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();

    // Register return type for handlers
    checker.insert_type(
        "return_type".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("Int".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Test successful case
    let mut root = Root {
        micro_agent_defs: vec![MicroAgentDef {
            name: "EventAgent".to_string(),
            observe: Some(ObserveDef {
                handlers: vec![EventHandler {
                    event_type: EventType::Custom("observe_handler".to_string()),
                    parameters: vec![],
                    block: HandlerBlock {
                        statements: vec![Statement::Return(Expression::Ok(Box::new(
                            Expression::Literal(Literal::Integer(42)),
                        )))],
                    },
                }],
            }),
            ..Default::default()
        }],
        world_def: None,
    };

    checker.check_types(&mut root)?;

    // Test error case - wrong return type
    let mut root = Root {
        micro_agent_defs: vec![MicroAgentDef {
            name: "EventAgent".to_string(),
            observe: Some(ObserveDef {
                handlers: vec![EventHandler {
                    event_type: EventType::Custom("observe_handler".to_string()),
                    parameters: vec![],
                    block: HandlerBlock {
                        statements: vec![Statement::Return(Expression::Ok(Box::new(
                            Expression::Literal(Literal::String("wrong type".to_string())),
                        )))],
                    },
                }],
            }),
            ..Default::default()
        }],
        world_def: None,
    };

    let result = checker.check_types(&mut root);
    assert!(matches!(result, Err(TypeCheckError::TypeMismatch { .. })));

    Ok(())
}

#[test]
fn test_request_handler_type_checking_ok() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();

    // Test successful case
    let mut root = Root {
        micro_agent_defs: vec![MicroAgentDef {
            name: "EventAgent".to_string(),
            answer: Some(AnswerDef {
                handlers: vec![RequestHandler {
                    request_type: RequestType::Custom("answer_handler".to_string()),
                    parameters: vec![Parameter {
                        name: "input".to_string(),
                        type_info: TypeInfo::Simple("String".to_string()),
                    }],
                    return_type: TypeInfo::Result {
                        ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                        err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                    },
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![Statement::Return(Expression::Ok(Box::new(
                            Expression::Literal(Literal::String("Response".to_string())),
                        )))],
                    },
                }],
            }),
            ..Default::default()
        }],
        world_def: None,
    };

    checker.check_types(&mut root)?;

    Ok(())
}

#[test]
fn test_request_handler_type_checking_ng() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();

    // Register parameter type
    checker.insert_type("input".to_string(), TypeInfo::Simple("String".to_string()));

    // Test error case - wrong parameter type
    let mut root = Root {
        micro_agent_defs: vec![MicroAgentDef {
            name: "EventAgent".to_string(),
            answer: Some(AnswerDef {
                handlers: vec![RequestHandler {
                    request_type: RequestType::Custom("answer_handler".to_string()),
                    parameters: vec![Parameter {
                        name: "input".to_string(),
                        type_info: TypeInfo::Simple("Int".to_string()), // Wrong type
                    }],
                    return_type: TypeInfo::Result {
                        ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                        err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                    },
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![Statement::Return(Expression::Ok(Box::new(
                            Expression::Literal(Literal::String("Response".to_string())),
                        )))],
                    },
                }],
            }),
            ..Default::default()
        }],
        world_def: None,
    };

    let result = checker.check_types(&mut root);
    assert!(matches!(result, Err(TypeCheckError::TypeMismatch { .. })));

    Ok(())
}

#[test]
fn test_error_handling_scenarios() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();

    // Register functions in scope with argument types
    checker.insert_type(
        "operation1".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Array(Box::new(TypeInfo::Simple(
                "Int".to_string(),
            )))),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );
    checker.insert_type(
        "operation2".to_string(),
        TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Array(Box::new(TypeInfo::Simple(
                "Int".to_string(),
            )))),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        },
    );

    // Test various error handling patterns
    let mut root = Root {
        micro_agent_defs: vec![MicroAgentDef {
            name: "ErrorHandlingAgent".to_string(),
            observe: Some(ObserveDef {
                handlers: vec![EventHandler {
                    event_type: EventType::Custom("complex_error_handling".to_string()),
                    parameters: vec![],
                    block: HandlerBlock {
                        statements: vec![Statement::WithError {
                            statement: Box::new(Statement::Expression(Expression::FunctionCall {
                                function: "operation1".to_string(),
                                arguments: vec![Expression::Literal(Literal::Integer(1))],
                            })),
                            error_handler_block: ErrorHandlerBlock {
                                error_binding: None,
                                error_handler_statements: vec![Statement::WithError {
                                    statement: Box::new(Statement::Expression(
                                        Expression::FunctionCall {
                                            function: "operation2".to_string(),
                                            arguments: vec![Expression::Literal(Literal::Integer(
                                                2,
                                            ))],
                                        },
                                    )),
                                    error_handler_block: ErrorHandlerBlock {
                                        error_binding: None,
                                        error_handler_statements: vec![Statement::Return(
                                            Expression::Err(Box::new(Expression::Literal(
                                                Literal::String("Nested error".to_string()),
                                            ))),
                                        )],
                                        control: None,
                                    },
                                }],
                                control: None,
                            },
                        }],
                    },
                }],
            }),
            ..Default::default()
        }],
        world_def: None,
    };

    checker.check_types(&mut root)
}
