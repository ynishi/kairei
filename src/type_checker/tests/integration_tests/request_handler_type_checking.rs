use crate::{
    ast::{
        AnswerDef, Expression, HandlerBlock, Literal, MicroAgentDef, Parameter, RequestHandler,
        RequestType, Root, Statement, TypeInfo,
    },
    type_checker::{TypeCheckError, TypeCheckResult, TypeChecker},
};

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
