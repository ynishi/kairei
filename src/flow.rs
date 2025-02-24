//! # KAIREI Flow Implementation
//! 
//! The Flow module implements the transformation and execution pipeline for the KAIREI DSL.
//! It manages the flow between the DSL Layer, AST Layer, and Runtime Layer.
//! 
//! ## Architecture
//! 
//! The flow system consists of three primary layers:
//! 
//! 1. DSL Layer
//!    - World DSL: Environment and global configuration
//!    - MicroAgent DSL: Individual agent behaviors
//! 
//! 2. AST Layer
//!    - WorldAgent (MA AST): Transformed World definition
//!    - Events AST: Extracted event definitions
//!    - MicroAgent AST: Transformed MicroAgent definitions
//! 
//! 3. Runtime Layer
//!    - Runtime: Agent behavior execution
//!    - EventRegistry: Event registration management
//!    - EventBus: Event distribution
//! 
//! ## Type Validation
//! 
//! The type checking phase validates:
//! 
//! - Language construct correctness
//! - State definition type safety
//! - Request/response handler compatibility
//! - Think block interpolation correctness
//! 
//! ## Error Handling
//! 
//! Errors are handled through:
//! 
//! - Aggregated error collection
//! - Critical error fast-fail
//! - Detailed source tracking
//! - Recovery mechanisms
//! 
//! ## Example
//! 
//! ```rust
//! use kairei::flow::TypeChecker;
//! 
//! impl TypeChecker {
//!     fn check_with_collection(&self, ast: &Root) -> TypeCheckResult<()> {
//!         let mut collector = ErrorCollector::new();
//!         
//!         // Collect errors from all phases
//!         self.check_types(ast, &mut collector)?;
//!         self.check_think_blocks(ast, &mut collector)?;
//!         
//!         // Process collected errors
//!         if collector.has_critical_errors() {
//!             Err(collector.take_critical_errors().into())
//!         } else if collector.has_errors() {
//!             Err(collector.take_errors().into())
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//! ```

use crate::{
    ast::Root,
    error::TypeCheckError,
    type_checker::{TypeCheckResult, TypeContext},
};

/// Collects and manages errors during type checking
pub struct ErrorCollector {
    errors: Vec<TypeCheckError>,
    critical_errors: Vec<TypeCheckError>,
}

impl ErrorCollector {
    /// Creates a new ErrorCollector
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            critical_errors: Vec::new(),
        }
    }

    /// Adds an error to the collector
    pub fn add_error(&mut self, error: TypeCheckError) {
        if error.is_critical() {
            self.critical_errors.push(error);
        } else {
            self.errors.push(error);
        }
    }

    /// Returns true if there are any critical errors
    pub fn has_critical_errors(&self) -> bool {
        !self.critical_errors.is_empty()
    }

    /// Returns true if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Takes ownership of all critical errors
    pub fn take_critical_errors(&mut self) -> Vec<TypeCheckError> {
        std::mem::take(&mut self.critical_errors)
    }

    /// Takes ownership of all errors
    pub fn take_errors(&mut self) -> Vec<TypeCheckError> {
        std::mem::take(&mut self.errors)
    }
}

/// Manages the type checking context and flow
pub struct TypeChecker {
    context: TypeContext,
}

impl TypeChecker {
    /// Creates a new TypeChecker
    pub fn new(context: TypeContext) -> Self {
        Self { context }
    }

    /// Performs type checking with error collection
    pub fn check_with_collection(&self, ast: &Root) -> TypeCheckResult<()> {
        let mut collector = ErrorCollector::new();
        
        // Collect errors from all phases
        self.check_types(ast, &mut collector)?;
        self.check_think_blocks(ast, &mut collector)?;
        
        // Process collected errors
        if collector.has_critical_errors() {
            Err(TypeCheckError::Multiple(collector.take_critical_errors()))
        } else if collector.has_errors() {
            Err(TypeCheckError::Multiple(collector.take_errors()))
        } else {
            Ok(())
        }
    }

    /// Checks types in the AST
    fn check_types(&self, ast: &Root, collector: &mut ErrorCollector) -> TypeCheckResult<()> {
        // Implementation details...
        Ok(())
    }

    /// Checks think blocks in the AST
    fn check_think_blocks(&self, ast: &Root, collector: &mut ErrorCollector) -> TypeCheckResult<()> {
        // Implementation details...
        Ok(())
    }
}
