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
    /// General parse error with message, found token, and position
    #[error("Parse error: {message}")]
    ParseError {
        /// Error message
        message: String,
        /// The token that was found
        found: String,
        /// Position information (line, column)
        position: (usize, usize),
        /// Full span information
        span: Option<crate::tokenizer::token::Span>,
    },
    /// Unexpected end of file
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    /// End of file
    #[error("EOF")]
    EOF,
    /// Unexpected token
    #[error("Unexpected")]
    Unexpected,
    /// No alternative matched
    #[error("No alternative")]
    NoAlternative,
    /// Explicit failure
    #[error("Fail: {0}")]
    Fail(String),
    /// Predicate error
    #[error("PredicateError")]
    PredicateError,
    /// Error with context
    #[error("WithContext: {message}, {inner}")]
    WithContext {
        /// Context message
        message: String,
        /// Inner error
        inner: Box<ParseError>,
        /// Full span information
        span: Option<crate::tokenizer::token::Span>,
    },
}
