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

// MicroAgentのトップレベル構造
#[derive(Debug, Clone, Default)]
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

// 状態定義
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

// イベント観察定義
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveDef {
    pub handlers: Vec<EventHandler>,
}

// リクエスト応答定義
#[derive(Debug, Clone, PartialEq)]
pub struct AnswerDef {
    pub handlers: Vec<RequestHandler>,
}

// システムへの反応定義
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

// 型情報
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
        constraints: HashMap<String, Expression>,
    },
}

impl From<&str> for TypeInfo {
    fn from(value: &str) -> Self {
        Self::Simple(value.to_string())
    }
}

// コードブロック
#[derive(Debug, Clone, PartialEq)]
pub struct HandlerBlock {
    pub statements: Vec<Statement>,
}

type Statements = Vec<Statement>;

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorHandlerBlock {
    // Error variable name to be bound in the handler scope
    pub error_binding: Option<String>,
    pub error_handler_statements: Statements,
}

// 文
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // expressions
    Expression(Expression),
    Assignment {
        target: Expression,
        value: Expression,
    },
    Return(Expression),
    // events
    Emit {
        event_type: EventType,
        parameters: Vec<Argument>,
        target: Option<String>, // Noneの場合はブロードキャスト
    },
    Request {
        agent: String,
        request_type: RequestType,
        parameters: Vec<Argument>,
        options: Option<RequestAttributes>,
    },
    // grouping
    Block(Statements),
    Await(AwaitType),
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

#[derive(Debug, Clone, PartialEq)]
pub enum AwaitType {
    // 複数のStatementを並列実行して全ての完了を待つ
    Block(Statements),
    // 単一のStatementの完了を待つ
    Single(Box<Statement>),
}

use std::fmt::{Display, Formatter};
use std::time::Duration;

use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

// 式
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
        };

        assert_eq!(handler.error_binding.unwrap(), "err");
    }
}
