//! # Symbol Token Handling
//!
//! This module defines the symbols (operators and delimiters) recognized by the KAIREI DSL
//! and provides functionality for parsing symbol tokens.
//!
//! ## Symbol Types
//!
//! Symbols are divided into two main categories:
//!
//! * [`Operator`]: Mathematical, logical, and special operators
//! * [`Delimiter`]: Structural elements like braces, parentheses, and punctuation
//!
//! ## Parsing Strategy
//!
//! Symbols are parsed using a longest-match approach to ensure that multi-character
//! operators like `=>` are correctly recognized instead of being parsed as separate
//! `=` and `>` tokens.
//!
//! ## Operator Precedence
//!
//! While the tokenizer itself doesn't enforce operator precedence (that's handled by the parser),
//! the order of operator matching in [`parse_operator`] ensures that longer operators are
//! matched before shorter ones to avoid ambiguity.

use strum_macros::{AsRefStr, Display, EnumString};

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    error::context,
};

use super::token::{ParserResult, Token};

/// Represents operators in the KAIREI DSL.
///
/// Operators are special symbols that perform operations on values and expressions.
/// They are categorized into function-related, access, comparison, arithmetic, and logical operators.
#[derive(Debug, Clone, PartialEq, EnumString, Display, AsRefStr)]
pub enum Operator {
    /// Function definition arrow (`=>`)
    #[strum(serialize = "=>")]
    Arrow,
    /// Return type arrow (`->`)
    #[strum(serialize = "->")]
    ThinArrow,

    /// Member access operator (`.`)
    #[strum(serialize = ".")]
    Dot,
    /// Namespace scope operator (`::`)
    #[strum(serialize = "::")]
    Scope,

    /// Equality comparison operator (`==`)
    #[strum(serialize = "==")]
    EqualEqual,
    /// Inequality comparison operator (`!=`)
    #[strum(serialize = "!=")]
    NotEqual,
    /// Greater than comparison operator (`>`)
    #[strum(serialize = ">")]
    Greater,
    /// Greater than or equal comparison operator (`>=`)
    #[strum(serialize = ">=")]
    GreaterEqual,
    /// Less than comparison operator (`<`)
    #[strum(serialize = "<")]
    Less,
    /// Less than or equal comparison operator (`<=`)
    #[strum(serialize = "<=")]
    LessEqual,

    /// Addition operator (`+`)
    #[strum(serialize = "+")]
    Plus,
    /// Subtraction operator (`-`)
    #[strum(serialize = "-")]
    Minus,
    /// Multiplication operator (`*`)
    #[strum(serialize = "*")]
    Multiply,
    /// Division operator (`/`)
    #[strum(serialize = "/")]
    Divide,

    /// Logical AND operator (`&&`)
    #[strum(serialize = "&&")]
    And,
    /// Logical OR operator (`||`)
    #[strum(serialize = "||")]
    Or,
    /// Logical NOT operator (`!`)
    #[strum(serialize = "!")]
    Not,
}

/// Constant for the close brace character, used because direct serialization in strum causes errors.
const CLOSE_BRACE: &str = "}";

/// Represents delimiters in the KAIREI DSL.
///
/// Delimiters are structural elements that define the boundaries of code blocks,
/// separate elements in lists, and mark the end of statements.
#[derive(Debug, Clone, PartialEq, EnumString, Display, AsRefStr)]
pub enum Delimiter {
    /// Opening brace (`{`) for blocks
    #[strum(serialize = "{")]
    OpenBrace,
    /// Closing brace (`}`) for blocks
    #[strum(serialize = "CLOSE_BRACE")]
    CloseBrace,
    /// Opening parenthesis (`(`) for grouping and function calls
    #[strum(serialize = "(")]
    OpenParen,
    /// Closing parenthesis (`)`) for grouping and function calls
    #[strum(serialize = ")")]
    CloseParen,
    /// Opening bracket (`[`) for arrays and indexing
    #[strum(serialize = "[")]
    OpenBracket,
    /// Closing bracket (`]`) for arrays and indexing
    #[strum(serialize = "]")]
    CloseBracket,
    /// Comma (`,`) for separating elements in lists
    #[strum(serialize = ",")]
    Comma,
    /// Semicolon (`;`) for terminating statements
    #[strum(serialize = ";")]
    Semicolon,
    /// Colon (`:`) for type annotations and key-value pairs
    #[strum(serialize = ":")]
    Colon,
    /// Equal sign (`=`) for assignment
    #[strum(serialize = "=")]
    Equal,
}

