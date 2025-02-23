use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
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
    /// Triple-quoted string with preserved formatting.
    ///
    /// Triple-quoted strings allow multiline content with preserved whitespace
    /// and proper handling of string interpolation. They are particularly useful
    /// in Think blocks where maintaining the exact formatting is important.
    ///
    /// Example:
    /// ```rust
    /// let s = """
    ///     Hello ${name},
    ///     This is a multiline string
    ///     with preserved indentation.
    ///     """
    /// ```
    TripleQuoted(Vec<StringPart>),
}

#[derive(Debug, Clone, PartialEq, strum::Display)]
pub enum Literal {
    String(Vec<StringPart>),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
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
/// Parses content within triple quotes, handling:
/// - Regular text content with preserved whitespace
/// - String interpolation (${...})
/// - Newlines with preserved indentation
fn parse_triple_quote_content(input: &str) -> ParserResult<StringPart> {
    alt((
        parse_interpolation,
        parse_newline,
        map(
            take_while1(|c| c != '$' && c != '\n' && c != '\r' && c != '"'),
            |content: &str| StringPart::Literal(content.to_string()),
        ),
    ))(input)
}

/// Parses a triple-quoted string, preserving all formatting including:
/// - Leading and trailing whitespace
/// - Indentation
/// - Newlines
/// - String interpolation
fn parse_triple_quote_string(input: &str) -> ParserResult<StringPart> {
    context(
        "triple quote string",
        map(
            delimited(
                tag("\"\"\""),
                map(
                    many0(alt((
                        parse_interpolation,
                        parse_newline,
                        map(
                            take_while1(|c| c != '$' && c != '\n' && c != '\r' && c != '"'),
                            |content: &str| StringPart::Literal(content.to_string()),
                        ),
                    ))),
                    |parts| {
                        let mut processed_parts = Vec::new();
                        let mut at_line_start = true;
                        let mut first_line = true;

                        for part in parts {
                            match part {
                                StringPart::Literal(s) => {
                                    if at_line_start {
                                        let content = s.trim_start().to_string();
                                        if !content.is_empty() {
                                            processed_parts.push(StringPart::Literal(content));
                                            at_line_start = false;
                                        }
                                    } else {
                                        processed_parts.push(StringPart::Literal(s));
                                    }
                                }
                                StringPart::NewLine => {
                                    if !first_line {
                                        processed_parts.push(StringPart::NewLine);
                                    }
                                    at_line_start = true;
                                    first_line = false;
                                }
                                StringPart::Interpolation(var) => {
                                    processed_parts.push(StringPart::Interpolation(var));
                                    at_line_start = false;
                                }
                                _ => {}
                            }
                        }
                        StringPart::TripleQuoted(processed_parts)
                    },
                ),
                tag("\"\"\""),
            ),
            |triple_quoted| triple_quoted,
        ),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
fn parse_string_literal(input: &str) -> ParserResult<Literal> {
    context(
        "string literal",
        map(
            alt((
                // Triple-quoted string literal
                map(parse_triple_quote_string, |triple_quoted| {
                    vec![triple_quoted]
                }),
                // Regular string literal
                delimited(
                    char('"'),
                    many0(alt((
                        parse_interpolation,
                        parse_newline,
                        parse_string_literal_part,
                    ))),
                    char('"'),
                ),
            )),
            Literal::String,
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

#[test]
fn test_triple_quote_string() {
    let input = "\"\"\"line one\nline two\nline three\"\"\"";
    let (rest, result) = parse_string_literal(input).unwrap();
    assert_eq!(rest, "");
    assert_eq!(
        result,
        Literal::String(vec![StringPart::TripleQuoted(vec![
            StringPart::Literal("line one".to_string()),
            StringPart::NewLine,
            StringPart::Literal("line two".to_string()),
            StringPart::NewLine,
            StringPart::Literal("line three".to_string()),
        ])])
    );
}

#[test]
fn test_triple_quote_with_interpolation() {
    let input = "\"\"\"Hello ${name},\nYour plan is ready\"\"\"";
    let (rest, result) = parse_string_literal(input).unwrap();
    assert_eq!(rest, "");
    assert_eq!(
        result,
        Literal::String(vec![StringPart::TripleQuoted(vec![
            StringPart::Literal("Hello ".to_string()),
            StringPart::Interpolation("name".to_string()),
            StringPart::Literal(",".to_string()),
            StringPart::NewLine,
            StringPart::Literal("Your plan is ready".to_string()),
        ])])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        let input = "\"hello world\"";
        let (rest, result) = parse_string_literal(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result,
            Literal::String(vec![StringPart::Literal("hello world".to_string()),])
        );
    }

    #[test]
    fn test_string_with_interpolation() {
        let input = "\"hello ${name}\"";
        let (rest, result) = parse_string_literal(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result,
            Literal::String(vec![
                StringPart::Literal("hello ".to_string()),
                StringPart::Interpolation("name".to_string()),
            ])
        );
    }

    #[test]
    fn test_multiline_string() {
        let input = "\"line one\nline two\nline three\"";
        let (rest, result) = parse_string_literal(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result,
            Literal::String(vec![
                StringPart::Literal("line one".to_string()),
                StringPart::NewLine,
                StringPart::Literal("line two".to_string()),
                StringPart::NewLine,
                StringPart::Literal("line three".to_string()),
            ])
        );
    }

    #[test]
    fn test_complex_string() {
        let input = "\"Hello ${name},\nYour total is: ${amount}\"";
        let (rest, result) = parse_string_literal(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result,
            Literal::String(vec![
                StringPart::Literal("Hello ".to_string()),
                StringPart::Interpolation("name".to_string()),
                StringPart::Literal(",".to_string()),
                StringPart::NewLine,
                StringPart::Literal("Your total is: ".to_string()),
                StringPart::Interpolation("amount".to_string()),
            ])
        );
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
