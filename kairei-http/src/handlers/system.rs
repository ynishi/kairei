use std::collections::HashMap;
use std::sync::Arc;

use crate::auth::{AuthAdmin, AuthUser};
use crate::models::{CreateSystemRequest, CreateSystemResponse, ListSystemsResponse};
use crate::server::AppState;
use crate::session::data::SessionDataBuilder;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{extract::State, response::Json};
use kairei_core::Root;
use kairei_core::config::ProviderSecretConfig;
use kairei_core::system::{System, SystemStatus};
use tokio::sync::RwLock;

/// Create the system
#[axum::debug_handler]
pub async fn create_system(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateSystemRequest>,
) -> Result<Json<CreateSystemResponse>, StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    // todo extract secret from user
    let mut secret = kairei_core::config::SecretConfig::default();
    secret.providers.insert(
        "default_provider".to_string(),
        ProviderSecretConfig {
            ..Default::default()
        },
    );

    let config = payload.config.clone();

    // impl create system using kairei-core with the session manager
    let system: System = System::new(&config, &secret).await;

    let session_data_builder = SessionDataBuilder::new()
        .system_config(config)
        .secret_config(secret)
        .system(Arc::new(RwLock::new(system)));

    let (session_id, system_id) = state
        .session_manager
        .create_session(&auth.user().user_id, session_data_builder)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create system: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(CreateSystemResponse {
        system_id,
        session_id,
    }))
}

/// Get system information
///
/// Returns information about the current state of the system.
/// Requires authentication with admin role.
#[axum::debug_handler]
pub async fn get_system(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Path(system_id): Path<String>,
) -> Result<Json<SystemStatus>, StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    // get system from session manager
    if let Some(data) = state.session_manager.get_session(&system_id).await {
        let system = data.system.read().await;
        let status = system.get_system_status().await.map_err(|e| {
            tracing::error!("Failed to get system status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        drop(system);
        Ok(Json(status))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// List systems
#[axum::debug_handler]
pub async fn list_systems(
    State(state): State<AppState>,
    auth: AuthAdmin,
) -> Result<Json<ListSystemsResponse>, StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    // In a real implementation, this would list all available systems
    // using kairei-core with the session manager. For now, we'll return mock data.
    let sessions = state
        .session_manager
        .get_sessions(&auth.user().user_id)
        .await;
    let mut system_statuses = HashMap::new();
    for session in sessions {
        let system = session.1.system.read().await;
        let status = system.get_system_status().await.map_err(|e| {
            tracing::error!("Failed to get system status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        drop(system);
        system_statuses.insert(session.0.clone(), status);
    }
    Ok(Json(ListSystemsResponse { system_statuses }))
}

// Start the system
#[axum::debug_handler]
pub async fn start_system(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Path(system_id): Path<String>,
) -> Result<(), StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    if let Some(data) = state.session_manager.get_session(&system_id).await {
        let mut system = data.system.write().await;
        system
            .initialize(Root {
                world_def: None,
                micro_agent_defs: vec![],
            })
            .await
            .map_err(|e| {
                tracing::error!("Failed to initialize system: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        system.start().await.map_err(|e| {
            tracing::error!("Failed to start system: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Delete the system
#[axum::debug_handler]
pub async fn delete_system(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Path(system_id): Path<String>,
) -> Result<(), StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    if state
        .session_manager
        .get_session(&system_id)
        .await
        .is_none()
    {
        return Err(StatusCode::NOT_FOUND);
    }

    state
        .session_manager
        .remove_session(&system_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to remove system: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
