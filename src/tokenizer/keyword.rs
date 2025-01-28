use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    error::context,
};

use super::token::{ParserResult, Token};

#[derive(
    Debug, Clone, PartialEq, strum::EnumString, strum::Display, strum::EnumIter, strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    Micro,
    World,
    Policy,
    State,
    Observe,
    Answer,
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
}

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_keyword(input: &str) -> ParserResult<Token> {
    context(
        "keyword",
        map(
            alt((
                value(Keyword::Micro, tag("micro")),
                value(Keyword::World, tag("world")),
                value(Keyword::Policy, tag("policy")),
                value(Keyword::State, tag("state")),
                value(Keyword::Observe, tag("observe")),
                value(Keyword::Answer, tag("answer")),
                value(Keyword::Request, tag("request")),
                value(Keyword::Emit, tag("emit")),
                value(Keyword::Think, tag("think")),
                value(Keyword::If, tag("if")),
                value(Keyword::Else, tag("else")),
                value(Keyword::Return, tag("return")),
                value(Keyword::Await, tag("await")),
                value(Keyword::OnFail, tag("onFail")),
                value(Keyword::OnInit, tag("onInit")),
                value(Keyword::OnDestroy, tag("onDestroy")),
                value(Keyword::With, tag("with")),
                value(Keyword::On, tag("on")),
                // TODO: Add more keywords when Keywords enum is updated
            )),
            Token::Keyword,
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use strum::IntoEnumIterator;

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
            let (rest, token) = parse_keyword(&keyword_string).unwrap();
            let k = Keyword::from_str(&keyword_string).unwrap();
            assert_eq!(token, Token::Keyword(k));
            assert_eq!(rest, "");
        }
    }
}
