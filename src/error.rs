use thiserror::Error;

use crate::context::ContextError;
use crate::evaluator::EvalError;
use crate::native_feature::types::FeatureError;
use crate::runtime::RuntimeError;
use crate::system::SystemError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("System error: {0}")]
    System(#[from] SystemError),
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    // context
    #[error("Context error: {0}")]
    Context(#[from] ContextError),
    // eval error
    #[error("Eval error: {0}")]
    Eval(#[from] EvalError),
    #[error("Feature error: {0}")]
    Feature(#[from] FeatureError),
    #[error("Agent error: {0}")]
    Agent(#[from] crate::agent_registry::AgentError),
    #[error("AST error: {0}")]
    AST(#[from] crate::ast::ASTError),
    // event error
    #[error("Event error: {0}")]
    Event(#[from] crate::event_bus::EventError),
    // type checking
    #[error("Type check error: {0}")]
    TypeCheck(#[from] crate::type_checker::TypeCheckError),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type InternalResult<T> = Result<T, Error>;

// エラー作成用のヘルパー関数
impl Error {
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Error::Internal(message.into())
    }
}
