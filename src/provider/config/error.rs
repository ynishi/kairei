use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Schema validation error: {message}")]
    Schema {
        message: String,
        location: SourceLocation,
        context: ValidationContext,
    },

    #[error("Provider-specific validation error: {message}")]
    ProviderSpecific {
        message: String,
        location: SourceLocation,
        context: ValidationContext,
    },

    #[error("Capability validation error: {message}")]
    Capability {
        message: String,
        location: SourceLocation,
        context: ValidationContext,
    },

    #[error("Dependency validation error: {message}")]
    Dependency {
        message: String,
        location: SourceLocation,
        context: ValidationContext,
    },
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub provider_type: String,
    pub validation_phase: ValidationPhase,
    pub suggestion: Option<String>,
    pub doc_reference: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ValidationPhase {
    TypeCheck,
    Runtime,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}
