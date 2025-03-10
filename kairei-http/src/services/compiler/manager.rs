use kairei_core::{
    config::{SecretConfig, SystemConfig},
    system::{System, SystemError},
};
use std::sync::Arc;

/// Manager for System instances used by the compiler service
#[derive(Debug, Clone, Default)]
pub struct CompilerSystemManager {
    config: SystemConfig,
    secret_config: SecretConfig,
}

impl CompilerSystemManager {
    /// Create a new CompilerSystemManager with custom configurations
    pub fn new(config: SystemConfig, secret_config: SecretConfig) -> Self {
        Self {
            config,
            secret_config,
        }
    }

    /// Create a new System instance for validation
    pub async fn create_validation_system(&self) -> Arc<System> {
        Arc::new(System::new(&self.config, &self.secret_config).await)
    }

    /// Validate DSL code using a System instance
    pub async fn validate_dsl(&self, dsl: &str) -> Result<(), SystemError> {
        let system = self.create_validation_system().await;
        system.parse_dsl(dsl).await.map(|_| ())
    }
}
