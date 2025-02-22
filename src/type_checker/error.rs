use crate::ast::TypeInfo;
use thiserror::Error;

const DEFAULT_HELP: &str = "No help available";
const DEFAULT_SUGGESTION: &str = "No suggestion available";

/// Error type for type checking operations
#[derive(Error, Debug, Clone)]
pub enum TypeCheckError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: TypeInfo,
        found: TypeInfo,
        meta: TypeCheckErrorMeta,
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
        meta: TypeCheckErrorMeta,
    },
}

#[derive(Error, Debug, Clone)]
pub struct TypeCheckErrorMeta {
    pub location: Location,
    pub help: String,
    pub suggestion: String,
}

impl std::fmt::Display for TypeCheckErrorMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TypeCheckErrorMeta at {}: help: {}, suggestion: {}",
            self.location, self.help, self.suggestion
        )
    }
}

impl Default for TypeCheckErrorMeta {
    fn default() -> Self {
        Self {
            location: Location::default(),
            help: DEFAULT_HELP.to_string(),
            suggestion: DEFAULT_SUGGESTION.to_string(),
        }
    }
}

impl TypeCheckError {
    pub fn with_meta(self, meta: TypeCheckErrorMeta) -> Self {
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
            Self::TypeMismatch {
                expected, found, ..
            } => Self::TypeMismatch {
                expected,
                found,
                meta,
            },
            _ => self,
        }
    }

    pub fn type_mismatch(expected: TypeInfo, found: TypeInfo, location: Location) -> Self {
        Self::TypeMismatch {
            expected,
            found,
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help("Type mismatch in expression")
                .with_suggestion("Make sure the types match the expected types"),
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

impl TypeCheckErrorMeta {
    pub fn new(location: Location, help: &str, suggestion: &str) -> Self {
        Self {
            location,
            help: help.to_string(),
            suggestion: suggestion.to_string(),
        }
    }

    // builder pattern
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
        self
    }

    pub fn with_help(mut self, help: &str) -> Self {
        self.help = help.to_string();
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = suggestion.to_string();
        self
    }

    pub fn context(location: Location, help: &str) -> Self {
        Self {
            location,
            help: help.to_string(),
            suggestion: DEFAULT_SUGGESTION.to_string(),
        }
    }
}

/// Location information for error reporting
#[derive(Debug, Clone, Default, PartialEq)]
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
        let meta = TypeCheckErrorMeta::context(location.clone(), "Invalid types for operation")
            .with_suggestion("Use numeric types");
        let error = TypeCheckError::InvalidOperatorType {
            operator: "+".to_string(),
            left_type: type_info.clone(),
            right_type: type_info,
            meta: TypeCheckErrorMeta::default().with_location(location.clone()),
        }
        .with_meta(meta);
        assert!(matches!(error, TypeCheckError::InvalidOperatorType { .. }));
    }

    #[test]
    fn test_builder_pattern() {
        let location = Location::default();

        // Test chaining with defaults
        let meta = TypeCheckErrorMeta::default()
            .with_help("Custom help")
            .with_suggestion("Custom suggestion");
        assert_eq!(meta.help, "Custom help");
        assert_eq!(meta.suggestion, "Custom suggestion");
        assert_eq!(meta.location, Location::default());

        // Test context constructor with custom location
        let meta = TypeCheckErrorMeta::context(
            Location {
                line: 1,
                column: 2,
                file: "test.rs".to_string(),
            },
            "Test help",
        );
        assert_eq!(meta.help, "Test help");
        assert_eq!(meta.suggestion, DEFAULT_SUGGESTION);
        assert_eq!(meta.location.line, 1);
        assert_eq!(meta.location.column, 2);
        assert_eq!(meta.location.file, "test.rs");

        // Test full builder chain
        let meta = TypeCheckErrorMeta::default()
            .with_location(location.clone())
            .with_help("Help message")
            .with_suggestion("Suggestion message");
        assert_eq!(meta.help, "Help message");
        assert_eq!(meta.suggestion, "Suggestion message");
        assert_eq!(meta.location, location);
    }

    #[test]
    fn test_error_formatting() {
        let location = Location {
            line: 10,
            column: 20,
            file: "main.rs".to_string(),
        };
        let meta = TypeCheckErrorMeta::new(location, "Test help", "Test suggestion");
        assert_eq!(
            meta.to_string(),
            "TypeCheckErrorMeta at main.rs:10:20: help: Test help, suggestion: Test suggestion"
        );
    }

    #[test]
    fn test_type_mismatch_with_meta() {
        let location = Location::default();
        let meta = TypeCheckErrorMeta::context(location.clone(), "Test help")
            .with_suggestion("Test suggestion");

        let error = TypeCheckError::type_mismatch(
            TypeInfo::Simple("Int".to_string()),
            TypeInfo::Simple("String".to_string()),
            location,
        )
        .with_meta(meta.clone());

        if let TypeCheckError::TypeMismatch {
            meta: error_meta, ..
        } = error
        {
            assert_eq!(error_meta.help, "Test help");
            assert_eq!(error_meta.suggestion, "Test suggestion");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }
}
