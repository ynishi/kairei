use nom::{branch::alt, bytes::complete::tag, combinator::value, IResult};

use super::tokenizer::Token;

#[derive(Debug, Clone, PartialEq, strum::EnumString, strum::Display, strum::EnumIter)]
pub enum Type {
    Ok,
    Err,
    // 他の型も追加予定
    Int,
    Float,
    String,
    Bool,
    Any,
}

// Parser for types
pub fn parse_types(input: &str) -> IResult<&str, Token> {
    alt((
        value(Type::Ok, tag("Ok")),
        value(Type::Err, tag("Err")),
        value(Type::Int, tag("Int")),
        value(Type::Float, tag("Float")),
        value(Type::String, tag("String")),
        value(Type::Bool, tag("Bool")),
        value(Type::Any, tag("Any")),
    ))(input)
    .map(|(input, ty)| (input, Token::Type(ty)))
}
