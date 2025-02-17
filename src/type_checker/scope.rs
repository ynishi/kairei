use crate::ast::TypeInfo;
use dashmap::DashMap;

/// Manages type scopes for type checking
#[derive(Clone)]
pub struct TypeScope {
    scopes: Vec<TypeScopeLayer>,
}

/// Single layer in the type scope stack
#[derive(Clone)]
struct TypeScopeLayer {
    types: DashMap<String, TypeInfo>,
}

impl Default for TypeScopeLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScopeLayer {
    fn new() -> Self {
        Self {
            types: DashMap::new(),
        }
    }
}

impl Default for TypeScope {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScope {
    /// Create a new type scope with an initial global scope
    pub fn new() -> Self {
        Self {
            scopes: vec![TypeScopeLayer::new()],
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(TypeScopeLayer::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Get a type by name, searching from innermost to outermost scope
    pub fn get_type(&self, name: &str) -> Option<TypeInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.types.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    /// Insert a type into the current scope
    pub fn insert_type(&self, name: String, ty: TypeInfo) {
        if let Some(scope) = self.scopes.last() {
            scope.types.insert(name, ty);
        }
    }

    /// Clear all scopes and reset to initial state
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.scopes.push(TypeScopeLayer::new());
    }

    /// Check if a type exists in any scope
    pub fn contains_type(&self, name: &str) -> bool {
        self.get_type(name).is_some()
    }

    /// Get the current scope depth
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_basics() {
        let mut scope = TypeScope::new();
        assert_eq!(scope.depth(), 1);

        // Test inserting and getting types
        scope.insert_type("int".to_string(), TypeInfo::Simple("Int".to_string()));
        assert!(scope.contains_type("int"));
        assert!(!scope.contains_type("float"));

        // Test scope nesting
        scope.enter_scope();
        assert_eq!(scope.depth(), 2);
        scope.insert_type("float".to_string(), TypeInfo::Simple("Float".to_string()));
        assert!(scope.contains_type("int")); // Can see outer scope
        assert!(scope.contains_type("float")); // Can see current scope

        // Test scope exit
        scope.exit_scope();
        assert_eq!(scope.depth(), 1);
        assert!(scope.contains_type("int")); // Still in outer scope
        assert!(!scope.contains_type("float")); // Inner scope gone

        // Test clear
        scope.clear();
        assert_eq!(scope.depth(), 1);
        assert!(!scope.contains_type("int"));
    }
}
