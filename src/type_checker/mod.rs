pub mod checker;
mod error;
mod init;
mod plugin_config_validator;
pub mod plugin_interface;
pub mod scope;
pub mod visitor;

#[cfg(test)]
pub mod tests;

pub use crate::type_checker::visitor::common::TypeVisitor;
pub use checker::TypeChecker;
pub use error::{TypeCheckError, TypeCheckResult};
pub use init::create_type_checker;
pub use plugin_config_validator::PluginConfigValidator;
pub use plugin_interface::TypeCheckerPlugin;
pub use scope::TypeScope;

use crate::ast;

/// MicroAgent DSL Type System
///
/// Implements static type checking for the MicroAgent DSL, ensuring
/// type safety across state definitions, event handlers, and requests.
///
/// # Type Categories
/// - Built-in types (String, Int, Float, etc.)
/// - Custom types defined in World
/// - Generic types (Result, Option, List)
///
/// # Type Checking Features
/// - State variable type validation
/// - Event parameter type checking
/// - Request/response signature validation
/// - Expression type inference
///
/// # Example
/// ```text
/// type UserProfile {
///     id: String
///     age: Int
///     preferences: List<String>
/// }
///
/// micro UserAgent {
///     state {
///         profile: UserProfile
///     }
/// }
/// ```
///
/// # Type Context
/// The type checking context maintains:
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

/// Run Type Checker
///
/// Performs comprehensive type checking on the AST, validating:
/// - State variable declarations
/// - Event handler signatures
/// - Request/response types
/// - Expression type correctness
///
/// # Type Validation Process
/// 1. Validate type definitions
/// 2. Check state variable types
/// 3. Verify event handler signatures
/// 4. Validate expression types
/// 5. Ensure request/response type safety
pub fn run_type_checker(root: &mut ast::Root) -> TypeCheckResult<()> {
    let mut checker = TypeChecker::new();
    checker.check_types(root)
}
