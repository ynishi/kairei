use super::{TypeCheckError, TypeCheckResult, TypeContext};
use crate::ast::*;

/// Visitor trait for type checking AST nodes
pub trait TypeVisitor {
    /// Visit a micro agent definition
    fn visit_micro_agent(
        &self,
        agent: &mut MicroAgentDef,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()>;

    /// Visit a state definition
    fn visit_state(&self, state: &mut StateDef, ctx: &mut TypeContext) -> TypeCheckResult<()>;

    /// Visit a handler definition
    fn visit_handler(&self, handler: &HandlerDef, ctx: &mut TypeContext) -> TypeCheckResult<()>;

    /// Visit an expression
    fn visit_expression(&self, expr: &Expression, ctx: &mut TypeContext) -> TypeCheckResult<()>;
}

/// Default implementation of TypeVisitor
pub struct DefaultTypeVisitor;

impl TypeVisitor for DefaultTypeVisitor {
    fn visit_micro_agent(
        &self,
        agent: &mut MicroAgentDef,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Visit state definition if present
        if let Some(state) = &mut agent.state {
            self.visit_state(state, ctx)?;
        }

        // Visit answer handlers if present
        if let Some(answer) = &agent.answer {
            for handler in &answer.handlers {
                self.visit_request_handler(handler, ctx)?;
            }
        }

        // Visit observe handlers if present
        if let Some(observe) = &agent.observe {
            for handler in &observe.handlers {
                self.visit_event_handler(handler, ctx)?;
            }
        }

        // Visit react handlers if present
        if let Some(react) = &agent.react {
            for handler in &react.handlers {
                self.visit_event_handler(handler, ctx)?;
            }
        }

        Ok(())
    }

    fn visit_state(&self, state: &mut StateDef, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        for (name, var) in &mut state.variables {
            // Infer type if not explicitly specified
            if var.type_info == TypeInfo::Simple("".to_string()) {
                if let Some(init) = &var.initial_value {
                    let inferred_type = self.infer_type(init, ctx)?;
                    ctx.scope.insert_type(name.clone(), inferred_type.clone());
                    // Update the variable's type info with inferred type
                    var.type_info = inferred_type;
                } else {
                    return Err(TypeCheckError::TypeInferenceError {
                        message: format!(
                            "Cannot infer type for state variable '{}' without initial value",
                            name
                        ),
                    });
                }
            } else {
                // Validate explicit type info
                self.validate_type_info(&var.type_info, ctx)?;
            }

            // Validate initial value if present
            if let Some(init) = &var.initial_value {
                self.visit_expression(init, ctx)?;
                // Check that initial value matches declared type
                self.check_type_compatibility(&var.type_info, init, ctx)?;
            }

            // Add type to scope
            ctx.scope.insert_type(name.clone(), var.type_info.clone());
        }
        Ok(())
    }

    fn visit_handler(&self, handler: &HandlerDef, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        // Validate parameter types
        for param in &handler.parameters {
            self.validate_type_info(&param.type_info, ctx)?;
        }

        // Visit handler block
        self.visit_handler_block(&handler.block, ctx)?;

        Ok(())
    }

    fn visit_expression(&self, expr: &Expression, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        match expr {
            Expression::Literal(_) => Ok(()), // Literals are always well-typed
            Expression::Variable(name) => {
                // Check variable exists in scope
                if !ctx.scope.contains_type(name) {
                    return Err(TypeCheckError::UndefinedType(name.clone()));
                }
                Ok(())
            }
            Expression::StateAccess(path) => {
                // Check state variable exists and is accessible
                if !ctx.scope.contains_type(&path.0.join(".")) {
                    return Err(TypeCheckError::InvalidStateVariable {
                        message: path.0.join("."),
                    });
                }
                Ok(())
            }
            Expression::Think { args, with_block } => {
                // Check think block arguments
                for arg in args {
                    match arg {
                        Argument::Named { value, .. } | Argument::Positional(value) => {
                            self.visit_expression(value, ctx)?;
                        }
                    }
                }
                // Validate think attributes if present
                if let Some(attrs) = with_block {
                    self.validate_think_attributes(attrs, ctx)?;
                }
                Ok(())
            }
            Expression::Request { parameters, .. } => {
                // Check request parameters
                for param in parameters {
                    match param {
                        Argument::Named { value, .. } | Argument::Positional(value) => {
                            self.visit_expression(value, ctx)?;
                        }
                    }
                }
                Ok(())
            }
            Expression::BinaryOp { left, right, .. } => {
                self.visit_expression(left, ctx)?;
                self.visit_expression(right, ctx)
            }
            Expression::Ok(expr) | Expression::Err(expr) => self.visit_expression(expr, ctx),
            Expression::Await(exprs) => {
                for expr in exprs {
                    self.visit_expression(expr, ctx)?;
                }
                Ok(())
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.visit_expression(arg, ctx)?;
                }
                Ok(())
            }
        }
    }
}

impl DefaultTypeVisitor {
    fn visit_request_handler(
        &self,
        handler: &RequestHandler,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Validate parameter types
        for param in &handler.parameters {
            self.validate_type_info(&param.type_info, ctx)?;
        }

        // Visit handler block
        self.visit_handler_block(&handler.block, ctx)
    }

    fn visit_event_handler(
        &self,
        handler: &EventHandler,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Validate parameter types
        for param in &handler.parameters {
            self.validate_type_info(&param.type_info, ctx)?;
        }

        // Visit handler block
        self.visit_handler_block(&handler.block, ctx)
    }

