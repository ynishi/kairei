use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::auth::{AuthStore, auth_middleware};
use crate::routes::create_api_router;
use crate::session::manager::{SessionConfig, SessionManager};
use kairei_core::config::{SecretConfig, SystemConfig};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,

    /// Port to listen on
    pub port: u16,

    /// System configuration
    pub system_config: Option<SystemConfig>,

    /// Secret configuration
    pub secret_config: Option<SecretConfig>,

    /// Enable authentication
    pub enable_auth: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            system_config: None,
            secret_config: None,
            enable_auth: false,
        }
    }
}

/// Application state containing shared resources
#[derive(Clone, Default)]
pub struct AppState {
    /// Session manager for handling user sessions
    pub session_manager: SessionManager,
    /// Authentication store for managing users and API keys
    pub auth_store: AuthStore,
}

/// Start the HTTP server
pub async fn start_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create the session manager
    let session_config = SessionConfig {
        system_config: config.system_config.unwrap_or_default(),
        secret_config: config.secret_config.unwrap_or_default(),
    };
    let session_manager = SessionManager::new(session_config);

    // Create the auth store
    let auth_store = AuthStore::default();

    // Create the application state
    let app_state = AppState {
        session_manager,
        auth_store: auth_store.clone(),
    };

    info!("Initialized session manager and auth store");

    // Create the router with all routes and add the app state
    let mut app = create_api_router().with_state(app_state.clone());

    // Apply authentication middleware if enabled
    if config.enable_auth {
        info!("Authentication enabled");
        // Apply the auth middleware to all routes
        let auth_store = Arc::new(app_state.auth_store.clone());
        app = app.layer(axum::middleware::from_fn_with_state(
            auth_store,
            auth_middleware,
        ));
    }

    // Add common middleware
    let app = app.layer(TraceLayer::new_for_http()).layer(cors);

    // Parse the socket address
    let addr = format!("{}:{}", config.host, config.port).parse::<SocketAddr>()?;

    // Start the server
    info!("Starting server on {}", addr);

    // In axum 0.8.x, we use this pattern to start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
