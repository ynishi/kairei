use crate::{
    ast::{
        AnswerDef, Expression, HandlerBlock, LifecycleDef, Literal, MicroAgentDef, ObserveDef,
        ReactDef, StateDef, Statement,
    },
    type_checker::{visitor::common::TypeVisitor, TypeCheckResult, TypeChecker, TypeContext},
};

#[test]
fn test_empty_micro_agent() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    let mut agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: None,
        lifecycle: None,
        answer: None,
        observe: None,
        react: None,
        policies: vec![],
    };

    checker.visit_micro_agent(&mut agent, &mut ctx)?;
    Ok(())
}

#[test]
fn test_micro_agent_with_state() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    let mut agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: Some(StateDef {
            variables: Default::default(),
        }),
        lifecycle: None,
        answer: None,
        observe: None,
        react: None,
        policies: vec![],
    };

    checker.visit_micro_agent(&mut agent, &mut ctx)?;
    Ok(())
}

#[test]
fn test_micro_agent_with_lifecycle() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    let mut agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: None,
        lifecycle: Some(LifecycleDef {
            on_init: Some(HandlerBlock {
                statements: vec![Statement::Expression(Expression::Literal(
                    Literal::Integer(42),
                ))],
            }),
            on_destroy: None,
        }),
        answer: None,
        observe: None,
        react: None,
        policies: vec![],
    };

    checker.visit_micro_agent(&mut agent, &mut ctx)?;
    Ok(())
}

#[test]
fn test_micro_agent_with_handlers() -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();

    let mut agent = MicroAgentDef {
        name: "test_agent".to_string(),
        state: None,
        lifecycle: None,
        answer: Some(AnswerDef { handlers: vec![] }),
        observe: Some(ObserveDef { handlers: vec![] }),
        react: Some(ReactDef { handlers: vec![] }),
        policies: vec![],
    };

    checker.visit_micro_agent(&mut agent, &mut ctx)?;
    Ok(())
}
