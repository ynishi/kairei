//! # Core Token Types and Tokenizer Implementation
//!
//! This module defines the fundamental token types and the main tokenizer implementation
//! for the KAIREI DSL.
//!
//! ## Token Structure
//!
//! The token system consists of:
//!
//! * [`Token`]: The core token enum representing different token types
//! * [`TokenSpan`]: A token with position information for error reporting
//! * [`Tokenizer`]: The main tokenizer implementation
//!
//! ## Design Rationale
//!
//! The tokenizer is designed to:
//!
//! * Provide detailed position information for error reporting
//! * Support incremental tokenization for large inputs
//! * Preserve formatting information for accurate source reconstruction
//! * Enable efficient error recovery during parsing
//!
//! ## Error Handling
//!
//! The [`TokenizerError`] type provides detailed error information, including:
//!
//! * Error message with context
//! * Position information (line, column, start/end)
//! * Found text for better error diagnostics

use std::fmt;

use nom::{
    branch::alt,
    bytes::complete::{take_while, take_while1},
    combinator::recognize,
    error::{context, VerboseError},
    sequence::pair,
    IResult,
};
use thiserror::Error;

use super::{
    comment::parse_comment,
    keyword::{parse_keyword, Keyword},
    literal::{parse_literal, Literal},
    symbol::{parse_delimiter, parse_operator, Delimiter, Operator},
    whitespace::{parse_newline, parse_whitespace},
};

/// Represents a token in the KAIREI DSL.
///
/// Tokens are the smallest units of meaning in the language, categorized into
/// keywords, identifiers, symbols, literals, and formatting elements.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Language keywords like `micro`, `world`, `state`, etc.
    Keyword(Keyword),
    /// User-defined identifiers for variables, functions, etc.
    Identifier(String),
    /// Operators like `+`, `-`, `==`, etc.
    Operator(Operator),
    /// Delimiters like `{`, `}`, `,`, etc.
    Delimiter(Delimiter),
    /// Literal values like strings, numbers, booleans.
    Literal(Literal),
    /// Whitespace (spaces and tabs).
    Whitespace(String),
    /// Line breaks.
    Newline,
    /// Comments and documentation.
    Comment {
        /// The content of the comment.
        content: String,
        /// The type of comment (line, block, documentation).
        comment_type: CommentType,
    },
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Keyword(kw) => write!(f, "Token::Keyword({})", kw),
            Token::Identifier(id) => write!(f, "Token::Identifier({})", id),
            Token::Operator(op) => write!(f, "Token::Operator({})", op),
            Token::Delimiter(d) => write!(f, "Token::Delimiter({})", d),
            Token::Literal(lit) => write!(f, "Token::Literal({})", lit),
            Token::Whitespace(ws) => write!(f, "Token::Whitespace({})", ws),
            Token::Newline => write!(f, "Newline"),
            Token::Comment {
                content,
                comment_type,
            } => {
                let comment_type = match comment_type {
                    CommentType::Line => "//",
                    CommentType::Block => "/*",
                    CommentType::DocumentationLine => "///",
                    CommentType::DocumentationBlock => "/**",
                };
                write!(
                    f,
                    "Token::Comment{{type: {}, content: {}}}",
                    comment_type, content
                )
            }
        }
    }
}

impl Token {
    /// Returns true if the token is whitespace.
    ///
    /// Used to filter out whitespace tokens when only semantic tokens are needed.
    pub fn is_whitespace(&self) -> bool {
        matches!(self, Token::Whitespace(_))
    }

    /// Returns true if the token is a comment.
    ///
    /// Used to filter out comment tokens when only semantic tokens are needed.
    pub fn is_comment(&self) -> bool {
        matches!(self, Token::Comment { .. })
    }

    /// Returns true if the token is a newline.
    ///
    /// Used to filter out newline tokens when only semantic tokens are needed.
    pub fn is_newline(&self) -> bool {
        matches!(self, Token::Newline)
    }
}

/// Represents the different types of comments in the KAIREI DSL.
///
/// Comments are preserved during tokenization to enable documentation generation
/// and source code formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentType {
    /// Line comment (`//`).
    Line,
    /// Block comment (`/* */`).
    Block,
    /// Documentation line comment (`///`).
    DocumentationLine,
    /// Documentation block comment (`/** */`).
    DocumentationBlock,
}

/// The main tokenizer for the KAIREI DSL.
///
/// The tokenizer transforms raw text into a stream of tokens with position information,
/// enabling accurate error reporting and source code reconstruction.
#[derive(Debug, Clone)]
pub struct Tokenizer {
    /// Current position in the input string.
    current_position: usize,
    /// Current line number (1-based).
    current_line: usize,
    /// Current column number (1-based).
    current_column: usize,
}

