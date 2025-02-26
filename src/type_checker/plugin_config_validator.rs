use crate::{
    ast,
    config::ProviderConfig,
    eval::expression::Value,
    provider::config::{
        CollectingValidator, TypeCheckerValidator,
    },
    type_checker::{
        error::{Location, TypeCheckError},
        visitor::common::PluginVisitor, TypeCheckResult, TypeContext,
    },
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub struct PluginConfigValidator {
    validator: TypeCheckerValidator,
}

impl Default for PluginConfigValidator {
    fn default() -> Self {
        Self {
            validator: TypeCheckerValidator,
        }
    }
}

impl PluginConfigValidator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PluginVisitor for PluginConfigValidator {
    fn before_expression(
        &mut self,
        expr: &ast::Expression,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        if let ast::Expression::Think {
            args: _,
            with_block: Some(attrs),
        } = expr
        {
            if let Some(config) = attrs.plugins.get("provider") {
                // Convert AST literals to JSON values for validation
                let config_map: HashMap<String, JsonValue> = config
                    .iter()
                    .map(|(k, v)| match v {
                        ast::Literal::String(s) => (k.clone(), JsonValue::String(s.clone())),
                        ast::Literal::Integer(n) => {
                            (
                                k.clone(),
                                JsonValue::Number(serde_json::Number::from(*n)),
                            )
                        }
                        ast::Literal::Float(n) => {
                            (
                                k.clone(),
                                JsonValue::Number(serde_json::Number::from_f64(*n).unwrap_or(
                                    serde_json::Number::from_f64(0.0).unwrap(),
                                )),
                            )
                        }
                        ast::Literal::Boolean(b) => (k.clone(), JsonValue::Bool(*b)),
                        _ => (k.clone(), JsonValue::String(v.to_string())),
                    })
                    .collect();

                // Validate using the new validation framework
                let collector = self.validator.validate_collecting(&config_map);

                // If there are errors, add them to the type context
                if collector.has_errors() {
                    for error in &collector.errors {
                        ctx.add_error(TypeCheckError::invalid_think_block(
                            error.to_string(),
                            Location::default(),
                        ));
                    }

                    // Return the first error
                    if !collector.errors.is_empty() {
                        return Err(TypeCheckError::invalid_think_block(
                            collector.errors[0].to_string(),
                            Location::default(),
                        ));
                    }
                }

                // For backward compatibility
                let value_map: HashMap<String, Value> = config
                    .iter()
                    .map(|(k, v)| match v {
                        ast::Literal::String(s) => (k.clone(), Value::String(s.clone())),
                        _ => (k.clone(), Value::String(v.to_string())),
                    })
                    .collect();
                ProviderConfig::try_from(value_map)?;
            }
        }
        Ok(())
    }
}
