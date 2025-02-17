use super::{error::Location, TypeCheckError, TypeCheckResult, TypeContext};
use crate::{ast::*, config::PluginConfig};

mod plugin_visitor;
pub use plugin_visitor::PluginTypeVisitor;

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
        // Create new scope for agent
        ctx.scope.enter_scope();

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

        // Exit agent scope
        ctx.scope.exit_scope();
        Ok(())
    }

    fn visit_state(&self, state: &mut StateDef, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        // Create new scope for state variables
        ctx.scope.enter_scope();

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
                // First validate the expression itself
                self.visit_expression(init, ctx)?;

                // Then check that initial value matches declared type
                self.check_type_compatibility(&var.type_info, init, ctx)?;

                // Ensure initial value is serializable
                if !self.is_serializable_type(&var.type_info) {
                    return Err(TypeCheckError::InvalidStateVariable {
                        message: format!("State variable '{}' must have a serializable type", name),
                    });
                }
            }

            // Add type to scope
            ctx.scope.insert_type(name.clone(), var.type_info.clone());
        }

        // Exit state scope
        ctx.scope.exit_scope();
        Ok(())
    }

    fn visit_handler(&self, handler: &HandlerDef, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        // Create new scope for handler
        ctx.scope.enter_scope();

        // Validate parameter types
        for param in &handler.parameters {
            // Validate type exists
            self.validate_type_info(&param.type_info, ctx)?;

            // Ensure concrete type (not generic)
            if let TypeInfo::Simple(name) = &param.type_info {
                if name.is_empty() {
                    return Err(TypeCheckError::InvalidHandlerSignature {
                        message: format!("Parameter '{}' must have a concrete type", param.name),
                    });
                }
            }

            // Ensure parameter type is serializable
            if !self.is_serializable_type(&param.type_info) {
                return Err(TypeCheckError::InvalidHandlerSignature {
                    message: format!(
                        "Handler parameter '{}' must have a serializable type",
                        param.name
                    ),
                });
            }

            // Add parameter type to scope
            ctx.scope
                .insert_type(param.name.clone(), param.type_info.clone());
        }

        // Visit handler block
        self.visit_handler_block(&handler.block, ctx)?;

        // Exit handler scope
        ctx.scope.exit_scope();
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
                            // First validate the expression itself
                            self.visit_expression(value, ctx)?;

                            // Then check if it can be stringified (has Display trait)
                            let value_type = self.infer_type(value, ctx)?;
                            if !self.has_display_trait(&value_type) {
                                return Err(TypeCheckError::InvalidThinkBlock {
                                    message: format!(
                                        "Type {} cannot be interpolated in think block - must implement Display",
                                        value_type
                                    ),
                                });
                            }
                        }
                    }
                }

                // Validate think attributes if present
                if let Some(attrs) = with_block {
                    // Validate temperature
                    if let Some(temp) = attrs.temperature {
                        if temp < 0.0 || temp > 1.0 {
                            return Err(TypeCheckError::InvalidThinkBlock {
                                message: format!("Temperature must be between 0 and 1, got {}", temp),
                            });
                        }
                    }

                    // Validate max_tokens
                    if let Some(tokens) = attrs.max_tokens {
                        if tokens < 1 {
                            return Err(TypeCheckError::InvalidThinkBlock {
                                message: format!("Max tokens must be positive, got {}", tokens),
                            });
                        }
                    }

                    // Validate plugins
                    for (plugin_name, config) in &attrs.plugins {
                        // First check if plugin exists
                        if !ctx.plugins.contains_key(plugin_name) {
                            return Err(TypeCheckError::InvalidPluginConfig {
                                message: format!("Unknown plugin: {}", plugin_name),
                            });
                        }

                        // Validate plugin configuration values
                        for (key, value) in config {
                            match value {
                                Literal::Integer(i) if *i < 0 => {
                                    return Err(TypeCheckError::InvalidPluginConfig {
                                        message: format!("Plugin {} config {} must be non-negative", plugin_name, key),
                                    });
                                }
                                Literal::Float(f) if *f < 0.0 || *f > 1.0 => {
                                    return Err(TypeCheckError::InvalidPluginConfig {
                                        message: format!("Plugin {} config {} must be between 0 and 1", plugin_name, key),
                                    });
                                }
                                Literal::String(s) if s.is_empty() => {
                                    return Err(TypeCheckError::InvalidPluginConfig {
                                        message: format!("Plugin {} config {} cannot be empty", plugin_name, key),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Ok(())
            }
            Expression::Request { parameters, .. } => {
                // Check request parameters
                for param in parameters {
                    match param {
                        Argument::Named { value, .. } | Argument::Positional(value) => {
                            self.visit_expression(value, ctx)?;
                            // Ensure parameter type is serializable
                            let param_type = self.infer_type(value, ctx)?;
                            if !self.is_serializable_type(&param_type) {
                                return Err(TypeCheckError::InvalidHandlerSignature {
                                    message: format!(
                                        "Request parameter type {} must be serializable",
                                        param_type
                                    ),
                                });
                            }
                        }
                    }
                }
                Ok(())
            }
            Expression::BinaryOp { left, right, op } => {
                self.visit_expression(left, ctx)?;
                self.visit_expression(right, ctx)?;

                // Check operand types are compatible
                let left_type = self.infer_type(left, ctx)?;
                let right_type = self.infer_type(right, ctx)?;

                match op {
                    BinaryOperator::Add
                    | BinaryOperator::Subtract
                    | BinaryOperator::Multiply
                    | BinaryOperator::Divide => {
                        if !matches!(
                            (&left_type, &right_type),
                            (
                                TypeInfo::Simple(l),
                                TypeInfo::Simple(r)
                            ) if (l == "Int" || l == "Float") && (r == "Int" || r == "Float")
                        ) {
                            return Err(TypeCheckError::TypeMismatch {
                                expected: TypeInfo::Simple("Numeric".to_string()),
                                found: left_type,
                                location: Location {
                                    line: 0,
                                    column: 0,
                                    file: String::new(),
                                },
                            });
                        }
                    }
                    _ => {} // Other operators can work with any comparable types
                }
                Ok(())
            }
            Expression::Ok(expr) | Expression::Err(expr) => self.visit_expression(expr, ctx),
            Expression::Await(exprs) => {
                for expr in exprs {
                    self.visit_expression(expr, ctx)?;
                    // Ensure awaited expression returns a Result
                    let expr_type = self.infer_type(expr, ctx)?;
                    if !matches!(expr_type, TypeInfo::Result { .. }) {
                        return Err(TypeCheckError::TypeInferenceError {
                            message: "Can only await Result types".to_string(),
                        });
                    }
                }
                Ok(())
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                // Visit all arguments
                for arg in arguments {
                    self.visit_expression(arg, ctx)?;
                }

                // For now, we just ensure the function exists in scope
                // In a full implementation, we would check against the function signature
                if !ctx.scope.contains_type(function) {
                    return Err(TypeCheckError::UndefinedType(function.clone()));
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

            // Ensure concrete type for request parameters
            match &param.type_info {
                TypeInfo::Simple(name) if name.is_empty() => {
                    return Err(TypeCheckError::InvalidHandlerSignature {
                        message: format!(
                            "Request parameter '{}' must have a concrete type",
                            param.name
                        ),
                    });
                }
                TypeInfo::Result { .. } | TypeInfo::Option(_) => {
                    return Err(TypeCheckError::InvalidHandlerSignature {
                        message: format!(
                            "Request parameter '{}' cannot be Result or Option",
                            param.name
                        ),
                    });
                }
                _ => {}
            }
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

            // Ensure concrete type for event parameters
            if let TypeInfo::Simple(name) = &param.type_info {
                if name.is_empty() {
                    return Err(TypeCheckError::InvalidHandlerSignature {
                        message: format!(
                            "Event parameter '{}' must have a concrete type",
                            param.name
                        ),
                    });
                }
            }
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



    #[allow(clippy::only_used_in_recursion)]
    fn infer_type(&self, expr: &Expression, ctx: &mut TypeContext) -> TypeCheckResult<TypeInfo> {
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
                    let item_type = self.infer_type(&Expression::Literal(items[0].clone()), ctx)?;
                    TypeInfo::Array(Box::new(item_type))
                }
                Literal::Map(entries) => {
                    if entries.is_empty() {
                        return Err(TypeCheckError::TypeInferenceError {
                            message: "Cannot infer type of empty map".to_string(),
                        });
                    }
                    let (first_key, first_value) = entries.iter().next().unwrap();
                    let key_type = self.infer_type(
                        &Expression::Literal(Literal::String(first_key.clone())),
                        ctx,
                    )?;
                    let value_type =
                        self.infer_type(&Expression::Literal(first_value.clone()), ctx)?;
                    TypeInfo::Map(Box::new(key_type), Box::new(value_type))
                }
                Literal::Null => TypeInfo::Simple("Null".to_string()),
                _ => {
                    return Err(TypeCheckError::TypeInferenceError {
                        message: "Cannot infer type from this literal".to_string(),
                    })
                }
            }),
            Expression::Variable(name) => {
                if let Some(type_info) = ctx.scope.get_type(name) {
                    Ok(type_info.clone())
                } else {
                    Err(TypeCheckError::UndefinedType(name.clone()))
                }
            }
            Expression::StateAccess(path) => {
                if let Some(type_info) = ctx.scope.get_type(&path.0.join(".")) {
                    Ok(type_info.clone())
                } else {
                    Err(TypeCheckError::InvalidStateVariable {
                        message: path.0.join("."),
                    })
                }
            }
            Expression::BinaryOp { left, right, op } => {
                let left_type = self.infer_type(left, ctx)?;
                let right_type = self.infer_type(right, ctx)?;
                match op {
                    BinaryOperator::Add
                    | BinaryOperator::Subtract
                    | BinaryOperator::Multiply
                    | BinaryOperator::Divide => match (&left_type, &right_type) {
                        (TypeInfo::Simple(l), TypeInfo::Simple(r))
                            if (l == "Int" || l == "Float") && (r == "Int" || r == "Float") =>
                        {
                            Ok(if l == "Float" || r == "Float" {
                                TypeInfo::Simple("Float".to_string())
                            } else {
                                TypeInfo::Simple("Int".to_string())
                            })
                        }
                        _ => Err(TypeCheckError::TypeInferenceError {
                            message: "Invalid operand types for arithmetic operation".to_string(),
                        }),
                    },
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::LessThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::LessThanEqual
                    | BinaryOperator::GreaterThanEqual
                    | BinaryOperator::And
                    | BinaryOperator::Or => Ok(TypeInfo::Simple("Boolean".to_string())),
                }
            }
            Expression::Ok(expr) => {
                let ok_type = self.infer_type(expr, ctx)?;
                Ok(TypeInfo::Result {
                    ok_type: Box::new(ok_type),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                })
            }
            Expression::Err(expr) => {
                let err_type = self.infer_type(expr, ctx)?;
                Ok(TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("Void".to_string())),
                    err_type: Box::new(err_type),
                })
            }
            Expression::FunctionCall { .. } => {
                // For now, assume function calls return a Result type
                // In a full implementation, this would look up the function signature
                Ok(TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("Any".to_string())),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                })
            }
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

    #[allow(clippy::only_used_in_recursion)]
    fn has_display_trait(&self, type_info: &TypeInfo) -> bool {
        match type_info {
            // Basic types that implement Display
            TypeInfo::Simple(name) => matches!(
                name.as_str(),
                "String" | "Int" | "Float" | "Boolean" | "Duration" | "Null"
            ),
            // Container types if their contents implement Display
            TypeInfo::Option(inner) | TypeInfo::Array(inner) => self.has_display_trait(inner),
            TypeInfo::Result { ok_type, .. } => self.has_display_trait(ok_type),
            TypeInfo::Map(key_type, value_type) => {
                self.has_display_trait(key_type) && self.has_display_trait(value_type)
            }
            // Custom types would need explicit Display implementation
            TypeInfo::Custom { .. } => false,
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn is_serializable_type(&self, type_info: &TypeInfo) -> bool {
        match type_info {
            // Basic serializable types
            TypeInfo::Simple(name) => matches!(
                name.as_str(),
                "String" | "Int" | "Float" | "Boolean" | "Duration" | "Null"
            ),
            // Container types if their contents are serializable
            TypeInfo::Option(inner) | TypeInfo::Array(inner) => self.is_serializable_type(inner),
            TypeInfo::Result { ok_type, err_type } => {
                self.is_serializable_type(ok_type) && self.is_serializable_type(err_type)
            }
            TypeInfo::Map(key_type, value_type) => {
                self.is_serializable_type(key_type) && self.is_serializable_type(value_type)
            }
            // Custom types need to be explicitly marked as serializable
            TypeInfo::Custom { .. } => false,
        }
    }
}
