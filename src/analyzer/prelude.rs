//! # Parser Prelude
//!
//! This module provides convenient functions for creating parsers,
//! making the parser combinator API more ergonomic to use.
//!
//! The prelude exports factory functions for all parser combinators,
//! allowing for a more readable and concise parser definition syntax.

use super::combinators::*;
use super::core::Parser;
use super::error_handling::*;

/// Creates a parser that matches a specific value
///
/// # Arguments
///
/// * `value` - The value to match
///
/// # Returns
///
/// A parser that succeeds if the input equals the specified value
pub fn equal<I: Clone + PartialEq>(value: I) -> Equal<I> {
    Equal::new(value)
}

/// Creates a parser that expects a specific value after transformation
///
/// # Arguments
///
/// * `parser` - The parser to run
/// * `value` - The expected value to match against
///
/// # Returns
///
/// A parser that succeeds if the parsed value equals the expected value
pub fn expected<P, I, O>(parser: P, value: O) -> Expected<P, I, O>
where
    P: Parser<I, O>,
    I: Clone,
    O: Clone + PartialEq,
{
    Expected::new(parser, value)
}

/// Creates a parser that consumes and returns the current input token
///
/// # Returns
///
/// A parser that consumes one token from the input and returns it
pub fn identity<I: Clone>() -> Identity<I> {
    Identity::new()
}

/// Creates a parser that always succeeds with a constant value
///
/// # Arguments
///
/// * `zero_value` - The constant value to return
///
/// # Returns
///
/// A parser that always succeeds with the specified value without consuming input
pub fn zero<I, O: Clone>(zero_value: O) -> Zero<I, O> {
    Zero::new(zero_value)
}

/// Creates a parser that always fails with a specific message
///
/// # Arguments
///
/// * `message` - The error message
///
/// # Returns
///
/// A parser that always fails with the specified message
pub fn fail<I, O>(message: &str) -> Fail<I, O> {
    Fail::new(message)
}

/// Creates a parser that succeeds if the input satisfies a predicate function
///
/// # Arguments
///
/// * `f` - A function that takes a token and returns Some(value) if it satisfies
///         the predicate, or None otherwise
///
/// # Returns
///
/// A parser that applies the predicate function to the current input token
pub fn satisfy<I: Clone, O, F>(f: F) -> Satisfy<I, O, F>
where
    F: Fn(&I) -> Option<O>,
{
    Satisfy::new(f)
}

/// Creates a parser that tries multiple parsers and succeeds with the first successful one
///
/// # Arguments
///
/// * `parsers` - A vector of boxed parsers to try in order
///
/// # Returns
///
/// A parser that implements the logical OR operation for parsers
pub fn choice<I, O: Clone>(parsers: Vec<Box<dyn Parser<I, O>>>) -> Choice<I, O> {
    Choice::new(parsers)
}

/// Creates a parser that applies two parsers in sequence, returning only the second result
///
/// # Arguments
///
/// * `parser1` - The first parser to apply (result is discarded)
/// * `parser2` - The second parser to apply (result is returned)
///
/// # Returns
///
/// A parser that applies two parsers in sequence, returning only the second result
pub fn preceded<P1, P2, I, O>(parser1: P1, parser2: P2) -> Preceded<P1, P2, I, O>
where
    P1: Parser<I, ()>,
    P2: Parser<I, O>,
    I: Clone,
{
    Preceded::new(parser1, parser2)
}

/// Creates a parser that applies multiple parsers in sequence
///
/// # Arguments
///
/// * `parsers` - A vector of boxed parsers to apply in sequence
///
/// # Returns
///
/// A parser that implements the logical AND operation for parsers
pub fn sequence<I, O: Clone>(parsers: Vec<Box<dyn Parser<I, O>>>) -> Sequence<I, O> {
    Sequence::new(parsers)
}

/// Creates a parser that transforms the output of another parser using a function
///
/// # Arguments
///
/// * `parser` - The parser whose output will be transformed
/// * `f` - The transformation function to apply to the parser's output
///
/// # Returns
///
/// A parser that applies a transformation function to the result of another parser
pub fn map<P, F, A, B, I>(parser: P, f: F) -> Map<P, F, A, B>
where
    P: Parser<I, A>,
    F: Fn(A) -> B,
{
    Map::new(parser, f)
}

