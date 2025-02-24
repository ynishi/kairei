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

/// KAIREI Type Checker System
///
/// The type checker serves as a critical validation layer between the Parser and AST phases
/// in the KAIREI DSL compilation pipeline, ensuring type safety and correctness across all
/// language constructs.
///
/// # Core Components and Responsibilities
///
/// ## Type Validation
/// - Validates type correctness across all DSL elements
/// - Ensures type safety of state variables and initial values
/// - Verifies type compatibility in handler signatures
/// - Validates think block interpolation types
///
/// ## Error Collection
/// - Comprehensive error types with detailed metadata
/// - Rich error context including locations and suggestions
/// - Support for both fail-fast and collect-all modes
/// - Standardized error reporting for plugins
///
/// ## Type Resolution
/// - Hierarchical type scope system for nested contexts
/// - Smart type inference for literals and expressions
/// - Generic type parameter validation
/// - Plugin type extension support
///
/// ## Integration Points
/// - Expression evaluation type checking hooks
/// - Global error reporting system integration
/// - Runtime type information provision
/// - Extensible visitor pattern implementation
///
/// # Implementation Architecture
///
/// ## Core Components
/// ```rust
/// use kairei::type_checker::{TypeScope, TypeCheckError};
/// use kairei::type_checker::visitor::common::PluginVisitor;
/// use kairei::type_checker::visitor::DefaultVisitor;
///
/// pub struct TypeChecker {
///     plugins: Vec<Box<dyn PluginVisitor>>,
///     default_visitor: DefaultVisitor,
///     context: TypeContext,
/// }
///
/// pub struct TypeContext {
///     scope: TypeScope,
///     errors: Vec<TypeCheckError>,
/// }
/// ```
///
/// ## Plugin System
/// ```rust
/// use kairei::ast::{Root, MicroAgentDef};
/// use kairei::type_checker::{TypeContext, TypeCheckResult};
///
/// pub trait TypeVisitor {
///     fn visit_root(&mut self, root: &mut Root, ctx: &mut TypeContext) -> TypeCheckResult<()>;
///     fn visit_micro_agent(&mut self, agent: &mut MicroAgentDef, ctx: &mut TypeContext) -> TypeCheckResult<()>;
///     // ... other visit methods
/// }
///
/// pub trait PluginVisitor: TypeVisitor {
///     fn before_root(&mut self, root: &mut Root, ctx: &mut TypeContext) -> TypeCheckResult<()>;
///     fn after_root(&mut self, root: &mut Root, ctx: &mut TypeContext) -> TypeCheckResult<()>;
///     // ... other lifecycle hooks
/// }
/// ```
///
/// # Type Categories
/// - Built-in types (String, Int, Float, etc.)
/// - Container types (Result, Option, Array, Map)
/// - Special types (Duration, Delay)
/// - Custom user-defined types
///
/// # Example Usage
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
/// # Development Guidelines
///
/// ## Adding New Types
/// 1. Define type in the type system
/// 2. Implement necessary validation rules
/// 3. Add appropriate error handling
/// 4. Update type inference logic
/// 5. Add test coverage
///
/// ## Creating Type Check Plugins
/// 1. Implement TypeVisitor trait
/// 2. Define plugin-specific validation rules
/// 3. Integrate with error reporting system
/// 4. Add appropriate lifecycle hooks
/// 5. Document plugin behavior
///
/// ## Error Handling Best Practices
/// 1. Use appropriate error variants
/// 2. Provide helpful error messages
/// 3. Include fix suggestions
/// 4. Consider error recovery
/// 5. Maintain error context
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
