use clap::{Parser, Subcommand, command};
use kairei_cli::{
    api_client::ApiClient,
    config::{Credentials, partial_show_secret},
};
use kairei_core::{
    Error,
    analyzer::Parser as _,
    config::{self, SecretConfig},
    preprocessor::Preprocessor,
    system::System,
    tokenizer::token::Token,
    type_checker::run_type_checker,
};
use kairei_http::{models::SystemConfig, services::compiler::models::ValidationError};
use secrecy::ExposeSecret;
use std::path::PathBuf;
use std::{
    fs,
    io::{self, Write},
};
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file
    #[arg(short, long, default_value = "config.json", global = true)]
    config: PathBuf,

    /// Path to secret file
    #[arg(short, long, default_value = "secret.json", global = true)]
    secret: PathBuf,

    /// Enable debug mode
    #[arg(short, long, global = true)]
    verbose: bool,

    /// API server URL
    #[arg(
        long,
        short = 'u',
        default_value = "http://localhost:3000",
        env = "KAIREI_API_URL",
        global = true
    )]
    api_url: Option<String>,

    /// API key for authentication
    #[arg(long, short = 'k', env = "KAIREI_API_KEY", global = true)]
    api_key: Option<String>,

    /// credentials directory
    #[arg(long, short = 'd', default_value = ".kairei", global = true)]
    credentials_dir: Option<String>,

    /// Output format (json, yaml, table)
    #[arg(long, default_value = "json", global = true)]
    output: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Format a Kairei DSL file
    Fmt(FmtArgs),

    /// Run the system locally
    Run(RunArgs),

    /// Compiler commands
    Compiler {
        #[command(subcommand)]
        command: CompilerCommands,
    },

    /// Manage Kairei systems (remote API)
    System {
        #[command(subcommand)]
        command: SystemCommands,
    },

    /// Manage agents within a system (remote API)
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },

    /// Manage events within a system (remote API)
    Event {
        #[command(subcommand)]
        command: EventCommands,
    },

    /// Manage API credentials
    Login(LoginArgs),
}

#[derive(Parser)]
struct FmtArgs {
    /// Path to the Kairei DSL file to format
    #[arg(default_value = "data/default.kairei")]
    file: PathBuf,

    /// Write formatted output to stdout instead of modifying the file
    #[arg(short, long)]
    stdout: bool,

    /// Strict mode, type check the DSL file
    #[arg(long)]
    strict: bool,
}

#[derive(Parser)]
struct RunArgs {
    /// Path to DSL file
    #[arg(default_value = "data/default.kairei")]
    dsl: PathBuf,
}

#[derive(Parser)]
struct LoginArgs {
    /// API key to save
    #[arg(short = 'u', long)]
    api_key: Option<String>,

    /// API URL to save
    #[arg(short = 'a', long)]
    api_url: Option<String>,

    /// Credential parent directory
    #[arg(short = 'd', long, default_value = ".kairei")]
    credentials_dir: Option<String>,

    /// Test the connection with saved credentials
    #[arg(short, long, default_value = "false")]
    test: bool,
}

#[derive(Subcommand)]
enum CompilerCommands {
    /// Validate a DSL
    Validate {
        #[arg(short, long)]
        dsl: String,

        #[arg(short = 'f', long)]
        dsl_file: Option<PathBuf>,
    },

