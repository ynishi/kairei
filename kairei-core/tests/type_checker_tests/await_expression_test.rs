use kairei_core::{
    Argument,
    ast::{
        AnswerDef, Expression, HandlerBlock, Literal, MicroAgentDef, RequestHandler, RequestType,
        Root, Statement, TypeInfo,
    },
    type_checker::{TypeCheckError, TypeContext, TypeVisitor, visitor::DefaultVisitor},
};

#[test]
fn test_await_expression_type_checking() {
    let mut ctx = TypeContext::new();
    let mut visitor = DefaultVisitor::new();

    // Create AST with a single await expression
    let ast = Root {
        world_def: None,
        micro_agent_defs: vec![MicroAgentDef {
            name: "TravelAgent".to_string(),
            policies: vec![],
            lifecycle: None,
            state: None,
            observe: None,
            answer: Some(AnswerDef {
                handlers: vec![RequestHandler {
                    request_type: RequestType::Custom("PlanTrip".to_string()),
                    parameters: vec![],
                    return_type: TypeInfo::Simple("Any".to_string()),
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![Statement::Return(Expression::Await(vec![
                            Expression::Request {
                                agent: "WeatherAgent".to_string(),
                                request_type: RequestType::Custom("GetWeather".to_string()),
                                parameters: vec![Argument::Positional(Expression::Literal(
                                    Literal::String("Tokyo".to_string()),
                                ))],
                                options: None,
                            },
                        ]))],
                    },
                }],
            }),
            react: None,
        }],
    };

    // This should pass with our implementation
    assert!(visitor.visit_root(&mut ast.clone(), &mut ctx).is_ok());

    // Test with incorrect return type to verify error handling
    let mut ast_with_wrong_type = ast;
    if let Some(answer_def) = &mut ast_with_wrong_type.micro_agent_defs[0].answer {
        answer_def.handlers[0].return_type = TypeInfo::Simple("Integer".to_string());
    }

    // This should fail with a type mismatch
    match visitor.visit_root(&mut ast_with_wrong_type, &mut ctx) {
        Err(TypeCheckError::TypeMismatch { .. }) => (),
        other => panic!("Expected TypeMismatch error, got {:?}", other),
    }
}

#[test]
fn test_multiple_await_expressions() {
    let mut ctx = TypeContext::new();
    let mut visitor = DefaultVisitor::new();

    // Create AST with multiple await expressions
    let ast = Root {
        world_def: None,
        micro_agent_defs: vec![MicroAgentDef {
            name: "TravelAgent".to_string(),
            policies: vec![],
            lifecycle: None,
            state: None,
            observe: None,
            answer: Some(AnswerDef {
                handlers: vec![RequestHandler {
                    request_type: RequestType::Custom("PlanTrip".to_string()),
                    parameters: vec![],
                    return_type: TypeInfo::Array(Box::new(TypeInfo::Simple("Any".to_string()))),
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![Statement::Return(Expression::Await(vec![
                            Expression::Request {
                                agent: "WeatherAgent".to_string(),
                                request_type: RequestType::Custom("GetWeather".to_string()),
                                parameters: vec![Argument::Positional(Expression::Literal(
                                    Literal::String("Tokyo".to_string()),
                                ))],
                                options: None,
                            },
                            Expression::Request {
                                agent: "HotelAgent".to_string(),
                                request_type: RequestType::Custom("FindHotels".to_string()),
                                parameters: vec![Argument::Positional(Expression::Literal(
                                    Literal::String("Tokyo".to_string()),
                                ))],
                                options: None,
                            },
                        ]))],
                    },
                }],
            }),
            react: None,
        }],
    };

    // This should pass with our implementation
    assert!(visitor.visit_root(&mut ast.clone(), &mut ctx).is_ok());
}

#[test]
fn test_nested_await_expressions() {
    let mut ctx = TypeContext::new();
    let mut visitor = DefaultVisitor::new();

    // Create AST with nested await expressions
    let ast = Root {
        world_def: None,
        micro_agent_defs: vec![MicroAgentDef {
            name: "TravelAgent".to_string(),
            policies: vec![],
            lifecycle: None,
            state: None,
            observe: None,
            answer: Some(AnswerDef {
                handlers: vec![RequestHandler {
                    request_type: RequestType::Custom("PlanTrip".to_string()),
                    parameters: vec![],
                    return_type: TypeInfo::Simple("Any".to_string()),
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![
                            Statement::Assignment {
                                target: vec![Expression::Variable("weather".to_string())],
                                value: Expression::Await(vec![Expression::Request {
                                    agent: "WeatherAgent".to_string(),
                                    request_type: RequestType::Custom("GetWeather".to_string()),
                                    parameters: vec![Argument::Positional(Expression::Literal(
                                        Literal::String("Tokyo".to_string()),
                                    ))],
                                    options: None,
                                }]),
                            },
                            Statement::Return(Expression::Await(vec![Expression::Request {
                                agent: "HotelAgent".to_string(),
                                request_type: RequestType::Custom("FindHotels".to_string()),
                                parameters: vec![Argument::Positional(Expression::Variable(
                                    "weather".to_string(),
                                ))],
                                options: None,
                            }])),
                        ],
                    },
                }],
            }),
            react: None,
        }],
    };

    // This should pass with our implementation
    assert!(visitor.visit_root(&mut ast.clone(), &mut ctx).is_ok());
}
