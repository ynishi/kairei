use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{
    LLMProvider, ProviderError, ProviderInstance, ProviderParams, ProviderResult, ProviderState,
};

/// プロバイダーリポジトリ
pub struct ProviderRegistry {
    providers: DashMap<String, ProviderInstance>,
    states: DashMap<String, ProviderState>,
    primary_provider: Arc<RwLock<Option<String>>>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
            states: DashMap::new(),
            primary_provider: Arc::new(RwLock::new(None)),
        }
    }

    /// プロバイダーの登録と初期化
    pub async fn register_provider(
        &self,
        provider: Arc<dyn LLMProvider>,
        params: ProviderParams,
    ) -> ProviderResult<()> {
        let provider_name = provider.name().to_string();

        // 設定のバリデーション
        provider.validate_config(&params.config).await?;

        // プロバイダーの初期化
        let provider = provider.initialize(params.clone()).await?;

        // 状態の初期化
        let state = ProviderState {
            is_healthy: true,
            last_health_check: std::time::SystemTime::now(),
        };

        let insance = ProviderInstance {
            provider,
            config: params.config.clone(),
        };

        self.providers.insert(provider_name.clone(), insance);
        self.states.insert(provider_name, state);

        Ok(())
    }

    /// デフォルトプロバイダーの設定
    pub async fn set_default_provider(&self, name: &str) -> ProviderResult<()> {
        if self.providers.contains_key(name) {
            let mut primary_provider = self.primary_provider.write().await;
            *primary_provider = Some(name.to_string());
            Ok(())
        } else {
            Err(ProviderError::NotFound(name.to_string()))
        }
    }

    /// プロバイダーの取得
    pub async fn get_provider(
        &self,
        name: Option<&str>,
    ) -> Result<Arc<dyn LLMProvider>, ProviderError> {
        let provider_name = if let Some(name) = name {
            if !self.providers.contains_key(name) {
                return Err(ProviderError::NotFound(name.to_string()));
            }
            name.to_string()
        } else if let Some(primary) = self.primary_provider.read().await.clone() {
            primary
        } else {
            return Err(ProviderError::NameNotSpecified);
        };
        self.providers
            .get(&provider_name)
            .map(|provider| provider.provider.clone())
            .ok_or(ProviderError::NotFound(provider_name))
    }

    /// プロバイダーの状態取得
    pub async fn get_provider_state(&self, name: &str) -> ProviderResult<ProviderState> {
        self.states
            .get(name)
            .map(|state| state.value().clone())
            .ok_or(ProviderError::NotFound(name.to_string()))
    }

    /// プロバイダーのヘルスチェック実行
    pub async fn check_provider_health(&self, name: &str) -> Result<(), ProviderError> {
        let provider = self.get_provider(Some(name)).await?;
        let health_result = provider.health_check().await;

        if let Some(mut state) = self.states.get_mut(name) {
            let value = state.value_mut();
            value.is_healthy = health_result.is_ok();
            value.last_health_check = std::time::SystemTime::now();
        }

        health_result
    }

    /// すべてのプロバイダーのシャットダウン
    pub async fn shutdown_all(&self) -> ProviderResult<()> {
        let names = self
            .providers
            .iter()
            .map(|entry| entry.key().clone())
            .collect::<Vec<_>>();
        for name in names {
            println!("start: {}", name);
            let entry = self
                .providers
                .get_mut(&name)
                .ok_or(ProviderError::NotFound(format!(
                    "Provider not found: {}",
                    name
                )))?;
            entry.provider.shutdown().await?;
            drop(entry);
            self.providers.remove(&name);
            self.states.remove(&name);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_trait::async_trait;

    use crate::provider::types::{CommonConfig, ProviderConfig};

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
            _params: ProviderParams,
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

    #[tokio::test]
    async fn test_provider_registry() {
        let manager = ProviderRegistry::new();

        // プロバイダーの登録
        let provider = MockProvider {
            name: "mock".to_string(),
        };
        let auth = HashMap::new();
        let params = ProviderParams {
            config: ProviderConfig {
                name: "mock".to_string(),
                common_config: CommonConfig {
                    temperature: 0.7,
                    max_tokens: 1000,
                },
                provider_specific: HashMap::new(),
            },
            auth,
        };

        manager
            .register_provider(Arc::new(provider), params)
            .await
            .unwrap();

        // デフォルトプロバイダーの設定
        manager.set_default_provider("mock").await.unwrap();

        // プロバイダーの取得
        assert!(manager.get_provider(Some("mock")).await.is_ok());
        assert!(manager.get_provider(None).await.is_ok());
        assert!(manager.get_provider(Some("nonexistent")).await.is_err());

        // ヘルスチェック
        manager.check_provider_health("mock").await.unwrap();
        let state = manager.get_provider_state("mock").await.unwrap();
        assert!(state.is_healthy);
        println!("{:?}", state);

        // シャットダウン
        manager.shutdown_all().await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let manager = Arc::new(ProviderRegistry::new());

        // 並行アクセスのテスト
        let mut handles = vec![];
        for i in 0..10 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let provider = MockProvider {
                    name: format!("mock_{}", i),
                };
                let params = ProviderParams {
                    config: ProviderConfig {
                        name: format!("mock_{}", i),
                        common_config: CommonConfig {
                            temperature: 0.7,
                            max_tokens: 1000,
                        },
                        provider_specific: HashMap::new(),
                    },
                    auth: HashMap::new(),
                };
                manager_clone
                    .register_provider(Arc::new(provider), params)
                    .await
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // 登録されたプロバイダーの確認
        let providers = manager.providers.iter().collect::<Vec<_>>();
        assert_eq!(providers.len(), 10);
    }
}
