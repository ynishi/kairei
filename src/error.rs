use nom::error::ErrorKind;
use thiserror::Error;

// トップレベルのエラー型
// ```
// fn compile_agent(source: &str) -> KaireiResult<Agent> {
//     let ast = parse_source(source).map_err(CompileError::from)?;
//     let agent = validate_ast(ast).map_err(CompileError::from)?;
//     Ok(agent)
// }
//
// fn run_agent(agent: Agent) -> RuntimeResult<()> {
//     agent.initialize().map_err(AgentError::from)?;
//     agent.run().map_err(RuntimeError::from)?;
//     Ok(())
// }
// ```
#[derive(Error, Debug)]
pub enum KaireiError {
    #[error("Compilation error: {0}")]
    Compile(#[from] CompileError),

    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),

    #[error("Internal error: {0}")]
    Internal(String),
}

// コンパイル時のエラー
#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Parse error: {message} at line {line}, column {column}, kind: {kind:?}")]
    Parse {
        message: String,
        line: usize,
        column: usize,
        kind: ErrorKind,
    },

    #[error("Type error: {0}")]
    Type(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

// ランタイムのエラー
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),

    #[error("Event error: {0}")]
    Event(#[from] EventError),

    #[error("State error: {0}")]
    State(#[from] StateError),

    #[error("Handler error: {0}")]
    Handler(#[from] HandlerError),

    #[error("Execution error: {0}")]
    Execution(#[from] ExecutionError),
}

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    NotFound(String),

    #[error("Agent initialization failed: {0}")]
    InitializationFailed(String),
}

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Event type not supported: {event_type}")]
    UnsupportedType { event_type: String },

    #[error("Invalid event parameters: {message}")]
    InvalidParameters { message: String },

    #[error("Event Send failed: {message}")]
    SendFailed { message: String },
}

#[derive(Error, Debug)]
pub enum StateError {
    #[error("State variable not found: {key}")]
    NotFound { key: String },

    #[error("Invalid state value for {key}: {message}")]
    InvalidValue { key: String, message: String },
}

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Handler not found for {handler_type}: {name}")]
    NotFound { handler_type: String, name: String },

    #[error("Handler execution failed: {0}")]
    ExecutionFailed(String),
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Receiver not found: {receiver}")]
    ReceiverNotFound { receiver: String },

    #[error("Expression evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Async operation failed: {0}")]
    AsyncFailed(String),
}

// 便利な Result 型エイリアス
pub type KaireiResult<T> = Result<T, KaireiError>;
pub type CompileResult<T> = Result<T, CompileError>;
pub type RuntimeResult<T> = Result<T, RuntimeError>;

// エラー作成用のヘルパー関数
impl KaireiError {
    pub fn internal<S: Into<String>>(message: S) -> Self {
        KaireiError::Internal(message.into())
    }
}
