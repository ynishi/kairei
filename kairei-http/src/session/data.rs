use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use dashmap::DashMap;
use kairei_core::{
    config::{ProviderSecretConfig, SecretConfig, SystemConfig},
    system::System,
};
use secrecy::{ExposeSecret, SecretString};
use tokio::sync::RwLock;

pub type SessionId = String;
pub type UserId = String;
pub type SystemId = String;

pub struct SessionSecret {
    pub admin_service_key: String,
    pub user_service_key: String,
}

#[derive(Debug, Clone)]
pub struct SessionSecretConfig {
    pub providers: DashMap<String, SessionProviderSecretConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionProviderSecretConfig {
    pub api_key: SecretString,
    pub additional_auth: DashMap<String, SecretString>, // 追加の認証情報
}

impl From<SecretConfig> for SessionSecretConfig {
    fn from(secret_config: SecretConfig) -> Self {
        let providers = DashMap::new();
        for (provider_name, secret) in secret_config.providers {
            providers.insert(
                provider_name,
                SessionProviderSecretConfig {
                    api_key: SecretString::from(secret.api_key),
                    additional_auth: secret.additional_auth.into_iter().fold(
                        DashMap::new(),
                        |map, (k, v)| {
                            map.insert(k, SecretString::from(v));
                            map
                        },
                    ),
                },
            );
        }
        Self { providers }
    }
}

impl From<SessionSecretConfig> for SecretConfig {
    fn from(secret_config: SessionSecretConfig) -> Self {
        let providers = secret_config
            .providers
            .iter()
            .fold(HashMap::new(), |mut map, kv| {
                map.insert(
                    kv.key().clone(),
                    ProviderSecretConfig {
                        api_key: kv.api_key.expose_secret().to_string(),
                        additional_auth: kv.additional_auth.iter().fold(
                            HashMap::new(),
                            |mut map, kv| {
                                map.insert(kv.key().clone(), kv.expose_secret().to_string());
                                map
                            },
                        ),
                    },
                );
                map
            });
        Self { providers }
    }
}

/// Data for the session
#[derive(Clone)]
pub struct SessionData {
    pub system_id: SystemId,
    pub user_id: UserId,
    pub system: Arc<RwLock<System>>,
    pub system_config: SystemConfig,
    pub secret_config: SessionSecretConfig,
}

/// Builder for session data
#[derive(Default, Clone)]
pub struct SessionDataBuilder {
    system_id: Option<SystemId>,
    user_id: Option<UserId>,
    system: Option<Arc<RwLock<System>>>,
    system_config: Option<SystemConfig>,
    secret_config: Option<SecretConfig>,
}

impl SessionDataBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn system_id(mut self, system_id: String) -> Self {
        self.system_id = Some(system_id);
        self
    }

    pub fn user_id(mut self, user_id: UserId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn system(mut self, system: Arc<RwLock<System>>) -> Self {
        self.system = Some(system);
        self
    }

    pub fn system_config(mut self, system_config: SystemConfig) -> Self {
        self.system_config = Some(system_config);
        self
    }

    pub fn secret_config(mut self, secret_config: SecretConfig) -> Self {
        self.secret_config = Some(secret_config);
        self
    }

    pub fn build(self) -> Result<SessionData> {
        Ok(SessionData {
            system_id: self.system_id.context("system_id not set")?,
            user_id: self.user_id.context("user_id not set")?,
            system: self.system.context("system not set")?,
            system_config: self.system_config.context("system_config not set")?,
            secret_config: SessionSecretConfig::from(
                self.secret_config.context("secret_config not set")?,
            ),
        })
    }
}

// test for session builder
#[cfg(test)]
mod tests {
    use super::*;
    use kairei_core::config::SystemConfig;

    #[tokio::test]
    async fn test_session_data() {
        let system_id = "test_system".to_string();
        let user_id = "test_user".to_string();
        let system_config = SystemConfig::default();
        let secret_config = SecretConfig::default();
        let system = Arc::new(RwLock::new(
            System::new(&system_config, &secret_config).await,
        ));

        let session_data = SessionDataBuilder::new()
            .system_id(system_id.clone())
            .user_id(user_id.clone())
            .system(system)
            .system_config(system_config.clone())
            .secret_config(secret_config.clone())
            .build()
            .unwrap();

        assert_eq!(session_data.system_id, system_id);
        assert_eq!(session_data.user_id, user_id);
        assert_eq!(
            format!("{:?}", session_data.system_config),
            format!("{:?}", system_config)
        );
        assert_eq!(
            format!("{:?}", SecretConfig::from(session_data.secret_config)),
            format!("{:?}", secret_config)
        );
    }
}
