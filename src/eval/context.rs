use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;

use crate::event_bus::{Event, EventBus};
use crate::{EventError, StateError};

use super::expression::Value;

#[derive(Debug, Clone)]
pub enum VariableAccess {
    State(String), // self.xxx形式でのアクセス
    Local(String), // 通常のローカル変数アクセス
}

pub struct ExecutionContext {
    state_access: StateAccess,
    scope_stack: ScopeStack,
    event_bus: Arc<EventBus>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: merge application errors
#[derive(Debug, strum::Display)]
pub enum ContextError {
    StateError(StateError),
    ScopeError(ScopeError),
    EventError(EventError),
    // アクセス制御のエラーを追加
    AccessError(String),
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            state_access: StateAccess::ReadOnly(Arc::new(DashMap::new())),
            scope_stack: ScopeStack::new(),
            event_bus: Arc::new(EventBus::new(16)),
        }
    }

    // コンストラクタの修正
    pub fn new_answer_context(
        state: Arc<DashMap<String, Value>>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            state_access: StateAccess::ReadOnly(state),
            scope_stack: ScopeStack::new(),
            event_bus,
        }
    }

    pub fn new_general_context(
        state: Arc<DashMap<String, Value>>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            state_access: StateAccess::ReadWrite(state),
            scope_stack: ScopeStack::new(),
            event_bus,
        }
    }

    // 統一された変数アクセスインターフェース
    pub async fn access_variable(&self, access: VariableAccess) -> Result<Value, ContextError> {
        match access {
            VariableAccess::State(key) => self
                .state_access
                .read(&key)
                .await
                .map_err(ContextError::StateError),
            VariableAccess::Local(name) => self
                .scope_stack
                .lookup(&name)
                .map_err(ContextError::ScopeError),
        }
    }
    pub async fn update_variable(
        &mut self,
        access: VariableAccess,
        value: Value,
    ) -> Result<(), ContextError> {
        match access {
            VariableAccess::State(key) => self
                .state_access
                .write(key, value)
                .await
                .map_err(ContextError::StateError),
            VariableAccess::Local(name) => {
                self.scope_stack.insert(name, value);
                Ok(())
            }
        }
    }

    // スコープ管理
    pub fn push_scope(&mut self) {
        self.scope_stack.push_scope();
    }

    pub fn pop_scope(&mut self) -> Option<Scope> {
        self.scope_stack.pop_scope()
    }

    // イベント関連
    pub async fn emit_event(&self, event: Event) -> Result<(), ContextError> {
        self.event_bus.publish(event).await.map_err(|_| {
            ContextError::EventError(EventError::SendFailed {
                message: "send failed".to_string(),
            })
        })
    }
}
#[derive(Clone)]
pub enum StateAccess {
    ReadOnly(Arc<DashMap<String, Value>>),
    ReadWrite(Arc<DashMap<String, Value>>),
}

impl StateAccess {
    pub async fn read(&self, key: &str) -> Result<Value, StateError> {
        match self {
            StateAccess::ReadOnly(state) | StateAccess::ReadWrite(state) => state
                .get(key)
                .map(|v| v.clone())
                .ok_or_else(|| StateError::NotFound {
                    key: key.to_string(),
                }),
        }
    }

    pub async fn write(&self, key: String, value: Value) -> Result<(), StateError> {
        match self {
            StateAccess::ReadWrite(state) => {
                state.insert(key, value);
                Ok(())
            }
            StateAccess::ReadOnly(_) => Err(StateError::AccessError(
                "Attempted to write to read-only state".to_string(),
            )),
        }
    }

    pub async fn exists(&self, key: &str) -> bool {
        match self {
            StateAccess::ReadOnly(state) | StateAccess::ReadWrite(state) => state.contains_key(key),
        }
    }
}

#[derive(Debug, Clone, strum::Display)]
pub enum ScopeError {
    VariableNotFound(String),
    TypeMismatch { expected: String, actual: String },
}

#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, Value>,
    parent: Option<Arc<Scope>>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Arc<Scope>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn lookup(&self, name: &str) -> Result<Value, ScopeError> {
        // 現在のスコープで検索
        if let Some(value) = self.variables.get(name) {
            return Ok(value.clone());
        }

        // 親スコープを再帰的に検索
        if let Some(parent) = &self.parent {
            return parent.lookup(name);
        }

        Err(ScopeError::VariableNotFound(name.to_string()))
    }

    pub fn insert(&mut self, name: String, value: Value) -> Option<Value> {
        self.variables.insert(name, value)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
            || self.parent.as_ref().map_or(false, |p| p.contains(name))
    }

    // スコープチェーンの深さを取得（デバッグ用）
    pub fn depth(&self) -> usize {
        1 + self.parent.as_ref().map_or(0, |p| p.depth())
    }
}

pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }

    pub fn push_scope(&mut self) {
        let parent = Arc::new(self.current_scope());
        self.scopes.push(Scope::with_parent(parent));
    }

    pub fn pop_scope(&mut self) -> Option<Scope> {
        if self.scopes.len() > 1 {
            self.scopes.pop()
        } else {
            None
        }
    }

    pub fn lookup(&self, name: &str) -> Result<Value, ScopeError> {
        self.current_scope().lookup(name)
    }

    pub fn insert(&mut self, name: String, value: Value) -> Option<Value> {
        self.current_scope_mut().insert(name, value)
    }

    pub fn current_scope(&self) -> Scope {
        self.scopes
            .last()
            .expect("At least one scope should exist")
            .clone()
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_context() -> ExecutionContext {
        let state = Arc::new(DashMap::new());
        let event_bus = Arc::new(EventBus::new(10));
        ExecutionContext::new_general_context(state, event_bus)
    }

    async fn setup_readonly_context() -> ExecutionContext {
        let state = Arc::new(DashMap::new());
        let event_bus = Arc::new(EventBus::new(10));
        ExecutionContext::new_answer_context(state, event_bus)
    }

    #[tokio::test]
    async fn test_state_variable_basic() {
        let mut context = setup_test_context().await;

        // write and read
        context
            .update_variable(
                VariableAccess::State("count".to_string()),
                Value::Integer(42),
            )
            .await
            .unwrap();

        let value = context
            .access_variable(VariableAccess::State("count".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(42));

        // read nonexistent
        let result = context
            .access_variable(VariableAccess::State("nonexistent".to_string()))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_readonly_state() {
        let mut context = setup_readonly_context().await;

        // attempt to write to readonly state
        let result = context
            .update_variable(
                VariableAccess::State("count".to_string()),
                Value::Integer(42),
            )
            .await;
        assert!(result.is_err());

        // can still write to local variables
        context
            .update_variable(
                VariableAccess::Local("local".to_string()),
                Value::Integer(42),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_local_variable_basic() {
        let mut context = setup_test_context().await;

        // write and read
        context
            .update_variable(VariableAccess::Local("x".to_string()), Value::Integer(42))
            .await
            .unwrap();

        let value = context
            .access_variable(VariableAccess::Local("x".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(42));

        // read nonexistent
        let result = context
            .access_variable(VariableAccess::Local("nonexistent".to_string()))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scope_hierarchy() {
        let mut context = setup_test_context().await;

        // set in outer scope
        context
            .update_variable(VariableAccess::Local("x".to_string()), Value::Integer(1))
            .await
            .unwrap();

        // create new scope and set same variable
        context.push_scope();
        context
            .update_variable(VariableAccess::Local("x".to_string()), Value::Integer(2))
            .await
            .unwrap();

        // verify inner scope value
        let value = context
            .access_variable(VariableAccess::Local("x".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(2));

        // pop scope and verify outer value
        context.pop_scope();
        let value = context
            .access_variable(VariableAccess::Local("x".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(1));
    }

    #[tokio::test]
    async fn test_state_and_local_variables() {
        let mut context = setup_test_context().await;

        // set both state and local variable
        context
            .update_variable(
                VariableAccess::State("state_var".to_string()),
                Value::Integer(1),
            )
            .await
            .unwrap();

        context
            .update_variable(
                VariableAccess::Local("local_var".to_string()),
                Value::Integer(2),
            )
            .await
            .unwrap();

        // verify both values
        let state_value = context
            .access_variable(VariableAccess::State("state_var".to_string()))
            .await
            .unwrap();
        assert_eq!(state_value, Value::Integer(1));

        let local_value = context
            .access_variable(VariableAccess::Local("local_var".to_string()))
            .await
            .unwrap();
        assert_eq!(local_value, Value::Integer(2));
    }

    #[tokio::test]
    async fn test_scope_isolation() {
        let mut context = setup_test_context().await;

        // set state variable
        context
            .update_variable(
                VariableAccess::State("shared".to_string()),
                Value::Integer(1),
            )
            .await
            .unwrap();

        // access in different scopes
        let value1 = context
            .access_variable(VariableAccess::State("shared".to_string()))
            .await
            .unwrap();

        context.push_scope();
        let value2 = context
            .access_variable(VariableAccess::State("shared".to_string()))
            .await
            .unwrap();

        assert_eq!(value1, value2);
    }

    #[test]
    fn test_scope_basic() {
        let mut scope = Scope::new();

        // 変数の挿入と検索
        scope.insert("x".to_string(), Value::Integer(42));
        assert_eq!(scope.lookup("x").unwrap(), Value::Integer(42));

        // 存在しない変数の検索
        assert!(matches!(
            scope.lookup("y"),
            Err(ScopeError::VariableNotFound(_))
        ));
    }

    #[test]
    fn test_scope_stack() {
        let mut stack = ScopeStack::new();

        // グローバルスコープに変数を追加
        stack.insert("global".to_string(), Value::Integer(1));

        // 新しいスコープをプッシュ
        stack.push_scope();
        stack.insert("local".to_string(), Value::Integer(2));

        // 変数の検索
        assert_eq!(stack.lookup("global").unwrap(), Value::Integer(1));
        assert_eq!(stack.lookup("local").unwrap(), Value::Integer(2));

        // スコープをポップ
        stack.pop_scope();
        assert_eq!(stack.lookup("global").unwrap(), Value::Integer(1));
        assert!(matches!(
            stack.lookup("local"),
            Err(ScopeError::VariableNotFound(_))
        ));
    }

    #[test]
    fn test_scope_shadowing() {
        let mut stack = ScopeStack::new();

        stack.insert("x".to_string(), Value::Integer(1));
        stack.push_scope();
        stack.insert("x".to_string(), Value::Integer(2));

        // 現在のスコープの値が取得される
        assert_eq!(stack.lookup("x").unwrap(), Value::Integer(2));

        stack.pop_scope();
        // 元の値が見える
        assert_eq!(stack.lookup("x").unwrap(), Value::Integer(1));
    }

    #[tokio::test]
    async fn test_state_variable_access() {
        let mut context = setup_test_context().await;

        // 状態変数の設定
        let result = context
            .update_variable(
                VariableAccess::State("count".to_string()),
                Value::Integer(42),
            )
            .await;
        assert!(result.is_ok());

        // 状態変数の読み取り
        let value = context
            .access_variable(VariableAccess::State("count".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(42));

        // 存在しない状態変数へのアクセス
        let error = context
            .access_variable(VariableAccess::State("nonexistent".to_string()))
            .await;
        assert!(error.is_err());
    }

    #[tokio::test]
    async fn test_local_variable_access() {
        let mut context = setup_test_context().await;

        // ローカル変数の設定
        let result = context
            .update_variable(VariableAccess::Local("x".to_string()), Value::Integer(1))
            .await;
        assert!(result.is_ok());

        // ローカル変数の読み取り
        let value = context
            .access_variable(VariableAccess::Local("x".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(1));

        // 存在しないローカル変数へのアクセス
        let error = context
            .access_variable(VariableAccess::Local("nonexistent".to_string()))
            .await;
        assert!(matches!(
            error,
            Err(ContextError::ScopeError(ScopeError::VariableNotFound(_)))
        ));
    }

    #[tokio::test]
    async fn test_readonly_state_access() {
        let mut context = setup_readonly_context().await;

        // 読み取り専用コンテキストでの書き込み試行
        let error = context
            .update_variable(
                VariableAccess::State("count".to_string()),
                Value::Integer(42),
            )
            .await;
        assert!(error.is_err());

        // ローカル変数への書き込みは許可される
        let result = context
            .update_variable(VariableAccess::Local("x".to_string()), Value::Integer(1))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scope_management() {
        let mut context = setup_test_context().await;

        // グローバルスコープでの変数設定
        context
            .update_variable(VariableAccess::Local("x".to_string()), Value::Integer(1))
            .await
            .unwrap();

        // 新しいスコープをプッシュ
        context.push_scope();

        // 新しいスコープでの同名変数の設定
        context
            .update_variable(VariableAccess::Local("x".to_string()), Value::Integer(2))
            .await
            .unwrap();

        // 現在のスコープでの値を確認
        let value = context
            .access_variable(VariableAccess::Local("x".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(2));

        // スコープをポップ
        context.pop_scope();

        // 元のスコープの値を確認
        let value = context
            .access_variable(VariableAccess::Local("x".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(1));
    }

    #[tokio::test]
    async fn test_mixed_variable_access() {
        let mut context = setup_test_context().await;

        // 状態変数とローカル変数の設定
        context
            .update_variable(
                VariableAccess::State("state_var".to_string()),
                Value::String("state value".to_string()),
            )
            .await
            .unwrap();

        context
            .update_variable(
                VariableAccess::Local("local_var".to_string()),
                Value::String("local value".to_string()),
            )
            .await
            .unwrap();

        // 両方の変数にアクセス
        let state_value = context
            .access_variable(VariableAccess::State("state_var".to_string()))
            .await
            .unwrap();

        let local_value = context
            .access_variable(VariableAccess::Local("local_var".to_string()))
            .await
            .unwrap();

        assert_eq!(state_value, Value::String("state value".to_string()));
        assert_eq!(local_value, Value::String("local value".to_string()));
    }

    #[tokio::test]
    async fn test_complex_scope_hierarchy() {
        let mut context = setup_test_context().await;

        // 複数のスコープレベルでのテスト
        context
            .update_variable(VariableAccess::Local("var".to_string()), Value::Integer(1))
            .await
            .unwrap();

        context.push_scope();
        context
            .update_variable(VariableAccess::Local("var".to_string()), Value::Integer(2))
            .await
            .unwrap();

        context.push_scope();

        // 最も内側のスコープでの値を確認
        let value = context
            .access_variable(VariableAccess::Local("var".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(2));

        // スコープを順にポップ
        context.pop_scope();
        let value = context
            .access_variable(VariableAccess::Local("var".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(2));

        context.pop_scope();
        let value = context
            .access_variable(VariableAccess::Local("var".to_string()))
            .await
            .unwrap();
        assert_eq!(value, Value::Integer(1));
    }
}
