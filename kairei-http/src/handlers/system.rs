use std::collections::HashMap;
use std::sync::Arc;

use crate::auth::{AuthAdmin, AuthUser};
use crate::models::{
    CompileSystemRequest, CompileSystemResponse, CreateSystemRequest, CreateSystemResponse,
    ListSystemsResponse, StartSystemRequest,
};
use crate::server::AppState;
use crate::session::data::SessionDataBuilder;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{extract::State, response::Json};
use kairei_core::Root;
use kairei_core::system::{System, SystemStatus};
use tokio::sync::RwLock;

/// Create the system
#[utoipa::path(
    post,
    path = "/systems",
    request_body = CreateSystemRequest,
    responses(
        (status = 200, description = "Create system successfully", body = CreateSystemResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    )
)]
#[axum::debug_handler]
pub async fn create_system(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateSystemRequest>,
) -> Result<Json<CreateSystemResponse>, StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    let secret = state.session_manager.secret_config.clone();
    let config = payload.config.clone();

    // impl create system using kairei-core with the session manager
    let system = System::new(&config, &secret).await;

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
#[utoipa::path(
    get,
    path = "/systems/{system_id}",
    responses(
        (status = 200, description = "System retrieved successfully", body = SystemStatus),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "System not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("system_id" = String, Path, description = "System identifier")
    )
)]
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
#[utoipa::path(
    get,
    path = "/systems",
    responses(
        (status = 200, description = "Systems listed successfully", body = ListSystemsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Compile the dsl, without starting the system
///
/// Response include compilation error.
/// This is useful for validating the DSL before starting the system.
#[utoipa::path(
    post,
    path = "/systems/{system_id}/compile",
    responses(
        (status = 200, description = "DSL compiled successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "System not found"),
    ),
    params(
        ("system_id" = String, Path, description = "System identifier")
    )
)]
#[axum::debug_handler]
pub async fn compile_system(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Path(system_id): Path<String>,
    Json(payload): Json<CompileSystemRequest>,
) -> Result<Json<CompileSystemResponse>, StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    if let Some(data) = state.session_manager.get_session(&system_id).await {
        let system = data.system.write().await;
        if let Err(e) = system.parse_dsl(&payload.dsl).await {
            tracing::error!("Failed to compile DSL: {}", e);
            Ok(Json(CompileSystemResponse::failure(e)))
        } else {
            Ok(Json(CompileSystemResponse::success()))
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Start the system
///
/// This will compile the DSL if provided, and start the system.
/// If the DSL is not provided, the system will be started with the existing configuration.
#[utoipa::path(
    post,
    path = "/systems/{system_id}/start",
    request_body = StartSystemRequest,
    responses(
        (status = 200, description = "System started successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "System not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("system_id" = String, Path, description = "System identifier")
    )
)]
#[axum::debug_handler]
pub async fn start_system(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Path(system_id): Path<String>,
    Json(payload): Json<StartSystemRequest>,
) -> Result<(), StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    if let Some(data) = state.session_manager.get_session(&system_id).await {
        let mut system = data.system.write().await;
        let root_def = if let Some(dsl) = &payload.dsl {
            system.parse_dsl(dsl).await.map_err(|e| {
                tracing::error!("Failed to load DSL: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        } else {
            Root {
                world_def: None,
                micro_agent_defs: vec![],
            }
        };

        system.initialize(root_def).await.map_err(|e| {
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

/// Shutdown the system
#[utoipa::path(
    post,
    path = "/systems/{system_id}/stop",
    responses(
        (status = 200, description = "System stopped successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "System not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("system_id" = String, Path, description = "System identifier")
    )
)]
#[axum::debug_handler]
pub async fn stop_system(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Path(system_id): Path<String>,
) -> Result<(), StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    if let Some(data) = state.session_manager.get_session(&system_id).await {
        let system = data.system.write().await;
        system.emergency_shutdown().await.map_err(|e| {
            tracing::error!("Failed to initialize system: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Delete the system
#[utoipa::path(
    delete,
    path = "/systems/{system_id}",
    responses(
        (status = 200, description = "System deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "System not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("system_id" = String, Path, description = "System identifier")
    )
)]
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
