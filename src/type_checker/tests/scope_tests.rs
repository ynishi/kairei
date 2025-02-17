use super::*;
use crate::ast::TypeInfo;
use std::thread;

#[test]
fn test_type_scope_basic_operations() {
    let scope = TypeScope::new();

    // Test initial state
    assert_eq!(scope.depth(), 1);
    assert!(!scope.contains_type("test"));

    // Test type insertion and lookup
    scope.insert_type("test".to_string(), TypeInfo::Simple("String".to_string()));
    assert!(scope.contains_type("test"));
    assert!(matches!(
        scope.get_type("test"),
        Some(TypeInfo::Simple(t)) if t == "String"
    ));
}

#[test]
fn test_type_scope_nesting() {
    let scope = TypeScope::new();

    // Add type in outer scope
    scope.insert_type("outer".to_string(), TypeInfo::Simple("Int".to_string()));

    // Create nested scope
    scope.enter_scope();
    assert_eq!(scope.depth(), 2);

    // Add type in inner scope
    scope.insert_type("inner".to_string(), TypeInfo::Simple("String".to_string()));

    // Test visibility
    assert!(scope.contains_type("outer")); // Outer type visible in inner scope
    assert!(scope.contains_type("inner")); // Inner type visible in current scope

    // Exit scope
    scope.exit_scope();
    assert_eq!(scope.depth(), 1);
    assert!(scope.contains_type("outer")); // Outer type still visible
    assert!(!scope.contains_type("inner")); // Inner type no longer visible
}

#[test]
fn test_type_scope_clear() {
    let scope = TypeScope::new();

    scope.insert_type("test".to_string(), TypeInfo::Simple("Int".to_string()));
    assert!(scope.contains_type("test"));

    scope.clear();
    assert_eq!(scope.depth(), 1);
    assert!(!scope.contains_type("test"));
}

#[test]
fn test_type_scope_complex_types() {
    let scope = TypeScope::new();

    // Test array type
    let array_type = TypeInfo::Array(Box::new(TypeInfo::Simple("Int".to_string())));
    scope.insert_type("numbers".to_string(), array_type.clone());
    assert!(scope.contains_type("numbers"));
    assert!(matches!(
        scope.get_type("numbers"),
        Some(TypeInfo::Array(_))
    ));

    // Test result type
    let result_type = TypeInfo::Result {
        ok_type: Box::new(TypeInfo::Simple("String".to_string())),
        err_type: Box::new(TypeInfo::Simple("Error".to_string())),
    };
    scope.insert_type("result".to_string(), result_type.clone());
    assert!(scope.contains_type("result"));
    assert!(matches!(
        scope.get_type("result"),
        Some(TypeInfo::Result { .. })
    ));
}

#[test]
fn test_type_scope_concurrent_access() {
    let scope = TypeScope::new();
    let scope_clone = scope.clone();

    // Insert type in main thread
    scope.insert_type("main".to_string(), TypeInfo::Simple("Int".to_string()));

    // Access type in another thread
    let handle = thread::spawn(move || {
        assert!(scope_clone.contains_type("main"));
        scope_clone.insert_type("thread".to_string(), TypeInfo::Simple("String".to_string()));
    });

    handle.join().unwrap();
    assert!(scope.contains_type("thread"));
}

#[test]
fn test_type_scope_deep_nesting() {
    let scope = TypeScope::new();

    // Create multiple nested scopes
    for i in 0..5 {
        scope.enter_scope();
        scope.insert_type(
            format!("var_{}", i),
            TypeInfo::Simple(format!("Type_{}", i)),
        );
    }

    // Verify all types are accessible from innermost scope
    for i in 0..5 {
        assert!(scope.contains_type(&format!("var_{}", i)));
    }

    // Exit scopes one by one and verify visibility
    for i in (0..5).rev() {
        scope.exit_scope();
        assert!(!scope.contains_type(&format!("var_{}", i)));
        for j in 0..i {
            assert!(scope.contains_type(&format!("var_{}", j)));
        }
    }
}
