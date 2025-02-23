use nom::{
    branch::alt,
    bytes::{
        complete::{tag, take_while1},
        streaming::take_until,
    },
    character::complete::{char, digit1},
    combinator::{map, map_res, opt, recognize},
    error::context,
    multi::many0,
    sequence::{delimited, pair, tuple},
};

use super::token::{ParserResult, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    /// A literal string segment without any interpolation or special formatting
    Literal(String),
    /// A string interpolation segment containing a variable name to be replaced
    Interpolation(String),
    /// A newline character in the string
    NewLine,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringLiteral {
    /// Single-quoted string with interpolation support
    Single(Vec<StringPart>),
    /// Triple-quoted string with preserved formatting
    Triple(Vec<StringPart>),
}

#[derive(Debug, Clone, PartialEq, strum::Display)]
pub enum Literal {
    String(StringLiteral),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

#[tracing::instrument(level = "debug", skip(input))]
fn parse_string_literal(input: &str) -> ParserResult<Literal> {
    context(
        "string literal",
        alt((
            // Triple-quoted string literal
            parse_triple_quote_string,
            // Regular string literal
            parse_single_quote_string,
        )),
    )(input)
}

const TRIPLE_QUOTE: &str = "\"\"\"";
fn parse_triple_quote_string(input: &str) -> ParserResult<Literal> {
    // 開始のトリプルクォート、内容、終了のトリプルクォートをパース
    let (remaining, (_, content, _)) = context(
        "triple quote string",
        tuple((
            tag(TRIPLE_QUOTE),        // 開始の"""
            take_until(TRIPLE_QUOTE), // """まで全ての文字を取得
            tag(TRIPLE_QUOTE),        // 終了の"""
        )),
    )(input)?;
    println!("content: {}", content);
    println!("remaining: START{}EMD", remaining);

    let (_, lit) = context(
        "triple quote string",
        map(
            many0(alt((
                parse_newline,
                parse_interpolation,
                map(
                    take_while1(|c| c != '$' && c != '\n' && c != '\r'),
                    |content: &str| StringPart::Literal(content.to_string()),
                ),
            ))),
            StringLiteral::Triple,
        ),
    )(content)?;
    Ok((remaining, Literal::String(lit)))
}

#[tracing::instrument(level = "debug", skip(input))]
fn parse_single_quote_string(input: &str) -> ParserResult<Literal> {
    context(
        "single quote string",
        map(
            delimited(
                char('"'),
                many0(alt((parse_interpolation, parse_string_literal_part))),
                char('"'),
            ),
            |parts| Literal::String(StringLiteral::Single(parts)),
        ),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
fn parse_interpolation(input: &str) -> ParserResult<StringPart> {
    context(
        "string interpolation",
        map(
            delimited(
                tag("${"),
                take_while1(|c: char| c.is_alphanumeric() || c == '_'),
                tag("}"),
            ),
            |ident: &str| StringPart::Interpolation(ident.to_string()),
        ),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
fn parse_newline(input: &str) -> ParserResult<StringPart> {
    context(
        "newline",
        map(alt((tag("\r\n"), tag("\n"))), |_| StringPart::NewLine),
    )(input)
}
#[tracing::instrument(level = "debug", skip(input))]
fn parse_string_literal_part(input: &str) -> ParserResult<StringPart> {
    context(
        "string literal part",
        map(
            take_while1(|c| c != '$' && c != '\n' && c != '\r' && c != '"'),
            |content: &str| StringPart::Literal(content.to_string()),
        ),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
fn parse_float_literal(input: &str) -> ParserResult<Literal> {
    context(
        "float literal",
        map_res(
            recognize(tuple((opt(char('-')), digit1, char('.'), digit1))),
            |s: &str| s.parse::<f64>().map(Literal::Float),
        ),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
fn parse_integer_literal(input: &str) -> ParserResult<Literal> {
    context(
        "integer literal",
        map_res(recognize(pair(opt(char('-')), digit1)), |s: &str| {
            s.parse::<i64>().map(Literal::Integer)
        }),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
fn parse_boolean_literal(input: &str) -> ParserResult<Literal> {
    context(
        "boolean literal",
        alt((
            map(tag("true"), |_| Literal::Boolean(true)),
            map(tag("false"), |_| Literal::Boolean(false)),
        )),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_literal(input: &str) -> ParserResult<Token> {
    context(
        "literal",
        map(
            alt((
                parse_string_literal,
                parse_float_literal,
                parse_integer_literal,
                parse_boolean_literal,
            )),
            Token::Literal,
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 基本的な文字列リテラルのテスト
    mod basic_string_literals {
        use super::*;

        #[test]
        fn test_empty_string() {
            let input = "\"\"";
            let (rest, result) = parse_single_quote_string(input).unwrap();
            assert_eq!(rest, "");
            assert_eq!(result, Literal::String(StringLiteral::Single(vec![])));
        }

        #[test]
        fn test_simple_string() {
            let input = "\"hello world\"";
            let (rest, result) = parse_single_quote_string(input).unwrap();
            assert_eq!(rest, "");
            assert_eq!(
                result,
                Literal::String(StringLiteral::Single(vec![StringPart::Literal(
                    "hello world".to_string()
                )]))
            );
        }

        #[test]
        fn test_string_with_spaces() {
            let input = "\"   spaced content   \"";
            let (rest, result) = parse_single_quote_string(input).unwrap();
            assert_eq!(rest, "");
            assert_eq!(
                result,
                Literal::String(StringLiteral::Single(vec![StringPart::Literal(
                    "   spaced content   ".to_string()
                )]))
            );
        }
    }

    // 文字列補間のテスト
    mod string_interpolation {
        use super::*;

        #[test]
        fn test_simple_interpolation() {
            let input = "\"Hello ${name}\"";
            let (rest, result) = parse_single_quote_string(input).unwrap();
            assert_eq!(rest, "");
            assert_eq!(
                result,
                Literal::String(StringLiteral::Single(vec![
                    StringPart::Literal("Hello ".to_string()),
                    StringPart::Interpolation("name".to_string()),
                ]))
            );
        }

        #[test]
        fn test_multiple_interpolations() {
            let input = "\"${greeting} ${name}, Your total is: ${amount}\"";
            let (rest, result) = parse_single_quote_string(input).unwrap();
            assert_eq!(rest, "");
            assert_eq!(
                result,
                Literal::String(StringLiteral::Single(vec![
                    StringPart::Interpolation("greeting".to_string()),
                    StringPart::Literal(" ".to_string()),
                    StringPart::Interpolation("name".to_string()),
                    StringPart::Literal(", Your total is: ".to_string()),
                    StringPart::Interpolation("amount".to_string()),
                ]))
            );
        }
    }

    // トリプルクォート文字列のテスト
    mod triple_quoted_strings {
        use super::*;

        #[test]
        fn test_simple_triple_quote() {
            let input = "\"\"\"line one\"\"\"";
            let (rest, result) = parse_triple_quote_string(input).unwrap();
            assert_eq!(rest, "");
            assert_eq!(
                result,
                Literal::String(StringLiteral::Triple(vec![StringPart::Literal(
                    "line one".to_string()
                )]))
            );
        }

        #[test]
        fn test_multiline_triple_quote() {
            let input = "\"\"\"\
                line one\n\
                line two\n\
                line three\
                \"\"\"";
            let (rest, result) = parse_triple_quote_string(input).unwrap();
            assert_eq!(rest, "");
            assert_eq!(
                result,
                Literal::String(StringLiteral::Triple(vec![
                    StringPart::Literal("line one".to_string()),
                    StringPart::NewLine,
                    StringPart::Literal("line two".to_string()),
                    StringPart::NewLine,
                    StringPart::Literal("line three".to_string()),
                ]))
            );
        }

        #[test]
        fn test_triple_quote_with_interpolation() {
            let input = "\"\"\"Hello ${name},\nYour plan is ready\"\"\"";
            let (rest, result) = parse_triple_quote_string(input).unwrap();
            assert_eq!(rest, "");
            assert_eq!(
                result,
                Literal::String(StringLiteral::Triple(vec![
                    StringPart::Literal("Hello ".to_string()),
                    StringPart::Interpolation("name".to_string()),
                    StringPart::Literal(",".to_string()),
                    StringPart::NewLine,
                    StringPart::Literal("Your plan is ready".to_string()),
                ]))
            );
        }
    }

    // エッジケースとエラーケースのテスト
    mod edge_cases {
        use super::*;

        #[test]
        fn test_embedded_quotes() {
            let input = "\"\"\"Hello \"world\"\"\"";
            let (rest, result) = parse_triple_quote_string(input).unwrap();
            assert_eq!(rest, "");
            assert_eq!(
                result,
                Literal::String(StringLiteral::Triple(vec![StringPart::Literal(
                    "Hello \"world".to_string()
                ),]))
            );
        }

        #[test]
        fn test_escaped_characters() {
            let input = "\"Hello \\n World\\t!\"";
            let (rest, result) = parse_single_quote_string(input).unwrap();
            assert_eq!(rest, "");
            assert_eq!(
                result,
                Literal::String(StringLiteral::Single(vec![StringPart::Literal(
                    "Hello \\n World\\t!".to_string()
                )]))
            );
        }

        #[test]
        fn test_empty_interpolation() {
            let input = "\"${}\"";
            let result = parse_single_quote_string(input);
            assert!(result.is_err());
        }

        #[test]
        fn test_unterminated_string() {
            let input = "\"unclosed string";
            assert!(parse_single_quote_string(input).is_err());
        }
    }

    #[test]
    fn test_number_literals() {
        // Integer
        let (rest, result) = parse_integer_literal("123").unwrap();
        assert_eq!(result, Literal::Integer(123));
        assert_eq!(rest, "");

        // Negative integer
        let (rest, result) = parse_integer_literal("-123").unwrap();
        assert_eq!(result, Literal::Integer(-123));
        assert_eq!(rest, "");

        // Float
        let (rest, result) = parse_float_literal("123.45").unwrap();
        assert_eq!(result, Literal::Float(123.45));
        assert_eq!(rest, "");

        // Negative float
        let (rest, result) = parse_float_literal("-123.45").unwrap();
        assert_eq!(result, Literal::Float(-123.45));
        assert_eq!(rest, "");
    }
}
