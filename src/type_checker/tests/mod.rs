use crate::*;

use super::{visitor::common::PluginVisitor, TypeCheckResult, TypeContext};

mod custom_type_tests;
mod error_tests;
mod expression_tests;
mod integration_tests;
mod scope_tests;

struct TestPlugin;
impl PluginVisitor for TestPlugin {
    fn before_root(&mut self, _root: &mut Root, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    fn after_root(&mut self, _root: &mut Root, _ctx: &mut TypeContext) -> TypeCheckResult<()> {
        Ok(())
    }

    fn before_micro_agent(
        &mut self,
        _agent: &mut MicroAgentDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn after_micro_agent(
        &mut self,
        _agent: &mut MicroAgentDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn before_state(
        &mut self,
        _state: &mut StateDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn after_state(
        &mut self,
        _state: &mut StateDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn before_handler(
        &mut self,
        _handler: &HandlerDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn after_handler(
        &mut self,
        _handler: &HandlerDef,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn before_handler_block(
        &mut self,
        _block: &HandlerBlock,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn after_handler_block(
        &mut self,
        _block: &HandlerBlock,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn before_statement(
        &mut self,
        _stmt: &Statement,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn after_statement(
        &mut self,
        _stmt: &Statement,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn before_expression(
        &mut self,
        _expr: &Expression,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }

    fn after_expression(
        &mut self,
        _expr: &Expression,
        _ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        Ok(())
    }
}
