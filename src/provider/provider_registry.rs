use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::{
    config::{ProviderConfigs, SecretConfig},
    event_bus::{ErrorEvent, Event, EventBus, Value},
    event_registry::EventType,
};

use super::{
    openai_assistant::OpenAIAssistantProvider,
    provider_secret::SecretRegistry,
    types::{
        LLMProvider, ProviderError, ProviderInstance, ProviderResult, ProviderState, ProviderType,
    },
};

/// プロバイダーリポジトリ
pub struct ProviderRegistry {
    configs: ProviderConfigs,
    providers: DashMap<String, ProviderInstance>,
    states: DashMap<String, ProviderState>,
    secret_registry: SecretRegistry,
    primary_provider: RwLock<Option<String>>,
    event_bus: Arc<EventBus>,
}

impl ProviderRegistry {
    // 同期的な基本初期化
    pub async fn new(
        provider_configs: ProviderConfigs,
        secret_config: SecretConfig,
        event_bus: Arc<EventBus>,
    ) -> Self {
        let primary_provider = RwLock::new(provider_configs.primary_provider.clone());
        Self {
            configs: provider_configs.clone(),
            providers: DashMap::new(),
            states: DashMap::new(),
            secret_registry: SecretRegistry::new(secret_config.clone()),
            primary_provider,
            event_bus,
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn register_providers(&self) -> ProviderResult<()> {
        for (name, config) in self.configs.providers.iter() {
            self.register_provider(name, config.provider_type.clone())
                .await?;
        }
        Ok(())
    }

    /// プロバイダーの登録と初期化
    #[instrument(level = "debug", skip(self))]
    pub async fn register_provider(
        &self,
        name: &str,
        provider_type: ProviderType,
    ) -> ProviderResult<()> {
        let provider = self.create_provider(&provider_type).await?;

        self.register_provider_with(name, provider).await?;

        let _ = self
            .event_bus
            .publish(Event {
                event_type: EventType::ProviderRegistered,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "provider_type".to_string(),
                        Value::String(provider_type.to_string()),
                    );
                    params.insert("provider_name".to_string(), Value::String(name.to_string()));
                    params
                },
            })
            .await;

        Ok(())
    }

    pub async fn register_provider_with(
        &self,
        name: &str,
        provider: Arc<dyn LLMProvider>,
    ) -> ProviderResult<()> {
        let secret = self.secret_registry.get_secret(name)?;
        let config = self
            .configs
            .providers
            .get(name)
            .ok_or(ProviderError::ProviderNotFound(name.to_string()))?;

        // 設定のバリデーション
        provider.validate_config(config).await?;

        // プロバイダーの初期化
        let provider = provider.initialize(config, &secret).await?;

        // 状態の初期化
        let state = ProviderState {
            is_healthy: true,
            last_health_check: std::time::SystemTime::now(),
            error_count: 0,
            last_error: None,
        };

        let insance = ProviderInstance {
            provider,
            secret,
            config: config.clone(),
        };

        self.providers.insert(name.to_string(), insance);
        self.states.insert(name.to_string(), state);

        Ok(())
    }

