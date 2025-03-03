use kairei_core::{
    type_checker::{TypeCheckResult, TypeContext, visitor::common::PluginVisitor},
    *,
};

pub mod await_expression_test;
pub mod plugin_integration;
pub mod request_expression_test;
pub mod request_handler_type_checking;
pub mod think_expression_test;

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
