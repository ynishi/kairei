use std::sync::Arc;

use dashmap::DashMap;
use kairei_core::{
    config::{SecretConfig, SystemConfig},
    system::System,
};
use tokio::sync::RwLock;

pub type SessionId = String;

/// Configuration for the session manager
#[derive(Clone, Default)]
pub struct SessionConfig {
    pub system_config: SystemConfig,
    pub secret_config: SecretConfig,
}

/// Manages user sessions and their associated Kairei systems
#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: Arc<DashMap<SessionId, Arc<RwLock<System>>>>,
    config: SessionConfig,
}

impl SessionManager {
    /// Create a new session manager with the given configuration
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Get or create a session for the given session ID
    pub async fn get_or_create_session(&self, session_id: &SessionId) -> Arc<RwLock<System>> {
        if let Some(system) = self.sessions.get(session_id) {
            return system.clone();
        }

        // Create new system for session
        let system = Arc::new(RwLock::new(
            System::new(&self.config.system_config, &self.config.secret_config).await,
        ));
        self.sessions.insert(session_id.clone(), system.clone());
        system
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &SessionId) {
        self.sessions.remove(session_id);
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}
