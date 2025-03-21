use crate::ast::TypeInfo;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct InvalidArgumentTypeData {
    pub function: String,
    pub argument: String,
    pub expected: TypeInfo,
    pub found: TypeInfo,
    pub meta: TypeCheckErrorMeta,
}

const DEFAULT_HELP: &str = "No help available";
const DEFAULT_SUGGESTION: &str = "No suggestion available";

/// Error types for type checking
/// Plugin errors are handled through the visitor pattern using general type error variants
#[derive(Error, Debug, Clone)]
pub enum TypeCheckError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: TypeInfo,
        found: TypeInfo,
        meta: TypeCheckErrorMeta,
    },

    #[error("Undefined type: {name}")]
    UndefinedType {
        name: String,
        meta: TypeCheckErrorMeta,
    },

    #[error("Invalid type arguments: {message}")]
    InvalidTypeArguments {
        message: String,
        meta: TypeCheckErrorMeta,
    },

    #[error("Invalid state variable: {message}")]
    InvalidStateVariable {
        message: String,
        meta: TypeCheckErrorMeta,
    },

    #[error("Invalid handler signature: {message}")]
    InvalidHandlerSignature {
        message: String,
        meta: TypeCheckErrorMeta,
    },

    #[error("Invalid think block: {message}")]
    InvalidThinkBlock {
        message: String,
        meta: TypeCheckErrorMeta,
    },

    #[error("Type inference error: {message}")]
    TypeInferenceError {
        message: String,
        meta: TypeCheckErrorMeta,
    },

    #[error("Undefined variable: {name}")]
    UndefinedVariable {
        name: String,
        meta: TypeCheckErrorMeta,
    },

    #[error("Undefined function: {name}")]
    UndefinedFunction {
        name: String,
        meta: TypeCheckErrorMeta,
    },

    #[error("Invalid return type: expected {expected}, found {found}")]
    InvalidReturnType {
        expected: TypeInfo,
        found: TypeInfo,
        meta: TypeCheckErrorMeta,
    },

    #[error("Invalid argument type for function {}: argument {} expected {}, found {}", .0.function, .0.argument, .0.expected, .0.found)]
    InvalidArgumentType(Box<InvalidArgumentTypeData>),

    #[error(
        "Invalid operator type: operator {operator} cannot be applied to {left_type} and {right_type}"
    )]
    InvalidOperatorType {
        operator: String,
        left_type: TypeInfo,
        right_type: TypeInfo,
        meta: TypeCheckErrorMeta,
    },

    #[error("Invalid will action: {message}")]
    InvalidWillActionError {
        message: String,
        meta: TypeCheckErrorMeta,
    },

    #[error("Will action parameter error: {message}")]
    WillActionParameterError {
        message: String,
        meta: TypeCheckErrorMeta,
    },

    #[error("Invalid sistence context: {message}")]
    InvalidSistenceContextError {
        message: String,
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
            Self::InvalidTypeArguments { message, .. } => {
                Self::InvalidTypeArguments { message, meta }
            }
            Self::InvalidThinkBlock { message, .. } => Self::InvalidThinkBlock { message, meta },
            Self::InvalidHandlerSignature { message, .. } => {
                Self::InvalidHandlerSignature { message, meta }
            }
            Self::InvalidStateVariable { message, .. } => {
                Self::InvalidStateVariable { message, meta }
            }
            Self::UndefinedFunction { name, .. } => Self::UndefinedFunction { name, meta },
            Self::InvalidWillActionError { message, .. } => {
                Self::InvalidWillActionError { message, meta }
            }
            Self::WillActionParameterError { message, .. } => {
                Self::WillActionParameterError { message, meta }
            }
            Self::InvalidSistenceContextError { message, .. } => {
                Self::InvalidSistenceContextError { message, meta }
            }
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

    pub fn undefined_type(name: String, location: Location) -> Self {
        Self::UndefinedType {
            name: name.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help(&format!(
                    "Type '{}' is not defined in the current scope",
                    name
                ))
                .with_suggestion(
                    "Check type name for typos or ensure the type is imported/defined",
                ),
        }
    }

    pub fn undefined_variable(name: String, location: Location) -> Self {
        Self::UndefinedVariable {
            name: name.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help(&format!(
                    "Variable '{}' is not defined in the current scope",
                    name
                ))
                .with_suggestion(
                    "Check variable name for typos or ensure it is declared before use",
                ),
        }
    }

    pub fn invalid_state_variable(message: String, location: Location) -> Self {
        Self::InvalidStateVariable {
            message: message.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help("Invalid state variable declaration or usage")
                .with_suggestion("Check that the state variable is properly declared and used within a valid scope"),
        }
    }

    pub fn type_inference_error(message: String, location: Location) -> Self {
        Self::TypeInferenceError {
            message,
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help("Error during type inference")
                .with_suggestion("Check that all types can be inferred from context"),
        }
    }

    pub fn invalid_think_block(message: String, location: Location) -> Self {
        Self::InvalidThinkBlock {
            message: message.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help("Think block contains invalid expressions or types")
                .with_suggestion("Check that the think block follows the expected format and contains valid expressions"),
        }
    }

    pub fn validation_error(message: &str) -> Self {
        Self::InvalidThinkBlock {
            message: message.to_string(),
            meta: TypeCheckErrorMeta::default()
                .with_help("Provider configuration validation error")
                .with_suggestion(
                    "Check that the provider configuration has valid properties and values",
                ),
        }
    }

    pub fn invalid_handler_signature(message: String, location: Location) -> Self {
        Self::InvalidHandlerSignature {
            message: message.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help("Handler signature does not match the expected format")
                .with_suggestion(
                    "Check handler parameter types and return type match the event definition",
                ),
        }
    }

    pub fn invalid_type_arguments(message: String, location: Location) -> Self {
        Self::InvalidTypeArguments {
            message: message.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help("Invalid arguments provided for type")
                .with_suggestion("Check the type arguments match the expected types and arity"),
        }
    }

    pub fn undefined_function(name: String, location: Location) -> Self {
        Self::UndefinedFunction {
            name: name.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help(&format!(
                    "Function '{}' is not defined in the current scope",
                    name
                ))
                .with_suggestion("Check function name for typos or ensure it is imported/defined"),
        }
    }

    pub fn invalid_return_type(expected: TypeInfo, found: TypeInfo, location: Location) -> Self {
        Self::InvalidReturnType {
            expected: expected.clone(),
            found: found.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help(&format!(
                    "Return type mismatch: expected {}, found {}",
                    expected, found
                ))
                .with_suggestion("Ensure the function returns a value of the expected type"),
        }
    }

    pub fn invalid_argument_type(
        function: String,
        argument: String,
        expected: TypeInfo,
        found: TypeInfo,
        location: Location,
    ) -> Self {
        Self::InvalidArgumentType(Box::new(InvalidArgumentTypeData {
            function: function.clone(),
            argument: argument.clone(),
            expected: expected.clone(),
            found: found.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help(&format!(
                    "Invalid argument type for function '{}': argument '{}' has wrong type",
                    function, argument
                ))
                .with_suggestion(&format!(
                    "Provide an argument of type {} instead of {}",
                    expected, found
                )),
        }))
    }

    pub fn invalid_will_action(message: String, location: Location) -> Self {
        Self::InvalidWillActionError {
            message: message.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help("Invalid will action definition or usage")
                .with_suggestion("Check that the will action follows the expected format and contains valid parameters"),
        }
    }

    pub fn will_action_parameter_error(message: String, location: Location) -> Self {
        Self::WillActionParameterError {
            message: message.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help("Will action parameter error")
                .with_suggestion("Check that the parameters provided to the will action are valid and of the correct types"),
        }
    }

    pub fn invalid_sistence_context(message: String, location: Location) -> Self {
        Self::InvalidSistenceContextError {
            message: message.clone(),
            meta: TypeCheckErrorMeta::default()
                .with_location(location)
                .with_help("Invalid sistence agent context")
                .with_suggestion("Check that the sistence agent is properly defined and used within a valid scope"),
        }
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
        let error = TypeCheckError::undefined_type("MyType".to_string(), location.clone());
        assert!(matches!(error, TypeCheckError::UndefinedType { .. }));

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

    #[test]
    fn test_type_inference_error_with_meta() {
        let location = Location {
            line: 10,
            column: 20,
            file: "test.rs".to_string(),
        };
        let error =
            TypeCheckError::type_inference_error("Cannot infer type".to_string(), location.clone());

        if let TypeCheckError::TypeInferenceError { meta, .. } = error {
            assert_eq!(meta.location, location);
            assert_eq!(meta.help, "Error during type inference");
            assert_eq!(
                meta.suggestion,
                "Check that all types can be inferred from context"
            );
        } else {
            panic!("Expected TypeInferenceError");
        }
    }
}
