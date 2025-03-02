//! Documentation references for provider configuration validation.
//!
//! This module defines functions for retrieving documentation
//! references for different error types in the provider configuration validation
//! framework.

/// Base URL for provider configuration documentation
pub const DOC_BASE_URL: &str =
    "https://github.com/ynishi/kairei/blob/main/docs/reference/provider_config_validation.md";

/// Section anchors for provider configuration documentation
pub mod anchors {
    /// Schema validation section
    pub const SCHEMA: &str = "#1-provider-configuration-overview";

    /// Validation process section
    pub const VALIDATION: &str = "#2-validation-process";

    /// Error handling section
    pub const ERROR_HANDLING: &str = "#3-error-handling-guide";

    /// Common scenarios section
    pub const COMMON_SCENARIOS: &str = "#4-common-validation-scenarios";

    /// Troubleshooting section
    pub const TROUBLESHOOTING: &str = "#5-troubleshooting-guide";
}

/// Documentation reference for schema validation
pub mod schema {
    /// Documentation reference for missing field errors
    pub fn missing_field() -> String {
        format!("{}{}", super::DOC_BASE_URL, super::anchors::SCHEMA)
    }

    /// Documentation reference for invalid type errors
    pub fn invalid_type() -> String {
        format!("{}{}", super::DOC_BASE_URL, super::anchors::SCHEMA)
    }

    /// Documentation reference for invalid structure errors
    pub fn invalid_structure() -> String {
        format!("{}{}", super::DOC_BASE_URL, super::anchors::SCHEMA)
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
        format!("{}{}", super::DOC_BASE_URL, super::anchors::ERROR_HANDLING)
    }

    /// Documentation reference for constraint violation errors
    pub fn constraint_violation() -> String {
        format!("{}{}", super::DOC_BASE_URL, super::anchors::ERROR_HANDLING)
    }

    /// Documentation reference for dependency errors
    pub fn dependency_error() -> String {
        format!("{}{}", super::DOC_BASE_URL, super::anchors::ERROR_HANDLING)
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
        format!("{}{}", super::DOC_BASE_URL, super::anchors::TROUBLESHOOTING)
    }

    /// Documentation reference for capability errors
    pub fn capability() -> String {
        format!("{}{}", super::DOC_BASE_URL, super::anchors::TROUBLESHOOTING)
    }

    /// Documentation reference for configuration errors
    pub fn configuration() -> String {
        format!("{}{}", super::DOC_BASE_URL, super::anchors::TROUBLESHOOTING)
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
