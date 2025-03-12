//! # Core Parser Definitions
//!
//! This module defines the fundamental parser interface and error types
//! that form the foundation of KAIREI's parser combinator system.

use thiserror::Error;

/// Parser trait defines the core parsing interface.
///
/// All parsers in the system implement this trait, which takes an input slice
/// and a position, and returns either a success result with a new position and
/// output value, or a parse error.
///
/// # Type Parameters
///
/// * `I` - The input token type
/// * `O` - The output value type
pub trait Parser<I, O> {
    /// Attempts to parse the input starting at the given position.
    ///
    /// # Arguments
    ///
    /// * `input` - The input token slice to parse
    /// * `pos` - The position to start parsing from
    ///
    /// # Returns
    ///
    /// * `Ok((new_pos, output))` - If parsing succeeds, returns the new position and the parsed value
    /// * `Err(error)` - If parsing fails, returns a ParseError
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O>;
}

/// Result type for parsing operations.
///
/// On success, returns a tuple of the new position and the parsed value.
/// On failure, returns a ParseError.
pub type ParseResult<O> = Result<(usize, O), ParseError>;

/// Error type for parsing operations.
///
/// Provides detailed information about parsing failures, including
/// error messages, position information, and context.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unexpected end of file
    #[error("Unexpected EOF: {message} at position {position}, context: {context:?}")]
    UnexpectedEOF {
        message: String,
        position: usize,
        context: Option<String>,
    },
    /// Unexpected token
    #[error(
        "Unexpected: Parsed value is not equal to, expected {expected}, parsed {parsed} at position {position}, context: {context:?}"
    )]
    Unexpected {
        expected: String,
        parsed: String,
        position: usize,
        context: Option<String>,
    },
    /// No alternative matched
    #[error(
        "No alternative: Parsed value is not matched any alternative at position {position}, context: {context:?}"
    )]
    NoAlternative {
        position: usize,
        context: Option<String>,
    },
    /// Explicit failure
    #[error("Failure: {message} at position {position}, context: {context:?}")]
    Failure {
        message: String,
        position: usize,
        context: Option<String>,
    },
}

impl ParseError {
    pub fn with_context(self, ctx: &str) -> Self {
        match self {
            ParseError::UnexpectedEOF {
                message,
                position,
                context,
            } => ParseError::UnexpectedEOF {
                message,
                position,
                context: context.map(|c| format!("{} -> {}", c, ctx)),
            },
            ParseError::Unexpected {
                expected,
                parsed,
                position,
                context,
            } => ParseError::Unexpected {
                expected,
                parsed,
                position,
                context: context.map(|c| format!("{} -> {}", c, ctx)),
            },
            ParseError::NoAlternative { position, context } => ParseError::NoAlternative {
                position,
                context: context.map(|c| format!("{} -> {}", c, ctx)),
            },
            ParseError::Failure {
                message,
                position,
                context,
            } => ParseError::Failure {
                message,
                position,
                context: context.map(|c| format!("{} -> {}", c, ctx)),
            },
        }
    }

    pub fn get_position(&self) -> usize {
        match self {
            ParseError::UnexpectedEOF { position, .. } => *position,
            ParseError::Unexpected { position, .. } => *position,
            ParseError::NoAlternative { position, .. } => *position,
            ParseError::Failure { position, .. } => *position,
        }
    }
}
