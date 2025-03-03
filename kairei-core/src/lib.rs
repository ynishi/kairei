//! # KAIREI: AI Agent Orchestration Platform
//!
//! KAIREI provides a robust foundation for developing and orchestrating AI agents
//! through a declarative approach and strong type safety guarantees.
//!
//! ## Technical Foundations
//!
//! ### 1. Declarative Development with DSL
//! KAIREI employs two domain-specific languages:
//! - World DSL: Defines the environment and interaction protocols
//! - MicroAgent DSL: Specifies agent behavior and capabilities
//!
//! Implementation components:
//! - Abstract Syntax Tree ([`ast`])
//! - Tokenization ([`tokenizer`])
//! - Formatting ([`formatter`])
//!
//! ### 2. Three-Layer Architecture
//! The system is organized into three distinct layers:
//! - Native Layer: Core system features ([`native_feature`], [`core`])
//! - Plugin Layer: Extensible components ([`provider`])
//! - MicroAgent Layer: Business logic ([`agent_registry`], [`runtime`])
//!
//! ### 3. Event-Based Async Processing
//! Asynchronous event processing forms the backbone of agent communication:
//! - Event system ([`event`])
//! - Runtime execution ([`runtime`])
//! - State management ([`core::types`])
//!
//! ### 4. LLM Integration
//! Flexible integration with Language Models:
//! - Provider interface ([`provider`])
//! - Evaluation system ([`eval`])
//! - Type-safe interactions ([`type_checker`])
//!
//! ### 5. Type Safety
//! Comprehensive type checking ensures system reliability:
//! - Static analysis ([`type_checker`])
//! - Runtime validation ([`runtime`])
//! - Error handling ([`error`])
//!
//! ## DSL Processing Pipeline
//!
//! KAIREI implements a comprehensive DSL processing pipeline:
//!
//! ```text
//! Source Code → Tokenizer → Preprocessor → Parser → Type Checker → Evaluator
//! ```
//!
//! ### Stage 1: Tokenization (Lexical Analysis)
//!
//! The [`tokenizer`] module transforms raw source code into a stream of tokens.
//! It identifies keywords, identifiers, literals, and other basic elements.
//!
//! ### Stage 2: Preprocessing
//!
//! The [`preprocessor`] module normalizes the token stream, removing comments
//! and unnecessary whitespace to prepare for parsing.
//!
//! ### Stage 3: Parsing (Syntactic Analysis)
//!
//! The [`analyzer`] module transforms the token stream into an Abstract Syntax Tree (AST).
//! It uses a parser combinator pattern to construct a hierarchical representation of the code.
//!
//! ### Stage 4: Type Checking (Semantic Analysis)
//!
//! The [`type_checker`] module validates the AST for type correctness and semantic rules.
//! It ensures that the code follows the DSL's semantic constraints.
//!
//! ### Stage 5: Evaluation (Execution)
//!
//! The [`eval`] module executes the validated AST at runtime, converting the code
//! into actual behavior within the KAIREI ecosystem.
//!
//! ## AST Registry and Coordination
//!
//! The [`ast_registry`] module coordinates the overall parse flow, acting as the bridge
//! between different stages of the pipeline. It provides a unified interface for
//! transforming source code into executable AST structures.
//!
//! ## Runtime and Event System
//!
//! The [`runtime`] and [`event`] modules execute the AST in an event-driven environment,
//! orchestrating agent interactions through asynchronous events and message passing.

pub mod agent_registry;
pub mod analyzer;
pub mod api;
pub mod ast;
pub mod ast_registry;
pub mod config;
pub mod core;
pub mod error;
pub mod eval;
pub mod event;
pub mod formatter;
pub mod r#gen;
pub mod native_feature;
pub mod preprocessor;
pub mod provider;
pub mod runtime;
pub mod system;
pub mod system_api_impl;
pub mod timestamp;
pub mod tokenizer;
pub mod type_checker;

// Re-exports
pub use ast::*;
pub use error::*;
pub use eval::*;
pub use event::*;

#[cfg(test)]
mod tests {
    use tracing_subscriber::{EnvFilter, FmtSubscriber};

    #[ctor::ctor]
    fn init_tests() {
        // テストの前に一度だけ実行したい処理
        // tracing_subscriberの初期化
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber");
    }
}
