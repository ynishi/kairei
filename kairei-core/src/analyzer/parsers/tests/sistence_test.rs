use crate::analyzer::parsers::agent::parse_sistence_agent_def;
use crate::analyzer::parsers::expression::parse_will_action;
use crate::ast;
use crate::tokenizer::keyword::Keyword;
use crate::tokenizer::literal::Literal;
use crate::tokenizer::symbol::Symbol;
use crate::tokenizer::token::Token;

#[test]
fn test_parse_sistence_agent() {
    let input = &[
        Token::Keyword(Keyword::Sistence),
        Token::Identifier("TestAgent".to_string()),
        Token::Symbol(Symbol::OpenBrace),
        Token::Keyword(Keyword::Policy),
        Token::StringLiteral(crate::tokenizer::literal::StringLiteral {
            parts: vec![crate::tokenizer::literal::StringPart::Text(
                "Proactively assist users".to_string(),
            )],
        }),
        Token::Symbol(Symbol::CloseBrace),
    ];

    let (rest, agent) = parse_sistence_agent_def().parse(input).unwrap();
    assert!(rest.is_empty());
    assert_eq!(agent.name, "TestAgent");
    assert_eq!(agent.policies.len(), 1);
    assert_eq!(agent.policies[0].text, "Proactively assist users");
}

#[test]
fn test_parse_will_action() {
    let input = &[
        Token::Keyword(Keyword::Will),
        Token::Identifier("notify".to_string()),
        Token::Symbol(Symbol::OpenParen),
        Token::StringLiteral(crate::tokenizer::literal::StringLiteral {
            parts: vec![crate::tokenizer::literal::StringPart::Text(
                "Important update".to_string(),
            )],
        }),
        Token::Symbol(Symbol::CloseParen),
    ];

    let (rest, expr) = parse_will_action().parse(input).unwrap();
    assert!(rest.is_empty());

    if let ast::Expression::WillAction {
        action,
        parameters,
        target,
    } = expr
    {
        assert_eq!(action, "notify");
        assert_eq!(parameters.len(), 1);
        assert!(target.is_none());
    } else {
        panic!("Expected WillAction expression");
    }
}
