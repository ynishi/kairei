//! Provider configuration validation framework.
//!
//! This module defines the validation framework for provider configurations,
//! including the `ProviderConfigValidator` trait and its implementations.
//!
//! # Overview
//!
//! Provider configuration validation is a critical part of the KAIREI system that ensures
//! provider configurations are correct, complete, and compatible with the system requirements.
//! The validation process occurs in multiple stages:
//!
//! 1. **Schema Validation**: Ensures the configuration has the correct structure, including
//!    required fields and field types.
//! 2. **Provider-Specific Validation**: Validates aspects specific to the provider type.
//! 3. **Capability Validation**: Ensures the configuration supports required capabilities.
//! 4. **Dependency Validation**: Verifies that dependencies are available and compatible.
//!
//! # Error Handling
//!
//! The validation framework supports two approaches to error handling:
//!
//! - **Early Return**: The `validate` method returns on the first error encountered.
//! - **Error Collection**: The `validate_collecting` method collects all errors and warnings.
//!
//! Errors are represented by the `ProviderConfigError` type, which includes detailed
//! information about the error, including location, error type, message, and suggestions.
//!
//! # Usage
//!
//! The validation framework is typically used during provider initialization and
//! configuration updates to ensure the configuration is valid before it is used.

use crate::event::event_bus::EventBus;
use crate::provider::config::errors::ProviderConfigError;
use crate::provider::config::events::ProviderErrorEvent;
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for validating provider configurations.
///
/// This trait defines methods for validating different aspects of provider
/// configurations, including schema validation, provider-specific validation,
/// capability validation, and dependency validation.
///
/// # Validation Process
///
/// The validation process is divided into four main stages:
///
/// 1. **Schema Validation**: Validates the basic structure of the configuration,
///    including required fields and field types.
/// 2. **Provider-Specific Validation**: Validates aspects specific to the provider type,
///    such as configuration values and constraints.
/// 3. **Capability Validation**: Ensures the configuration supports the required capabilities.
/// 4. **Dependency Validation**: Verifies that dependencies are available and compatible.
///
/// Each stage can be performed independently or combined using the `validate` method.
///
/// # Error Handling
///
/// Validation methods return a `Result<(), ProviderConfigError>` where:
///
/// - `Ok(())` indicates successful validation.
/// - `Err(ProviderConfigError)` indicates a validation error.
///
/// The `ProviderConfigError` type includes detailed information about the error,
/// including location, error type, message, and suggestions for fixing the issue.
///
/// # Examples
///
/// ```rust,ignore
/// use std::collections::HashMap;
/// use serde_json::json;
/// use kairei_core::provider::config::validator::{ProviderConfigValidator, CollectingValidator};
/// use kairei_core::provider::config::validators::type_checker::TypeCheckerValidator;
///
/// // Create a validator
/// let validator = TypeCheckerValidator::new();
///
/// // Create a configuration
/// let mut config = HashMap::new();
/// config.insert("type".to_string(), json!("memory"));
/// config.insert("ttl".to_string(), json!(3600));
///
/// // Validate the configuration
/// match validator.validate(&config) {
///     Ok(()) => println!("Configuration is valid"),
///     Err(error) => println!("Validation error: {}", error),
/// }
///
/// // Collect all errors and warnings
/// let collector = validator.validate_collecting(&config);
/// if collector.has_errors() {
///     println!("Validation errors found");
/// }
/// if collector.has_warnings() {
///     println!("Validation warnings found");
/// }
/// ```
pub trait ProviderConfigValidator {
    /// Validates the schema of a provider configuration.
    ///
    /// This method checks that the configuration has the correct structure,
    /// including required fields and field types. Schema validation typically includes:
    ///
    /// - Verifying that required fields are present
    /// - Checking that field types match expected types
    /// - Validating field value ranges and constraints
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the schema is valid
    /// * `Err(ProviderConfigError)` if the schema is invalid, with details about the error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = TypeCheckerValidator::new();
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("memory"));
    ///
    /// // Validate schema only
    /// match validator.validate_schema(&config) {
    ///     Ok(()) => println!("Schema is valid"),
    ///     Err(error) => println!("Schema error: {}", error),
    /// }
    /// ```
    #[allow(clippy::result_large_err)]
    fn validate_schema(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError>;

    /// Validates provider-specific aspects of a configuration.
    ///
    /// This method checks that the configuration is valid for the specific
    /// provider it is intended for. Provider-specific validation typically includes:
    ///
    /// - Validating provider-specific field values and constraints
    /// - Checking for compatibility between configuration options
    /// - Validating provider-specific business rules
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the provider-specific configuration is valid
    /// * `Err(ProviderConfigError)` if the configuration is invalid, with details about the error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = EvaluatorValidator::new();
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("rag"));
    /// config.insert("chunk_size".to_string(), json!(512));
    /// config.insert("max_tokens".to_string(), json!(1000));
    ///
    /// // Validate provider-specific configuration
    /// match validator.validate_provider_specific(&config) {
    ///     Ok(()) => println!("Provider-specific configuration is valid"),
    ///     Err(error) => println!("Provider-specific error: {}", error),
    /// }
    /// ```
    #[allow(clippy::result_large_err)]
    fn validate_provider_specific(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError>;

    /// Validates that the configuration is compatible with the required capabilities.
    ///
    /// This method checks that the configuration supports the capabilities
    /// required by the system. Capability validation typically includes:
    ///
    /// - Verifying that the provider supports required capabilities
    /// - Checking capability-specific configuration options
    /// - Validating capability constraints and limitations
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the capability configuration is valid
    /// * `Err(ProviderConfigError)` if the configuration is invalid, with details about the error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = EvaluatorValidator::new();
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("search"));
    /// config.insert("max_results".to_string(), json!(10));
    ///
    /// // Validate capability configuration
    /// match validator.validate_capabilities(&config) {
    ///     Ok(()) => println!("Capability configuration is valid"),
    ///     Err(error) => println!("Capability error: {}", error),
    /// }
    /// ```
    #[allow(clippy::result_large_err)]
    fn validate_capabilities(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError>;

    /// Validates that the configuration's dependencies are satisfied.
    ///
    /// This method checks that any dependencies required by the configuration
    /// are available and compatible. Dependency validation typically includes:
    ///
    /// - Verifying that required dependencies are available
    /// - Checking dependency version compatibility
    /// - Validating dependency configuration compatibility
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the dependency configuration is valid
    /// * `Err(ProviderConfigError)` if the configuration is invalid, with details about the error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = EvaluatorValidator::new();
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("rag"));
    /// config.insert("dependencies".to_string(), json!([
    ///     {"type": "memory", "ttl": 3600}
    /// ]));
    ///
    /// // Validate dependency configuration
    /// match validator.validate_dependencies(&config) {
    ///     Ok(()) => println!("Dependency configuration is valid"),
    ///     Err(error) => println!("Dependency error: {}", error),
    /// }
    /// ```
    #[allow(clippy::result_large_err)]
    fn validate_dependencies(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ProviderConfigError>;

    /// Validates the schema of the configuration and returns warnings.
    ///
    /// This method checks for non-critical issues in the schema structure
    /// and returns warnings instead of errors. Schema warnings typically include:
    ///
    /// - Deprecated field usage
    /// - Suboptimal configuration choices
    /// - Performance recommendations
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// A Vec of `ProviderConfigError` objects representing warnings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = TypeCheckerValidator::new();
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("memory"));
    /// config.insert("legacy_mode".to_string(), json!(true)); // Deprecated field
    ///
    /// // Get schema warnings
    /// let warnings = validator.validate_schema_warnings(&config);
    /// for warning in warnings {
    ///     println!("Warning: {}", warning);
    /// }
    /// ```
    fn validate_schema_warnings(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        Vec::new()
    }

    /// Validates provider-specific configuration and returns warnings.
    ///
    /// This method checks for non-critical issues in the provider-specific
    /// configuration and returns warnings instead of errors. Provider-specific warnings typically include:
    ///
    /// - Deprecated provider-specific features
    /// - Suboptimal provider-specific configuration choices
    /// - Provider-specific performance recommendations
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// A Vec of `ProviderConfigError` objects representing warnings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = EvaluatorValidator::new();
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("rag"));
    /// config.insert("use_legacy_chunking".to_string(), json!(true)); // Deprecated feature
    ///
    /// // Get provider-specific warnings
    /// let warnings = validator.validate_provider_specific_warnings(&config);
    /// for warning in warnings {
    ///     println!("Warning: {}", warning);
    /// }
    /// ```
    fn validate_provider_specific_warnings(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        Vec::new()
    }

    /// Validates provider capabilities and returns warnings.
    ///
    /// This method checks for non-critical issues in the capability
    /// configuration and returns warnings instead of errors. Capability warnings typically include:
    ///
    /// - Deprecated capability features
    /// - Suboptimal capability configuration choices
    /// - Capability performance recommendations
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// A Vec of `ProviderConfigError` objects representing warnings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = EvaluatorValidator::new();
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("search"));
    /// config.insert("use_fuzzy".to_string(), json!(true)); // Deprecated feature
    ///
    /// // Get capability warnings
    /// let warnings = validator.validate_capabilities_warnings(&config);
    /// for warning in warnings {
    ///     println!("Warning: {}", warning);
    /// }
    /// ```
    fn validate_capabilities_warnings(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        Vec::new()
    }

    /// Validates provider dependencies and returns warnings.
    ///
    /// This method checks for non-critical issues in the dependency
    /// configuration and returns warnings instead of errors. Dependency warnings typically include:
    ///
    /// - Deprecated dependency features
    /// - Suboptimal dependency configuration choices
    /// - Dependency performance recommendations
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// A Vec of `ProviderConfigError` objects representing warnings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = EvaluatorValidator::new();
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("rag"));
    /// config.insert("dependencies".to_string(), json!([
    ///     {"type": "memory", "legacy_mode": true} // Dependency with deprecated feature
    /// ]));
    ///
    /// // Get dependency warnings
    /// let warnings = validator.validate_dependencies_warnings(&config);
    /// for warning in warnings {
    ///     println!("Warning: {}", warning);
    /// }
    /// ```
    fn validate_dependencies_warnings(
        &self,
        _config: &HashMap<String, serde_json::Value>,
    ) -> Vec<ProviderConfigError> {
        Vec::new()
    }

    /// Validates a provider configuration.
    ///
    /// This method combines all validation methods to perform a complete
    /// validation of the configuration. It executes each validation method in sequence
    /// and returns on the first error encountered.
    ///
    /// # Validation Sequence
    ///
    /// 1. Schema validation
    /// 2. Provider-specific validation
    /// 3. Capability validation
    /// 4. Dependency validation
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the configuration is valid
    /// * `Err(ProviderConfigError)` if the configuration is invalid, with details about the error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = TypeCheckerValidator::new();
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("memory"));
    /// config.insert("ttl".to_string(), json!(3600));
    ///
    /// // Validate the entire configuration
    /// match validator.validate(&config) {
    ///     Ok(()) => println!("Configuration is valid"),
    ///     Err(error) => println!("Validation error: {}", error),
    /// }
    /// ```
    ///
    /// # See Also
    ///
    /// * `validate_collecting` - For collecting all errors and warnings instead of returning on the first error
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
#[derive(Clone)]
pub struct ErrorCollector {
    /// Errors collected during validation.
    pub errors: Vec<ProviderConfigError>,
    /// Warnings collected during validation.
    pub warnings: Vec<ProviderConfigError>,
    /// Event bus for publishing error events.
    pub event_bus: Option<Arc<EventBus>>,
    /// Provider ID for error events.
    pub provider_id: Option<String>,
}

