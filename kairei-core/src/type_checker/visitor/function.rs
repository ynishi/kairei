/// # Function Call Type Checking
///
/// ## Overview
///
/// This module implements function call type checking in the KAIREI type checker.
/// The implementation focuses on validating function calls, their arguments, and return types.
///
/// ## Implementation Details
///
/// ### 1. Function Type Checker
///
/// The core functionality is implemented through the `FunctionTypeChecker` trait:
///
/// ```text
/// pub(crate) trait FunctionTypeChecker {
///     fn check_function_call(
///         &self,
///         function: &str,
///         arguments: &[Expression],
///         ctx: &TypeContext,
///     ) -> TypeCheckResult<TypeInfo>;
/// }
/// ```
///
/// ### 2. Type Checking Process
///
/// 1. Function Signature Resolution
/// ```text
/// fn get_function_signature(&self, function: &str, ctx: &TypeContext) -> TypeCheckResult<TypeInfo>
/// ```
/// - Looks up function type in the scope
/// - Returns error for undefined functions
/// - Validates function type signature format
///
/// 2. Parameter Type Extraction
/// ```text
/// fn extract_parameter_types(&self, type_info: &TypeInfo) -> TypeCheckResult<(Vec<TypeInfo>, TypeInfo)>
/// ```
/// - Extracts expected parameter types from function signature
/// - Extracts return type information
/// - Validates function type structure
///
/// 3. Argument Type Checking
/// ```text
/// fn check_argument_types(
///     &self,
///     function: &str,
///     arguments: &[Expression],
///     expected_types: &[TypeInfo],
///     ctx: &TypeContext,
/// ) -> TypeCheckResult<()>
/// ```
/// - Validates argument count matches parameter count
/// - Checks each argument's type against expected parameter type
/// - Reports detailed errors for mismatches
///
/// ### 3. Error Handling
///
/// The implementation provides specific error types for:
/// - Undefined functions
/// - Wrong number of arguments
/// - Type mismatches in arguments
/// - Invalid function signatures
///
/// Example error messages:
/// ```text
/// "Function test_func requires 2 arguments, but 1 was provided"
/// "Invalid argument type for function test_func: expected Int, found String"
/// "Undefined function: unknown_func"
/// ```
///
/// ### 4. Testing
///
/// The implementation includes tests for:
/// - Basic function call validation
/// - Argument type checking
/// - Error cases:
///   - Undefined functions
///   - Wrong number of arguments
///   - Invalid argument types
///
/// ## Current Limitations
///
/// 1. Function Types
///    - Currently assumes Result<T, Error> return type
///    - Limited support for complex parameter types
///    - No support for generic functions
///
/// 2. Type Checking
///    - Only literal arguments are fully supported
///    - Limited type inference for complex expressions
///    - No support for function overloading
///
/// ## Future Considerations
///
/// 1. Type System Extensions
///    - Support for generic functions
///    - Function overloading
///    - Named arguments
///    - Optional parameters
///
/// 2. Error Reporting
///    - More detailed error messages
///    - Suggestions for fixing type errors
///    - Better location information in errors
///
/// 3. Performance
///    - Function signature caching
///    - Optimized type checking for common cases
use crate::{
    ast::{Argument, Expression, TypeInfo},
    type_checker::{TypeCheckResult, TypeContext, error::TypeCheckError},
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

    // Add a method to infer expression types that can handle function calls
    #[allow(unreachable_patterns)]
    pub fn infer_expression_type(
        &self,
        expr: &Expression,
        ctx: &TypeContext,
    ) -> TypeCheckResult<TypeInfo> {
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
                let left_type = self.infer_expression_type(left, ctx)?;
                let right_type = self.infer_expression_type(right, ctx)?;
                self.expression_checker
                    .infer_binary_op_type(&left_type, &right_type, op)
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => self.check_function_call(function, arguments, ctx),
            Expression::Think { args, .. } => {
                // Check argument types
                for arg in args {
                    match arg {
                        Argument::Named { value, .. } | Argument::Positional(value) => {
                            self.infer_expression_type(value, ctx)?;
                        }
                    }
                }
                // Think expressions return Result<String, Error>
                Ok(TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                })
            }
            Expression::StateAccess(path) => {
                // For state access, delegate to the same logic as in DefaultVisitor
                // This ensures consistent behavior between different implementations
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
                                        current_type =
                                            self.infer_expression_type(default_value, ctx)?;
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
                } else if let Some(type_info) = ctx.scope.get_type(&path.0.join(".")) {
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
            Expression::Ok(expr) => {
                let ok_type = self.infer_expression_type(expr, ctx)?;
                Ok(TypeInfo::Result {
                    ok_type: Box::new(ok_type),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                })
            }
            Expression::Err(expr) => {
                let err_type = self.infer_expression_type(expr, ctx)?;
                Ok(TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("Any".to_string())),
                    err_type: Box::new(err_type),
                })
            }
            Expression::Request { .. } => {
                // Request expressions typically return Result<T, Error>
                // For simplicity, we'll assume Any as the success type
                Ok(TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("Any".to_string())),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                })
            }
            Expression::Await(exprs) => {
                // Check all expressions in the await
                for expr in exprs {
                    self.infer_expression_type(expr, ctx)?;
                }
                // Await expressions typically return the same type as the inner expression
                // For simplicity, we'll assume Any
                Ok(TypeInfo::Simple("Any".to_string()))
            }
            // Handle any other expression types
            other => Err(TypeCheckError::type_inference_error(
                format!("Unsupported expression type: {:?}", other),
                Default::default(),
            )),
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

    fn get_parameter_types(
        &self,
        function: &str,
        ctx: &TypeContext,
    ) -> TypeCheckResult<Vec<TypeInfo>> {
        // Check if there's a specific parameter type definition
        let param_key = format!("{}.params", function);
        if let Some(param_type) = ctx.scope.get_type(&param_key) {
            match param_type {
                TypeInfo::Array(elem_type) => {
                    // If it's an array, return the element type as a single parameter
                    Ok(vec![(*elem_type).clone()])
                }
                _ => {
                    // For other types, return as is
                    Ok(vec![param_type.clone()])
                }
            }
        } else {
            // Fall back to extracting from function signature
            let func_type = self.get_function_signature(function, ctx)?;
            let (param_types, _) = self.extract_parameter_types(&func_type)?;
            Ok(param_types)
        }
    }

    #[allow(unreachable_patterns)]
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
                    TypeInfo::Map(key_type, value_type) => {
                        vec![(**key_type).clone(), (**value_type).clone()]
                    }
                    TypeInfo::Custom { fields, .. } => {
                        // For custom types, extract field types as parameters
                        fields
                            .values()
                            .filter_map(|field| field.type_info.clone())
                            .collect()
                    }
                    TypeInfo::Result {
                        ok_type: inner_ok,
                        err_type: inner_err,
                    } => {
                        // For nested Result types
                        vec![(**inner_ok).clone(), (**inner_err).clone()]
                    }
                    TypeInfo::Option(inner_type) => {
                        // For Option types
                        vec![(**inner_type).clone()]
                    }
                    // Default case for unsupported types
                    other => {
                        return Err(TypeCheckError::type_inference_error(
                            format!("Invalid function parameter type: {:?}", other),
                            Default::default(),
                        ));
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
            // Use infer_expression_type to handle all expression types including nested function calls
            let arg_type = self.infer_expression_type(arg, ctx)?;

            // Special handling for Result types - if the expected type is a Result, we can accept it directly
            // If the expected type is not a Result but the argument is, we can extract the ok_type
            let compatible = match (expected_type, &arg_type) {
                // If both are Result types, check compatibility of ok and err types
                (
                    TypeInfo::Result {
                        ok_type: expected_ok,
                        err_type: expected_err,
                    },
                    TypeInfo::Result {
                        ok_type: actual_ok,
                        err_type: actual_err,
                    },
                ) => **expected_ok == **actual_ok && **expected_err == **actual_err,
                // If expected is not Result but arg is, check if ok_type matches expected
                (expected, TypeInfo::Result { ok_type, .. }) => **ok_type == *expected,
                // Default case - direct equality check
                _ => arg_type == *expected_type,
            };

            if !compatible {
                return Err(TypeCheckError::invalid_argument_type(
                    function.to_string(),
                    format!("arg{}", i),
                    expected_type.clone(),
                    arg_type,
                    Default::default(),
                ));
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
        // Get function signature for return type
        let func_type = self.get_function_signature(function, ctx)?;
        let (_, return_type) = self.extract_parameter_types(&func_type)?;

        // Get parameter types (either from specific param definition or function signature)
        let param_types = self.get_parameter_types(function, ctx)?;

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
                    ));
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
                    ));
                }
            },
            _ => {
                return Err(TypeCheckError::type_inference_error(
                    format!("Invalid return type for function {}", function),
                    Default::default(),
                ));
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
