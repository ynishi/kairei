use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Instant,
};

use async_trait::async_trait;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::{
    config::MetricsConfig,
    event_bus::{Event, Value},
    event_registry::EventType,
};

use super::types::{
    FeatureError, FeatureResult, NativeFeature, NativeFeatureContext, NativeFeatureStatus,
    NativeFeatureType,
};

pub struct MetricsFeature {
    context: Arc<NativeFeatureContext>,
    metrics_store: Arc<RwLock<MetricsStore>>,
    status: Arc<RwLock<NativeFeatureStatus>>,
    running: Arc<AtomicBool>,
    tick_count: Arc<AtomicUsize>, // Tickカウント用
    publish_interval: usize,      // 何Tick毎に公開するか
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MetricsStore {
    request_metrics: HashMap<String, RequestMetrics>,
    response_metrics: HashMap<String, ResponseMetrics>,
    llm_metrics: HashMap<String, LLMMetrics>,
}

#[derive(Debug, Clone)]
pub struct RequestMetrics {
    start_time: Instant,
    agent_id: String,
    request_type: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ResponseMetrics {
    end_time: Instant,
    execution_time: Duration,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LLMMetrics {
    calls: usize,
    tokens: usize,
}

#[async_trait]
impl NativeFeature for MetricsFeature {
    fn feature_type(&self) -> NativeFeatureType {
        NativeFeatureType::Metrics
    }

    async fn status(&self) -> NativeFeatureStatus {
        self.status.read().await.clone()
    }

    fn publish(&self, event: Event) -> FeatureResult<()> {
        self.context
            .event_bus()
            .sync_publish(event)
            .map_err(FeatureError::from)
    }

    async fn start(&self) -> FeatureResult<()> {
        self.running.store(true, Ordering::SeqCst);

        let event_bus = self.context.event_bus();
        let metrics_store = self.metrics_store.clone();
        let running = self.running.clone();

        let tick_count = self.tick_count.clone();
        let publish_interval = self.publish_interval;
        let context = self.context.clone();

        tokio::spawn(async move {
            let (mut sub, _) = event_bus.subscribe();
            while running.load(Ordering::SeqCst) {
                if let Ok(event) = sub.recv().await {
                    match event.event_type {
                        EventType::Request {
                            request_id,
                            request_type,
                            requester,
                            ..
                        } => {
                            let metrics = RequestMetrics {
                                start_time: Instant::now(),
                                agent_id: requester,
                                request_type,
                            };
                            metrics_store
                                .write()
                                .await
                                .request_metrics
                                .insert(request_id, metrics);
                        }
                        EventType::ResponseSuccess { request_id, .. }
                        | EventType::ResponseFailure { request_id, .. } => {
                            let now = Instant::now();
                            let mut store = metrics_store.write().await;

                            if let Some(req_metrics) = store.request_metrics.get(&request_id) {
                                let resp_metrics = ResponseMetrics {
                                    end_time: now,
                                    execution_time: now.duration_since(req_metrics.start_time),
                                };
                                store.response_metrics.insert(request_id, resp_metrics);
                            }
                        }
                        EventType::Tick => {
                            let count = tick_count.fetch_add(1, Ordering::SeqCst);
                            if count % publish_interval == 0 {
                                // メトリクスの公開
                                let _ =
                                    Self::publish_metrics(metrics_store.clone(), context.clone())
                                        .await;
                            }
                        }
                        // 他のイベントタイプへの対応は後で追加
                        _ => {}
                    }
                }
            }
        });

        *self.status.write().await = NativeFeatureStatus::Active;
        self.emit_status().await
    }

    async fn stop(&self) -> FeatureResult<()> {
        self.running.store(false, Ordering::SeqCst);

        // 停止前にサマリーを生成して表示
        self.publish_metrics_summary().await?;

        *self.status.write().await = NativeFeatureStatus::Inactive;
        self.emit_status().await
    }
}

impl MetricsFeature {
    pub fn new(context: Arc<NativeFeatureContext>, config: MetricsConfig) -> Self {
        Self {
            context,
            metrics_store: Arc::new(RwLock::new(MetricsStore {
                request_metrics: HashMap::new(),
                response_metrics: HashMap::new(),
                llm_metrics: HashMap::new(),
            })),
            status: Arc::new(RwLock::new(NativeFeatureStatus::Inactive)),
            running: Arc::new(AtomicBool::new(false)),
            tick_count: Arc::new(AtomicUsize::new(0)),
            publish_interval: config.metrics_interval,
        }
    }

    // メトリクス取得用のヘルパーメソッド
    pub async fn get_request_metrics(&self, request_id: &str) -> Option<RequestMetrics> {
        self.metrics_store
            .read()
            .await
            .request_metrics
            .get(request_id)
            .cloned()
    }

    pub async fn get_response_metrics(&self, request_id: &str) -> Option<ResponseMetrics> {
        self.metrics_store
            .read()
            .await
            .response_metrics
            .get(request_id)
            .cloned()
    }
}

impl MetricsFeature {
    // サマリーの生成と表示を行うメソッド
    async fn publish_metrics_summary(&self) -> FeatureResult<()> {
        let metrics_store = self.metrics_store.clone();
        let context = self.context.clone();
        Self::publish_metrics(metrics_store, context).await
    }

    async fn publish_metrics(
        metrics_store: Arc<RwLock<MetricsStore>>,
        context: Arc<NativeFeatureContext>,
    ) -> FeatureResult<()> {
        let store = metrics_store.read().await;
        let summary = MetricsSummary {
            total_requests: store.request_metrics.len(),
            total_responses: store.response_metrics.len(),

            // 実行時間の統計
            execution_times: store
                .response_metrics
                .values()
                .map(|m| m.execution_time)
                .collect::<Vec<_>>(),

            // リクエストタイプごとの集計
            request_types: store.request_metrics.values().fold(
                HashMap::new(),
                |mut acc, metrics| {
                    *acc.entry(metrics.request_type.clone()).or_insert(0) += 1;
                    acc
                },
            ),

            // エージェントごとの集計
            agent_metrics: store.request_metrics.values().fold(
                HashMap::new(),
                |mut acc, metrics| {
                    *acc.entry(metrics.agent_id.clone()).or_insert(0) += 1;
                    acc
                },
            ),
        };

        // サマリーイベントの発行
        context
            .event_bus
            .publish(Event {
                event_type: EventType::MetricsSummary,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "total_requests".to_string(),
                        Value::Float(summary.total_requests as f64),
                    );
                    params.insert(
                        "total_responses".to_string(),
                        Value::Float(summary.total_responses as f64),
                    );
                    params.insert(
                        "avg_execution_time_ms".to_string(),
                        Value::Float(summary.average_execution_time().as_millis() as f64),
                    );
                    params.insert(
                        "request_types".to_string(),
                        Value::String(format!("{:?}", summary.request_types)),
                    );
                    params.insert(
                        "agent_metrics".to_string(),
                        Value::String(format!("{:?}", summary.agent_metrics)),
                    );
                    params
                },
            })
            .await
            .map_err(FeatureError::from)?;

        Ok(())
    }
}

#[derive(Debug)]
struct MetricsSummary {
    total_requests: usize,
    total_responses: usize,
    execution_times: Vec<Duration>,
    request_types: HashMap<String, usize>,
    agent_metrics: HashMap<String, usize>,
}

impl MetricsSummary {
    fn average_execution_time(&self) -> Duration {
        if self.execution_times.is_empty() {
            return Duration::from_secs(0);
        }

        let total = self.execution_times.iter().sum::<Duration>();
        total / self.execution_times.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{event_bus::EventBus, event_registry::EventType};
    use tokio::time::Duration;
    use tokio::time::sleep;
    use uuid::Uuid;

    async fn setup_test_context() -> Arc<NativeFeatureContext> {
        let event_bus = Arc::new(EventBus::new(100));
        Arc::new(NativeFeatureContext { event_bus })
    }

    #[tokio::test]
    async fn test_metrics_initialization() {
        let context = setup_test_context().await;
        let config = MetricsConfig::default();
        let metrics = MetricsFeature::new(context, config);

        assert_eq!(metrics.status().await, NativeFeatureStatus::Inactive);
        assert!(!metrics.running.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_metrics_start_stop() {
        let context = setup_test_context().await;
        let config = MetricsConfig::default();
        let metrics = MetricsFeature::new(context, config);

        // メトリクス収集を開始
        metrics.start().await.unwrap();
        assert_eq!(metrics.status().await, NativeFeatureStatus::Active);
        assert!(metrics.running.load(Ordering::SeqCst));

        // 停止
        metrics.stop().await.unwrap();
        assert_eq!(metrics.status().await, NativeFeatureStatus::Inactive);
        assert!(!metrics.running.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let context = setup_test_context().await;
        let config = MetricsConfig::default();
        let metrics = MetricsFeature::new(context.clone(), config);
        let event_bus = context.event_bus();

        // メトリクス収集を開始
        metrics.start().await.unwrap();

        sleep(Duration::from_millis(10)).await;

        // テスト用のリクエスト/レスポンスイベントを生成
        let request_id = Uuid::new_v4().to_string();
        let test_agent_id = "test_agent";
        let responder = "test_responder";
        let test_request_type = "test_request";

        // リクエストイベントの発行
        event_bus
            .publish(Event {
                event_type: EventType::Request {
                    request_id: request_id.clone(),
                    request_type: test_request_type.to_string(),
                    requester: test_agent_id.to_string(),
                    responder: responder.to_string(),
                },
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "agent_id".to_string(),
                        Value::String(test_agent_id.to_string()),
                    );
                    params.insert(
                        test_request_type.to_string(),
                        Value::String(test_request_type.to_string()),
                    );
                    params
                },
            })
            .await
            .unwrap();

        sleep(Duration::from_millis(20)).await;

        // レスポンスイベントの発行
        event_bus
            .sync_publish(Event {
                event_type: EventType::ResponseSuccess {
                    request_type: test_request_type.to_string(),
                    requester: test_request_type.to_string(),
                    responder: responder.to_string(),
                    request_id: request_id.clone(),
                },
                parameters: HashMap::new(),
            })
            .unwrap();

        sleep(Duration::from_millis(20)).await;
        // メトリクスの検証 - 非同期でロックを取得

        let store = metrics.metrics_store.read().await.clone();
        let request_metrics = store.request_metrics.get(&request_id).unwrap().clone();
        assert_eq!(request_metrics.agent_id, test_agent_id);
        assert_eq!(request_metrics.request_type, "test_request");
        let response_metrics = store.response_metrics.get(&request_id).unwrap().clone();
        assert!(response_metrics.execution_time.as_millis() >= 10);
    }

    #[tokio::test]
    async fn test_metrics_status_events() {
        let context = setup_test_context().await;
        let config = MetricsConfig::default();
        let metrics = MetricsFeature::new(context.clone(), config);

        let (mut status_receiver, _) = context.event_bus.subscribe();
        let received_statuses = Arc::new(RwLock::new(Vec::new()));
        let received_statuses_clone = received_statuses.clone();

        tokio::spawn(async move {
            while let Ok(event) = status_receiver.recv().await {
                if let EventType::FeatureStatusUpdated { .. } = event.event_type {
                    if let Some(Value::String(s)) = event.parameters.get("new_status") {
                        received_statuses_clone.write().await.push(s.clone());
                    }
                }
            }
        });

        metrics.init().await.unwrap();
        metrics.start().await.unwrap();
        metrics.stop().await.unwrap();

        sleep(Duration::from_millis(10)).await;

        let statuses = received_statuses.read().await;
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0], "Active");
        assert_eq!(statuses[1], "Inactive");
    }

    #[tokio::test]
    async fn test_metrics_periodic_publishing() {
        let context = setup_test_context().await;
        let config = MetricsConfig {
            metrics_interval: 5,
            enabled: true,
        };

        let metrics = Arc::new(MetricsFeature::new(context.clone(), config));
        let event_bus = context.event_bus();

        let metrics_wait = {
            let (mut metrics_receiver, _) = context.event_bus.subscribe();
            tokio::spawn(async move {
                let mut found_metrics_update = false;
                while let Ok(event) =
                    tokio::time::timeout(Duration::from_millis(100), metrics_receiver.recv()).await
                {
                    if let EventType::MetricsSummary = event.unwrap().event_type {
                        found_metrics_update = true;
                        break;
                    }
                }
                found_metrics_update
            })
        };

        // メトリクス収集を開始
        metrics.start().await.unwrap();

        sleep(Duration::from_millis(10)).await;

        // Tickイベントを60回送信
        for _ in 0..60 {
            event_bus
                .sync_publish(Event {
                    event_type: EventType::Tick,
                    parameters: HashMap::new(),
                })
                .unwrap();
        }

        let found_metrics_update = metrics_wait.await.unwrap();

        assert!(found_metrics_update);

        metrics.stop().await.unwrap();
    }
}