    fn visit_handler_block(
        &self,
        block: &HandlerBlock,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Create new scope for block
        ctx.scope.enter_scope();

        // Visit all statements in block
        for stmt in &block.statements {
            self.visit_statement(stmt, ctx)?;
        }

        // Exit block scope
        ctx.scope.exit_scope();
        Ok(())
    }

    fn visit_statement(&self, stmt: &Statement, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        match stmt {
            Statement::Expression(expr) => self.visit_expression(expr, ctx),
            Statement::Assignment { target, value } => {
                // Check target expressions
                for expr in target {
                    self.visit_expression(expr, ctx)?;
                }
                // Check value expression
                self.visit_expression(value, ctx)
            }
            Statement::Return(expr) => self.visit_expression(expr, ctx),
            Statement::Emit { parameters, .. } => {
                // Check emit parameters
                for param in parameters {
                    match param {
                        Argument::Named { value, .. } | Argument::Positional(value) => {
                            self.visit_expression(value, ctx)?;
                        }
                    }
                }
                Ok(())
            }
            Statement::Block(statements) => {
                ctx.scope.enter_scope();
                for stmt in statements {
                    self.visit_statement(stmt, ctx)?;
                }
                ctx.scope.exit_scope();
                Ok(())
            }
            Statement::WithError {
                statement,
                error_handler_block,
            } => {
                self.visit_statement(statement, ctx)?;
                ctx.scope.enter_scope();
                for stmt in &error_handler_block.error_handler_statements {
                    self.visit_statement(stmt, ctx)?;
                }
                ctx.scope.exit_scope();
                Ok(())
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.visit_expression(condition, ctx)?;

                ctx.scope.enter_scope();
                for stmt in then_block {
                    self.visit_statement(stmt, ctx)?;
                }
                ctx.scope.exit_scope();

                if let Some(else_stmts) = else_block {
                    ctx.scope.enter_scope();
                    for stmt in else_stmts {
                        self.visit_statement(stmt, ctx)?;
                    }
                    ctx.scope.exit_scope();
                }
                Ok(())
            }
        }
    }

    fn validate_type_info(
        &self,
        type_info: &TypeInfo,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        match type_info {
            TypeInfo::Simple(name) => {
                if !ctx.scope.contains_type(name) {
                    return Err(TypeCheckError::UndefinedType(name.clone()));
                }
                Ok(())
            }
            TypeInfo::Result { ok_type, err_type } => {
                self.validate_type_info(ok_type, ctx)?;
                self.validate_type_info(err_type, ctx)
            }
            TypeInfo::Option(inner) | TypeInfo::Array(inner) => self.validate_type_info(inner, ctx),
            TypeInfo::Map(key_type, value_type) => {
                self.validate_type_info(key_type, ctx)?;
                self.validate_type_info(value_type, ctx)
            }
            TypeInfo::Custom { name, fields } => {
                if !ctx.scope.contains_type(name) {
                    return Err(TypeCheckError::UndefinedType(name.clone()));
                }
                for field_info in fields.values() {
                    if let Some(field_type) = &field_info.type_info {
                        self.validate_type_info(field_type, ctx)?;
                    }
                    if let Some(default_value) = &field_info.default_value {
                        self.visit_expression(default_value, ctx)?;
                    }
                }
                Ok(())
            }
        }
    }

    #[allow(unused_variables)]
    fn validate_think_attributes(
        &self,
        attrs: &ThinkAttributes,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Validate plugin configurations
        for (plugin_name, config) in &attrs.plugins {
            if !ctx.plugins.contains_key(plugin_name) {
                return Err(TypeCheckError::InvalidPluginConfig {
                    message: format!("Unknown plugin: {}", plugin_name),
                });
            }
            // Additional plugin config validation could be added here
        }
        Ok(())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn infer_type(&self, expr: &Expression, _ctx: &mut TypeContext) -> TypeCheckResult<TypeInfo> {
        match expr {
            Expression::Literal(lit) => Ok(match lit {
                Literal::Integer(_) => TypeInfo::Simple("Int".to_string()),
                Literal::Float(_) => TypeInfo::Simple("Float".to_string()),
                Literal::String(_) => TypeInfo::Simple("String".to_string()),
                Literal::Boolean(_) => TypeInfo::Simple("Boolean".to_string()),
                Literal::Duration(_) => TypeInfo::Simple("Duration".to_string()),
                Literal::List(items) => {
                    if items.is_empty() {
                        return Err(TypeCheckError::TypeInferenceError {
                            message: "Cannot infer type of empty list".to_string(),
                        });
                    }
                    let item_type =
                        self.infer_type(&Expression::Literal(items[0].clone()), _ctx)?;
                    TypeInfo::Array(Box::new(item_type))
                }
                _ => {
                    return Err(TypeCheckError::TypeInferenceError {
                        message: "Cannot infer type from this literal".to_string(),
                    })
                }
            }),
            _ => Err(TypeCheckError::TypeInferenceError {
                message: "Type inference not supported for this expression".to_string(),
            }),
        }
    }

    #[allow(unused_variables)]
    fn check_type_compatibility(
        &self,
        expected: &TypeInfo,
        expr: &Expression,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // This is a placeholder for actual type compatibility checking
        // In a full implementation, this would infer the type of the expression
        // and check if it's compatible with the expected type
        Ok(())
    }
}
