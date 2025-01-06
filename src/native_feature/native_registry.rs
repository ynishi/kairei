use std::{collections::HashMap, sync::Arc};

use tokio::{sync::RwLock, time::timeout};
use tracing::{error, info};

use crate::{
    config::NativeFeatureConfig, native_feature::ticker::Ticker, ExecutionError, RuntimeError,
    RuntimeResult,
};

use super::types::{NativeFeature, NativeFeatureContext, NativeFeatureType};

#[derive(Clone)]
pub struct NativeFeatureRegistry {
    features: Arc<RwLock<HashMap<NativeFeatureType, Arc<dyn NativeFeature>>>>,
    context: Arc<NativeFeatureContext>,
    config: Arc<RwLock<NativeFeatureConfig>>,
}

impl NativeFeatureRegistry {
    pub fn new(context: Arc<NativeFeatureContext>, config: NativeFeatureConfig) -> Self {
        let features: Arc<RwLock<HashMap<NativeFeatureType, Arc<dyn NativeFeature>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let config = Arc::new(RwLock::new(config));

        Self {
            features,
            context,
            config,
        }
    }

    pub async fn register(&self) -> RuntimeResult<()> {
        let config = self.config.read().await;

        if let Some(config) = config.ticker.clone() {
            if config.enabled {
                let feature_type = NativeFeatureType::Ticker;
                if let Some(ticker) = self.create_feature(&feature_type).await {
                    self.register_feature(feature_type, ticker).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn start(&self) -> RuntimeResult<()> {
        let features = self.enabled_feature_type().await.clone();

        for feature_type in features {
            let feature_type_clone = feature_type.clone();
            /*
            let feature = self
                .get_registered_feature(&feature_type_clone)
                .await
                .ok_or_else(|| {
                    RuntimeError::Execution(ExecutionError::NativeFeatureError(format!(
                        "Feature not found: {:?}",
                        feature_type
                    )))
                })?.clone();
             */
            let feature = self
                .get_registered_feature(&feature_type_clone)
                .await
                .ok_or_else(|| {
                    RuntimeError::Execution(ExecutionError::NativeFeatureError(format!(
                        "Feature not found: {:?}",
                        feature_type
                    )))
                })?
                .clone();
            feature.init().await.map_err(|e| {
                RuntimeError::Execution(ExecutionError::NativeFeatureError(format!(
                    "Init Feature error: {}, {}",
                    feature_type, e
                )))
            })?;
            tokio::spawn(async move {
                feature.start().await.map_err(|e| {
                    RuntimeError::Execution(ExecutionError::NativeFeatureError(format!(
                        "Start Feature error: {}, {}",
                        feature_type, e
                    )))
                })
            })
            .await
            .unwrap()
            .expect("TODO: panic message");
        }
        Ok(())
    }

    pub async fn register_feature(
        &self,
        feature_type: NativeFeatureType,
        feature: Arc<dyn NativeFeature>,
    ) -> RuntimeResult<()> {
        println!("register_feature: {:?}", feature_type);
        if self.features.read().await.contains_key(&feature_type) {
            return Err(RuntimeError::Execution(ExecutionError::NativeFeatureError(
                format!("Native Feature already exists: {}", feature_type),
            )));
        }

        self.features.write().await.insert(feature_type, feature);
        Ok(())
    }

    pub async fn shutdown(&self) -> RuntimeResult<()> {
        info!("Starting NativeFeatureRegistry shutdown");

        let features = self.features.read().await;
        for (feature_type, feature) in features.iter() {
            // set timeout tokio
            let _ = timeout(self.config.read().await.shutdown_timeout, feature.stop())
                .await
                .map_err(|e| {
                    error!("Error stopping feature {:?}: {:?}", feature_type, e);
                    // エラーを記録するが、他のfeatureの停止は継続
                });
        }
        Ok(())
    }

    pub async fn get_registered_feature(
        &self,
        feature_type: &NativeFeatureType,
    ) -> Option<Arc<dyn NativeFeature>> {
        self.features.read().await.get(feature_type).cloned()
    }

    pub async fn enabled_feature_type(&self) -> Vec<NativeFeatureType> {
        let mut res = vec![];
        if self
            .config
            .read()
            .await
            .ticker
            .clone()
            .unwrap_or_default()
            .enabled
        {
            res.push(NativeFeatureType::Ticker)
        }
        res
    }

    // factory method for native feature.
    pub async fn create_feature(
        &self,
        feature_type: &NativeFeatureType,
    ) -> Option<Arc<dyn NativeFeature>> {
        match feature_type {
            NativeFeatureType::Ticker => Some(Arc::new(Ticker::new(
                self.context.clone(),
                self.config.read().await.clone().ticker.unwrap_or_default(),
            ))),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::context::AgentType;
    use crate::{
        config::{NativeFeatureConfig, SystemConfig, TickerConfig},
        event_bus::EventBus,
        native_feature::types::{
            NativeFeature, NativeFeatureContext, NativeFeatureStatus, NativeFeatureType,
        },
    };
    use std::time::Duration;
    use tokio::sync::broadcast;
    use tokio::time::sleep;

    // テスト用の共通セットアップ関数
    async fn setup_test_registry() -> NativeFeatureRegistry {
        let event_bus = Arc::new(EventBus::new(20));
        let (shutdown_tx, _) = broadcast::channel::<AgentType>(1);
        let context = Arc::new(NativeFeatureContext::new(event_bus));
        let config = NativeFeatureConfig::default();
        NativeFeatureRegistry::new(context, config)
    }

    #[tokio::test]
    async fn test_register_feature() {
        let registry = setup_test_registry().await;
        let ticker = registry
            .create_feature(&NativeFeatureType::Ticker)
            .await
            .unwrap();
        registry
            .register_feature(NativeFeatureType::Ticker, ticker)
            .await
            .unwrap();

        assert_eq!(
            registry
                .get_registered_feature(&NativeFeatureType::Ticker)
                .await
                .is_some(),
            true
        );
    }

    #[tokio::test]
    async fn test_register_duplicate_feature() {
        let registry = setup_test_registry().await;
        let ticker = registry
            .create_feature(&NativeFeatureType::Ticker)
            .await
            .unwrap();
        registry
            .register_feature(NativeFeatureType::Ticker, ticker.clone())
            .await
            .unwrap();

        // 重複して登録しようとするとエラーになる
        let result = registry
            .register_feature(NativeFeatureType::Ticker, ticker)
            .await;
        assert!(matches!(result, Err(RuntimeError::Execution(_))));
    }

    #[tokio::test]
    async fn test_initialize_native_features() {
        let mut registry = setup_test_registry().await;
        registry.register().await.unwrap();

        assert_eq!(
            registry
                .get_registered_feature(&NativeFeatureType::Ticker)
                .await
                .is_some(),
            true
        );
    }

    #[tokio::test]
    async fn test_start_native_features() {
        let mut registry = setup_test_registry().await;
        registry.register().await.unwrap();

        /*
         let registry_clone = registry.clone();
         tokio::spawn(async move {
             registry_clone.start().await.unwrap();
         }).await.unwrap();

        let ticker = registry
             .get_registered_feature(&NativeFeatureType::Ticker)
             .await
             .unwrap().clone();

        // 非同期タスクが開始されるのを少し待つ
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(ticker.status().await, NativeFeatureStatus::Active);
        */
    }
    /*
           #[tokio::test]
           async fn test_shutdown_native_features() {
               let mut registry = setup_test_registry().await;
               registry.register().await.unwrap();
               registry.start().await.unwrap();
               registry.shutdown().await.unwrap();

               /*
               let ticker = registry
                   .get_registered_feature(&NativeFeatureType::Ticker)
                   .await
                   .unwrap();
                */
               // 停止まで少し待つ
               tokio::time::sleep(Duration::from_millis(50)).await;
               // assert_eq!(ticker.status().await, NativeFeatureStatus::Inactive);
           }

    */
    #[tokio::test]
    async fn test_enabled_feature_types() {
        let registry = setup_test_registry().await;
        let enabled_types = registry.enabled_feature_type().await;

        assert!(enabled_types.contains(&NativeFeatureType::Ticker));
    }
}
