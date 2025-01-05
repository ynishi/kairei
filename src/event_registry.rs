use crate::{ast, EventError, RuntimeError, RuntimeResult, TypeInfo};
use dashmap::DashMap;
use std::{collections::HashMap, str::FromStr, sync::Arc};

/// イベントのメタデータ
#[derive(Clone, Debug)]
pub struct EventInfo {
    pub event_type: EventType,
    pub parameters: HashMap<String, ParameterType>, // (パラメータ名, 型)
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, strum::EnumString, Default, PartialOrd, Ord)]
pub enum EventType {
    #[default]
    Tick,
    StateUpdated {
        agent_name: String,
        state_name: String,
    },
    // メッセージイベント
    // emit something で Publish する際に使用する。
    Message {
        content_type: String,
    },
    // something.onFail などで停止した際にこのイベントを使用する。
    Failure {
        error_type: String,
    },
    // Request/Response
    Request {
        request_type: String,
        requester: String,  // リクエストの送信者
        responder: String,  // 期待される応答者
        request_id: String, // Globally unique request ID like UUID
    },
    // Response は、Ok(ResponseSuccess)/Err(ResponseFailure) を使用する。
    ResponseSuccess {
        request_type: String,
        requester: String,  // 元のリクエストの送信者
        responder: String,  // 応答者
        request_id: String, // Globally unique request ID like UUID
    },
    ResponseFailure {
        request_type: String,
        requester: String,  // 元のリクエストの送信者
        responder: String,  // 応答者
        request_id: String, // Globally unique request ID like UUID
    },
    // Lifecycle
    AgentCreated,
    AgentAdded,
    AgentRemoved,
    AgentStarting,
    AgentStarted,
    AgentStopping,
    AgentStopped,
    // SystemLifecycle
    SystemCreated,
    SystemWorldRegistered,
    SystemBuiltinAgentsRegistered,
    SystemUserAgentsRegistered,
    SystemStarting,
    SystemStarted,
    SystemStopping,
    SystemStopped,
    Custom(String), // 拡張性のために残す
}

impl EventType {
    /// リクエストイベントかどうか
    pub fn is_request(&self) -> bool {
        matches!(self, EventType::Request { .. })
    }

    pub fn is_response(&self) -> bool {
        matches!(
            self,
            EventType::ResponseSuccess { .. } | EventType::ResponseFailure { .. }
        )
    }

    pub fn is_for_me(&self, agent_name: &str) -> bool {
        match self {
            EventType::ResponseSuccess { requester, .. }
            | EventType::ResponseFailure { requester, .. } => requester == agent_name,
            _ => false,
        }
    }

    pub fn request_id(&self) -> Option<&str> {
        match self {
            EventType::Request { request_id, .. } => Some(request_id),
            EventType::ResponseSuccess { request_id, .. } => Some(request_id),
            EventType::ResponseFailure { request_id, .. } => Some(request_id),
            _ => None,
        }
    }

    /// 成功イベントかどうか
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            EventType::Message { .. } | EventType::ResponseSuccess { .. }
        )
    }

    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            EventType::Failure { .. } | EventType::ResponseFailure { .. }
        )
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Tick => write!(f, "Tick"),
            EventType::StateUpdated {
                agent_name,
                state_name,
            } => write!(f, "StateUpdated({}.{})", agent_name, state_name),
            EventType::Message { content_type } => write!(f, "{}", content_type),
            EventType::Failure { error_type } => write!(f, "{}", error_type),
            EventType::Request { request_type, .. } => write!(f, "{}", request_type),
            EventType::ResponseSuccess { request_type, .. } => write!(f, "{}", request_type),
            EventType::ResponseFailure { request_type, .. } => write!(f, "{}", request_type),
            EventType::AgentCreated => write!(f, "AgentCreated"),
            EventType::AgentAdded => write!(f, "AgentAdded"),
            EventType::AgentRemoved => write!(f, "AgentRemoved"),
            EventType::AgentStarting => write!(f, "AgentStarting"),
            EventType::AgentStarted => write!(f, "AgentStarted"),
            EventType::AgentStopping => write!(f, "AgentStopping"),
            EventType::AgentStopped => write!(f, "AgentStopped"),
            EventType::SystemCreated => write!(f, "SystemCreated"),
            EventType::SystemWorldRegistered => write!(f, "SystemWorldRegistered"),
            EventType::SystemBuiltinAgentsRegistered => write!(f, "SystemBuiltinAgentsRegistered"),
            EventType::SystemUserAgentsRegistered => write!(f, "SystemUserAgentsRegistered"),
            EventType::SystemStarting => write!(f, "SystemStarting"),
            EventType::SystemStarted => write!(f, "SystemStarted"),
            EventType::SystemStopping => write!(f, "SystemStopping"),
            EventType::SystemStopped => write!(f, "SystemStopped"),
            EventType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl From<&ast::EventType> for EventType {
    fn from(event_type: &ast::EventType) -> Self {
        match event_type {
            ast::EventType::Tick => Self::Tick,
            ast::EventType::StateUpdated {
                agent_name,
                state_name,
            } => Self::StateUpdated {
                agent_name: agent_name.clone(),
                state_name: state_name.clone(),
            },
            ast::EventType::Message { content_type } => Self::Message {
                content_type: content_type.clone(),
            },

            ast::EventType::Custom(name) => Self::Custom(name.clone()),
        }
    }
}

/// ライフサイクルイベント
#[derive(Debug, Clone, PartialEq, Hash, Eq, strum::Display)]
pub enum LifecycleEvent {
    OnInit,
    OnDestroy,
}

