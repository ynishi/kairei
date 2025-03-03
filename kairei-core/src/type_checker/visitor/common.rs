/// # Event Handler Type Checking
///
/// ## Overview
///
/// This module implements type checking rules and validation for Event Handlers in the KAIREI type system.
/// Event Handlers are a core component of the system, requiring specific type checking considerations
/// due to their unique role in handling various types of events and maintaining state.
///
/// ## Event Handler Types
///
/// ### 1. Answer Handlers
/// ```text
/// on answer {
///     // Must return Result<String, Error>
///     return Ok("Response text");
/// }
/// ```
/// - Return type must be Result<String, Error>
/// - String responses for direct communication
/// - Error handling for response failures
///
/// ### 2. Observe Handlers
/// ```text
/// on observe {
///     // Can return Result<Any, Error>
///     return Ok(observed_data);
/// }
/// ```
/// - Return type can be Result<Any, Error>
/// - Flexible return types for different observation scenarios
/// - Error handling for observation failures
///
/// ### 3. React Handlers
/// ```text
/// on react {
///     // Typically return Result<Unit, Error>
///     perform_action();
///     return Ok(());
/// }
/// ```
/// - Usually return Result<Unit, Error>
/// - Focus on side effects rather than return values
/// - Error handling for action failures
///
/// ### 4. Lifecycle Handlers
/// ```text
/// lifecycle {
///     on_init {
///         // Initialization logic
///         return Ok(());
///     }
///     on_destroy {
///         // Cleanup logic
///         return Ok(());
///     }
/// }
/// ```
/// - Return Result<Unit, Error>
/// - Specific to initialization and cleanup
/// - Error handling for lifecycle operations
///
/// ## Type Checking Rules
///
/// ### 1. Return Type Validation
///
/// All handlers must follow specific return type rules:
///
/// ```text
/// // Basic structure
/// Result<T, Error>
///
/// // Handler-specific types
/// Answer   -> Result<String, Error>
/// Observe  -> Result<Any, Error>
/// React    -> Result<Unit, Error>
/// Lifecycle-> Result<Unit, Error>
/// ```
///
/// Key validation points:
/// - All handlers must return a Result type
/// - The error type must be Error
/// - The success type must match the handler type
/// - Proper wrapping with Ok() or Err() is required
///
/// ### 2. State Access Rules
///
/// Event Handlers have specific rules for state access:
///
/// ```text
/// state {
///     counter: Int,
///     config: Config,
///     data: CustomType,
/// }
///
/// on answer {
///     // State access validation
///     state.counter += 1;
///     let cfg = state.config;
///     return Ok(state.data.to_string());
/// }
/// ```
///
/// Validation requirements:
/// - State variables must be defined in the state block
/// - Access paths must be valid
/// - Type compatibility must be maintained
/// - Mutations must respect type constraints
///
/// ### 3. Error Handling
///
/// Error handling must follow these rules:
///
/// ```text
/// // Error propagation
/// with_error {
///     risky_operation()?;
/// } handle_error {
///     return Err(error_message);
/// }
/// ```
///
/// Requirements:
/// - Error types must be convertible to Error
/// - Error handlers must maintain return type consistency
/// - Proper error propagation using ? operator
///
/// ## Implementation Details
///
/// ### 1. Type Checking Process
///
/// The type checker performs these validations:
///
/// ```text
/// fn check_handler(&self, handler: &HandlerDef) -> TypeCheckResult<()> {
///     // 1. Validate return type
///     self.check_return_type(handler.block)?;
///     
///     // 2. Validate state access
///     self.check_state_access(handler.block)?;
///     
///     // 3. Validate error handling
///     self.check_error_handling(handler.block)?;
///     
///     Ok(())
/// }
/// ```
///
/// ### 2. Context Management
///
/// The type checker maintains context for each handler:
///
/// ```text
/// pub struct HandlerContext {
///     // Handler-specific information
///     handler_type: HandlerType,
///     return_type: TypeInfo,
///     
///     // State access information
///     state_vars: HashMap<String, TypeInfo>,
///     
///     // Error handling context
///     in_error_handler: bool,
/// }
/// ```
///
/// ### 3. Error Reporting
///
/// The type checker provides detailed error messages:
///
/// ```text
/// // Return type errors
/// InvalidEventHandlerReturn {
///     message: String,
///     expected: TypeInfo,
///     found: TypeInfo,
///     help: Option<String>,
///     suggestion: Option<String>,
/// }
///
/// // State access errors
/// InvalidEventHandlerStateAccess {
///     message: String,
///     help: Option<String>,
///     suggestion: Option<String>,
/// }
/// ```
///
/// ## Future Considerations
///
/// 1. Type System Extensions
/// - Support for generic event handlers
/// - Custom return type constraints
/// - Advanced state access patterns
///
/// 2. Error Handling Improvements
/// - More detailed error messages
/// - Context-aware suggestions
/// - Better error recovery strategies
///
/// 3. Performance Optimizations
/// - Caching of type information
/// - Efficient state access validation
/// - Optimized error handling paths
use crate::{
    ast::{Expression, HandlerBlock, HandlerDef, MicroAgentDef, Root, StateDef, Statement},
    type_checker::{TypeCheckResult, TypeContext},
};

/// Common visitor trait for type checking AST nodes
pub trait TypeVisitor {
    /// Visit the root node of the AST
    fn visit_root(&mut self, root: &mut Root, ctx: &mut TypeContext) -> TypeCheckResult<()>;

    /// Visit a micro agent definition
    fn visit_micro_agent(
        &mut self,
        agent: &mut MicroAgentDef,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()>;

    /// Visit a state definition
    fn visit_state(&mut self, state: &mut StateDef, ctx: &mut TypeContext) -> TypeCheckResult<()>;

    /// Visit a handler definition
    fn visit_handler(&mut self, handler: &HandlerDef, ctx: &mut TypeContext)
    -> TypeCheckResult<()>;

    /// Visit a handler block
    fn visit_handler_block(
        &mut self,
        block: &HandlerBlock,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()>;

    /// Visit a statement
    fn visit_statement(&mut self, stmt: &Statement, ctx: &mut TypeContext) -> TypeCheckResult<()>;

    /// Visit an expression
    fn visit_expression(&mut self, expr: &Expression, ctx: &mut TypeContext)
    -> TypeCheckResult<()>;
}

/// Plugin visitor trait for type checking
pub trait PluginVisitor {
    /// Called before visiting the root node
    fn before_root(&mut self, _root: &mut Root, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting the root node
    fn after_root(&mut self, _root: &mut Root, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a micro agent
    fn before_micro_agent(
        &mut self,
        _agent: &mut MicroAgentDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a micro agent
    fn after_micro_agent(
        &mut self,
        _agent: &mut MicroAgentDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a state definition
    fn before_state(
        &mut self,
        _state: &mut StateDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a state definition
    fn after_state(
        &mut self,
        _state: &mut StateDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a handler
    fn before_handler(
        &mut self,
        _handler: &HandlerDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a handler
    fn after_handler(
        &mut self,
        _handler: &HandlerDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a handler block
    fn before_handler_block(
        &mut self,
        _block: &HandlerBlock,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a handler block
    fn after_handler_block(
        &mut self,
        _block: &HandlerBlock,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a statement
    fn before_statement(
        &mut self,
        _stmt: &Statement,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a statement
    fn after_statement(
        &mut self,
        _stmt: &Statement,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting an expression
    fn before_expression(
        &mut self,
        _expr: &Expression,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting an expression
    fn after_expression(
        &mut self,
        _expr: &Expression,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }
}
