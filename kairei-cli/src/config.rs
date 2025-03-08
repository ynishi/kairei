use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("No home directory found")]
    NoHomeDir,
    
    #[error("Config directory not found")]
    NoConfigDir,
    
    #[error("Failed to create config directory: {0}")]
    CreateConfigDir(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

/// Credentials for accessing the Kairei API
#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    /// API key for authentication
    pub api_key: String,
    
    /// Default API server URL
    #[serde(default = "default_api_url")]
    pub api_url: String,
}

fn default_api_url() -> String {
    "http://localhost:3000".to_string()
}

impl Default for Credentials {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_url: default_api_url(),
        }
    }
}

/// Get the credentials file path in the user's config directory
fn get_credentials_file_path() -> ConfigResult<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "kairei", "kairei-cli")
        .ok_or(ConfigError::NoHomeDir)?;
    
    let config_dir = proj_dirs.config_dir();
    
    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)
            .map_err(|e| ConfigError::CreateConfigDir(e.to_string()))?;
    }
    
    Ok(config_dir.join("credentials.json"))
}

/// Load credentials from various sources in order of precedence:
/// 1. Environment variables (KAIREI_API_KEY, KAIREI_API_URL)
/// 2. .env file in the current directory
/// 3. Credentials file in the user's config directory
pub fn load_credentials() -> ConfigResult<Credentials> {
    // Load .env file if it exists (doesn't overwrite existing env vars)
    let _ = dotenv::dotenv();
    
    // Start with default credentials
    let mut credentials = Credentials::default();
    
    // Try to load from credentials file
    let file_path = get_credentials_file_path()?;
    if file_path.exists() {
        let mut file = File::open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        credentials = serde_json::from_str(&contents)?;
    }
    
    // Override with environment variables if they exist
    if let Ok(api_key) = env::var("KAIREI_API_KEY") {
        credentials.api_key = api_key;
    }
    
    if let Ok(api_url) = env::var("KAIREI_API_URL") {
        credentials.api_url = api_url;
    }
    
    Ok(credentials)
}

/// Save credentials to the user's config directory
pub fn save_credentials(credentials: &Credentials) -> ConfigResult<()> {
    let file_path = get_credentials_file_path()?;
    let json = serde_json::to_string_pretty(credentials)?;
    
    let mut file = File::create(file_path)?;
    file.write_all(json.as_bytes())?;
    
    Ok(())
}

/// Check if .env file exists in the current directory and create it if requested
pub fn setup_env_file<P: AsRef<Path>>(path: P, api_key: &str, api_url: Option<&str>) -> ConfigResult<()> {
    let path = path.as_ref();
    
    // Create or append to .env file
    let mut content = String::new();
    
    // If file exists, read its content
    if path.exists() {
        let mut file = File::open(path)?;
        file.read_to_string(&mut content)?;
    }

    let find_content = content.clone();
    
    // Helper to check if a variable is already set in the content
    let has_var = |name: &str| find_content.lines().any(|line| line.starts_with(&format!("{}=", name)));
    
    // Add API key if not already set
    if !has_var("KAIREI_API_KEY") && !api_key.is_empty() {
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&format!("KAIREI_API_KEY={}\n", api_key));
    }
    
    // Add API URL if provided and not already set
    if let Some(url) = api_url {
        if !has_var("KAIREI_API_URL") && !url.is_empty() {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&format!("KAIREI_API_URL={}\n", url));
        }
    }
    
    // Write to file
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    
    Ok(())
}

/// Get the best available API key from various sources
pub fn get_api_key(cli_key: Option<&str>) -> String {
    // First priority: CLI argument
    if let Some(key) = cli_key {
        if !key.is_empty() {
            return key.to_string();
        }
    }
    
    // Second priority: Environment variable
    if let Ok(key) = env::var("KAIREI_API_KEY") {
        if !key.is_empty() {
            return key;
        }
    }
    
    // Third priority: Stored credentials
    if let Ok(creds) = load_credentials() {
        if !creds.api_key.is_empty() {
            return creds.api_key;
        }
    }
    
    // Default: empty string
    String::new()
}

/// Get the best available API URL from various sources
pub fn get_api_url(cli_url: &str) -> String {
    // First priority: CLI argument
    if !cli_url.is_empty() {
        return cli_url.to_string();
    }
    
    // Second priority: Environment variable
    if let Ok(url) = env::var("KAIREI_API_URL") {
        if !url.is_empty() {
            return url;
        }
    }
    
    // Third priority: Stored credentials
    if let Ok(creds) = load_credentials() {
        if !creds.api_url.is_empty() {
            return creds.api_url;
        }
    }
    
    // Default
    default_api_url()
}