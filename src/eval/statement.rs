use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;

use tracing::debug;
use uuid::Uuid;

use crate::{
    event_bus::{self, Event},
    event_registry, Argument, AwaitType, EventType, ExecutionError, Expression, RequestOptions,
    RequestType, RuntimeError, RuntimeResult, Statement,
};

use super::{
    context::{ExecutionContext, VariableAccess},
    expression::{ExpressionEvaluator, Value},
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
    ) -> RuntimeResult<StatementResult> {
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
            Statement::Request {
                agent,
                request_type,
                parameters,
                options,
            } => Ok(StatementResult::Value(
                self.eval_request(agent, request_type, parameters, options, context)
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
            Statement::Await(await_type) => self.eval_await(await_type, context).await,
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
    ) -> RuntimeResult<Value> {
        self.expression_evaluator
            .eval_expression(expr, context)
            .await
    }

    pub async fn eval_expression(
        &self,
        expr: &Expression,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<Value> {
        self.expression_evaluator
            .eval_expression(expr, context)
            .await
    }

    async fn eval_assignment(
        &self,
        target: &Expression,
        value: &Expression,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<Value> {
        let value = self
            .expression_evaluator
            .eval_expression(value, context.clone())
            .await?;

        // targetの評価結果に基づいて適切な場所に値を格納
        let access = match target {
            Expression::Variable(name) => VariableAccess::Local(name.clone()),
            Expression::StateAccess(path) => VariableAccess::State(path.to_string()),
            _ => {
                return Err(RuntimeError::Execution(ExecutionError::InvalidOperation(
                    "invalid".to_string(),
                )))
            }
        };
        context.set(access, value.clone()).await.map_err(|e| {
            RuntimeError::Execution(ExecutionError::InvalidOperation(e.to_string()))
        })?;

        Ok(Value::Unit)
    }

    async fn eval_emit(
        &self,
        event_type: &EventType,
        parameters: &[Argument],
        target: &Option<String>,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<Value> {
        // パラメータの評価
        let mut evaluated_params = self.eval_argumants(parameters, context.clone()).await?;
        if let Some(to) = target {
            evaluated_params.insert("to".to_string(), event_bus::Value::String(to.clone()));
        }
        let event_type = event_registry::EventType::from(event_type);

        // イベントの構築と発行
        let event = Event::new(&event_type, &evaluated_params);
        context.emit_event(event).await.map_err(|e| {
            RuntimeError::Execution(ExecutionError::EvaluationFailed(e.to_string()))
        })?;

        Ok(Value::Unit)
    }

    async fn eval_argumants(
        &self,
        arguments: &[Argument],
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<HashMap<String, event_bus::Value>> {
        let mut evaluated_params = HashMap::new();
        for (i, param) in arguments.iter().enumerate() {
            let (name, value) = match param {
                Argument::Named { name, value } => {
                    let value = self
                        .expression_evaluator
                        .eval_expression(value, context.clone())
                        .await?;
                    (name.clone(), value)
                }
                Argument::Positional(value) => {
                    let value = self
                        .expression_evaluator
                        .eval_expression(value, context.clone())
                        .await?;
                    ((i + 1).to_string(), value)
                }
            };
            evaluated_params.insert(name, event_bus::Value::from(value));
        }
        Ok(evaluated_params)
    }

    async fn eval_request(
        &self,
        agent: &str,
        request_type: &RequestType,
        parameters: &[Argument],
        _options: &Option<RequestOptions>,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<Value> {
        // パラメータの評価
        let evaluated_params = self.eval_argumants(parameters, context.clone()).await?;

        // リクエストの構築と送信
        let request = Event {
            event_type: event_registry::EventType::Request {
                request_id: Uuid::new_v4().to_string(),
                requester: context.agent_name().clone(),
                responder: agent.to_string(),
                request_type: request_type.to_string(),
            },
            parameters: evaluated_params,
        };
        debug!("Create Request: {:?}", request);
        let response_event = context.send_request(request).await.map_err(|e| {
            RuntimeError::Execution(ExecutionError::EvaluationFailed(e.to_string()))
        })?;
        debug!("Got Reponse: {:?}", response_event);
        let response = response_event
            .parameters
            .get("response")
            .ok_or(RuntimeError::Execution(ExecutionError::EvaluationFailed(
                "response not found".to_string(),
            )))?
            .clone()
            .into();
        Ok(response)
    }

    async fn eval_if(
        &self,
        condition: &Expression,
        then_block: &[Statement],
        else_block: &Option<Vec<Statement>>,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<StatementResult> {
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
            _ => Err(RuntimeError::Execution(ExecutionError::EvalError(format!(
                "{}, {:?}",
                "boolean", condition_value,
            )))),
        }
    }

    pub async fn eval_block(
        &self,
        statements: &[Statement],
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<StatementResult> {
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

    async fn eval_await(
        &self,
        await_type: &AwaitType,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<StatementResult> {
        match await_type {
            AwaitType::Block(statements) => {
                let mut futures = Vec::with_capacity(statements.len());

                // 各ステートメントに対して新しいコンテキストをフォーク
                for stmt in statements {
                    let forked_context = Arc::new(context.fork(None).await);
                    futures.push(self.eval_statement(stmt, forked_context));
                }

                // 並列実行して結果を収集
                let results = futures::future::join_all(futures).await;
                let mut values = Vec::new();

                for result in results {
                    match result? {
                        StatementResult::Value(v) => values.push(v),
                        StatementResult::Control(cf) => return Ok(StatementResult::Control(cf)),
                    }
                }

                Ok(StatementResult::Value(Value::Tuple(values)))
            }
            AwaitType::Single(statement) => {
                // 単一のStatementを実行して完了を待つ
                self.eval_statement(statement, context).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use event_bus::EventBus;

    use crate::{
        eval::context::{AgentInfo, StateAccessMode},
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
            target: Expression::Variable("x".to_string()),
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
            target: Expression::Variable("x".to_string()),
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
                target: Expression::Variable("x".to_string()),
                value: Expression::Literal(Literal::Integer(10)),
            },
            Statement::Assignment {
                target: Expression::Variable("y".to_string()),
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
                target: Expression::Variable("x".to_string()),
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
                target: Expression::Variable("x".to_string()),
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
                        target: Expression::Variable("y".to_string()),
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
