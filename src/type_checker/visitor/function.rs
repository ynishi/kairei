use crate::{
    ast::{Expression, TypeInfo},
    type_checker::{TypeCheckError, TypeCheckResult, TypeContext},
};

use super::expression::{DefaultExpressionChecker, ExpressionTypeChecker};

/// Function type checking implementation
pub(crate) trait FunctionTypeChecker {
    fn check_function_call(
        &self,
        function: &str,
        arguments: &[Expression],
        ctx: &TypeContext,
    ) -> TypeCheckResult<TypeInfo>;

    #[allow(dead_code)]
    fn check_return_type(
        &self,
        function: &str,
        return_expr: &Expression,
        expected_type: &TypeInfo,
        ctx: &TypeContext,
    ) -> TypeCheckResult<()>;
}

pub(crate) struct DefaultFunctionChecker {
    expression_checker: DefaultExpressionChecker,
}

impl DefaultFunctionChecker {
    pub fn new() -> Self {
        Self {
            expression_checker: DefaultExpressionChecker::new(),
        }
    }

    fn get_function_signature(
        &self,
        function: &str,
        ctx: &TypeContext,
    ) -> TypeCheckResult<TypeInfo> {
        ctx.scope.get_type(function).ok_or_else(|| {
            TypeCheckError::undefined_function(function.to_string(), Default::default())
        })
    }

    fn extract_parameter_types(
        &self,
        type_info: &TypeInfo,
    ) -> TypeCheckResult<(Vec<TypeInfo>, TypeInfo)> {
        match type_info {
            TypeInfo::Result { ok_type, err_type } => {
                // Extract parameter types from ok_type
                let param_types = match &**ok_type {
                    TypeInfo::Simple(_) => vec![(**ok_type).clone()],
                    TypeInfo::Array(elem_type) => vec![(**elem_type).clone()],
                    _ => {
                        return Err(TypeCheckError::type_inference_error(
                            "Invalid function parameter type".to_string(),
                            Default::default(),
                        ))
                    }
                };

                // Return type is Result<T, Error>
                let return_type = TypeInfo::Result {
                    ok_type: ok_type.clone(),
                    err_type: err_type.clone(),
                };

                Ok((param_types, return_type))
            }
            _ => Err(TypeCheckError::type_inference_error(
                "Invalid function type signature".to_string(),
                Default::default(),
            )),
        }
    }

    fn check_argument_types(
        &self,
        function: &str,
        arguments: &[Expression],
        expected_types: &[TypeInfo],
        ctx: &TypeContext,
    ) -> TypeCheckResult<()> {
        // Check number of arguments
        if arguments.len() != expected_types.len() {
            return Err(TypeCheckError::type_inference_error(
                format!(
                    "Function {} requires {} arguments, but {} were provided",
                    function,
                    expected_types.len(),
                    arguments.len()
                ),
                Default::default(),
            ));
        }

        // Check each argument type
        for (i, (arg, expected_type)) in arguments.iter().zip(expected_types.iter()).enumerate() {
            match arg {
                Expression::Literal(lit) => {
                    let arg_type = self.expression_checker.infer_literal_type(lit, ctx)?;
                    if arg_type != *expected_type {
                        return Err(TypeCheckError::InvalidArgumentType {
                            function: function.to_string(),
                            argument: format!("arg{}", i),
                            expected: expected_type.clone(),
                            found: arg_type,
                            location: Default::default(),
                        });
                    }
                }
                _ => {
                    return Err(TypeCheckError::type_inference_error(
                        format!("Unsupported argument type for function {}", function),
                        Default::default(),
                    ))
                }
            }
        }

        Ok(())
    }
}

impl FunctionTypeChecker for DefaultFunctionChecker {
    fn check_function_call(
        &self,
        function: &str,
        arguments: &[Expression],
        ctx: &TypeContext,
    ) -> TypeCheckResult<TypeInfo> {
        // Get function signature
        let func_type = self.get_function_signature(function, ctx)?;

        // Extract parameter types and return type
        let (param_types, return_type) = self.extract_parameter_types(&func_type)?;

        // Check argument types
        self.check_argument_types(function, arguments, &param_types, ctx)?;

        // Return function's return type
        Ok(return_type)
    }

