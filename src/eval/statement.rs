use std::sync::Arc;

use async_recursion::async_recursion;

use super::{
    context::{ExecutionContext, StateAccessMode, VariableAccess},
    expression::{ExpressionEvaluator, Value},
};
use crate::eval::evaluator::{EvalError, EvalResult};
use crate::{
    event_bus::{self, Event},
    event_registry, Argument, ErrorHandlerBlock, EventType, Expression, Statement,
};

/// 文の評価結果を表す型
#[derive(Debug, Clone)]
pub enum StatementResult {
    /// 値を返す文 (Unitを含む)
    Value(Value),

    /// 制御フロー
    Control(ControlFlow),
}

#[derive(Debug, Clone)]
pub enum ControlFlow {
    Break(Value), // breakに値を含められるように
    Continue,
    Return(Value),
}

/// 基本的なStatement評価の実装
pub struct StatementEvaluator {
    pub expression_evaluator: Arc<ExpressionEvaluator>,
}

impl StatementEvaluator {
    #[async_recursion]
    pub async fn eval_statement(
        &self,
        statement: &Statement,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<StatementResult> {
        // Dispatch to the appropriate evaluation method based on the statement type
        match statement {
            Statement::Expression(expr) => Ok(StatementResult::Value(
                self.eval_expression(expr, context).await?,
            )),
            Statement::Return(expr) => Ok(StatementResult::Control(ControlFlow::Return(
                self.eval_return(expr, context).await?,
            ))),
            Statement::Assignment { target, value } => Ok(StatementResult::Value(
                self.eval_assignment(target, value, context).await?,
            )),
            Statement::Emit {
                event_type,
                parameters,
                target,
            } => Ok(StatementResult::Value(
                self.eval_emit(event_type, parameters, target, context)
                    .await?,
            )),
            Statement::Block(block) => self.eval_block(block, context).await,
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.eval_if(condition, then_block, else_block, context)
                    .await
            }
            Statement::WithError {
                statement,
                error_handler_block,
            } => {
                self.eval_with_error(statement, error_handler_block, context)
                    .await
            }
        }
    }
}

impl Default for StatementEvaluator {
    fn default() -> Self {
        Self {
            expression_evaluator: Arc::new(ExpressionEvaluator::new()),
        }
    }
}

impl StatementEvaluator {
    pub fn new(expression_evaluator: Arc<ExpressionEvaluator>) -> Self {
        Self {
            expression_evaluator,
        }
    }

