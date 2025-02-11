use std::collections::HashMap;

use crate::analyzer::parsers::world::*;
use crate::analyzer::Parser;
use crate::ast;
use crate::tokenizer::{keyword::*, literal::*, symbol::*, token::Token};

#[test]
fn test_parse_world() {
    let input = vec![
        Token::Keyword(Keyword::World),
        Token::Identifier("test".to_string()),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Keyword(Keyword::Policy),
        Token::Literal(Literal::String(vec![StringPart::Literal(
            "test".to_string(),
        )])),
        Token::Delimiter(Delimiter::CloseBrace),
    ];
    let (rest, world) = parse_world().parse(&input, 0).unwrap();
    assert_eq!(rest, 6);
    assert_eq!(
        world,
        ast::WorldDef {
            name: "test".to_string(),
            policies: vec![ast::Policy {
                text: "test".to_string(),
                scope: ast::PolicyScope::World(Default::default()),
                internal_id: world.policies[0].internal_id.clone(), // PolicyId is randomly generated
            }],
            config: None,
            events: ast::EventsDef { events: vec![] },
            handlers: ast::HandlersDef { handlers: vec![] },
        }
    );
}

#[test]
fn test_parse_config() {
    let input = vec![
        Token::Keyword(Keyword::Config),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::Integer(1)),
        Token::Delimiter(Delimiter::CloseBrace),
    ];
    let (rest, config) = parse_config().parse(&input, 0).unwrap();
    assert_eq!(rest, 6);
    let mut items = HashMap::new();
    items.insert("name".to_string(), ast::Literal::Integer(1));
    assert_eq!(config, ast::ConfigDef::from(items));
}

#[test]
fn test_parse_config_item() {
    let input = vec![
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Literal(Literal::Integer(1)),
    ];
    let (rest, item) = parse_config_item().parse(&input, 0).unwrap();
    assert_eq!(rest, 3);
    assert_eq!(item, ("name".to_string(), ast::Literal::Integer(1)));
}

#[test]
fn test_parse_events() {
    let input = vec![
        Token::Keyword(Keyword::Events),
        Token::Delimiter(Delimiter::OpenBrace),
        Token::Identifier("test".to_string()),
        Token::Delimiter(Delimiter::OpenParen),
        Token::Delimiter(Delimiter::CloseParen),
        Token::Delimiter(Delimiter::CloseBrace),
    ];
    let result = parse_events().parse(&input, 0);
    assert_eq!(
        result,
        Ok((
            6,
            ast::EventsDef {
                events: vec![ast::CustomEventDef {
                    name: "test".to_string(),
                    parameters: vec![]
                }]
            }
        ))
    );
}

#[test]
fn test_parse_parameters() {
    let input = vec![
        Token::Delimiter(Delimiter::OpenParen),
        Token::Identifier("param1".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
        Token::Delimiter(Delimiter::CloseParen),
    ];
    let expected = vec![ast::Parameter {
        name: "param1".to_string(),
        type_info: ast::TypeInfo::Simple("String".to_string()),
    }];
    assert_eq!(parse_parameters().parse(&input, 0), Ok((5, expected)));
}

#[test]
fn test_parse_parameter() {
    let input = vec![
        Token::Identifier("name".to_string()),
        Token::Delimiter(Delimiter::Colon),
        Token::Identifier("String".to_string()),
    ];
    let expected = ast::Parameter {
        name: "name".to_string(),
        type_info: ast::TypeInfo::Simple("String".to_string()),
    };
    assert_eq!(parse_parameter().parse(&input, 0), Ok((3, expected)));
}
