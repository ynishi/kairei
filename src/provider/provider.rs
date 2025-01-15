use std::sync::Arc;

use crate::config::ProviderConfig;

use super::{
    capability::{Capabilities, RequiredCapabilities, RequiresCapabilities},
    plugin::PluginContext,
    request::{ProviderContext, ProviderRequest, ProviderResponse},
    types::{ProviderResult, ProviderSecret},
};
use async_trait::async_trait;
use mockall::automock;

#[async_trait]
#[automock]
pub trait Provider: Send + Sync {
    async fn execute(
        &self,
        context: &ProviderContext,
        request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse>;
    fn capabilities(&self) -> Capabilities;

    fn name(&self) -> &str;

    // validate the provider configuration
    async fn initialize(
        &self,
        config: &ProviderConfig,
        secret: &ProviderSecret,
    ) -> ProviderResult<()>;

    async fn shutdown(&self) -> ProviderResult<()>;

    async fn health_check(&self) -> ProviderResult<()> {
        Ok(())
    }
}
