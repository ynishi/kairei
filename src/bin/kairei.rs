use clap::{command, Parser};
use kairei::{config::SystemConfig, system::System, KaireiError};
use std::path::PathBuf;
use tracing::{debug, info};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    #[arg(short, long, default_value = "data/default.kairei")]
    dsl: PathBuf,

    /// Enable debug mode
    #[arg(short, long)]
    verbose: bool,
}

async fn run(cli: &Cli) -> Result<(), KaireiError> {
    // Load config
    let config_path = cli.config.clone();
    let config: SystemConfig = if config_path.clone().exists() {
        let content = std::fs::read_to_string(config_path)
            .map_err(|e| KaireiError::Internal(format!("Failed to read config file: {}", e)))?;
        serde_json::from_str(&content)
            .map_err(|e| KaireiError::Internal(format!("Failed to parse config file: {}", e)))?
    } else {
        // Default config
        SystemConfig::default()
    };

    info!("config loaded.");

    debug!("config: {:?}", config);

    // Initialize system
    let mut system = System::new(&config).await;

    // Load and parse DSL
    let dsl = std::fs::read_to_string(&cli.dsl)
        .map_err(|e| KaireiError::Internal(format!("Failed to read DSL file: {}", e)))?;

    debug!("Parsing DSL file: {:?}", cli.dsl);

    let asts = system.parse_dsl(&dsl).await?;

    debug!("Successfully parsed DSL, initializing system...");

    // Initialize system with parsed definitions
    system.initialize(asts).await?;

    debug!("System initialized, starting...");

    // Start system
    system.start().await?;

    // Message to user as UI.
    println!("Welcome to Kairei! System started. Press Ctrl+C to shutdown.");

    // Wait for shutdown signal
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| KaireiError::Internal(format!("Failed to wait for Ctrl+C: {}", e)))?;

    println!("Shutdown signal received, performing clean shutdown...");

    // Clean shutdown
    system.shutdown().await?;

    println!("System shutdown completed.");

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    if let Err(e) = run(&cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
