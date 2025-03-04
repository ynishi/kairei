use crate::models::user::User;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

/// Extractor for the authenticated user
///
/// This extractor gets the authenticated user from the request extensions
/// without consuming the request body.
pub struct AuthUser(pub User);

impl AuthUser {
    /// Get a reference to the inner user
    pub fn user(&self) -> &User {
        &self.0
    }

    /// Unwrap the extractor to get the inner user
    pub fn into_inner(self) -> User {
        self.0
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<User>()
            .ok_or(StatusCode::UNAUTHORIZED)?
            .clone();

        Ok(AuthUser(user))
    }
}

/// Extractor for the authenticated admin user
///
/// This extractor gets the authenticated user from the request extensions
/// and ensures that the user has admin role.
pub struct AuthAdmin(pub User);

impl AuthAdmin {
    /// Get a reference to the inner user
    pub fn user(&self) -> &User {
        &self.0
    }

    /// Unwrap the extractor to get the inner user
    pub fn into_inner(self) -> User {
        self.0
    }
}

impl<S> FromRequestParts<S> for AuthAdmin
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<User>()
            .ok_or(StatusCode::UNAUTHORIZED)?
            .clone();

        if !user.is_admin() {
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(AuthAdmin(user))
    }
}
