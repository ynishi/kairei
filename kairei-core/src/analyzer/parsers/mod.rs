//! # Specialized Parsers
//!
//! This module contains specialized parsers for the KAIREI DSL syntax.
//! These parsers are built using the parser combinators defined in the parent module.
//!
//! ## Parser Categories
//!
//! * **Common Parsers**: Reusable parsers for common language constructs
//! * **Agent Parsers**: Parsers for MicroAgent DSL syntax
//! * **Expression Parsers**: Parsers for expressions
//! * **Statement Parsers**: Parsers for statements
//! * **Type Parsers**: Parsers for type definitions
//! * **World Parsers**: Parsers for World DSL syntax
//! * **Handler Parsers**: Parsers for event handlers

/// Common parsers for reusable language constructs
pub mod common;
pub use common::*;

/// Parsers for MicroAgent DSL syntax
pub mod agent;
/// Documentation for answer handler parsers
pub mod doc_answer_handlers;
/// Documentation for event handler parsers
pub mod doc_event_handlers;
/// Documentation for expression parsers
pub mod doc_expression;
/// Documentation for lifecycle handler parsers
pub mod doc_lifecycle_handlers;
/// Documentation for statement parsers
pub mod doc_statement;
/// Documentation for system handler parsers
pub mod doc_system_handlers;
/// Documentation for type parsers
pub mod doc_types;
/// Parsers for expressions in the KAIREI DSL
pub mod expression;
/// Parsers for event handlers
mod handlers;
/// Parsers for statements in the KAIREI DSL
pub mod statement;
/// Parsers for type definitions
mod types;
/// Parsers for World DSL syntax
pub mod world;

#[cfg(test)]
mod tests;
