use crate::{
    ast::{Expression, Literal, ThinkAttributes},
    type_checker::{
        visitor::common::PluginVisitor, PluginConfigValidator, TypeChecker, TypeCheckResult,
        TypeContext,
    },
};
use std::collections::HashMap;

/// Test plugin implementation that tracks lifecycle calls
struct TestPlugin {
    before_called: bool,
    after_called: bool,
}

impl TestPlugin {
    fn new() -> Self {
        Self {
            before_called: false,
            after_called: false,
        }
    }
}

impl PluginVisitor for TestPlugin {
    fn before_expression(
        &mut self,
        _expr: &Expression,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        self.before_called = true;
        Ok(())
    }

    fn after_expression(
        &mut self,
        _expr: &Expression,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        self.after_called = true;
        Ok(())
    }
}

#[test]
fn test_plugin_lifecycle() {
    let mut plugin = TestPlugin::new();
    let mut ctx = TypeContext::default();

    // Test expression visit
    let expr = Expression::Literal(Literal::Integer(42));

    // Before hook should be called
    assert!(!plugin.before_called);
    plugin.before_expression(&expr, &mut ctx).unwrap();
    assert!(plugin.before_called);

    // After hook should be called
    assert!(!plugin.after_called);
    plugin.after_expression(&expr, &mut ctx).unwrap();
    assert!(plugin.after_called);
}

#[test]
fn test_plugin_with_think_attributes() {
    let mut plugin = TestPlugin::new();
    let mut ctx = TypeContext::default();

    let expr = Expression::Think {
        args: vec![],
        with_block: Some(ThinkAttributes::default()),
    };

    // Plugin hooks should be called for think expressions
    plugin.before_expression(&expr, &mut ctx).unwrap();
    assert!(plugin.before_called);
    plugin.after_expression(&expr, &mut ctx).unwrap();
    assert!(plugin.after_called);
}

#[test]
fn test_plugin_config_validation() {
    let mut type_checker = TypeChecker::new();
    type_checker.register_plugin(Box::new(PluginConfigValidator));
    let mut ctx = TypeContext::new();

    // Test valid config
    let mut valid_config = HashMap::new();
    valid_config.insert("provider_type".to_string(), Literal::String("test".to_string()));
    valid_config.insert("name".to_string(), Literal::String("test".to_string()));

    let expr = Expression::Think {
        args: vec![],
        with_block: Some(ThinkAttributes {
            plugins: {
                let mut plugins = HashMap::new();
                plugins.insert("provider".to_string(), valid_config);
                plugins
            },
            ..Default::default()
        }),
    };
    assert!(type_checker.visit_expression(&expr, &mut ctx).is_ok());

    // Test invalid config (missing required fields)
    let invalid_config = HashMap::new();
    let expr = Expression::Think {
        args: vec![],
        with_block: Some(ThinkAttributes {
            plugins: {
                let mut plugins = HashMap::new();
                plugins.insert("provider".to_string(), invalid_config);
                plugins
            },
            ..Default::default()
        }),
    };
    assert!(type_checker.visit_expression(&expr, &mut ctx).is_err());
}

#[test]
fn test_plugin_config_validation_error_messages() {
    let mut type_checker = TypeChecker::new();
    type_checker.register_plugin(Box::new(PluginConfigValidator));
    let mut ctx = TypeContext::new();

    // Test missing provider_type error
    let mut config = HashMap::new();
    config.insert("name".to_string(), Literal::String("test".to_string()));
    
    let expr = Expression::Think {
        args: vec![],
        with_block: Some(ThinkAttributes {
            plugins: {
                let mut plugins = HashMap::new();
                plugins.insert("provider".to_string(), config);
                plugins
            },
            ..Default::default()
        }),
    };
    let result = type_checker.visit_expression(&expr, &mut ctx);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Missing required field 'provider_type'"));

    // Test missing name error
    let mut config = HashMap::new();
    config.insert("provider_type".to_string(), Literal::String("test".to_string()));
    
    let expr = Expression::Think {
        args: vec![],
        with_block: Some(ThinkAttributes {
            plugins: {
                let mut plugins = HashMap::new();
                plugins.insert("provider".to_string(), config);
                plugins
            },
            ..Default::default()
        }),
    };
    let result = type_checker.visit_expression(&expr, &mut ctx);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Missing required field 'name'"));
}
