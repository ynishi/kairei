//! Type checker module for KAIREI DSL
//! 
//! This module implements type checking for the KAIREI DSL, validating type
//! correctness across all language constructs including state definitions,
//! request handlers, think blocks, and plugin interactions.

use std::sync::Arc;
use dashmap::DashMap;
use crate::ast::{Root, MicroAgentDef, StateDef, HandlerDef, Expression};
use crate::error::Error;

/// Result type for type checking operations
pub type TypeCheckResult<T> = Result<T, Error>;

/// Core type checker trait defining the main interface for type validation
pub trait TypeChecker {
    /// Perform type checking on an entire AST
    fn check_types(&self, ast: &Root) -> TypeCheckResult<()>;
    
    /// Get any collected type errors
    fn collect_errors(&self) -> Vec<Error>;
}

/// Context for type checking operations
pub struct TypeContext {
    /// Collected errors during type checking
    errors: Vec<Error>,
    /// Current type scope
    scope: TypeScope,
    /// Plugin type information
    plugins: Arc<DashMap<String, PluginTypeInfo>>,
}

impl TypeContext {
    /// Create a new type checking context
    pub fn new() -> Self {
        Self {
            errors: Vec::with_capacity(10),
            scope: TypeScope::new(),
            plugins: Arc::new(DashMap::new()),
        }
    }
    
    /// Clear the context state
    pub fn clear(&mut self) {
        self.errors.clear();
        self.scope.clear();
        self.plugins.clear();
    }
    
    /// Add an error to the context
    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }
    
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Take the collected errors
    pub fn take_errors(&mut self) -> Vec<Error> {
        std::mem::take(&mut self.errors)
    }
}

/// Scope management for type checking
pub struct TypeScope {
    /// Stack of scopes for nested contexts
    scopes: Vec<TypeScopeLayer>,
}

impl TypeScope {
    /// Create a new type scope
    pub fn new() -> Self {
        Self {
            scopes: vec![TypeScopeLayer::new()],
        }
    }
    
    /// Clear all scopes
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.scopes.push(TypeScopeLayer::new());
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(TypeScopeLayer::new());
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
}

/// Single layer in the type scope stack
struct TypeScopeLayer {
    /// Type definitions in this scope
    types: DashMap<String, TypeInfo>,
}

impl TypeScopeLayer {
    /// Create a new scope layer
    fn new() -> Self {
        Self {
            types: DashMap::new(),
        }
    }
}

/// Information about a type
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// Name of the type
    name: String,
    /// Type parameters if generic
    type_params: Vec<TypeInfo>,
}

/// Plugin type information
#[derive(Debug, Clone)]
pub struct PluginTypeInfo {
    /// Plugin name
    name: String,
    /// Plugin schema
    schema: PluginSchema,
}

/// Plugin type schema
#[derive(Debug, Clone)]
pub struct PluginSchema {
    /// Request type information
    request: TypeInfo,
    /// Response type information
    response: TypeInfo,
}

/// Visitor trait for type checking different AST nodes
pub trait TypeVisitor {
    /// Visit a micro agent definition
    fn visit_micro_agent(&self, agent: &MicroAgentDef, ctx: &mut TypeContext) -> TypeCheckResult<()>;
    
    /// Visit a state definition
    fn visit_state(&self, state: &StateDef, ctx: &mut TypeContext) -> TypeCheckResult<()>;
    
    /// Visit a handler definition
    fn visit_handler(&self, handler: &HandlerDef, ctx: &mut TypeContext) -> TypeCheckResult<()>;
    
    /// Visit an expression
    fn visit_expression(&self, expr: &Expression, ctx: &mut TypeContext) -> TypeCheckResult<()>;
}
