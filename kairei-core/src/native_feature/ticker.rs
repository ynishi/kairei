use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    config::TickerConfig,
    event_bus::{self, Event},
    event_registry,
};
use async_trait::async_trait;
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
};
use tracing::debug;

use super::types::{
    FeatureError, FeatureResult, NativeFeature, NativeFeatureContext, NativeFeatureStatus,
    NativeFeatureType,
};

// Tickerの実装
#[derive(Clone)]
pub struct Ticker {
    pub context: Arc<NativeFeatureContext>,
    pub status: Arc<RwLock<NativeFeatureStatus>>,
    pub running: Arc<AtomicBool>,
    pub config: TickerConfig,
    pub task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Ticker {
    pub fn new(context: Arc<NativeFeatureContext>, config: TickerConfig) -> Self {
        Self {
            context,
            status: Arc::new(RwLock::new(NativeFeatureStatus::Inactive)),
            running: Arc::new(AtomicBool::new(false)),
            config,
            task_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_status(&self, status: NativeFeatureStatus) {
        debug!("set_status: {:?} to {:?}", self.status.read().await, status);
        if *self.status.read().await == status {
            return;
        }
        *self.status.write().await = status;
        let _ = self.emit_status().await;
    }

    async fn tick(
        &self,
        mut interval_timer: tokio::time::Interval,
        event: Event,
    ) -> FeatureResult<()> {
        while self.running.load(Ordering::SeqCst) {
            interval_timer.tick().await;
            if let Err(e) = self.context.event_bus.publish(event.clone()).await {
                debug!("Tick published: {:?}", e);
                self.set_status(NativeFeatureStatus::Error {
                    message: format!("Tick publication failed: {}", e),
                })
                .await;

                let _ = self
                    .emit_failure(format!("Failed to publish tick: {}", e).as_str())
                    .await;

                return Ok(());
            }
        }
        Ok(())
    }
}

#[async_trait]
impl NativeFeature for Ticker {
    fn feature_type(&self) -> NativeFeatureType {
        NativeFeatureType::Ticker
    }

    async fn status(&self) -> NativeFeatureStatus {
        self.status.read().await.clone()
    }

    fn publish(&self, event: Event) -> FeatureResult<()> {
        self.context
            .event_bus
            .sync_publish(event)
            .map_err(FeatureError::from)
    }

    async fn start(&self) -> FeatureResult<()> {
        debug!("Ticker started: {:?}", self.config);
        if self.status().await == NativeFeatureStatus::Active {
            debug!("Ticker already started: {:?}", self.config);
            return Ok(());
        }
        self.running.store(true, Ordering::SeqCst);
        // validate config
        if self.config.tick_interval.as_millis() == 0 {
            let message = "Tick interval must be greater than 0".to_string();
            self.set_status(NativeFeatureStatus::Error {
                message: message.clone(),
            })
            .await;
            self.emit_failure(&message)
                .await
                .map_err(|e| FeatureError::StartError {
                    feature: self.feature_type(),
                    message: format!("Failed to emit failure: {:?}", e),
                })?;
            return Ok(());
        }
        let interval_timer = tokio::time::interval(self.config.tick_interval);

        let event = Event {
            event_type: event_registry::EventType::Tick,
            parameters: {
                let mut param = HashMap::new();
                param.insert(
                    "sender".to_string(),
                    event_bus::Value::String("NativeFeature::Ticker".to_string()),
                );
                param.insert(
                    "interval".to_string(),
                    event_bus::Value::Duration(self.config.tick_interval),
                );
                param
            },
        };

        let self_clone = self.clone();
        tokio::spawn(async move {
            let _ = self_clone.tick(interval_timer, event).await;
            self_clone.set_status(NativeFeatureStatus::Inactive).await;
        });
        self.set_status(NativeFeatureStatus::Active).await;
        Ok(())
    }

    async fn stop(&self) -> FeatureResult<()> {
        debug!("Ticker stopping");
        self.running.store(false, Ordering::SeqCst);
        self.set_status(NativeFeatureStatus::Inactive).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{event_bus::EventBus, event_registry::EventType};
    use tokio::time::Duration;
    use tokio::time::sleep;

    // テスト用のセットアップ関数
    async fn setup_test_context() -> Arc<NativeFeatureContext> {
        let event_bus = Arc::new(EventBus::new(100));
        Arc::new(NativeFeatureContext { event_bus })
    }

    #[tokio::test]
    async fn test_ticker_initialization() {
        let context = setup_test_context().await;
        let config = TickerConfig::default();
        let ticker = Ticker::new(context.clone(), config);

        assert_eq!(ticker.status().await, NativeFeatureStatus::Inactive);
        assert!(!ticker.running.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_ticker_start_stop() {
        let context = setup_test_context().await;
        let config = TickerConfig::default();
        let ticker = Arc::new(Ticker::new(context.clone(), config));

        // 非同期タスクでTickerを開始
        let ticker_clone = ticker.clone();
        let task = tokio::spawn(async move {
            ticker_clone.start().await.unwrap();
        });

        // Tickerが開始されるのを待つ
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(ticker.status().await, NativeFeatureStatus::Active);
        assert!(ticker.running.load(Ordering::SeqCst));

        ticker.stop().await.unwrap();

        // Tickerが停止するまで待機
        let _ = task.await;

        assert_eq!(ticker.status().await, NativeFeatureStatus::Inactive);
        assert!(!ticker.running.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_ticker_publish_event() {
        let context = setup_test_context().await;
        let config = TickerConfig {
            enabled: true,
            tick_interval: Duration::from_millis(50), // テスト用に短い間隔に設定
        };
        let ticker = Arc::new(Ticker::new(context.clone(), config));

        // EventBusをサブスクライブして、Tickイベントを受信
        let (mut event_receiver, _) = context.event_bus.subscribe();

        // 非同期タスクでTickerを開始
        let ticker_clone = ticker.clone();
        ticker_clone.start().await.unwrap();

        // Tickイベントの受信を待機
        // StatusUpdatedイベントも受信するが、ここではTickイベントのみを検証
        tokio::time::timeout(Duration::from_millis(100), event_receiver.recv())
            .await
            .unwrap()
            .unwrap();
        let received_event =
            tokio::time::timeout(Duration::from_millis(100), event_receiver.recv())
                .await
                .unwrap()
                .unwrap();

        ticker.stop().await.unwrap();

        sleep(Duration::from_millis(10)).await;

        // 受信したイベントの検証
        assert_eq!(received_event.event_type, EventType::Tick);
        assert_eq!(
            received_event.parameters.get("sender"),
            Some(&event_bus::Value::String(
                "NativeFeature::Ticker".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn test_ticker_emit_status() {
        let context = setup_test_context().await;
        let config = TickerConfig::default();
        let ticker = Arc::new(Ticker::new(context.clone(), config));

        // EventBusをサブスクライブして、FeatureStatusイベントを受信
        let (mut status_receiver, _) = context.event_bus.subscribe();

        let received_statuses = Arc::new(RwLock::new(Vec::new()));
        let received_statuses_clone = received_statuses.clone();
        tokio::spawn(async move {
            loop {
                let received_event = status_receiver.recv().await.unwrap();
                if let EventType::FeatureStatusUpdated { .. } = received_event.event_type {
                    if let Some(event_bus::Value::String(s)) =
                        received_event.parameters.get("new_status")
                    {
                        received_statuses_clone.write().await.push(s.clone());
                    }
                }
            }
        });

        // Tickerを開始
        ticker.start().await.unwrap();

        sleep(Duration::from_millis(101)).await;

        // Tickerを停止
        ticker.stop().await.unwrap();

        sleep(Duration::from_millis(10)).await;

        // 受信したステータスの検証
        let received_statuses = received_statuses.read().await.clone();
        assert_eq!(received_statuses.len(), 2);
        assert_eq!(received_statuses[0], "Active");
        assert_eq!(received_statuses[1], "Inactive");
    }

    #[tokio::test]
    async fn test_ticker_error_handling() {
        let context = setup_test_context().await;
        let config = TickerConfig {
            enabled: true,
            tick_interval: Duration::from_millis(0), // 不正な値を設定
        };
        let ticker = Arc::new(Ticker::new(context.clone(), config));

        let (mut failure_receiver, _) = context.event_bus.subscribe();
        let received_event = Arc::new(RwLock::new(vec![]));
        let received_event_clone = received_event.clone();
        tokio::spawn(async move {
            loop {
                let event = failure_receiver.recv().await.unwrap();
                if let EventType::FeatureFailure { .. } = event.event_type {
                    received_event_clone.write().await.push(event);
                }
            }
        });

        ticker.start().await.unwrap();
        sleep(Duration::from_millis(10)).await;

        assert!(matches!(
            received_event.read().await[0].event_type,
            EventType::FeatureFailure { .. }
        ));
        assert!(matches!(
            ticker.status().await,
            NativeFeatureStatus::Error { .. }
        ));

        // Tickerを停止
        ticker.stop().await.unwrap();
    }
}