/// Creates a parser that discards the result of another parser
///
/// # Arguments
///
/// * `parser` - The parser whose result will be discarded
///
/// # Returns
///
/// A parser that returns unit (()) regardless of the inner parser's result
pub fn as_unit<I, O, P>(parser: P) -> AsUnit<P, O>
where
    P: Parser<I, O>,
{
    AsUnit::new(parser)
}

/// Creates a parser that applies another parser zero or more times
///
/// # Arguments
///
/// * `parser` - The parser to apply repeatedly
///
/// # Returns
///
/// A parser that collects results into a vector, always succeeding (possibly with empty vector)
pub fn many<P, I, O>(parser: P) -> Many<P, I, O>
where
    P: Parser<I, O>,
{
    Many::new(parser)
}

/// Creates an error-collecting many parser that preserves error information
///
/// # Arguments
///
/// * `parser` - The parser to apply repeatedly
/// * `context` - Context information for error reporting
///
/// # Returns
///
/// A parser that collects results into a vector, always succeeding (possibly with empty vector),
/// while collecting error information for better diagnostics
pub fn error_collecting_many<P, I, O>(
    parser: P,
    context: impl Into<String>,
) -> ErrorCollectingMany<P, I, O>
where
    P: Parser<I, O>,
{
    ErrorCollectingMany::new(parser, context)
}

/// Creates a parser that applies another parser one or more times
///
/// # Arguments
///
/// * `parser` - The parser to apply repeatedly
///
/// # Returns
///
/// A parser that collects results into a vector, failing if the inner parser never succeeds
pub fn many1<P, I, O>(parser: P) -> Many1<P, I, O>
where
    P: Parser<I, O>,
{
    Many1::new(parser)
}

/// Creates a parser for lists of items separated by a delimiter
///
/// # Arguments
///
/// * `item_parser` - Parser for list items
/// * `separator_parser` - Parser for the separator between items
///
/// # Returns
///
/// A parser that handles common list patterns like comma-separated values
pub fn separated_list<P, S, I, O>(item_parser: P, separator_parser: S) -> SeparatedList<P, S, I, O>
where
    P: Parser<I, O>,
    S: Parser<I, ()>,
{
    SeparatedList::new(item_parser, separator_parser)
}

/// Creates a parser that makes another parser optional
///
/// # Arguments
///
/// * `parser` - The parser to make optional
///
/// # Returns
///
/// A parser that returns Some(value) if the inner parser succeeds, or None if it fails
pub fn optional<P, I, O>(parser: P) -> Optional<P, I, O>
where
    P: Parser<I, O>,
{
    Optional::new(parser)
}

/// Creates an error-collecting optional parser that preserves error information
///
/// # Arguments
///
/// * `parser` - The parser to make optional
/// * `context` - Context information for error reporting
///
/// # Returns
///
/// A parser that returns Some(value) if the inner parser succeeds, or None if it fails,
/// while collecting error information for better diagnostics
pub fn error_collecting_optional<P, I, O>(
    parser: P,
    context: impl Into<String>,
) -> ErrorCollectingOptional<P, I, O>
where
    P: Parser<I, O>,
{
    ErrorCollectingOptional::new(parser, context)
}

/// Creates a parser for content between left and right delimiters
///
/// # Arguments
///
/// * `left` - Parser for the left delimiter
/// * `parser` - Parser for the content between delimiters
/// * `right` - Parser for the right delimiter
///
/// # Returns
///
/// A parser that handles common patterns like parenthesized expressions
pub fn delimited<L, P, R, I, O>(left: L, parser: P, right: R) -> Delimited<L, P, R, I, O>
where
    L: Parser<I, ()>,
    P: Parser<I, O>,
    R: Parser<I, ()>,
{
    Delimited::new(left, parser, right)
}

/// Creates a parser that applies two parsers in sequence and returns their results as a tuple
///
/// # Arguments
///
/// * `parser1` - The first parser to apply
/// * `parser2` - The second parser to apply
///
/// # Returns
///
/// A parser that returns a tuple of the two parser results
pub fn tuple2<P1, P2, I, O1, O2>(parser1: P1, parser2: P2) -> Tuple2<P1, P2, I, O1, O2>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
{
    Tuple2::new(parser1, parser2)
}

