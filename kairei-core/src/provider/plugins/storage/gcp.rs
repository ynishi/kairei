//! Google Cloud Storage implementation of StorageBackend for PersistentSharedMemoryPlugin.
//!
//! This module provides a GCP Storage-based persistence layer for shared memory data,
//! storing each namespace as a separate object in a GCP Storage bucket.
//!
//! # Features
//!
//! - GCP Storage-based persistence for shared memory data
//! - Namespace isolation using object naming
//! - Authentication via service account or application default credentials
//! - Automatic error handling and retries with jitter
//! - Optional compression for storage efficiency
//!
//! # Example
//!
//! ```no_run
//! use kairei_core::provider::plugins::storage::gcp::GCPStorageBackend;
//! use kairei_core::provider::config::plugins::{GCPStorageConfig, GCPAuthMethod};
//!
//! let config = GCPStorageConfig {
//!     project_id: "my-project".to_string(),
//!     bucket_name: "shared-memory-bucket".to_string(),
//!     base_path: "shared-memory".to_string(),
//!     auth_method: GCPAuthMethod::ADC,
//! };
//!
//! let backend = GCPStorageBackend::new(config).expect("Failed to create GCP backend");
//! ```

use async_trait::async_trait;
use cloud_storage::{Client, Error as GCPError};
use rand::Rng;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, warn};

use crate::provider::capabilities::storage::{StorageBackend, StorageError, ValueWithMetadata};
use crate::provider::config::plugins::{GCPAuthMethod, GCPStorageConfig};

const MAX_RETRIES: usize = 3;
const RETRY_DELAY_MS: u64 = 200;
const MAX_JITTER_MS: u64 = 100;

/// Google Cloud Storage implementation of the StorageBackend trait.
///
/// This backend stores shared memory data in JSON objects in a GCP Storage bucket,
/// with each namespace corresponding to a separate object.
#[derive(Clone)]
pub struct GCPStorageBackend {
    /// Configuration for the GCP Storage backend
    config: Arc<GCPStorageConfig>,

    /// GCP Storage client
    client: Arc<Client>,

    /// Whether to use compression
    use_compression: bool,
}

impl GCPStorageBackend {
    /// Create a new GCPStorageBackend with the given configuration
    ///
    /// # Arguments
    /// * `config` - Configuration for the GCP Storage backend
    ///
    /// # Returns
    /// * `Result<Self, StorageError>` - A new instance or an error if creation fails
    ///
    /// # Errors
    /// * `StorageError::AuthenticationError` - If authentication fails
    /// * `StorageError::ConfigurationError` - If configuration is invalid
    pub fn new(config: GCPStorageConfig) -> Result<Self, StorageError> {
        // Create GCP client based on authentication method
        let client = match &config.auth_method {
            GCPAuthMethod::ADC => {
                // Using default client with application default credentials
                Client::new()
            }
            GCPAuthMethod::ServiceAccount(_key_json) => {
                // Log warning that service account auth is not fully implemented
                // in the current version of cloud-storage crate
                warn!(
                    "Service account authentication is not fully implemented in the current version of cloud-storage crate. Using default client."
                );

                // For now, use the default client
                // In a production environment, this would need to be properly implemented
                Client::new()
            }
        };

        Ok(Self {
            config: Arc::new(config),
            client: Arc::new(client),
            use_compression: false, // Default to no compression
        })
    }

    /// Create a new GCPStorageBackend with the given configuration and compression enabled
    ///
    /// # Arguments
    /// * `config` - Configuration for the GCP Storage backend
    /// * `use_compression` - Whether to use compression for stored data
    ///
    /// # Returns
    /// * `Result<Self, StorageError>` - A new instance or an error if creation fails
    pub fn new_with_compression(
        config: GCPStorageConfig,
        use_compression: bool,
    ) -> Result<Self, StorageError> {
        let mut backend = Self::new(config)?;
        backend.use_compression = use_compression;
        Ok(backend)
    }

    /// Get the object name for a specific namespace
    ///
    /// # Arguments
    /// * `namespace` - The namespace to get the object name for
    ///
    /// # Returns
    /// * `String` - The object name for the namespace
    fn get_object_name(&self, namespace: &str) -> String {
        let sanitized_namespace = sanitize_namespace(namespace);

        // Combine base path and namespace to form the object name
        if self.config.base_path.is_empty() {
            format!("{}.json", sanitized_namespace)
        } else {
            format!("{}/{}.json", self.config.base_path, sanitized_namespace)
        }
    }

