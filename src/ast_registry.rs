use std::sync::Arc;

use dashmap::DashMap;

use crate::{
    ast, parse_root, EventsDef, HandlersDef, MicroAgentDef, RuntimeError, RuntimeResult, WorldDef,
};

#[derive(Debug, Clone, Default)]
pub struct AstRegistry {
    asts: Arc<DashMap<String, Arc<MicroAgentDef>>>,
}

impl AstRegistry {
    pub async fn create_ast_from_dsl(&self, dsl: &str) -> RuntimeResult<ast::Root> {
        let (_, root) = parse_root(dsl).map_err(|e| {
            RuntimeError::Execution(crate::ExecutionError::ASTError(format!(
                "Failed to parse DSL: {}",
                e
            )))
        })?;
        Ok(root)
    }
    pub async fn register_agent_ast(
        &mut self,
        _agent_name: &str,
        _ast: &MicroAgentDef,
    ) -> RuntimeResult<()> {
        self.asts
            .insert(_agent_name.to_string(), Arc::new(_ast.clone()));
        Ok(())
    }

    pub async fn get_agent_ast(&self, agent_name: &str) -> RuntimeResult<Arc<MicroAgentDef>> {
        let ast = self.asts.get(agent_name).ok_or(RuntimeError::Execution(
            crate::ExecutionError::ASTNotFound(agent_name.to_string()),
        ))?;
        Ok(ast.value().clone())
    }

    pub async fn list_agent_asts(&self) -> Vec<String> {
        self.asts.iter().map(|entry| entry.key().clone()).collect()
    }

    // factory method for creating a world AST
    pub fn create_world_ast(&self) -> WorldDef {
        WorldDef {
            name: "world".to_string(),
            config: None,
            events: EventsDef { events: vec![] },
            handlers: HandlersDef { handlers: vec![] },
        }
    }
}
