use crate::{
    ast::Expression,
    config::ProviderConfig,
    type_checker::{TypeCheckResult, TypeContext, TypeCheckerPlugin},
};

pub struct PluginConfigValidator;

impl TypeCheckerPlugin for PluginConfigValidator {
    fn before_think(&self, expr: &mut Expression, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        if let Expression::Think { config, .. } = expr {
            if let Some(config_map) = config {
                ProviderConfig::try_from(config_map.clone())?;
            }
        }
        Ok(())
    }
}