    async fn eval_return(
        &self,
        expr: &Expression,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Value> {
        self.expression_evaluator
            .eval_expression(expr, context)
            .await
    }

    pub async fn eval_expression(
        &self,
        expr: &Expression,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Value> {
        self.expression_evaluator
            .eval_expression(expr, context)
            .await
    }

    async fn eval_assignment(
        &self,
        targets: &[Expression],
        value: &Expression,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Value> {
        let value = self
            .expression_evaluator
            .eval_expression(value, context.clone())
            .await?;

        match (targets.len(), &value) {
            // 単一のターゲットへの代入
            (1, _) => {
                let access = self.get_variable_access(&targets[0])?;
                context.set(access, value.clone()).await?;
            }
            // タプル値を複数のターゲットに分配
            (n, Value::Tuple(values)) if n == values.len() => {
                for (target, value) in targets.iter().zip(values) {
                    let access = self.get_variable_access(target)?;
                    context.set(access, value.clone()).await?;
                }
            }
            // 不整合な場合（ターゲットの数と値の数が合わない）
            (n, Value::Tuple(values)) => {
                return Err(EvalError::InvalidOperation(format!(
                    "mismatched assignment: {} targets but got {} values",
                    n,
                    values.len()
                )))
            }
            // タプルでない値を複数のターゲットに代入しようとした場合
            (n, _) => {
                return Err(EvalError::InvalidOperation(format!(
                    "cannot assign single value to {} targets",
                    n
                )))
            }
        }
        Ok(Value::Unit)
    }

    fn get_variable_access(&self, target: &Expression) -> EvalResult<VariableAccess> {
        match target {
            Expression::Variable(name) => Ok(VariableAccess::Local(name.clone())),
            Expression::StateAccess(path) => Ok(VariableAccess::State(path.to_string())),
            _ => Err(EvalError::InvalidOperation(
                "invalid access expression".to_string(),
            )),
        }
    }

    async fn eval_emit(
        &self,
        event_type: &EventType,
        parameters: &[Argument],
        target: &Option<String>,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Value> {
        // パラメータの評価
        let mut evaluated_params = self
            .expression_evaluator
            .eval_arguments(parameters, context.clone())
            .await?;
        if let Some(to) = target {
            evaluated_params.insert("to".to_string(), Value::String(to.clone()));
        }
        let event_type = event_registry::EventType::from(event_type);
        let event_params = evaluated_params
            .iter()
            .map(|(k, v)| (k.clone(), event_bus::Value::from(v.clone())))
            .collect();

        // イベントの構築と発行
        let event = Event::new(&event_type, &event_params);
        context.emit_event(event).await?;

        Ok(Value::Unit)
    }

    async fn eval_if(
        &self,
        condition: &Expression,
        then_block: &[Statement],
        else_block: &Option<Vec<Statement>>,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<StatementResult> {
        let condition_value = self
            .expression_evaluator
            .eval_expression(condition, context.clone())
            .await?;

        match condition_value {
            Value::Boolean(true) => self.eval_block(then_block, context.clone()).await,
            Value::Boolean(false) => {
                if let Some(else_block) = else_block {
                    self.eval_block(else_block, context).await
                } else {
                    Ok(StatementResult::Value(Value::Unit))
                }
            }
            _ => Err(EvalError::Eval(format!(
                "{}, {:?}",
                "boolean", condition_value,
            ))),
        }
    }

    pub async fn eval_block(
        &self,
        statements: &[Statement],
        context: Arc<ExecutionContext>,
    ) -> EvalResult<StatementResult> {
        let mut last = Value::Unit;
        for stmt in statements.iter() {
            let result = self.eval_statement(stmt, context.clone()).await?;
            match result {
                StatementResult::Value(value) => {
                    last = value;
                }
                StatementResult::Control(ControlFlow::Break(value)) => {
                    return Ok(StatementResult::Value(value))
                }
                StatementResult::Control(ControlFlow::Continue) => continue,
                StatementResult::Control(ControlFlow::Return(value)) => {
                    return Ok(StatementResult::Control(ControlFlow::Return(value)))
                }
            }
        }
        Ok(StatementResult::Value(last))
    }

    async fn eval_with_error(
        &self,
        statement: &Statement,
        error_handler_block: &ErrorHandlerBlock,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<StatementResult> {
        match self.eval_statement(statement, context.clone()).await {
            Ok(value) => Ok(value),
            Err(error) => {
                // Create new scope for error handler
                let error_context = Arc::new(context.fork(Some(StateAccessMode::ReadOnly)).await);

                // Bind error if binding name is provided
                if let Some(binding) = &error_handler_block.error_binding {
                    error_context
                        .set_variable(binding.as_str(), Value::Error(error.to_string()))
                        .await
                        .map_err(|e| EvalError::Eval(format!("Error Binding Failed: {}", e)))?;
                }

                // Execute error handler block
                let result = self
                    .eval_block(&error_handler_block.error_handler_statements, error_context)
                    .await;

                // Return Unit or propagate error
                // TODO: check return type compile(parse) time.
                result.map(|_| StatementResult::Value(Value::Unit))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use dashmap::DashMap;
    use event_bus::EventBus;

    use crate::{
        config::ContextConfig,
        eval::context::{AgentInfo, StateAccessMode},
        provider::provider_registry::ProviderInstance,
        BinaryOperator, Literal,
    };

    use super::*;
    use std::sync::Arc;

    // テスト用のヘルパー関数
    async fn setup_context() -> Arc<ExecutionContext> {
        // 基本的なコンテキストのセットアップ
        Arc::new(ExecutionContext::new(
            Arc::new(EventBus::new(16)),
            AgentInfo::default(),
            StateAccessMode::ReadWrite,
            ContextConfig::default(),
            Arc::new(ProviderInstance::default()),
            Arc::new(DashMap::new()),
            vec![],
        ))
    }

    #[tokio::test]
    async fn test_expression_statement() {
        let evaluator = StatementEvaluator::new(Arc::new(ExpressionEvaluator::new()));
        let context = setup_context().await;

        // 単純な式の評価
        let stmt = Statement::Expression(Expression::Literal(Literal::Integer(42)));
        let result = evaluator
            .eval_statement(&stmt, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, StatementResult::Value(Value::Integer(42))));

        // 二項演算式の評価
        let stmt = Statement::Expression(Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Literal(Literal::Integer(10))),
            right: Box::new(Expression::Literal(Literal::Integer(5))),
        });
        let result = evaluator.eval_statement(&stmt, context).await.unwrap();
        assert!(matches!(result, StatementResult::Value(Value::Integer(15))));
    }

    #[tokio::test]
    async fn test_assignment_statement() {
        let evaluator = StatementEvaluator::new(Arc::new(ExpressionEvaluator::new()));
        let context = setup_context().await;

        // 基本的な代入
        let stmt = Statement::Assignment {
            target: vec![Expression::Variable("x".to_string())],
            value: Expression::Literal(Literal::Integer(42)),
        };
        let result = evaluator
            .eval_statement(&stmt, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, StatementResult::Value(Value::Unit)));

        // 代入された値の確認
        let value = context.get_variable("x").await.unwrap();
        assert_eq!(value, Value::Integer(42));

        // 代入後の演算
        let stmt = Statement::Assignment {
            target: vec![Expression::Variable("x".to_string())],
            value: Expression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(Expression::Literal(Literal::Integer(10))),
                right: Box::new(Expression::Variable("x".to_string())),
            },
        };
        evaluator
            .eval_statement(&stmt, context.clone())
            .await
            .unwrap();
        let value = context.get_variable("x").await.unwrap();
        assert_eq!(value, Value::Integer(52));
    }

    #[tokio::test]
    async fn test_block_statement() {
        let evaluator = StatementEvaluator::new(Arc::new(ExpressionEvaluator::new()));
        let context = setup_context().await;

        // 複数のステートメントを含むブロック
        let stmt = Statement::Block(vec![
            Statement::Assignment {
                target: vec![Expression::Variable("x".to_string())],
                value: Expression::Literal(Literal::Integer(10)),
            },
            Statement::Assignment {
                target: vec![Expression::Variable("y".to_string())],
                value: Expression::Literal(Literal::Integer(20)),
            },
            Statement::Expression(Expression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(Expression::Variable("y".to_string())),
                right: Box::new(Expression::Variable("x".to_string())),
            }),
        ]);

        let result = evaluator
            .eval_statement(&stmt, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, StatementResult::Value(Value::Integer(30))));
    }

    #[tokio::test]
    async fn test_if_statement() {
        let evaluator = StatementEvaluator::new(Arc::new(ExpressionEvaluator::new()));
        let context = setup_context().await;

        // true条件のIf文
        let stmt = Statement::If {
            condition: Expression::Literal(Literal::Boolean(true)),
            then_block: vec![Statement::Expression(Expression::Literal(
                Literal::Integer(1),
            ))],
            else_block: Some(vec![Statement::Expression(Expression::Literal(
                Literal::Integer(2),
            ))]),
        };
        let result = evaluator
            .eval_statement(&stmt, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, StatementResult::Value(Value::Integer(1))));

        // false条件のIf文
        let stmt = Statement::If {
            condition: Expression::Literal(Literal::Boolean(false)),
            then_block: vec![Statement::Expression(Expression::Literal(
                Literal::Integer(1),
            ))],
            else_block: Some(vec![Statement::Expression(Expression::Literal(
                Literal::Integer(2),
            ))]),
        };
        let result = evaluator
            .eval_statement(&stmt, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, StatementResult::Value(Value::Integer(2))));

        // else節なしのIf文
        let stmt = Statement::If {
            condition: Expression::Literal(Literal::Boolean(false)),
            then_block: vec![Statement::Expression(Expression::Literal(
                Literal::Integer(1),
            ))],
            else_block: None,
        };
        let result = evaluator.eval_statement(&stmt, context).await.unwrap();
        assert!(matches!(result, StatementResult::Value(Value::Unit)));
    }

