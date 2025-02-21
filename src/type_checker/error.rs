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

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Invalid return type: expected {expected}, found {found}")]
    InvalidReturnType {
        expected: TypeInfo,
        found: TypeInfo,
        location: Location,
    },

    #[error("Invalid argument type for function {function}: argument {argument} expected {expected}, found {found}")]
    InvalidArgumentType {
        function: String,
        argument: String,
        expected: TypeInfo,
        found: TypeInfo,
        location: Location,
    },

    #[error("Invalid operator type: operator {operator} cannot be applied to {left_type} and {right_type}")]
    InvalidOperatorType {
        operator: String,
        left_type: TypeInfo,
        right_type: TypeInfo,
        meta: MetaError,
    },
}

#[derive(Error, Debug, Clone)]
pub struct MetaError {
    pub location: Location,
    pub help: Option<String>,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for MetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MetaError at {}: help: {:?}, suggestion: {:?}",
            self.location, self.help, self.suggestion
        )
    }
}

impl MetaError {
    pub fn new(location: Location, help: &str, suggestion: &str) -> Self {
        Self {
            location,
            help: Some(help.to_string()),
            suggestion: Some(suggestion.to_string()),
        }
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
}

/// Location information for error reporting
#[derive(Debug, Clone, Default)]
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
