use crate::{
    config::PluginConfig,
    eval::expression::Value,
    provider::{
        plugin::ProviderPlugin,
        request::{ProviderRequest, ProviderResponse},
    },
    type_checker::{TypeCheckError, TypeCheckResult, TypeContext},
};

/// Visitor for plugin-specific type checking
pub struct PluginTypeVisitor;

impl PluginTypeVisitor {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Validate plugin request type compatibility
    pub fn validate_plugin_request(
        &self,
        request: &ProviderRequest,
        plugin: &dyn ProviderPlugin,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Validate input query type
        self.validate_value_type(&request.input.query)?;

        // Validate plugin parameters
        for (_param_name, value) in &request.input.parameters {
            // For now, just validate the value type
            // In a full implementation, we would check against plugin-specific parameter types
            self.validate_value_type(value)?;
        }

        // Validate plugin configuration
        if let Some(config) = request
            .config
            .plugin_configs
            .get(&format!("{:?}", plugin.capability()))
        {
            self.validate_plugin_config(config, ctx)?;
        } else {
            // Configuration is required for most plugins
            return Err(TypeCheckError::InvalidPluginConfig {
                message: format!(
                    "Missing configuration for plugin capability: {:?}",
                    plugin.capability()
                ),
            });
        }

        Ok(())
    }

    /// Validate plugin response type compatibility
    pub fn validate_plugin_response(
        &self,
        response: &ProviderResponse,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Validate output string
        if response.output.is_empty() {
            return Err(TypeCheckError::PluginTypeError {
                message: "Plugin response output cannot be empty".to_string(),
            });
        }

        // Validate metadata
        let timestamp = response.metadata.timestamp.to_string();
        if timestamp.is_empty() {
            return Err(TypeCheckError::PluginTypeError {
                message: "Response metadata timestamp cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Validate plugin configuration
    pub fn validate_plugin_config(
        &self,
        config: &crate::config::PluginConfig,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Validate configuration values based on plugin type
        match config {
            PluginConfig::Memory(config) => {
                if config.importance_threshold < 0.0 || config.importance_threshold > 1.0 {
                    return Err(TypeCheckError::InvalidPluginConfig {
                        message: format!(
                            "Importance threshold must be between 0 and 1, got {}",
                            config.importance_threshold
                        ),
                    });
                }
                if config.max_items < 1 {
                    return Err(TypeCheckError::InvalidPluginConfig {
                        message: format!(
                            "Max items must be positive, got {}",
                            config.max_items
                        ),
                    });
                }
            }
            PluginConfig::Rag(config) => {
                if config.similarity_threshold < 0.0 || config.similarity_threshold > 1.0 {
                    return Err(TypeCheckError::InvalidPluginConfig {
                        message: format!(
                            "Similarity threshold must be between 0 and 1, got {}",
                            config.similarity_threshold
                        ),
                    });
                }
                if config.max_results < 1 {
                    return Err(TypeCheckError::InvalidPluginConfig {
                        message: format!(
                            "Max results must be positive, got {}",
                            config.max_results
                        ),
                    });
                }
            }
            PluginConfig::Search(config) => {
                if config.max_results < 1 {
                    return Err(TypeCheckError::InvalidPluginConfig {
                        message: format!(
                            "Max results must be positive, got {}",
                            config.max_results
                        ),
                    });
                }
            }
            PluginConfig::Unknown(config) => {
                return Err(TypeCheckError::InvalidPluginConfig {
                    message: format!("Unknown plugin configuration type: {:?}", config),
                });
            }
        }
        Ok(())
    }

    /// Validate a value's type
    fn validate_value_type(&self, value: &Value) -> TypeCheckResult<()> {
        match value {
            Value::String(s) => {
                if s.is_empty() {
                    return Err(TypeCheckError::PluginTypeError {
                        message: "String value cannot be empty".to_string(),
                    });
                }
                Ok(())
            }
            Value::Integer(i) => {
                if *i < i64::MIN || *i > i64::MAX {
                    return Err(TypeCheckError::PluginTypeError {
                        message: format!("Integer value {} out of range", i),
                    });
                }
                Ok(())
            }
            Value::Float(f) => {
                if !f.is_finite() {
                    return Err(TypeCheckError::PluginTypeError {
                        message: format!("Float value must be finite, got {}", f),
                    });
                }
                Ok(())
            }
            Value::Boolean(_) => Ok(()),
            Value::List(items) => {
                if items.is_empty() {
                    return Err(TypeCheckError::PluginTypeError {
                        message: "List value cannot be empty".to_string(),
                    });
                }
                for (index, item) in items.iter().enumerate() {
                    self.validate_value_type(item).map_err(|e| TypeCheckError::PluginTypeError {
                        message: format!("Invalid list item at index {}: {}", index, e),
                    })?;
                }
                Ok(())
            }
            Value::Map(map) => {
                if map.is_empty() {
                    return Err(TypeCheckError::PluginTypeError {
                        message: "Map value cannot be empty".to_string(),
                    });
                }
                for (key, value) in map {
                    if key.is_empty() {
                        return Err(TypeCheckError::PluginTypeError {
                            message: "Map key cannot be empty".to_string(),
                        });
                    }
                    self.validate_value_type(value).map_err(|e| TypeCheckError::PluginTypeError {
                        message: format!("Invalid map value for key '{}': {}", key, e),
                    })?;
                }
                Ok(())
            }
            Value::Null => Err(TypeCheckError::PluginTypeError {
                message: "Null values are not supported in plugin values".to_string(),
            }),
            _ => Err(TypeCheckError::PluginTypeError {
                message: format!("Unsupported value type for plugin: {:?}", value),
            }),
        }
    }
}
