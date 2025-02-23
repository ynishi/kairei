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
//! ## Module Organization
pub mod agent_registry;
pub mod analyzer;
pub mod ast;
pub mod ast_registry;
pub mod config;
pub mod core;
pub mod error;
pub mod eval;
pub mod event;
pub mod formatter;
pub mod gen;
pub mod native_feature;
pub mod preprocessor;
pub mod provider;
pub mod runtime;
pub mod system;
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
