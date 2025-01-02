use std::sync::Arc;

use dashmap::DashMap;

use crate::{MicroAgentDef, RuntimeError, RuntimeResult};

#[derive(Debug, Clone, Default)]
pub struct AstRegistry {
    asts: Arc<DashMap<String, Arc<MicroAgentDef>>>,
}

impl AstRegistry {
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
}
