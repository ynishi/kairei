use directories::ProjectDirs;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, ser::SerializeMap};
use std::{
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
/// Supported sources(in order of precedence):
/// - CLI arguments(by clap)
/// - Environment variables(by clap)
/// - Dot-env file(by clap and dotenv)
/// - Credentials file
///   - Support Store and Load
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    /// API key for authentication
    pub api_key: SecretString,

    /// Default API server URL
    #[serde(default = "default_api_url")]
    pub api_url: String,

    // for internal use
    credentials_dir: Option<String>,
}

pub fn partial_show_secret(s: &SecretString) -> String {
    // show last 4 characters
    let chars = s.expose_secret().chars();
    if chars.clone().count() <= 4 {
        "**************************".to_string()
    } else {
        let last_4 = chars.rev().take(4).collect::<String>();
        format!(
            "**********************{}",
            last_4.chars().rev().collect::<String>()
        )
    }
}

impl Serialize for Credentials {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("api_key", &self.api_key.expose_secret())?;
        map.serialize_entry("api_url", &self.api_url)?;
        map.end()
    }
}

fn default_api_url() -> String {
    "http://localhost:3000".to_string()
}

impl Default for Credentials {
    fn default() -> Self {
        Self {
            api_key: SecretString::new(Box::default()),
            api_url: default_api_url(),
            credentials_dir: None,
        }
    }
}

impl Credentials {
    pub fn new(credentials_dir: String) -> Self {
        Self {
            credentials_dir: Some(credentials_dir),
            ..Default::default()
        }
    }

    pub fn initialize(
        credentials_dir: Option<String>,
        url: Option<String>,
        key: Option<String>,
    ) -> Self {
        let mut credentials = if let Some(dir) = credentials_dir {
            let mut credentials = Credentials::new(dir);
            let _ = credentials.load_credentials();
            credentials
        } else {
            Credentials::default()
        };

        if let Some(url) = url {
            if !url.is_empty() {
                credentials.api_url = url;
            }
        }

        if let Some(key) = key {
            if !key.is_empty() {
                credentials.api_key = SecretString::new(Box::from(key));
            }
        }
        credentials
    }

    /// Get the credentials file path in the user's config directory
    pub fn get_credentials_file_path(&self) -> ConfigResult<PathBuf> {
        if let Some(parent) = self.credentials_dir.clone() {
            let parent_path = Path::new(&parent);
            if !parent_path.exists() {
                fs::create_dir_all(parent_path)
                    .map_err(|e| ConfigError::CreateConfigDir(e.to_string()))?;
            }
            return Ok(parent_path.join("credentials.json"));
        }
        let proj_dirs =
            ProjectDirs::from("com", "kairei", "kairei-cli").ok_or(ConfigError::NoHomeDir)?;

        let config_dir = proj_dirs.config_dir();

        // Create the config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)
                .map_err(|e| ConfigError::CreateConfigDir(e.to_string()))?;
        }

        Ok(config_dir.join("credentials.json"))
    }

    pub fn load_credentials(&mut self) -> ConfigResult<Credentials> {
        // Try to load from credentials file
        let file_path = self.get_credentials_file_path()?;
        if file_path.exists() {
            let mut file = File::open(&file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let credentials: Self = serde_json::from_str(&contents)?;
            self.api_key = credentials.api_key;
            self.api_url = credentials.api_url;
        }

        Ok(self.clone())
    }

    pub fn save_credentials(&self) -> ConfigResult<()> {
        let file_path = self.get_credentials_file_path()?;
        let json = serde_json::to_string_pretty(self)?;

        let mut file = File::create(file_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    fn test_key() -> String {
        "test-key-123".to_string()
    }

    fn test_url() -> String {
        "https://test-url.example.com".to_string()
    }

    // Helper function to create a temporary directory and return its path
    fn create_temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory")
    }

    fn setup_test_credentials_file() -> TempDir {
        let temp_dir = create_temp_dir();
        let credentials = Credentials {
            api_key: SecretString::new(Box::from(test_key())),
            api_url: test_url(),
            credentials_dir: temp_dir.path().to_str().map(|s| s.to_string()),
        };
        credentials.save_credentials().unwrap();
        temp_dir
    }

    #[test]
    fn test_credentials_default() {
        let credentials = Credentials::default();
        assert_eq!(credentials.api_key.expose_secret(), "");
        assert_eq!(credentials.api_url, default_api_url());
    }

    #[test]
    fn test_credentials_serialization() {
        let key = "test-key-123";
        let credentials = Credentials {
            api_key: SecretString::new(Box::from("test-key-123")),
            api_url: "https://test-url.example.com".to_string(),
            credentials_dir: None,
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&credentials).unwrap();

        // Deserialize back
        let deserialized: Credentials = serde_json::from_str(&json).unwrap();

        // Verify
        assert_eq!(credentials.api_key.expose_secret(), key);
        assert_eq!(deserialized.api_key.expose_secret(), key);
        assert_eq!(credentials.api_url, deserialized.api_url);
    }

    #[test]
    fn test_initialize_credential() {
        let credentials = Credentials::initialize(None, Some(test_url()), Some(test_key()));

        assert_eq!(credentials.api_key.expose_secret(), test_key());
        assert_eq!(credentials.api_url, test_url());

        let dir = setup_test_credentials_file();
        let credentials =
            Credentials::initialize(dir.path().to_str().map(|d| d.to_string()), None, None);
        assert_eq!(credentials.api_key.expose_secret(), test_key());
        assert_eq!(credentials.api_url, test_url());

        let key = "test-key-124";
        let url = "https://test-url.example2.com";
        let credentials = Credentials::initialize(
            dir.path().to_str().map(|d| d.to_string()),
            Some(url.to_string()),
            Some(key.to_string()),
        );
        assert_eq!(credentials.api_key.expose_secret(), key);
        assert_eq!(credentials.api_url, url);
    }
}
