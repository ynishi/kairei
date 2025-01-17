use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;

use super::context::{ExecutionContext, VariableAccess};
use crate::eval::evaluator::{EvalError, EvalResult};
use crate::provider::request::{
    ExecutionState, ProviderContext, ProviderRequest, ProviderResponse, RequestInput,
};
use crate::provider::types::{ProviderError, ProviderInstance};
use crate::timestamp::Timestamp;
use crate::{
    ast, Argument, BinaryOperator, Expression, Literal, Policy, RetryDelay, ThinkAttributes,
};

// 値の型システム
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Value {
    Integer(i64),
    UInteger(u64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Duration(std::time::Duration),
    Delay(RetryDelay),
    Tuple(Vec<Value>),
    Unit,          // Return value for statements
    Error(String), // Error name for handling.
    #[default]
    Null,
}

pub struct ExpressionEvaluator;

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ProviderResponse> for Value {
    fn from(response: ProviderResponse) -> Self {
        let mut hash_map = HashMap::new();
        hash_map.insert("output".to_string(), Value::String(response.output));
        Value::Map(hash_map)
    }
}

impl ExpressionEvaluator {
    #[async_recursion]
    pub async fn eval_expression(
        &self,
        expr: &Expression,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Value> {
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
            Expression::Think { args, with_block } => {
                self.eval_think(args, with_block, context).await
            }
        }
    }

    pub fn new() -> Self {
        Self
    }

    pub async fn eval_arguments(
        &self,
        arguments: &[Argument],
        context: Arc<ExecutionContext>,
    ) -> EvalResult<HashMap<String, Value>> {
        self.eval_arguments_inner(arguments, context, false).await
    }

    pub async fn eval_arguments_detect_name(
        &self,
        arguments: &[Argument],
        context: Arc<ExecutionContext>,
    ) -> EvalResult<HashMap<String, Value>> {
        self.eval_arguments_inner(arguments, context, true).await
    }

    async fn eval_arguments_inner(
        &self,
        arguments: &[Argument],
        context: Arc<ExecutionContext>,
        is_named: bool,
    ) -> EvalResult<HashMap<String, Value>> {
        let mut evaluated_params = HashMap::new();
        for (i, param) in arguments.iter().enumerate() {
            let (name, value) = match param {
                Argument::Named { name, value } => {
                    let value = self.eval_expression(value, context.clone()).await?;
                    (name.clone(), value)
                }
                Argument::Positional(value) => {
                    let indexed_name = (i + 1).to_string();
                    let name = if is_named {
                        match value {
                            Expression::Variable(name) => name.clone(),
                            _ => indexed_name,
                        }
                    } else {
                        indexed_name
                    };
                    let value = self.eval_expression(value, context.clone()).await?;
                    (name, value)
                }
            };
            evaluated_params.insert(name, value);
        }
        Ok(evaluated_params)
    }

    fn eval_literal(lit: &Literal) -> EvalResult<Value> {
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
            Literal::Retry(retry) => {
                // make hashmap
                let mut map = HashMap::new();
                map.insert("type".to_string(), Value::String("retry".to_string()));
                match retry.delay {
                    RetryDelay::Fixed(d) => {
                        map.insert(
                            "delay".to_string(),
                            Value::Duration(Duration::from_millis(d)),
                        );
                    }
                    RetryDelay::Exponential { initial, max } => {
                        map.insert(
                            "initial_delay".to_string(),
                            Value::Duration(Duration::from_millis(initial)),
                        );
                        map.insert("multiplier".to_string(), Value::UInteger(max));
                    }
                }
                Value::Map(map)
            }
        })
    }

    // 変数の評価
    async fn eval_variable(&self, name: &str, context: Arc<ExecutionContext>) -> EvalResult<Value> {
        let access = VariableAccess::Local(name.to_string());
        context.get(access).await.map_err(EvalError::from)
    }

    // 状態アクセスの評価
    async fn eval_state_access(
        &self,
        path: &str,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Value> {
        let access = VariableAccess::State(path.to_string());
        context.get(access).await.map_err(EvalError::from)
    }

    #[tracing::instrument(skip(self, with_block, context))]
    pub async fn eval_think(
        &self,
        args: &[Argument],
        with_block: &Option<ThinkAttributes>,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Value> {
        let provider_name = if let Some(with_block) = with_block {
            with_block.provider.clone()
        } else {
            None
        };

        let provider = self.select_provider(provider_name, context.clone()).await?;

        let policies = self.collect_policies(context.clone(), with_block.as_ref())?;

        let request = self
            .to_provider_request(provider.as_ref(), args, with_block, context, policies)
            .await?;

        let context = ProviderContext {
            config: provider.config.clone(),
            secret: provider.secret.clone(),
        };

        let response = provider
            .provider
            .execute(&context, &request)
            .await
            .map_err(EvalError::from)?;

        Ok(Value::from(response))
    }

    async fn select_provider(
        &self,
        provider_name: Option<String>,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Arc<ProviderInstance>> {
        if let Some(provider_name) = provider_name {
            let entry = context
                .shared
                .providers
                .get(&provider_name)
                .ok_or(EvalError::from(ProviderError::ProviderNotFound(
                    provider_name.clone(),
                )))?;
            Ok(entry.value().clone())
        } else {
            Ok(context.shared.primary.clone())
        }
    }

    async fn build_arg_map_from_args(
        &self,
        args: &[Argument],
        context: Arc<ExecutionContext>,
    ) -> EvalResult<HashMap<String, Value>> {
        let evaled = self.eval_arguments_detect_name(args, context).await?;

        Ok(evaled)
    }

    async fn to_provider_request(
        &self,
        provider: &ProviderInstance,
        args: &[Argument],
        think_attrs: &Option<ThinkAttributes>,
        context: Arc<ExecutionContext>,
        policies: Vec<Policy>,
    ) -> EvalResult<ProviderRequest> {
        let (query, tail_args) = self.query_from_args(args, context.clone()).await?;
        let parameters = self
            .eval_arguments(tail_args.as_slice(), context.clone())
            .await?;
        let input = RequestInput { query, parameters };

        // 実行状態の取得
        let state = ExecutionState {
            session_id: context.session_id().await?,
            policies,
            timestamp: Timestamp::default(),
            agent_name: context.agent_name().to_string(),
            agent_info: context.agent_info().clone(),
            trace_id: context.generate_trace_id(),
        };

        let mut config = provider.config.clone();
        if let Some(attrs) = think_attrs {
            if let Some(model) = attrs.model.clone() {
                config.common_config.model = model;
            }
            if let Some(temperature) = attrs.temperature {
                config.common_config.temperature = temperature as f32;
            }
            if let Some(max_tokens) = attrs.max_tokens {
                config.common_config.max_tokens = max_tokens as usize;
            }
            if let Some(retry) = attrs.retry.clone() {
                config.provider_specific.insert(
                    "retry".to_string(),
                    serde_json::to_value(retry).map_err(EvalError::from)?,
                );
            }
        }

        Ok(ProviderRequest {
            input,
            state,
            config,
        })
    }

    async fn query_from_args(
        &self,
        args: &[Argument],
        context: Arc<ExecutionContext>,
    ) -> EvalResult<(Value, Vec<Argument>)> {
        if args.len() == 1 {
            let content = match args.get(0) {
                Some(Argument::Positional(expr)) => self.eval_expression(expr, context).await?,
                Some(Argument::Named { value, .. }) => self.eval_expression(value, context).await?,
                None => Value::Null,
            };
            return Ok((content, vec![]));
        }

        if let Some(Argument::Named { value, .. }) = args
            .iter()
            .find(|arg| matches!(arg, Argument::Named { name, .. } if name == "query"))
        {
            let filter_not_query = args
                .iter()
                .filter(|arg| !matches!(arg, Argument::Named { name, .. } if name == "query"))
                .cloned()
                .collect::<Vec<Argument>>();
            return Ok((
                self.eval_expression(value, context).await?,
                filter_not_query,
            ));
        }

        if let Some(Argument::Named { value, .. }) = args
            .iter()
            .find(|arg| matches!(arg, Argument::Named { name, .. } if name == "query"))
        {
            let filter_not_query = args
                .iter()
                .filter(|arg| !matches!(arg, Argument::Named { name, .. } if name == "query"))
                .cloned()
                .collect::<Vec<Argument>>();
            return Ok((
                self.eval_expression(value, context).await?,
                filter_not_query,
            ));
        }
        if let Some(Argument::Named { value, .. }) = args
            .iter()
            .find(|arg| matches!(arg, Argument::Named { name, .. } if name == "message"))
        {
            let filter_not_query = args
                .iter()
                .filter(|arg| !matches!(arg, Argument::Named { name, .. } if name == "message"))
                .cloned()
                .collect::<Vec<Argument>>();
            return Ok((
                self.eval_expression(value, context).await?,
                filter_not_query,
            ));
        }
        if let Some(arg) = args.get(0) {
            let tail = args[1..].to_vec();
            if let Argument::Positional(expr) = arg {
                return Ok((self.eval_expression(expr, context).await?, tail));
            }
        }
        Ok((Value::Null, args.to_vec()))
    }

    fn collect_policies(
        &self,
        _context: Arc<ExecutionContext>,
        attributes: Option<&ThinkAttributes>,
    ) -> EvalResult<Vec<Policy>> {
        Ok(attributes
            .map(|attr| {
                let mut sorted = attr
                    .policies
                    .iter()
                    .enumerate()
                    .collect::<Vec<(usize, &Policy)>>()
                    .clone();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                sorted
                    .iter()
                    .map(|(_, p)| (*p).clone())
                    .clone()
                    .collect::<Vec<ast::Policy>>()
            })
            .unwrap_or_default())
    }

    // 関数呼び出しの評価
    async fn eval_function_call(
        &self,
        function: &str,
        arguments: &[Expression],
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Value> {
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
            _ => Err(EvalError::Eval(format!("Unknown function: {}", function))),
        }
    }

    // 二項演算の評価
    async fn eval_binary_op(
        &self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
        context: Arc<ExecutionContext>,
    ) -> EvalResult<Value> {
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

    fn eval_len_function(&self, args: &[Value]) -> EvalResult<Value> {
        if args.len() != 1 {
            return Err(EvalError::Eval(
                "len function requires exactly one argument".to_string(),
            ));
        }

        match &args[0] {
            Value::String(s) => Ok(Value::Integer(s.len() as i64)),
            Value::List(l) => Ok(Value::Integer(l.len() as i64)),
            Value::Map(m) => Ok(Value::Integer(m.len() as i64)),
            _ => Err(EvalError::Eval(format!(
                "len function requires string, list, or map, but got {:?}",
                args[0]
            ))),
        }
    }

    fn eval_sum_function(&self, args: &[Value]) -> EvalResult<Value> {
        if args.len() != 1 {
            return Err(EvalError::Eval(
                "sum function requires exactly one argument".to_string(),
            ));
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
                            return Err(EvalError::Eval(format!(
                                "sum function requires list of numbers, but got {:?}",
                                value
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
            _ => Err(EvalError::Eval(format!(
                "sum function requires list of numbers, but got {:?}",
                args[0]
            ))),
        }
    }

    fn eval_avg_function(&self, args: &[Value]) -> EvalResult<Value> {
        if args.len() != 1 {
            return Err(EvalError::Eval(
                "avg function requires exactly one argument".to_string(),
            ));
        }

        match &args[0] {
            Value::List(list) => {
                if list.is_empty() {
                    return Err(EvalError::Eval(
                        "cannot calculate average of empty list".to_string(),
                    ));
                }

                let sum = self.eval_sum_function(args)?;
                match sum {
                    Value::Integer(i) => Ok(Value::Float(i as f64 / list.len() as f64)),
                    Value::Float(f) => Ok(Value::Float(f / list.len() as f64)),
                    _ => unreachable!(),
                }
            }
            _ => Err(EvalError::Eval(format!(
                "avg function requires list of numbers, but got {:?}",
                args[0]
            ))),
        }
    }

    // 二項演算子の実装

    fn eval_add(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 + r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l + *r as f64)),
            (Value::String(l), Value::String(r)) => Ok(Value::String(l.clone() + r)),
            _ => Err(EvalError::Eval(format!("{:?} + {:?}", left, right))),
        }
    }

    fn eval_subtract(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 - r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l - *r as f64)),
            _ => Err(EvalError::Eval(format!("{:?} - {:?}", left, right))),
        }
    }

    fn eval_multiply(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 * r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l * *r as f64)),
            _ => Err(EvalError::Eval(format!("{:?} * {:?}", left, right))),
        }
    }

    fn eval_divide(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                if *r == 0 {
                    return Err(EvalError::Eval("division by zero".to_string()));
                }
                Ok(Value::Float(*l as f64 / *r as f64))
            }
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l / r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 / r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l / *r as f64)),
            _ => Err(EvalError::Eval(format!("{:?} / {:?}", left, right))),
        }
    }

    fn eval_equal(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        Ok(Value::Boolean(left == right))
    }

    fn eval_not_equal(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        Ok(Value::Boolean(left != right))
    }

    fn eval_less_than(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        self.compare_values(left, right, |ordering| ordering.is_lt())
    }

    fn eval_greater_than(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        self.compare_values(left, right, |ordering| ordering.is_gt())
    }

    fn eval_less_than_equal(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        self.compare_values(left, right, |ordering| ordering.is_le())
    }

    fn eval_greater_than_equal(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        self.compare_values(left, right, |ordering| ordering.is_ge())
    }

    fn eval_and(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        match (left, right) {
            (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l && *r)),
            _ => Err(EvalError::Eval(format!("{:?} && {:?}", left, right))),
        }
    }

    fn eval_or(&self, left: &Value, right: &Value) -> EvalResult<Value> {
        match (left, right) {
            (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l || *r)),
            _ => Err(EvalError::Eval(format!("{:?} || {:?}", left, right))),
        }
    }

    // ヘルパーメソッド

    fn compare_values<F>(&self, left: &Value, right: &Value, compare: F) -> EvalResult<Value>
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
            _ => Err(EvalError::Eval(format!("{:?} <=> {:?}", left, right))),
        }
    }
}

#[cfg(test)]
mod tests {
    use dashmap::DashMap;

    use crate::{
        config::ContextConfig,
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
            ContextConfig::default(),
            Arc::new(ProviderInstance::default()),
            Arc::new(DashMap::new()),
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
