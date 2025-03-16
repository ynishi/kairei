use crate::auth::AuthStore;
use crate::models::user::User;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Axum middleware for API key authentication
pub async fn auth_middleware(
    State(auth_store): State<Arc<AuthStore>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    println!(
        "auth_middleware, request.uri().path(): {:?}",
        request.uri().path()
    );
    let path = request.uri().path();
    if ignore_auth_path(path) {
        println!("auth_middleware, ignore_auth_path, path: {:?}", path);
        return Ok(next.run(request).await);
    }
    // Extract API key from headers
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Look up user by API key
    let user = auth_store
        .get_user_by_api_key(api_key)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Add user to request extensions
    request.extensions_mut().insert(user);

    // Continue with the request
    Ok(next.run(request).await)
}

fn ignore_auth_path(path: &str) -> bool {
    path.starts_with("/health")
        || is_swagger_path(path)
        || is_api_docs_path(path)
        || is_docs_path(path)
}

pub fn is_health_path(path: &str) -> bool {
    path.starts_with("/health")
}

pub fn is_swagger_path(path: &str) -> bool {
    path.starts_with("/swagger-ui")
}

pub fn is_api_docs_path(path: &str) -> bool {
    path.starts_with("/api-docs")
}

pub fn is_docs_path(path: &str) -> bool {
    path.starts_with("/api/v1/docs")
}

/// Extension trait for Request to easily extract the authenticated user
pub trait AuthExt {
    /// Get the authenticated user from the request
    fn user(&self) -> Option<&User>;
}

impl AuthExt for Request {
    fn user(&self) -> Option<&User> {
        self.extensions().get::<User>()
    }
}
