//! # Whitespace Token Handling
//!
//! This module provides functionality for parsing whitespace and newline tokens in the KAIREI DSL.
//!
//! ## Whitespace Preservation
//!
//! Unlike many tokenizers that discard whitespace, the KAIREI tokenizer preserves whitespace
//! as tokens to enable accurate source code formatting and reconstruction.
//!
//! ## Token Types
//!
//! Two types of whitespace tokens are recognized:
//!
//! * [`Token::Whitespace`]: Spaces and tabs
//! * [`Token::Newline`]: Line breaks (both `\n` and `\r\n`)
//!
//! ## Design Rationale
//!
//! Preserving whitespace as tokens enables:
//!
//! * Accurate source code formatting
//! * Precise error position reporting
//! * Formatting tools that maintain the original code style

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::map,
    error::context,
};

use super::token::{ParserResult, Token};

/// Parses whitespace (spaces and tabs) from the input string.
///
/// This function recognizes sequences of spaces and tabs and converts them
/// into a Whitespace token with the exact whitespace content preserved.
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
/// # use kairei::tokenizer::whitespace::parse_whitespace;
/// # use kairei::tokenizer::token::Token;
/// let input = "   hello";
/// let (rest, token) = parse_whitespace(input).unwrap();
/// assert_eq!(token, Token::Whitespace("   ".to_string()));
/// assert_eq!(rest, "hello");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_whitespace(input: &str) -> ParserResult<Token> {
    context(
        "whitespace expected",
        map(take_while1(|c| c == ' ' || c == '\t'), |ws: &str| {
            Token::Whitespace(ws.to_string())
        }),
    )(input)
}

/// Parses newline characters from the input string.
///
/// This function recognizes both Unix-style (`\n`) and Windows-style (`\r\n`)
/// line endings and converts them into a Newline token.
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
/// # use kairei::tokenizer::whitespace::parse_newline;
/// # use kairei::tokenizer::token::Token;
/// let input = "\nhello";
/// let (rest, token) = parse_newline(input).unwrap();
/// assert_eq!(token, Token::Newline);
/// assert_eq!(rest, "hello");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_newline(input: &str) -> ParserResult<Token> {
    context(
        "newline expected",
        map(alt((tag("\r\n"), tag("\n"))), |_| Token::Newline),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        let input = "   hello";
        let (rest, token) = parse_whitespace(input).unwrap();
        assert_eq!(token, Token::Whitespace("   ".to_string()));
        assert_eq!(rest, "hello");

        let input = "\t\t  hello";
        let (rest, token) = parse_whitespace(input).unwrap();
        assert_eq!(token, Token::Whitespace("\t\t  ".to_string()));
        assert_eq!(rest, "hello");
    }

    #[test]
    fn test_newline() {
        let input = "\nhello";
        let (rest, token) = parse_newline(input).unwrap();
        assert_eq!(token, Token::Newline);
        assert_eq!(rest, "hello");

        let input = "\r\nworld";
        let (rest, token) = parse_newline(input).unwrap();
        assert_eq!(token, Token::Newline);
        assert_eq!(rest, "world");
    }

    #[test]
    fn test_error() {
        let input = "hello";
        let result = parse_whitespace(input);
        assert!(result.is_err());

        let input = "hello";
        let result = parse_newline(input);
        assert!(result.is_err());
    }
}
