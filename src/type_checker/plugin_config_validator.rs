use crate::{
    ast,
    config::ProviderConfig,
    eval::expression::Value,
    type_checker::{visitor::common::PluginVisitor, TypeCheckResult, TypeContext},
};
use std::collections::HashMap;

pub struct PluginConfigValidator;

impl PluginVisitor for PluginConfigValidator {
    fn before_expression(
        &mut self,
        expr: &ast::Expression,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        if let ast::Expression::Think {
            args: _,
            with_block: Some(attrs),
        } = expr
        {
            if let Some(config) = attrs.plugins.get("provider") {
                let config_map: HashMap<String, Value> = config
                    .iter()
                    .map(|(k, v)| match v {
                        ast::Literal::String(s) => (k.clone(), Value::String(s.clone())),
                        _ => (k.clone(), Value::String(v.to_string())),
                    })
                    .collect();
                ProviderConfig::try_from(config_map)?;
            }
        }
        Ok(())
    }
}
