use strum_macros::{AsRefStr, Display, EnumString};

use nom::{branch::alt, bytes::complete::tag, combinator::map, IResult};

use super::tokenizer::Token;

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
    #[strum(serialize = "=")]
    Equal,
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
}

// 2文字の演算子を先にパースする
pub fn operator(input: &str) -> IResult<&str, Token> {
    alt((
        // 2文字演算子
        map(tag("=>"), |_| Token::Operator(Operator::Arrow)),
        map(tag("->"), |_| Token::Operator(Operator::ThinArrow)),
        map(tag("::"), |_| Token::Operator(Operator::Scope)),
        map(tag("=="), |_| Token::Operator(Operator::EqualEqual)),
        map(tag("!="), |_| Token::Operator(Operator::NotEqual)),
        map(tag(">="), |_| Token::Operator(Operator::GreaterEqual)),
        map(tag("<="), |_| Token::Operator(Operator::LessEqual)),
        map(tag("&&"), |_| Token::Operator(Operator::And)),
        map(tag("||"), |_| Token::Operator(Operator::Or)),
        // 1文字演算子
        map(tag("."), |_| Token::Operator(Operator::Dot)),
        map(tag("="), |_| Token::Operator(Operator::Equal)),
        map(tag(">"), |_| Token::Operator(Operator::Greater)),
        map(tag("<"), |_| Token::Operator(Operator::Less)),
        map(tag("+"), |_| Token::Operator(Operator::Plus)),
        map(tag("-"), |_| Token::Operator(Operator::Minus)),
        map(tag("*"), |_| Token::Operator(Operator::Multiply)),
        map(tag("/"), |_| Token::Operator(Operator::Divide)),
        map(tag("!"), |_| Token::Operator(Operator::Not)),
    ))(input)
}

pub fn delimiter(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("{"), |_| Token::Delimiter(Delimiter::OpenBrace)),
        map(tag("}"), |_| Token::Delimiter(Delimiter::CloseBrace)),
        map(tag("("), |_| Token::Delimiter(Delimiter::OpenParen)),
        map(tag(")"), |_| Token::Delimiter(Delimiter::CloseParen)),
        map(tag("["), |_| Token::Delimiter(Delimiter::OpenBracket)),
        map(tag("]"), |_| Token::Delimiter(Delimiter::CloseBracket)),
        map(tag(","), |_| Token::Delimiter(Delimiter::Comma)),
        map(tag(";"), |_| Token::Delimiter(Delimiter::Semicolon)),
        map(tag(":"), |_| Token::Delimiter(Delimiter::Colon)),
    ))(input)
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
            ("=", Token::Operator(Operator::Equal)),
            (">", Token::Operator(Operator::Greater)),
        ];

        for (input, expected) in test_cases.iter() {
            let (rest, token) = operator(input).unwrap();
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
        ];

        for (input, expected) in test_cases.iter() {
            let (rest, token) = delimiter(input).unwrap();
            assert_eq!(token, *expected);
            assert_eq!(rest, "");
        }
    }

    #[test]
    fn test_operator_precedence() {
        // ">="が">"として誤って解釈されないことを確認
        let (rest, token) = operator(">=").unwrap();
        assert_eq!(token, Token::Operator(Operator::GreaterEqual));
        assert_eq!(rest, "");
    }
}
