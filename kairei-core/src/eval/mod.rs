//! KAIREI Evaluation System
//!
//! The evaluation system is responsible for executing KAIREI DSL code at runtime,
//! transforming the parsed and type-checked AST into actual behavior. It serves as
//! the execution engine for the KAIREI microagent system.
//!
//! # Core Components
//!
//! ## Evaluator
//! The central component that orchestrates the evaluation process, handling
//! handler blocks, answer handler blocks, and expressions.
//!
//! ## Statement Evaluator
//! Evaluates individual statements within handler blocks, managing control flow
//! and statement execution.
//!
//! ## Expression Evaluator
//! Evaluates expressions, including literals, variables, function calls, and
//! operations.
//!
//! ## Execution Context
//! Maintains the runtime state, including variables, agent state, and event handling.
//!
//! ## Generator
//! Handles prompt generation for LLM integration.
//!
//! # Evaluation Pipeline
//!
//! 1. AST nodes from the parser are passed to the Evaluator
//! 2. The Evaluator delegates to specialized evaluators (Statement, Expression)
//! 3. Evaluation results are processed and returned
//! 4. Side effects (state changes, events) are managed through the ExecutionContext
//!
//! # Integration Points
//!
//! - Parser: Provides AST for evaluation
//! - Type Checker: Ensures type safety before evaluation
//! - Runtime: Manages agent lifecycle and event processing
//! - Provider System: Integrates with external services and LLMs

pub mod context;
pub mod evaluator;
pub mod expression;
pub mod generator;
pub mod statement;
