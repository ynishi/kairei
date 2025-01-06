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

    #[error("Unsupported request event: {request_type}")]
    UnsupportedRequest { request_type: String },

    #[error("Invalid event parameters: {message}")]
    InvalidParameters { message: String },

    #[error("Event parameters length not matched: {event_type}, expected {expected}, got {got}")]
    ParametersLengthNotMatched {
        event_type: String,
        expected: usize,
        got: usize,
    },

    #[error("Event parameter type mismatch: {event_type}, expected {expected}, got {got}")]
    TypeMismatch {
        event_type: String,
        expected: String,
        got: String,
    },

    #[error("Event Send failed: {message}")]
    SendFailed { message: String },

    #[error("Event Recieve failed: {message}")]
    RecieveFailed { message: String },

    #[error("Event Recieve response failed: {message}")]
    RecieveResponseFailed { request_id: String, message: String },

    #[error("Event Recieve response timeout: {request_id}")]
    RecieveResponseTimeout {
        request_id: String,
        timeout_secs: u64,
        message: String,
    },

    #[error("Event lagged: {count}")]
    Lagged { count: u64 },

    #[error("Event already registered: {event_type}")]
    AlreadyRegistered { event_type: String },

    #[error("Built-in event already registered")]
    BuiltInAlreadyRegistered,

    #[error("Event not found: {0}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum StateError {
    #[error("State variable not found: {key}")]
    NotFound { key: String },

    #[error("Invalid state value for {key}: {message}")]
    InvalidValue { key: String, message: String },

    #[error("State access error: {0}")]
    AccessError(String),
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
    #[error("Agent already exists failed: {id}")]
    AgentAlreadyExists { id: String },

    // agnt not found
    #[error("Agent not found: {id}")]
    AgentNotFound { id: String },

    #[error("Receiver not found: {receiver}")]
    ReceiverNotFound { receiver: String },

    #[error("Expression evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Async operation failed: {0}")]
    AsyncFailed(String),

    #[error("Request timeout: {request_id}")]
    RequestTimeout { request_id: String },

    #[error("Invalid Pending Request: {request_id}")]
    InvalidPendingRequest { request_id: String },

    #[error("Send Shutdown failed: {message}")]
    SendShutdownFailed { agent_name: String, message: String },

    #[error("Shutdown failed: {agent_name}, {message}")]
    ShutdownFailed { agent_name: String, message: String },

    #[error("Shutdown timeout: {agent_id}, {timeout_secs} secs,")]
    ShutdownTimeout { agent_id: String, timeout_secs: u64 },

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Scaling error: {0}")]
    ScalingError(String),
    #[error("Scaling not enough agents: {base_name}, request: {required}, actual: {current}")]
    ScalingNotEnoughAgents {
        base_name: String,
        required: usize,
        current: usize,
    },
    #[error("Event error: {0}")]
    EventError(String),
    #[error("AST error: {0}")]
    ASTError(String),
    #[error("AST not found: {0}")]
    ASTNotFound(String),
    #[error("System error: {0}")]
    SystemError(String),
    #[error("Clean up failed: {agent_name}, {message}")]
    CleanUpFailed { agent_name: String, message: String },

    // eval error
    #[error("Eval error: {0}")]
    EvalError(String),

    #[error("Eval error: {0}")]
    NativeFeatureError(String),
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
