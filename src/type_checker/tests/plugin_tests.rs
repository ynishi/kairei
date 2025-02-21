use crate::{
    ast::{Expression, Literal, ThinkAttributes},
    type_checker::{visitor::common::PluginVisitor, TypeCheckResult, TypeContext},
};

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
