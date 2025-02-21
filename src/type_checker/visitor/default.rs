use crate::{
    ast::{Expression, HandlerBlock, HandlerDef, MicroAgentDef, Root, StateDef, Statement},
    type_checker::{visitor::common::TypeVisitor, TypeCheckError, TypeCheckResult, TypeContext},
    Argument,
};

/// Default implementation of type checking logic
pub struct DefaultVisitor;

impl DefaultVisitor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeVisitor for DefaultVisitor {
    fn visit_root(&mut self, root: &mut Root, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        // Visit world definition if present
        if let Some(world_def) = &mut root.world_def {
            for handler in &world_def.handlers.handlers {
                self.visit_handler(handler, ctx)?;
            }
        }

        // Visit all micro agents
        for agent in &mut root.micro_agent_defs {
            self.visit_micro_agent(agent, ctx)?;
        }

        Ok(())
    }

    fn visit_micro_agent(
        &mut self,
        agent: &mut MicroAgentDef,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Visit state definition if present
        if let Some(state) = &mut agent.state {
            self.visit_state(state, ctx)?;
        }

        // Visit lifecycle handlers if present
        if let Some(lifecycle) = &agent.lifecycle {
            if let Some(init) = &lifecycle.on_init {
                self.visit_handler_block(init, ctx)?;
            }
            if let Some(destroy) = &lifecycle.on_destroy {
                self.visit_handler_block(destroy, ctx)?;
            }
        }

        // Visit answer handlers if present
        if let Some(answer) = &agent.answer {
            for handler in &answer.handlers {
                self.visit_handler_block(&handler.block, ctx)?;
            }
        }

        // Visit observe handlers if present
        if let Some(observe) = &agent.observe {
            for handler in &observe.handlers {
                self.visit_handler_block(&handler.block, ctx)?;
            }
        }

        // Visit react handlers if present
        if let Some(react) = &agent.react {
            for handler in &react.handlers {
                self.visit_handler_block(&handler.block, ctx)?;
            }
        }

        Ok(())
    }

    fn visit_state(&mut self, state: &mut StateDef, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        // Check each state variable's type
        for var_def in &state.variables {
            let var_def = var_def.1;
            if let crate::ast::TypeInfo::Simple(type_name) = &var_def.type_info {
                if !ctx.scope.contains_type(type_name) {
                    return Err(TypeCheckError::UndefinedType(type_name.clone()));
                }
            }

            // If there's an initial value, check its type
            if let Some(init_value) = &var_def.initial_value {
                self.visit_expression(init_value, ctx)?;
            }
        }
        Ok(())
    }

    fn visit_handler(
        &mut self,
        handler: &HandlerDef,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        self.visit_handler_block(&handler.block, ctx)
    }

    fn visit_handler_block(
        &mut self,
        block: &HandlerBlock,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        for stmt in &block.statements {
            self.visit_statement(stmt, ctx)?;
        }
        Ok(())
    }

    fn visit_statement(&mut self, stmt: &Statement, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        match stmt {
            Statement::Expression(expr) => self.visit_expression(expr, ctx),
            Statement::Assignment { target, value } => {
                for expr in target {
                    self.visit_expression(expr, ctx)?;
                }
                self.visit_expression(value, ctx)
            }
            Statement::Return(expr) => self.visit_expression(expr, ctx),
            Statement::Block(statements) => {
                for stmt in statements {
                    self.visit_statement(stmt, ctx)?;
                }
                Ok(())
            }
            Statement::WithError {
                statement,
                error_handler_block,
            } => {
                self.visit_statement(statement, ctx)?;
                for stmt in &error_handler_block.error_handler_statements {
                    self.visit_statement(stmt, ctx)?;
                }
                Ok(())
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.visit_expression(condition, ctx)?;
                for stmt in then_block {
                    self.visit_statement(stmt, ctx)?;
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.visit_statement(stmt, ctx)?;
                    }
                }
                Ok(())
            }
            Statement::Emit { parameters, .. } => {
                for param in parameters {
                    match param {
                        Argument::Named { value, .. } | Argument::Positional(value) => {
                            self.visit_expression(value, ctx)?;
                        }
                    }
                }
                Ok(())
            }
        }
    }

    fn visit_expression(
        &mut self,
        _expr: &Expression,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }
}
