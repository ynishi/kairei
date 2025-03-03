use crate::auth::AuthStore;
use crate::models::user::User;
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Axum middleware for API key authentication
pub async fn auth_middleware(
    auth_store: Arc<AuthStore>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract API key from headers
    let api_key = headers
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
