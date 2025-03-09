use core::fmt;
use std::collections::HashMap;

// Root AST Definition
#[derive(Debug, Clone)]
pub struct Root {
    pub world_def: Option<WorldDef>,
    pub micro_agent_defs: Vec<MicroAgentDef>,
}

impl Root {
    pub fn new(world_def: Option<WorldDef>, micro_agent_defs: Vec<MicroAgentDef>) -> Self {
        Self {
            world_def,
            micro_agent_defs,
        }
    }
}

/// MicroAgent DSL Core Definition
///
/// The MicroAgent DSL provides a structured way to define autonomous agents that can:
/// - Maintain internal state
/// - Observe and react to events
/// - Handle requests with type-safe responses
/// - Integrate with LLM capabilities
///
/// # Core Components
/// - `state`: Define agent's internal state variables
/// - `observe`: Monitor and respond to environment changes
/// - `answer`: Handle explicit requests with type-safe responses
/// - `react`: Implement proactive behaviors
///
/// # Example
/// ```text
/// micro CounterAgent {
///     state {
///         count: Int = 0
///     }
///
///     observe {
///         on Tick {
///             self.count += 1
///         }
///     }
///
///     answer {
///         on request GetCount() -> Result<Int> {
///             Ok(self.count)
///         }
///     }
/// }
/// ```
///
/// # State Management
/// State variables are strongly typed and can be:
/// - Read in any block type
/// - Modified in observe and react blocks
/// - Read-only in answer blocks
///
/// # Event Handling
/// The DSL supports different types of event handlers:
/// - `observe`: For monitoring state changes and system events
/// - `answer`: For handling explicit requests with responses
/// - `react`: For implementing autonomous behaviors
///
/// # Type Safety
/// The DSL enforces type safety through:
/// - Strong typing of state variables
/// - Type-checked event parameters
/// - Validated request/response signatures
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MicroAgentDef {
    pub name: String,
    pub policies: Vec<Policy>,
    pub lifecycle: Option<LifecycleDef>,
    pub state: Option<StateDef>,
    pub observe: Option<ObserveDef>,
    pub answer: Option<AnswerDef>,
    pub react: Option<ReactDef>,
}

// ライフサイクル定義
#[derive(Debug, Clone, PartialEq)]
pub struct LifecycleDef {
    pub on_init: Option<HandlerBlock>,
    pub on_destroy: Option<HandlerBlock>,
}

