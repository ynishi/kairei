//! # Tokenizer Component
//!
//! The Tokenizer component is responsible for lexical analysis of KAIREI DSL source code,
//! transforming raw text into a structured token stream for further processing by the parser.
//!
//! ## Design Principles
//!
//! The tokenizer follows these key design principles:
//!
//! * **Comprehensive Token Information**: Each token includes detailed position information
//!   (line, column, start/end positions) to enable precise error reporting.
//! * **Format Preservation**: Whitespace, comments, and newlines are preserved as tokens
//!   to enable accurate source code formatting.
//! * **Error Recovery**: Detailed error information is provided to help diagnose syntax issues.
//! * **Extensibility**: The modular design allows for easy addition of new token types.
//!
//! ## Component Structure
//!
//! * [`token`]: Core token types and tokenizer implementation
//! * [`keyword`]: Keyword token parsing and representation
//! * [`symbol`]: Operators and delimiters parsing
//! * [`literal`]: String, number, and boolean literal parsing
//! * [`whitespace`]: Whitespace and newline handling
//! * [`comment`]: Comment parsing and categorization
//!
//! ## Integration Points
//!
//! The Tokenizer serves as the first phase in the DSL processing pipeline:
//!
//! 1. **Input**: Raw DSL text
//! 2. **Processing**: Lexical analysis via [`Tokenizer::tokenize`](token::Tokenizer::tokenize)
//! 3. **Output**: Stream of [`TokenSpan`](token::TokenSpan) objects
//! 4. **Next Stage**: Parser consumes the token stream to build the AST
//!
//! ## Error Handling
//!
//! The tokenizer provides detailed error information through [`TokenizerError`](token::TokenizerError),
//! including the exact position and context of syntax errors.
//!
//! ## Usage Example
//!
//! ```rust
//! use kairei_core::tokenizer::token::{Tokenizer, TokenSpan};
//!
//! fn tokenize_example() -> Result<Vec<TokenSpan>, Box<dyn std::error::Error>> {
//!     let input = r#"micro ExampleAgent {
//!         state {
//!             count: Int = 0
//!         }
//!     }"#;
//!     
//!     let mut tokenizer = Tokenizer::new();
//!     let tokens = tokenizer.tokenize(input)?;
//!     Ok(tokens)
//! }
//! ```

pub mod comment;
pub mod keyword;
pub mod literal;
pub mod symbol;
pub mod token;
pub mod whitespace;