    /// Convert GCP errors to StorageErrors
    ///
    /// # Arguments
    /// * `error` - The GCP error to convert
    /// * `context` - Additional context for the error
    ///
    /// # Returns
    /// * `StorageError` - The converted error
    fn map_gcp_error(&self, error: GCPError, context: &str) -> StorageError {
        // Map GCP errors to StorageErrors based on error message content
        // since the cloud-storage crate doesn't have specific error variants
        let error_msg = error.to_string().to_lowercase();

        if error_msg.contains("not found") || error_msg.contains("404") {
            StorageError::FileNotFound(context.to_string())
        } else if error_msg.contains("forbidden") || error_msg.contains("403") {
            StorageError::AccessDenied(format!("Access denied: {}", context))
        } else if error_msg.contains("unauthorized") || error_msg.contains("401") {
            StorageError::AuthenticationError(format!("Authentication failed: {}", context))
        } else {
            StorageError::StorageError(format!("GCP error: {} ({})", error, context))
        }
    }

    /// Helper function to perform operations with retries and jitter
    ///
    /// # Arguments
    /// * `operation` - The operation to perform
    /// * `context` - Additional context for error messages
    ///
    /// # Returns
    /// * `Result<T, StorageError>` - The result of the operation or an error
    async fn with_retries<F, Fut, T>(&self, operation: F, context: &str) -> Result<T, StorageError>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T, GCPError>> + Send,
        T: Send,
    {
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Store the error for potential retry
                    last_error = Some(e);

                    // Don't retry for certain errors (auth, permission)
                    let error_msg = last_error.as_ref().unwrap().to_string().to_lowercase();
                    if error_msg.contains("forbidden")
                        || error_msg.contains("403")
                        || error_msg.contains("unauthorized")
                        || error_msg.contains("401")
                    {
                        break;
                    }

                    // Log retry attempt
                    if attempt < MAX_RETRIES - 1 {
                        warn!(
                            "GCP operation failed (attempt {}/{}): {}. Retrying...",
                            attempt + 1,
                            MAX_RETRIES,
                            last_error.as_ref().unwrap()
                        );

                        // Exponential backoff with jitter
                        let base_delay = RETRY_DELAY_MS * (2_u64.pow(attempt as u32));
                        // Create a new RNG instance each time to avoid Send issues
                        let jitter = Self::calculate_jitter();
                        let delay = base_delay + jitter;

                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        // All retries failed
        Err(self.map_gcp_error(last_error.unwrap(), context))
    }

    /// Calculate jitter for backoff strategy
    ///
    /// This is a separate function to avoid Send issues with ThreadRng
    fn calculate_jitter() -> u64 {
        rand::thread_rng().gen_range(0..MAX_JITTER_MS)
    }

    /// Compress data using gzip
    ///
    /// # Arguments
    /// * `data` - The data to compress
    ///
    /// # Returns
    /// * `Result<Vec<u8>, StorageError>` - The compressed data or an error
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, StorageError> {
        use flate2::{Compression, write::GzEncoder};
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).map_err(|e| {
            StorageError::SerializationError(format!("Failed to compress data: {}", e))
        })?;

        encoder.finish().map_err(|e| {
            StorageError::SerializationError(format!("Failed to finish compression: {}", e))
        })
    }

    /// Decompress data using gzip
    ///
    /// # Arguments
    /// * `data` - The compressed data
    ///
    /// # Returns
    /// * `Result<Vec<u8>, StorageError>` - The decompressed data or an error
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, StorageError> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder.read_to_end(&mut decompressed).map_err(|e| {
            StorageError::DeserializationError(format!("Failed to decompress data: {}", e))
        })?;

        Ok(decompressed)
    }
}

/// Sanitize a namespace string to be safe for GCP object names
///
/// # Arguments
/// * `namespace` - The namespace to sanitize
///
/// # Returns
/// * `String` - The sanitized namespace
pub fn sanitize_namespace(namespace: &str) -> String {
    // Replace characters that are problematic in GCP object names
    namespace.replace(['/', '\\', '?', '*', ':', '<', '>', '|', '"', '#'], "_")
}

#[async_trait]
impl StorageBackend for GCPStorageBackend {
    fn clone_backend(&self) -> Box<dyn StorageBackend> {
        Box::new(self.clone())
    }

    async fn load(
        &self,
        namespace: &str,
    ) -> Result<HashMap<String, ValueWithMetadata>, StorageError> {
        let object_name = self.get_object_name(namespace);
        let bucket_name = &self.config.bucket_name;

        // Try to download the object directly
        // If it doesn't exist, we'll get a 404 error which we'll handle
        let object_data = match self
            .with_retries(
                || {
                    let client = self.client.clone();
                    let bucket_name = bucket_name.clone();
                    let object_name = object_name.clone();

                    async move { client.object().download(&bucket_name, &object_name).await }
                },
                &format!("Downloading object: {}/{}", bucket_name, object_name),
            )
            .await
        {
            Ok(data) => data,
            Err(e) => {
                // If the error is "not found", return an empty HashMap
                if e.to_string().to_lowercase().contains("not found")
                    || e.to_string().contains("404")
                {
                    return Ok(HashMap::new());
                }
                // Otherwise, return the error
                return Err(e);
            }
        };

        // Decompress if needed
        let data_to_deserialize = if self.use_compression {
            self.decompress_data(&object_data)?
        } else {
            object_data
        };

        // Deserialize the content
        match serde_json::from_slice(&data_to_deserialize) {
            Ok(data) => Ok(data),
            Err(e) => Err(StorageError::DeserializationError(format!(
                "Failed to deserialize data from {}/{}: {}",
                bucket_name, object_name, e
            ))),
        }
    }

