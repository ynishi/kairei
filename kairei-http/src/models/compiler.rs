use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request for validating DSL code
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationRequest {
    /// DSL code to validate
    pub code: String,
}

/// Response for DSL validation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationResponse {
    /// Whether the DSL code is valid
    pub valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Suggestions for fixing errors
    pub suggestions: Option<ValidationSuggestion>,
}

/// Detailed validation error
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Error location
    pub location: ErrorLocation,
    /// Error code
    pub error_code: String,
    /// Suggestion for fixing the error
    pub suggestion: String,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,
    /// Warning location
    pub location: ErrorLocation,
    /// Warning code
    pub warning_code: String,
}

/// Error location information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Context around the error
    pub context: String,
}

/// Suggestions for fixing validation errors
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationSuggestion {
    /// Suggested code with fixes
    pub code: String,
}

/// Request for suggesting fixes for DSL code
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuggestionRequest {
    /// Original DSL code
    pub code: String,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
}

/// Response for suggestion request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuggestionResponse {
    /// Original DSL code
    pub original_code: String,
    /// Fixed DSL code
    pub fixed_code: String,
    /// Explanation of the fixes
    pub explanation: String,
}
