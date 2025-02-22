use crate::{
    ast::{BinaryOperator, Literal, TypeInfo},
    type_checker::{error::TypeCheckErrorMeta, TypeCheckError, TypeCheckResult, TypeContext},
};

/// Expression type inference implementation
pub(crate) trait ExpressionTypeChecker {
    fn infer_literal_type(&self, lit: &Literal, ctx: &TypeContext) -> TypeCheckResult<TypeInfo>;
    fn infer_binary_op_type(
        &self,
        left: &TypeInfo,
        right: &TypeInfo,
        op: &BinaryOperator,
    ) -> TypeCheckResult<TypeInfo>;
    fn is_numeric(&self, type_info: &TypeInfo) -> bool;
    fn is_float(&self, type_info: &TypeInfo) -> bool;
    fn is_boolean(&self, type_info: &TypeInfo) -> bool;
}

pub(crate) struct DefaultExpressionChecker;

impl DefaultExpressionChecker {
    pub fn new() -> Self {
        Self
    }
}

impl ExpressionTypeChecker for DefaultExpressionChecker {
    fn infer_literal_type(&self, lit: &Literal, _ctx: &TypeContext) -> TypeCheckResult<TypeInfo> {
        Ok(match lit {
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
                // Infer type from first item
                let first_type = self.infer_literal_type(&items[0], _ctx)?;

                // Check that all items have the same type
                for item in items.iter().skip(1) {
                    let item_type = self.infer_literal_type(item, _ctx)?;
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
            Literal::Map(entries) => {
                if entries.is_empty() {
                    return Err(TypeCheckError::TypeInferenceError {
                        message: "Cannot infer type of empty map".to_string(),
                    });
                }

                // Get first entry to infer key and value types
                let (first_key, first_value) = entries.iter().next().unwrap();
                let key_type =
                    self.infer_literal_type(&Literal::String(first_key.clone()), _ctx)?;
                let value_type = self.infer_literal_type(first_value, _ctx)?;

                // Check that all entries have consistent types
                for (key, value) in entries.iter().skip(1) {
                    let k_type = self.infer_literal_type(&Literal::String(key.clone()), _ctx)?;
                    let v_type = self.infer_literal_type(value, _ctx)?;

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
            Literal::Null => TypeInfo::Simple("Null".to_string()),
            _ => {
                return Err(TypeCheckError::TypeInferenceError {
                    message: "Unsupported literal type".to_string(),
                })
            }
        })
    }

    fn infer_binary_op_type(
        &self,
        left: &TypeInfo,
        right: &TypeInfo,
        op: &BinaryOperator,
    ) -> TypeCheckResult<TypeInfo> {
        use BinaryOperator::*;

        match op {
            Add | Subtract | Multiply | Divide => {
                if !self.is_numeric(left) || !self.is_numeric(right) {
                    return Err(TypeCheckError::InvalidOperatorType {
                        operator: op.to_string(),
                        left_type: left.clone(),
                        right_type: right.clone(),
                        meta: TypeCheckErrorMeta::new(
                            Default::default(),
                            "Only numeric types are supported",
                            "Use Int or Float types",
                        ),
                    });
                }
                // If either operand is Float, result is Float
                if self.is_float(left) || self.is_float(right) {
                    Ok(TypeInfo::Simple("Float".to_string()))
                } else {
                    Ok(TypeInfo::Simple("Int".to_string()))
                }
            }
            Equal | NotEqual | LessThan | GreaterThan | LessThanEqual | GreaterThanEqual => {
                Ok(TypeInfo::Simple("Boolean".to_string()))
            }
            And | Or => {
                if !self.is_boolean(left) || !self.is_boolean(right) {
                    return Err(TypeCheckError::InvalidOperatorType {
                        operator: op.to_string(),
                        left_type: left.clone(),
                        right_type: right.clone(),
                        meta: TypeCheckErrorMeta::new(
                            Default::default(),
                            "Only boolean types are supported",
                            "Use Boolean type",
                        ),
                    });
                }
                Ok(TypeInfo::Simple("Boolean".to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_literal_type_inference() -> TypeCheckResult<()> {
        let checker = DefaultExpressionChecker::new();
        let ctx = TypeContext::new();

        // Test basic literals
        assert!(matches!(
            checker.infer_literal_type(&Literal::Integer(42), &ctx)?,
            TypeInfo::Simple(s) if s == "Int"
        ));
        assert!(matches!(
            checker.infer_literal_type(&Literal::String("hello".to_string()), &ctx)?,
            TypeInfo::Simple(s) if s == "String"
        ));

        // Test list literals
        let list = vec![Literal::Integer(1), Literal::Integer(2)];
        let result = checker.infer_literal_type(&Literal::List(list), &ctx)?;
        assert!(matches!(
            result,
            TypeInfo::Array(inner) if matches!(*inner, TypeInfo::Simple(ref s) if s == "Int")
        ));

        // Test map literals
        let mut map = HashMap::new();
        map.insert("key".to_string(), Literal::Integer(42));
        let result = checker.infer_literal_type(&Literal::Map(map), &ctx)?;
        assert!(matches!(
            result,
            TypeInfo::Map(key_type, value_type)
            if matches!(*key_type, TypeInfo::Simple(ref s) if s == "String")
            && matches!(*value_type, TypeInfo::Simple(ref s) if s == "Int")
        ));

        Ok(())
    }

    #[test]
    fn test_binary_op_type_inference() -> TypeCheckResult<()> {
        let checker = DefaultExpressionChecker::new();

        // Test arithmetic operations
        let int_type = TypeInfo::Simple("Int".to_string());
        let float_type = TypeInfo::Simple("Float".to_string());
        let bool_type = TypeInfo::Simple("Boolean".to_string());

        // Int + Int = Int
        let result = checker.infer_binary_op_type(&int_type, &int_type, &BinaryOperator::Add)?;
        assert!(matches!(result, TypeInfo::Simple(s) if s == "Int"));

        // Int + Float = Float
        let result = checker.infer_binary_op_type(&int_type, &float_type, &BinaryOperator::Add)?;
        assert!(matches!(result, TypeInfo::Simple(s) if s == "Float"));

        // Boolean && Boolean = Boolean
        let result = checker.infer_binary_op_type(&bool_type, &bool_type, &BinaryOperator::And)?;
        assert!(matches!(result, TypeInfo::Simple(s) if s == "Boolean"));

        // Int < Int = Boolean
        let result =
            checker.infer_binary_op_type(&int_type, &int_type, &BinaryOperator::LessThan)?;
        assert!(matches!(result, TypeInfo::Simple(s) if s == "Boolean"));

        Ok(())
    }
}
