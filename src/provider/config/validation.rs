use super::base::ConfigError;

pub fn validate_required_field<T>(field: &Option<T>, field_name: &str) -> Result<(), ConfigError> {
    field
        .as_ref()
        .ok_or_else(|| ConfigError::MissingField(field_name.to_string()))?;
    Ok(())
}

pub fn validate_range<T>(value: T, min: T, max: T, field_name: &str) -> Result<(), ConfigError>
where
    T: PartialOrd + std::fmt::Display,
{
    if value < min || value > max {
        return Err(ConfigError::InvalidValue {
            field: field_name.to_string(),
            message: format!("Value must be between {} and {}", min, max),
        });
    }
    Ok(())
}
