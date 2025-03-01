//! # Error Handling for Parser Combinators
//!
//! This module provides enhanced error handling capabilities for the parser combinators.
//! It includes error collection mechanisms and improved combinators that preserve error
//! information instead of silently discarding it.

use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;

use super::core::{ParseError, ParseResult, Parser};

/// Information about a parse error
#[derive(Debug, Clone)]
pub struct ParseErrorInfo {
    /// The actual parse error
    pub error: ParseError,
    /// Context information about where the error occurred
    pub context: String,
    /// Whether the error occurred in an optional parsing context
    pub is_optional: bool,
}

impl fmt::Display for ParseErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_type = if self.is_optional { "Optional" } else { "Repeated" };
        write!(
            f,
            "{} parsing failed in '{}': {}",
            error_type, self.context, self.error
        )
    }
}

/// Collects parse errors during parsing
#[derive(Debug, Default)]
pub struct ParseErrorCollector {
    errors: Vec<ParseErrorInfo>,
}

impl ParseErrorCollector {
    /// Creates a new empty error collector
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Adds an error to the collection
    pub fn add_error(&mut self, error_info: ParseErrorInfo) {
        self.errors.push(error_info);
    }

    /// Returns true if the collector contains any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns a slice of all collected errors
    pub fn get_errors(&self) -> &[ParseErrorInfo] {
        &self.errors
    }

    /// Clears all collected errors
    pub fn clear(&mut self) {
        self.errors.clear();
    }
}

// Thread-local storage for the error collector
thread_local! {
    pub static ERROR_COLLECTOR: RefCell<ParseErrorCollector> = RefCell::new(ParseErrorCollector::new());
}

/// An Optional combinator that collects errors instead of discarding them
pub struct ErrorCollectingOptional<P, I, O> {
    parser: P,
    context: String,
    _phantom: PhantomData<(I, O)>,
}

impl<P, I, O> ErrorCollectingOptional<P, I, O> {
    /// Creates a new ErrorCollectingOptional combinator
    pub fn new(parser: P, context: impl Into<String>) -> Self {
        Self {
            parser,
            context: context.into(),
            _phantom: PhantomData,
        }
    }
}

impl<I, O, P> Parser<I, Option<O>> for ErrorCollectingOptional<P, I, O>
where
    P: Parser<I, O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<Option<O>> {
        match self.parser.parse(input, pos) {
            Ok((new_pos, value)) => Ok((new_pos, Some(value))),
            Err(err) => {
                // Store the error in the thread-local collector
                ERROR_COLLECTOR.with(|collector| {
                    let mut collector = collector.borrow_mut();
                    collector.add_error(ParseErrorInfo {
                        error: err,
                        context: self.context.clone(),
                        is_optional: true,
                    });
                });
                // Still return Ok with None, but we've preserved the error
                Ok((pos, None))
            }
        }
    }
}

/// A Many combinator that collects errors instead of silently stopping
pub struct ErrorCollectingMany<P, I, O> {
    parser: P,
    context: String,
    _phantom: PhantomData<(I, O)>,
}

impl<P, I, O> ErrorCollectingMany<P, I, O> {
    /// Creates a new ErrorCollectingMany combinator
    pub fn new(parser: P, context: impl Into<String>) -> Self {
        Self {
            parser,
            context: context.into(),
            _phantom: PhantomData,
        }
    }
}

impl<I, O, P> Parser<I, Vec<O>> for ErrorCollectingMany<P, I, O>
where
    P: Parser<I, O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<Vec<O>> {
        let mut results = Vec::new();
        let mut current_pos = pos;
        let mut had_error = false;

        while !had_error && current_pos < input.len() {
            match self.parser.parse(input, current_pos) {
                Ok((new_pos, value)) => {
                    // If we didn't make progress, break to avoid infinite loop
                    if new_pos == current_pos {
                        break;
                    }
                    results.push(value);
                    current_pos = new_pos;
                }
                Err(err) => {
                    // Store the error but continue parsing
                    ERROR_COLLECTOR.with(|collector| {
                        let mut collector = collector.borrow_mut();
                        collector.add_error(ParseErrorInfo {
                            error: err,
                            context: self.context.clone(),
                            is_optional: false,
                        });
                    });
                    had_error = true;
                }
            }
        }

        Ok((current_pos, results))
    }
}

/// Creates an Optional combinator that collects errors
pub fn error_collecting_optional<I, O, P>(
    parser: P,
    context: impl Into<String>,
) -> ErrorCollectingOptional<P, I, O>
where
    P: Parser<I, O>,
{
    ErrorCollectingOptional::new(parser, context)
}

/// Creates a Many combinator that collects errors
pub fn error_collecting_many<I, O, P>(
    parser: P,
    context: impl Into<String>,
) -> ErrorCollectingMany<P, I, O>
where
    P: Parser<I, O>,
{
    ErrorCollectingMany::new(parser, context)
}

/// Formats a detailed error message including collected errors
pub fn format_detailed_error_message(
    main_error: &ParseError,
    collected_errors: &[ParseErrorInfo],
) -> String {
    let mut message = format!("Parse error: {}\n", main_error);
    
    // Add information about optional and many errors
    if !collected_errors.is_empty() {
        message.push_str("\nAdditional parsing issues:\n");
        
        for (i, error_info) in collected_errors.iter().enumerate() {
            message.push_str(&format!(
                "{}. {}\n",
                i + 1,
                error_info
            ));
        }
    }
    
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::core::*;
    use crate::analyzer::prelude::*;

    #[test]
    fn test_error_collecting_optional() {
        // Create a parser that always fails
        let fail_parser = fail::<char, i32>("test failure");
        let parser = error_collecting_optional(fail_parser, "test context");
        
        // Clear any previous errors
        ERROR_COLLECTOR.with(|collector| {
            collector.borrow_mut().clear();
        });
        
        // Parse should return None but collect the error
        let input = vec!['a', 'b', 'c'];
        let result = parser.parse(&input, 0);
        assert_eq!(result, Ok((0, None)));
        
        // Check that the error was collected
        ERROR_COLLECTOR.with(|collector| {
            let collector = collector.borrow();
            assert!(collector.has_errors());
            assert_eq!(collector.get_errors().len(), 1);
            
            let error_info = &collector.get_errors()[0];
            assert_eq!(error_info.context, "test context");
            assert!(error_info.is_optional);
            match &error_info.error {
                ParseError::Fail(msg) => assert_eq!(msg, "test failure"),
                _ => panic!("Unexpected error type"),
            }
        });
    }

    #[test]
    fn test_error_collecting_many() {
        // Create a parser that succeeds once then fails
        let input = vec!['a', 'b', 'c'];
        
        let parser = error_collecting_many(
            satisfy(|c: &char| if *c == 'a' { Some(*c) } else { None }),
            "test context"
        );
        
        // Clear any previous errors
        ERROR_COLLECTOR.with(|collector| {
            collector.borrow_mut().clear();
        });
        
        // Parse should return ['a'] and collect an error for 'b'
        let result = parser.parse(&input, 0);
        assert_eq!(result, Ok((1, vec!['a'])));
        
        // Check that the error was collected
        ERROR_COLLECTOR.with(|collector| {
            let collector = collector.borrow();
            assert!(collector.has_errors());
            assert_eq!(collector.get_errors().len(), 1);
            
            let error_info = &collector.get_errors()[0];
            assert_eq!(error_info.context, "test context");
            assert!(!error_info.is_optional);
            assert!(matches!(error_info.error, ParseError::EOF));
        });
    }
}
