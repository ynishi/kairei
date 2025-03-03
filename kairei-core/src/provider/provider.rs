use core::fmt;
use std::collections::HashMap;

use crate::{
    config::{ProviderConfig, ProviderSecretConfig},
    timestamp::Timestamp,
};

use super::{
    capability::Capabilities,
    config::{ErrorCollector, ProviderConfigError},
    request::{ProviderContext, ProviderRequest, ProviderResponse},
    types::ProviderResult,
};
use async_trait::async_trait;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

/// # Provider Interface
///
/// The `Provider` trait defines the core contract for LLM providers in KAIREI.
/// It serves as the primary integration point for different language models
/// and AI services, abstracting their specific implementations behind a
/// consistent interface.
///
/// ## Lifecycle
///
/// Providers follow a defined lifecycle:
/// 1. Creation - Provider instance is created
/// 2. Initialization - Provider is configured and validated
/// 3. Execution - Provider processes requests
/// 4. Shutdown - Provider releases resources
///
/// ## Capabilities
///
/// Each provider declares its capabilities, allowing the system to:
/// - Match provider capabilities with request requirements
/// - Enable plugins that require specific capabilities
/// - Optimize request routing based on provider strengths
///
/// ## Implementation Example
///
/// ```ignore
/// # use kairei_core::provider::provider::Provider;
/// # use kairei_core::provider::request::{ProviderContext, ProviderRequest, ProviderResponse};
/// # use kairei_core::provider::types::ProviderResult;
/// # use kairei_core::provider::capability::Capabilities;
/// # use kairei_core::config::ProviderConfig;
/// # use kairei_core::provider::provider::ProviderSecret;
/// # use async_trait::async_trait;
/// #
/// # struct MyProvider {
/// #     name: String,
/// # }
/// #
/// # #[async_trait]
/// # impl Provider for MyProvider {
/// #     async fn execute(
/// #         &self,
/// #         _context: &ProviderContext,
/// #         _request: &ProviderRequest,
/// #     ) -> ProviderResult<ProviderResponse> {
/// #         unimplemented!()
/// #     }
/// #
/// #     async fn capabilities(&self) -> Capabilities {
/// #         unimplemented!()
/// #     }
/// #
/// #     fn name(&self) -> &str {
/// #         &self.name
/// #     }
/// #
/// #     async fn initialize(
/// #         &mut self,
/// #         _config: &ProviderConfig,
/// #         _secret: &ProviderSecret,
/// #     ) -> ProviderResult<()> {
/// #         unimplemented!()
/// #     }
/// # }
/// ```
#[async_trait]
pub trait Provider: Send + Sync {
    /// Executes a request and returns a response.
    ///
    /// This is the core method of the Provider interface, responsible for processing
    /// requests and generating responses. The implementation should handle:
    ///
    /// - Request validation
    /// - Provider-specific processing
    /// - Error handling and recovery
    /// - Response generation
    ///
    /// # Parameters
    ///
    /// * `context` - The execution context containing environment information
    /// * `request` - The request to be processed
    ///
    /// # Returns
    ///
    /// A `ProviderResult` containing either a `ProviderResponse` or an error
    async fn execute(
        &self,
        context: &ProviderContext,
        request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse>;

    /// Returns the provider's capabilities.
    ///
    /// This method allows the provider to declare what capabilities it supports,
    /// enabling the system to match requests with appropriate providers and
    /// to enable plugins that require specific capabilities.
    ///
    /// # Returns
    ///
    /// A `Capabilities` object describing the provider's supported features
    async fn capabilities(&self) -> Capabilities;

    /// Returns the provider's name.
    ///
    /// The name is used for identification, logging, and debugging purposes.
    ///
    /// # Returns
    ///
    /// A string slice containing the provider's name
    fn name(&self) -> &str;

    /// Initializes the provider with the given configuration.
    ///
    /// This method is called during provider registration to:
    /// - Validate the configuration
    /// - Set up connections to external services
    /// - Initialize internal state
    /// - Prepare the provider for request processing
    ///
    /// # Parameters
    ///
    /// * `config` - The provider configuration
    /// * `secret` - The provider secrets (API keys, etc.)
    ///
    /// # Returns
    ///
    /// A `ProviderResult` indicating success or failure
    async fn initialize(
        &mut self,
        config: &ProviderConfig,
        secret: &ProviderSecret,
    ) -> ProviderResult<()>;

    /// Validates the provider configuration.
    ///
    /// This method checks that the configuration is valid for the provider,
    /// including schema validation, provider-specific validation, capability
    /// validation, and dependency validation.
    ///
    /// # Parameters
    ///
    /// * `config` - The provider configuration to validate
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    #[allow(clippy::result_large_err)]
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), ProviderConfigError> {
        // Default implementation that delegates to validate_config_collecting
        let collector = self.validate_config_collecting(config);
        collector.result()
    }

