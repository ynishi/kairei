use clap::{Parser, Subcommand};
use kairei_http::{
    self,
    server::{Secret, ServerConfig},
};
use std::path::PathBuf;
use tracing::{debug, info, warn};

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

    /// Secret json file path
    #[arg(short, long, default_value = "/etc/secrets/kairei-secret.json")]
    secret_json: PathBuf,

    /// Enable authentication
    #[arg(short, long, default_value = "true")]
    enable_auth: bool,

    /// Servers for documentation
    #[arg(long, env = "KAIREI_SERVERS")]
    servers: Option<String>,

    /// Directory containing DSL files for compiler services
    #[arg(long, env = "KAIREI_DSL_DIR", default_value = "dsl")]
    dsl_dir: String,

    /// Enable DSL-based compiler services
    #[arg(long, env = "KAIREI_ENABLE_DSL_COMPILER", default_value = "true")]
    enable_dsl_compiler: bool,

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
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    debug!("secret_json path: {:?}", cli.secret_json);

    if cli.secret_json.exists() {
        info!("Secret file exists");
    } else {
        warn!("Secret file does not exist: {:?}", cli.secret_json);
    }

    let secret = std::fs::read_to_string(&cli.secret_json).unwrap_or_default();
    let secret: Secret = serde_json::from_str(&secret).unwrap_or_default();

    // Handle subcommands
    let config = match &cli.command {
        Some(Commands::Config { file }) => {
            println!("Loading configuration from file: {}", file.display());
            // In a real implementation, we would load the configuration from the file
            // For now, we'll just use the default configuration
            let config: String = std::fs::read_to_string(file)?;
            serde_json::from_str(&config)?
        }
        None => {
            // Use the command line arguments to build the server configuration
            ServerConfig {
                host: cli.host,
                port: cli.port,
                enable_auth: cli.enable_auth,
                servers: cli
                    .servers
                    .map(|s| s.split(',').map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
                dsl_directory: cli.dsl_dir,
                enable_dsl_compiler: cli.enable_dsl_compiler,
            }
        }
    };
    debug!("Starting server with config: {:?}", config);
    kairei_http::start_with_config_and_secret(config, secret).await?;

    Ok(())
}
