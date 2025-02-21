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

impl TypeCheckError {
    pub fn with_meta(self, meta: MetaError) -> Self {
        match self {
            Self::InvalidOperatorType {
                operator,
                left_type,
                right_type,
                ..
            } => Self::InvalidOperatorType {
                operator,
                left_type,
                right_type,
                meta,
            },
            _ => self,
        }
    }

    pub fn type_mismatch(expected: TypeInfo, found: TypeInfo, location: Location) -> Self {
        Self::TypeMismatch {
            expected,
            found,
            location,
        }
    }

    pub fn undefined_type(name: String) -> Self {
        Self::UndefinedType(name)
    }

    pub fn undefined_variable(name: String) -> Self {
        Self::UndefinedVariable(name)
    }

    pub fn type_inference_error(message: String) -> Self {
        Self::TypeInferenceError { message }
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

    pub fn with_location(location: Location) -> Self {
        Self {
            location,
            help: None,
            suggestion: None,
        }
    }

    pub fn with_context(location: Location, help: &str) -> Self {
        Self {
            location,
            help: Some(help.to_string()),
            suggestion: None,
        }
    }

    pub fn with_suggestion(location: Location, help: &str, suggestion: &str) -> Self {
        Self {
            location,
            help: Some(help.to_string()),
            suggestion: Some(suggestion.to_string()),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_helpers() {
        let location = Location::default();
        let type_info = TypeInfo::Simple("Int".to_string());

        // Test type mismatch helper
        let error = TypeCheckError::type_mismatch(
            type_info.clone(),
            TypeInfo::Simple("String".to_string()),
            location.clone(),
        );
        assert!(matches!(error, TypeCheckError::TypeMismatch { .. }));

        // Test undefined type helper
        let error = TypeCheckError::undefined_type("MyType".to_string());
        assert!(matches!(error, TypeCheckError::UndefinedType(..)));

        // Test with_meta helper
        let meta = MetaError::with_suggestion(
            location,
            "Invalid types for operation",
            "Use numeric types",
        );
        let error = TypeCheckError::InvalidOperatorType {
            operator: "+".to_string(),
            left_type: type_info.clone(),
            right_type: type_info,
            meta: MetaError::default(),
        }
        .with_meta(meta);
        assert!(matches!(error, TypeCheckError::InvalidOperatorType { .. }));
    }
}
