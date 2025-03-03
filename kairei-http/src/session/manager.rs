use std::sync::Arc;

use dashmap::DashMap;
use kairei_core::{
    config::{SecretConfig, SystemConfig},
    system::System,
};
use tokio::sync::RwLock;

pub type SessionId = String;
pub type UserId = String;

/// Composite key for user sessions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserSessionKey {
    /// User ID
    pub user_id: UserId,
    /// Session ID
    pub session_id: SessionId,
}

impl UserSessionKey {
    /// Create a new user session key
    pub fn new(user_id: impl Into<UserId>, session_id: impl Into<SessionId>) -> Self {
        Self {
            user_id: user_id.into(),
            session_id: session_id.into(),
        }
    }
}

/// Configuration for the session manager
#[derive(Clone, Default)]
pub struct SessionConfig {
    pub system_config: SystemConfig,
    pub secret_config: SecretConfig,
}

/// Manages user sessions and their associated Kairei systems
#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: Arc<DashMap<UserSessionKey, Arc<RwLock<System>>>>,
    config: SessionConfig,
}

impl SessionManager {
    /// Create a new session manager with the given configuration
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Get or create a session for the given user and session ID
    pub async fn get_or_create_user_session(
        &self,
        user_id: &UserId,
        session_id: &SessionId,
    ) -> Arc<RwLock<System>> {
        let key = UserSessionKey::new(user_id, session_id);

        if let Some(system) = self.sessions.get(&key) {
            return system.clone();
        }

        // Create new system for session
        let system = Arc::new(RwLock::new(
            System::new(&self.config.system_config, &self.config.secret_config).await,
        ));
        self.sessions.insert(key, system.clone());
        system
    }

    /// Get or create a session for the given session ID (for backward compatibility)
    pub async fn get_or_create_session(&self, session_id: &SessionId) -> Arc<RwLock<System>> {
        // Use a default user ID for backward compatibility
        self.get_or_create_user_session(&"default".to_string(), session_id)
            .await
    }

    /// Remove a user session
    pub async fn remove_user_session(&self, user_id: &UserId, session_id: &SessionId) {
        let key = UserSessionKey::new(user_id, session_id);
        self.sessions.remove(&key);
    }

    /// Remove a session (for backward compatibility)
    pub async fn remove_session(&self, session_id: &SessionId) {
        self.remove_user_session(&"default".to_string(), session_id)
            .await;
    }

    /// Get all sessions for a user
    pub fn get_user_sessions(&self, user_id: &UserId) -> Vec<(SessionId, Arc<RwLock<System>>)> {
        self.sessions
            .iter()
            .filter(|entry| entry.key().user_id == *user_id)
            .map(|entry| (entry.key().session_id.clone(), entry.value().clone()))
            .collect()
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get the number of active sessions for a user
    pub fn user_session_count(&self, user_id: &UserId) -> usize {
        self.sessions
            .iter()
            .filter(|entry| entry.key().user_id == *user_id)
            .count()
    }
}
