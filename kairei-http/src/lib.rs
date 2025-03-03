//! Kairei HTTP API Server
//!
//! This crate provides an HTTP API for interacting with the Kairei agent system.

pub mod handlers;
pub mod models;
pub mod routes;
pub mod server;

use server::{ServerConfig, start_server};

/// Start the Kairei HTTP server with the default configuration
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Start the server with default configuration
    start_server(ServerConfig::default()).await
}

/// Start the Kairei HTTP server with a custom configuration
pub async fn start_with_config(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Start the server with the provided configuration
    start_server(config).await
}