    /// Validates the provider configuration and collects errors.
    ///
    /// This method checks that the configuration is valid for the provider,
    /// collecting all errors rather than stopping at the first error.
    ///
    /// # Parameters
    ///
    /// * `config` - The provider configuration to validate
    ///
    /// # Returns
    ///
    /// An `ErrorCollector` containing any errors or warnings
    fn validate_config_collecting(&self, _config: &ProviderConfig) -> ErrorCollector {
        // Default implementation that returns an empty collector
        ErrorCollector::new()
    }

    /// Shuts down the provider and releases resources.
    ///
    /// This method is called during system shutdown to allow the provider
    /// to clean up resources and perform any necessary finalization.
    ///
    /// # Returns
    ///
    /// A `ProviderResult` indicating success or failure
    async fn shutdown(&self) -> ProviderResult<()> {
        Ok(())
    }

    /// Performs a health check on the provider.
    ///
    /// This method is called periodically to verify that the provider
    /// is functioning correctly and is ready to process requests.
    ///
    /// # Returns
    ///
    /// A `ProviderResult` indicating the provider's health status
    async fn health_check(&self) -> ProviderResult<()> {
        Ok(())
    }
}

/// # Section
///
/// A `Section` represents a part of a prompt or response in the provider system.
/// Sections are used to build structured prompts and to organize responses
/// from language models.
///
/// ## Fields
///
/// * `content` - The text content of the section
/// * `priority` - The priority of the section (used for ordering)
/// * `metadata` - Additional information about the section
#[derive(Debug, Default)]
pub struct Section {
    pub content: String,
    pub priority: i32,
    pub metadata: SectionMetadata,
}

impl Section {
    /// Creates a new section with the given content.
    ///
    /// # Parameters
    ///
    /// * `content` - The text content of the section
    ///
    /// # Returns
    ///
    /// A new `Section` with default priority and metadata
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            ..Default::default()
        }
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// # Section Metadata
///
/// `SectionMetadata` provides additional information about a section,
/// such as its source and creation timestamp.
///
/// ## Fields
///
/// * `source` - The source of the section (e.g., plugin name)
/// * `timestamp` - When the section was created
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SectionMetadata {
    pub source: String,
    pub timestamp: Timestamp,
}

impl SectionMetadata {
    /// Creates new metadata with the given source.
    ///
    /// # Parameters
    ///
    /// * `source` - The source of the section
    ///
    /// # Returns
    ///
    /// A new `SectionMetadata` with the current timestamp
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            timestamp: Timestamp::now(),
        }
    }
}

impl fmt::Display for SectionMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source: {}, timestamp: {}", self.source, self.timestamp)
    }
}

/// # Provider Type
///
/// `ProviderType` enumerates the different types of providers supported by the system.
/// This is used for provider identification, configuration, and capability matching.
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    Default,
    PartialEq,
    PartialOrd,
)]
pub enum ProviderType {
    #[default]
    OpenAIAssistant,
    SimpleExpert,
    OpenAIChat,
    Unknown,
}

impl From<ProviderType> for String {
    fn from(provider_type: ProviderType) -> Self {
        provider_type.to_string()
    }
}

/// # Provider Secret
///
/// `ProviderSecret` manages sensitive authentication information for providers,
/// such as API keys and other credentials.
///
/// ## Security Considerations
///
/// - API keys and other secrets are stored using `SecretString` to prevent accidental exposure
/// - Secrets are not logged or serialized in plain text
/// - Additional authentication parameters can be stored in the `additional_auth` map
#[derive(Clone, Default)]
pub struct ProviderSecret {
    pub api_key: SecretString,
    pub additional_auth: HashMap<String, SecretString>,
}

impl From<ProviderSecretConfig> for ProviderSecret {
    fn from(secret: ProviderSecretConfig) -> Self {
        let additional_auth = secret
            .additional_auth
            .into_iter()
            .map(|(k, v)| (k, SecretString::from(v)))
            .collect();
        Self {
            api_key: SecretString::from(secret.api_key),
            additional_auth,
        }
    }
}
