use kairei::{
    ast::{MicroAgentDef, Root},
    type_checker::{TypeCheckResult, TypeChecker},
};

use super::TestPlugin;

#[test]
fn test_plugin_integration() -> TypeCheckResult<()> {
    // Create an AST using plugin features
    let mut root = Root {
        micro_agent_defs: vec![MicroAgentDef {
            name: "PluginAgent".to_string(),
            ..Default::default()
        }],
        world_def: None,
    };

    let mut checker = TypeChecker::new();
    checker.register_plugin(Box::new(TestPlugin));
    checker.check_types(&mut root)
}
