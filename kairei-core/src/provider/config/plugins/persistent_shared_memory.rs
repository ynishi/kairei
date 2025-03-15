//! Persistent Shared Memory plugin configuration.
//!
//! This module defines the configuration options for the PersistentSharedMemoryPlugin,
//! allowing customization of behavior such as storage backend type, synchronization
//! intervals, and auto-load/save options.
//!
//! # Example
//!
//! ```no_run
//! use kairei_core::provider::config::plugins::{PersistentSharedMemoryConfig, SharedMemoryConfig};
//! use kairei_core::provider::config::BasePluginConfig;
//! use std::time::Duration;
//!
//! let config = PersistentSharedMemoryConfig {
//!     base: SharedMemoryConfig {
//!         base: BasePluginConfig::default(),
//!         max_keys: 5000,
//!         ttl: Duration::from_secs(7200),
//!         namespace: "my_application".to_string(),
//!     },
//!     persistence: PersistenceConfig {
//!         backend_type: BackendType::LocalFileSystem,
//!         sync_interval: Duration::from_secs(60),
//!         auto_load: true,
//!         auto_save: true,
//!         backend_config: BackendSpecificConfig::Local(LocalFileSystemConfig {
//!             base_dir: "/tmp/kairei/shared_memory".to_string(),
//!             file_extension: "json".to_string(),
//!         }),
//!     },
//! };
//! ```

use super::{BasePluginConfig, ProviderSpecificConfig, SharedMemoryConfig};
use crate::provider::config::base::ConfigError;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use utoipa::ToSchema;

/// Persistent Shared Memory plugin configuration
///
/// This structure defines the configuration options for the PersistentSharedMemoryPlugin,
/// allowing customization of behavior such as storage backend type, synchronization
/// intervals, and auto-load/save options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PersistentSharedMemoryConfig {
    /// Base SharedMemory configuration
    #[serde(flatten)]
    pub base: SharedMemoryConfig,
    
    /// Persistence configuration
    pub persistence: PersistenceConfig,
}

/// Configuration for persistence features
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PersistenceConfig {
    /// Backend type
    pub backend_type: BackendType,
    
    /// Auto-sync interval (0 means no auto-sync)
    #[serde(with = "crate::config::duration_ms")]
    #[schema(value_type = u64, pattern = "uint64 as milliseconds")]
    pub sync_interval: Duration,
    
    /// Auto-load on startup
    pub auto_load: bool,
    
    /// Auto-save on changes
    pub auto_save: bool,
    
    /// Backend-specific configuration
    pub backend_config: BackendSpecificConfig,
}

/// Supported storage backend types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum BackendType {
    /// Google Cloud Storage
    GCPStorage,
    
    /// Local file system
    LocalFileSystem,
    
    // Future expansion
}

/// Backend-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum BackendSpecificConfig {
    /// Google Cloud Storage configuration
    GCP(GCPStorageConfig),
    
    /// Local file system configuration
    Local(LocalFileSystemConfig),
    
    // Future expansion
}

/// Google Cloud Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct GCPStorageConfig {
    /// GCP project ID
    pub project_id: String,
    
    /// GCP bucket name
    pub bucket_name: String,
    
    /// Base path within the bucket
    pub base_path: String,
    
    /// Authentication method
    pub auth_method: GCPAuthMethod,
}

/// Local file system configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct LocalFileSystemConfig {
    /// Base directory path
    pub base_dir: String,
    
    /// File extension for stored data
    pub file_extension: String,
}

/// GCP authentication methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum GCPAuthMethod {
    /// Application Default Credentials
    ADC,
    
    /// Service Account Key
    ServiceAccount(String),
}

impl Default for PersistentSharedMemoryConfig {
    fn default() -> Self {
        Self {
            base: SharedMemoryConfig::default(),
            persistence: PersistenceConfig::default(),
        }
    }
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            backend_type: BackendType::LocalFileSystem,
            sync_interval: Duration::from_secs(60), // 1 minute
            auto_load: true,
            auto_save: true,
            backend_config: BackendSpecificConfig::Local(LocalFileSystemConfig::default()),
        }
    }
}

impl Default for LocalFileSystemConfig {
    fn default() -> Self {
        Self {
            base_dir: std::env::temp_dir().to_string_lossy().to_string(),
            file_extension: "json".to_string(),
        }
    }
}

impl ProviderSpecificConfig for PersistentSharedMemoryConfig {
    /// Validates the configuration values
    ///
    /// # Validation Rules
    ///
    /// - Base SharedMemoryConfig must be valid
    /// - For GCPStorage backend:
    ///   - project_id must not be empty
    ///   - bucket_name must not be empty
    /// - For LocalFileSystem backend:
    ///   - base_dir must not be empty
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate base configuration
        self.base.validate()?;
        
