use nom::{branch::alt, bytes::complete::tag, combinator::value, IResult};

use super::tokenizer::Token;

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
    With,
    On,
}

// Parser for keywords
pub fn parse_keyword(input: &str) -> IResult<&str, Token> {
    let (input, kw) = alt((
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
    ))(input)?;
    Ok((input, Token::Keyword(kw)))
}
