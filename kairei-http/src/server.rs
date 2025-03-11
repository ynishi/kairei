use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{debug, info, warn};

use crate::auth::{AuthStore, auth_middleware};
use crate::routes::create_api_router;
use crate::services::compiler::{CompilerSystemManager, DslLoader};
use crate::session::manager::{SessionConfig, SessionManager};
use kairei_core::config::{SystemConfig, TickerConfig};

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,

    /// Port to listen on
    pub port: u16,

    /// Enable authentication
    pub enable_auth: bool,

    /// For servers documentation
    pub servers: Vec<String>,

    /// Directory containing DSL files for compiler services
    pub dsl_directory: String,

    /// Enable DSL-based compiler services
    pub enable_dsl_compiler: bool,

    /// Enable the ticker for compiler services
    pub enable_ticker: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            enable_auth: true,
            servers: vec![],
            dsl_directory: "dsl".to_string(),
            enable_dsl_compiler: true,
            enable_ticker: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Secret {
    admin_service_key: String,
    user_service_key: String,
}

impl Default for Secret {
    fn default() -> Self {
        Self {
            admin_service_key: "admin_service_key".to_string(),
            user_service_key: "user_service_key".to_string(),
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
    /// System instance for DSL validation and execution
    pub compiler_system_manager: Option<Arc<CompilerSystemManager>>,
}

/// Start the HTTP server
pub async fn start_server(
    config: ServerConfig,
    secret: Secret,
    system_secret: Option<kairei_core::config::SecretConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create the session manager
    let session_config = SessionConfig::default();
    let session_manager = SessionManager::new(session_config, system_secret.clone());

    // Create the auth store
    let auth_store = AuthStore::default();
    auth_store.clean_keys();
    auth_store.add_api_key(secret.admin_service_key, "admin");
    auth_store.add_api_key(format!("{}_1", secret.user_service_key.clone()), "user1");
    auth_store.add_api_key(format!("{}_2", secret.user_service_key.clone()), "user2");

    let compiler_system_manager = if config.enable_dsl_compiler {
        // Initialize the system
        let mut system_config = SystemConfig::default();
        system_config.native_feature_config.ticker = Some(TickerConfig {
            enabled: config.enable_ticker,
            ..Default::default()
        });
        let secret_config = system_secret.unwrap_or_default();
        let dsl_loader = DslLoader::with_base_dir(config.dsl_directory.clone());
        debug!("dsl_loader setup: {:?}", dsl_loader);
        let mut compiler_system_manager =
            CompilerSystemManager::new(system_config, secret_config, Some(dsl_loader));
        compiler_system_manager.initialize(true).await?;

        info!("Initialized Kairei system for DSL-based compiler services");
        Some(Arc::new(compiler_system_manager))
    } else {
        warn!("DSL-based compiler services are disabled");
        None
    };

    // Create the application state
    let app_state = AppState {
        session_manager,
        auth_store: auth_store.clone(),
        compiler_system_manager,
    };

    info!("Initialized session manager and auth store");

    // Create the router with all routes and add the app state
    let mut app = create_api_router(&config).with_state(app_state.clone());

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
