use std::collections::HashMap;

use crate::{
    Argument,
    ast::{
        Expression, FieldInfo, HandlerBlock, HandlerDef, MicroAgentDef, RequestType, Root,
        StateDef, Statement, TypeInfo,
    },
    type_checker::{TypeCheckError, TypeCheckResult, TypeContext, visitor::common::TypeVisitor},
};

use super::{
    expression::{DefaultExpressionChecker, ExpressionTypeChecker},
    function::{DefaultFunctionChecker, FunctionTypeChecker},
};

/// Default implementation of type checking logic
pub struct DefaultVisitor {
    expression_checker: DefaultExpressionChecker,
    function_checker: DefaultFunctionChecker,
}

impl DefaultVisitor {
    pub fn new() -> Self {
        Self {
            expression_checker: DefaultExpressionChecker::new(),
            function_checker: DefaultFunctionChecker::new(),
        }
    }

    fn check_return_type(
        &self,
        expr: &Expression,
        expected_type: &TypeInfo,
        ctx: &TypeContext,
    ) -> TypeCheckResult<()> {
        match expected_type {
            TypeInfo::Result { ok_type, err_type } => {
                let expr_type = self.infer_type(expr, ctx)?;
                match expr {
                    Expression::Ok(inner_expr) => {
                        let inner_type = self.infer_type(inner_expr, ctx)?;
                        if let TypeInfo::Simple(type_name) = &**ok_type {
                            if type_name != "Any" && inner_type != **ok_type {
                                return Err(TypeCheckError::type_mismatch(
                                    (**ok_type).clone(),
                                    inner_type,
                                    Default::default(),
                                ));
                            }
                        } else if inner_type != **ok_type {
                            return Err(TypeCheckError::type_mismatch(
                                (**ok_type).clone(),
                                inner_type,
                                Default::default(),
                            ));
                        }
                    }
                    Expression::Err(inner_expr) => {
                        let inner_type = self.infer_type(inner_expr, ctx)?;
                        if let TypeInfo::Simple(type_name) = &inner_type {
                            if type_name == "String" {
                                return Ok(());
                            }
                        }
                        return Err(TypeCheckError::type_mismatch(
                            (**err_type).clone(),
                            inner_type,
                            Default::default(),
                        ));
                    }
                    _ => {
                        // For expressions that return Result type (like Think)
                        if let TypeInfo::Result {
                            ok_type: ref found_ok,
                            err_type: ref found_err,
                        } = expr_type
                        {
                            if **ok_type != **found_ok || **err_type != **found_err {
                                return Err(TypeCheckError::type_mismatch(
                                    expected_type.clone(),
                                    expr_type,
                                    Default::default(),
                                ));
                            }
                            return Ok(());
                        } else {
                            return Err(TypeCheckError::type_mismatch(
                                expected_type.clone(),
                                expr_type,
                                Default::default(),
                            ));
                        }
                    }
                }
            }
            _ => {
                let expr_type = self.infer_type(expr, ctx)?;
                if expr_type != *expected_type {
                    return Err(TypeCheckError::type_mismatch(
                        expected_type.clone(),
                        expr_type,
                        Default::default(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_condition(&self, condition: &Expression, ctx: &TypeContext) -> TypeCheckResult<()> {
        let cond_type = self.infer_type(condition, ctx)?;
        if !self.expression_checker.is_boolean(&cond_type) {
            return Err(TypeCheckError::type_mismatch(
                TypeInfo::Simple("Boolean".to_string()),
                cond_type,
                Default::default(),
            ));
        }
        Ok(())
    }

    pub fn infer_type(&self, expr: &Expression, ctx: &TypeContext) -> TypeCheckResult<TypeInfo> {
        match expr {
            Expression::Literal(lit) => self.expression_checker.infer_literal_type(lit, ctx),
            Expression::Variable(name) => {
                if let Some(type_info) = ctx.scope.get_type(name) {
                    Ok(type_info.clone())
                } else {
                    Err(TypeCheckError::undefined_variable(
                        name.clone(),
                        Default::default(),
                    ))
                }
            }
            Expression::BinaryOp { op, left, right } => {
                let left_type = self.infer_type(left, ctx)?;
                let right_type = self.infer_type(right, ctx)?;
                self.expression_checker
                    .infer_binary_op_type(&left_type, &right_type, op)
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => self
                .function_checker
                .check_function_call(function, arguments, ctx),
            Expression::Think { args, with_block } => {
                // Check argument types
                for arg in args {
                    match arg {
                        Argument::Named { value, .. } | Argument::Positional(value) => {
                            self.infer_type(value, ctx)?;
                        }
                    }
                }

                // Check if there's a custom provider specified in the with_block
                if let Some(with_attrs) = with_block {
                    if let Some(_provider) = &with_attrs.provider {
                        // If a custom provider is specified, check if it's registered
                        // For now, we still return Result<String, Error> as the default
                        // This can be extended in the future to support provider-specific return types
                    }

                    // Check plugin configurations if present
                    for config in with_attrs.plugins.values() {
                        // Validate plugin configuration values
                        for value in config.values() {
                            // We don't need to do anything with the result, just ensure it's valid
                            self.expression_checker.infer_literal_type(value, ctx)?;
                        }
                    }
                }

                // Think expressions return Result<String, Error>
                // In Normal mode, we infer this type for all Think expressions
                Ok(TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                })
            }
            Expression::Request {
                parameters,
                agent,
                request_type,
                options,
            } => {
                // Check parameter types
                for param in parameters {
                    match param {
                        Argument::Named { value, .. } | Argument::Positional(value) => {
                            self.infer_type(value, ctx)?;
                        }
                    }
                }

                // Validate agent name (in a real implementation, we would check if the agent exists)
                // For now, we just ensure it's not empty
                if agent.is_empty() {
                    return Err(TypeCheckError::type_inference_error(
                        "Agent name cannot be empty".to_string(),
                        Default::default(),
                    ));
                }

                // Validate request type (in a real implementation, we would check if the request type is valid)
                // For now, we just ensure it's not empty for custom request types
                match request_type {
                    RequestType::Custom(name) if name.is_empty() => {
                        return Err(TypeCheckError::type_inference_error(
                            "Request type name cannot be empty".to_string(),
                            Default::default(),
                        ));
                    }
                    _ => {}
                }

                // Check request options if present
                if let Some(req_options) = options {
                    // Validate timeout if specified
                    if let Some(_timeout) = &req_options.timeout {
                        // Timeout is a Duration, no need for additional validation
                    }

                    // Validate retry count if specified
                    if let Some(_retry) = &req_options.retry {
                        // Retry is a u32, no need for additional validation
                    }
                }

                // Request expressions return Result<Any, Error> in Normal mode
                // This allows for flexibility in return types from different agent handlers
                Ok(TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("Any".to_string())),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                })
            }
            Expression::Ok(expr) => {
                let ok_type = self.infer_type(expr, ctx)?;

                // Enhanced error handling for Ok expressions
                // Provide more detailed type information in the Result
                // Handle nested Ok/Err expressions by preserving the inner type structure
                let result = TypeInfo::Result {
                    ok_type: Box::new(ok_type),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                };

                // Add detailed error metadata for better error messages
                match expr.as_ref() {
                    // If the inner expression is also an Ok or Err, we have a nested Result
                    Expression::Ok(_) | Expression::Err(_) => {
                        // The type system already handles this correctly by preserving the inner type
                        // We just need to ensure the error messages are clear
                    }
                    _ => {
                        // For non-nested expressions, the standard behavior is fine
                    }
                }

                Ok(result)
            }
            Expression::Err(expr) => {
                let err_type = self.infer_type(expr, ctx)?;

                // Enhanced error handling for Err expressions
                // Provide more detailed type information in the Result
                // Handle nested Ok/Err expressions by preserving the inner type structure
                let result = TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("Any".to_string())),
                    err_type: Box::new(err_type),
                };

                // Add detailed error metadata for better error messages
                match expr.as_ref() {
                    // If the inner expression is also an Ok or Err, we have a nested Result
                    Expression::Ok(_) | Expression::Err(_) => {
                        // The type system already handles this correctly by preserving the inner type
                        // We just need to ensure the error messages are clear
                    }
                    _ => {
                        // For non-nested expressions, the standard behavior is fine
                    }
                }

                Ok(result)
            }
            Expression::StateAccess(path) => {
                let full_path = path.0.join(".");

                // Check if the path is empty
                if path.0.is_empty() {
                    return Err(TypeCheckError::type_inference_error(
                        "Empty state access path".to_string(),
                        Default::default(),
                    ));
                }

                // First check if the root variable exists
                if let Some(type_info) = ctx.scope.get_type(&path.0[0]) {
                    // If there's only one component in the path, return the type directly
                    if path.0.len() == 1 {
                        return Ok(type_info.clone());
                    }

                    // Handle nested field access recursively
                    let mut current_type = type_info.clone();
                    let mut current_path = path.0[0].clone();

                    // Start from the second component (index 1)
                    for i in 1..path.0.len() {
                        let field_name = &path.0[i];
                        current_path = format!("{}.{}", current_path, field_name);

                        match &current_type {
                            TypeInfo::Custom { fields, .. } => {
                                // Check if the field exists in the custom type
                                if let Some(field_info) = fields.get(field_name) {
                                    if let Some(field_type) = &field_info.type_info {
                                        // Update current_type for the next iteration
                                        current_type = field_type.clone();
                                    } else if let Some(default_value) = &field_info.default_value {
                                        // Infer type from default value if available
                                        current_type = self.infer_type(default_value, ctx)?;
                                    } else {
                                        return Err(TypeCheckError::type_inference_error(
                                            format!(
                                                "Cannot infer type for field {} in path {}",
                                                field_name, current_path
                                            ),
                                            Default::default(),
                                        ));
                                    }
                                } else {
                                    return Err(TypeCheckError::undefined_variable(
                                        format!(
                                            "Field {} not found in type at path {}",
                                            field_name, current_path
                                        ),
                                        Default::default(),
                                    ));
                                }
                            }
                            _ => {
                                return Err(TypeCheckError::type_inference_error(
                                    format!(
                                        "Cannot access field {} on non-custom type at path {}",
                                        field_name, current_path
                                    ),
                                    Default::default(),
                                ));
                            }
                        }
                    }

                    // Return the final type after traversing the entire path
                    Ok(current_type)
                } else if let Some(type_info) = ctx.scope.get_type(&full_path) {
                    // Try to get the full path directly from the scope
                    // This handles cases where the full path is registered as a variable
                    Ok(type_info.clone())
                } else {
                    // Root variable doesn't exist
                    Err(TypeCheckError::undefined_variable(
                        path.0[0].clone(),
                        Default::default(),
                    ))
                }
            }
            Expression::Await(exprs) => {
                // For a single expression, return the ok_type of the Result
                if exprs.len() == 1 {
                    let expr_type = self.infer_type(&exprs[0], ctx)?;
                    if let TypeInfo::Result { ok_type, .. } = expr_type {
                        Ok(*ok_type)
                    } else {
                        Err(TypeCheckError::type_inference_error(
                            "Can only await Result types".to_string(),
                            Default::default(),
                        ))
                    }
                } else {
                    // For multiple expressions, return an array of ok_types
                    let mut types = Vec::new();
                    for expr in exprs {
                        let expr_type = self.infer_type(expr, ctx)?;
                        if let TypeInfo::Result { ok_type, .. } = expr_type {
                            types.push(*ok_type);
                        } else {
                            return Err(TypeCheckError::type_inference_error(
                                "Can only await Result types".to_string(),
                                Default::default(),
                            ));
                        }
                    }
                    Ok(TypeInfo::Array(Box::new(TypeInfo::Simple(
                        "Any".to_string(),
                    ))))
                }
            }
            Expression::WillAction { parameters, .. } => {
                // Check parameter types
                for param in parameters {
                    self.infer_type(param, ctx)?;
                }
                // WillAction returns a map with action details
                // WillAction returns a structured type with action details
                Ok(TypeInfo::Custom {
                    name: "WillAction".to_string(),
                    fields: {
                        let mut fields = HashMap::new();
                        fields.insert(
                            "action".to_string(),
                            FieldInfo {
                                type_info: Some(TypeInfo::Simple("String".to_string())),
                                default_value: None,
                            },
                        );
                        fields.insert(
                            "parameters".to_string(),
                            FieldInfo {
                                type_info: Some(TypeInfo::Array(Box::new(TypeInfo::Simple(
                                    "Any".to_string(),
                                )))),
                                default_value: None,
                            },
                        );
                        fields.insert(
                            "target".to_string(),
                            FieldInfo {
                                type_info: Some(TypeInfo::Option(Box::new(TypeInfo::Simple(
                                    "String".to_string(),
                                )))),
                                default_value: None,
                            },
                        );
                        fields
                    },
                })
            }
        }
    }

    fn check_custom_type_fields(
        &self,
        fields: &HashMap<String, FieldInfo>,
        ctx: &TypeContext,
    ) -> TypeCheckResult<()> {
        for field_info in fields {
            // Check field type if specified
            if let Some(field_type) = &field_info.1.type_info {
                match field_type {
                    TypeInfo::Simple(type_name) => {
                        if !ctx.scope.contains_type(type_name) {
                            return Err(TypeCheckError::undefined_type(
                                type_name.clone(),
                                Default::default(),
                            ));
                        }
                    }
                    TypeInfo::Custom { name, fields } => {
                        // Recursively check nested custom types
                        if !ctx.scope.contains_type(name) {
                            return Err(TypeCheckError::undefined_type(
                                name.clone(),
                                Default::default(),
                            ));
                        }
                        self.check_custom_type_fields(fields, ctx)?;
                    }
                    _ => {} // Other type variants are valid
                }
            }

            // Check default value type if provided
            if let Some(default_value) = &field_info.1.default_value {
                let default_type = self.infer_type(default_value, ctx)?;
                if let Some(field_type) = &field_info.1.type_info {
                    if default_type != *field_type {
                        return Err(TypeCheckError::type_mismatch(
                            field_type.clone(),
                            default_type,
                            Default::default(),
                        ));
                    }
                }
            }
        }
        Ok(())
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
                // 既存の型定義がない場合のみデフォルト値を設定
                if ctx.scope.get_type("return_type").is_none() {
                    ctx.scope.insert_type(
                        "return_type".to_string(),
                        TypeInfo::Result {
                            ok_type: Box::new(TypeInfo::Simple("Any".to_string())),
                            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                        },
                    );
                }

                // Check parameter types
                for param in &handler.parameters {
                    if let Some(existing_type) = ctx.scope.get_type(&param.name) {
                        if existing_type != param.type_info {
                            return Err(TypeCheckError::type_mismatch(
                                existing_type.clone(),
                                param.type_info.clone(),
                                Default::default(),
                            ));
                        }
                    }

                    // Add parameter to scope for use in handler block
                    ctx.scope
                        .insert_type(param.name.clone(), param.type_info.clone());
                }
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
        // Create an isolated scope for the micro agent
        ctx.enter_isolated_scope();

        // Visit state definition if present
        if let Some(state) = &mut agent.state {
            self.visit_state(state, ctx)?;
        }

        // Visit lifecycle handlers if present
        if let Some(lifecycle) = &agent.lifecycle {
            if let Some(init) = &lifecycle.on_init {
                // Create an isolated scope for the init handler
                ctx.enter_isolated_scope();
                let result = self.visit_handler_block(init, ctx);
                ctx.exit_isolated_scope();
                result?;
            }
            if let Some(destroy) = &lifecycle.on_destroy {
                // Create an isolated scope for the destroy handler
                ctx.enter_isolated_scope();
                let result = self.visit_handler_block(destroy, ctx);
                ctx.exit_isolated_scope();
                result?;
            }
        }

        // Visit answer handlers if present
        if let Some(answer) = &agent.answer {
            for handler in &answer.handlers {
                // Create an isolated scope for each answer handler
                ctx.enter_isolated_scope();

                // Register handler return type in scope
                ctx.scope.insert_type(
                    "handler_return_type".to_string(),
                    handler.return_type.clone(),
                );

                // Check parameter types
                for param in &handler.parameters {
                    if let Some(existing_type) = ctx.scope.get_type(&param.name) {
                        if existing_type != param.type_info {
                            ctx.exit_isolated_scope();
                            return Err(TypeCheckError::type_mismatch(
                                existing_type.clone(),
                                param.type_info.clone(),
                                Default::default(),
                            ));
                        }
                    }

                    // Add parameter to scope for use in handler block
                    ctx.scope
                        .insert_type(param.name.clone(), param.type_info.clone());
                }

                let result = self.visit_handler_block(&handler.block, ctx);
                ctx.exit_isolated_scope();
                result?;
            }
        }

        // Visit observe handlers if present
        if let Some(observe) = &agent.observe {
            for handler in &observe.handlers {
                // Create an isolated scope for each observe handler
                ctx.enter_isolated_scope();

                // 既存の型定義がない場合のみデフォルト値を設定
                if ctx.scope.get_type("return_type").is_none() {
                    ctx.scope.insert_type(
                        "return_type".to_string(),
                        TypeInfo::Result {
                            ok_type: Box::new(TypeInfo::Simple("Any".to_string())),
                            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                        },
                    );
                }

                // Check parameter types
                for param in &handler.parameters {
                    if let Some(existing_type) = ctx.scope.get_type(&param.name) {
                        if existing_type != param.type_info {
                            return Err(TypeCheckError::type_mismatch(
                                existing_type.clone(),
                                param.type_info.clone(),
                                Default::default(),
                            ));
                        }
                    }

                    // Add parameter to scope for use in handler block
                    ctx.scope
                        .insert_type(param.name.clone(), param.type_info.clone());
                }
                let result = self.visit_handler_block(&handler.block, ctx);
                ctx.exit_isolated_scope();
                result?;
            }
        }

        // Visit react handlers if present
        if let Some(react) = &agent.react {
            for handler in &react.handlers {
                // Create an isolated scope for each react handler
                ctx.enter_isolated_scope();

                // 既存の型定義がない場合のみデフォルト値を設定
                if ctx.scope.get_type("return_type").is_none() {
                    ctx.scope.insert_type(
                        "return_type".to_string(),
                        TypeInfo::Result {
                            ok_type: Box::new(TypeInfo::Simple("Any".to_string())),
                            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                        },
                    );
                }

                // Check parameter types
                for param in &handler.parameters {
                    if let Some(existing_type) = ctx.scope.get_type(&param.name) {
                        if existing_type != param.type_info {
                            ctx.exit_isolated_scope();
                            return Err(TypeCheckError::type_mismatch(
                                existing_type.clone(),
                                param.type_info.clone(),
                                Default::default(),
                            ));
                        }
                    }

                    // Add parameter to scope for use in handler block
                    ctx.scope
                        .insert_type(param.name.clone(), param.type_info.clone());
                }
                let result = self.visit_handler_block(&handler.block, ctx);
                ctx.exit_isolated_scope();
                result?;
            }
        }

        // Exit the isolated scope for the micro agent
        ctx.exit_isolated_scope();

        Ok(())
    }

    fn visit_state(&mut self, state: &mut StateDef, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        // Check each state variable's type
        for var_def in &state.variables {
            let var_def = var_def.1;
            match &var_def.type_info {
                TypeInfo::Simple(type_name) => {
                    if !ctx.scope.contains_type(type_name) {
                        return Err(TypeCheckError::undefined_type(
                            type_name.clone(),
                            Default::default(),
                        ));
                    }
                }
                TypeInfo::Custom { name, fields } => {
                    if !ctx.scope.contains_type(name) {
                        return Err(TypeCheckError::undefined_type(
                            name.clone(),
                            Default::default(),
                        ));
                    }
                    self.check_custom_type_fields(fields, ctx)?;
                }
                _ => {}
            }

            // If there's an initial value, check its type
            if let Some(init_value) = &var_def.initial_value {
                let init_type = self.infer_type(init_value, ctx)?;
                if init_type != var_def.type_info {
                    return Err(TypeCheckError::type_mismatch(
                        var_def.type_info.clone(),
                        init_type,
                        Default::default(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn visit_handler(
        &mut self,
        handler: &HandlerDef,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Create an isolated scope for the handler
        ctx.enter_isolated_scope();

        // Register parameters in scope before checking handler block
        for param in &handler.parameters {
            ctx.scope
                .insert_type(param.name.clone(), param.type_info.clone());
        }

        let result = self.visit_handler_block(&handler.block, ctx);

        // Exit the isolated scope to clean up
        ctx.exit_isolated_scope();

        result
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
                // Get value type first
                let value_type = self.infer_type(value, ctx)?;

                // Handle target based on expression type
                match &target[0] {
                    Expression::Variable(name) => {
                        // Try to get the target type
                        let target_type_result = self.infer_type(&target[0], ctx);

                        match target_type_result {
                            Ok(target_type) => {
                                // Variable already has a type, check compatibility
                                if target_type != value_type {
                                    return Err(TypeCheckError::type_mismatch(
                                        target_type,
                                        value_type,
                                        Default::default(),
                                    ));
                                }
                            }
                            Err(TypeCheckError::UndefinedVariable { .. }) => {
                                // Variable doesn't have a type yet
                                // In Normal mode, infer the type from the value
                                ctx.scope.insert_type(name.clone(), value_type);
                            }
                            Err(err) => {
                                // Propagate other errors
                                return Err(err);
                            }
                        }
                    }
                    _ => {
                        // For other expressions (e.g., StateAccess), get target type and check compatibility
                        let target_type = self.infer_type(&target[0], ctx)?;
                        if target_type != value_type {
                            return Err(TypeCheckError::type_mismatch(
                                target_type,
                                value_type,
                                Default::default(),
                            ));
                        }
                    }
                }
                Ok(())
            }
            Statement::Return(expr) => {
                // Get current function's return type from context
                // For RequestHandler, use its own return_type
                // For other handlers, use return_type from scope
                let expected_type =
                    if let Some(return_type) = ctx.scope.get_type("handler_return_type") {
                        return_type.clone()
                    } else if let Some(return_type) = ctx.scope.get_type("return_type") {
                        return_type.clone()
                    } else {
                        return Err(TypeCheckError::type_inference_error(
                            "No return type found for handler".to_string(),
                            Default::default(),
                        ));
                    };
                self.check_return_type(expr, &expected_type, ctx)?;
                Ok(())
            }
            Statement::Block(statements) => {
                // Create a checkpoint before entering the block
                let checkpoint = ctx.create_scope_checkpoint();

                // Enter a new scope for the block
                ctx.scope.enter_scope();

                // Visit all statements in the block
                for stmt in statements {
                    self.visit_statement(stmt, ctx)?;
                }

                // Restore the checkpoint after exiting the block
                ctx.restore_scope_checkpoint(checkpoint);

                Ok(())
            }
            Statement::WithError {
                statement,
                error_handler_block,
            } => {
                self.visit_statement(statement, ctx)?;

                // Create a checkpoint before entering the error handler block
                let checkpoint = ctx.create_scope_checkpoint();
                ctx.scope.enter_scope();

                for stmt in &error_handler_block.error_handler_statements {
                    self.visit_statement(stmt, ctx)?;
                }

                // Restore the checkpoint after exiting the error handler block
                ctx.restore_scope_checkpoint(checkpoint);

                Ok(())
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                // Check condition is boolean
                self.check_condition(condition, ctx)?;

                // Create a checkpoint before entering the then block
                let checkpoint = ctx.create_scope_checkpoint();

                // Enter a new scope for the then block
                ctx.scope.enter_scope();

                // Check then block
                for stmt in then_block {
                    self.visit_statement(stmt, ctx)?;
                }

                // Restore checkpoint before potentially entering else block
                ctx.restore_scope_checkpoint(checkpoint);

                // Handle else block if present
                if let Some(else_stmts) = else_block {
                    // Create a new scope for the else block
                    ctx.scope.enter_scope();

                    for stmt in else_stmts {
                        self.visit_statement(stmt, ctx)?;
                    }

                    // Exit the else block scope
                    ctx.scope.exit_scope();
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
        expr: &Expression,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Infer type to perform type checking
        self.infer_type(expr, ctx)?;
        Ok(())
    }
}
