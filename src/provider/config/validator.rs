//! Provider configuration validation framework.
//!
//! This module defines the validation framework for provider configurations,
//! including the ProviderConfigValidator trait and its implementations.

use crate::provider::config::errors::ProviderConfigError;
use std::collections::HashMap;

/// Trait for validating provider configurations.
///
/// This trait defines methods for validating different aspects of provider
/// configurations, including schema validation, provider-specific validation,
/// capability validation, and dependency validation.
pub trait ProviderConfigValidator {
    /// Validates the schema of a provider configuration.
    ///
    /// This method checks that the configuration has the correct structure,
    /// including required fields and field types.
    #[allow(clippy::result_large_err)]
    fn validate_schema(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError>;

    /// Validates provider-specific aspects of a configuration.
    ///
    /// This method checks that the configuration is valid for the specific
    /// provider it is intended for.
    #[allow(clippy::result_large_err)]
    fn validate_provider_specific(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError>;

    /// Validates that the configuration is compatible with the required capabilities.
    ///
    /// This method checks that the configuration supports the capabilities
    /// required by the system.
    #[allow(clippy::result_large_err)]
    fn validate_capabilities(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError>;

    /// Validates that the configuration's dependencies are satisfied.
    ///
    /// This method checks that any dependencies required by the configuration
    /// are available and compatible.
    #[allow(clippy::result_large_err)]
    fn validate_dependencies(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError>;

    /// Validates the schema of the configuration and returns warnings.
    ///
    /// This method checks for non-critical issues in the schema structure
    /// and returns warnings instead of errors.
    fn validate_schema_warnings(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        Vec::new()
    }

    /// Validates provider-specific configuration and returns warnings.
    ///
    /// This method checks for non-critical issues in the provider-specific
    /// configuration and returns warnings instead of errors.
    fn validate_provider_specific_warnings(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        Vec::new()
    }

    /// Validates provider capabilities and returns warnings.
    ///
    /// This method checks for non-critical issues in the capability
    /// configuration and returns warnings instead of errors.
    fn validate_capabilities_warnings(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        Vec::new()
    }

    /// Validates provider dependencies and returns warnings.
    ///
    /// This method checks for non-critical issues in the dependency
    /// configuration and returns warnings instead of errors.
    fn validate_dependencies_warnings(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        Vec::new()
    }

    /// Validates a provider configuration.
    ///
    /// This method combines all validation methods to perform a complete
    /// validation of the configuration.
    #[allow(clippy::result_large_err)]
    fn validate(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError> {
        self.validate_schema(config)?;
        self.validate_provider_specific(config)?;
        self.validate_capabilities(config)?;
        self.validate_dependencies(config)?;
        Ok(())
    }
}

/// Collects and aggregates errors during validation.
///
/// This struct allows validators to collect multiple errors during validation
/// rather than stopping at the first error.
#[derive(Debug, Default, Clone)]
pub struct ErrorCollector {
    /// Errors collected during validation.
    pub errors: Vec<ProviderConfigError>,
    /// Warnings collected during validation.
    pub warnings: Vec<ProviderConfigError>,
}

impl ErrorCollector {
    /// Creates a new error collector.
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Adds an error to the collector.
    pub fn add_error(&mut self, error: ProviderConfigError) {
        self.errors.push(error);
    }

    /// Adds a warning to the collector.
    pub fn add_warning(&mut self, warning: ProviderConfigError) {
        self.warnings.push(warning);
    }

    /// Returns the result of the validation.
    ///
    /// If there are any errors, returns an error containing all collected errors.
    /// Otherwise, returns Ok(()).
    #[allow(clippy::result_large_err)]
    pub fn result(&self) -> Result<(), ProviderConfigError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            // For now, just return the first error
            // In the future, we could aggregate errors into a single error
            Err(self.errors[0].clone())
        }
    }

    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Trait for validators that collect errors during validation.
///
/// This trait extends the ProviderConfigValidator trait to support collecting
/// multiple errors during validation rather than stopping at the first error.
pub trait CollectingValidator: ProviderConfigValidator {
    /// Validates a provider configuration and collects errors.
    ///
    /// This method combines all validation methods to perform a complete
    /// validation of the configuration, collecting all errors rather than
    /// stopping at the first error.
    fn validate_collecting(&self, config: &HashMap<String, serde_json::Value>) -> ErrorCollector {
        let mut collector = ErrorCollector::new();

        // Validate schema
        if let Err(error) = self.validate_schema(config) {
            collector.add_error(error);
        }

        // Validate provider-specific
        if let Err(error) = self.validate_provider_specific(config) {
            collector.add_error(error);
        }

        // Validate capabilities
        if let Err(error) = self.validate_capabilities(config) {
            collector.add_error(error);
        }

        // Validate dependencies
        if let Err(error) = self.validate_dependencies(config) {
            collector.add_error(error);
        }

        // Collect warnings
        for warning in self.validate_schema_warnings(config) {
            collector.add_warning(warning);
        }

        for warning in self.validate_provider_specific_warnings(config) {
            collector.add_warning(warning);
        }

        for warning in self.validate_capabilities_warnings(config) {
            collector.add_warning(warning);
        }

        for warning in self.validate_dependencies_warnings(config) {
            collector.add_warning(warning);
        }

        collector
    }
}

// Implement CollectingValidator for all types that implement ProviderConfigValidator
impl<T: ProviderConfigValidator> CollectingValidator for T {}
