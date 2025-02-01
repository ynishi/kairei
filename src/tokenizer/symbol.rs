use strum_macros::{AsRefStr, Display, EnumString};

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    error::context,
};

use super::token::{ParserResult, Token};

#[derive(Debug, Clone, PartialEq, EnumString, Display, AsRefStr)]
pub enum Operator {
    // 関数関連
    #[strum(serialize = "=>")]
    Arrow,
    #[strum(serialize = "->")]
    ThinArrow,

    // アクセス
    #[strum(serialize = ".")]
    Dot,
    #[strum(serialize = "::")]
    Scope,

    // 比較
    #[strum(serialize = "==")]
    EqualEqual,
    #[strum(serialize = "!=")]
    NotEqual,
    #[strum(serialize = ">")]
    Greater,
    #[strum(serialize = ">=")]
    GreaterEqual,
    #[strum(serialize = "<")]
    Less,
    #[strum(serialize = "<=")]
    LessEqual,

    // 算術
    #[strum(serialize = "+")]
    Plus,
    #[strum(serialize = "-")]
    Minus,
    #[strum(serialize = "*")]
    Multiply,
    #[strum(serialize = "/")]
    Divide,

    // 論理
    #[strum(serialize = "&&")]
    And,
    #[strum(serialize = "||")]
    Or,
    #[strum(serialize = "!")]
    Not,
}

// CloseBrace のシリアライザに直接設定するとエラーになるため、定数を定義
#[allow(dead_code)]
const CLOSE_BRACE: &str = "}";

#[derive(Debug, Clone, PartialEq, EnumString, Display, AsRefStr)]
pub enum Delimiter {
    #[strum(serialize = "{")]
    OpenBrace,
    #[strum(serialize = "CLOSE_BRACE")]
    CloseBrace,
    #[strum(serialize = "(")]
    OpenParen,
    #[strum(serialize = ")")]
    CloseParen,
    #[strum(serialize = "[")]
    OpenBracket,
    #[strum(serialize = "]")]
    CloseBracket,
    #[strum(serialize = ",")]
    Comma,
    #[strum(serialize = ";")]
    Semicolon,
    #[strum(serialize = ":")]
    Colon,
    #[strum(serialize = "=")]
    Equal,
}

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_operator(input: &str) -> ParserResult<Token> {
    context(
        "operator",
        map(
            alt((
                // 2文字演算子
                value(Operator::Arrow, tag("=>")),
                value(Operator::ThinArrow, tag("->")),
                value(Operator::Scope, tag("::")),
                value(Operator::EqualEqual, tag("==")),
                value(Operator::NotEqual, tag("!=")),
                value(Operator::GreaterEqual, tag(">=")),
                value(Operator::LessEqual, tag("<=")),
                value(Operator::And, tag("&&")),
                value(Operator::Or, tag("||")),
                // 1文字演算子
                value(Operator::Dot, tag(".")),
                value(Operator::Greater, tag(">")),
                value(Operator::Less, tag("<")),
                value(Operator::Plus, tag("+")),
                value(Operator::Minus, tag("-")),
                value(Operator::Multiply, tag("*")),
                value(Operator::Divide, tag("/")),
                value(Operator::Not, tag("!")),
            )),
            Token::Operator,
        ),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_delimiter(input: &str) -> ParserResult<Token> {
    context(
        "delimiter",
        map(
            alt((
                value(Delimiter::OpenBrace, tag("{")),
                value(Delimiter::CloseBrace, tag(CLOSE_BRACE)),
                value(Delimiter::OpenParen, tag("(")),
                value(Delimiter::CloseParen, tag(")")),
                value(Delimiter::OpenBracket, tag("[")),
                value(Delimiter::CloseBracket, tag("]")),
                value(Delimiter::Comma, tag(",")),
                value(Delimiter::Semicolon, tag(";")),
                value(Delimiter::Colon, tag(":")),
                value(Delimiter::Equal, tag("=")),
            )),
            Token::Delimiter,
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operators() {
        let test_cases = [
            ("=>", Token::Operator(Operator::Arrow)),
            ("->", Token::Operator(Operator::ThinArrow)),
            ("::", Token::Operator(Operator::Scope)),
            ("==", Token::Operator(Operator::EqualEqual)),
            ("!=", Token::Operator(Operator::NotEqual)),
            (">=", Token::Operator(Operator::GreaterEqual)),
            (".", Token::Operator(Operator::Dot)),
            (">", Token::Operator(Operator::Greater)),
        ];

        for (input, expected) in test_cases.iter() {
            let (rest, token) = parse_operator(input).unwrap();
            assert_eq!(token, *expected);
            assert_eq!(rest, "");
        }
    }

    #[test]
    fn test_delimiters() {
        let test_cases = [
            ("{", Token::Delimiter(Delimiter::OpenBrace)),
            ("}", Token::Delimiter(Delimiter::CloseBrace)),
            ("(", Token::Delimiter(Delimiter::OpenParen)),
            (")", Token::Delimiter(Delimiter::CloseParen)),
            ("[", Token::Delimiter(Delimiter::OpenBracket)),
            ("]", Token::Delimiter(Delimiter::CloseBracket)),
            (",", Token::Delimiter(Delimiter::Comma)),
            (";", Token::Delimiter(Delimiter::Semicolon)),
            (":", Token::Delimiter(Delimiter::Colon)),
            ("=", Token::Delimiter(Delimiter::Equal)),
        ];

        for (input, expected) in test_cases.iter() {
            let (rest, token) = parse_delimiter(input).unwrap();
            assert_eq!(token, *expected);
            assert_eq!(rest, "");
        }
    }

    #[test]
    fn test_operator_precedence() {
        // ">="が">"として誤って解釈されないことを確認
        let (rest, token) = parse_operator(">=").unwrap();
        assert_eq!(token, Token::Operator(Operator::GreaterEqual));
        assert_eq!(rest, "");
    }
}
