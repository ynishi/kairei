use crate::ast::{
    Expression, HandlerBlock, Literal, Parameter, RequestHandler, RequestType, Root,
    SistenceAgentDef, SistenceConfig, Statement, TypeInfo,
};
use crate::type_checker::{TypeCheckError, run_type_checker};
use std::collections::HashMap;

#[test]
fn test_sistence_agent_valid_config() {
    let mut root = Root {
        world_def: None,
        micro_agent_defs: vec![],
        sistence_agent_defs: vec![SistenceAgentDef {
            name: "TestSistenceAgent".to_string(),
            policies: vec![],
            lifecycle: None,
            state: None,
            observe: None,
            answer: None,
            react: None,
            sistence_config: Some(SistenceConfig {
                level: 0.5,
                initiative_threshold: 0.7,
                domains: vec!["test".to_string()],
                parameters: HashMap::new(),
            }),
        }],
    };

    let result = run_type_checker(&mut root);
    assert!(result.is_ok());
}

#[test]
fn test_sistence_agent_invalid_level() {
    let mut root = Root {
        world_def: None,
        micro_agent_defs: vec![],
        sistence_agent_defs: vec![SistenceAgentDef {
            name: "TestSistenceAgent".to_string(),
            policies: vec![],
            lifecycle: None,
            state: None,
            observe: None,
            answer: None,
            react: None,
            sistence_config: Some(SistenceConfig {
                level: 1.5, // Invalid: should be between 0.0 and 1.0
                initiative_threshold: 0.7,
                domains: vec!["test".to_string()],
                parameters: HashMap::new(),
            }),
        }],
    };

    let result = run_type_checker(&mut root);
    assert!(result.is_err());
    if let Err(err) = result {
        match err {
            TypeCheckError::TypeInferenceError { message, .. } => {
                assert!(message.contains("Sistence proactivity level must be between 0.0 and 1.0"));
            }
            _ => panic!("Expected TypeInferenceError"),
        }
    }
}

#[test]
fn test_sistence_agent_invalid_initiative_threshold() {
    let mut root = Root {
        world_def: None,
        micro_agent_defs: vec![],
        sistence_agent_defs: vec![SistenceAgentDef {
            name: "TestSistenceAgent".to_string(),
            policies: vec![],
            lifecycle: None,
            state: None,
            observe: None,
            answer: None,
            react: None,
            sistence_config: Some(SistenceConfig {
                level: 0.5,
                initiative_threshold: -0.1, // Invalid: should be between 0.0 and 1.0
                domains: vec!["test".to_string()],
                parameters: HashMap::new(),
            }),
        }],
    };

    let result = run_type_checker(&mut root);
    assert!(result.is_err());
    if let Err(err) = result {
        match err {
            TypeCheckError::TypeInferenceError { message, .. } => {
                assert!(
                    message.contains("Sistence initiative threshold must be between 0.0 and 1.0")
                );
            }
            _ => panic!("Expected TypeInferenceError"),
        }
    }
}

#[test]
fn test_will_action_expression() {
    let mut root = Root {
        world_def: None,
        micro_agent_defs: vec![],
        sistence_agent_defs: vec![SistenceAgentDef {
            name: "TestSistenceAgent".to_string(),
            policies: vec![],
            lifecycle: None,
            state: None,
            observe: None,
            answer: Some(crate::ast::AnswerDef {
                handlers: vec![RequestHandler {
                    parameters: vec![],
                    return_type: TypeInfo::Result {
                        ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                        err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                    },
                    request_type: RequestType::Custom("TestRequest".to_string()),
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![Statement::Expression(Expression::WillAction {
                            action: "test_action".to_string(),
                            parameters: vec![Expression::Literal(Literal::String(
                                "param1".to_string(),
                            ))],
                            target: Some("target_agent".to_string()),
                        })],
                    },
                }],
            }),
            react: None,
            sistence_config: Some(SistenceConfig {
                level: 0.5,
                initiative_threshold: 0.7,
                domains: vec!["test".to_string()],
                parameters: HashMap::new(),
            }),
        }],
    };

    let result = run_type_checker(&mut root);
    assert!(result.is_ok());
}

#[test]
fn test_will_action_with_invalid_parameters() {
    // This test would be more complex in a real implementation
    // For now, we'll just test that the WillAction is properly type-checked
    let mut root = Root {
        world_def: None,
        micro_agent_defs: vec![],
        sistence_agent_defs: vec![SistenceAgentDef {
            name: "TestSistenceAgent".to_string(),
            policies: vec![],
            lifecycle: None,
            state: None,
            observe: None,
            answer: Some(crate::ast::AnswerDef {
                handlers: vec![RequestHandler {
                    parameters: vec![Parameter {
                        name: "param".to_string(),
                        type_info: TypeInfo::Simple("String".to_string()),
                    }],
                    return_type: TypeInfo::Result {
                        ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                        err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                    },
                    request_type: RequestType::Custom("TestRequest".to_string()),
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![
                            Statement::Expression(Expression::Variable(
                                "undefined_var".to_string(),
                            )),
                            Statement::Expression(Expression::WillAction {
                                action: "test_action".to_string(),
                                parameters: vec![Expression::Variable("undefined_var".to_string())],
                                target: Some("target_agent".to_string()),
                            }),
                        ],
                    },
                }],
            }),
            react: None,
            sistence_config: Some(SistenceConfig {
                level: 0.5,
                initiative_threshold: 0.7,
                domains: vec!["test".to_string()],
                parameters: HashMap::new(),
            }),
        }],
    };

    let result = run_type_checker(&mut root);
    assert!(result.is_err());
    if let Err(err) = result {
        match err {
            TypeCheckError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "undefined_var");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }
}