/// State Definition Block
///
/// Defines the internal state variables of a MicroAgent.
/// Each state variable has a name, type, and optional initial value.
///
/// # Example
/// ```text
/// state {
///     counter: Int = 0
///     name: String = "agent"
///     data: CustomType
/// }
/// ```
///
/// State variables are:
/// - Strongly typed
/// - Accessible from all handler blocks
/// - Mutable in observe and react blocks
/// - Read-only in answer blocks
#[derive(Debug, Clone, PartialEq)]
pub struct StateDef {
    pub variables: HashMap<String, StateVarDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateVarDef {
    pub name: String,
    pub type_info: TypeInfo,
    pub initial_value: Option<Expression>,
}

/// Observe Block Definition
///
/// Defines handlers for monitoring and responding to events in the environment.
/// Handlers in this block can modify agent state and emit new events.
///
/// # Example
/// ```text
/// observe {
///     on Tick {
///         self.counter += 1
///     }
///
///     on StateUpdated.otherAgent.status {
///         // React to other agent's state change
///     }
/// }
/// ```
///
/// Observe handlers:
/// - Can modify agent state
/// - Can emit new events
/// - Cannot return values
/// - Are triggered by system or custom events
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveDef {
    pub handlers: Vec<EventHandler>,
}

/// Answer Block Definition
///
/// Defines handlers for responding to explicit requests from other agents.
/// These handlers provide type-safe responses and have read-only access to state.
///
/// # Example
/// ```text
/// answer {
///     on request GetStatus() -> Result<Status> {
///         with {
///             strictness: 0.9,
///             stability: 0.8
///         }
///         Ok(self.status)
///     }
/// }
/// ```
///
/// Answer handlers:
/// - Have read-only access to state
/// - Must return a Result type
/// - Can specify quality constraints
/// - Support error handling
#[derive(Debug, Clone, PartialEq)]
pub struct AnswerDef {
    pub handlers: Vec<RequestHandler>,
}

/// React Block Definition
///
/// Defines handlers for implementing proactive behaviors in response to events.
/// These handlers can modify state and initiate interactions with other agents.
///
/// # Example
/// ```text
/// react {
///     on NewData(data: Data) {
///         self.process_data = data
///         emit DataProcessed(self.process_data)
///     }
/// }
/// ```
///
/// React handlers:
/// - Can modify agent state
/// - Can emit events and make requests
/// - Support complex event processing
/// - Enable autonomous behavior
#[derive(Debug, Clone, PartialEq)]
pub struct ReactDef {
    pub handlers: Vec<EventHandler>,
}

// Worldの定義
// World全体の定義
#[derive(Debug, Clone, PartialEq)]
pub struct WorldDef {
    pub name: String,
    pub policies: Vec<Policy>,
    pub config: Option<ConfigDef>,
    pub events: EventsDef,
    pub handlers: HandlersDef,
}

// 設定定義
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigDef {
    pub tick_interval: Duration,
    pub max_agents: usize,
    pub event_buffer_size: usize,
}

impl Default for ConfigDef {
    fn default() -> Self {
        Self {
            tick_interval: Duration::from_secs(1),
            max_agents: 1000,
            event_buffer_size: 1000,
        }
    }
}

impl From<HashMap<String, Literal>> for ConfigDef {
    fn from(map: HashMap<String, Literal>) -> Self {
        let tick_interval = map
            .get("tick_interval")
            .and_then(|v| match v {
                Literal::Duration(d) => Some(*d),
                _ => None,
            })
            .unwrap_or(Duration::from_secs(1));
        let max_agents = map
            .get("max_agents")
            .and_then(|v| match v {
                Literal::Integer(i) => Some(*i as usize),
                _ => None,
            })
            .unwrap_or(1000);
        let event_buffer_size = map
            .get("event_buffer_size")
            .and_then(|v| match v {
                Literal::Integer(i) => Some(*i as usize),
                _ => None,
            })
            .unwrap_or(1000);
        Self {
            tick_interval,
            max_agents,
            event_buffer_size,
        }
    }
}

// イベント定義のコレクション
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EventsDef {
    pub events: Vec<CustomEventDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomEventDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
}

impl From<EventsDef> for Vec<EventType> {
    fn from(events_def: EventsDef) -> Self {
        events_def
            .events
            .into_iter()
            .map(|event| EventType::Custom(event.name))
            .collect()
    }
}

// ハンドラー定義のコレクション
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HandlersDef {
    pub handlers: Vec<HandlerDef>,
}

// 個別のハンドラー定義
#[derive(Debug, Clone, PartialEq)]
pub struct HandlerDef {
    pub event_name: String,
    pub parameters: Vec<Parameter>,
    pub block: HandlerBlock,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum EventType {
    Tick,
    StateUpdated {
        agent_name: String,
        state_name: String,
    },
    Message {
        content_type: String,
    },
    Custom(String), // 拡張性のために残す
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EventType::Tick => write!(f, "Tick"),
            EventType::StateUpdated {
                agent_name,
                state_name,
            } => write!(f, "StateUpdated.{}.{}", agent_name, state_name),
            EventType::Message { content_type } => write!(f, "{}", content_type),
            EventType::Custom(name) => write!(f, "{}", name),
        }
    }
}

// イベントハンドラ
#[derive(Debug, Clone, PartialEq)]
pub struct EventHandler {
    pub event_type: EventType,
    pub parameters: Vec<Parameter>, // イベントの型に応じたパラメータ定義
    pub block: HandlerBlock,
}

impl From<HandlerDef> for EventHandler {
    fn from(handler: HandlerDef) -> Self {
        Self {
            event_type: EventType::Custom(handler.event_name),
            parameters: handler.parameters,
            block: handler.block,
        }
    }
}

