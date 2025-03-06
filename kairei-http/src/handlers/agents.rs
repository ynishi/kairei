use core::panic;

use crate::auth::{AuthAdmin, AuthUser};
use crate::models::{
    GetAgentResponse, ListAgentsResponse, ScaleDownAgentRequest, ScaleUpAgentRequest,
    SendRequestAgentRequest, SendRequestAgentResponse,
};
use crate::server::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use kairei_core::event_bus;

/// Get agent details
///
/// Returns details about a specific agent.
/// Requires authentication.
#[axum::debug_handler]
pub async fn get_agent(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((system_id, agent_id)): Path<(String, String)>,
) -> Result<Json<GetAgentResponse>, StatusCode> {
    let user = auth.user();
    let session = state
        .session_manager
        .get_session(&system_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    if user.user_id != session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let system = session.system.read().await;
    let status = system.get_agent_status(&agent_id).await.map_err(|e| {
        tracing::error!("Failed to get agent details: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(GetAgentResponse { agent_id, status }))
}

/// List agents
#[axum::debug_handler]
pub async fn list_agents(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(system_id): Path<String>,
) -> Result<Json<ListAgentsResponse>, StatusCode> {
    let user = auth.user();
    let session = state
        .session_manager
        .get_session(&system_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    if user.user_id != session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let system = session.system.read().await;
    let responses: Vec<GetAgentResponse> = system
        .list_agents()
        .await
        .map_err(|e| {
            tracing::error!("Failed to list agents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .iter()
        .map(|status| GetAgentResponse {
            agent_id: status.name.clone(),
            status: status.clone(),
        })
        .collect();

    Ok(Json(ListAgentsResponse { agents: responses }))
}

/// Start agent
#[axum::debug_handler]
pub async fn start_agent(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Path((system_id, agent_id)): Path<(String, String)>,
) -> Result<(), StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    let session = state
        .session_manager
        .get_session(&system_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let system = session.system.write().await;
    system.start_agent(&agent_id).await.map_err(|e| {
        tracing::error!("Failed to start agent: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}

/// Shutdown agent
#[axum::debug_handler]
pub async fn stop_agent(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Path((system_id, agent_id)): Path<(String, String)>,
) -> Result<(), StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    let session = state
        .session_manager
        .get_session(&system_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let system = session.system.write().await;
    system.stop_agent(&agent_id).await.map_err(|e| {
        tracing::error!("Failed to shutdown agent: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}

/// Scale up agent
#[axum::debug_handler]
pub async fn scale_up_agent(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Path((system_id, agent_id)): Path<(String, String)>,
    Json(payload): Json<ScaleUpAgentRequest>,
) -> Result<(), StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    let session = state
        .session_manager
        .get_session(&system_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let system = session.system.write().await;
    let metadata = payload
        .options
        .iter()
        .map(|(k, v)| (k.to_string(), event_bus::Value::from(v.to_string())))
        .collect();
    system
        .scale_up(&agent_id, payload.instances, metadata)
        .await
        .map_err(|e| {
            tracing::error!("Failed to scale up agent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}

/// Scale down agent
#[axum::debug_handler]
pub async fn scale_down_agent(
    State(state): State<AppState>,
    auth: AuthAdmin,
    Path((system_id, agent_id)): Path<(String, String)>,
    Json(payload): Json<ScaleDownAgentRequest>,
) -> Result<(), StatusCode> {
    if !auth.user().is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    let session = state
        .session_manager
        .get_session(&system_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let system = session.system.write().await;
    let metadata = payload
        .options
        .iter()
        .map(|(k, v)| (k.to_string(), event_bus::Value::from(v.to_string())))
        .collect();
    system
        .scale_down(&agent_id, payload.instances, metadata)
        .await
        .map_err(|e| {
            tracing::error!("Failed to scale down agent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}

/// Request agent
#[axum::debug_handler]
pub async fn request_agent(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((system_id, agent_id)): Path<(String, String)>,
    Json(payload): Json<SendRequestAgentRequest>,
) -> Result<Json<SendRequestAgentResponse>, StatusCode> {
    let user = auth.user();
    let session = state
        .session_manager
        .get_session(&system_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    if user.user_id != session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let system_clone = session.system.clone();
    drop(session);

    let request_id = uuid::Uuid::new_v4();
    let request = event_bus::Event::request_builder()
        .request_type(&payload.request_type)
        .requester(&user.user_id)
        .responder(&agent_id)
        .request_id(&request_id.to_string())
        .build()
        .map_err(|e| {
            tracing::error!("Failed to build request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let request_clone = request.clone();

    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let system = system_clone.read().await;
        match system.send_request(request_clone).await {
            Ok(result) => {
                // 成功時の処理
                tracing::info!("Request succeeded: {:?}", result);
                let _ = tx.send(result);
            }
            Err(e) => {
                // エラー時の処理
                tracing::error!("Failed to request agent: {}", e);
            }
        }
    });

    let value = match rx.await {
        Ok(result) => serde_json::Value::from(&result),
        Err(_) => {
            tracing::error!("Failed to receive response from task");
            serde_json::Value::Null
        }
    };

    Ok(Json(SendRequestAgentResponse { value }))
}
