use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing::debug;
use uuid::Uuid;

use super::expression::Value;
use super::generator::{PromptGenerator, StandardPromptGenerator};
use crate::Policy;
use crate::config::ContextConfig;
use crate::event::event_bus::{self, Event, EventBus, EventError, ToEventType};
use crate::event_registry::EventType;
use crate::provider::provider_registry::ProviderInstance;
use crate::provider::types::ProviderError;
use crate::request_manager::{RequestError, RequestManager};
use crate::runtime::RuntimeError;

pub struct SafeRwLock<T> {
    inner: RwLock<T>,
    last_access: AtomicU64,
    owner: AtomicU64,
    lock_counter: AtomicU64, // ロックの識別に使用
}

impl<T> SafeRwLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: RwLock::new(value),
            last_access: AtomicU64::new(0),
            owner: AtomicU64::new(0),
            lock_counter: AtomicU64::new(0),
        }
    }

    pub async fn read_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<RwLockReadGuard<T>, LockError> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 読み取りロックは複数同時に取得可能なので、
        // デッドロック検出はタイムアウトのみで十分
        match tokio::time::timeout(timeout, self.inner.read()).await {
            Ok(guard) => {
                self.last_access.store(current_time, Ordering::SeqCst);
                Ok(guard)
            }
            Err(_) => Err(LockError::Timeout),
        }
    }

    pub async fn write_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<RwLockWriteGuard<T>, LockError> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 前回のロックが長時間解放されていない場合(初回アクセスはチェックしない)
        let last_access = self.last_access.load(Ordering::SeqCst);
        if last_access > 0 && current_time - last_access > timeout.as_secs() {
            return Err(LockError::Deadlock);
        }

        match tokio::time::timeout(timeout, self.inner.write()).await {
            Ok(guard) => {
                self.last_access.store(current_time, Ordering::SeqCst);
                // スレッドIDの代わりにカウンターを使用
                let lock_id = self.lock_counter.fetch_add(1, Ordering::SeqCst);
                self.owner.store(lock_id, Ordering::SeqCst);
                Ok(guard)
            }
            Err(_) => Err(LockError::Timeout),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("Lock timeout")]
    Timeout,
    #[error("Deadlock")]
    Deadlock,
    // 他のエラーケース
}

#[derive(Debug, Clone)]
pub enum VariableAccess {
    State(String), // self.xxx形式でのアクセス
    Local(String), // 通常のローカル変数アクセス
}

/// 実行コンテキスト
#[derive(Clone)]
pub struct ExecutionContext {
    pub shared: SharedContext,
    current_scope: DashMap<String, Arc<SafeRwLock<Value>>>,
    access_mode: StateAccessMode,
    // config Read/Write timeout
    pub timeout: Duration,
}

impl ExecutionContext {
    pub fn generate_trace_id(&self) -> String {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("trace-{}-{}", self.agent_name(), current_time)
    }
}

/// Context生成時に追加されて、共有される。更新されない。
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AgentInfo {
    pub agent_name: String,
    pub agent_type: AgentType,
    #[serde(skip)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub enum AgentType {
    World,
    ScaleManager,
    Monitor,
    User,
    Custom(String),
    #[default]
    Unknown,
}

pub const AGENT_TYPE_CUSTOM_ALL: &str = "all";

