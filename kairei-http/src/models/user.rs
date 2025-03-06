use serde::{Deserialize, Serialize};
use std::fmt;

/// User role for basic authorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UserRole {
    /// Regular user with standard permissions
    #[default]
    User,
    /// Administrator with elevated permissions
    Admin,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::User => write!(f, "user"),
            UserRole::Admin => write!(f, "admin"),
        }
    }
}

/// User model representing a system user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for the user
    pub user_id: String,
    /// Username for display purposes
    pub username: String,
    /// User's role for authorization
    pub role: UserRole,
}

impl User {
    /// Create a new user with the given ID, username, and role
    pub fn new(user_id: impl Into<String>, username: impl Into<String>, role: UserRole) -> Self {
        Self {
            user_id: user_id.into(),
            username: username.into(),
            role,
        }
    }

    /// Create a new regular user
    pub fn new_user(user_id: impl Into<String>, username: impl Into<String>) -> Self {
        Self::new(user_id, username, UserRole::User)
    }

    /// Create a new admin user
    pub fn new_admin(user_id: impl Into<String>, username: impl Into<String>) -> Self {
        Self::new(user_id, username, UserRole::Admin)
    }

    /// Check if the user has admin role
    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }
}
