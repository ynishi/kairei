use crate::{
    ast::{Expression, HandlerBlock, HandlerDef, MicroAgentDef, Root, StateDef, Statement},
    type_checker::{TypeCheckResult, TypeContext},
};

/// Interface for type checker plugins
pub trait TypeCheckerPlugin {
    /// Called before visiting the root node
    fn before_root(&self, _root: &mut Root, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting the root node
    fn after_root(&self, _root: &mut Root, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a micro agent
    fn before_micro_agent(
        &self,
        _agent: &mut MicroAgentDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a micro agent
    fn after_micro_agent(
        &self,
        _agent: &mut MicroAgentDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a state definition
    fn before_state(&self, _state: &mut StateDef, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a state definition
    fn after_state(&self, _state: &mut StateDef, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a handler definition
    fn before_handler(&self, _handler: &HandlerDef, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a handler definition
    fn after_handler(&self, _handler: &HandlerDef, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a handler block
    fn before_handler_block(
        &self,
        _block: &HandlerBlock,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a handler block
    fn after_handler_block(
        &self,
        _block: &HandlerBlock,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting a statement
    fn before_statement(&self, _stmt: &Statement, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting a statement
    fn after_statement(&self, _stmt: &Statement, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called before visiting an expression
    fn before_expression(&self, _expr: &Expression, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    /// Called after visiting an expression
    fn after_expression(&self, _expr: &Expression, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }
}
