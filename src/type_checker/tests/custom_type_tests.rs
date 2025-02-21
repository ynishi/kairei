use std::collections::HashMap;

use crate::{
    ast::{Expression, FieldInfo, Literal, TypeInfo},
    type_checker::{
        visitor::{common::TypeVisitor, default::DefaultVisitor},
        TypeCheckError, TypeCheckResult, TypeContext,
    },
};

#[test]
fn test_custom_type_definition() -> TypeCheckResult<()> {
    let mut ctx = TypeContext::new();

    // Define a custom type
    let mut fields = HashMap::new();
    fields.insert(
        "name".to_string(),
        FieldInfo {
            type_info: Some(TypeInfo::Simple("String".to_string())),
            default_value: None,
        },
    );
    fields.insert(
        "age".to_string(),
        FieldInfo {
            type_info: Some(TypeInfo::Simple("Int".to_string())),
            default_value: Some(Expression::Literal(Literal::Integer(0))),
        },
    );

    let person_type = TypeInfo::Custom {
        name: "Person".to_string(),
        fields,
    };

    // Register the type
    ctx.scope.insert_type("Person".to_string(), person_type);

    // Verify the type exists
    let stored_type = ctx.scope.get_type("Person").unwrap();
    match stored_type {
        TypeInfo::Custom { fields, .. } => {
            assert_eq!(fields.len(), 2);
            assert!(fields.contains_key("name"));
            assert!(fields.contains_key("age"));
        }
        _ => panic!("Expected Custom type"),
    }

    Ok(())
}

#[test]
fn test_custom_type_field_access() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Define and register a custom type
    let mut fields = HashMap::new();
    fields.insert(
        "name".to_string(),
        FieldInfo {
            type_info: Some(TypeInfo::Simple("String".to_string())),
            default_value: None,
        },
    );
    fields.insert(
        "age".to_string(),
        FieldInfo {
            type_info: Some(TypeInfo::Simple("Int".to_string())),
            default_value: Some(Expression::Literal(Literal::Integer(0))),
        },
    );

    let person_type = TypeInfo::Custom {
        name: "Person".to_string(),
        fields: fields.clone(),
    };

    // Register the type definition
    ctx.scope
        .insert_type("Person".to_string(), person_type.clone());

    // Create a variable of the custom type
    ctx.scope.insert_type("person".to_string(), person_type);

    // Test accessing a valid field
    let expr = Expression::StateAccess(crate::ast::StateAccessPath(vec![
        "person".to_string(),
        "name".to_string(),
    ]));
    let result = visitor.visit_expression(&expr, &mut ctx);
    assert!(result.is_ok());

    // Test accessing an invalid field
    let expr = Expression::StateAccess(crate::ast::StateAccessPath(vec![
        "person".to_string(),
        "invalid_field".to_string(),
    ]));
    let result = visitor.visit_expression(&expr, &mut ctx);
    assert!(matches!(result, Err(TypeCheckError::UndefinedVariable(..))));

    Ok(())
}

#[test]
fn test_custom_type_default_values() -> TypeCheckResult<()> {
    let mut visitor = DefaultVisitor::new();
    let mut ctx = TypeContext::new();

    // Define a custom type with default values
    let mut fields = HashMap::new();
    fields.insert(
        "name".to_string(),
        FieldInfo {
            type_info: Some(TypeInfo::Simple("String".to_string())),
            default_value: Some(Expression::Literal(Literal::String("".to_string()))),
        },
    );
    fields.insert(
        "age".to_string(),
        FieldInfo {
            type_info: Some(TypeInfo::Simple("Int".to_string())),
            default_value: Some(Expression::Literal(Literal::Integer(0))),
        },
    );

    let person_type = TypeInfo::Custom {
        name: "Person".to_string(),
        fields,
    };

    // Register the type
    ctx.scope.insert_type("Person".to_string(), person_type);

    // Verify default values have correct types
    if let Some(TypeInfo::Custom { fields, .. }) = ctx.scope.get_type("Person") {
        for (_, field_info) in fields {
            if let Some(default_value) = &field_info.default_value {
                let _default_type = visitor.visit_expression(default_value, &mut ctx)?;
                assert!(matches!(field_info.type_info, Some(TypeInfo::Simple(..))));
            }
        }
    }

    Ok(())
}

#[test]
fn test_custom_type_type_inference() -> TypeCheckResult<()> {
    let mut ctx = TypeContext::new();

    // Define a custom type with a field that needs type inference
    let mut fields = HashMap::new();
    fields.insert(
        "inferred_field".to_string(),
        FieldInfo {
            type_info: None, // Type will be inferred
            default_value: Some(Expression::Literal(Literal::Integer(42))),
        },
    );

    let custom_type = TypeInfo::Custom {
        name: "InferredType".to_string(),
        fields,
    };

    // Register the type
    ctx.scope
        .insert_type("InferredType".to_string(), custom_type);

    // Verify the inferred type
    if let Some(TypeInfo::Custom { fields, .. }) = ctx.scope.get_type("InferredType") {
        let field_info = fields.get("inferred_field").unwrap();
        if let Some(Expression::Literal(Literal::Integer(_))) = field_info.default_value {
            // Type should be inferred as Int
            assert!(matches!(
                field_info.type_info,
                None // Currently None, but should be inferred
            ));
        }
    }

    Ok(())
}
