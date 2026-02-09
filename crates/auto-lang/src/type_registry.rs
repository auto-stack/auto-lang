/// Type Registry for REPL
///
/// This module provides a simple type registry that persists type definitions
/// across REPL inputs. It allows node instance syntax like `Point{x:1, y:2}`
/// to work in the REPL by remembering previously defined types.
///
/// **Plan 087**: This is a temporary solution for REPL type persistence.
/// Long-term, this should be integrated with Plan 064's Database.

use crate::ast::Type;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Type Registry for REPL
///
/// Stores type definitions by name, allowing parser to check if
/// an identifier is a type before parsing node instance syntax.
#[derive(Debug, Clone)]
pub struct TypeRegistry {
    /// Map from type name to Type
    types: HashMap<String, Type>,
}

impl TypeRegistry {
    /// Create a new empty type registry
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// Register a type definition
    pub fn register_type(&mut self, name: String, ty: Type) {
        self.types.insert(name, ty);
    }

    /// Check if a name is a registered type
    pub fn is_type(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }

    /// Get type by name
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    /// Clear all type definitions
    pub fn clear(&mut self) {
        self.types.clear();
    }
}

/// Shared type registry handle
pub type SharedTypeRegistry = Rc<RefCell<TypeRegistry>>;

/// Create a new shared type registry
pub fn new_type_registry() -> SharedTypeRegistry {
    Rc::new(RefCell::new(TypeRegistry::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_registry() {
        let mut registry = TypeRegistry::new();

        // Initially empty
        assert!(!registry.is_type("Point"));

        // Register a type
        registry.register_type("Point".to_string(), Type::Int);
        assert!(registry.is_type("Point"));

        // Get type back
        let ty = registry.get_type("Point");
        assert!(matches!(ty, Some(Type::Int)));

        // Clear
        registry.clear();
        assert!(!registry.is_type("Point"));
    }
}