/// Default implementation for Tokenizer that calls new().
impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer {
    /// Creates a new Tokenizer with initial position at the start of the input.
    pub fn new() -> Self {
        Self {
            current_position: 0,
            current_line: 1,   // 1-based
            current_column: 1, // 1-based
        }
    }

    /// Tokenizes the input string into a sequence of tokens with position information.
    ///
    /// This is the main entry point for the tokenizer. It processes the input string
    /// character by character, recognizing tokens and tracking position information.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to tokenize
    ///
    /// # Returns
    ///
    /// * `TokenizerResult<Vec<TokenSpan>>` - A result containing either a vector of token spans
    ///   or a TokenizerError if tokenization fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use kairei::tokenizer::token::{Tokenizer, Token};
    /// let mut tokenizer = Tokenizer::new();
    /// let input = "micro Agent { state { count: Int = 0 } }";
    /// let tokens = tokenizer.tokenize(input).unwrap();
    /// ```
    #[tracing::instrument(level = "debug", skip(input))]
    pub fn tokenize(&mut self, input: &str) -> TokenizerResult<Vec<TokenSpan>> {
        let mut tokens = Vec::new();
        let mut remaining = input;

        while !remaining.is_empty() {
            let start_position = self.current_position;
            let start_line = self.current_line;
            let start_column = self.current_column;

            let result = alt((
                // Formatting
                parse_whitespace,
                parse_newline,
                // Literals
                parse_literal,
                // Comments
                parse_comment,
                // Code elements
                parse_keyword,
                parse_operator,
                parse_delimiter,
                parse_identifier,
            ))(remaining);

            match result {
                Ok((new_remaining, token)) => {
                    let consumed = &remaining[..(remaining.len() - new_remaining.len())];
                    self.update_position(consumed);

                    tokens.push(TokenSpan {
                        token,
                        start: start_position,
                        end: self.current_position,
                        line: start_line,
                        column: start_column,
                    });

                    remaining = new_remaining;
                }
                Err(e) => {
                    let found = remaining.chars().take(20).collect::<String>();
                    let span = Span {
                        start: self.current_position,
                        end: self.current_position + 1,
                        line: self.current_line,
                        column: self.current_column,
                    };
                    let error = match e {
                        nom::Err::Incomplete(e) => TokenizerError::ParseError {
                            message: format!("Incomplete input, {:?}", e),
                            found,
                            span,
                        },
                        nom::Err::Error(e) | nom::Err::Failure(e) => TokenizerError::ParseError {
                            message: nom::error::convert_error(remaining, e).to_string(),
                            found,
                            span,
                        },
                    };
                    tracing::error!("{}", error);
                    return Err(error);
                }
            }
        }

        Ok(tokens)
    }

    /// Updates the current position, line, and column based on the consumed text.
    ///
    /// This method is called after each token is recognized to update the position
    /// information for the next token.
    ///
    /// # Arguments
    ///
    /// * `text` - The text that was consumed by the last token
    fn update_position(&mut self, text: &str) {
        for c in text.chars() {
            self.current_position += c.len_utf8();
            if c == '\n' {
                self.current_line += 1;
                self.current_column = 1;
            } else {
                self.current_column += 1;
            }
        }
    }
}

/// A token with position information for error reporting and source mapping.
///
/// TokenSpan combines a token with its position in the source code, enabling
/// precise error reporting and source reconstruction.
#[derive(Debug, Clone)]
pub struct TokenSpan {
    /// The token itself.
    pub token: Token,
    /// Start position in the input string (byte offset).
    pub start: usize,
    /// End position in the input string (byte offset).
    pub end: usize,
    /// Line number where the token starts (1-based).
    pub line: usize,
    /// Column number where the token starts (1-based).
    pub column: usize,
}

/// Represents a span of text in the source code.
///
/// Used for error reporting and source mapping to provide precise location
/// information for syntax errors.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    /// Start position in the input string (byte offset).
    pub start: usize,
    /// End position in the input string (byte offset).
    pub end: usize,
    /// Line number where the span starts (1-based).
    pub line: usize,
    /// Column number where the span starts (1-based).
    pub column: usize,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line: {}, column: {}, start: {}, end: {}",
            self.line, self.column, self.start, self.end
        )
    }
}

/// Parses an identifier or keyword from the input string.
///
/// An identifier starts with a letter or underscore, followed by zero or more
/// letters, digits, or underscores. If the parsed identifier matches a keyword,
/// a Keyword token is returned instead.
#[tracing::instrument(level = "debug", skip(input))]
fn parse_identifier(input: &str) -> ParserResult<Token> {
    let (input, id) = context(
        "identifier",
        recognize(pair(
            take_while1(|c: char| c.is_alphabetic() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_'),
        )),
    )(input)?;

    // Check if identifier is not a specials
    if let Ok(kw) = Keyword::try_from(id) {
        return Ok((input, Token::Keyword(kw)));
    }

    Ok((input, Token::Identifier(id.to_string())))
}

/// Type alias for parser results using nom's IResult with VerboseError.
pub type ParserResult<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

/// Type alias for tokenizer results that may contain TokenizerError.
pub type TokenizerResult<'a, T> = Result<T, TokenizerError>;

/// Represents errors that can occur during tokenization.
///
/// TokenizerError provides detailed information about syntax errors,
/// including the error message, the text that caused the error, and
/// the exact position in the source code.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum TokenizerError {
    /// Error that occurs during parsing when invalid syntax is encountered.
    #[error("Parse error: {message} at position {span}")]
    ParseError {
        /// Detailed error message describing the syntax error.
        message: String,
        /// The text that was found at the error location.
        found: String,
        /// The position information for the error.
        span: Span,
    },
}