impl std::fmt::Debug for ErrorCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorCollector")
            .field("errors", &self.errors)
            .field("warnings", &self.warnings)
            .field("event_bus", &"<EventBus>".to_string())
            .field("provider_id", &self.provider_id)
            .finish()
    }
}

#[derive(Default)]
struct ErrorCollectorDefault {
    errors: Vec<ProviderConfigError>,
    warnings: Vec<ProviderConfigError>,
    event_bus: Option<Arc<EventBus>>,
    provider_id: Option<String>,
}

impl Default for ErrorCollector {
    fn default() -> Self {
        let default = ErrorCollectorDefault::default();
        Self {
            errors: default.errors,
            warnings: default.warnings,
            event_bus: default.event_bus,
            provider_id: default.provider_id,
        }
    }
}

impl ErrorCollector {
    /// Creates a new error collector.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new error collector with an event bus.
    pub fn new_with_event_bus(event_bus: Arc<EventBus>, provider_id: impl Into<String>) -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            event_bus: Some(event_bus),
            provider_id: Some(provider_id.into()),
        }
    }

    /// Adds an error to the collector and publishes an error event if an event bus is available.
    pub fn add_error(&mut self, error: ProviderConfigError) {
        // Publish error event if event bus is available
        if let (Some(event_bus), Some(provider_id)) = (&self.event_bus, &self.provider_id) {
            let error_event = ProviderErrorEvent::new(error.clone(), provider_id.clone());
            // Use sync_publish to avoid requiring async
            let _ = event_bus.sync_publish_error(error_event.into());
        }

        self.errors.push(error);
    }

    /// Adds a warning to the collector and publishes a warning event if an event bus is available.
    pub fn add_warning(&mut self, warning: ProviderConfigError) {
        // Publish warning event if event bus is available
        if let (Some(event_bus), Some(provider_id)) = (&self.event_bus, &self.provider_id) {
            let warning_event = ProviderErrorEvent::new(warning.clone(), provider_id.clone())
                .with_context("Warning during provider config validation");
            // Use sync_publish to avoid requiring async
            let _ = event_bus.sync_publish_error(warning_event.into());
        }

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
///
/// # Error Collection
///
/// The `validate_collecting` method collects all errors and warnings from all validation
/// stages, allowing for a more comprehensive validation report. This is particularly
/// useful during development and debugging, as it provides a complete picture of all
/// validation issues at once.
///
/// # Usage
///
/// This trait is automatically implemented for all types that implement `ProviderConfigValidator`,
/// so you can call `validate_collecting` on any validator instance.
///
/// # Examples
///
/// ```rust,ignore
/// use kairei_core::provider::config::validator::{ProviderConfigValidator, CollectingValidator};
/// use kairei_core::provider::config::validators::type_checker::TypeCheckerValidator;
///
/// let validator = TypeCheckerValidator::new();
/// let mut config = HashMap::new();
/// config.insert("type".to_string(), json!("memory"));
///
/// // Collect all errors and warnings
/// let collector = validator.validate_collecting(&config);
///
/// // Check for errors
/// if collector.has_errors() {
///     println!("Validation errors:");
///     for error in &collector.errors {
///         println!("- {}", error);
///     }
/// }
///
/// // Check for warnings
/// if collector.has_warnings() {
///     println!("Validation warnings:");
///     for warning in &collector.warnings {
///         println!("- {}", warning);
///     }
/// }
/// ```
pub trait CollectingValidator: ProviderConfigValidator {
    /// Validates a provider configuration and collects errors.
    ///
    /// This method combines all validation methods to perform a complete
    /// validation of the configuration, collecting all errors rather than
    /// stopping at the first error. It executes each validation method in sequence
    /// and collects all errors and warnings from all stages.
    ///
    /// # Validation Sequence
    ///
    /// 1. Schema validation
    /// 2. Provider-specific validation
    /// 3. Capability validation
    /// 4. Dependency validation
    ///
    /// # Parameters
    ///
    /// * `config` - A HashMap containing the provider configuration to validate
    ///
    /// # Returns
    ///
    /// An `ErrorCollector` containing all errors and warnings found during validation
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let validator = TypeCheckerValidator::new();
    /// let mut config = HashMap::new();
    /// config.insert("type".to_string(), json!("memory"));
    ///
    /// // Collect all errors and warnings
    /// let collector = validator.validate_collecting(&config);
    ///
    /// // Process results
    /// if collector.has_errors() {
    ///     println!("Validation failed with {} errors", collector.errors.len());
    /// } else if collector.has_warnings() {
    ///     println!("Validation passed with {} warnings", collector.warnings.len());
    /// } else {
    ///     println!("Validation passed with no issues");
    /// }
    /// ```
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
