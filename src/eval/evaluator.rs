use super::{
    context::{ContextError, ExecutionContext},
    expression::Value,
    statement::{ControlFlow, StatementEvaluator, StatementResult},
};
use crate::{
    event_registry::EventType, provider::types::ProviderError, runtime::RuntimeError, Expression,
    HandlerBlock,
};
use std::sync::Arc;

/// Top-level evaluator for the KAIREI DSL execution pipeline
///
/// The Evaluator is responsible for executing KAIREI DSL code at runtime,
/// serving as the primary entry point for the evaluation system. It orchestrates
/// the evaluation of handler blocks, answer handler blocks, and expressions,
/// delegating to specialized evaluators for specific language constructs.
///
/// # Responsibilities
///
/// - Evaluating handler blocks for event processing
/// - Evaluating answer handler blocks for request/response patterns
/// - Evaluating expressions for value computation
/// - Managing evaluation context and error handling
///
/// # Architecture
///
/// The Evaluator works with several key components:
///
/// - `StatementEvaluator`: Handles statement evaluation and control flow
/// - `ExecutionContext`: Manages runtime state and environment
/// - `ExpressionEvaluator`: Evaluates expressions (via StatementEvaluator)
///
/// # Example Usage
///
/// ```ignore
/// use kairei::eval::{Evaluator, ExecutionContext};
/// use kairei::HandlerBlock;
/// use std::sync::Arc;
///
/// async fn evaluate_handler(handler_block: &HandlerBlock) {
///     let evaluator = Evaluator::new();
///     let context = Arc::new(ExecutionContext::new(/* ... */));
///     
///     let result = evaluator.eval_handler_block(handler_block, context).await;
///     // Process result...
/// }
/// ```
#[derive(Default)]
pub struct Evaluator {
    statement_evaluator: StatementEvaluator,
}

