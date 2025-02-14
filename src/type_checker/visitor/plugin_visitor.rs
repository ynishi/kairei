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
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Validate input query type
        self.validate_value_type(&request.input.query)?;

        // Validate plugin parameters
        for value in request.input.parameters.values() {
            self.validate_value_type(value)?;
        }

        // Validate plugin configuration
        if let Some(config) = request
            .config
            .plugin_configs
            .get(&format!("{:?}", plugin.capability()))
        {
            self.validate_plugin_config(config, _ctx)?;
        }

        Ok(())
    }

    /// Validate plugin response type compatibility
    pub fn validate_plugin_response(
        &self,
        response: &ProviderResponse,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Validate output string
        if response.output.is_empty() {
            return Err(TypeCheckError::PluginTypeError {
                message: "Plugin response output cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Validate plugin configuration
    pub fn validate_plugin_config(
        &self,
        config: &crate::config::PluginConfig,
        _ctx: &mut TypeContext,
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
            }
            PluginConfig::Search(_) | PluginConfig::Unknown(_) => {}
        }
        Ok(())
    }

    /// Validate a value's type
    #[allow(clippy::only_used_in_recursion)]
    fn validate_value_type(&self, value: &Value) -> TypeCheckResult<()> {
        match value {
            Value::String(_) => Ok(()),
            Value::Integer(_) => Ok(()),
            Value::Float(_) => Ok(()),
            Value::Boolean(_) => Ok(()),
            Value::List(items) => {
                for item in items {
                    self.validate_value_type(item)?;
                }
                Ok(())
            }
            Value::Map(map) => {
                for value in map.values() {
                    self.validate_value_type(value)?;
                }
                Ok(())
            }
            _ => Err(TypeCheckError::PluginTypeError {
                message: format!("Unsupported value type for plugin: {:?}", value),
            }),
        }
    }
}
