use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use std::path::Path;

use crate::auth::{AuthStore, auth_middleware};
use crate::routes::create_api_router;
use crate::session::manager::{SessionConfig, SessionManager};
use crate::services::compiler::DslLoader;
use kairei_core::{config::{SecretConfig, SystemConfig}, system::System};

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
    pub dsl_directory: Option<String>,
    
    /// Enable DSL-based compiler services
    pub enable_dsl_compiler: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            enable_auth: true,
            servers: vec![],
            dsl_directory: Some("dsl".to_string()),
            enable_dsl_compiler: true,
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
#[derive(Clone)]
pub struct AppState {
    /// Session manager for handling user sessions
    pub session_manager: SessionManager,
    /// Authentication store for managing users and API keys
    pub auth_store: AuthStore,
    /// System instance for DSL validation and execution
    pub system: Option<Arc<System>>,
    /// DSL loader for compiler services
    pub dsl_loader: Option<DslLoader>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            session_manager: SessionManager::default(),
            auth_store: AuthStore::default(),
            system: None,
            dsl_loader: None,
        }
    }
}

/// Start the HTTP server
pub async fn start_server(
    config: ServerConfig,
    secret: Secret,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create the session manager
    let session_config = SessionConfig::default();
    let session_manager = SessionManager::new(session_config);

    // Create the auth store
    let auth_store = AuthStore::default();
    auth_store.clean_keys();
    auth_store.add_api_key(secret.admin_service_key, "admin");
    auth_store.add_api_key(format!("{}_1", secret.user_service_key.clone()), "user1");
    auth_store.add_api_key(format!("{}_2", secret.user_service_key.clone()), "user2");

    // Create the kairei system for DSL execution
    let mut system_opt = None;
    let mut dsl_loader_opt = None;
    
    if config.enable_dsl_compiler {
        // Initialize the system
        let system_config = SystemConfig::default();
        let secret_config = SecretConfig::default();
        let system = Arc::new(System::new(&system_config, &secret_config).await);
        
        if let Some(dsl_dir) = &config.dsl_directory {
            // Create DSL loader
            let dsl_path = Path::new(dsl_dir);
            if dsl_path.exists() {
                let loader = DslLoader::new(dsl_path, system.clone(), None);
                
                // Load DSL files
                match loader.load_all().await {
                    Ok(agents) => {
                        info!("Successfully loaded {} DSL agents for compiler services", agents.len());
                        for agent in &agents {
                            info!("Loaded agent: {}", agent);
                        }
                        dsl_loader_opt = Some(loader);
                    }
                    Err(e) => {
                        warn!("Failed to load DSL files: {}", e);
                    }
                }
            } else {
                warn!("DSL directory not found: {:?}", dsl_path);
            }
        }
        
        system_opt = Some(system);
        info!("Initialized Kairei system for DSL-based compiler services");
    }

    // Create the application state
    let app_state = AppState {
        session_manager,
        auth_store: auth_store.clone(),
        system: system_opt,
        dsl_loader: dsl_loader_opt,
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