impl AgentType {
    pub fn same(&self, target: AgentType) -> bool {
        match self {
            AgentType::World
            | AgentType::ScaleManager
            | AgentType::Monitor
            | AgentType::User
            | AgentType::Unknown => self == &target,
            AgentType::Custom(..) => matches!(target, AgentType::Custom(..)),
        }
    }
    pub fn is_all(&self) -> bool {
        if let AgentType::Custom(name) = self {
            name.to_lowercase() == AGENT_TYPE_CUSTOM_ALL
        } else {
            false
        }
    }
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::World => write!(f, "World"),
            Self::ScaleManager => write!(f, "ScaleManager"),
            Self::Monitor => write!(f, "Monitor"),
            Self::User => write!(f, "User"),
            Self::Custom(name) => write!(f, "{}", name),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

type ParentScopes = Vec<Arc<DashMap<String, Arc<SafeRwLock<Value>>>>>;

/// 共有可能なコンテキストの状態
#[derive(Clone)]
pub struct SharedContext {
    // 状態をRwLockで保護
    pub state: Arc<DashMap<String, Arc<SafeRwLock<Value>>>>,
    event_bus: Arc<EventBus>,
    parent_scopes: Arc<ParentScopes>,
    request_manager: Arc<RequestManager>,
    agent_info: AgentInfo, // システム提供の情報を追加
    // LLM 関連
    pub primary: Arc<ProviderInstance>,
    pub providers: Arc<DashMap<String, Arc<ProviderInstance>>>,
    pub prompt_generator: Arc<dyn PromptGenerator>,
    pub policies: Vec<Policy>,
}

#[derive(Debug, Copy, Clone)]
pub enum StateAccessMode {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("Lock timeout: {0}")]
    Lock(#[from] LockError),
    #[error("Invalid state value for {key}: {message}")]
    InvalidValue { key: String, message: String },
    #[error("State error: {0}")]
    EventError(EventError),
    // request error
    #[error("Requst error: {0}")]
    Request(#[from] RequestError),
    // アクセス制御のエラーを追加
    #[error("Access error: {0}")]
    AccessError(String),
    #[error("Lock timeout: {0}")]
    LockTimeout(String),
    #[error("Deadlock: {0}")]
    Deadlock(String),
    #[error("Variable not found: {0}")]
    VariableNotFound(String),
    #[error("Read-only violation")]
    ReadOnlyViolation,
    #[error("No parent scope")]
    NoParentScope,
    #[error("Event send failed: {0}")]
    EventSendFailed(String),
    #[error("State not found: {0}")]
    StateNotFound(String),
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    #[error("Failure: {0}")]
    Failure(String),
}

impl ToEventType for ContextError {
    fn to_event_type(&self) -> String {
        match self {
            ContextError::Lock(_) => "LockError".to_string(),
            ContextError::InvalidValue { .. } => "InvalidValue".to_string(),
            ContextError::EventError(_) => "EventError".to_string(),
            ContextError::Request(_) => "RequestError".to_string(),
            ContextError::AccessError(_) => "AccessError".to_string(),
            ContextError::LockTimeout(_) => "LockTimeout".to_string(),
            ContextError::Deadlock(_) => "Deadlock".to_string(),
            ContextError::VariableNotFound(_) => "VariableNotFound".to_string(),
            ContextError::ReadOnlyViolation => "ReadOnlyViolation".to_string(),
            ContextError::NoParentScope => "NoParentScope".to_string(),
            ContextError::EventSendFailed(_) => "EventSendFailed".to_string(),
            ContextError::StateNotFound(_) => "StateNotFound".to_string(),
            ContextError::Provider(_) => "ProviderError".to_string(),
            ContextError::Failure(_) => "Failure".to_string(),
        }
    }
}

impl ExecutionContext {
    pub fn new(
        event_bus: Arc<EventBus>,
        agent_info: AgentInfo,
        access_mode: StateAccessMode,
        config: ContextConfig,
        primary: Arc<ProviderInstance>,
        providers: Arc<DashMap<String, Arc<ProviderInstance>>>,
        policies: Vec<Policy>,
    ) -> Self {
        let (mut event_rx, _) = event_bus.subscribe();
        let request_manager = Arc::new(RequestManager::new(
            event_bus.clone(),
            config.request_timeout,
        ));
        let new_self = Self {
            shared: SharedContext {
                state: Arc::new(DashMap::new()),
                event_bus,
                parent_scopes: Arc::new(Vec::new()),
                request_manager,
                agent_info,
                primary,
                providers,
                // only 1 variant, when appended type, inject here.
                prompt_generator: Arc::new(StandardPromptGenerator),
                policies,
            },
            current_scope: DashMap::new(),
            access_mode,
            timeout: config.access_timeout,
        };
        let self_ref = new_self.clone();
        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                let _ = self_ref.shared.request_manager.handle_event(&event);
            }
        });
        new_self
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn fork(&self, access_mode: Option<StateAccessMode>) -> Self {
        // 現在のスコープの内容を新しいスコープにコピー
        let new_scope = {
            let new_scope = DashMap::new();
            for entry in self.current_scope.iter() {
                new_scope.insert(entry.key().clone(), entry.value().clone());
            }
            new_scope
        };

        // 親スコープチェーンを更新
        let mut new_parents = Vec::new();

        // 既存の親スコープをコピー
        for scope in self.shared.parent_scopes.iter() {
            new_parents.push(scope.clone());
        }

        // 新しい共有コンテキストを作成
        let mut new_shared = self.shared.clone();
        new_shared.parent_scopes = Arc::new(new_parents);

        Self {
            shared: new_shared,
            current_scope: new_scope,
            access_mode: access_mode.unwrap_or(self.access_mode),
            timeout: self.timeout,
        }
    }

