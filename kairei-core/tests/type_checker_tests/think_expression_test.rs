use kairei_core::{
    Argument,
    ast::{
        AnswerDef, Expression, HandlerBlock, Literal, MicroAgentDef, RequestHandler, RequestType,
        Root, Statement, TypeInfo,
    },
    type_checker::{TypeCheckError, TypeContext, TypeVisitor, visitor::DefaultVisitor},
};

#[test]
fn test_think_expression_type_checking() {
    let mut ctx = TypeContext::new();
    let mut visitor = DefaultVisitor::new();

    // Create AST that matches the original failing case
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
                        statements: vec![Statement::Return(Expression::Think {
                            args: vec![Argument::Positional(Expression::Literal(Literal::String(
                                "Tokyo".to_string(),
                            )))],
                            with_block: None,
                        })],
                    },
                }],
            }),
            react: None,
        }],
        sistence_agent_defs: vec![],
    };

    // This should now pass with our fix
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
fn test_think_expression_in_assignment() {
    let mut ctx = TypeContext::new();
    let mut visitor = DefaultVisitor::new();

    // Create AST with a variable assignment from a Think expression
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
                                target: vec![Expression::Variable("plan".to_string())],
                                value: Expression::Think {
                                    args: vec![Argument::Positional(Expression::Literal(
                                        Literal::String("Tokyo".to_string()),
                                    ))],
                                    with_block: None,
                                },
                            },
                            Statement::Return(Expression::Variable("plan".to_string())),
                        ],
                    },
                }],
            }),
            react: None,
        }],
        sistence_agent_defs: vec![],
    };

    // This should pass with our implementation
    assert!(visitor.visit_root(&mut ast.clone(), &mut ctx).is_ok());
}
