use super::*;
use crate::ast::TypeInfo;

#[test]
fn test_type_scope_basic_operations() {
    let mut scope = TypeScope::new();
    
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
    let mut scope = TypeScope::new();
    
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
    let mut scope = TypeScope::new();
    
    scope.insert_type("test".to_string(), TypeInfo::Simple("Int".to_string()));
    assert!(scope.contains_type("test"));
    
    scope.clear();
    assert_eq!(scope.depth(), 1);
    assert!(!scope.contains_type("test"));
}