/// Parses an operator token from the input string.
///
/// This function attempts to match one of the defined operators at the current position
/// in the input string. It uses a longest-match approach to ensure that multi-character
/// operators are correctly recognized.
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
/// # use kairei::tokenizer::symbol::parse_operator;
/// # use kairei::tokenizer::token::Token;
/// # use kairei::tokenizer::symbol::Operator;
/// let input = "=> rest";
/// let (rest, token) = parse_operator(input).unwrap();
/// assert_eq!(token, Token::Operator(Operator::Arrow));
/// assert_eq!(rest, " rest");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_operator(input: &str) -> ParserResult<Token> {
    context(
        "operator",
        map(
            alt((
                // Multi-character operators (matched first for longest-match)
                value(Operator::Arrow, tag("=>")),
                value(Operator::ThinArrow, tag("->")),
                value(Operator::Scope, tag("::")),
                value(Operator::EqualEqual, tag("==")),
                value(Operator::NotEqual, tag("!=")),
                value(Operator::GreaterEqual, tag(">=")),
                value(Operator::LessEqual, tag("<=")),
                value(Operator::And, tag("&&")),
                value(Operator::Or, tag("||")),
                // Single-character operators
                value(Operator::Dot, tag(".")),
                value(Operator::Greater, tag(">")),
                value(Operator::Less, tag("<")),
                value(Operator::Plus, tag("+")),
                value(Operator::Minus, tag("-")),
                value(Operator::Multiply, tag("*")),
                value(Operator::Divide, tag("/")),
                value(Operator::Not, tag("!")),
            )),
            Token::Operator,
        ),
    )(input)
}

/// Parses a delimiter token from the input string.
///
/// This function attempts to match one of the defined delimiters at the current position
/// in the input string.
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
/// # use kairei::tokenizer::symbol::parse_delimiter;
/// # use kairei::tokenizer::token::Token;
/// # use kairei::tokenizer::symbol::Delimiter;
/// let input = "{ code }";
/// let (rest, token) = parse_delimiter(input).unwrap();
/// assert_eq!(token, Token::Delimiter(Delimiter::OpenBrace));
/// assert_eq!(rest, " code }");
/// ```
#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_delimiter(input: &str) -> ParserResult<Token> {
    context(
        "delimiter",
        map(
            alt((
                value(Delimiter::OpenBrace, tag("{")),
                value(Delimiter::CloseBrace, tag(CLOSE_BRACE)),
                value(Delimiter::OpenParen, tag("(")),
                value(Delimiter::CloseParen, tag(")")),
                value(Delimiter::OpenBracket, tag("[")),
                value(Delimiter::CloseBracket, tag("]")),
                value(Delimiter::Comma, tag(",")),
                value(Delimiter::Semicolon, tag(";")),
                value(Delimiter::Colon, tag(":")),
                value(Delimiter::Equal, tag("=")),
            )),
            Token::Delimiter,
        ),
    )(input)
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
            (">", Token::Operator(Operator::Greater)),
        ];

        for (input, expected) in test_cases.iter() {
            let (rest, token) = parse_operator(input).unwrap();
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
            ("=", Token::Delimiter(Delimiter::Equal)),
        ];

        for (input, expected) in test_cases.iter() {
            let (rest, token) = parse_delimiter(input).unwrap();
            assert_eq!(token, *expected);
            assert_eq!(rest, "");
        }
    }

    #[test]
    fn test_operator_precedence() {
        // ">="が">"として誤って解釈されないことを確認
        let (rest, token) = parse_operator(">=").unwrap();
        assert_eq!(token, Token::Operator(Operator::GreaterEqual));
        assert_eq!(rest, "");
    }
}
