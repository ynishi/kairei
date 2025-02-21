use crate::ast::{
    Expression, HandlerBlock, HandlerDef, MicroAgentDef, Root, StateDef, Statement, TypeInfo,
};
use crate::type_checker::{
    visitor::common::TypeVisitor, TypeCheckError, TypeCheckResult, TypeContext,
};
use crate::Argument;

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
                if let Some(type_info) = ctx.scope.get_type(&path.0.join(".")) {
                    Ok(type_info.clone())
                } else {
                    Err(TypeCheckError::UndefinedVariable(path.0.join(".")))
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
                if let Some(expected_type) = ctx.scope.get_type("return_type") {
                    self.function_checker.check_return_type(
                        "current_function",
                        expr,
                        &expected_type,
                        ctx,
                    )?;
                }
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
                let cond_type = self.infer_type(condition, ctx)?;
                if !self.expression_checker.is_boolean(&cond_type) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: TypeInfo::Simple("Boolean".to_string()),
                        found: cond_type,
                        location: Default::default(),
                    });
                }
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
