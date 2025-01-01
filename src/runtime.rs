use dashmap::DashMap;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::event_resitory::EventType;
use crate::{
    EventError, ExecutionError, Expression, HandlerError, Literal, MicroAgentDef, RuntimeError,
    RuntimeResult,
};

// MicroAgentの実行時表現
pub struct RuntimeAgent {
    name: String,
    pub state: Arc<DashMap<String, Value>>,
    observe_handlers: DashMap<String, ObserveHandler>,
    answer_handlers: DashMap<String, AnswerHandler>,
    react_handlers: DashMap<String, ReactHandler>,
}

pub struct Event {
    pub event_type: EventType,
    pub parameters: HashMap<String, Value>,
}

// 値の型
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
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

impl RuntimeAgent {
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

// シンプルなランタイム
pub struct Runtime {
    agents: Arc<DashMap<String, Arc<RuntimeAgent>>>,
    event_sender: mpsc::Sender<Event>,
    event_receiver: Option<mpsc::Receiver<Event>>,
}

impl Clone for Runtime {
    fn clone(&self) -> Self {
        Self {
            agents: self.agents.clone(),
            event_sender: self.event_sender.clone(),
            event_receiver: None, // クローンではreceiverは共有しない
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100); // バッファサイズは適宜調整

        Self {
            agents: Arc::new(DashMap::new()),
            event_sender: tx,
            event_receiver: Some(rx),
        }
    }

    // エージェントの登録
    pub fn register_agent(&self, agent: RuntimeAgent) {
        self.agents.insert(agent.name.clone(), Arc::new(agent));
    }

    // エージェントの取得
    pub fn get_agent(&self, name: &str) -> Option<Arc<RuntimeAgent>> {
        self.agents.get(name).map(|agent| agent.clone())
    }

    // イベントの送信
    pub async fn send_event(&self, event: Event) -> Result<(), EventError> {
        self.event_sender
            .send(event)
            .await
            .map_err(|e| EventError::SendFailed {
                message: e.to_string(),
            })
    }

    // メインループの実行
    pub async fn run(&mut self) -> RuntimeResult<()> {
        while let Some(event) = self
            .event_receiver
            .as_mut()
            .ok_or_else(|| ExecutionError::ReceiverNotFound {
                receiver: "event_receiver".to_string(),
            })?
            .recv()
            .await
        {
            let agents: Vec<_> = self
                .agents
                .iter()
                .map(|agent| agent.value().clone())
                .collect();

            let futures: Vec<_> = agents
                .iter()
                .map(|agent| agent.handle_event(&event))
                .collect();

            // 全エージェントのイベント処理を並行実行
            let results = futures::future::join_all(futures).await;

            // 新しいイベントの処理
            for result in results {
                match result {
                    Ok(new_events) => {
                        for new_event in new_events {
                            self.send_event(new_event).await?;
                        }
                    }
                    Err(e) => log::error!("Error handling event: {:?}", e),
                }
            }
        }
        Ok(())
    }
}

// テスト用のヘルパー関数
#[cfg(test)]
mod tests {
    use crate::{event_resitory::EventType, MicroAgentDef, StateDef};

    use super::*;
    use tokio::test;

    #[test]
    async fn test_simple_agent() {
        let runtime = Runtime::new();

        // テスト用のエージェントを作成
        let mut agent = RuntimeAgent::new(&MicroAgentDef {
            name: "test".to_string(),
            state: Some(StateDef {
                variables: HashMap::new(),
            }),
            ..Default::default()
        })
        .unwrap();

        // observe ハンドラの登録
        agent.register_observe(
            "test".to_string(),
            Box::new(|_event| {
                Box::pin(async move {
                    let mut updates = HashMap::new();
                    updates.insert("count".to_string(), Value::Integer(1));
                    Some(updates)
                })
            }),
        );

        runtime.register_agent(agent);

        // テストイベントの送信
        runtime
            .send_event(Event {
                event_type: EventType::Custom("test".to_string()),
                parameters: HashMap::new(),
            })
            .await
            .unwrap();

        // 非同期処理のテストなので、少し待機
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
