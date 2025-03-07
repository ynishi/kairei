use crate::models::user::User;
use dashmap::DashMap;
use std::sync::Arc;

/// A simple in-memory store for API keys and users
#[derive(Clone, Debug)]
pub struct AuthStore {
    /// Maps API keys to user IDs
    api_keys: Arc<DashMap<String, String>>,
    /// Maps user IDs to User objects
    users: Arc<DashMap<String, User>>,
}

impl AuthStore {
    /// Create a new empty auth store
    pub fn new() -> Self {
        Self {
            api_keys: Arc::new(DashMap::new()),
            users: Arc::new(DashMap::new()),
        }
    }

    /// Create a new auth store with some default users and API keys
    pub fn with_defaults() -> Self {
        let store = Self::new();

        // Add some default users
        let admin = User::new_admin("admin", "Administrator");
        let user1 = User::new_user("user1", "Regular User 1");
        let user2 = User::new_user("user2", "Regular User 2");

        store.add_user(admin.clone());
        store.add_user(user1.clone());
        store.add_user(user2.clone());

        // Add API keys for the users
        store.add_api_key("admin-key", admin.user_id);
        store.add_api_key("user1-key", user1.user_id);
        store.add_api_key("user2-key", user2.user_id);

        store
    }

    /// Add a user to the store
    pub fn add_user(&self, user: User) {
        self.users.insert(user.user_id.clone(), user);
    }

    /// Get a user by ID
    pub fn get_user(&self, user_id: &str) -> Option<User> {
        self.users.get(user_id).map(|u| u.clone())
    }

    /// Add an API key for a user
    pub fn add_api_key(&self, api_key: impl Into<String>, user_id: impl Into<String>) {
        self.api_keys.insert(api_key.into(), user_id.into());
    }

    /// Get a user by API key
    pub fn get_user_by_api_key(&self, api_key: &str) -> Option<User> {
        let user_id = self.api_keys.get(api_key)?;
        self.get_user(&user_id)
    }

    /// Remove an API key
    pub fn remove_api_key(&self, api_key: &str) {
        self.api_keys.remove(api_key);
    }

    /// Remove all
    pub fn clean_keys(&self) {
        self.api_keys.clear();
    }

    /// Remove a user and all associated API keys
    pub fn remove_user(&self, user_id: &str) {
        // Remove the user
        self.users.remove(user_id);

        // Remove all API keys for this user
        let keys_to_remove: Vec<String> = self
            .api_keys
            .iter()
            .filter(|entry| entry.value() == user_id)
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys_to_remove {
            self.api_keys.remove(&key);
        }
    }
}

impl Default for AuthStore {
    fn default() -> Self {
        Self::with_defaults()
    }
}