impl EventHandler {
    /// イベントタイプに応じた適切なパラメータを持っているか検証
    pub fn validate_parameters(&self) -> Result<(), String> {
        match &self.event_type {
            EventType::Tick => {
                if !self.parameters.is_empty() {
                    return Err("Tick event should not have parameters".to_string());
                }
            }
            EventType::StateUpdated { .. } => {
                // StateUpdatedイベントのパラメータ検証
            }
            EventType::Message { .. } => {
                // Messageイベントのパラメータ検証
            }
            EventType::Custom(_) => {
                // カスタムイベントは任意のパラメータを許容
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestType {
    Query { query_type: String },
    Action { action_type: String },
    Custom(String), // 拡張性のために残す
}

impl From<&str> for RequestType {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_string())
    }
}

impl fmt::Display for RequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestType::Query { query_type } => write!(f, "Query.{}", query_type),
            RequestType::Action { action_type } => write!(f, "Action.{}", action_type),
            RequestType::Custom(name) => write!(f, "{}", name),
        }
    }
}

// リクエストハンドラ
#[derive(Debug, Clone, PartialEq)]
pub struct RequestHandler {
    pub request_type: RequestType,
    pub parameters: Vec<Parameter>, // リクエストの型に応じたパラメータ定義
    pub return_type: TypeInfo,
    pub constraints: Option<Constraints>,
    pub block: HandlerBlock,
}

impl RequestHandler {
    /// リクエストタイプに応じた適切なパラメータと戻り値の型を持っているか検証
    pub fn validate_signature(&self) -> Result<(), String> {
        match &self.request_type {
            RequestType::Query { .. } => {
                // クエリタイプに応じたパラメータと戻り値の型を検証
            }
            RequestType::Action { .. } => {
                // アクションタイプに応じたパラメータと戻り値の型を検証
            }
            RequestType::Custom(_) => {
                // カスタムリクエストは任意のパラメータを許容
            }
        }
        Ok(())
    }
}

// パラメータ定義
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_info: TypeInfo,
}

// 制約定義
#[derive(Debug, Clone, PartialEq)]
pub struct Constraints {
    pub strictness: Option<f64>,
    pub stability: Option<f64>,
    pub latency: Option<u32>,
}

/// Type Information for the MicroAgent DSL
///
/// Represents the type system that ensures type safety across the DSL.
/// Supports basic types, generic types, and custom type definitions.
///
/// # Type Categories
/// - Simple types (Int, String, etc.)
/// - Generic types (Result, Option, Array)
/// - Custom types with fields
/// - Map types for key-value structures
///
/// # Example
/// ```text
/// type UserProfile {
///     id: String
///     data: Map<String, Any>
///     settings: Option<Settings>
/// }
/// ```
///
/// The type system ensures:
/// - Type safety in state definitions
/// - Parameter type checking
/// - Return type validation
/// - Generic type handling
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    Simple(String), // 基本型 (Int, String等)
    Result {
        ok_type: Box<TypeInfo>,
        err_type: Box<TypeInfo>,
    },
    Option(Box<TypeInfo>),
    Array(Box<TypeInfo>),
    Map(Box<TypeInfo>, Box<TypeInfo>),
    Custom {
        name: String,
        fields: HashMap<String, FieldInfo>,
    },
}

impl TypeInfo {
    pub fn any() -> Self {
        Self::Simple("Any".to_string())
    }

    pub fn is_any(&self) -> bool {
        match self {
            Self::Simple(name) => name == "Any",
            _ => false,
        }
    }
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeInfo::Simple(name) => write!(f, "{}", name),
            TypeInfo::Result { ok_type, err_type } => {
                write!(f, "Result<{}, {}>", ok_type, err_type)
            }
            TypeInfo::Option(inner) => write!(f, "Option<{}>", inner),
            TypeInfo::Array(inner) => write!(f, "Array<{}>", inner),
            TypeInfo::Map(key, value) => write!(f, "Map<{}, {}>", key, value),
            TypeInfo::Custom { name, fields } => {
                write!(f, "{}", name)?;

                if !fields.is_empty() {
                    write!(f, " {{")?;
                    for field_name in fields.keys() {
                        write!(f, "{}", field_name)?;
                        write!(f, ", ")?;
                    }
                    write!(f, "}}")?;
                }
                Ok(())
            }
        }
    }
}