    #[tokio::test]
    async fn test_return_statement() {
        let evaluator = StatementEvaluator::new(Arc::new(ExpressionEvaluator::new()));
        let context = setup_context().await;

        // 単純なreturn
        let stmt = Statement::Return(Expression::Literal(Literal::Integer(42)));
        let result = evaluator
            .eval_statement(&stmt, context.clone())
            .await
            .unwrap();
        assert!(matches!(
            result,
            StatementResult::Control(ControlFlow::Return(Value::Integer(42)))
        ));

        // ブロック内のreturn
        let stmt = Statement::Block(vec![
            Statement::Assignment {
                target: vec![Expression::Variable("x".to_string())],
                value: Expression::Literal(Literal::Integer(10)),
            },
            Statement::Return(Expression::Variable("x".to_string())),
            Statement::Expression(Expression::Literal(Literal::Integer(20))), // この行は実行されない
        ]);
        let result = evaluator.eval_statement(&stmt, context).await.unwrap();
        assert!(matches!(
            result,
            StatementResult::Control(ControlFlow::Return(Value::Integer(10)))
        ));
    }

    #[tokio::test]
    async fn test_complex_nested_statements() {
        let evaluator = StatementEvaluator::new(Arc::new(ExpressionEvaluator::new()));
        let context = setup_context().await;

        // 複雑なネストされた文の評価
        let stmt = Statement::Block(vec![
            Statement::Assignment {
                target: vec![Expression::Variable("x".to_string())],
                value: Expression::Literal(Literal::Integer(10)),
            },
            Statement::If {
                condition: Expression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(Expression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(Expression::Variable("x".to_string())),
                        right: Box::new(Expression::Literal(Literal::Integer(5))),
                    }),
                    right: Box::new(Expression::Literal(Literal::Integer(15))),
                },
                then_block: vec![
                    Statement::Assignment {
                        target: vec![Expression::Variable("y".to_string())],
                        value: Expression::Literal(Literal::Integer(20)),
                    },
                    Statement::Return(Expression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(Expression::Variable("x".to_string())),
                        right: Box::new(Expression::Variable("y".to_string())),
                    }),
                ],
                else_block: Some(vec![Statement::Return(Expression::Literal(
                    Literal::Integer(0),
                ))]),
            },
        ]);

        let result = evaluator.eval_statement(&stmt, context).await.unwrap();
        assert!(matches!(
            result,
            StatementResult::Control(ControlFlow::Return(Value::Integer(30)))
        ));
    }

    // エラーケースのテスト
    #[tokio::test]
    async fn test_error_cases() {
        let evaluator = StatementEvaluator::new(Arc::new(ExpressionEvaluator::new()));
        let context = setup_context().await;

        // 未定義変数の参照
        let stmt = Statement::Expression(Expression::Variable("undefined".to_string()));
        let result = evaluator.eval_statement(&stmt, context.clone()).await;
        assert!(result.is_err());

        // 型の不一致（数値の加算に文字列を使用）
        let stmt = Statement::Expression(Expression::BinaryOp {
            left: Box::new(Expression::Literal(Literal::Integer(10))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::String("20".to_string()))),
        });
        let result = evaluator.eval_statement(&stmt, context).await;
        assert!(result.is_err());
    }
}
