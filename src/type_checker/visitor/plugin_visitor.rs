use crate::{
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
        _plugin: &dyn ProviderPlugin,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Validate input query type
        self.validate_value_type(&request.input.query)?;

        // Basic type validation for parameters
        for value in request.input.parameters.values() {
            self.validate_value_type(value)?;
        }

        Ok(())
    }

    /// Validate plugin response type compatibility
    pub fn validate_plugin_response(
        &self,
        response: &ProviderResponse,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Basic type validation for response output
        self.validate_value_type(&Value::String(response.output.clone()))?;
        Ok(())
    }

    /// Validate plugin configuration
    pub fn validate_plugin_config(
        &self,
        _config: &crate::config::PluginConfig,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Configuration validation is handled elsewhere
        Ok(())
    }

    /// Validate a value's type
    #[allow(clippy::only_used_in_recursion)]
    fn validate_value_type(&self, value: &Value) -> TypeCheckResult<()> {
        match value {
            Value::String(_) | Value::Integer(_) | Value::Float(_) | Value::Boolean(_) => Ok(()),
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
