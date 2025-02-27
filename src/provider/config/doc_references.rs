//! Documentation references for provider configuration validation.
//!
//! This module defines functions for retrieving documentation
//! references for different error types in the provider configuration validation
//! framework.

/// Base URL for provider configuration documentation
pub const DOC_BASE_URL: &str = "https://kairei.dev/docs/provider/config";

/// Documentation reference for schema validation
pub mod schema {
    /// Documentation reference for missing field errors
    pub fn missing_field() -> String {
        format!("{}/schema#missing-field", super::DOC_BASE_URL)
    }

    /// Documentation reference for invalid type errors
    pub fn invalid_type() -> String {
        format!("{}/schema#invalid-type", super::DOC_BASE_URL)
    }

    /// Documentation reference for invalid structure errors
    pub fn invalid_structure() -> String {
        format!("{}/schema#invalid-structure", super::DOC_BASE_URL)
    }

    /// Get documentation reference for schema errors
    pub fn get_doc_reference(error_type: &str) -> Option<String> {
        match error_type {
            "missing_field" => Some(missing_field()),
            "invalid_type" => Some(invalid_type()),
            "invalid_structure" => Some(invalid_structure()),
            _ => None,
        }
    }
}

/// Documentation reference for validation errors
pub mod validation {
    /// Documentation reference for invalid value errors
    pub fn invalid_value() -> String {
        format!("{}/validation#invalid-value", super::DOC_BASE_URL)
    }

    /// Documentation reference for constraint violation errors
    pub fn constraint_violation() -> String {
        format!("{}/validation#constraint-violation", super::DOC_BASE_URL)
    }

    /// Documentation reference for dependency errors
    pub fn dependency_error() -> String {
        format!("{}/validation#dependency-error", super::DOC_BASE_URL)
    }

    /// Get documentation reference for validation errors
    pub fn get_doc_reference(error_type: &str) -> Option<String> {
        match error_type {
            "invalid_value" => Some(invalid_value()),
            "constraint_violation" => Some(constraint_violation()),
            "dependency_error" => Some(dependency_error()),
            _ => None,
        }
    }
}

/// Documentation reference for provider errors
pub mod provider {
    /// Documentation reference for initialization errors
    pub fn initialization() -> String {
        format!("{}/provider#initialization", super::DOC_BASE_URL)
    }

    /// Documentation reference for capability errors
    pub fn capability() -> String {
        format!("{}/provider#capability", super::DOC_BASE_URL)
    }

    /// Documentation reference for configuration errors
    pub fn configuration() -> String {
        format!("{}/provider#configuration", super::DOC_BASE_URL)
    }

    /// Get documentation reference for provider errors
    pub fn get_doc_reference(error_type: &str) -> Option<String> {
        match error_type {
            "initialization" => Some(initialization()),
            "capability" => Some(capability()),
            "configuration" => Some(configuration()),
            _ => None,
        }
    }
}
