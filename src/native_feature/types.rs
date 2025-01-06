use std::{collections::HashMap, sync::Arc};

use crate::{
    event_bus::{self, Event},
    event_registry::{self, EventType},
    RuntimeError,
};
use async_trait::async_trait;
use thiserror::Error;

use crate::{event_bus::EventBus, RuntimeResult};

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, strum::EnumString, strum::Display, PartialOrd, Ord, Default,
)]
pub enum NativeFeatureType {
    #[default]
    Ticker,
    ResourceMonitor,
}

#[derive(Debug, Clone, strum::Display, PartialEq)]
pub enum NativeFeatureStatus {
    Inactive,
    Active,
    Error { message: String },
}

#[derive(Debug, Error)]
pub enum FeatureError {
    #[error("Event publish failed: {message}")]
    EventPublishError { message: String },
    #[error("Feature operation failed: {operation} - {message}")]
    OperationError {
        operation: &'static str,
        message: String,
    },
    #[error("Status update failed: {message}")]
    StatusError { message: String },
}
#[async_trait]
pub trait NativeFeature: Send + Sync {
    fn feature_type(&self) -> NativeFeatureType;

    async fn status(&self) -> NativeFeatureStatus;

    fn publish(&self, event: Event) -> RuntimeResult<()>;

    async fn init(&self) -> RuntimeResult<()> {
        Ok(())
    }
    /// Start is the method to begin the main processing.
    /// This method must be non-blocking to ensure concurrent access after startup from the registry.
    /// When the main processing is completed, a FeatureStatusUpdated event must be emitted to notify the new status.
    /// Implement stop functionality by monitoring internal async variables or similar mechanisms to handle shutdown when stop is called.
    async fn start(&self) -> RuntimeResult<()>;
    async fn stop(&self) -> RuntimeResult<()>;

    // ヘルパー機能
    async fn emit_status(&self) -> RuntimeResult<()> {
        let status_event = Event {
            event_type: event_registry::EventType::FeatureStatusUpdated {
                feature_type: self.feature_type(),
            },
            parameters: {
                let mut hashmap = HashMap::new();
                hashmap.insert(
                    "new_status".to_string(),
                    event_bus::Value::String(self.status().await.to_string()),
                );
                hashmap
            },
        };
        self.publish(status_event)
    }

    async fn emit_failure(&self, message: &str) -> Result<(), RuntimeError> {
        let failure = Event {
            event_type: EventType::FeatureFailure {
                error: message.to_string(),
            },
            parameters: {
                let mut hashmap = HashMap::new();
                hashmap.insert(
                    "feature_type".to_string(),
                    event_bus::Value::String(self.feature_type().to_string()),
                );
                hashmap
            },
        };
        self.publish(failure)
    }
}

pub struct NativeFeatureContext {
    pub event_bus: Arc<EventBus>,
}

impl NativeFeatureContext {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }

    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }
}
