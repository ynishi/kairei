use kairei::{
    ast::{
        AnswerDef, Expression, HandlerBlock, Literal, MicroAgentDef, RequestAttributes,
        RequestHandler, RequestType, Root, Statement, TypeInfo,
    },
    type_checker::{visitor::DefaultVisitor, TypeCheckError, TypeContext, TypeVisitor},
    Argument,
};
use std::time::Duration;

#[test]
fn test_request_expression_type_checking() {
    let mut ctx = TypeContext::new();
    let mut visitor = DefaultVisitor::new();

    // Create AST with a Request expression
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
                    return_type: TypeInfo::Result {
                        ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                        err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                    },
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![Statement::Return(Expression::Request {
                            agent: "WeatherAgent".to_string(),
                            request_type: RequestType::Custom("GetWeather".to_string()),
                            parameters: vec![Argument::Positional(Expression::Literal(
                                Literal::String("Tokyo".to_string()),
                            ))],
                            options: None,
                        })],
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
fn test_request_expression_in_assignment() {
    let mut ctx = TypeContext::new();
    let mut visitor = DefaultVisitor::new();

    // Create AST with a variable assignment from a Request expression
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
                    return_type: TypeInfo::Result {
                        ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                        err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                    },
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![
                            Statement::Assignment {
                                target: vec![Expression::Variable("weather".to_string())],
                                value: Expression::Request {
                                    agent: "WeatherAgent".to_string(),
                                    request_type: RequestType::Custom("GetWeather".to_string()),
                                    parameters: vec![Argument::Positional(Expression::Literal(
                                        Literal::String("Tokyo".to_string()),
                                    ))],
                                    options: Some(RequestAttributes {
                                        timeout: Some(Duration::from_secs(5)),
                                        retry: Some(3),
                                    }),
                                },
                            },
                            Statement::Return(Expression::Variable("weather".to_string())),
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
