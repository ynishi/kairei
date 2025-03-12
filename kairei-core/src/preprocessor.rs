//! # KAIREI Preprocessor
//!
//! The Preprocessor module serves as a bridge between tokenization and parsing in the KAIREI DSL
//! processing pipeline. It transforms and normalizes tokens and source code to prepare them for
//! further processing.
//!
//! ## Core Components
//!
//! * **Preprocessor Trait**: A generic interface for different preprocessing operations
//! * **TokenPreprocessor**: Specialization for token stream preprocessing
//! * **StringPreprocessor**: Specialization for string preprocessing
//!
//! ## Position in the Pipeline
//!
//! The Preprocessor sits between the Tokenizer and Parser in the KAIREI compilation pipeline:
//!
//! ```text
//! Source Code → Tokenizer → Preprocessor → Parser → Type Checker → Evaluator
//! ```
//!
//! ## Preprocessing Operations
//!
//! ### Token Stream Preprocessing
//!
//! * **Comment Removal**: Filters out comment tokens
//! * **Whitespace Normalization**: Removes redundant whitespace tokens
//! * **Token Simplification**: Converts `TokenSpan` to simple `Token` objects for parsing
//!
//! ### String Preprocessing
//!
//! * **Comment Removal**: Removes block and line comments from source text
//! * **Whitespace Normalization**: Normalizes empty lines and trims trailing spaces
//! * **Text Preparation**: Prepares raw text for specialized processing needs
//!
//! ## Purpose and Benefits
//!
//! * **Cleaner Input**: Simplifies downstream parsing by removing non-essential elements
//! * **Normalization**: Creates consistent input format for the parser
//! * **Error Prevention**: Reduces potential error sources in the parsing stage
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use kairei_core::preprocessor::{Preprocessor, TokenPreprocessor};
//! use kairei_core::tokenizer::token::{Token, TokenSpan, Tokenizer};
//!
//! let source_code = r#"
//!     micro ExampleAgent {
//!         // This is a comment
//!         state {
//!             counter: i64 = 0;
//!         }
//!     }
//! "#;
//!
//! // Tokenize the source
//! let mut tokenizer = Tokenizer::new();
//! let token_spans = tokenizer.tokenize(source_code).unwrap();
//!
//! // Preprocess tokens
//! let preprocessor = TokenPreprocessor::default();
//! let tokens: Vec<Token> = preprocessor.process(token_spans).iter().map(|span| span.token.clone()).collect();
//! ```
//!
//! ## Integration Points
//!
//! * **Tokenizer**: Receives the initial token stream from the tokenization process
//! * **Parser**: Provides the preprocessed token stream to the parsing phase

use crate::tokenizer::token::TokenSpan;
use regex::Regex;

/// A trait for preprocessing different types of input
pub trait Preprocessor<T, U = T> {
    /// Process the input of type T and return the processed result
    fn process(&self, input: T) -> U;
}

/// Token-specific preprocessor implementation
pub struct TokenPreprocessor {}

impl Default for TokenPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenPreprocessor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Preprocessor<Vec<TokenSpan>> for TokenPreprocessor {
    fn process(&self, input: Vec<TokenSpan>) -> Vec<TokenSpan> {
        // Filter out comments and normalize whitespace
        input
            .into_iter()
            .filter(|span| {
                !span.token.is_comment() && !span.token.is_whitespace() && !span.token.is_newline()
            })
            .collect()
    }
}

/// String-specific preprocessor implementation
pub struct StringPreprocessor {
    re_block_comment: Regex,
    re_line_comment: Regex,
    re_empty_lines: Regex,
}

impl Default for StringPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl StringPreprocessor {
    pub fn new() -> Self {
        Self {
            re_block_comment: Regex::new(r"/\*(?:[^*]|\*[^/])*\*/").unwrap(),
            re_line_comment: Regex::new(r"//[^\n]*").unwrap(),
            re_empty_lines: Regex::new(r"\n\s*\n").unwrap(),
        }
    }

    fn remove_comments(&self, input: &str) -> String {
        let without_block = self.re_block_comment.replace_all(input, "");
        self.re_line_comment
            .replace_all(&without_block, "")
            .to_string()
    }

    fn normalize_empty_lines(&self, input: &str) -> String {
        self.re_empty_lines.replace_all(input, "\n").to_string()
    }

    fn trim_lines(&self, input: &str) -> String {
        input
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Preprocessor<&str, String> for StringPreprocessor {
    fn process(&self, input: &str) -> String {
        let mut output = input.to_string();

        // コメントの除去
        output = self.remove_comments(&output);

        // 空白行の正規化
        output = self.normalize_empty_lines(&output);

        // 行末の空白を除去
        output = self.trim_lines(&output);

        output
    }
}
