//! # AST Registry: Coordinating the DSL Processing Pipeline
//!
//! The AST Registry module coordinates the complete DSL processing pipeline in KAIREI,
//! serving as the central hub that connects tokenization, parsing, type checking, and AST management.
//!
//! ## Pipeline Coordination
//!
//! This module implements the full parsing flow:
//!
//! ```text
//! Source Code → Tokenizer → Preprocessor → Parser → Type Checker → AST
//! ```
//!
//! ### Complete Processing Sequence
//!
//! 1. **Source Code Reception**: Raw DSL strings are received for processing
//! 2. **Tokenization**: The source is tokenized into a stream of lexical elements
//! 3. **Preprocessing**: Tokens are normalized (comments removed, whitespace filtered)
//! 4. **Parsing**: Tokens are transformed into an Abstract Syntax Tree (AST)
//! 5. **Type Checking**: The AST is validated for type correctness
//! 6. **AST Management**: The resulting AST is stored and made available for evaluation
//!
//! ## Core Components
//!
//! * **AST Registry**: Central registry for storing and retrieving ASTs
//! * **Flow Coordination**: Methods for processing DSL and managing the resulting ASTs
//! * **Error Handling**: Comprehensive error management across the pipeline
//!
//! ## AST Management
//!
//! Beyond the initial parsing process, the AST Registry provides:
//!
//! * **AST Caching**: Storing processed ASTs for reuse
//! * **Agent AST Registry**: Registering and retrieving agent definitions
//! * **Built-in Agent Creation**: Generation of system-defined agents
//!
//! ## Integration Points
//!
//! * **Tokenizer**: First stage of processing for raw source code
//! * **Preprocessor**: Normalization and simplification of the token stream
//! * **Parser**: Construction of the AST from tokens
//! * **Type Checker**: Validation of the AST's type correctness
//! * **System**: High-level interface for DSL processing
//! * **Runtime**: Execution environment for the validated AST

use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use tracing::{debug, warn};

use crate::{
    ASTError, ASTResult, AnswerDef, EventsDef, Expression, HandlerBlock, HandlersDef, Literal,
    MicroAgentDef, RequestHandler, RequestType, StateAccessPath, StateDef, StateVarDef, Statement,
    TypeInfo, WorldDef,
    analyzer::{self, Parser},
    ast,
    config::AgentConfig,
    preprocessor::{self, Preprocessor},
    tokenizer::{self, token::Token},
    type_checker::run_type_checker,
};

/// Central registry for managing Abstract Syntax Trees (ASTs) in KAIREI
///
/// The AstRegistry coordinates the end-to-end processing pipeline for KAIREI DSL,
/// from raw source code to a fully validated Abstract Syntax Tree.
#[derive(Debug, Clone, Default)]
pub struct AstRegistry {
    asts: Arc<DashMap<String, Arc<MicroAgentDef>>>,
}

impl AstRegistry {
    /// Transforms a DSL string into an Abstract Syntax Tree (AST) representation.
    ///
    /// This method implements the complete parsing pipeline for KAIREI DSL:
    /// 1. **Tokenization**: Converts raw text into a sequence of tokens
    /// 2. **Preprocessing**: Normalizes and transforms tokens for consistent parsing
    /// 3. **Parsing**: Builds a hierarchical AST structure from the token stream
    /// 4. **Type Checking**: Validates type correctness across the entire AST
    ///
    /// The parsing flow is:
    /// ```text
    /// DSL String → Tokenizer → Preprocessor → Parser → Type Checker → AST Root
    /// ```
    ///
    /// # Arguments
    /// * `dsl` - A string slice containing the KAIREI DSL code to parse
    ///
    /// # Returns
    /// * `ASTResult<ast::Root>` - On success, returns the parsed AST root
    ///   containing world and agent definitions
    ///
    /// # Errors
    /// * `ASTError::ParseError` - If the DSL cannot be parsed correctly
    /// * `ASTError::TypeError` - If type checking fails
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use kairei_core::ast_registry::AstRegistry;
    ///
    /// let registry = AstRegistry::default();
    /// let dsl = r#"
    ///     micro ExampleAgent {
    ///         state {
    ///             counter: i64 = 0;
    ///         }
    ///     }
    /// "#;
    ///
    /// let ast = registry.create_ast_from_dsl(dsl).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_ast_from_dsl(&self, dsl: &str) -> ASTResult<ast::Root> {
        // 1. Tokenization: Convert DSL string into tokens
        let mut tokenizer = tokenizer::token::Tokenizer::new();
        let tokens = tokenizer.tokenize(dsl).unwrap();

