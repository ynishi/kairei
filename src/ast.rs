use std::collections::HashMap;

// MicroAgentのトップレベル構造
#[derive(Debug, Clone, Default)]
pub struct MicroAgentDef {
    pub name: String,
    pub lifecycle: Option<LifecycleDef>,
    pub state: Option<StateDef>,
    pub observe: Option<ObserveDef>,
    pub answer: Option<AnswerDef>,
    pub react: Option<ReactDef>,
}

// ライフサイクル定義
#[derive(Debug, Clone)]
pub struct LifecycleDef {
    pub on_init: Option<Block>,
    pub on_destroy: Option<Block>,
}

// 状態定義
#[derive(Debug, Clone)]
pub struct StateDef {
    pub variables: HashMap<String, StateVarDef>,
}

#[derive(Debug, Clone)]
pub struct StateVarDef {
    pub name: String,
    pub type_info: TypeInfo,
    pub initial_value: Option<Expression>,
}

// イベント観察定義
#[derive(Debug, Clone)]
pub struct ObserveDef {
    pub handlers: Vec<EventHandler>,
}

// リクエスト応答定義
#[derive(Debug, Clone)]
pub struct AnswerDef {
    pub handlers: Vec<RequestHandler>,
}

// システムへの反応定義
#[derive(Debug, Clone)]
pub struct ReactDef {
    pub handlers: Vec<EventHandler>,
}

#[derive(Debug, Clone, PartialEq)]
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

// イベントハンドラ
#[derive(Debug, Clone)]
pub struct EventHandler {
    pub event_type: EventType,
    pub parameters: Vec<Parameter>, // イベントの型に応じたパラメータ定義
    pub block: Block,
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

// リクエストハンドラ
#[derive(Debug, Clone)]
pub struct RequestHandler {
    pub request_type: RequestType,
    pub parameters: Vec<Parameter>, // リクエストの型に応じたパラメータ定義
    pub return_type: TypeInfo,
    pub constraints: Option<Constraints>,
    pub block: Block,
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
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_info: TypeInfo,
}

// 制約定義
#[derive(Debug, Clone)]
pub struct Constraints {
    pub strictness: Option<f64>,
    pub stability: Option<f64>,
    pub latency: Option<u32>,
}

// 型情報
#[derive(Debug, Clone)]
pub enum TypeInfo {
    Simple(String), // 基本型 (Int, String等)
    Result {
        ok_type: Box<TypeInfo>,
        err_type: Box<TypeInfo>,
    },
    Option(Box<TypeInfo>),
    Array(Box<TypeInfo>),
    Custom {
        name: String,
        constraints: HashMap<String, Expression>,
    },
}

// コードブロック
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

// 文
#[derive(Debug, Clone)]
pub enum Statement {
    Assignment {
        target: Expression,
        value: Expression,
    },
    Emit {
        event_type: EventType,
        parameters: Vec<Expression>,
        target: Option<String>, // Noneの場合はブロードキャスト
    },
    Request {
        agent: String,
        request_type: RequestType,
        parameters: Vec<Expression>,
        options: Option<RequestOptions>,
    },
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    Return(Expression),
}

use std::time::Duration;

use proc_macro2::TokenStream;

// リクエストオプション
#[derive(Debug, Clone)]
pub struct RequestOptions {
    pub timeout: Option<Duration>,
    pub retry: Option<u32>, // 回数
}

#[derive(Debug, Clone)]
pub struct StateAccessPath(pub Vec<String>); // user.profile.name -> vec!["user", "profile", "name"]

impl StateAccessPath {
    pub fn from_dot_path(path: &str) -> Self {
        Self(path.split('.').map(String::from).collect())
    }
}

// 式
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    StateAccess(StateAccessPath),
    FunctionCall {
        function: String,
        arguments: Vec<Expression>,
    },
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Await(Box<Expression>),
}

// リテラル
#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

// 二項演算子
#[derive(Debug, Clone)]
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

// コード生成用のトレイト
pub trait CodeGen {
    fn generate_rust(&self) -> TokenStream;
}

// パーサートレイト
pub trait Parser {
    fn parse(&self, input: &str) -> Result<MicroAgentDef, ParseError>;
}

// パースエラー
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}
