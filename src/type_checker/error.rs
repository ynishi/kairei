use crate::ast::TypeInfo;
use thiserror::Error;

/// Error type for type checking operations
#[derive(Error, Debug, Clone)]
pub enum TypeCheckError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: TypeInfo,
        found: TypeInfo,
        location: Location,
    },

    #[error("Undefined type: {0}")]
    UndefinedType(String),

    #[error("Invalid type arguments: {0}")]
    InvalidTypeArguments(String),

    #[error("Plugin type error: {message}")]
    PluginTypeError { message: String },

    #[error("Invalid state variable: {message}")]
    InvalidStateVariable { message: String },

    #[error("Invalid handler signature: {message}")]
    InvalidHandlerSignature { message: String },

    #[error("Invalid think block: {message}")]
    InvalidThinkBlock { message: String },

    #[error("Invalid plugin configuration: {message}")]
    InvalidPluginConfig { message: String },

    #[error("Type inference error: {message}")]
    TypeInferenceError { message: String },
}

/// Location information for error reporting
#[derive(Debug, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub file: String,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Result type for type checking operations
pub type TypeCheckResult<T> = Result<T, TypeCheckError>;