        // 2. Preprocessing: Apply token transformations
        let preprocessor = preprocessor::TokenPreprocessor::default();
        let tokens: Vec<Token> = preprocessor.process(tokens);
        debug!("{:?}", tokens);

        // 3. Parsing: Convert tokens into AST structure
        let (pos, mut root) = analyzer::parsers::world::parse_root()
            .parse(tokens.as_slice(), 0)
            .map_err(|e: analyzer::ParseError| ASTError::ParseError {
                message: format!("failed to parse DSL {}", e),
                target: "root".to_string(),
            })?;
        debug!("{:?}", root);

        // Verify that all tokens were consumed
        if pos != tokens.len() {
            warn!(
                "Failed to parse DSL: {:?}, {}, {}",
                tokens,
                pos,
                tokens.len()
            );
            return Err(ASTError::ParseError {
                message: "failed to parse DSL".to_string(),
                target: "root".to_string(),
            });
        }

        // 4. Type Checking: Validate type correctness in the AST
        run_type_checker(&mut root).map_err(ASTError::from)?;

        Ok(root)
    }
    pub async fn register_agent_ast(
        &mut self,
        _agent_name: &str,
        _ast: &MicroAgentDef,
    ) -> ASTResult<()> {
        self.asts
            .insert(_agent_name.to_string(), Arc::new(_ast.clone()));
        Ok(())
    }

    pub async fn get_agent_ast(&self, agent_name: &str) -> ASTResult<Arc<MicroAgentDef>> {
        let ast = self
            .asts
            .get(agent_name)
            .ok_or(ASTError::ASTNotFound(agent_name.to_string()))?;
        Ok(ast.value().clone())
    }

    pub async fn list_agent_asts(&self) -> Vec<String> {
        self.asts.iter().map(|entry| entry.key().clone()).collect()
    }

    // factory method for creating a world AST
    pub fn create_world_ast(&self) -> WorldDef {
        WorldDef {
            name: "world".to_string(),
            policies: vec![],
            config: None,
            events: EventsDef { events: vec![] },
            handlers: HandlersDef { handlers: vec![] },
        }
    }

    pub async fn create_builtin_agent_asts(
        &self,
        config: &AgentConfig,
    ) -> ASTResult<Vec<MicroAgentDef>> {
        let config = config.clone().scale_manager.unwrap_or_default();
        let scale_manager_def = MicroAgentDef {
            name: "scale_manager".to_string(),
            state: Some(StateDef {
                variables: {
                    let mut vars = HashMap::new();
                    vars.insert(
                        "enabled".to_string(),
                        StateVarDef {
                            name: "enabled".to_string(),
                            type_info: TypeInfo::Simple("boolean".to_string()),
                            initial_value: Some(Expression::Literal(Literal::Boolean(
                                config.enabled,
                            ))),
                        },
                    );
                    vars.insert(
                        "max_instances_per_agent".to_string(),
                        StateVarDef {
                            name: "self.max_instances_per_agent".to_string(),
                            type_info: TypeInfo::Simple("i64".to_string()),
                            initial_value: Some(Expression::Literal(Literal::Integer(
                                config.max_instances_per_agent as i64,
                            ))),
                        },
                    );
                    vars
                },
            }),
            // simply return the value of max_instances_per_agent for agent request event.
            answer: Some(AnswerDef {
                handlers: vec![RequestHandler {
                    request_type: RequestType::Custom("get_max_instances_per_agent".to_string()),
                    parameters: vec![],
                    return_type: TypeInfo::Simple("i64".to_string()),
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![Statement::Return(Expression::StateAccess(
                            StateAccessPath(vec!["self".into(), "max_instances_per_agent".into()]),
                        ))],
                    },
                }],
            }),
            ..Default::default()
        };
        Ok(vec![scale_manager_def])
    }
}