    async fn save(
        &self,
        namespace: &str,
        data: &HashMap<String, ValueWithMetadata>,
    ) -> Result<(), StorageError> {
        let object_name = self.get_object_name(namespace);
        let bucket_name = &self.config.bucket_name;

        // Serialize the data
        let serialized = serde_json::to_vec(data).map_err(|e| {
            StorageError::SerializationError(format!("Failed to serialize data: {}", e))
        })?;

        // Compress if needed
        let content = if self.use_compression {
            self.compress_data(&serialized)?
        } else {
            serialized
        };

        // Upload to GCP Storage
        self.with_retries(
            || {
                let client = self.client.clone();
                let bucket_name = bucket_name.clone();
                let object_name = object_name.clone();
                let content = content.clone();

                async move {
                    let content_type = if self.use_compression {
                        "application/gzip"
                    } else {
                        "application/json"
                    };

                    client
                        .object()
                        .create(&bucket_name, content, content_type, &object_name)
                        .await
                }
            },
            &format!("Uploading object: {}/{}", bucket_name, object_name),
        )
        .await?;

        Ok(())
    }

    async fn save_key(
        &self,
        namespace: &str,
        key: &str,
        value: &ValueWithMetadata,
    ) -> Result<(), StorageError> {
        // Load existing data
        let mut data = self.load(namespace).await?;

        // Update or insert the key
        data.insert(key.to_string(), value.clone());

        // Save the updated data
        self.save(namespace, &data).await
    }

    async fn delete_key(&self, namespace: &str, key: &str) -> Result<(), StorageError> {
        // Load existing data
        let mut data = self.load(namespace).await?;

        // Remove the key (if it exists)
        if data.remove(key).is_none() {
            // Key wasn't found, which is fine (not an error)
            return Ok(());
        }

        // If this was the last key, delete the object
        if data.is_empty() {
            let object_name = self.get_object_name(namespace);
            let bucket_name = &self.config.bucket_name;

            // Delete the object
            self.with_retries(
                || {
                    let client = self.client.clone();
                    let bucket_name = bucket_name.clone();
                    let object_name = object_name.clone();

                    async move { client.object().delete(&bucket_name, &object_name).await }
                },
                &format!("Deleting object: {}/{}", bucket_name, object_name),
            )
            .await?;
        } else {
            // Save the updated data
            self.save(namespace, &data).await?;
        }

        Ok(())
    }

    async fn is_available(&self) -> bool {
        // Check if we can access the bucket by listing it
        // The cloud-storage crate doesn't have a direct "exists" method for buckets
        match self.client.bucket().list().await {
            Ok(_) => {
                // If we can list buckets, the service is available
                true
            }
            Err(e) => {
                // Log the error but return false as required by the API
                error!("GCP Storage is not available: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::capabilities::shared_memory::Metadata;
    use chrono::Utc;

    // Helper function to create a test value with metadata
    fn create_test_value(value: serde_json::Value) -> ValueWithMetadata {
        let now = Utc::now();
        ValueWithMetadata {
            value,
            metadata: Metadata {
                created_at: now,
                last_modified: now,
                content_type: "application/json".to_string(),
                size: 0, // Will be updated by the system
                tags: Default::default(),
            },
            expiry: None,
        }
    }

    #[test]
    fn test_sanitize_namespace() {
        let invalid_namespace = "test/namespace:with*invalid?chars";
        let sanitized = sanitize_namespace(invalid_namespace);

        assert_eq!(sanitized, "test_namespace_with_invalid_chars");
    }

    #[test]
    fn test_get_object_name() {
        // With base path
        let config = GCPStorageConfig {
            project_id: "test-project".to_string(),
            bucket_name: "test-bucket".to_string(),
            base_path: "base/path".to_string(),
            auth_method: GCPAuthMethod::ADC,
        };

        let backend = GCPStorageBackend {
            config: Arc::new(config),
            client: Arc::new(Client::new()),
            use_compression: false,
        };

        let object_name = backend.get_object_name("test-namespace");
        assert_eq!(object_name, "base/path/test-namespace.json");

        // Without base path
        let config = GCPStorageConfig {
            project_id: "test-project".to_string(),
            bucket_name: "test-bucket".to_string(),
            base_path: "".to_string(),
            auth_method: GCPAuthMethod::ADC,
        };

        let backend = GCPStorageBackend {
            config: Arc::new(config),
            client: Arc::new(Client::new()),
            use_compression: false,
        };

        let object_name = backend.get_object_name("test-namespace");
        assert_eq!(object_name, "test-namespace.json");
    }

    // Note: Integration tests for GCP Storage would need to be run against a real GCP Storage bucket
    // or a more sophisticated mock. These will be added in a separate test file.
}
