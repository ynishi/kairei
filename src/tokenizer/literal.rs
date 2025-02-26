//! # Literal Token Handling
//!
//! This module provides functionality for parsing literal tokens in the KAIREI DSL,
//! including strings, numbers, and boolean values.
//!
//! ## Literal Types
//!
//! The following literal types are supported:
//!
//! * **String Literals**: Both single-quoted (`"text"`) and triple-quoted (`"""text"""`)
//! * **Numeric Literals**: Integers (`42`) and floating-point numbers (`3.14`)
//! * **Boolean Literals**: `true` and `false`
//! * **Null Literal**: `null`
//!
//! ## String Interpolation
//!
//! String literals support interpolation using the `${variable}` syntax:
//!
//! ```no_run
//! let example = "Hello, ${name}!";
//! ```
//!
//! ## Triple-Quoted Strings
//!
//! Triple-quoted strings (`"""`) preserve formatting and can span multiple lines:
//!
//! ```ignore
//! let example = """
//! This is a multi-line string
//! with preserved formatting.
//! """;
//! ```
//!
//! ## Parsing Strategy
//!
//! Literals are parsed using specialized parsers for each type:
//!
//! * [`parse_string_literal`]: Handles both single and triple-quoted strings
//! * [`parse_float_literal`]: Parses floating-point numbers
//! * [`parse_integer_literal`]: Parses integer numbers
//! * [`parse_boolean_literal`]: Parses boolean values

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

/// Represents a part of a string literal.
///
/// String literals in KAIREI can contain regular text, interpolated variables,
/// and newlines, each represented by a different variant of this enum.
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    /// A literal string segment without any interpolation or special formatting
    Literal(String),
    /// A string interpolation segment containing a variable name to be replaced
    Interpolation(String),
    /// A newline character in the string
    NewLine,
}

/// Represents a string literal in the KAIREI DSL.
///
/// KAIREI supports two types of string literals:
/// - Single-quoted strings with interpolation support
/// - Triple-quoted strings with preserved formatting and multi-line support
#[derive(Debug, Clone, PartialEq)]
pub enum StringLiteral {
    /// Single-quoted string with interpolation support
    Single(Vec<StringPart>),
    /// Triple-quoted string with preserved formatting
    Triple(Vec<StringPart>),
}

/// Represents a literal value in the KAIREI DSL.
///
/// Literals are constant values that appear directly in the source code,
/// such as strings, numbers, booleans, and null.
#[derive(Debug, Clone, PartialEq, strum::Display)]
pub enum Literal {
    /// String literal, either single-quoted or triple-quoted
    String(StringLiteral),
    /// Integer numeric literal
    Integer(i64),
    /// Floating-point numeric literal
    Float(f64),
    /// Boolean literal (`true` or `false`)
    Boolean(bool),
    /// Null literal
    Null,
}

/// Parses a string literal from the input string.
///
/// This function attempts to parse either a triple-quoted string or a single-quoted string.
/// Triple-quoted strings are tried first to ensure correct parsing of strings that start with
/// triple quotes.
///
/// # Arguments
///
/// * `input` - The input string to parse
///
/// # Returns
///
/// * `ParserResult<Literal>` - A result containing either the parsed literal and remaining input,
///   or an error if parsing fails
///
/// Note: This is a private function used internally by the tokenizer.
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

/// Constant for the triple quote delimiter used in multi-line strings.
const TRIPLE_QUOTE: &str = "\"\"\"";

