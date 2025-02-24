mod base;
mod validation;

#[cfg(test)]
mod tests;

pub use base::{ConfigError, ConfigValidation, PluginConfig};
pub use validation::{
    check_property_type, check_required_properties, validate_range, validate_required_field,
};
