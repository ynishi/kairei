use crate::type_checker::{PluginConfigValidator, TypeChecker};

/// Creates a new TypeChecker with default plugins
pub fn create_type_checker() -> TypeChecker {
    let mut checker = TypeChecker::new();
    checker.register_plugin(Box::new(PluginConfigValidator::new()));
    checker
}
