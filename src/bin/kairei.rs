use clap::{command, Parser};
use kairei::{
    config::{self, SecretConfig, SystemConfig},
    system::System,
    Error,
};
use std::path::PathBuf;
use tracing::{debug, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    #[arg(short, long, default_value = "data/default.kairei")]
    dsl: PathBuf,

    #[arg(short, long, default_value = "secret.json")]
    secret: PathBuf,

    /// Enable debug mode
    #[arg(short, long)]
    verbose: bool,
}

async fn run(cli: &Cli) -> Result<(), Error> {
    // Load config
    let config_path = cli.config.clone();
    let config: SystemConfig = if config_path.clone().exists() {
        config::from_file(config_path)?
    } else {
        // Default config
        SystemConfig::default()
    };
    let secret_path = cli.secret.clone();
    let secret_config: SecretConfig = if secret_path.clone().exists() {
        config::from_file(secret_path)?
    } else {
        // Default secret config
        SecretConfig::default()
    };

    info!("config loaded.");

    debug!("config: {:?}", config);

    debug!("secret_config: {:?}", secret_config);

    // Initialize system
    let mut system = System::new(&config, &secret_config).await;

    // Load and parse DSL
    let dsl = std::fs::read_to_string(&cli.dsl)
        .map_err(|e| Error::Internal(format!("Failed to read DSL file: {}", e)))?;

    debug!("Parsing DSL file: {:?}", cli.dsl);

    let root = system.parse_dsl(&dsl).await?;

    debug!("Successfully parsed DSL, initializing system...");

    // Initialize system with parsed definitions
    system.initialize(root).await?;

    debug!("System initialized, starting...");

    // Start system
    system.start().await?;

    // Message to user as UI.
    println!("Welcome to Kairei! System started. Press Ctrl+C to shutdown.");

    // Wait for shutdown signal
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| Error::Internal(format!("Failed to wait for Ctrl+C: {}", e)))?;

    println!("Shutdown signal received, performing clean shutdown...");

    // Clean shutdown
    system.shutdown().await?;

    debug!("Shutdown completed.");

    println!("System shutdown completed.");

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(&cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
