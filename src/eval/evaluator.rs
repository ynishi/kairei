use super::{
    context::{ContextError, ExecutionContext},
    expression::Value,
    statement::{ControlFlow, StatementEvaluator, StatementResult},
};
use crate::{event_registry::EventType, runtime::RuntimeError, Expression, HandlerBlock};
use std::sync::Arc;

#[derive(Default)]
pub struct Evaluator {
    statement_evaluator: StatementEvaluator,
}

impl Evaluator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Top level entry point for MicroAgent Evaluator
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
                context
                    .send_response(event, Ok(value))
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
            // その他の場合はエラーを返す
            Ok(s) => {
                return Err(EvalError::Eval(format!(
                    "Unexpected statement result: {:?}",
                    s
                )))?;
            }
        }
        Ok(StatementResult::Value(Value::Unit))
    }

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
    #[error("Send response failed: {0}")]
    SendResponseFailed(String),
    #[error("Variable not found: {name}, {messages}")]
    VariableNotFound { name: String, messages: String },
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type EvalResult<T> = Result<T, EvalError>;
