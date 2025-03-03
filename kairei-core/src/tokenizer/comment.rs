//! # Comment Token Handling
//!
//! This module provides functionality for parsing comment tokens in the KAIREI DSL.
//!
//! ## Comment Types
//!
//! Four types of comments are supported:
//!
//! * **Line Comments**: `// Comment text`
//! * **Block Comments**: `/* Comment text */`
//! * **Documentation Line Comments**: `/// Documentation text`
//! * **Documentation Block Comments**: `/** Documentation text */`
//!
//! ## Comment Preservation
//!
//! Comments are preserved as tokens to enable:
//!
//! * Documentation generation
//! * Source code formatting
//! * IDE features like hover documentation
//!
//! ## Parsing Strategy
//!
//! Comments are parsed in order of specificity to ensure correct recognition:
//!
//! 1. Documentation block comments (`/**`)
//! 2. Documentation line comments (`///`)
//! 3. Block comments (`/*`)
//! 4. Line comments (`//`)

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::not_line_ending,
    combinator::map,
    error::context,
    sequence::{delimited, preceded},
};

use super::token::{CommentType, ParserResult, Token};

/// Parses a line comment from the input string.
///
/// Line comments start with `//` and continue until the end of the line.
/// The content of the comment is trimmed of leading and trailing whitespace.
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
/// # use kairei_core::tokenizer::comment::parse_line_comment;
/// # use kairei_core::tokenizer::token::{Token, CommentType};
/// let input = "// This is a comment\ncode";
/// let (rest, token) = parse_line_comment(input).unwrap();
/// assert_eq!(token, Token::Comment {
///     content: "This is a comment".to_string(),
///     comment_type: CommentType::Line,
/// });
/// assert_eq!(rest, "\ncode");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_line_comment(input: &str) -> ParserResult<Token> {
    context(
        "line comment",
        map(
            preceded(tag("//"), not_line_ending),
            |parse_comment: &str| Token::Comment {
                content: parse_comment.trim().to_string(),
                comment_type: CommentType::Line,
            },
        ),
    )(input)
}

/// Parses a block comment from the input string.
///
/// Block comments start with `/*` and end with `*/`. They can span multiple lines.
/// The content of the comment is preserved as-is, including whitespace and newlines.
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
/// # use kairei_core::tokenizer::comment::parse_block_comment;
/// # use kairei_core::tokenizer::token::{Token, CommentType};
/// let input = "/* This is a\n block comment */code";
/// let (rest, token) = parse_block_comment(input).unwrap();
/// assert_eq!(token, Token::Comment {
///     content: " This is a\n block comment ".to_string(),
///     comment_type: CommentType::Block,
/// });
/// assert_eq!(rest, "code");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_block_comment(input: &str) -> ParserResult<Token> {
    context(
        "block comment",
        map(
            delimited(tag("/*"), take_until("*/"), tag("*/")),
            |content: &str| Token::Comment {
                content: content.to_string(),
                comment_type: CommentType::Block,
            },
        ),
    )(input)
}

/// Parses a line documentation comment from the input string.
///
/// Line documentation comments start with `///` and continue until the end of the line.
/// They are used for generating documentation for the following item.
/// The content of the comment is trimmed of leading and trailing whitespace.
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
/// # use kairei_core::tokenizer::comment::parse_line_documentation_comment;
/// # use kairei_core::tokenizer::token::{Token, CommentType};
/// let input = "/// This is a doc comment\nfn test() {}";
/// let (rest, token) = parse_line_documentation_comment(input).unwrap();
/// assert_eq!(token, Token::Comment {
///     content: "This is a doc comment".to_string(),
///     comment_type: CommentType::DocumentationLine,
/// });
/// assert_eq!(rest, "\nfn test() {}");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_line_documentation_comment(input: &str) -> ParserResult<Token> {
    context(
        "line document comment",
        map(
            preceded(tag("///"), not_line_ending),
            |parse_comment: &str| Token::Comment {
                content: parse_comment.trim().to_string(),
                comment_type: CommentType::DocumentationLine,
            },
        ),
    )(input)
}