/// Creates a parser that applies three parsers in sequence and returns their results as a tuple
///
/// # Arguments
///
/// * `parser1` - The first parser to apply
/// * `parser2` - The second parser to apply
/// * `parser3` - The third parser to apply
///
/// # Returns
///
/// A parser that returns a tuple of the three parser results
pub fn tuple3<P1, P2, P3, I, O1, O2, O3>(
    parser1: P1,
    parser2: P2,
    parser3: P3,
) -> Tuple3<P1, P2, P3, I, O1, O2, O3>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
{
    Tuple3::new(parser1, parser2, parser3)
}

/// Creates a parser that applies four parsers in sequence and returns their results as a tuple
///
/// # Arguments
///
/// * `parser1` - The first parser to apply
/// * `parser2` - The second parser to apply
/// * `parser3` - The third parser to apply
/// * `parser4` - The fourth parser to apply
///
/// # Returns
///
/// A parser that returns a tuple of the four parser results
pub fn tuple4<P1, P2, P3, P4, I, O1, O2, O3, O4>(
    parser1: P1,
    parser2: P2,
    parser3: P3,
    parser4: P4,
) -> Tuple4<P1, P2, P3, P4, I, O1, O2, O3, O4>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
    P4: Parser<I, O4>,
{
    Tuple4::new(parser1, parser2, parser3, parser4)
}

/// Creates a parser that applies five parsers in sequence and returns their results as a tuple
///
/// # Arguments
///
/// * `parser1` - The first parser to apply
/// * `parser2` - The second parser to apply
/// * `parser3` - The third parser to apply
/// * `parser4` - The fourth parser to apply
/// * `parser5` - The fifth parser to apply
///
/// # Returns
///
/// A parser that returns a tuple of the five parser results
pub fn tuple5<P1, P2, P3, P4, P5, I, O1, O2, O3, O4, O5>(
    parser1: P1,
    parser2: P2,
    parser3: P3,
    parser4: P4,
    parser5: P5,
) -> Tuple5<P1, P2, P3, P4, P5, I, O1, O2, O3, O4, O5>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
    P4: Parser<I, O4>,
    P5: Parser<I, O5>,
{
    Tuple5::new(parser1, parser2, parser3, parser4, parser5)
}

/// Creates a parser that applies six parsers in sequence and returns their results as a tuple
///
/// # Arguments
///
/// * `parser1` - The first parser to apply
/// * `parser2` - The second parser to apply
/// * `parser3` - The third parser to apply
/// * `parser4` - The fourth parser to apply
/// * `parser5` - The fifth parser to apply
/// * `parser6` - The sixth parser to apply
///
/// # Returns
///
/// A parser that returns a tuple of the six parser results
// based on practical idiom for tuple
#[allow(clippy::type_complexity)]
pub fn tuple6<P1, P2, P3, P4, P5, P6, I, O1, O2, O3, O4, O5, O6>(
    parser1: P1,
    parser2: P2,
    parser3: P3,
    parser4: P4,
    parser5: P5,
    parser6: P6,
) -> Tuple6<P1, P2, P3, P4, P5, P6, I, O1, O2, O3, O4, O5, O6>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    P3: Parser<I, O3>,
    P4: Parser<I, O4>,
    P5: Parser<I, O5>,
    P6: Parser<I, O6>,
{
    Tuple6::new(parser1, parser2, parser3, parser4, parser5, parser6)
}

/// Creates a parser that adds context information to error messages
///
/// # Arguments
///
/// * `parser` - The parser to run
/// * `c` - The context information to add to error messages
///
/// # Returns
///
/// A parser that adds context information to error messages
pub fn with_context<P, I, O, C>(parser: P, c: C) -> WithContext<P, C>
where
    P: Parser<I, O>,
{
    WithContext::new(parser, c)
}

/// Creates a parser that lazily constructs another parser
///
/// # Arguments
///
/// * `f` - A function that constructs a parser when called
///
/// # Returns
///
/// A parser that defers parser construction until parsing time,
/// useful for recursive parser definitions
pub fn lazy<I, O, F, P>(f: F) -> Lazy<F>
where
    F: Fn() -> P,
    P: Parser<I, O>,
{
    Lazy::new(f)
}