impl From<&str> for TypeInfo {
    fn from(value: &str) -> Self {
        Self::Simple(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfo {
    pub type_info: Option<TypeInfo>, // None の場合は型推論
    pub default_value: Option<Expression>,
}

impl fmt::Display for FieldInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.type_info.as_ref().unwrap())
    }
}

// コードブロック
#[derive(Debug, Clone, PartialEq)]
pub struct HandlerBlock {
    pub statements: Vec<Statement>,
}

pub type Statements = Vec<Statement>;

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorHandlerBlock {
    // Error variable name to be bound in the handler scope
    pub error_binding: Option<String>,
    pub error_handler_statements: Statements,
    pub control: Option<OnFailControl>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OnFailControl {
    Return(OnFailReturn),
    Rethrow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OnFailReturn {
    Ok(Expression),
    Err(Expression),
}

// 文
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // expressions
    Expression(Expression),
    Assignment {
        target: Vec<Expression>,
        value: Expression,
    },
    Return(Expression),
    // events
    Emit {
        event_type: EventType,
        parameters: Vec<Argument>,
        target: Option<String>, // Noneの場合はブロードキャスト
    },
    // grouping
    Block(Statements),
    WithError {
        statement: Box<Statement>,
        error_handler_block: ErrorHandlerBlock,
    },
    // control flow
    If {
        condition: Expression,
        then_block: Statements,
        else_block: Option<Statements>,
    },
}

// Extension trait for Statement building
pub trait StatementExt {
    fn on_fail(self, handler: ErrorHandlerBlock) -> Statement;
}

impl StatementExt for Statement {
    fn on_fail(self, error_handler_block: ErrorHandlerBlock) -> Statement {
        Statement::WithError {
            statement: Box::new(self),
            error_handler_block,
        }
    }
}

use std::fmt::{Display, Formatter};
use std::time::Duration;

use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::tokenizer::token::TokenizerError;
use crate::type_checker::TypeCheckError;

// リクエストオプション
#[derive(Debug, Clone, PartialEq)]
pub struct RequestAttributes {
    pub timeout: Option<Duration>,
    pub retry: Option<u32>, // 回数
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateAccessPath(pub Vec<String>); // user.profile.name -> vec!["user", "profile", "name"]

impl Display for StateAccessPath {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}

impl StateAccessPath {
    pub fn from_dot_path(path: &str) -> Self {
        Self(path.split('.').map(String::from).collect())
    }
}

/// Expression Types in the MicroAgent DSL
///
/// Defines all possible expressions that can appear in agent code.
/// Includes state access, function calls, think blocks, and requests.
///
/// # Expression Categories
/// - Literals and variables
/// - State access expressions
/// - Function calls
/// - Think blocks for LLM integration
/// - Request expressions
/// - Await expressions for async operations
/// - Binary operations
///
/// # Examples
/// ```text
/// // State access
/// self.counter
///
/// // Think block
/// think("Analyze data") with {
///     model: "gpt-4"
/// }
///
/// // Request
/// request otherAgent.GetStatus()
/// ```
///
/// Expressions support:
/// - Type inference
/// - Async/await operations
/// - Error handling with Result
/// - LLM integration through think blocks
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    StateAccess(StateAccessPath),
    FunctionCall {
        function: String,
        arguments: Vec<Expression>,
    },
    Think {
        args: Vec<Argument>,
        with_block: Option<ThinkAttributes>,
    },
    Request {
        agent: String,
        request_type: RequestType,
        parameters: Vec<Argument>,
        options: Option<RequestAttributes>,
    },
    Await(Vec<Expression>),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Ok(Box<Expression>),
    Err(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ThinkAttributes {
    // 必須項目
    pub provider: Option<String>, // Noneの場合はデフォルトプロバイダー
    pub prompt_generator_type: Option<PromptGeneratorType>,
    pub policies: Vec<Policy>,

    // オプション項目
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,

    // リトライ設定
    pub retry: Option<RetryConfig>,

    // Plugin用の拡張項目
    pub plugins: HashMap<String, HashMap<String, Literal>>,
}

// プロンプトジェネレータータイプ
#[derive(Debug, Clone, PartialEq)]
pub enum PromptGeneratorType {
    Standard,
    // 将来の拡張用
    // Detailed,
    // Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u64,
    pub delay: RetryDelay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetryDelay {
    Fixed(u64), // ミリ秒
    Exponential {
        initial: u64, // ミリ秒
        max: u64,     // ミリ秒
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Policy {
    pub text: String,
    pub scope: PolicyScope,
    // 内部的なID - ビルトインポリシーやシステムでの追跡用
    pub internal_id: PolicyId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct PolicyId(pub String);

impl PolicyId {
    // ビルトインポリシー用のID生成
    pub fn builtin(name: &str) -> Self {
        PolicyId(format!("builtin:{}", name))
    }

    // 通常のポリシー用のID生成（自動生成）
    pub fn new() -> Self {
        use uuid::Uuid;
        PolicyId(format!("policy:{}", Uuid::new_v4()))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum PolicyScope {
    World(String), // World名
    Agent(String), // Agent名
    Think,         // Think式スコープ
}

// リテラル
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Duration(Duration),
    List(Vec<Literal>),
    Map(HashMap<String, Literal>),
    Retry(RetryConfig),
    Null,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Integer(i) => write!(f, "{}", i),
            Literal::Float(f_data) => write!(f, "{}", f_data),
            Literal::String(s) => write!(f, "{}", s),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Duration(d) => write!(f, "{:?}", d),
            Literal::List(literals) => {
                write!(f, "[")?;
                for (i, literal) in literals.iter().enumerate() {
                    write!(f, "{}", literal)?;
                    if i < literals.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Literal::Map(hash_map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in hash_map.iter().enumerate() {
                    write!(f, "{}: {}", key, value)?;
                    if i < hash_map.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
            Literal::Retry(retry_config) => write!(f, "{:?}", retry_config),
            Literal::Null => write!(f, "null"),
        }
    }
}

// Argumentの型
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    Named { name: String, value: Expression },
    Positional(Expression),
}

// 二項演算子
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    And,
    Or,
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::LessThanEqual => write!(f, "<="),
            BinaryOperator::GreaterThanEqual => write!(f, ">="),
            BinaryOperator::And => write!(f, "&&"),
            BinaryOperator::Or => write!(f, "||"),
        }
    }
}

// 文（MicroAgentと共通だが、World用に制限される）
#[derive(Debug, Clone, PartialEq)]
pub enum WorldStatement {
    Log(Expression),
    EmitEvent {
        event_name: String,
        parameters: Vec<Expression>,
    },
    Expression(Expression),
    If {
        condition: Expression,
        then_block: Statements,
        else_block: Option<Statements>,
    },
}

// 式（MicroAgentと共通だが、World用に制限される）
#[derive(Debug, Clone, PartialEq)]
pub enum WorldExpression {
    Literal(Literal),
    Variable(String),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
}

impl From<WorldDef> for (MicroAgentDef, EventsDef) {
    fn from(world: WorldDef) -> (MicroAgentDef, EventsDef) {
        let mut variables = HashMap::new();
        // world.config.tick_interval
        let config = world.config.unwrap_or_default();
        variables.insert(
            "tick_interval".to_string(),
            StateVarDef {
                name: "tick_interval".to_string(),
                type_info: TypeInfo::Simple("Duration".to_string()),
                initial_value: Some(Expression::Literal(Literal::Duration(config.tick_interval))),
            },
        );
        // world.config.max_agents
        variables.insert(
            "max_agents".to_string(),
            StateVarDef {
                name: "max_agents".to_string(),
                type_info: TypeInfo::Simple("usize".to_string()),
                initial_value: Some(Expression::Literal(Literal::Integer(
                    config.max_agents as i64,
                ))),
            },
        );
        // world.config.event_buffer_size
        variables.insert(
            "event_buffer_size".to_string(),
            StateVarDef {
                name: "event_buffer_size".to_string(),
                type_info: TypeInfo::Simple("usize".to_string()),
                initial_value: Some(Expression::Literal(Literal::Integer(
                    config.event_buffer_size as i64,
                ))),
            },
        );
        let agent = MicroAgentDef {
            name: "world".to_string(),
            policies: vec![],
            state: Some(StateDef { variables }),
            observe: Some(ObserveDef {
                handlers: world
                    .handlers
                    .handlers
                    .iter()
                    .filter(|h| !h.event_name.starts_with("request"))
                    .map(|h| h.clone().into())
                    .collect(),
            }),
            answer: None,
            react: Some(ReactDef {
                handlers: world
                    .handlers
                    .handlers
                    .iter()
                    .filter(|h| h.event_name.starts_with("request"))
                    .map(|h| h.clone().into())
                    .collect(),
            }),
            lifecycle: None,
        };

        (agent, world.events)
    }
}

// コード生成用のトレイト
pub trait CodeGen {
    fn generate_rust(&self) -> TokenStream;
}

#[derive(Error, Debug)]
pub enum ASTError {
    #[error("Parse error: {target}: {message}")]
    ParseError { target: String, message: String },
    #[error("AST not found: {0}")]
    ASTNotFound(String),
    #[error("Type check error: {0}")]
    TypeCheckError(#[from] TypeCheckError),
    #[error("Tokenization error: {0}")]
    TokenizeError(#[from] TokenizerError),
}

pub type ASTResult<T> = Result<T, ASTError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_conversion() {
        let world = WorldDef {
            name: "TestWorld".to_string(),
            policies: vec![],
            config: Some(ConfigDef {
                tick_interval: Duration::from_millis(100),
                max_agents: 1000,
                event_buffer_size: 500,
            }),
            events: EventsDef {
                events: vec![CustomEventDef {
                    name: "TestEvent".to_string(),
                    parameters: vec![],
                }],
            },
            handlers: HandlersDef {
                handlers: vec![HandlerDef {
                    event_name: "Tick".to_string(),
                    parameters: vec![],
                    block: HandlerBlock { statements: vec![] },
                }],
            },
        };

        let (agent, events) = world.into();

        // 変換結果の検証
        assert_eq!(agent.name, "world");
        let state = agent.state.unwrap();
        assert_eq!(state.variables.len(), 3);
        assert!(agent.observe.is_some());
        assert!(agent.answer.is_none());

        // イベント定義の検証
        assert_eq!(events.events.len(), 1);
        assert_eq!(events.events[0].name, "TestEvent");
    }

    #[test]
    fn test_statement_error_handler() {
        let emit_stmt = Statement::Emit {
            event_type: EventType::Custom("TestEvent".into()),
            parameters: vec![],
            target: None,
        };

        let handler = ErrorHandlerBlock {
            error_binding: None,
            error_handler_statements: vec![Statement::Emit {
                event_type: EventType::Custom("TestEvent".into()),
                parameters: vec![],
                target: None,
            }],
            control: None,
        };

        let with_handler = emit_stmt.on_fail(handler);

        match with_handler {
            Statement::WithError {
                statement,
                error_handler_block,
            } => {
                assert!(matches!(*statement, Statement::Emit { .. }));
                assert!(error_handler_block.error_binding.is_none());
            }
            _ => panic!("Expected WithErrorHandler variant"),
        }
    }

    #[test]
    fn test_error_handler_with_binding() {
        let handler = ErrorHandlerBlock {
            error_binding: Some("err".to_string()),
            error_handler_statements: vec![],
            control: None,
        };

        assert_eq!(handler.error_binding.unwrap(), "err");
    }
}
