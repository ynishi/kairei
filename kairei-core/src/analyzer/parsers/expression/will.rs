use super::{
    super::{super::prelude::*},
    parse_arguments, parse_identifier,
};
use crate::ast;
use crate::tokenizer::{keyword::Keyword, token::Token, symbol::Delimiter};
use crate::analyzer::core::Parser;

/// Parse a will action expression
pub fn parse_will_action() -> impl Parser<Token, ast::Expression> {
    with_context(
        map(
            tuple3(
                as_unit(parse_will_keyword()),
                parse_identifier(), // action name
                choice(vec![
                    // With target
                    Box::new(map(
                        tuple3(
                            parse_arguments(),
                            as_unit(parse_to_keyword()),
                            parse_identifier(),
                        ),
                        |(parameters, _, target)| (parameters, Some(target)),
                    )),
                    // Without target
                    Box::new(map(parse_arguments(), |parameters| (parameters, None))),
                ]),
            ),
            |(_, action, (parameters, target))| ast::Expression::WillAction {
                action,
                parameters: parameters.into_iter().map(|arg| match arg {
                    ast::Argument::Named { value, .. } => value,
                    ast::Argument::Positional(value) => value,
                }).collect(),
                target,
            },
        ),
        "will action",
    )
}

/// Parse the 'will' keyword
fn parse_will_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Will)), "will keyword")
}

/// Parse the 'to' keyword
fn parse_to_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::To)), "to keyword")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{
        token::Token,
        literal::{Literal, StringLiteral, StringPart},
        symbol::Delimiter,
    };
    use crate::analyzer::core::Parser;

    #[test]
    fn test_parse_will_action_simple() {
        let input = &[
            Token::Keyword(Keyword::Will),
            Token::Identifier("notify".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Literal(Literal::String(StringLiteral::Single(vec![
                StringPart::Literal("Important update".to_string()),
            ]))),
            Token::Delimiter(Delimiter::CloseParen),
        ];

        let (rest, expr) = parse_will_action().parse(input, 0).unwrap();
        assert_eq!(rest, input.len());

        if let ast::Expression::WillAction { action, parameters, target } = expr {
            assert_eq!(action, "notify");
            assert_eq!(parameters.len(), 1);
            assert!(target.is_none());
        } else {
            panic!("Expected WillAction expression");
        }
    }

    #[test]
    fn test_parse_will_action_with_target() {
        let input = &[
            Token::Keyword(Keyword::Will),
            Token::Identifier("suggest".to_string()),
            Token::Delimiter(Delimiter::OpenParen),
            Token::Identifier("options".to_string()),
            Token::Delimiter(Delimiter::CloseParen),
            Token::Keyword(Keyword::To),
            Token::Identifier("user".to_string()),
        ];

        let (rest, expr) = parse_will_action().parse(input, 0).unwrap();
        assert_eq!(rest, input.len());

        if let ast::Expression::WillAction { action, parameters, target } = expr {
            assert_eq!(action, "suggest");
            assert_eq!(parameters.len(), 1);
            assert_eq!(target, Some("user".to_string()));
        } else {
            panic!("Expected WillAction expression");
        }
    }
}