#[cfg(test)]
mod tests {

    use crate::tokenizer::literal::{StringLiteral, StringPart};

    use super::*;

    #[test]
    fn test_identifier_for_keyword() {
        let input = "micro";
        let (rest, token) = parse_identifier(input).unwrap();
        assert_eq!(token, Token::Keyword(Keyword::Micro));
        assert_eq!(rest, "");
    }

    #[test]
    fn test_identifier() {
        let input = "my_var123 other";
        let (rest, token) = parse_identifier(input).unwrap();
        assert_eq!(token, Token::Identifier("my_var123".to_string()));
        assert_eq!(rest, " other");
    }

    #[test]
    fn test_tokenizer_with_position() {
        let mut tokenizer = Tokenizer::new();
        let input = "x\nother";
        let tokens = tokenizer.tokenize(input).unwrap();

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);
        assert_eq!(tokens[0].token, Token::Identifier("x".to_string()));

        // 2行目のtokenを確認
        let print_token = &tokens[2];
        assert_eq!(print_token.line, 2);
        assert_eq!(print_token.column, 1);
    }

    #[test]
    fn test_world_block() {
        let mut tokenizer = Tokenizer::new();
        let input = r#"world TravelPlanning {
               policy "Consider budget constraints and optimize value for money"
           }"#;

        let tokens = tokenizer.tokenize(input).unwrap();

        // 期待されるトークンの確認
        let important_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| !matches!(t.token, Token::Whitespace(_) | Token::Newline))
            .collect();

        assert!(matches!(
            important_tokens[0].token,
            Token::Keyword(Keyword::World)
        ));
        assert!(
            matches!(important_tokens[1].token, Token::Identifier(ref s) if s == "TravelPlanning")
        );
        assert!(matches!(
            important_tokens[2].token,
            Token::Delimiter(Delimiter::OpenBrace)
        ));
        assert!(matches!(
            important_tokens[3].token,
            Token::Keyword(Keyword::Policy)
        ));
        assert!(matches!(important_tokens[4].token,
            Token::Literal(Literal::String(StringLiteral::Single(ref parts)))
            if parts.len() == 1
            && matches!(parts[0], StringPart::Literal(ref s)
                if s == "Consider budget constraints and optimize value for money")));
    }

    #[test]
    fn test_micro_block() {
        let mut tokenizer = Tokenizer::new();
        let input = r#"micro TravelPlanner {
               state {
                   current_plan: String = "none",
                   planning_stage: String = "none"
               }
           }"#;

        let tokens = tokenizer.tokenize(input).unwrap();

        let micro_tokens = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::Micro)))
            .count();
        assert_eq!(micro_tokens, 1);

        let state_tokens = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::State)))
            .count();
        assert_eq!(state_tokens, 1);

        let identifiers = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Identifier(_)))
            .collect::<Vec<_>>();
        assert!(identifiers
            .iter()
            .any(|t| matches!(t.token, Token::Identifier(ref s) if s == "current_plan")));
        assert!(identifiers
            .iter()
            .any(|t| matches!(t.token, Token::Identifier(ref s) if s == "planning_stage")));
    }

    #[test]
    fn test_answer_block() {
        let mut tokenizer = Tokenizer::new();
        let input = r#"answer {
               on request PlanTrip(destination: String, budget: Float) -> Result<String, Error> {
                   return Ok(plan)
               }
           }"#;

        let tokens = tokenizer.tokenize(input).unwrap();

        let answer_tokens = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::Answer)))
            .count();
        assert_eq!(answer_tokens, 1);

        let request_tokens = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::Request)))
            .count();
        assert_eq!(request_tokens, 1);

        let plantrip_tokens = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Identifier(ref s) if s == "PlanTrip"))
            .count();
        assert_eq!(plantrip_tokens, 1);
    }

    #[test]
    fn test_complete_dsl() {
        let mut tokenizer = Tokenizer::new();
        let input = r#"
           world TravelPlanning {
               policy "Consider budget constraints"
               policy "Ensure traveler safety"
           }

           micro TravelPlanner {
               state {
                   current_plan: String = "none"
               }
               answer {
                   on request PlanTrip(destination: String) -> Result<String, Error> {
                       return Ok(plan)
                   }
               }
           }"#;

        let result = tokenizer.tokenize(input);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let world_count = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::World)))
            .count();
        assert_eq!(world_count, 1);

        let micro_count = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::Micro)))
            .count();
        assert_eq!(micro_count, 1);

        let state_count = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::State)))
            .count();
        assert_eq!(state_count, 1);

        let answer_count = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::Answer)))
            .count();
        assert_eq!(answer_count, 1);
    }
}
