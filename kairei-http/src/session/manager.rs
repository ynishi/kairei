use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result, bail};
use dashmap::DashMap;

use super::data::{SessionData, SessionDataBuilder};

pub type SessionId = String;
pub type UserId = String;
pub type SystemId = String;

pub type SessionConfig = HashMap<String, String>;

/// Manages user sessions and their associated Kairei systems
#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: Arc<DashMap<SessionId, SessionData>>,
    users: Arc<DashMap<UserId, Vec<SessionId>>>,
    _config: SessionConfig,
}

impl SessionManager {
    /// Create a new session manager with the given configuration
    pub fn new(config: SessionConfig) -> Self {
        Self {
            _config: config,
            ..Default::default()
        }
    }

    // session data creation is user matter
    pub async fn create_session(
        &self,
        user_id: &UserId,
        builder: SessionDataBuilder,
    ) -> Result<(SessionId, SystemId)> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let data = builder
            .user_id(user_id.clone())
            .system_id(session_id.clone())
            .build()
            .with_context(|| "Failed to build session data")?;
        let system_id = data.system_id.clone();
        self.sessions.insert(session_id.clone(), data);
        self.users
            .entry(user_id.clone())
            .or_default()
            .push(session_id.clone());
        Ok((session_id, system_id))
    }

    pub async fn get_session(&self, session_id: &SessionId) -> Option<SessionData> {
        self.sessions
            .get(session_id)
            .map(|data| data.value().clone())
    }

    pub async fn get_sessions(&self, user_id: &UserId) -> Vec<(SessionId, SessionData)> {
        let session_ids = self.users.get(user_id).map(|sessions| sessions.clone());
        session_ids
            .map(|ids| {
                ids.into_iter()
                    .filter_map(|id| {
                        self.sessions
                            .get(&id)
                            .map(|data| (id, data.value().clone()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub async fn remove_session(&self, session_id: &SessionId) -> Result<()> {
        if let Some(data) = self.sessions.remove(session_id) {
            // remove session from users
            if let Some(mut sessions) = self.users.get_mut(&data.1.user_id) {
                sessions.retain(|id| id != session_id)
            }
            Ok(())
        } else {
            bail!("Session not found".to_string())
        }
    }

    pub async fn remove_sessions(&self, user_id: &UserId) {
        if let Some(sessions) = self.users.remove(user_id) {
            for session_id in sessions.1 {
                self.sessions.remove(&session_id);
            }
        }
    }
}

// test for session manager
#[cfg(test)]
mod tests {
    use super::*;
    use kairei_core::{
        config::{SecretConfig, SystemConfig},
        system::System,
    };
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_session_manager() {
        let manager = SessionManager::default();
        let user_id = "test_user".to_string();
        let system_config = SystemConfig::default();
        let secret_config = SecretConfig::default();
        let system = Arc::new(RwLock::new(
            System::new(&system_config, &secret_config).await,
        ));

        let session_data_builder = SessionDataBuilder::new()
            .system_config(system_config.clone())
            .secret_config(secret_config.clone())
            .system(system.clone());

        let (session_id, _) = manager
            .create_session(&user_id, session_data_builder.clone())
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(
            format!("{:?}", session.system_config),
            format!("{:?}", system_config.clone())
        );
        assert_eq!(
            format!("{:?}", SecretConfig::from(session.secret_config)),
            format!("{:?}", &secret_config.clone())
        );
        assert_eq!(session.user_id, user_id);

        let sessions = manager.get_sessions(&user_id).await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].0, session_id);
        assert_eq!(
            format!("{:?}", sessions[0].1.system_config),
            format!("{:?}", system_config)
        );
        assert_eq!(
            format!(
                "{:?}",
                SecretConfig::from(sessions[0].1.secret_config.clone())
            ),
            format!("{:?}", secret_config)
        );

        manager.remove_session(&session_id).await.unwrap();
        assert!(manager.get_session(&session_id).await.is_none());

        manager.remove_sessions(&user_id).await;
        assert!(manager.get_sessions(&user_id).await.is_empty());
    }
}
