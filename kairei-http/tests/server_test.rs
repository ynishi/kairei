use kairei_http::server::{ServerConfig, start_server};
use std::net::{SocketAddr, TcpListener};
use std::time::Duration;
use tokio::time::timeout;

#[test]
fn test_server_config_default() {
    // Create a default server config
    let config = ServerConfig::default();

    // Verify the default values
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3000);
    //assert_eq!(config.enable_auth, false);
}

#[test]
fn test_server_config_custom() {
    // Create a custom server config
    let config = ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        enable_auth: true,
    };

    // Verify the custom values
    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8080);
    assert!(config.enable_auth);
}

#[tokio::test]
async fn test_server_address_parsing() {
    // Create a server config
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8081,
        enable_auth: false,
    };

    // Parse the socket address
    let addr = format!("{}:{}", config.host, config.port)
        .parse::<SocketAddr>()
        .unwrap();

    // Verify the parsed address
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_eq!(addr.port(), 8081);
}

#[tokio::test]
#[ignore] // This test starts an actual server, so we mark it as ignored by default
async fn test_server_startup_with_auth() {
    // Create a server config with a random available port and auth enabled
    let port = find_available_port().expect("Failed to find an available port");
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        enable_auth: true,
    };

    // Start the server with a timeout
    let server_future = start_server(config.clone());
    let result = timeout(Duration::from_secs(1), server_future).await;

    // The server should still be running after the timeout
    assert!(result.is_err(), "Server should still be running");

    // Try to connect to the server without an API key
    let addr = format!("{}:{}", config.host, config.port);
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/api/v1/system/info", addr))
        .timeout(Duration::from_secs(1))
        .send()
        .await;

    // We should be able to connect to the server
    assert!(response.is_ok(), "Failed to connect to the server");

    // The response should be unauthorized
    let response = response.unwrap();
    assert_eq!(response.status(), 401, "Server should return unauthorized");

    // Try to connect to the server with an API key
    let response = client
        .get(format!("http://{}/api/v1/system/info", addr))
        .header("X-API-Key", "admin-key")
        .timeout(Duration::from_secs(1))
        .send()
        .await;

    // We should be able to connect to the server
    assert!(response.is_ok(), "Failed to connect to the server");

    // The response should be successful
    let response = response.unwrap();
    assert!(response.status().is_success(), "Server returned an error");
}

#[tokio::test]
#[ignore] // This test starts an actual server, so we mark it as ignored by default
async fn test_server_startup() {
    // Create a server config with a random available port
    let port = find_available_port().expect("Failed to find an available port");
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        enable_auth: false,
    };

    // Start the server with a timeout
    let server_future = start_server(config.clone());
    let result = timeout(Duration::from_secs(1), server_future).await;

    // The server should still be running after the timeout
    assert!(result.is_err(), "Server should still be running");

    // Try to connect to the server
    let addr = format!("{}:{}", config.host, config.port);
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/api/v1/system/info", addr))
        .timeout(Duration::from_secs(1))
        .send()
        .await;

    // We should be able to connect to the server
    assert!(response.is_ok(), "Failed to connect to the server");

    // The response should be successful
    let response = response.unwrap();
    assert!(response.status().is_success(), "Server returned an error");
}

// Helper function to find an available port
fn find_available_port() -> Option<u16> {
    // Try to bind to port 0, which will assign a random available port
    if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
        return Some(listener.local_addr().unwrap().port());
    }
    None
}
