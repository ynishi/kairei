//! Type checker module for KAIREI DSL
//!
//! This module implements type checking for the KAIREI DSL, validating type
//! correctness across all language constructs including state definitions,
//! request handlers, think blocks, and plugin interactions.

mod error;
mod scope;
mod visitor;

pub use error::{TypeCheckError, TypeCheckResult};
pub use scope::TypeScope;
pub use visitor::{DefaultTypeVisitor, PluginTypeVisitor, TypeVisitor};

use crate::ast::Root;
use crate::provider::plugin::ProviderPlugin;
use dashmap::DashMap;
use std::sync::Arc;

/// Core type checker trait defining the main interface for type validation
pub trait TypeChecker {
    /// Perform type checking on an entire AST
    fn check_types(&mut self, ast: &mut Root) -> TypeCheckResult<()>;

    /// Get any collected type errors
    fn collect_errors(&self) -> Vec<TypeCheckError>;
}

/// Context for type checking operations
pub struct TypeContext {
    /// Collected errors during type checking
    errors: Vec<TypeCheckError>,
    /// Current type scope
    scope: TypeScope,
    /// Plugin type information
    plugins: Arc<DashMap<String, Box<dyn ProviderPlugin>>>,
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeContext {
    /// Create a new type checking context
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            scope: TypeScope::new(),
            plugins: Arc::new(DashMap::new()),
        }
    }

    /// Add an error to the context
    pub fn add_error(&mut self, error: TypeCheckError) {
        self.errors.push(error);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Take the collected errors
    pub fn take_errors(&mut self) -> Vec<TypeCheckError> {
        std::mem::take(&mut self.errors)
    }

    /// Clear the context state
    pub fn clear(&mut self) {
        self.errors.clear();
        self.scope.clear();
        self.plugins.clear();
    }

    /// Register a plugin for type checking
    pub fn register_plugin(&self, name: String, plugin: Box<dyn ProviderPlugin>) {
        self.plugins.insert(name, plugin);
    }
}

/// Default implementation of TypeChecker
pub struct DefaultTypeChecker {
    visitor: DefaultTypeVisitor,
    context: TypeContext,
}

impl DefaultTypeChecker {
    /// Create a new type checker instance
    pub fn new() -> Self {
        Self {
            visitor: DefaultTypeVisitor,
            context: TypeContext::new(),
        }
    }
}

impl TypeChecker for DefaultTypeChecker {
    fn check_types(&mut self, ast: &mut Root) -> TypeCheckResult<()> {
        // Clear any previous state
        self.context.clear();

        // Visit all micro agents
        for agent in &mut ast.micro_agent_defs {
            self.visitor.visit_micro_agent(agent, &mut self.context)?;
        }

        // Visit world definition if present
        if let Some(world) = &ast.world_def {
            // Add world-specific type checking here if needed
            for handler in &world.handlers.handlers {
                self.visitor.visit_handler(handler, &mut self.context)?;
            }
        }

        Ok(())
    }

    fn collect_errors(&self) -> Vec<TypeCheckError> {
        self.context.errors.clone()
    }
}

impl Default for DefaultTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
