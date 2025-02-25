//! # KAIREI Provider Architecture
//!
//! The Provider module implements KAIREI's plugin architecture, enabling extensible
//! integration with various LLM providers and additional capabilities through plugins.
//!
//! ## Core Components
//!
//! * **Provider Interface**: Defines the core contract for LLM providers
//! * **Provider Plugin**: Extends provider functionality with specific capabilities
//! * **Capability System**: Manages and negotiates provider capabilities
//!
//! ## Architecture Design
//!
//! The Provider architecture follows a modular design with clear separation of concerns:
//!
//! 1. **Core Interfaces**: `Provider` and `ProviderPlugin` traits define the extension points
//! 2. **Capability Negotiation**: Providers declare capabilities, plugins require capabilities
//! 3. **Plugin Lifecycle**: Initialization, execution, and cleanup phases
//! 4. **Resource Management**: Controlled access to external resources
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use kairei::provider::provider::Provider;
//! use kairei::provider::plugin::ProviderPlugin;
//! use kairei::provider::request::{ProviderContext, ProviderRequest};
//! use kairei::config::ProviderConfig;
//! use kairei::provider::provider::ProviderSecret;
//! 
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure and initialize a provider
//!     let mut provider = MyProvider::new();
//!     let config = ProviderConfig::default();
//!     let secret = ProviderSecret::default();
//!     provider.initialize(&config, &secret).await?;
//!
//!     // Execute a request
//!     let context = ProviderContext::default();
//!     let request = ProviderRequest::default();
//!     let response = provider.execute(&context, &request).await?;
//!     Ok(())
//! }
//! ```

pub mod provider_registry;
pub mod provider_secret;

pub mod capability;
pub mod config;
pub mod generator;
pub mod llm;
pub mod llms;
pub mod plugin;
pub mod plugins;
#[allow(clippy::module_inception)]
pub mod provider;
pub mod providers;
pub mod request;
pub mod types;
