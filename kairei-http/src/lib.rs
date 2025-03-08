//! Kairei HTTP API Server
//!
//! This crate provides an HTTP API for interacting with the Kairei agent system.

pub mod auth;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod server;
pub mod session;

use server::{Secret, ServerConfig, start_server};

/// Start the Kairei HTTP server with the default configuration
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Start the server with default configuration
    start_server(ServerConfig::default(), Secret::default()).await
}

/// Start the Kairei HTTP server with a custom configuration
pub async fn start_with_config(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Start the server with the provided configuration
    start_server(config, Secret::default()).await
}

pub async fn start_with_config_and_secret(
    config: ServerConfig,
    secret: Secret,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Start the server with the provided configuration
    start_server(config, secret).await
}