    /// デフォルトプロバイダーの設定
    #[instrument(level = "debug", skip(self))]
    pub async fn set_default_provider(&self, name: &str) -> ProviderResult<()> {
        if self.providers.contains_key(name) {
            let mut primary_provider = self.primary_provider.write().await;
            *primary_provider = Some(name.to_string());

            let _ = self
                .event_bus
                .publish(Event {
                    event_type: EventType::ProviderPrimarySet,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("provider_name".to_string(), Value::String(name.to_string()));
                        params
                    },
                })
                .await;
            Ok(())
        } else {
            Err(ProviderError::ProviderNotFound(name.to_string()))
        }
    }

    /// デフォルトプロバイダーの取得
    pub async fn get_primary_provider_name(&self) -> ProviderResult<String> {
        self.primary_provider
            .read()
            .await
            .clone()
            .ok_or(ProviderError::PrimaryNameNotSet)
    }

    /// プロバイダーの取得
    pub async fn get_provider(&self, name: &str) -> ProviderResult<ProviderInstance> {
        if !self.providers.contains_key(name) {
            return Err(ProviderError::ProviderNotFound(name.to_string()));
        }
        self.providers
            .get(&name.to_string())
            .map(|entry| entry.value().clone())
            .ok_or(ProviderError::ProviderNotFound(name.to_string()))
    }

    /// プロバイダーの状態取得
    pub async fn get_provider_state(&self, name: &str) -> ProviderResult<ProviderState> {
        self.states
            .get(name)
            .map(|state| state.value().clone())
            .ok_or(ProviderError::ProviderNotFound(name.to_string()))
    }

    /// プロバイダーのヘルスチェック実行
    pub async fn check_providers_health(&self) -> ProviderResult<()> {
        let names = self
            .providers
            .iter()
            .map(|entry| entry.key().clone())
            .collect::<Vec<_>>();
        for name in names {
            self.check_provider_health(&name).await?;
        }
        Ok(())
    }

    pub async fn check_provider_health(&self, name: &str) -> ProviderResult<()> {
        let instance = self.get_provider(name).await?;
        let health_result = instance.provider.health_check().await;

        if let Some(mut state) = self.states.get_mut(name) {
            let value = state.value_mut();
            value.is_healthy = health_result.is_ok();
            value.last_health_check = std::time::SystemTime::now();
            if let Err(err) = &health_result {
                value.error_count += 1;
                value.last_error = Some(err.to_string());
                let error_event = if self.get_primary_provider_name().await? == name {
                    ErrorEvent {
                        error_type: "primary provider unhealthy".to_string(),
                        message: err.to_string(),
                        severity: crate::event_bus::ErrorSeverity::Error,
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert(
                                "provider_name".to_string(),
                                Value::String(name.to_string()),
                            );
                            params
                        },
                    }
                } else {
                    ErrorEvent {
                        error_type: "provider unhealthy".to_string(),
                        message: err.to_string(),
                        severity: crate::event_bus::ErrorSeverity::Warning,
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert(
                                "provider_name".to_string(),
                                Value::String(name.to_string()),
                            );
                            params
                        },
                    }
                };
                let _ = self.event_bus.publish_error(error_event).await;
            } else {
                let _ = self
                    .event_bus
                    .publish(Event {
                        event_type: EventType::ProviderStatusUpdated,
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert(
                                "provider_name".to_string(),
                                Value::String(name.to_string()),
                            );
                            params
                                .insert("is_healthy".to_string(), Value::Boolean(value.is_healthy));
                            params.insert(
                                "last_updated_at".to_string(),
                                Value::String(format!("{:?}", value.last_health_check)),
                            );
                            params
                        },
                    })
                    .await;
            }
        }

        health_result
    }

    /// すべてのプロバイダーのシャットダウン
    #[instrument(level = "debug", skip(self))]
    pub async fn shutdown(&self) -> ProviderResult<()> {
        let names = self
            .providers
            .iter()
            .map(|entry| entry.key().clone())
            .collect::<Vec<_>>();
        for name in names {
            debug!("shutdown start: {}", name);
            let entry = self
                .providers
                .get_mut(&name)
                .ok_or(ProviderError::ProviderNotFound(name.clone()))?;
            entry.provider.shutdown().await?;
            drop(entry);
            self.providers.remove(&name);
            self.states.remove(&name);
            let _ = self
                .event_bus
                .publish(Event {
                    event_type: EventType::ProviderShutdown,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("provider_name".to_string(), Value::String(name.clone()));
                        params
                    },
                })
                .await;
        }

        Ok(())
    }

    /// Factory method to create a new assistant
    #[instrument(level = "debug", skip(self))]
    pub async fn create_provider(
        &self,
        provider_type: &ProviderType,
    ) -> ProviderResult<Arc<dyn LLMProvider>> {
        match provider_type {
            ProviderType::OpenAIAssistant => self.create_assistant().await,
            _ => Err(ProviderError::UnknownProvider(provider_type.to_string())),
        }
    }

    pub async fn create_assistant(&self) -> ProviderResult<Arc<dyn LLMProvider>> {
        Ok(Arc::new(OpenAIAssistantProvider::new()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_trait::async_trait;

    use crate::{
        config::{CommonConfig, EndpointConfig, ProviderConfig, ProviderSecretConfig},
        provider::types::ProviderSecret,
    };

    use super::*;

    #[derive(Clone)]
    struct MockProvider {
        name: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn initialize(
            &self,
            _config: &ProviderConfig,
            _secret: &ProviderSecret,
        ) -> ProviderResult<Arc<dyn LLMProvider>> {
            Ok(Arc::new(self.clone()))
        }

        async fn shutdown(&self) -> ProviderResult<()> {
            Ok(())
        }

        async fn validate_config(&self, _config: &ProviderConfig) -> ProviderResult<()> {
            Ok(())
        }

        async fn create_assistant(&self, _config: &ProviderConfig) -> ProviderResult<String> {
            Ok("mock_assistant_id".to_string())
        }

        async fn create_thread(&self) -> ProviderResult<String> {
            Ok("mock_thread_id".to_string())
        }

        async fn send_message(
            &self,
            _thread_id: &str,
            _assistant_id: &str,
            _content: &str,
        ) -> ProviderResult<String> {
            Ok("mock_response".to_string())
        }

        async fn delete_thread(&self, _thread_id: &str) -> ProviderResult<()> {
            Ok(())
        }

        async fn health_check(&self) -> ProviderResult<()> {
            Ok(())
        }
    }

    async fn get_registry(names: &[String]) -> ProviderRegistry {
        let mut provider_configs = HashMap::new();
        let mut secret_configs = SecretConfig {
            providers: HashMap::new(),
        };
        let primary_name = names[0].clone();
        for name in names.iter() {
            let config = ProviderConfig {
                provider_type: ProviderType::Unknown,
                name: name.to_string(),
                common_config: CommonConfig {
                    temperature: 0.7,
                    max_tokens: 1000,
                    model: "gpt-3.5-turbo".to_string(),
                },
                provider_specific: HashMap::new(),
                endpoint: EndpointConfig::default(),
            };
            provider_configs.insert(name.to_string(), config);

            secret_configs.providers.insert(
                name.to_string(),
                ProviderSecretConfig {
                    api_key: "mock_api_key".to_string(),
                    additional_auth: HashMap::new(),
                },
            );
        }

        let config = ProviderConfigs {
            primary_provider: Some(primary_name.to_string()),
            providers: provider_configs,
        };
        let event_bus = Arc::new(EventBus::new(20));
        ProviderRegistry::new(config, secret_configs, event_bus).await
    }

    #[tokio::test]
    async fn test_provider_registry() {
        let name = "mock";
        let registry = get_registry(&[name.to_string()]).await;

        // プロバイダーの登録

        let provider = MockProvider {
            name: "mock".to_string(),
        };

        registry
            .register_provider_with(&name, Arc::new(provider))
            .await
            .unwrap();

        // デフォルトプロバイダー
        registry.set_default_provider("mock").await.unwrap();
        assert_eq!(registry.get_primary_provider_name().await.unwrap(), "mock");

        // プロバイダーの取得
        assert!(registry.get_provider("mock").await.is_ok());
        assert!(registry.get_provider("nonexistent").await.is_err());

        // ヘルスチェック
        registry.check_provider_health("mock").await.unwrap();
        let state = registry.get_provider_state("mock").await.unwrap();
        assert!(state.is_healthy);
        println!("{:?}", state);

        // シャットダウン
        registry.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let mut names = vec![];
        for i in 0..10 {
            names.push(format!("mock_{}", i));
        }
        let registry = Arc::new(get_registry(names.as_slice()).await);

        // 並行アクセスのテスト
        let mut handles = vec![];
        for name in names.clone() {
            let registry_clone = registry.clone();
            let handle = tokio::spawn(async move {
                let provider = MockProvider { name: name.clone() };
                registry_clone
                    .register_provider_with(&name, Arc::new(provider))
                    .await
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // 登録されたプロバイダーの確認
        let providers = registry.providers.iter().collect::<Vec<_>>();
        assert_eq!(providers.len(), 10);
    }
}