    fn check_return_type(
        &self,
        function: &str,
        return_expr: &Expression,
        expected_type: &TypeInfo,
        ctx: &TypeContext,
    ) -> TypeCheckResult<()> {
        match (return_expr, expected_type) {
            (Expression::Literal(lit), TypeInfo::Result { ok_type, .. }) => {
                let actual_type = self.expression_checker.infer_literal_type(lit, ctx)?;
                if actual_type != **ok_type {
                    return Err(TypeCheckError::invalid_return_type(
                        (**ok_type).clone(),
                        actual_type,
                        Default::default(),
                    ));
                }
            }
            (Expression::Ok(expr), TypeInfo::Result { ok_type, .. }) => match &**expr {
                Expression::Literal(lit) => {
                    let actual_type = self.expression_checker.infer_literal_type(lit, ctx)?;
                    if actual_type != **ok_type {
                        return Err(TypeCheckError::invalid_return_type(
                            (**ok_type).clone(),
                            actual_type,
                            Default::default(),
                        ));
                    }
                }
                _ => {
                    return Err(TypeCheckError::type_inference_error(
                        format!("Unsupported Ok value type for function {}", function),
                        Default::default(),
                    ))
                }
            },
            (Expression::Err(expr), TypeInfo::Result { err_type, .. }) => match &**expr {
                Expression::Literal(lit) => {
                    let actual_type = self.expression_checker.infer_literal_type(lit, ctx)?;
                    if actual_type != **err_type {
                        return Err(TypeCheckError::invalid_return_type(
                            (**err_type).clone(),
                            actual_type,
                            Default::default(),
                        ));
                    }
                }
                _ => {
                    return Err(TypeCheckError::type_inference_error(
                        format!("Unsupported Err value type for function {}", function),
                        Default::default(),
                    ))
                }
            },
            _ => {
                return Err(TypeCheckError::type_inference_error(
                    format!("Invalid return type for function {}", function),
                    Default::default(),
                ))
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Literal;

    use super::*;

    #[test]
    fn test_function_call_type_checking() -> TypeCheckResult<()> {
        let checker = DefaultFunctionChecker::new();
        let mut ctx = TypeContext::new();

        // Register a test function
        ctx.scope.insert_type(
            "test_func".to_string(),
            TypeInfo::Result {
                ok_type: Box::new(TypeInfo::Simple("Int".to_string())),
                err_type: Box::new(TypeInfo::Simple("Error".to_string())),
            },
        );

        // Test with correct argument type
        let args = vec![Expression::Literal(Literal::Integer(42))];
        let result = checker.check_function_call("test_func", &args, &ctx)?;
        assert!(matches!(result, TypeInfo::Result { .. }));

        // Test with wrong number of arguments
        let args = vec![];
        let result = checker.check_function_call("test_func", &args, &ctx);
        assert!(matches!(
            result,
            Err(TypeCheckError::TypeInferenceError { .. })
        ));

        // Test with wrong argument type
        let args = vec![Expression::Literal(Literal::String(
            "wrong type".to_string(),
        ))];
        let result = checker.check_function_call("test_func", &args, &ctx);
        assert!(matches!(
            result,
            Err(TypeCheckError::InvalidArgumentType { .. })
        ));

        // Test undefined function
        let args = vec![Expression::Literal(Literal::Integer(42))];
        let result = checker.check_function_call("undefined_func", &args, &ctx);
        assert!(matches!(
            result,
            Err(TypeCheckError::UndefinedFunction { name: _, meta: _ })
        ));

        Ok(())
    }

    #[test]
    fn test_return_type_checking() -> TypeCheckResult<()> {
        let checker = DefaultFunctionChecker::new();
        let ctx = TypeContext::new();

        let return_type = TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("Int".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        };

        // Test correct return type
        let return_expr = Expression::Ok(Box::new(Expression::Literal(Literal::Integer(42))));
        let result = checker.check_return_type("test_func", &return_expr, &return_type, &ctx);
        assert!(result.is_ok());

        // Test wrong return type
        let return_expr = Expression::Ok(Box::new(Expression::Literal(Literal::String(
            "wrong".to_string(),
        ))));
        let result = checker.check_return_type("test_func", &return_expr, &return_type, &ctx);
        assert!(matches!(
            result,
            Err(TypeCheckError::InvalidReturnType { .. })
        ));

        // Test error return type
        let return_expr = Expression::Err(Box::new(Expression::Literal(Literal::String(
            "error".to_string(),
        ))));
        let result = checker.check_return_type("test_func", &return_expr, &return_type, &ctx);
        assert!(matches!(
            result,
            Err(TypeCheckError::InvalidReturnType { .. })
        ));

        Ok(())
    }
}