/// Parses a triple-quoted string from the input string.
///
/// Triple-quoted strings are delimited by `"""` and can span multiple lines.
/// They preserve formatting and support variable interpolation.
///
/// # Arguments
///
/// * `input` - The input string to parse
///
/// # Returns
///
/// * `ParserResult<Literal>` - A result containing either the parsed literal and remaining input,
///   or an error if parsing fails
///
/// # Examples
///
/// ```
/// # use kairei::tokenizer::literal::{parse_triple_quote_string, StringLiteral, StringPart, Literal};
/// let input = "\"\"\"Hello ${name}\nWelcome!\"\"\"";
/// let (rest, literal) = parse_triple_quote_string(input).unwrap();
/// ```
pub fn parse_triple_quote_string(input: &str) -> ParserResult<Literal> {
    // Parse the opening triple quote, content, and closing triple quote
    let (remaining, (_, content, _)) = context(
        "triple quote string",
        tuple((
            tag(TRIPLE_QUOTE),        // Opening """
            take_until(TRIPLE_QUOTE), // Get all characters until """
            tag(TRIPLE_QUOTE),        // Closing """
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

/// Parses a single-quoted string from the input string.
///
/// Single-quoted strings are delimited by double quotes (`"`) and support variable interpolation.
/// They cannot span multiple lines unless the newline is part of an interpolated variable.
///
/// # Arguments
///
/// * `input` - The input string to parse
///
/// # Returns
///
/// * `ParserResult<Literal>` - A result containing either the parsed literal and remaining input,
///   or an error if parsing fails
///
/// # Examples
///
/// ```
/// # use kairei::tokenizer::literal::{parse_single_quote_string, StringLiteral, StringPart, Literal};
/// let input = "\"Hello ${name}!\"";
/// let (rest, literal) = parse_single_quote_string(input).unwrap();
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_single_quote_string(input: &str) -> ParserResult<Literal> {
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

/// Parses a string interpolation from the input string.
///
/// String interpolations are delimited by `${` and `}` and contain a variable name
/// that will be replaced with its value at runtime.
///
/// # Arguments
///
/// * `input` - The input string to parse
///
/// # Returns
///
/// * `ParserResult<StringPart>` - A result containing either the parsed string part and remaining input,
///   or an error if parsing fails
///
/// # Examples
///
/// ```
/// # use kairei::tokenizer::literal::{parse_interpolation, StringPart};
/// let input = "${name} rest";
/// let (rest, part) = parse_interpolation(input).unwrap();
/// assert_eq!(part, StringPart::Interpolation("name".to_string()));
/// assert_eq!(rest, " rest");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_interpolation(input: &str) -> ParserResult<StringPart> {
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

/// Parses a newline character from the input string.
///
/// This function recognizes both Unix-style (`\n`) and Windows-style (`\r\n`)
/// line endings and converts them into a NewLine string part.
///
/// # Arguments
///
/// * `input` - The input string to parse
///
/// # Returns
///
/// * `ParserResult<StringPart>` - A result containing either the parsed string part and remaining input,
///   or an error if parsing fails
///
/// Note: This is a private function used internally by the tokenizer.
#[tracing::instrument(level = "debug", skip(input))]
fn parse_newline(input: &str) -> ParserResult<StringPart> {
    context(
        "newline",
        map(alt((tag("\r\n"), tag("\n"))), |_| StringPart::NewLine),
    )(input)
}

/// Parses a literal string part from the input string.
///
/// A literal string part is a sequence of characters that doesn't contain
/// interpolation markers, newlines, or string delimiters.
///
/// # Arguments
///
/// * `input` - The input string to parse
///
/// # Returns
///
/// * `ParserResult<StringPart>` - A result containing either the parsed string part and remaining input,
///   or an error if parsing fails
///
/// Note: This is a private function used internally by the tokenizer.
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

/// Parses a floating-point number from the input string.
///
/// Floating-point numbers consist of an optional negative sign,
/// one or more digits, a decimal point, and one or more digits after the decimal point.
///
/// # Arguments
///
/// * `input` - The input string to parse
///
/// # Returns
///
/// * `ParserResult<Literal>` - A result containing either the parsed literal and remaining input,
///   or an error if parsing fails
///
/// # Examples
///
/// ```
/// # use kairei::tokenizer::literal::{parse_float_literal, Literal};
/// let input = "3.14159";
/// let (rest, literal) = parse_float_literal(input).unwrap();
/// assert_eq!(literal, Literal::Float(3.14159));
/// assert_eq!(rest, "");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_float_literal(input: &str) -> ParserResult<Literal> {
    context(
        "float literal",
        map_res(
            recognize(tuple((opt(char('-')), digit1, char('.'), digit1))),
            |s: &str| s.parse::<f64>().map(Literal::Float),
        ),
    )(input)
}

/// Parses an integer number from the input string.
///
/// Integer numbers consist of an optional negative sign followed by one or more digits.
///
/// # Arguments
///
/// * `input` - The input string to parse
///
/// # Returns
///
/// * `ParserResult<Literal>` - A result containing either the parsed literal and remaining input,
///   or an error if parsing fails
///
/// # Examples
///
/// ```
/// # use kairei::tokenizer::literal::{parse_integer_literal, Literal};
/// let input = "42";
/// let (rest, literal) = parse_integer_literal(input).unwrap();
/// assert_eq!(literal, Literal::Integer(42));
/// assert_eq!(rest, "");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_integer_literal(input: &str) -> ParserResult<Literal> {
    context(
        "integer literal",
        map_res(recognize(pair(opt(char('-')), digit1)), |s: &str| {
            s.parse::<i64>().map(Literal::Integer)
        }),
    )(input)
}

/// Parses a boolean value from the input string.
///
/// Boolean values are either `true` or `false`.
///
/// # Arguments
///
/// * `input` - The input string to parse
///
/// # Returns
///
/// * `ParserResult<Literal>` - A result containing either the parsed literal and remaining input,
///   or an error if parsing fails
///
/// # Examples
///
/// ```
/// # use kairei::tokenizer::literal::{parse_boolean_literal, Literal};
/// let input = "true";
/// let (rest, literal) = parse_boolean_literal(input).unwrap();
/// assert_eq!(literal, Literal::Boolean(true));
/// assert_eq!(rest, "");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_boolean_literal(input: &str) -> ParserResult<Literal> {
    context(
        "boolean literal",
        alt((
            map(tag("true"), |_| Literal::Boolean(true)),
            map(tag("false"), |_| Literal::Boolean(false)),
        )),
    )(input)
}

/// Parses any type of literal from the input string.
///
/// This function attempts to match one of the supported literal types:
/// - String literals (both single and triple-quoted)
/// - Floating-point numbers
/// - Integer numbers
/// - Boolean values
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
/// # use kairei::tokenizer::literal::parse_literal;
/// # use kairei::tokenizer::token::Token;
/// # use kairei::tokenizer::literal::{Literal, StringLiteral, StringPart};
/// let input = "\"Hello, world!\"";
/// let (rest, token) = parse_literal(input).unwrap();
/// assert_eq!(rest, "");
/// ```
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
