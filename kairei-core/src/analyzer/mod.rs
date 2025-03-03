//! # KAIREI Analyzer (Parser) System
//!
//! The Analyzer module implements KAIREI's parsing system, transforming token streams from the Tokenizer
//! into Abstract Syntax Trees (AST) using a Parser Combinator pattern.
//!
//! ## Core Components
//!
//! * **Parser Trait**: Defines the core parsing interface
//! * **Combinators**: Building blocks for creating complex parsers
//! * **Specialized Parsers**: Implementations for specific language constructs
//!
//! ## Architecture Design
//!
//! The Analyzer follows a parser combinator design pattern:
//!
//! 1. **Core Parser Interface**: The `Parser` trait defines the parsing contract
//! 2. **Combinators**: Small, composable parser units that can be combined
//! 3. **Specialized Parsers**: Domain-specific parsers for KAIREI DSL constructs
//! 4. **Error Handling**: Detailed error reporting with context
//!
//! ## Position in the Pipeline
//!
//! The Analyzer sits between the Tokenizer and Type Checker in the KAIREI compilation pipeline:
//!
//! ```text
//! Source Code → Tokenizer → Analyzer/Parser → Type Checker → Evaluator
//! ```
//!
//! ## Usage Example
//!
//! ```ignore
//! use kairei_core::analyzer::prelude::*;
//! use kairei_core::analyzer::Parser;
//! use kairei_core::tokenizer::token::Token;
//!
//! // Create a simple parser that matches a specific token
//! let parser = equal(Token::Identifier("example".to_string()));
//!
//! // Parse a token stream
//! let tokens = vec![Token::Identifier("example".to_string())];
//! let result = parser.parse(&tokens, 0);
//! ```

pub mod combinators;
pub mod core;
pub mod parsers;
pub mod prelude;

pub use core::ParseError;
pub use core::ParseResult;
pub use core::Parser;

pub use crate::ast;
