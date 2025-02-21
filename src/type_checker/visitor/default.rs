use std::collections::HashMap;

use crate::{
    ast::{
        Expression, FieldInfo, HandlerBlock, HandlerDef, MicroAgentDef, Root, StateDef, Statement,
        TypeInfo,
    },
    type_checker::{visitor::common::TypeVisitor, TypeCheckError, TypeCheckResult, TypeContext},
    Argument,
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
            TypeInfo::Result { ok_type, err_type } => match expr {
                Expression::Ok(inner_expr) => {
                    let inner_type = self.infer_type(inner_expr, ctx)?;
                    // Any型は任意の型を受け入れる
                    if let TypeInfo::Simple(type_name) = &**ok_type {
                        if type_name != "Any" && inner_type != **ok_type {
                            return Err(TypeCheckError::TypeMismatch {
                                expected: (**ok_type).clone(),
                                found: inner_type,
                                location: Default::default(),
                            });
                        }
                    } else if inner_type != **ok_type {
                        return Err(TypeCheckError::TypeMismatch {
                            expected: (**ok_type).clone(),
                            found: inner_type,
                            location: Default::default(),
                        });
                    }
                }
                Expression::Err(inner_expr) => {
                    let inner_type = self.infer_type(inner_expr, ctx)?;
                    // エラーの場合、String型をError型として扱う
                    if let TypeInfo::Simple(type_name) = &inner_type {
                        if type_name == "String" {
                            return Ok(());
                        }
                    }
                    return Err(TypeCheckError::TypeMismatch {
                        expected: (**err_type).clone(),
                        found: inner_type,
                        location: Default::default(),
                    });
                }
                _ => {
                    let expr_type = self.infer_type(expr, ctx)?;
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_type.clone(),
                        found: expr_type,
                        location: Default::default(),
                    });
                }
            },
            _ => {
                let expr_type = self.infer_type(expr, ctx)?;
                if expr_type != *expected_type {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_type.clone(),
                        found: expr_type,
                        location: Default::default(),
                    });
                }
            }
        }
        Ok(())
    }

    fn check_condition(&self, condition: &Expression, ctx: &TypeContext) -> TypeCheckResult<()> {
        let cond_type = self.infer_type(condition, ctx)?;
        if !self.expression_checker.is_boolean(&cond_type) {
            return Err(TypeCheckError::TypeMismatch {
                expected: TypeInfo::Simple("Boolean".to_string()),
                found: cond_type,
                location: Default::default(),
            });
        }
        Ok(())
    }

    fn infer_type(&self, expr: &Expression, ctx: &TypeContext) -> TypeCheckResult<TypeInfo> {
        match expr {
            Expression::Literal(lit) => self.expression_checker.infer_literal_type(lit, ctx),
            Expression::Variable(name) => {
                if let Some(type_info) = ctx.scope.get_type(name) {
                    Ok(type_info.clone())
                } else {
                    Err(TypeCheckError::UndefinedVariable(name.clone()))
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
            Expression::Think { args, .. } => {
                // Check argument types
                for arg in args {
                    match arg {
                        Argument::Named { value, .. } | Argument::Positional(value) => {
                            self.infer_type(value, ctx)?;
                        }
                    }
                }
                // Think expressions return Result<String, Error>
                Ok(TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                })
            }
            Expression::Request { parameters, .. } => {
                // Check parameter types
                for param in parameters {
                    match param {
                        Argument::Named { value, .. } | Argument::Positional(value) => {
                            self.infer_type(value, ctx)?;
                        }
                    }
                }
                // Request expressions return Result<Any, Error>
                Ok(TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("Any".to_string())),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                })
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
                    ok_type: Box::new(TypeInfo::Simple("Any".to_string())),
                    err_type: Box::new(err_type),
                })
            }
            Expression::StateAccess(path) => {
                let full_path = path.0.join(".");
                if let Some(type_info) = ctx.scope.get_type(&path.0[0]) {
                    if let TypeInfo::Custom { fields, .. } = type_info.clone() {
                        if path.0.len() > 1 {
                            // Field access
                            let field_name = &path.0[1];
                            if let Some(field_info) = fields.get(field_name) {
                                if let Some(field_type) = &field_info.type_info {
                                    Ok(field_type.clone())
                                } else {
                                    // Infer type from default value if available
                                    if let Some(default_value) = &field_info.default_value {
                                        self.infer_type(default_value, ctx)
                                    } else {
                                        Err(TypeCheckError::TypeInferenceError {
                                            message: format!(
                                                "Cannot infer type for field {}",
                                                field_name
                                            ),
                                        })
                                    }
                                }
                            } else {
                                Err(TypeCheckError::UndefinedVariable(format!(
                                    "Field {} not found in type",
                                    field_name
                                )))
                            }
                        } else {
                            Ok(type_info.clone())
                        }
                    } else if let Some(type_info) = ctx.scope.get_type(&full_path) {
                        Ok(type_info.clone())
                    } else {
                        Err(TypeCheckError::UndefinedVariable(full_path.clone()))
                    }
                } else {
                    Err(TypeCheckError::UndefinedVariable(full_path))
                }
            }
            Expression::Await(exprs) => {
                for expr in exprs {
                    let expr_type = self.infer_type(expr, ctx)?;
                    if !matches!(expr_type, TypeInfo::Result { .. }) {
                        return Err(TypeCheckError::TypeInferenceError {
                            message: "Can only await Result types".to_string(),
                        });
                    }
                }
                Ok(TypeInfo::Simple("Any".to_string()))
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
                            return Err(TypeCheckError::UndefinedType(type_name.clone()));
                        }
                    }
                    TypeInfo::Custom { name, fields } => {
                        // Recursively check nested custom types
                        if !ctx.scope.contains_type(name) {
                            return Err(TypeCheckError::UndefinedType(name.clone()));
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
                        return Err(TypeCheckError::TypeMismatch {
                            expected: field_type.clone(),
                            found: default_type,
                            location: Default::default(),
                        });
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
                            return Err(TypeCheckError::TypeMismatch {
                                expected: existing_type.clone(),
                                found: param.type_info.clone(),
                                location: Default::default(),
                            });
                        }
                    }
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
                // Register handler return type in scope
                ctx.scope.insert_type(
                    "handler_return_type".to_string(),
                    handler.return_type.clone(),
                );

                // Check parameter types
                for param in &handler.parameters {
                    if let Some(existing_type) = ctx.scope.get_type(&param.name) {
                        if existing_type != param.type_info {
                            return Err(TypeCheckError::TypeMismatch {
                                expected: existing_type.clone(),
                                found: param.type_info.clone(),
                                location: Default::default(),
                            });
                        }
                    }
                }
                self.visit_handler_block(&handler.block, ctx)?;
            }
        }

        // Visit observe handlers if present
        if let Some(observe) = &agent.observe {
            for handler in &observe.handlers {
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
                            return Err(TypeCheckError::TypeMismatch {
                                expected: existing_type.clone(),
                                found: param.type_info.clone(),
                                location: Default::default(),
                            });
                        }
                    }
                }
                self.visit_handler_block(&handler.block, ctx)?;
            }
        }

        // Visit react handlers if present
        if let Some(react) = &agent.react {
            for handler in &react.handlers {
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
                            return Err(TypeCheckError::TypeMismatch {
                                expected: existing_type.clone(),
                                found: param.type_info.clone(),
                                location: Default::default(),
                            });
                        }
                    }
                }
                self.visit_handler_block(&handler.block, ctx)?;
            }
        }

        Ok(())
    }

    fn visit_state(&mut self, state: &mut StateDef, ctx: &mut TypeContext) -> TypeCheckResult<()> {
        // Check each state variable's type
        for var_def in &state.variables {
            let var_def = var_def.1;
            match &var_def.type_info {
                TypeInfo::Simple(type_name) => {
                    if !ctx.scope.contains_type(type_name) {
                        return Err(TypeCheckError::UndefinedType(type_name.clone()));
                    }
                }
                TypeInfo::Custom { name, fields } => {
                    if !ctx.scope.contains_type(name) {
                        return Err(TypeCheckError::UndefinedType(name.clone()));
                    }
                    self.check_custom_type_fields(fields, ctx)?;
                }
                _ => {}
            }

            // If there's an initial value, check its type
            if let Some(init_value) = &var_def.initial_value {
                let init_type = self.infer_type(init_value, ctx)?;
                if init_type != var_def.type_info {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: var_def.type_info.clone(),
                        found: init_type,
                        location: Default::default(),
                    });
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
                // Get target type
                let target_type = self.infer_type(&target[0], ctx)?;
                // Get value type
                let value_type = self.infer_type(value, ctx)?;
                // Check compatibility
                if target_type != value_type {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: target_type,
                        found: value_type,
                        location: Default::default(),
                    });
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
                        return Err(TypeCheckError::TypeInferenceError {
                            message: "No return type found for handler".to_string(),
                        });
                    };
                self.check_return_type(expr, &expected_type, ctx)?;
                Ok(())
            }
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
                // Check condition is boolean
                self.check_condition(condition, ctx)?;
                // Check blocks
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
        expr: &Expression,
        ctx: &mut TypeContext,
    ) -> TypeCheckResult<()> {
        // Infer type to perform type checking
        self.infer_type(expr, ctx)?;
        Ok(())
    }
}
