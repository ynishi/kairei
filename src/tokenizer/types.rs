use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    error::context,
};

use super::token::{ParserResult, Token};

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

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_types(input: &str) -> ParserResult<Token> {
    context(
        "type",
        map(
            alt((
                value(Type::Ok, tag("Ok")),
                value(Type::Err, tag("Err")),
                value(Type::Int, tag("Int")),
                value(Type::Float, tag("Float")),
                value(Type::String, tag("String")),
                value(Type::Bool, tag("Bool")),
                value(Type::Any, tag("Any")),
            )),
            Token::Type,
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_types() {
        let test_cases = [
            ("Ok Test", Type::Ok),
            ("Err Test", Type::Err),
            ("Int Test", Type::Int),
            ("Float Test", Type::Float),
            ("String Test", Type::String),
            ("Bool Test", Type::Bool),
            ("Any Test", Type::Any),
        ];

        for (input, expected_type) in test_cases.iter() {
            let (rest, token) = parse_types(input).unwrap();
            assert_eq!(token, Token::Type(expected_type.clone()));
            assert_eq!(rest, " Test");
        }
    }

    #[test]
    fn test_all_types() {
        for type_string in Type::iter().map(|t| t.to_string()) {
            let (rest, token) = parse_types(&type_string).unwrap();
            let t = Type::from_str(&type_string).unwrap();
            assert_eq!(token, Token::Type(t));
            assert_eq!(rest, "");
        }
    }
}