    /// Suggest for a DSL
    Suggest {
        #[arg(short, long)]
        dsl: String,

        #[arg(short, long)]
        errors: String,

        #[arg(short = 'f', long)]
        dsl_file: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SystemCommands {
    /// List all systems
    List,

    /// Create a new system
    Create {
        /// System name
        #[arg(short, long)]
        name: String,

        /// System description
        #[arg(short, long)]
        description: Option<String>,

        /// System config file (JSON)
        #[arg(short, long)]
        config_file: Option<PathBuf>,
    },

    /// Get system details
    Get {
        /// System ID
        #[arg()]
        id: String,
    },

    /// Start a system
    Start {
        /// System ID
        #[arg()]
        id: String,

        /// DSL file to use (optional)
        #[arg(short, long)]
        dsl: Option<PathBuf>,
    },

    /// Stop a system
    Stop {
        /// System ID
        #[arg()]
        id: String,
    },

    /// Delete a system
    Delete {
        /// System ID
        #[arg()]
        id: String,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// List all agents in a system
    List {
        /// System ID
        #[arg()]
        system_id: String,
    },

    /// Get agent details
    Get {
        /// System ID
        #[arg()]
        system_id: String,

        /// Agent ID
        #[arg()]
        agent_id: String,
    },

    /// Create a new agent
    Create {
        /// System ID
        #[arg()]
        system_id: String,

        /// Agent definition file (JSON)
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Update an agent
    Update {
        /// System ID
        #[arg()]
        system_id: String,

        /// Agent ID
        #[arg()]
        agent_id: String,

        /// Agent definition file (JSON)
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Start an agent
    Start {
        /// System ID
        #[arg()]
        system_id: String,

        /// Agent ID
        #[arg()]
        agent_id: String,
    },

    /// Stop an agent
    Stop {
        /// System ID
        #[arg()]
        system_id: String,

        /// Agent ID
        #[arg()]
        agent_id: String,
    },

    /// Delete an agent
    Delete {
        /// System ID
        #[arg()]
        system_id: String,

        /// Agent ID
        #[arg()]
        agent_id: String,
    },
}

#[derive(Subcommand)]
enum EventCommands {
    /// List events in a system
    List {
        /// System ID
        #[arg()]
        system_id: String,
    },

    /// Emit an event to a system
    Emit {
        /// System ID
        #[arg()]
        system_id: String,

        /// Event definition file (JSON)
        #[arg(short, long)]
        file: PathBuf,
    },
}

async fn format_file(args: &FmtArgs) -> Result<(), Error> {
    // Read input file
    let input = std::fs::read_to_string(&args.file)
        .map_err(|e| Error::Internal(format!("Failed to read input file: {}", e)))?;

    // Parse using existing workflow
    let tokens = kairei_core::tokenizer::token::Tokenizer::new()
        .tokenize(&input)
        .map_err(|e| Error::Internal(format!("Failed to tokenize: {}", e)))?;

    let preprocessor = kairei_core::preprocessor::TokenPreprocessor::default();
    let tokens: Vec<Token> = preprocessor.process(tokens);

    let (_, root) = kairei_core::analyzer::parsers::world::parse_root()
        .parse(tokens.as_slice(), 0)
        .map_err(|e| Error::Internal(format!("Failed to parse: {}", e)))?;

    // Type check
    if args.strict {
        let mut root = root.clone();
        run_type_checker(&mut root)
            .map_err(|e| Error::Internal(format!("Failed to type check: {}", e)))?;
    }

    // Format using existing formatter
    let formatter = kairei_core::formatter::Formatter::new(
        kairei_core::formatter::config::FormatterConfig::default(),
    );
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

async fn run_local(
    args: &RunArgs,
    config_path: &PathBuf,
    secret_path: &PathBuf,
) -> Result<(), Error> {
    // Load config
    let config: SystemConfig = if config_path.exists() {
        config::from_file(config_path)?
    } else {
        // Default config
        SystemConfig::default()
    };

    let secret_config: SecretConfig = if secret_path.exists() {
        config::from_file(secret_path)?
    } else {
        // Default secret config
        SecretConfig::default()
    };

    info!("Config loaded.");
    debug!("config: {:?}", config);
    debug!("secret_config: {:?}", secret_config);

    // Initialize system
    let mut system = System::new(
        &kairei_core::config::SystemConfig::from(config),
        &secret_config,
    )
    .await;

    // Load and parse DSL
    let dsl = std::fs::read_to_string(&args.dsl)
        .map_err(|e| Error::Internal(format!("Failed to read DSL file: {}", e)))?;

    debug!("Parsing DSL file: {:?}", args.dsl);

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

fn get_api_client(cli: &Cli) -> ApiClient {
    let credential = Credentials::initialize(
        cli.credentials_dir.clone(),
        cli.api_url.clone(),
        cli.api_key.clone(),
    );

    ApiClient::new(&credential.api_url, &credential.api_key)
}

fn output_json<T: serde::Serialize>(data: &T, pretty: bool) -> Result<(), Error> {
    let output = if pretty {
        serde_json::to_string_pretty(data)
    } else {
        serde_json::to_string(data)
    }
    .map_err(|e| Error::Internal(format!("JSON serialization error: {}", e)))?;

    println!("{}", output);
    Ok(())
}

async fn handle_compiler_commands(cmd: &CompilerCommands, cli: &Cli) -> Result<(), Error> {
    let client = get_api_client(cli);

    match cmd {
        CompilerCommands::Validate { dsl, dsl_file } => {
            let dsl_content = if let Some(path) = dsl_file {
                fs::read_to_string(path)
                    .map_err(|e| Error::Internal(format!("Failed to read DSL file: {}", e)))?
            } else {
                dsl.clone()
            };

            let response = client
                .compiler_dsl(&dsl_content)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)
        }

        CompilerCommands::Suggest {
            dsl,
            errors,
            dsl_file,
        } => {
            let dsl_content = if let Some(path) = dsl_file {
                fs::read_to_string(path)
                    .map_err(|e| Error::Internal(format!("Failed to read DSL file: {}", e)))?
            } else {
                dsl.clone()
            };

            let errors: Vec<ValidationError> = serde_json::from_str(errors)
                .map_err(|e| Error::Internal(format!("Failed to parse errors: {}", e)))?;

            let response = client
                .compiler_suggest(&dsl_content, errors.as_slice())
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)
        }
    }
}

async fn handle_system_commands(cmd: &SystemCommands, cli: &Cli) -> Result<(), Error> {
    let client = get_api_client(cli);

    match cmd {
        SystemCommands::List => {
            let response = client
                .list_systems()
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        SystemCommands::Create {
            name,
            description,
            config_file,
        } => {
            let config = config_file
                .clone()
                .map(|path| {
                    config::from_file::<kairei_core::config::SystemConfig, &PathBuf>(&path)
                        .map_err(|e| Error::Internal(format!("Failed to read config file: {}", e)))
                        .unwrap()
                })
                .unwrap_or_default();

            let response = client
                .create_system(name, description.as_deref(), config)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        SystemCommands::Get { id } => {
            let response = client
                .get_system(id)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        SystemCommands::Start { id, dsl } => {
            let dsl_content = if let Some(path) = dsl {
                Some(
                    fs::read_to_string(path)
                        .map_err(|e| Error::Internal(format!("Failed to read DSL file: {}", e)))?,
                )
            } else {
                None
            };

            let response = client
                .start_system(id, dsl_content.as_deref())
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        SystemCommands::Stop { id } => {
            let response = client
                .stop_system(id)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        SystemCommands::Delete { id } => {
            let response = client
                .delete_system(id)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
            println!("System {} deleted successfully", id);
        }
    }

    Ok(())
}

async fn handle_agent_commands(cmd: &AgentCommands, cli: &Cli) -> Result<(), Error> {
    let client = get_api_client(cli);

    match cmd {
        AgentCommands::List { system_id } => {
            let response = client
                .list_agents(system_id)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        AgentCommands::Get {
            system_id,
            agent_id,
        } => {
            let response = client
                .get_agent(system_id, agent_id)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        AgentCommands::Create { system_id, file } => {
            let content = fs::read_to_string(file).map_err(|e| {
                Error::Internal(format!("Failed to read agent definition file: {}", e))
            })?;

            let definition: kairei_http::models::AgentCreationRequest =
                serde_json::from_str(&content).map_err(|e| {
                    Error::Internal(format!("Failed to parse agent definition: {}", e))
                })?;

            let response = client
                .create_agent(system_id, &definition)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        AgentCommands::Update {
            system_id,
            agent_id,
            file,
        } => {
            let content = fs::read_to_string(file).map_err(|e| {
                Error::Internal(format!("Failed to read agent definition file: {}", e))
            })?;

            let definition: kairei_http::models::AgentCreationRequest =
                serde_json::from_str(&content).map_err(|e| {
                    Error::Internal(format!("Failed to parse agent definition: {}", e))
                })?;

            let response = client
                .update_agent(system_id, agent_id, &definition)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        AgentCommands::Start {
            system_id,
            agent_id,
        } => {
            let response = client
                .start_agent(system_id, agent_id)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        AgentCommands::Stop {
            system_id,
            agent_id,
        } => {
            let response = client
                .stop_agent(system_id, agent_id)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        AgentCommands::Delete {
            system_id,
            agent_id,
        } => {
            let response = client
                .delete_agent(system_id, agent_id)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
            println!("Agent {} deleted successfully", agent_id);
        }
    }

    Ok(())
}

async fn handle_event_commands(cmd: &EventCommands, cli: &Cli) -> Result<(), Error> {
    let client = get_api_client(cli);

    match cmd {
        EventCommands::List { system_id } => {
            let response = client
                .list_events(system_id)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
        }

        EventCommands::Emit { system_id, file } => {
            let content = fs::read_to_string(file).map_err(|e| {
                Error::Internal(format!("Failed to read event definition file: {}", e))
            })?;

            let definition: kairei_http::models::EventRequest = serde_json::from_str(&content)
                .map_err(|e| Error::Internal(format!("Failed to parse event definition: {}", e)))?;

            let response = client
                .emit_event(system_id, &definition)
                .await
                .map_err(|e| Error::Internal(format!("API error: {}", e)))?;
            output_json(&response, true)?;
            println!("Event emitted successfully");
        }
    }

    Ok(())
}

async fn handle_login_command(args: &LoginArgs) -> Result<(), Error> {
    // Start with existing credentials or defaults
    let credentials = Credentials::initialize(
        args.credentials_dir.clone(),
        args.api_url.clone(),
        args.api_key.clone(),
    );

    // Display current settings
    println!("Current API settings:");
    println!("API URL: {}", credentials.api_url);

    if !credentials.api_key.expose_secret().is_empty() {
        println!("API Key: {:#?}", partial_show_secret(&credentials.api_key));
    } else {
        println!("API Key: Not set");
    }

    // Test connection if requested
    if args.test {
        print!("Testing API connection... ");
        io::stdout()
            .flush()
            .map_err(|e| Error::Internal(e.to_string()))?;

        let client = ApiClient::new(&credentials.api_url, &credentials.api_key);
        match client.health_check().await {
            Ok(_) => println!("✅ Success"),
            Err(e) => println!("❌ Failed: {}", e),
        }
    }

    Ok(())
}

async fn run(cli: &Cli) -> Result<(), Error> {
    match &cli.command {
        Commands::Fmt(args) => format_file(args).await,
        Commands::Run(args) => run_local(args, &cli.config, &cli.secret).await,
        Commands::Compiler { command } => handle_compiler_commands(command, cli).await,
        Commands::System { command } => handle_system_commands(command, cli).await,
        Commands::Agent { command } => handle_agent_commands(command, cli).await,
        Commands::Event { command } => handle_event_commands(command, cli).await,
        Commands::Login(args) => handle_login_command(args).await,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let _ = dotenv::dotenv();

    let cli = Cli::parse();

    if let Err(e) = run(&cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
