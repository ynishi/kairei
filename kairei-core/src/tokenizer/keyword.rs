//! # Keyword Token Handling
//!
//! This module defines the keywords recognized by the KAIREI DSL and provides
//! functionality for parsing keyword tokens.
//!
//! ## Keyword Types
//!
//! Keywords are categorized into several groups:
//!
//! * **Structure Keywords**: `micro`, `world`, `state`, etc.
//! * **Handler Keywords**: `observe`, `answer`, `react`, etc.
//! * **Control Flow Keywords**: `if`, `else`, `return`, etc.
//! * **Lifecycle Keywords**: `onInit`, `onDestroy`, etc.
//!
//! ## Parsing Strategy
//!
//! Keywords are parsed using a boundary-aware approach to ensure that identifiers
//! that start with keywords are not mistakenly recognized as keywords. For example,
//! `microservice` should be parsed as an identifier, not as the keyword `micro` followed
//! by the identifier `service`.
//!
//! ## Extensibility
//!
//! The [`Keyword`] enum uses `strum` derive macros to enable:
//!
//! * String conversion via `EnumString`
//! * Display formatting via `Display`
//! * Iteration over all keywords via `EnumIter`
//! * String reference access via `AsRefStr`

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::{map, not, peek, value},
    error::context,
    sequence::terminated,
};

use super::token::{ParserResult, Token};

/// Represents the keywords recognized by the KAIREI DSL.
///
/// Keywords are reserved words that have special meaning in the language.
/// They are used to define the structure and behavior of KAIREI agents.
#[derive(
    Debug, Clone, PartialEq, strum::EnumString, strum::Display, strum::EnumIter, strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    /// Defines a MicroAgent component.
    Micro,
    /// Defines a World component.
    World,
    /// Defines a handlers block.
    Handlers,
    /// Defines an events block.
    Events,
    /// Defines a configuration block.
    Config,
    /// Defines a policy statement.
    Policy,
    /// Defines a state block.
    State,
    /// Defines an observe block for event handling.
    Observe,
    /// Defines an answer block for request handling.
    Answer,
    /// Defines a query block.
    Query,
    /// Defines an action block.
    Action,
    /// Defines a react block for event reactions.
    React,
    /// Used in request handler definitions.
    Request,
    /// Used to emit events.
    Emit,
    /// Used for LLM prompting.
    Think,
    /// Control flow keyword for conditional execution.
    If,
    /// Control flow keyword for alternative execution.
    Else,
    /// Used to return values from handlers.
    Return,
    /// Used for asynchronous operations.
    Await,
    /// Lifecycle hook for failure handling.
    #[strum(serialize = "onFail")]
    OnFail,
    /// Lifecycle hook for initialization.
    #[strum(serialize = "onInit")]
    OnInit,
    /// Lifecycle hook for cleanup.
    #[strum(serialize = "onDestroy")]
    OnDestroy,
    /// Defines a lifecycle block.
    #[strum(serialize = "lifecycle")]
    Lifecycle,
    /// Used with think blocks for additional parameters.
    With,
    /// Used in various contexts for targeting.
    To,
    /// Used in event and request handler definitions.
    On,
    /// Used for error propagation.
    #[strum(serialize = "reThrow")]
    ReThrow,
    /// Used for defining sistence agents.
    Sistence,
    /// Used for proactive actions.
    Will,
}

/// Parses a keyword token from the input string.
///
/// This function attempts to match one of the defined keywords at the current position
/// in the input string. It uses a boundary-aware approach to ensure that identifiers
/// that start with keywords are not mistakenly recognized as keywords.
///
/// # Arguments
///
/// * `input` - The input string to parse
///
/// # Returns
///
/// * `ParserResult<Token>` - A result containing either the parsed token and remaining input,
///   or an error if parsing fails
///
/// # Examples
///
/// ```
/// # use kairei_core::tokenizer::keyword::parse_keyword;
/// # use kairei_core::tokenizer::token::Token;
/// # use kairei_core::tokenizer::keyword::Keyword;
/// let input = "micro Agent";
/// let (rest, token) = parse_keyword(input).unwrap();
/// assert_eq!(token, Token::Keyword(Keyword::Micro));
/// assert_eq!(rest, " Agent");
/// ```
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
                    value(
                        Keyword::To,
                        terminated(
                            tag("to"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Sistence,
                        terminated(
                            tag("sistence"),
                            not(peek(take_while1(|c: char| c.is_alphanumeric() || c == '_'))),
                        ),
                    ),
                    value(
                        Keyword::Will,
                        terminated(
                            tag("will"),
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

    use crate::tokenizer::token::Token;

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