/// イベントパラメータの型情報
#[derive(Clone, Debug, PartialEq, strum::EnumString, strum::Display, Default)]
pub enum ParameterType {
    #[default]
    String,
    Int,
    Float,
    Boolean,
    Duration,
    DateTime,
    Json,
    Custom(String), // カスタム型
    List(Box<ParameterType>),
    Map(Box<ParameterType>, Box<ParameterType>),
}

impl From<TypeInfo> for ParameterType {
    fn from(type_info: TypeInfo) -> Self {
        match type_info {
            TypeInfo::Simple(s) => ParameterType::from_str(s.as_str()).unwrap(),
            TypeInfo::Custom { name, .. } => ParameterType::from_str(name.as_str()).unwrap(),
            _ => todo!(),
        }
    }
}

/// イベントレジストリ
#[derive(Default)]
pub struct EventRegistry {
    events: Arc<DashMap<EventType, EventInfo>>,
}

impl EventRegistry {
    /// 新しいEventRegistryを作成
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_builtin_events();
        registry
    }

    /// 組み込みイベントを登録
    fn register_builtin_events(&mut self) {
        // Tick イベント
        let mut parameters = HashMap::new();
        parameters.insert("delta_time".to_string(), ParameterType::Float);
        self.register_event(EventInfo {
            event_type: EventType::Tick,
            parameters,
        })
        .unwrap();
    }

    /// 新しいイベントを登録
    pub fn register_event(&mut self, event_info: EventInfo) -> RuntimeResult<()> {
        if self.events.contains_key(&event_info.event_type) {
            match &event_info.event_type {
                EventType::Custom(name) => {
                    return Err(RuntimeError::Event(EventError::AlreadyRegistered {
                        event_type: name.clone(),
                    }))
                }
                _ => return Err(RuntimeError::Event(EventError::BuiltInAlreadyRegistered)),
            }
        }

        self.events
            .insert(event_info.event_type.clone(), event_info);
        Ok(())
    }

    /// カスタムイベントを登録（DSLからの登録用）
    pub fn register_custom_event(
        &mut self,
        name: String,
        parameters: HashMap<String, ParameterType>,
    ) -> RuntimeResult<()> {
        let event_info = EventInfo {
            event_type: EventType::Custom(name.clone()),
            parameters,
        };
        self.register_event(event_info)
    }

    /// イベント情報を取得
    pub fn get_event_info(&self, event_type: &EventType) -> Option<EventInfo> {
        self.events.get(event_type).map(|info| info.clone())
    }

    /// イベントが登録されているか確認
    pub fn contains_event(&self, event_type: &EventType) -> bool {
        self.events.contains_key(event_type)
    }

    /// イベントのパラメータを検証
    pub fn validate_parameters(
        &self,
        event_type: &EventType,
        parameters: &[(String, ParameterType)],
    ) -> RuntimeResult<()> {
        let event_info = self
            .get_event_info(event_type)
            .ok_or_else(|| RuntimeError::Event(EventError::NotFound(event_type.to_string())))?;

        if parameters.len() != event_info.parameters.len() {
            return Err(RuntimeError::Event(
                EventError::ParametersLengthNotMatched {
                    event_type: event_type.to_string(),
                    expected: event_info.parameters.len(),
                    got: parameters.len(),
                },
            ));
        }

        // パラメータの名前と型を検証
        for (name, param_type) in parameters.iter() {
            let expected = &event_info.parameters[name];
            if param_type != expected {
                return Err(RuntimeError::Event(EventError::TypeMismatch {
                    event_type: event_type.to_string(),
                    expected: param_type.to_string(),
                    got: expected.to_string(),
                }));
            }
        }

        Ok(())
    }

    /// 全てのイベント情報を取得
    pub fn get_all_events(&self) -> Vec<EventInfo> {
        self.events
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// カスタムイベントの一覧を取得
    pub fn get_custom_events(&self) -> Vec<EventInfo> {
        self.events
            .iter()
            .filter_map(|entry| match &entry.value().event_type {
                EventType::Custom(_) => Some(entry.value().clone()),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_events() {
        let registry = EventRegistry::new();
        assert!(registry.contains_event(&EventType::Tick));
    }

    #[test]
    fn test_custom_event_registration() {
        let mut registry = EventRegistry::new();
        let mut parameters = HashMap::new();
        parameters.insert("player_id".to_string(), ParameterType::String);
        parameters.insert("x".to_string(), ParameterType::Float);
        parameters.insert("y".to_string(), ParameterType::Float);
        let result = registry.register_custom_event("PlayerMoved".to_string(), parameters);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parameter_validation() {
        let mut registry = EventRegistry::new();
        let event_type = EventType::Custom("TestEvent".to_string());

        let mut parameters = HashMap::new();
        parameters.insert("param1".to_string(), ParameterType::String);
        parameters.insert("param2".to_string(), ParameterType::Int);

        // イベントを登録
        registry
            .register_event(EventInfo {
                event_type: event_type.clone(),
                parameters,
            })
            .unwrap();

        // 正しいパラメータ
        let valid_params = vec![
            ("param1".to_string(), ParameterType::String),
            ("param2".to_string(), ParameterType::Int),
        ];
        assert!(registry
            .validate_parameters(&event_type, &valid_params)
            .is_ok());

        // 誤ったパラメータ
        let invalid_params = vec![
            ("param1".to_string(), ParameterType::Int),
            ("param2".to_string(), ParameterType::String),
        ];
        assert!(registry
            .validate_parameters(&event_type, &invalid_params)
            .is_err());
    }
}
