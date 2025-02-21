use crate::{
    ast::{Expression, HandlerBlock, HandlerDef, MicroAgentDef, Root, StateDef, Statement},
    type_checker::{TypeCheckResult, TypeContext},
};

/// Common visitor trait for type checking AST nodes
pub trait TypeVisitor {
    /// Visit the root node of the AST
    fn visit_root(&mut self, root: &mut Root, ctx: &mut TypeContext) -> TypeCheckResult<()>;

    /// Visit a micro agent definition
    fn visit_micro_agent(
        &mut self,
        agent: &mut MicroAgentDef,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()>;

    /// Visit a state definition
    fn visit_state(&mut self, state: &mut StateDef, ctx: &mut TypeContext) -> TypeCheckResult<()>;

    /// Visit a handler definition
    fn visit_handler(&mut self, handler: &HandlerDef, ctx: &mut TypeContext)
        -> TypeCheckResult<()>;

    /// Visit a handler block
    fn visit_handler_block(
        &mut self,
        block: &HandlerBlock,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()>;

    /// Visit a statement
    fn visit_statement(&mut self, stmt: &Statement, ctx: &mut TypeContext) -> TypeCheckResult<()>;

    /// Visit an expression
    fn visit_expression(&mut self, expr: &Expression, ctx: &mut TypeContext)
        -> TypeCheckResult<()>;
}

/// Plugin visitor trait for type checking
pub trait PluginVisitor {
    /// Called before visiting the root node
    fn before_root(&mut self, _root: &mut Root, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting the root node
    fn after_root(&mut self, _root: &mut Root, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a micro agent
    fn before_micro_agent(
        &mut self,
        _agent: &mut MicroAgentDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a micro agent
    fn after_micro_agent(
        &mut self,
        _agent: &mut MicroAgentDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a state definition
    fn before_state(
        &mut self,
        _state: &mut StateDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a state definition
    fn after_state(
        &mut self,
        _state: &mut StateDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a handler
    fn before_handler(
        &mut self,
        _handler: &HandlerDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a handler
    fn after_handler(
        &mut self,
        _handler: &HandlerDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a handler block
    fn before_handler_block(
        &mut self,
        _block: &HandlerBlock,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a handler block
    fn after_handler_block(
        &mut self,
        _block: &HandlerBlock,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a statement
    fn before_statement(
        &mut self,
        _stmt: &Statement,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a statement
    fn after_statement(
        &mut self,
        _stmt: &Statement,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting an expression
    fn before_expression(
        &mut self,
        _expr: &Expression,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting an expression
    fn after_expression(
        &mut self,
        _expr: &Expression,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }
}
