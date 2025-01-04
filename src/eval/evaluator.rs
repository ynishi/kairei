use std::sync::Arc;

use crate::{event_registry::EventType, Expression, HandlerBlock, RuntimeError, RuntimeResult};

use super::{
    context::ExecutionContext,
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
        self.statement_evaluator
            .eval_block(&block.statements, context)
            .await
    }

    pub async fn eval_answer_handler_block(
        &self,
        block: &HandlerBlock,
        context: Arc<ExecutionContext>,
        event: EventType,
    ) -> RuntimeResult<StatementResult> {
        let res = self
            .statement_evaluator
            .eval_block(&block.statements, context.clone())
            .await?;
        if let StatementResult::Control(ControlFlow::Return(value)) = res {
            context.send_response(event, value).await.map_err(|e| {
                RuntimeError::Execution(crate::ExecutionError::EvalError(format!(
                    "Failed to send response: {}",
                    e
                )))
            })?;
        } else {
            context
                .send_response(event, Value::Unit)
                .await
                .map_err(|e| {
                    RuntimeError::Execution(crate::ExecutionError::EvalError(format!(
                        "Failed to send response: {}",
                        e
                    )))
                })?;
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
