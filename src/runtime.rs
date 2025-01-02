use async_trait::async_trait;
use dashmap::DashMap;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::Arc;

use crate::event_bus::{Event, Value};
use crate::{
    ExecutionError, Expression, HandlerError, Literal, MicroAgentDef, RuntimeError, RuntimeResult,
};

// 並行処理のためのTrait
#[async_trait]
pub trait RuntimeAgent: Send + Sync {
    async fn run(&self) -> RuntimeResult<()>;
    fn name(&self) -> String;
}

// MicroAgentの実行時表現
pub struct RuntimeAgentData {
    name: String,
    pub state: Arc<DashMap<String, Value>>,
    observe_handlers: DashMap<String, ObserveHandler>,
    answer_handlers: DashMap<String, AnswerHandler>,
    react_handlers: DashMap<String, ReactHandler>,
}

#[async_trait]
impl RuntimeAgent for RuntimeAgentData {
    async fn run(&self) -> RuntimeResult<()> {
        Ok(())
    }
    fn name(&self) -> String {
        self.name.clone()
    }
}

// ハンドラの型
type ObserveHandler =
    Box<dyn Fn(&Event) -> BoxFuture<'static, Option<HashMap<String, Value>>> + Send + Sync>;
type AnswerHandler =
    Box<dyn Fn(&Request) -> BoxFuture<'static, RuntimeResult<Value>> + Send + Sync>;
type ReactHandler = Box<dyn Fn(&Event) -> BoxFuture<'static, Option<Vec<Event>>> + Send + Sync>;

#[derive(Clone, Debug)]
pub struct Request {
    pub request_type: String,
    pub parameters: HashMap<String, Value>,
}

impl RuntimeAgentData {
    pub fn new(agent_def: &MicroAgentDef) -> RuntimeResult<Self> {
        let state = Arc::new(DashMap::new());

        // 初期状態の設定
        if let Some(state_def) = &agent_def.state {
            for (name, var_def) in &state_def.variables {
                if let Some(initial) = &var_def.initial_value {
                    state.insert(name.clone(), eval_expression(initial)?);
                }
            }
        }

        Ok(Self {
            name: agent_def.name.clone(),
            state,
            observe_handlers: DashMap::new(),
            answer_handlers: DashMap::new(),
            react_handlers: DashMap::new(),
        })
    }

    // observe ハンドラの登録
    pub fn register_observe(&mut self, event_type: String, handler: ObserveHandler) {
        self.observe_handlers.insert(event_type, handler);
    }

    // answer ハンドラの登録
    pub fn register_answer(&mut self, request_type: String, handler: AnswerHandler) {
        self.answer_handlers.insert(request_type, handler);
    }

    // react ハンドラの登録
    pub fn register_react(&mut self, event_type: String, handler: ReactHandler) {
        self.react_handlers.insert(event_type, handler);
    }

    // イベントの処理
    pub async fn handle_event(&self, event: &Event) -> RuntimeResult<Vec<Event>> {
        let mut new_events = Vec::new();

        // observe ハンドラの実行
        if let Some(handler) = self.observe_handlers.get(&event.event_type.to_string()) {
            if let Some(updates) = handler.value()(event).await {
                for (key, value) in updates {
                    self.state.insert(key, value);
                }
            }
        }

        // react ハンドラの実行
        if let Some(handler) = self.react_handlers.get(&event.event_type.to_string()) {
            if let Some(events) = handler(event).await {
                new_events.extend(events);
            }
        }

        Ok(new_events)
    }

    // リクエストの処理
    pub async fn handle_request(&self, request: &Request) -> RuntimeResult<Value> {
        if let Some(handler) = self.answer_handlers.get(&request.request_type) {
            handler.value()(request).await
        } else {
            Err(RuntimeError::Handler(HandlerError::NotFound {
                handler_type: "answer".to_string(),
                name: request.request_type.clone(),
            }))
        }
    }
}

// 式の評価
fn eval_expression(expr: &Expression) -> RuntimeResult<Value> {
    match expr {
        Expression::Literal(lit) => Ok(match lit {
            Literal::Integer(i) => Value::Integer(*i),
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::Duration(d) => Value::Float(d.as_secs_f64()),
            Literal::Null => Value::Null,
        }),
        // 他の式の評価は必要に応じて実装
        _ => Err(RuntimeError::Execution(ExecutionError::EvaluationFailed(
            "Unsupported expression".to_string(),
        ))),
    }
}
