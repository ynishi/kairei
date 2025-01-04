use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;

use crate::{BinaryOperator, ExecutionError, Expression, Literal, RuntimeError, RuntimeResult};

use super::context::{ExecutionContext, VariableAccess};

// 値の型システム
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Duration(std::time::Duration),
    Tuple(Vec<Value>),
    Unit, // Return value for statements
    Null,
}

pub struct ExpressionEvaluator;

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionEvaluator {
    #[async_recursion]
    pub async fn eval_expression(
        &self,
        expr: &Expression,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<Value> {
        match expr {
            Expression::Literal(lit) => Self::eval_literal(lit),
            Expression::Variable(name) => self.eval_variable(name, context).await,
            Expression::StateAccess(path) => {
                self.eval_state_access(path.0.join(".").as_str(), context)
                    .await
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => self.eval_function_call(function, arguments, context).await,
            Expression::BinaryOp { op, left, right } => {
                self.eval_binary_op(op, left, right, context).await
            }
        }
    }

    pub fn new() -> Self {
        Self
    }

    fn eval_literal(lit: &Literal) -> RuntimeResult<Value> {
        Ok(match lit {
            Literal::Integer(i) => Value::Integer(*i),
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::Duration(d) => Value::Duration(*d),
            Literal::List(items) => {
                let mut list = Vec::with_capacity(items.len());
                for item in items {
                    list.push(Self::eval_literal(item)?);
                }
                Value::List(list)
            }
            Literal::Map(items) => {
                let mut map = HashMap::new();
                for (key, value) in items {
                    map.insert(key.clone(), Self::eval_literal(value)?);
                }
                Value::Map(map)
            }
            Literal::Null => Value::Null,
        })
    }

    // 変数の評価
    async fn eval_variable(
        &self,
        name: &str,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<Value> {
        let access = VariableAccess::Local(name.to_string());
        context.get(access).await.map_err(|e| {
            RuntimeError::Execution(ExecutionError::EventError(format!(
                "Variable not found: {}, {}",
                name, e
            )))
        })
    }

    // 状態アクセスの評価
    async fn eval_state_access(
        &self,
        path: &str,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<Value> {
        let access = VariableAccess::State(path.to_string());
        context.get(access).await.map_err(|e| {
            RuntimeError::Execution(ExecutionError::EventError(format!(
                "State not found: {}, {}",
                path, e
            )))
        })
    }

    // 関数呼び出しの評価
    async fn eval_function_call(
        &self,
        function: &str,
        arguments: &[Expression],
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<Value> {
        // 引数を評価
        let mut evaluated_args = Vec::with_capacity(arguments.len());
        for arg in arguments {
            let value = self.eval_expression(arg, context.clone()).await?;
            evaluated_args.push(value);
        }

        // 組み込み関数の評価
        match function {
            "len" => self.eval_len_function(&evaluated_args),
            "sum" => self.eval_sum_function(&evaluated_args),
            "avg" => self.eval_avg_function(&evaluated_args),
            //"max" => self.eval_max_function(&evaluated_args),
            //"min" => self.eval_min_function(&evaluated_args),
            //"now" => self.eval_now_function(),
            //"log" => self.eval_log_function(&evaluated_args),
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!("Unknown function: {}", function),
            ))),
        }
    }

    // 二項演算の評価
    async fn eval_binary_op(
        &self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
        context: Arc<ExecutionContext>,
    ) -> RuntimeResult<Value> {
        let left_val = self.eval_expression(left, context.clone()).await?;
        let right_val = self.eval_expression(right, context).await?;

        match op {
            BinaryOperator::Add => self.eval_add(&left_val, &right_val),
            BinaryOperator::Subtract => self.eval_subtract(&left_val, &right_val),
            BinaryOperator::Multiply => self.eval_multiply(&left_val, &right_val),
            BinaryOperator::Divide => self.eval_divide(&left_val, &right_val),
            BinaryOperator::Equal => self.eval_equal(&left_val, &right_val),
            BinaryOperator::NotEqual => self.eval_not_equal(&left_val, &right_val),
            BinaryOperator::LessThan => self.eval_less_than(&left_val, &right_val),
            BinaryOperator::GreaterThan => self.eval_greater_than(&left_val, &right_val),
            BinaryOperator::LessThanEqual => self.eval_less_than_equal(&left_val, &right_val),
            BinaryOperator::GreaterThanEqual => self.eval_greater_than_equal(&left_val, &right_val),
            BinaryOperator::And => self.eval_and(&left_val, &right_val),
            BinaryOperator::Or => self.eval_or(&left_val, &right_val),
        }
    }

    // 以下、組み込み関数の実装

    fn eval_len_function(&self, args: &[Value]) -> RuntimeResult<Value> {
        if args.len() != 1 {
            return Err(RuntimeError::Execution(ExecutionError::EventError(
                "len function requires exactly one argument".to_string(),
            )));
        }

        match &args[0] {
            Value::String(s) => Ok(Value::Integer(s.len() as i64)),
            Value::List(l) => Ok(Value::Integer(l.len() as i64)),
            Value::Map(m) => Ok(Value::Integer(m.len() as i64)),
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!(
                    "len function requires string, list, or map, but got {:?}",
                    args[0]
                ),
            ))),
        }
    }

    fn eval_sum_function(&self, args: &[Value]) -> RuntimeResult<Value> {
        if args.len() != 1 {
            return Err(RuntimeError::Execution(ExecutionError::EventError(
                "sum function requires exactly one argument".to_string(),
            )));
        }

        match &args[0] {
            Value::List(list) => {
                let mut sum_int = 0i64;
                let mut sum_float = 0.0;
                let mut using_float = false;

                for value in list {
                    match value {
                        Value::Integer(i) => {
                            if using_float {
                                sum_float += *i as f64;
                            } else {
                                sum_int += i;
                            }
                        }
                        Value::Float(f) => {
                            if !using_float {
                                sum_float = sum_int as f64;
                                using_float = true;
                            }
                            sum_float += f;
                        }
                        _ => {
                            return Err(RuntimeError::Execution(ExecutionError::EventError(
                                format!(
                                    "sum function requires list of numbers, but got {:?}",
                                    value
                                ),
                            )));
                        }
                    }
                }

                if using_float {
                    Ok(Value::Float(sum_float))
                } else {
                    Ok(Value::Integer(sum_int))
                }
            }
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!(
                    "sum function requires list of numbers, but got {:?}",
                    args[0]
                ),
            ))),
        }
    }

    fn eval_avg_function(&self, args: &[Value]) -> RuntimeResult<Value> {
        if args.len() != 1 {
            return Err(RuntimeError::Execution(ExecutionError::EventError(
                "avg function requires exactly one argument".to_string(),
            )));
        }

        match &args[0] {
            Value::List(list) => {
                if list.is_empty() {
                    return Err(RuntimeError::Execution(ExecutionError::EventError(
                        "cannot calculate average of empty list".to_string(),
                    )));
                }

                let sum = self.eval_sum_function(args)?;
                match sum {
                    Value::Integer(i) => Ok(Value::Float(i as f64 / list.len() as f64)),
                    Value::Float(f) => Ok(Value::Float(f / list.len() as f64)),
                    _ => unreachable!(),
                }
            }
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!(
                    "avg function requires list of numbers, but got {:?}",
                    args[0]
                ),
            ))),
        }
    }

    // 二項演算子の実装

    fn eval_add(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 + r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l + *r as f64)),
            (Value::String(l), Value::String(r)) => Ok(Value::String(l.clone() + r)),
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!("{:?} + {:?}", left, right),
            ))),
        }
    }

    fn eval_subtract(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 - r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l - *r as f64)),
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!("{:?} - {:?}", left, right),
            ))),
        }
    }

    fn eval_multiply(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 * r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l * *r as f64)),
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!("{:?} * {:?}", left, right),
            ))),
        }
    }

    fn eval_divide(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                if *r == 0 {
                    return Err(RuntimeError::Execution(ExecutionError::EventError(
                        "division by zero".to_string(),
                    )));
                }
                Ok(Value::Float(*l as f64 / *r as f64))
            }
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l / r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 / r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l / *r as f64)),
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!("{:?} / {:?}", left, right),
            ))),
        }
    }

    fn eval_equal(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        Ok(Value::Boolean(left == right))
    }

    fn eval_not_equal(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        Ok(Value::Boolean(left != right))
    }

    fn eval_less_than(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        self.compare_values(left, right, |ordering| ordering.is_lt())
    }

    fn eval_greater_than(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        self.compare_values(left, right, |ordering| ordering.is_gt())
    }

    fn eval_less_than_equal(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        self.compare_values(left, right, |ordering| ordering.is_le())
    }

    fn eval_greater_than_equal(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        self.compare_values(left, right, |ordering| ordering.is_ge())
    }

    fn eval_and(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        match (left, right) {
            (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l && *r)),
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!("{:?} && {:?}", left, right),
            ))),
        }
    }

    fn eval_or(&self, left: &Value, right: &Value) -> RuntimeResult<Value> {
        match (left, right) {
            (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l || *r)),
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!("{:?} || {:?}", left, right),
            ))),
        }
    }

    // ヘルパーメソッド

    fn compare_values<F>(&self, left: &Value, right: &Value, compare: F) -> RuntimeResult<Value>
    where
        F: Fn(std::cmp::Ordering) -> bool,
    {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(compare(l.cmp(r)))),
            (Value::Float(l), Value::Float(r)) => {
                Ok(Value::Boolean(compare(l.partial_cmp(r).unwrap())))
            }
            (Value::Integer(l), Value::Float(r)) => {
                Ok(Value::Boolean(compare((*l as f64).partial_cmp(r).unwrap())))
            }
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Boolean(compare(
                l.partial_cmp(&(*r as f64)).unwrap(),
            ))),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(compare(l.cmp(r)))),
            _ => Err(RuntimeError::Execution(ExecutionError::EventError(
                format!("{:?} <=> {:?}", left, right),
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        eval::context::{AgentInfo, StateAccessMode},
        event_bus::EventBus,
        StateAccessPath,
    };

    use super::*;
    use std::time::Duration;

    // テスト用のヘルパー関数
    async fn setup_context() -> Arc<ExecutionContext> {
        Arc::new(ExecutionContext::new(
            Arc::new(EventBus::new(16)),
            AgentInfo::default(),
            StateAccessMode::ReadWrite,
        ))
    }

    #[tokio::test]
    async fn test_literal_evaluation() {
        let evaluator = ExpressionEvaluator::new();
        let context = setup_context().await;

        // Integer
        let expr = Expression::Literal(Literal::Integer(42));
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Integer(42)));

        // Float
        let expr = Expression::Literal(Literal::Float(3.14));
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Float(f) if (f - 3.14).abs() < f64::EPSILON));

        // String
        let expr = Expression::Literal(Literal::String("hello".to_string()));
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello"));

        // Boolean
        let expr = Expression::Literal(Literal::Boolean(true));
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Duration
        let expr = Expression::Literal(Literal::Duration(Duration::from_secs(60)));
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Duration(d) if d == Duration::from_secs(60)));

        // Null
        let expr = Expression::Literal(Literal::Null);
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Null));
    }

    #[tokio::test]
    async fn test_variable_evaluation() {
        let evaluator = ExpressionEvaluator::new();
        let context = setup_context().await;

        let access = VariableAccess::Local("x".to_string());
        context.set(access, Value::Integer(42)).await.unwrap();
        let access = VariableAccess::Local("name".to_string());
        context
            .set(access, Value::String("Alice".to_string()))
            .await
            .unwrap();

        let expr = Expression::Variable("x".to_string());
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Integer(42)));

        let expr = Expression::Variable("name".to_string());
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::String(s) if s == "Alice"));

        // 存在しない変数の評価
        let expr = Expression::Variable("undefined".to_string());
        let result = evaluator.eval_expression(&expr, context.clone()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_state_access() {
        let evaluator = ExpressionEvaluator::new();
        let context = setup_context().await;

        // 状態を設定
        {
            context.set_state("counter", Value::Integer(10)).unwrap();
            context
                .set_state("settings.enabled", Value::Boolean(true))
                .unwrap();
        }

        // 状態アクセスのテスト
        let expr = Expression::StateAccess(StateAccessPath(vec!["counter".to_string()]));
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Integer(10)));

        let expr = Expression::StateAccess(StateAccessPath(vec![
            "settings".to_string(),
            "enabled".to_string(),
        ]));
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // 存在しない状態へのアクセス
        let expr = Expression::StateAccess(StateAccessPath(vec!["nonexistent".to_string()]));
        let result = evaluator.eval_expression(&expr, context.clone()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_binary_operations() {
        let evaluator = ExpressionEvaluator::new();
        let context = setup_context().await;

        // Addition
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Literal(Literal::Integer(5))),
            right: Box::new(Expression::Literal(Literal::Integer(3))),
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Integer(8)));

        // Mixed type addition
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Literal(Literal::Integer(5))),
            right: Box::new(Expression::Literal(Literal::Float(3.5))),
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Float(f) if (f - 8.5).abs() < f64::EPSILON));

        // String concatenation
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Literal(Literal::String("Hello ".to_string()))),
            right: Box::new(Expression::Literal(Literal::String("World".to_string()))),
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::String(s) if s == "Hello World"));

        // Division by zero
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Divide,
            left: Box::new(Expression::Literal(Literal::Integer(10))),
            right: Box::new(Expression::Literal(Literal::Integer(0))),
        };
        let result = evaluator.eval_expression(&expr, context.clone()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_function_calls() {
        let evaluator = ExpressionEvaluator::new();
        let context = setup_context().await;

        // len function
        let expr = Expression::FunctionCall {
            function: "len".to_string(),
            arguments: vec![Expression::Literal(Literal::String("hello".to_string()))],
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Integer(5)));

        // sum function
        let expr = Expression::FunctionCall {
            function: "sum".to_string(),
            arguments: vec![Expression::Literal(Literal::List(vec![
                Literal::Integer(1),
                Literal::Integer(2),
                Literal::Integer(3),
            ]))],
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Integer(6)));

        // avg function

        let expr = Expression::FunctionCall {
            function: "avg".to_string(),
            arguments: vec![Expression::Literal(Literal::List(vec![
                Literal::Integer(2),
                Literal::Integer(4),
                Literal::Integer(6),
            ]))],
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Float(f) if (f - 4.0).abs() < f64::EPSILON));

        // Invalid function
        let expr = Expression::FunctionCall {
            function: "nonexistent".to_string(),
            arguments: vec![],
        };
        let result = evaluator.eval_expression(&expr, context.clone()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_comparison_operations() {
        let evaluator = ExpressionEvaluator::new();
        let context = setup_context().await;

        // Equal
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Literal(Literal::Integer(5))),
            right: Box::new(Expression::Literal(Literal::Integer(5))),
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Less than
        let expr = Expression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(Expression::Literal(Literal::Float(3.14))),
            right: Box::new(Expression::Literal(Literal::Float(3.15))),
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Greater than equal
        let expr = Expression::BinaryOp {
            op: BinaryOperator::GreaterThanEqual,
            left: Box::new(Expression::Literal(Literal::Integer(10))),
            right: Box::new(Expression::Literal(Literal::Integer(5))),
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[tokio::test]
    async fn test_logical_operations() {
        let evaluator = ExpressionEvaluator::new();
        let context = setup_context().await;

        // AND
        let expr = Expression::BinaryOp {
            op: BinaryOperator::And,
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            right: Box::new(Expression::Literal(Literal::Boolean(false))),
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // OR
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Or,
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            right: Box::new(Expression::Literal(Literal::Boolean(false))),
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Type mismatch
        let expr = Expression::BinaryOp {
            op: BinaryOperator::And,
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            right: Box::new(Expression::Literal(Literal::Integer(1))),
        };
        let result = evaluator.eval_expression(&expr, context.clone()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_expressions() {
        let evaluator = ExpressionEvaluator::new();
        let context = setup_context().await;

        // ネストされた式
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(Expression::Literal(Literal::Integer(5))),
                right: Box::new(Expression::Literal(Literal::Integer(2))),
            }),
            right: Box::new(Expression::Literal(Literal::Integer(3))),
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Integer(13)));

        // 関数呼び出しを含む式
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::FunctionCall {
                function: "len".to_string(),
                arguments: vec![Expression::Literal(Literal::String("hello".to_string()))],
            }),
            right: Box::new(Expression::Literal(Literal::Integer(5))),
        };
        let result = evaluator
            .eval_expression(&expr, context.clone())
            .await
            .unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }
}
