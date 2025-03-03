use clap::{Parser, Subcommand};
use kairei_http::{self, server::ServerConfig};
use std::path::PathBuf;

/// Kairei HTTP API Server
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Host address to bind to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the server with a specific configuration file
    Config {
        /// Path to the configuration file
        #[arg(short, long)]
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Note: We don't initialize tracing here because it's already initialized in the library
    // This avoids the "a global default trace dispatcher has already been set" error

    // Handle subcommands
    match &cli.command {
        Some(Commands::Config { file }) => {
            println!("Loading configuration from file: {}", file.display());
            // In a real implementation, we would load the configuration from the file
            // For now, we'll just use the default configuration
            kairei_http::start().await?;
        }
        None => {
            // Use the host and port from the command line arguments
            let config = ServerConfig {
                host: cli.host,
                port: cli.port,
            };

            println!(
                "Starting Kairei HTTP server on {}:{}",
                config.host, config.port
            );
            kairei_http::start_with_config(config).await?;
        }
    }

    Ok(())
}
