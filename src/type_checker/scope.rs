use std::collections::HashMap;

use crate::ast::TypeInfo;

/// Manages type scopes for type checking
#[derive(Clone)]
/// TypeScope is designed to store type information for type checking.
/// Although it currently uses DashMap to allow for potential concurrent access,
/// the overall type resolution process is inherently sequential and deterministic.
/// This design decision favors simplicity and predictability over parallelism.
/// In cases where no actual concurrent mutation is required, a standard HashMap might suffice.
pub struct TypeScope {
    scopes: Vec<TypeScopeLayer>,
}

/// Single layer in the type scope stack
#[derive(Clone)]
pub struct TypeScopeLayer {
    pub types: HashMap<String, TypeInfo>,
}

impl Default for TypeScopeLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScopeLayer {
    fn new() -> Self {
        Self {
            types: HashMap::new(),
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
    pub fn insert_type(&mut self, name: String, ty: TypeInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.types.insert(name, ty);
        }
    }

    /// Remove a type from the current scope
    pub fn remove_type(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.types.remove(name);
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

    /// Create a scope checkpoint for later restoration
    /// Returns the current scope depth as a checkpoint
    pub fn create_checkpoint(&self) -> usize {
        self.scopes.len()
    }

    /// Restore a scope checkpoint
    /// Truncates the scope stack to the checkpoint depth
    pub fn restore_checkpoint(&mut self, checkpoint: usize) {
        if checkpoint <= self.scopes.len() && checkpoint > 0 {
            self.scopes.truncate(checkpoint);
        }
    }

    /// Enter an isolated scope
    /// This is a semantic alias for enter_scope() to make code more readable
    /// when the intention is to create an isolated context
    pub fn enter_isolated_scope(&mut self) {
        self.enter_scope();
    }

    /// Exit an isolated scope and clean up
    /// This is a semantic alias for exit_scope() to make code more readable
    /// when the intention is to clean up an isolated context
    pub fn exit_isolated_scope(&mut self) {
        self.exit_scope();
    }

    /// Get a type from the current scope only (not parent scopes)
    /// This is useful for checking if a type exists in the current scope
    /// without considering parent scopes
    pub fn get_type_from_current_scope(&self, name: &str) -> Option<TypeInfo> {
        if let Some(scope) = self.scopes.last() {
            scope.types.get(name).cloned()
        } else {
            None
        }
    }

    /// Get the scope at a specific level
    /// Level 0 is the global scope, higher levels are nested scopes
    /// Returns None if the level is out of bounds
    pub fn get_scope_at_level(&self, level: usize) -> Option<&TypeScopeLayer> {
        if level < self.scopes.len() {
            Some(&self.scopes[level])
        } else {
            None
        }
    }

    /// Insert a type at a specific scope level
    /// Level 0 is the global scope, higher levels are nested scopes
    /// Does nothing if the level is out of bounds
    pub fn insert_type_at_level(&mut self, level: usize, name: String, ty: TypeInfo) {
        if level < self.scopes.len() {
            self.scopes[level].types.insert(name, ty);
        }
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