impl Evaluator {
    /// Creates a new Evaluator instance with default configuration
    ///
    /// This initializes a new Evaluator with a default StatementEvaluator,
    /// which in turn contains a default ExpressionEvaluator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates a handler block in the given execution context
    ///
    /// This is the primary entry point for evaluating event handler blocks in the KAIREI
    /// DSL. It processes the statements within the handler block sequentially and
    /// manages error handling and context updates.
    ///
    /// # Parameters
    ///
    /// - `block`: The handler block to evaluate
    /// - `context`: The execution context containing runtime state
    ///
    /// # Returns
    ///
    /// - `Ok(StatementResult)`: The result of the evaluation
    /// - `Err(EvalError)`: If an unrecoverable error occurs during evaluation
    ///
    /// # Error Handling
    ///
    /// This method handles errors by:
    /// 1. Converting evaluation errors to context failures
    /// 2. Emitting failure events through the context
    /// 3. Returning a Unit value to allow execution to continue
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kairei::HandlerBlock;
    /// use kairei::eval::{Evaluator, ExecutionContext};
    /// use std::sync::Arc;
    /// 
    /// let handler_block = HandlerBlock { statements: vec![/* ... */] };
    /// let evaluator = Evaluator::new();
    /// let context = Arc::new(ExecutionContext::new(/* ... */));
    /// let result = evaluator.eval_handler_block(&handler_block, context).await?;
    /// ```
    pub async fn eval_handler_block(
        &self,
        block: &HandlerBlock,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<StatementResult> {
        let result = self
            .statement_evaluator
            .eval_block(&block.statements, context.clone())
            .await;
        let res = match result {
            Ok(StatementResult::Value(Value::Error(e))) => Err(ContextError::Failure(e)),
            Err(e) => Err(ContextError::Failure(e.to_string())),
            Ok(s) => Ok(s),
        };
        match res {
            Ok(s) => Ok(s),
            Err(e) => {
                let _ = context.emit_failure(e).await;
                Ok(StatementResult::Value(Value::Unit))
            }
        }
    }

    /// Evaluates an answer handler block and sends the response
    ///
    /// This method is specifically designed for request-response patterns in the KAIREI
    /// system. It evaluates the handler block and automatically sends the appropriate
    /// response based on the evaluation result.
    ///
    /// # Parameters
    ///
    /// - `block`: The answer handler block to evaluate
    /// - `context`: The execution context containing runtime state
    /// - `event`: The event type that triggered this handler
    ///
    /// # Returns
    ///
    /// - `Ok(StatementResult)`: The result of the evaluation (typically Unit)
    /// - `Err(EvalError)`: If an unrecoverable error occurs during evaluation
    ///
    /// # Response Handling
    ///
    /// The method handles different evaluation results:
    /// - `Return(Value::Ok(v))`: Sends a success response with the inner value
    /// - `Return(Value::Err(e))`: Sends a failure response with the error
    /// - `Return(Value::Error(e))`: Returns an unhandled exception error
    /// - `Value::Unit`: Sends a success response with Unit value
    /// - Other results: Returns an unexpected result error
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kairei::HandlerBlock;
    /// use kairei::event_registry::EventType;
    /// use kairei::eval::{Evaluator, ExecutionContext};
    /// use std::sync::Arc;
    /// 
    /// let answer_block = HandlerBlock { statements: vec![/* ... */] };
    /// let evaluator = Evaluator::new();
    /// let context = Arc::new(ExecutionContext::new(/* ... */));
    /// let event = EventType::Request { /* ... */ };
    /// let result = evaluator.eval_answer_handler_block(&answer_block, context, event).await?;
    /// ```
    pub async fn eval_answer_handler_block(
        &self,
        block: &HandlerBlock,
        context: Arc<ExecutionContext>,
        event: EventType,
    ) -> EvalResult<StatementResult> {
        let result = self
            .statement_evaluator
            .eval_block(&block.statements, context.clone())
            .await;
        match result {
            Ok(StatementResult::Control(ControlFlow::Return(value))) => {
                let response = match value {
                    Value::Ok(inner) => Ok(*inner),
                    Value::Err(inner) => Err(RuntimeError::EvalFailure(*inner)),
                    // Exception case returns an error
                    Value::Error(e) => {
                        return Err(EvalError::Eval(format!("Unhandled exception: {:?}", e)));
                    }
                    other => Ok(other),
                };
                context
                    .send_response(event, response)
                    .await
                    .map_err(|e| EvalError::SendResponseFailed(format!("error: {}", e)))?;
            }
            Ok(StatementResult::Value(Value::Unit)) => {
                context
                    .send_response(event, Ok(Value::Unit))
                    .await
                    .map_err(|e| EvalError::SendResponseFailed(format!("error: {}", e)))?;
            }
            Err(e) => {
                context
                    .send_response(event, Err(RuntimeError::from(e)))
                    .await
                    .map_err(|e| EvalError::SendResponseFailed(format!("error: {}", e)))?;
            }
            // Other cases return an error
            Ok(s) => {
                return Err(EvalError::Eval(format!(
                    "Unexpected statement result: {:?}",
                    s
                )))?;
            }
        }
        Ok(StatementResult::Value(Value::Unit))
    }

    /// Evaluates an expression and returns its value
    ///
    /// This method provides a direct way to evaluate expressions without the context
    /// of a handler block. It delegates to the StatementEvaluator's expression
    /// evaluation functionality.
    ///
    /// # Parameters
    ///
    /// - `expression`: The expression to evaluate
    /// - `context`: The execution context containing runtime state
    ///
    /// # Returns
    ///
    /// - `Ok(Value)`: The evaluated value of the expression
    /// - `Err(EvalError)`: If an error occurs during evaluation
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kairei::Expression;
    /// use kairei::tokenizer::literal::Literal;
    /// use kairei::expression::Value;
    /// use kairei::eval::{Evaluator, ExecutionContext};
    /// use std::sync::Arc;
    /// 
    /// let expr = Expression::Literal(Literal::Integer(42));
    /// let evaluator = Evaluator::new();
    /// let context = Arc::new(ExecutionContext::new(/* ... */));
    /// let value = evaluator.eval_expression(&expr, context).await?;
    /// assert_eq!(value, Value::Integer(42));
    /// ```
    pub async fn eval_expression(
        &self,
        expression: &Expression,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Value> {
        self.statement_evaluator
            .eval_expression(expression, context)
            .await
    }
}

// eval error
#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("Eval error: {0}")]
    Eval(String),
    #[error("Context error: {0}")]
    Context(#[from] ContextError),
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Send response failed: {0}")]
    SendResponseFailed(String),
    #[error("Variable not found: {name}, {messages}")]
    VariableNotFound { name: String, messages: String },
    #[error("Parameter not found: {name}")]
    ParameterNotFound { name: String },
    #[error("Invalid parameter: {name}, {value}")]
    InvalidParameter { name: String, value: String },
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type EvalResult<T> = Result<T, EvalError>;
