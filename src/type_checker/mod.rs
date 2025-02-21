pub mod checker;
mod error;
pub mod plugin_interface;
pub mod scope;
pub mod visitor;

#[cfg(test)]
pub mod tests;

pub use crate::type_checker::visitor::common::TypeVisitor;
pub use checker::TypeChecker;
pub use error::{TypeCheckError, TypeCheckResult};
pub use plugin_interface::TypeCheckerPlugin;
pub use scope::TypeScope;

use crate::ast;

/// Type checking context
#[derive(Clone)]
pub struct TypeContext {
    pub scope: TypeScope,
    errors: Vec<TypeCheckError>,
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeContext {
    /// Create a new type context with an initial global scope
    pub fn new() -> Self {
        Self {
            scope: TypeScope::new(),
            errors: Vec::new(),
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

    /// Take all errors, leaving the error list empty
    pub fn take_errors(&mut self) -> Vec<TypeCheckError> {
        std::mem::take(&mut self.errors)
    }

    /// Clear all errors
    pub fn clear(&mut self) {
        self.errors.clear();
    }
}

/// Run type checker on AST
pub fn run_type_checker(root: &mut ast::Root) -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    checker.check_types(root)
}
