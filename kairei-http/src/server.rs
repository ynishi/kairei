use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            system_config: None,
            secret_config: None,
        }
    }
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

    info!("Initialized session manager");

    // Create the router with all routes and add the session manager as state
    let app = create_api_router()
        .with_state(session_manager)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Parse the socket address
    let addr = format!("{}:{}", config.host, config.port).parse::<SocketAddr>()?;

    // Start the server
    info!("Starting server on {}", addr);

    // In axum 0.8.x, we use this pattern to start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
