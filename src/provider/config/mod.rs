mod base;
mod validation;

#[cfg(test)]
mod tests;

pub use base::{ConfigError, ConfigValidation, PluginConfig};
pub use validation::{validate_range, validate_required_field, check_required_properties, check_property_type};
