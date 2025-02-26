//! Plugin Configuration System
//!
//! Provides a type-safe configuration system for KAIREI plugins with provider-specific
//! extensions. The system consists of:
//!
//! ## Core Components
//!
//! ### Base Configuration
//! - [`BasePluginConfig`]: Common settings shared by all plugins
//! - [`ProviderSpecificConfig`]: Trait for provider-specific extensions
//!
//! ### Plugin-Specific Configurations
//! - [`RagConfig`]: RAG plugin configuration with chunk size and similarity settings
//! - [`MemoryConfig`]: Memory plugin configuration with TTL and capacity controls
//! - [`SearchConfig`]: Search plugin configuration with result limits and filters
//!
//! ### Provider Extensions
//! Provider-specific implementations extend base configurations:
//! ```rust
//! use kairei::provider::config::{RagConfig, OpenAIApiConfig};
//!
//! pub struct OpenAIRagConfig {
//!     pub base: RagConfig,
//!     pub api_config: OpenAIApiConfig,
//! }
//! ```
//!
//! ## Validation
//! The configuration system ensures type safety through:
//! - Compile-time type checking
//! - Runtime validation via [`ConfigValidation`] trait
//! - Clear error messages with [`ConfigError`]
//!
//! ## Example Usage
//! ```rust
//! use kairei::provider::config::{RagConfig, ProviderSpecificConfig, ConfigError};
//!
//! fn main() -> Result<(), ConfigError> {
//!     let config = RagConfig {
//!         chunk_size: 512,
//!         max_tokens: 1000,
//!         similarity_threshold: 0.7,
//!         ..Default::default()
//!     };
//!     config.validate()?;
//!     Ok(())
//! }
//! ```

mod base;
mod errors;
mod plugins;
mod providers;
mod validation;
mod validator;
mod validators;

#[cfg(test)]
mod tests;

pub use base::{ConfigError, ConfigValidation, PluginConfig, PluginType};
pub use errors::{
    ErrorContext, ErrorSeverity, ProviderConfigError, ProviderError, SchemaError, SourceLocation,
    ValidationError,
};
pub use plugins::{
    BasePluginConfig, MemoryConfig, ProviderSpecificConfig, RagConfig, SearchConfig,
};
pub use providers::{OpenAIApiConfig, OpenAIMemoryConfig, OpenAIRagConfig, OpenAISearchConfig};
pub use validation::{
    check_property_type, check_required_properties, validate_range, validate_required_field,
};
pub use validator::{CollectingValidator, ErrorCollector, ProviderConfigValidator};
pub use validators::{
    create_evaluator_validator, create_type_checker_validator, EvaluatorValidator,
    TypeCheckerValidator,
};
