use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;

use crate::{
    ast, config::AgentConfig, parse_root, ASTError, ASTResult, AnswerDef, EventsDef, Expression,
    HandlerBlock, HandlersDef, Literal, MicroAgentDef, RequestHandler, RequestType,
    StateAccessPath, StateDef, StateVarDef, Statement, TypeInfo, WorldDef,
};
#[derive(Debug, Clone, Default)]
pub struct AstRegistry {
    asts: Arc<DashMap<String, Arc<MicroAgentDef>>>,
}

impl AstRegistry {
    pub async fn create_ast_from_dsl(&self, dsl: &str) -> ASTResult<ast::Root> {
        let (_, root) = parse_root(dsl).map_err(|e| ASTError::ParseError {
            message: format!("failed to parse DSL {}", e),
            target: "root".to_string(),
        })?;
        Ok(root)
    }
    pub async fn register_agent_ast(
        &mut self,
        _agent_name: &str,
        _ast: &MicroAgentDef,
    ) -> ASTResult<()> {
        self.asts
            .insert(_agent_name.to_string(), Arc::new(_ast.clone()));
        Ok(())
    }

    pub async fn get_agent_ast(&self, agent_name: &str) -> ASTResult<Arc<MicroAgentDef>> {
        let ast = self
            .asts
            .get(agent_name)
            .ok_or(ASTError::ASTNotFound(agent_name.to_string()))?;
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

    pub async fn create_builtin_agent_asts(
        &self,
        config: &AgentConfig,
    ) -> ASTResult<Vec<MicroAgentDef>> {
        let config = config.clone().scale_manager.unwrap_or_default();
        let scale_manager_def = MicroAgentDef {
            name: "scale_manager".to_string(),
            state: Some(StateDef {
                variables: {
                    let mut vars = HashMap::new();
                    vars.insert(
                        "enabled".to_string(),
                        StateVarDef {
                            name: "enabled".to_string(),
                            type_info: TypeInfo::Simple("boolean".to_string()),
                            initial_value: Some(Expression::Literal(Literal::Boolean(
                                config.enabled,
                            ))),
                        },
                    );
                    vars.insert(
                        "max_instances_per_agent".to_string(),
                        StateVarDef {
                            name: "self.max_instances_per_agent".to_string(),
                            type_info: TypeInfo::Simple("i64".to_string()),
                            initial_value: Some(Expression::Literal(Literal::Integer(
                                config.max_instances_per_agent as i64,
                            ))),
                        },
                    );
                    vars
                },
            }),
            // simply return the value of max_instances_per_agent for agent request event.
            answer: Some(AnswerDef {
                handlers: vec![RequestHandler {
                    request_type: RequestType::Custom("get_max_instances_per_agent".to_string()),
                    parameters: vec![],
                    return_type: TypeInfo::Simple("i64".to_string()),
                    constraints: None,
                    block: HandlerBlock {
                        statements: vec![Statement::Return(Expression::StateAccess(
                            StateAccessPath(vec!["self".into(), "max_instances_per_agent".into()]),
                        ))],
                    },
                }],
            }),
            ..Default::default()
        };
        Ok(vec![scale_manager_def])
    }
}
