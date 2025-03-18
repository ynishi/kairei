use crate::{
    ast::{
        Expression, HandlerBlock, HandlerDef, MicroAgentDef, Root, SistenceAgentDef, StateDef,
        Statement, TypeInfo,
    },
    type_checker::{
        TypeCheckError, TypeCheckResult, TypeContext,
        visitor::{common::PluginVisitor, common::TypeVisitor, default::DefaultVisitor},
    },
};

/// Main type checker implementation
pub struct TypeChecker {
    plugins: Vec<Box<dyn PluginVisitor>>,
    default_visitor: DefaultVisitor,
    context: TypeContext,
}

impl TypeChecker {
    /// Creates a new TypeChecker instance without plugins
    pub fn new() -> Self {
        let mut checker = Self {
            plugins: Vec::new(),
            default_visitor: DefaultVisitor::new(),
            context: TypeContext::new(),
        };

        // Register built-in types
        checker.register_builtin_types();

        checker
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn PluginVisitor>) {
        self.plugins.push(plugin);
    }

    /// Register all built-in types
    fn register_builtin_types(&mut self) {
        let builtin_types = ["Int", "Float", "String", "Boolean", "Duration"];
        for type_name in builtin_types.iter() {
            self.context.scope.insert_type(
                type_name.to_string(),
                TypeInfo::Simple(type_name.to_string()),
            );
        }
    }

    /// Insert a type into the current scope
    pub fn insert_type(&mut self, name: String, type_info: TypeInfo) {
        self.context.scope.insert_type(name, type_info);
    }

    /// Check if a type exists in the current scope
    pub fn contains_type(&self, name: &str) -> bool {
        self.context.scope.contains_type(name)
    }

    /// Get the current scope depth
    pub fn scope_depth(&self) -> usize {
        self.context.scope.depth()
    }

    /// Check types for the entire AST
    pub fn check_types(&mut self, root: &mut Root) -> TypeCheckResult<()> {
        let mut ctx = self.context.clone();
        self.visit_root(root, &mut ctx)
    }

    /// Collect any errors that occurred during type checking
    pub fn collect_errors(&mut self) -> Vec<TypeCheckError> {
        self.context.take_errors()
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeVisitor for TypeChecker {
    fn visit_root(&mut self, root: &mut Root, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        // Run plugins before root
        for plugin in &mut self.plugins {
            plugin.before_root(root, ctx)?;
        }

        // Run default visitor
        self.default_visitor.visit_root(root, ctx)?;

        // Run plugins after root
        for plugin in &mut self.plugins {
            plugin.after_root(root, ctx)?;
        }

        Ok(())
    }

    fn visit_micro_agent(
        &mut self,
        agent: &mut MicroAgentDef,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Run plugins before micro agent
        for plugin in &mut self.plugins {
            plugin.before_micro_agent(agent, ctx)?;
        }

        // Run default visitor
        self.default_visitor.visit_micro_agent(agent, ctx)?;

        // Run plugins after micro agent
        for plugin in &mut self.plugins {
            plugin.after_micro_agent(agent, ctx)?;
        }

        Ok(())
    }

    fn visit_state(&mut self, state: &mut StateDef, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        // Run plugins before state
        for plugin in &mut self.plugins {
            plugin.before_state(state, ctx)?;
        }

        // Run default visitor
        self.default_visitor.visit_state(state, ctx)?;

        // Run plugins after state
        for plugin in &mut self.plugins {
            plugin.after_state(state, ctx)?;
        }

        Ok(())
    }

    fn visit_handler(
        &mut self,
        handler: &HandlerDef,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Run plugins before handler
        for plugin in &mut self.plugins {
            plugin.before_handler(handler, ctx)?;
        }

        // Run default visitor
        self.default_visitor.visit_handler(handler, ctx)?;

        // Run plugins after handler
        for plugin in &mut self.plugins {
            plugin.after_handler(handler, ctx)?;
        }

        Ok(())
    }

    fn visit_handler_block(
        &mut self,
        block: &HandlerBlock,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Run plugins before handler block
        for plugin in &mut self.plugins {
            plugin.before_handler_block(block, ctx)?;
        }

        // Run default visitor
        self.default_visitor.visit_handler_block(block, ctx)?;

        // Run plugins after handler block
        for plugin in &mut self.plugins {
            plugin.after_handler_block(block, ctx)?;
        }

        Ok(())
    }

    fn visit_statement(&mut self, stmt: &Statement, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        // Run plugins before statement
        for plugin in &mut self.plugins {
            plugin.before_statement(stmt, ctx)?;
        }

        // Run default visitor
        self.default_visitor.visit_statement(stmt, ctx)?;

        // Run plugins after statement
        for plugin in &mut self.plugins {
            plugin.after_statement(stmt, ctx)?;
        }

        Ok(())
    }

    fn visit_expression(
        &mut self,
        expr: &Expression,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Run plugins before expression
        for plugin in &mut self.plugins {
            plugin.before_expression(expr, ctx)?;
        }

        // Run default visitor
        self.default_visitor.visit_expression(expr, ctx)?;

        // Run plugins after expression
        for plugin in &mut self.plugins {
            plugin.after_expression(expr, ctx)?;
        }

        Ok(())
    }

    fn visit_sistence_agent(
        &mut self,
        agent: &mut SistenceAgentDef,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Run plugins before sistence agent
        for plugin in &mut self.plugins {
            plugin.before_sistence_agent(agent, ctx)?;
        }

        // Run default visitor
        self.default_visitor.visit_sistence_agent(agent, ctx)?;

        // Run plugins after sistence agent
        for plugin in &mut self.plugins {
            plugin.after_sistence_agent(agent, ctx)?;
        }

        Ok(())
    }
}
