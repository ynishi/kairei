use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use kairei_core::{
    config::{SecretConfig, SystemConfig},
    system::System,
};

use crate::integration::KaireiSystem;
use crate::routes::create_api_router;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,

    /// Port to listen on
    pub port: u16,

    /// Kairei system configuration
    pub system_config: Option<SystemConfig>,

    /// Kairei secret configuration
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
    // Initialize the kairei system
    let system_config = config.system_config.unwrap_or_default();
    let secret_config = config.secret_config.unwrap_or_default();

    let system = Arc::new(System::new(&system_config, &secret_config).await);
    let kairei_system = Arc::new(KaireiSystem::new(system.clone()));

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create the router with all routes
    let app = create_api_router(kairei_system)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Parse the socket address
    let addr = format!("{}:{}", config.host, config.port).parse::<SocketAddr>()?;

    // Start the server
    info!("Starting server on {}", addr);

    // Start the kairei system
    info!("Starting kairei system");
    system
        .start()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // In axum 0.8.x, we use this pattern to start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(system.clone()))
        .await?;

    Ok(())
}

/// Signal handler for graceful shutdown
async fn shutdown_signal(system: Arc<System>) {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");

    // Shutdown the kairei system
    info!("Shutting down kairei system");
    if let Err(e) = system.shutdown().await {
        tracing::error!("Error shutting down kairei system: {}", e);
    }
}
