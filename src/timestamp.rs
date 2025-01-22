use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Timestamp(SystemTime);

impl Timestamp {
    pub fn now() -> Self {
        Self(SystemTime::now())
    }

    pub fn into_inner(self) -> SystemTime {
        self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl From<SystemTime> for Timestamp {
    fn from(time: SystemTime) -> Self {
        Self(time)
    }
}

impl From<Timestamp> for SystemTime {
    fn from(timestamp: Timestamp) -> Self {
        timestamp.0
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.elapsed().unwrap().as_secs())
    }
}

// 必要に応じてDerefも実装可能
impl std::ops::Deref for Timestamp {
    type Target = SystemTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::sleep;
    use tracing::debug;

    use super::*;

    #[test]
    fn test_timestamp_default() {
        let timestamp = Timestamp::default();
        assert!(timestamp.0.elapsed().unwrap().as_secs() < 1);
    }

    #[test]
    fn test_timestamp_now() {
        let timestamp = Timestamp::now();
        assert!(timestamp.0.elapsed().unwrap().as_secs() < 1);
    }

    #[test]
    fn test_timestamp_into_inner() {
        let timestamp = Timestamp::now();
        let system_time = timestamp.into_inner();
        assert!(system_time.elapsed().unwrap().as_secs() < 1);
    }

    #[test]
    fn test_timestamp_from_system_time() {
        let system_time = SystemTime::now();
        let timestamp = Timestamp::from(system_time);
        assert!(timestamp.0.elapsed().unwrap().as_secs() < 1);
    }

    #[test]
    fn test_timestamp_from_timestamp() {
        let timestamp = Timestamp::now();
        let timestamp2 = Timestamp::from(timestamp.clone());
        assert_eq!(timestamp.0, timestamp2.0);
    }

    #[test]
    fn test_timestamp_display() {
        let timestamp = Timestamp::now();
        let display = format!("{}", timestamp);
        assert!(display.parse::<u64>().is_ok());
    }

    #[test]
    fn test_timestamp_deref() {
        let timestamp = Timestamp::now();
        let system_time = *timestamp;
        assert!(system_time.elapsed().unwrap().as_secs() < 1);
    }

    #[test]
    fn test_timestamp_deref_eq() {
        let timestamp = Timestamp::now();
        let system_time = *timestamp;
        assert_eq!(timestamp.0, system_time);
    }

    #[tokio::test]
    async fn test_timestamp_deref_ne() {
        let timestamp = Timestamp::now();
        sleep(Duration::from_millis(10)).await;
        let system_time = SystemTime::now();
        assert_ne!(timestamp.0, system_time);
    }

    #[test]
    fn test_timestamp_serialize() {
        let timestamp = Timestamp::now();
        let serialized = serde_json::to_string(&timestamp).unwrap();
        debug!("{}", serialized);
        assert!(serialized.contains("secs_since_epoch"));
        assert!(serialized.contains("nanos_since_epoch"));
    }

    #[test]
    fn test_timestamp_deserialize() {
        let timestamp = Timestamp::now();
        let serialized = serde_json::to_string(&timestamp).unwrap();
        let deserialized: Timestamp = serde_json::from_str(&serialized).unwrap();
        assert_eq!(timestamp.0, deserialized.0);
    }
}