/// Parses a block documentation comment from the input string.
///
/// Block documentation comments start with `/**` and end with `*/`. They can span multiple lines.
/// They are used for generating documentation for the following item.
/// The content of the comment is preserved as-is, including whitespace and newlines.
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
/// # use kairei_core::tokenizer::comment::parse_block_documentation_comment;
/// # use kairei_core::tokenizer::token::{Token, CommentType};
/// let input = "/** This is a\n * documentation comment\n */code";
/// let (rest, token) = parse_block_documentation_comment(input).unwrap();
/// assert_eq!(token, Token::Comment {
///     content: " This is a\n * documentation comment\n ".to_string(),
///     comment_type: CommentType::DocumentationBlock,
/// });
/// assert_eq!(rest, "code");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_block_documentation_comment(input: &str) -> ParserResult<Token> {
    context(
        "block document comment",
        map(
            delimited(tag("/**"), take_until("*/"), tag("*/")),
            |content: &str| Token::Comment {
                content: content.to_string(),
                comment_type: CommentType::DocumentationBlock,
            },
        ),
    )(input)
}

/// Parses any type of comment from the input string.
///
/// This function attempts to match one of the four comment types in order of specificity:
/// 1. Block documentation comments (`/**`)
/// 2. Line documentation comments (`///`)
/// 3. Block comments (`/*`)
/// 4. Line comments (`//`)
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
/// # use kairei_core::tokenizer::comment::parse_comment;
/// # use kairei_core::tokenizer::token::{Token, CommentType};
/// let input = "// This is a comment\ncode";
/// let (rest, token) = parse_comment(input).unwrap();
/// assert_eq!(token, Token::Comment {
///     content: "This is a comment".to_string(),
///     comment_type: CommentType::Line,
/// });
/// assert_eq!(rest, "\ncode");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_comment(input: &str) -> ParserResult<Token> {
    context(
        "comment",
        alt((
            parse_block_documentation_comment, // Try /** */ first
            parse_line_documentation_comment,  // Then try ///
            parse_block_comment,               // Then try /* */
            parse_line_comment,                // Finally try //
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_comment() {
        let input = "// This is a line parse_comment\ncode";
        let (rest, token) = parse_comment(input).unwrap();
        assert_eq!(
            token,
            Token::Comment {
                content: "This is a line parse_comment".to_string(),
                comment_type: CommentType::Line,
            }
        );
        assert_eq!(rest, "\ncode");
    }

    #[test]
    fn test_block_comment() {
        let input = "/* This is a\n block parse_comment */code";
        let (rest, token) = parse_comment(input).unwrap();
        assert_eq!(
            token,
            Token::Comment {
                content: " This is a\n block parse_comment ".to_string(),
                comment_type: CommentType::Block,
            }
        );
        assert_eq!(rest, "code");
    }

    #[test]
    fn test_line_documentation_comment() {
        let input = "/// This is a doc parse_comment\nfn test() {}";
        let (rest, token) = parse_comment(input).unwrap();
        assert_eq!(
            token,
            Token::Comment {
                content: "This is a doc parse_comment".to_string(),
                comment_type: CommentType::DocumentationLine,
            }
        );
        assert_eq!(rest, "\nfn test() {}");
    }

    #[test]
    fn test_documentation_comment() {
        let input = "/** This is a\n * documentation parse_comment\n */code";
        let (rest, token) = parse_comment(input).unwrap();
        assert_eq!(
            token,
            Token::Comment {
                content: " This is a\n * documentation parse_comment\n ".to_string(),
                comment_type: CommentType::DocumentationBlock,
            }
        );
        assert_eq!(rest, "code");
    }

    #[test]
    fn test_nested_looking_comment() {
        let input = "/* outer /* not nested */ */";
        let (_, token) = parse_comment(input).unwrap();
        assert_eq!(
            token,
            Token::Comment {
                content: " outer /* not nested ".to_string(),
                comment_type: CommentType::Block,
            }
        );
    }
}
