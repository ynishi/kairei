use std::sync::Arc;

use crate::{HandlerBlock, RuntimeResult};

use super::{
    context::ExecutionContext,
    statement::{StatementEvaluator, StatementResult},
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
}
