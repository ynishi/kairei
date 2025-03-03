use crate::{
    ast::{BinaryOperator, Expression, Literal, TypeInfo},
    type_checker::{TypeCheckResult, TypeContext},
    type_checker::visitor::expression::{DefaultExpressionChecker, ExpressionTypeChecker},
};

#[test]
fn test_binary_op_type_inference_mixed_types() -> TypeCheckResult<()> {
    let checker = DefaultExpressionChecker::new();
    let ctx = TypeContext::new();

    // Test numeric operations
    let int_type = TypeInfo::Simple("Int".to_string());
    let float_type = TypeInfo::Simple("Float".to_string());
    let string_type = TypeInfo::Simple("String".to_string());
    let bool_type = TypeInfo::Simple("Boolean".to_string());

    // Int + Int = Int
    let result = checker.infer_binary_op_type(&int_type, &int_type, &BinaryOperator::Add)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "Int"));

    // Int + Float = Float (type promotion)
    let result = checker.infer_binary_op_type(&int_type, &float_type, &BinaryOperator::Add)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "Float"));

    // Float + Int = Float (type promotion)
    let result = checker.infer_binary_op_type(&float_type, &int_type, &BinaryOperator::Add)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "Float"));

    // String + String = String
    let result = checker.infer_binary_op_type(&string_type, &string_type, &BinaryOperator::Add)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "String"));

    // String + Int = String (string concatenation)
    let result = checker.infer_binary_op_type(&string_type, &int_type, &BinaryOperator::Add)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "String"));

    // Int + String = String (string concatenation)
    let result = checker.infer_binary_op_type(&int_type, &string_type, &BinaryOperator::Add)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "String"));

    // Boolean && Boolean = Boolean
    let result = checker.infer_binary_op_type(&bool_type, &bool_type, &BinaryOperator::And)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "Boolean"));

    // Int < Int = Boolean
    let result = checker.infer_binary_op_type(&int_type, &int_type, &BinaryOperator::LessThan)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "Boolean"));

    // Int == Int = Boolean
    let result = checker.infer_binary_op_type(&int_type, &int_type, &BinaryOperator::Equal)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "Boolean"));

    // String == Int = Boolean (any types can be compared for equality)
    let result = checker.infer_binary_op_type(&string_type, &int_type, &BinaryOperator::Equal)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "Boolean"));

    Ok(())
}

#[test]
fn test_binary_op_type_inference_nested() -> TypeCheckResult<()> {
    let checker = DefaultExpressionChecker::new();
    let ctx = TypeContext::new();

    // Create nested binary operations
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Literal(Literal::Integer(1))),
            right: Box::new(Expression::Literal(Literal::Integer(2))),
            op: BinaryOperator::Add,
        }),
        right: Box::new(Expression::Literal(Literal::Integer(3))),
        op: BinaryOperator::Multiply,
    };

    // Infer the type of the nested binary operation
    let result = checker.infer_type(&expr, &ctx)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "Int"));

    // Create nested binary operations with mixed types
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Literal(Literal::Integer(1))),
            right: Box::new(Expression::Literal(Literal::Float(2.0))),
            op: BinaryOperator::Add,
        }),
        right: Box::new(Expression::Literal(Literal::Integer(3))),
        op: BinaryOperator::Multiply,
    };

    // Infer the type of the nested binary operation with mixed types
    let result = checker.infer_type(&expr, &ctx)?;
    assert!(matches!(result, TypeInfo::Simple(s) if s == "Float"));

    Ok(())
}

#[test]
fn test_binary_op_type_inference_errors() -> TypeCheckResult<()> {
    let checker = DefaultExpressionChecker::new();
    let ctx = TypeContext::new();

    let int_type = TypeInfo::Simple("Int".to_string());
    let string_type = TypeInfo::Simple("String".to_string());
    let bool_type = TypeInfo::Simple("Boolean".to_string());

    // String * Int = Error
    let result = checker.infer_binary_op_type(&string_type, &int_type, &BinaryOperator::Multiply);
    assert!(result.is_err());

    // Int && Boolean = Error
    let result = checker.infer_binary_op_type(&int_type, &bool_type, &BinaryOperator::And);
    assert!(result.is_err());

    // String < Int = Error
    let result = checker.infer_binary_op_type(&string_type, &int_type, &BinaryOperator::LessThan);
    assert!(result.is_err());

    Ok(())
}