        // Validate persistence configuration
        match self.persistence.backend_type {
            BackendType::GCPStorage => {
                if let BackendSpecificConfig::GCP(ref config) = self.persistence.backend_config {
                    // Validate GCP configuration
                    if config.project_id.is_empty() {
                        return Err(ConfigError::InvalidValue {
                            field: "project_id".to_string(),
                            message: "Project ID cannot be empty".to_string(),
                        });
                    }
                    
                    if config.bucket_name.is_empty() {
                        return Err(ConfigError::InvalidValue {
                            field: "bucket_name".to_string(),
                            message: "Bucket name cannot be empty".to_string(),
                        });
                    }
                } else {
                    return Err(ConfigError::InvalidValue {
                        field: "backend_config".to_string(),
                        message: "GCP backend type requires GCP configuration".to_string(),
                    });
                }
            },
            BackendType::LocalFileSystem => {
                if let BackendSpecificConfig::Local(ref config) = self.persistence.backend_config {
                    // Validate local file system configuration
                    if config.base_dir.is_empty() {
                        return Err(ConfigError::InvalidValue {
                            field: "base_dir".to_string(),
                            message: "Base directory cannot be empty".to_string(),
                        });
                    }
                } else {
                    return Err(ConfigError::InvalidValue {
                        field: "backend_config".to_string(),
                        message: "LocalFileSystem backend type requires Local configuration".to_string(),
                    });
                }
            },
        }
        
        Ok(())
    }
    
    /// Merges default values for unspecified fields
    fn merge_defaults(&mut self) {
        // Merge defaults for base configuration
        self.base.merge_defaults();
        
        // Set default sync interval if not specified
        if self.persistence.sync_interval.as_millis() == 0 {
            self.persistence.sync_interval = Duration::from_secs(60); // 1 minute
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = PersistentSharedMemoryConfig::default();
        assert_eq!(config.persistence.backend_type, BackendType::LocalFileSystem);
        assert_eq!(config.persistence.sync_interval, Duration::from_secs(60));
        assert!(config.persistence.auto_load);
        assert!(config.persistence.auto_save);
    }
    
    #[test]
    fn test_validate_gcp_config() {
        let mut config = PersistentSharedMemoryConfig::default();
        config.persistence.backend_type = BackendType::GCPStorage;
        config.persistence.backend_config = BackendSpecificConfig::GCP(GCPStorageConfig {
            project_id: "".to_string(),
            bucket_name: "test-bucket".to_string(),
            base_path: "test-path".to_string(),
            auth_method: GCPAuthMethod::ADC,
        });
        
        // Empty project_id should fail validation
        assert!(config.validate().is_err());
        
        // Set project_id but empty bucket_name should fail
        if let BackendSpecificConfig::GCP(ref mut gcp_config) = config.persistence.backend_config {
            gcp_config.project_id = "test-project".to_string();
            gcp_config.bucket_name = "".to_string();
        }
        assert!(config.validate().is_err());
        
        // Valid configuration should pass
        if let BackendSpecificConfig::GCP(ref mut gcp_config) = config.persistence.backend_config {
            gcp_config.bucket_name = "test-bucket".to_string();
        }
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_validate_local_config() {
        let mut config = PersistentSharedMemoryConfig::default();
        config.persistence.backend_config = BackendSpecificConfig::Local(LocalFileSystemConfig {
            base_dir: "".to_string(),
            file_extension: "json".to_string(),
        });
        
        // Empty base_dir should fail validation
        assert!(config.validate().is_err());
        
        // Valid configuration should pass
        if let BackendSpecificConfig::Local(ref mut local_config) = config.persistence.backend_config {
            local_config.base_dir = "/tmp".to_string();
        }
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_backend_type_mismatch() {
        let mut config = PersistentSharedMemoryConfig::default();
        config.persistence.backend_type = BackendType::GCPStorage;
        // But backend_config is still Local
        
        // Type mismatch should fail validation
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_merge_defaults() {
        let mut config = PersistentSharedMemoryConfig {
            base: SharedMemoryConfig::default(),
            persistence: PersistenceConfig {
                backend_type: BackendType::LocalFileSystem,
                sync_interval: Duration::from_secs(0), // This should be replaced with default
                auto_load: false,
                auto_save: false,
                backend_config: BackendSpecificConfig::Local(LocalFileSystemConfig::default()),
            },
        };
        
        config.merge_defaults();
        
        // sync_interval should be set to default
        assert_eq!(config.persistence.sync_interval, Duration::from_secs(60));
    }
}
