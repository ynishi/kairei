use std::collections::HashMap;

use crate::config::SecretConfig;

use super::{
    provider::ProviderSecret,
    types::{ProviderError, ProviderResult},
};

pub struct SecretRegistry {
    secrets: HashMap<String, ProviderSecret>,
}

impl SecretRegistry {
    pub fn new(config: SecretConfig) -> Self {
        let mut secrets = HashMap::new();
        for (provider_name, secret) in config.providers {
            secrets.insert(provider_name, ProviderSecret::from(secret));
        }
        Self { secrets }
    }

    pub fn get_secret(&self, provider_name: &str) -> ProviderResult<ProviderSecret> {
        self.secrets
            .get(provider_name)
            .cloned()
            .ok_or(ProviderError::SecretNotFound(provider_name.to_string()))
    }
}