    /// 変数アクセス（スコープチェーンを遡って検索）
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn get_variable(&self, name: &str) -> Result<Value, ContextError> {
        // 現在のスコープをまず確認
        if let Some(value) = self.current_scope.get(name) {
            return match value.read_with_timeout(self.timeout).await {
                Ok(guard) => Ok(guard.clone()),
                Err(LockError::Timeout) => Err(ContextError::LockTimeout(name.to_string())),
                Err(LockError::Deadlock) => Err(ContextError::Deadlock(name.to_string())),
            };
        }

        // 親スコープを順に確認
        for scope in self.shared.parent_scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return match value.read_with_timeout(self.timeout).await {
                    Ok(guard) => Ok(guard.clone()),
                    Err(LockError::Timeout) => Err(ContextError::LockTimeout(name.to_string())),
                    Err(LockError::Deadlock) => Err(ContextError::Deadlock(name.to_string())),
                };
            }
        }

        // 最後にグローバル状態を確認
        if let Some(value) = self.shared.state.get(name) {
            return match value.read_with_timeout(self.timeout).await {
                Ok(guard) => Ok(guard.clone()),
                Err(LockError::Timeout) => Err(ContextError::LockTimeout(name.to_string())),
                Err(LockError::Deadlock) => Err(ContextError::Deadlock(name.to_string())),
            };
        }

        Err(ContextError::VariableNotFound(name.to_string()))
    }

    /// 変数の更新（現在のスコープのみ）
    #[tracing::instrument(skip(self, value), level = "debug")]
    pub async fn set_variable(&self, name: &str, value: Value) -> Result<(), ContextError> {
        match self.access_mode {
            StateAccessMode::ReadOnly if self.is_state(name) => {
                Err(ContextError::ReadOnlyViolation)
            }
            _ => {
                let safe_value = Arc::new(SafeRwLock::new(value));
                self.current_scope.insert(name.to_string(), safe_value);
                Ok(())
            }
        }
    }

    /// 状態変数の読み取り
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn get_state(&self, name: &str) -> Result<Value, ContextError> {
        if let Some(value) = self.shared.state.get(name) {
            match value.read_with_timeout(self.timeout).await {
                Ok(guard) => Ok(guard.clone()),
                Err(LockError::Timeout) => Err(ContextError::LockTimeout(name.to_string())),
                Err(LockError::Deadlock) => Err(ContextError::Deadlock(name.to_string())),
            }
        } else {
            Err(ContextError::VariableNotFound(name.to_string()))
        }
    }

    /// 状態変数の更新
    pub fn set_state(&self, name: &str, value: Value) -> Result<(), ContextError> {
        match self.access_mode {
            StateAccessMode::ReadOnly => Err(ContextError::ReadOnlyViolation),
            StateAccessMode::ReadWrite => {
                let value_ref = value.clone();
                let safe_value = Arc::new(SafeRwLock::new(value));
                self.shared.state.insert(name.to_string(), safe_value);
                self.notify_state_update(name, &value_ref)
            }
        }
    }

    /// 状態変数の更新（クロージャを使用）
    #[tracing::instrument(skip(self, f), level = "debug")]
    pub async fn update_state<F>(&self, name: &str, f: F) -> Result<(), ContextError>
    where
        F: FnOnce(&mut Value) -> Result<(), ContextError> + Send + Sync,
    {
        match self.access_mode {
            StateAccessMode::ReadOnly => Err(ContextError::ReadOnlyViolation),
            StateAccessMode::ReadWrite => {
                if let Some(value) = self.shared.state.get(name) {
                    match value.write_with_timeout(self.timeout).await {
                        Ok(mut guard) => {
                            if f(&mut guard).is_ok() {
                                drop(guard);
                                self.notify_state_update(name, &Value::Unit)
                            } else {
                                Err(ContextError::InvalidValue {
                                    key: name.to_string(),
                                    message: "Invalid value".to_string(),
                                })
                            }
                        }
                        Err(LockError::Timeout) => Err(ContextError::LockTimeout(name.to_string())),
                        Err(LockError::Deadlock) => Err(ContextError::Deadlock(name.to_string())),
                    }
                } else {
                    Err(ContextError::VariableNotFound(name.to_string()))
                }
            }
        }
    }

    /// 状態変数の確認
    pub fn is_state(&self, name: &str) -> bool {
        self.shared.state.contains_key(name)
    }

    /// 状態変数の削除
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn remove_state(&self, name: &str) -> Result<(), ContextError> {
        match self.access_mode {
            StateAccessMode::ReadOnly => Err(ContextError::ReadOnlyViolation),
            StateAccessMode::ReadWrite => {
                if self.shared.state.remove(name).is_some() {
                    Ok(())
                } else {
                    Err(ContextError::VariableNotFound(name.to_string()))
                }
            }
        }
    }

    /// 状態変数の一覧を取得
    pub fn list_state_variables(&self) -> Vec<String> {
        self.shared
            .state
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub fn agent_info(&self) -> AgentInfo {
        self.shared.agent_info.clone()
    }

    pub fn agent_name(&self) -> String {
        self.shared.agent_info.agent_name.clone()
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn session_id(&self) -> Result<String, ContextError> {
        let session_id = if let Some(session_id) = self.shared.state.get("session_id") {
            session_id
                .value()
                .read_with_timeout(Duration::from_secs(5))
                .await
                .map(|v| match v.clone() {
                    Value::String(s) => s,
                    _ => "".to_string(),
                })
                .map_err(ContextError::from)?
        } else {
            let session_id = Uuid::new_v4().to_string();
            let _ = self.set_state("session_id", Value::String(session_id.clone()));
            session_id
        };
        Ok(session_id)
    }

    // 統一された変数アクセスインターフェース
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn get(&self, access: VariableAccess) -> Result<Value, ContextError> {
        match access {
            VariableAccess::State(key) => self.get_state(&key).await,
            VariableAccess::Local(name) => self.get_variable(&name).await,
        }
    }

    #[tracing::instrument(skip(self, value), level = "debug")]
    pub async fn set(&self, access: VariableAccess, value: Value) -> Result<(), ContextError> {
        match access {
            VariableAccess::State(key) => self.set_state(&key, value),
            VariableAccess::Local(name) => self.set_variable(&name, value).await,
        }
    }
    /// 新しいスコープフレームの作成
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn push_scope(&mut self) -> Result<(), ContextError> {
        let mut new_parents = (*self.shared.parent_scopes).clone();
        new_parents.push(Arc::new(self.current_scope.clone()));

        self.shared.parent_scopes = Arc::new(new_parents);
        self.current_scope = DashMap::new();

        Ok(())
    }

    /// スコープフレームの破棄
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn pop_scope(&mut self) -> Result<(), ContextError> {
        let mut new_parents = (*self.shared.parent_scopes).clone();

        if let Some(last_scope) = new_parents.pop() {
            self.current_scope = (*last_scope).clone();
            self.shared.parent_scopes = Arc::new(new_parents);
            Ok(())
        } else {
            Err(ContextError::NoParentScope)
        }
    }

    // イベント関連のメソッド
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn emit_event(&self, event: Event) -> Result<(), ContextError> {
        self.shared
            .event_bus
            .publish(event)
            .await
            .map_err(|e| ContextError::EventSendFailed(e.to_string()))
    }

    // onFail などのエラーイベントの発行
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn emit_failure(&self, error: ContextError) -> Result<(), ContextError> {
        let error_event = Event {
            event_type: EventType::Failure {
                error_type: error.to_event_type(),
            },
            parameters: {
                let mut parameters = HashMap::new();
                parameters.insert(
                    "agent_name".to_string(),
                    event_bus::Value::from(self.agent_name()),
                );
                parameters.insert(
                    "error".to_string(),
                    event_bus::Value::from(error.to_string()),
                );
                parameters
            },
        };
        self.shared
            .event_bus
            .publish(error_event)
            .await
            .map_err(|e| {
                ContextError::EventError(EventError::SendFailed {
                    message: e.to_string(),
                })
            })
    }

    pub async fn send_response(
        &self,
        request: EventType,
        result: Result<Value, RuntimeError>,
    ) -> Result<(), ContextError> {
        if let EventType::Request {
            request_type,
            requester,
            responder,
            request_id,
        } = request
        {
            let event = match result {
                Ok(value) => Event {
                    event_type: EventType::ResponseSuccess {
                        request_id,
                        request_type,
                        requester,
                        responder,
                    },
                    parameters: vec![("response".to_string(), event_bus::Value::from(value))]
                        .into_iter()
                        .collect::<HashMap<String, event_bus::Value>>(),
                },
                Err(error) => Event {
                    event_type: EventType::ResponseFailure {
                        request_id,
                        request_type,
                        requester,
                        responder,
                    },
                    parameters: vec![(
                        "error".to_string(),
                        event_bus::Value::from(error.to_string()),
                    )]
                    .into_iter()
                    .collect::<HashMap<String, event_bus::Value>>(),
                },
            };
            self.emit_event(event).await?
        } else {
            return Err(ContextError::EventError(EventError::UnsupportedType {
                event_type: request.to_string(),
            }));
        }
        Ok(())
    }

    pub fn notify_state_update(&self, key: &str, value: &Value) -> Result<(), ContextError> {
        let mut parameters = HashMap::new();
        parameters.insert("value".to_string(), event_bus::Value::from(value.clone()));
        self.shared
            .event_bus
            .sync_publish(Event {
                event_type: EventType::StateUpdated {
                    agent_name: self.shared.agent_info.agent_name.clone(),
                    state_name: key.to_string(),
                },
                parameters,
            })
            .map_err(|e| ContextError::EventSendFailed(e.to_string()))?;
        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn send_request(&self, request: Event) -> Result<Event, ContextError> {
        debug!("Send Request, I'm {}", self.agent_name());
        self.shared
            .request_manager
            .request(&request)
            .await
            .map_err(ContextError::from)
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn cancel_pending_requests(&self) -> Result<(), ContextError> {
        self.shared
            .request_manager
            .cancel_waiting_requests("Agent shutdown")
            .await
            .map_err(ContextError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub async fn setup_readwrite_test_context() -> ExecutionContext {
        let event_bus = Arc::new(EventBus::new(10));
        ExecutionContext::new(
            event_bus,
            AgentInfo::default(),
            StateAccessMode::ReadWrite,
            ContextConfig::default(),
            Arc::new(ProviderInstance::default()),
            Arc::new(DashMap::new()),
            vec![],
        )
    }

    async fn setup_readonly_context() -> ExecutionContext {
        let event_bus = Arc::new(EventBus::new(10));
        ExecutionContext::new(
            event_bus,
            AgentInfo::default(),
            StateAccessMode::ReadOnly,
            ContextConfig::default(),
            Arc::new(ProviderInstance::default()),
            Arc::new(DashMap::new()),
            vec![],
        )
    }

    #[tokio::test]
    async fn test_state_variable_access_readwrite() {
        let context = setup_readwrite_test_context().await;

        // 状態変数の設定
        let result = context.set_state("count", Value::Integer(42));
        assert!(result.is_ok());

        // 状態変数の読み取り
        let value = context.get_state("count").await.unwrap();
        assert_eq!(value, Value::Integer(42));

        // 存在しない状態変数へのアクセス
        let error = context.get_state("nonexistent").await;
        assert!(matches!(error, Err(ContextError::VariableNotFound(_))));
    }

    #[tokio::test]
    async fn test_state_variable_access_readonly() {
        let context = setup_readwrite_test_context().await;
        let result = context.set_state("count", Value::Integer(42));
        assert!(result.is_ok());

        let context = context.fork(Some(StateAccessMode::ReadOnly)).await;

        // 状態変数の設定
        let result = context.set_state("count", Value::Integer(42));
        assert!(result.is_err());

        // 状態変数の読み取り
        let value = context.get_state("count").await.unwrap();
        assert_eq!(value, Value::Integer(42));

        // 存在しない状態変数へのアクセス
        let error = context.get_state("nonexistent").await;
        assert!(matches!(error, Err(ContextError::VariableNotFound(_))));
    }

    #[tokio::test]
    async fn test_local_variable_access() {
        let context = setup_readwrite_test_context().await;

        // ローカル変数の設定
        let result = context.set_variable("x", Value::Integer(1)).await;
        assert!(result.is_ok());

        // ローカル変数の読み取り
        let value = context.get_variable("x").await.unwrap();
        assert_eq!(value, Value::Integer(1));

        // 存在しないローカル変数へのアクセス
        let error = context.get_variable("nonexistent").await;
        assert!(matches!(error, Err(ContextError::VariableNotFound(_))));

        // state readonly でもローカル変数の書き込みは許可される
        let context = setup_readonly_context().await;
        let result = context.set_variable("x", Value::Integer(1)).await;
        assert!(result.is_ok());

        // ローカル変数の読み取り
        let value = context.get_variable("x").await.unwrap();
        assert_eq!(value, Value::Integer(1));

        // 存在しないローカル変数へのアクセス
        let error = context.get_variable("nonexistent").await;
        assert!(matches!(error, Err(ContextError::VariableNotFound(_))));
    }

    #[tokio::test]
    async fn test_scope_management() {
        let mut context = setup_readwrite_test_context().await;

        // グローバルスコープでの変数設定
        context.set_variable("x", Value::Integer(1)).await.unwrap();

        // 新しいスコープをプッシュ
        context.push_scope().await.unwrap();

        // 新しいスコープでの同名変数の設定
        context.set_variable("x", Value::Integer(2)).await.unwrap();

        // 現在のスコープでの値を確認
        let value = context.get_variable("x").await.unwrap();
        assert_eq!(value, Value::Integer(2));

        // スコープをポップ
        context.pop_scope().await.unwrap();

        // 元のスコープの値を確認
        let value = context.get_variable("x").await.unwrap();
        assert_eq!(value, Value::Integer(1));
    }

    #[tokio::test]
    async fn test_concurrent_variable_access() {
        let context = setup_readwrite_test_context().await;
        let context = Arc::new(context);

        // 複数のタスクから同時にアクセス
        let mut handles = vec![];
        for i in 0..10 {
            let context = context.clone();
            handles.push(tokio::spawn(async move {
                context
                    .set_state(&format!("key_{}", i), Value::Integer(i))
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(10)).await;
                let value = context.get_state(&format!("key_{}", i)).await.unwrap();
                assert_eq!(value, Value::Integer(i));
            }));
        }

        // すべてのタスクの完了を待つ
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_fork() {
        let context = setup_readwrite_test_context().await;

        // 親コンテキストで変数を設定
        context.set_state("shared", Value::Integer(1)).unwrap();
        context
            .set_variable("local", Value::Integer(2))
            .await
            .unwrap();

        // コンテキストをフォーク
        let forked = context.fork(None).await;

        // フォークされたコンテキストで変数を確認
        assert_eq!(forked.get_state("shared").await.unwrap(), Value::Integer(1));
        assert_eq!(
            forked.get_variable("local").await.unwrap(),
            Value::Integer(2)
        );

        // フォークされたコンテキストで新しい変数を設定
        forked
            .set_variable("fork_local", Value::Integer(3))
            .await
            .unwrap();

        // 親コンテキストには影響しないことを確認
        assert!(context.get_variable("fork_local").await.is_err());
    }

    #[tokio::test]
    async fn test_safe_rwlock_read() {
        let lock = Arc::new(SafeRwLock::new(0));

        // 正常なロック取得
        {
            let guard = lock
                .read_with_timeout(Duration::from_secs(1))
                .await
                .unwrap();
            assert_eq!(*guard, 0);
        }

        // タイムアウトのテスト
        let handle = tokio::spawn({
            let lock = lock.clone();
            async move {
                let _guard = lock
                    .read_with_timeout(Duration::from_secs(10))
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });

        // 少し待ってから別のロック取得を試みる
        tokio::time::sleep(Duration::from_millis(100)).await;
        let result = lock.write_with_timeout(Duration::from_secs(1)).await;
        assert!(matches!(result, Err(LockError::Timeout)));

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_safe_rwlock_write() {
        let lock = Arc::new(SafeRwLock::new(0));

        // 正常なロック取得
        {
            let mut guard = lock
                .write_with_timeout(Duration::from_secs(1))
                .await
                .unwrap();
            *guard = 42;
        }

        // タイムアウトのテスト
        let handle = tokio::spawn({
            let lock = lock.clone();
            async move {
                let _guard = lock
                    .write_with_timeout(Duration::from_secs(10))
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });

        // 少し待ってから別のロック取得を試みる
        tokio::time::sleep(Duration::from_millis(100)).await;
        let result = lock.write_with_timeout(Duration::from_secs(1)).await;
        assert!(matches!(result, Err(LockError::Timeout)));

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_deadlock_detection_read() {
        let lock = Arc::new(SafeRwLock::new(0));

        // 長時間のロック保持
        let handle = tokio::spawn({
            let lock = lock.clone();
            async move {
                let _guard = lock
                    .read_with_timeout(Duration::from_secs(10))
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_secs(6)).await;
            }
        });

        // デッドロック検出のテスト
        tokio::time::sleep(Duration::from_secs(7)).await;
        let result = lock.write_with_timeout(Duration::from_secs(1)).await;
        assert!(matches!(result, Err(LockError::Deadlock)));

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_deadlock_detection_write() {
        let lock = Arc::new(SafeRwLock::new(0));

        // 長時間のロック保持
        let handle = tokio::spawn({
            let lock = lock.clone();
            async move {
                let _guard = lock
                    .write_with_timeout(Duration::from_secs(10))
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_secs(6)).await;
            }
        });

        // デッドロック検出のテスト
        tokio::time::sleep(Duration::from_secs(7)).await;
        let result = lock.write_with_timeout(Duration::from_secs(1)).await;
        assert!(matches!(result, Err(LockError::Deadlock)));

        handle.await.unwrap();
    }
}
