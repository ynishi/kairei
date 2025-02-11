use clap::{command, Parser};
use kairei::{
    analyzer::Parser as _,
    config::{self, SecretConfig, SystemConfig},
    preprocessor::Preprocessor,
    system::System,
    tokenizer::token::Token,
    Error,
};
use std::path::PathBuf;
use tracing::{debug, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
enum Commands {
    Fmt(FmtArgs),
}

#[derive(Parser)]
struct FmtArgs {
    /// Path to the Kairei DSL file to format
    #[arg(default_value = "data/default.kairei")]
    file: PathBuf,

    /// Write formatted output to stdout instead of modifying the file
    #[arg(short, long)]
    stdout: bool,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to config file
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    /// Path to DSL file
    #[arg(short, long, default_value = "data/default.kairei")]
    dsl: PathBuf,

    #[arg(short, long, default_value = "secret.json")]
    secret: PathBuf,

    /// Enable debug mode
    #[arg(short, long)]
    verbose: bool,
}

async fn format_file(args: &FmtArgs) -> Result<(), Error> {
    // Read input file
    let input = std::fs::read_to_string(&args.file)
        .map_err(|e| Error::Internal(format!("Failed to read input file: {}", e)))?;

    // Parse using existing workflow
    let tokens = kairei::tokenizer::token::Tokenizer::new()
        .tokenize(&input)
        .map_err(|e| Error::Internal(format!("Failed to tokenize: {}", e)))?;

    let preprocessor = kairei::preprocessor::TokenPreprocessor::default();
    let tokens: Vec<Token> = preprocessor.process(tokens);

    let (_, root) = kairei::analyzer::parsers::world::parse_root()
        .parse(tokens.as_slice(), 0)
        .map_err(|e| Error::Internal(format!("Failed to parse: {}", e)))?;

    // Format using existing formatter
    let formatter =
        kairei::formatter::Formatter::new(kairei::formatter::config::FormatterConfig::default());
    let formatted = formatter
        .format(&root)
        .map_err(|e| Error::Internal(format!("Failed to format: {}", e)))?;

    // Output
    if args.stdout {
        println!("{}", formatted);
    } else {
        std::fs::write(&args.file, formatted)
            .map_err(|e| Error::Internal(format!("Failed to write output file: {}", e)))?;
    }

    Ok(())
}

async fn run(cli: &Cli) -> Result<(), Error> {
    match &cli.command {
        Some(Commands::Fmt(args)) => format_file(args).await,
        None => {
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
    }
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
