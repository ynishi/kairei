use std::sync::Arc;

use crate::{
    event_registry::EventType, ExecutionError, Expression, HandlerBlock, RuntimeError,
    RuntimeResult,
};

use super::{
    context::{ContextError, ExecutionContext},
    expression::Value,
    statement::{ControlFlow, StatementEvaluator, StatementResult},
};

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
    ) -> RuntimeResult<StatementResult> {
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
    ) -> RuntimeResult<StatementResult> {
        let result = self
            .statement_evaluator
            .eval_block(&block.statements, context.clone())
            .await;
        match result {
            Ok(StatementResult::Control(ControlFlow::Return(value))) => {
                context.send_response(event, Ok(value)).await.map_err(|e| {
                    RuntimeError::Execution(ExecutionError::EvalError(format!(
                        "Failed to send response: {}",
                        e
                    )))
                })?;
            }
            Ok(StatementResult::Value(Value::Unit)) => {
                context
                    .send_response(event, Ok(Value::Unit))
                    .await
                    .map_err(|e| {
                        RuntimeError::Execution(ExecutionError::EvalError(format!(
                            "Failed to send response: {}",
                            e
                        )))
                    })?;
            }
            Err(e) => {
                context.send_response(event, Err(e)).await.map_err(|e| {
                    RuntimeError::Execution(ExecutionError::EvalError(format!(
                        "Failed to send response: {}",
                        e
                    )))
                })?;
            }
            // その他の場合はエラーを返す
            Ok(s) => {
                return Err(RuntimeError::Execution(ExecutionError::EvalError(format!(
                    "Unexpected statement result: {:?}",
                    s
                ))))
            }
        }
        Ok(StatementResult::Value(Value::Unit))
    }

    pub async fn eval_expression(
        &self,
        expression: &Expression,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<Value> {
        self.statement_evaluator
            .eval_expression(expression, context)
            .await
    }
}
