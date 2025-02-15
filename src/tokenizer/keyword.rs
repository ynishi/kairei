use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::{map, not, peek, value},
    error::context,
    sequence::terminated,
};

use super::token::{ParserResult, Token};

#[derive(
    Debug, Clone, PartialEq, strum::EnumString, strum::Display, strum::EnumIter, strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    Micro,
    World,
    Handlers,
    Events,
    Config,
    Policy,
    State,
    Observe,
    Answer,
    Query,
    Action,
    React,
    Request,
    Emit,
    Think,
    If,
    Else,
    Return,
    Await,
    #[strum(serialize = "onFail")]
    OnFail,
    #[strum(serialize = "onInit")]
    OnInit,
    #[strum(serialize = "onDestroy")]
    OnDestroy,
    #[strum(serialize = "lifecycle")]
    Lifecycle,
    With,
    On,
    #[strum(serialize = "reThrow")]
    ReThrow,
}

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_keyword(input: &str) -> ParserResult<Token> {
    context(
        "keyword",
        map(
            alt((
                alt((
                    value(
                        Keyword::Micro,
                        terminated(
                            tag("micro"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::World,
                        terminated(
                            tag("world"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Handlers,
                        terminated(
                            tag("handlers"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Events,
                        terminated(
                            tag("events"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Config,
                        terminated(
                            tag("config"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Policy,
                        terminated(
                            tag("policy"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::State,
                        terminated(
                            tag("state"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Observe,
                        terminated(
                            tag("observe"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Answer,
                        terminated(
                            tag("answer"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Query,
                        terminated(
                            tag("query"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Action,
                        terminated(
                            tag("action"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::React,
                        terminated(
                            tag("react"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Request,
                        terminated(
                            tag("request"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Emit,
                        terminated(
                            tag("emit"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Think,
                        terminated(
                            tag("think"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::If,
                        terminated(
                            tag("if"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Else,
                        terminated(
                            tag("else"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Return,
                        terminated(
                            tag("return"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Await,
                        terminated(
                            tag("await"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::OnFail,
                        terminated(
                            tag("onFail"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::OnInit,
                        terminated(
                            tag("onInit"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    // separated for alt max size limit
                )),
                alt((
                    value(
                        Keyword::OnDestroy,
                        terminated(
                            tag("onDestroy"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Lifecycle,
                        terminated(
                            tag("lifecycle"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::With,
                        terminated(
                            tag("with"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::On,
                        terminated(
                            tag("on"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::ReThrow,
                        terminated(
                            tag("reThrow"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    // TODO: Add more keywords when Keywords enum is updated
                )),
            )),
            Token::Keyword,
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use strum::IntoEnumIterator;
    use tracing::debug;

    use super::*;

    #[test]
    fn test_keywords() {
        let test_cases = [
            ("micro Test", Keyword::Micro),
            ("if Test", Keyword::If),
            ("return Test", Keyword::Return),
            ("await Test", Keyword::Await),
            ("on Test", Keyword::On),
            ("with Test", Keyword::With),
        ];

        for (input, expected_keyword) in test_cases.iter() {
            let (rest, token) = parse_keyword(input).unwrap();
            assert_eq!(token, Token::Keyword(expected_keyword.clone()));
            assert_eq!(rest, " Test");
        }
    }

    // check if all keywords are parsed correctly
    #[test]
    fn test_all_keyword() {
        for keyword_string in Keyword::iter().map(|t| t.to_string()) {
            debug!("Testing keyword: {}", keyword_string);
            let (rest, token) = parse_keyword(&keyword_string).unwrap();
            let k = Keyword::from_str(&keyword_string).unwrap();
            assert_eq!(token, Token::Keyword(k));
            assert_eq!(rest, "");
        }
    }

    #[test]
    fn test_keyword_boundary_failure() {
        let test_cases = ["microX", "if123", "returnx", "onFailExtra"];
        for input in test_cases.iter() {
            assert!(
                parse_keyword(input).is_err(),
                "Input {} should not be recognized as a keyword",
                input
            );
        }
    }
}
