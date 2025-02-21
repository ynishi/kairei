use crate::{
    ast::{
        BinaryOperator, Expression, HandlerBlock, HandlerDef, MicroAgentDef, Root, StateDef,
        Statement, TypeInfo,
    },
    type_checker::{visitor::common::TypeVisitor, TypeCheckError, TypeCheckResult, TypeContext},
    Argument,
};

/// Default implementation of type checking logic
pub struct DefaultVisitor;

impl DefaultVisitor {
    pub fn new() -> Self {
        Self
    }

    fn infer_type(&self, expr: &Expression, ctx: &TypeContext) -> TypeCheckResult<TypeInfo> {
        match expr {
            Expression::Literal(lit) => Ok(match lit {
                crate::ast::Literal::Integer(_) => TypeInfo::Simple("Int".to_string()),
                crate::ast::Literal::Float(_) => TypeInfo::Simple("Float".to_string()),
                crate::ast::Literal::String(_) => TypeInfo::Simple("String".to_string()),
                crate::ast::Literal::Boolean(_) => TypeInfo::Simple("Boolean".to_string()),
                crate::ast::Literal::Duration(_) => TypeInfo::Simple("Duration".to_string()),
                crate::ast::Literal::List(items) => {
                    if items.is_empty() {
                        return Err(TypeCheckError::TypeInferenceError {
                            message: "Cannot infer type of empty list".to_string(),
                        });
                    }
                    // Infer type from first item
                    let first_type =
                        self.infer_type(&Expression::Literal(items[0].clone()), ctx)?;

                    // Check that all items have the same type
                    for item in items.iter().skip(1) {
                        let item_type = self.infer_type(&Expression::Literal(item.clone()), ctx)?;
                        if item_type != first_type {
                            return Err(TypeCheckError::TypeInferenceError {
                                message: format!(
                                    "List contains mixed types: found both {} and {}",
                                    first_type, item_type
                                ),
                            });
                        }
                    }

                    TypeInfo::Array(Box::new(first_type))
                }
                crate::ast::Literal::Map(entries) => {
                    if entries.is_empty() {
                        return Err(TypeCheckError::TypeInferenceError {
                            message: "Cannot infer type of empty map".to_string(),
                        });
                    }

                    // Get first entry to infer key and value types
                    let (first_key, first_value) = entries.iter().next().unwrap();
                    let key_type = self.infer_type(
                        &Expression::Literal(crate::ast::Literal::String(first_key.clone())),
                        ctx,
                    )?;
                    let value_type =
                        self.infer_type(&Expression::Literal(first_value.clone()), ctx)?;

                    // Check that all entries have consistent types
                    for (key, value) in entries.iter().skip(1) {
                        let k_type = self.infer_type(
                            &Expression::Literal(crate::ast::Literal::String(key.clone())),
                            ctx,
                        )?;
                        let v_type = self.infer_type(&Expression::Literal(value.clone()), ctx)?;

                        if k_type != key_type {
                            return Err(TypeCheckError::TypeInferenceError {
                                message: format!(
                                    "Map contains mixed key types: found both {} and {}",
                                    key_type, k_type
                                ),
                            });
                        }
                        if v_type != value_type {
                            return Err(TypeCheckError::TypeInferenceError {
                                message: format!(
                                    "Map contains mixed value types: found both {} and {}",
                                    value_type, v_type
                                ),
                            });
                        }
                    }

                    TypeInfo::Map(Box::new(key_type), Box::new(value_type))
                }
                crate::ast::Literal::Null => TypeInfo::Simple("Null".to_string()),
                _ => {
                    return Err(TypeCheckError::TypeInferenceError {
                        message: "Unsupported literal type".to_string(),
                    })
                }
            }),
            // ... rest of the match arms remain unchanged ...
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
                match op {
                    BinaryOperator::Add
                    | BinaryOperator::Subtract
                    | BinaryOperator::Multiply
                    | BinaryOperator::Divide => {
                        if !self.is_numeric(&left_type) || !self.is_numeric(&right_type) {
                            return Err(TypeCheckError::InvalidOperatorType {
                                operator: op.to_string(),
                                left_type,
                                right_type,
                                location: Default::default(),
                            });
                        }
                        // If either operand is Float, result is Float
                        if self.is_float(&left_type) || self.is_float(&right_type) {
                            Ok(TypeInfo::Simple("Float".to_string()))
                        } else {
                            Ok(TypeInfo::Simple("Int".to_string()))
                        }
                    }
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::LessThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::LessThanEqual
                    | BinaryOperator::GreaterThanEqual => {
                        Ok(TypeInfo::Simple("Boolean".to_string()))
                    }
                    BinaryOperator::And | BinaryOperator::Or => {
                        if !self.is_boolean(&left_type) || !self.is_boolean(&right_type) {
                            return Err(TypeCheckError::InvalidOperatorType {
                                operator: op.to_string(),
                                left_type,
                                right_type,
                                location: Default::default(),
                            });
                        }
                        Ok(TypeInfo::Simple("Boolean".to_string()))
                    }
                }
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                // Get function type from scope
                let func_type = ctx
                    .scope
                    .get_type(function)
                    .ok_or_else(|| TypeCheckError::UndefinedFunction(function.clone()))?;

                // Check argument types
                for (i, arg) in arguments.iter().enumerate() {
                    let arg_type = self.infer_type(arg, ctx)?;
                    // For now, we don't have function parameter types, so we skip type checking
                    // In a full implementation, we would check against the function's parameter types
                    if false {
                        return Err(TypeCheckError::InvalidArgumentType {
                            function: function.clone(),
                            argument: format!("arg{}", i),
                            expected: TypeInfo::Simple("Any".to_string()),
                            found: arg_type,
                            location: Default::default(),
                        });
                    }
                }

                // For now, assume function calls return Result<Any, Error>
                Ok(func_type.clone())
            }
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

    fn is_numeric(&self, type_info: &TypeInfo) -> bool {
        matches!(
            type_info,
            TypeInfo::Simple(name) if name == "Int" || name == "Float"
        )
    }

    fn is_float(&self, type_info: &TypeInfo) -> bool {
        matches!(
            type_info,
            TypeInfo::Simple(name) if name == "Float"
        )
    }

    fn is_boolean(&self, type_info: &TypeInfo) -> bool {
        matches!(
            type_info,
            TypeInfo::Simple(name) if name == "Boolean"
        )
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
                let expr_type = self.infer_type(expr, ctx)?;
                // For now, we don't have function return types, so we skip type checking
                // In a full implementation, we would check against the function's return type
                if false {
                    return Err(TypeCheckError::InvalidReturnType {
                        expected: TypeInfo::Simple("Any".to_string()),
                        found: expr_type,
                        location: Default::default(),
                    });
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
                if !self.is_boolean(&cond_type) {
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

#[cfg(test)]
mod tests {
    use crate::Literal;

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_map_type_inference() -> TypeCheckResult<()> {
        let visitor = DefaultVisitor::new();
        let ctx = TypeContext::new();

        // Test empty map
        let empty_map: HashMap<String, Literal> = HashMap::new();
        let expr = Expression::Literal(Literal::Map(empty_map));
        let result = visitor.infer_type(&expr, &ctx);
        assert!(matches!(
            result,
            Err(TypeCheckError::TypeInferenceError { .. })
        ));

        // Test map with consistent types
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Literal::Integer(1));
        map.insert("key2".to_string(), Literal::Integer(2));
        let expr = Expression::Literal(Literal::Map(map));
        let result = visitor.infer_type(&expr, &ctx)?;
        assert!(matches!(
            result,
            TypeInfo::Map(key_type, value_type)
            if matches!(*key_type, TypeInfo::Simple(ref s) if s == "String")
            && matches!(*value_type, TypeInfo::Simple(ref s) if s == "Int")
        ));

        // Test map with mixed value types
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Literal::Integer(1));
        map.insert("key2".to_string(), Literal::String("value".to_string()));
        let expr = Expression::Literal(Literal::Map(map));
        let result = visitor.infer_type(&expr, &ctx);
        assert!(matches!(
            result,
            Err(TypeCheckError::TypeInferenceError { .. })
        ));

        Ok(())
    }

    #[test]
    fn test_list_type_inference() -> TypeCheckResult<()> {
        let visitor = DefaultVisitor::new();
        let ctx = TypeContext::new();

        // Test empty list
        let expr = Expression::Literal(Literal::List(vec![]));
        let result = visitor.infer_type(&expr, &ctx);
        assert!(matches!(
            result,
            Err(TypeCheckError::TypeInferenceError { .. })
        ));

        // Test list with consistent types
        let expr = Expression::Literal(Literal::List(vec![
            Literal::Integer(1),
            Literal::Integer(2),
            Literal::Integer(3),
        ]));
        let result = visitor.infer_type(&expr, &ctx)?;
        assert!(matches!(
            result,
            TypeInfo::Array(inner) if matches!(*inner, TypeInfo::Simple(ref s) if s == "Int")
        ));

        // Test list with mixed types
        let expr = Expression::Literal(Literal::List(vec![
            Literal::Integer(1),
            Literal::String("value".to_string()),
        ]));
        let result = visitor.infer_type(&expr, &ctx);
        assert!(matches!(
            result,
            Err(TypeCheckError::TypeInferenceError { .. })
        ));

        Ok(())
    }
}
