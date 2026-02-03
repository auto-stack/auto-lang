/// BigVM Native Function Registry
///
/// Runtime registry for mapping function names (like "List.new", "List.len")
/// to native function IDs used by CALL_NAT opcode.
///
/// This is the BigVM equivalent of the linker's symbol table:
/// - Function names are "symbols" (like "printf" in C)
/// - Native IDs are "addresses" (like 0x12345678 in machine code)
///
/// # Example
///
/// ```rust
/// // Register native functions during compilation
/// let id = BIGVM_NATIVES.lock().unwrap().register("List.new");
/// assert!(id >= 100); // IDs start at 100
///
/// // Look up native ID during codegen
/// if let Some(native_id) = BIGVM_NATIVES.lock().unwrap().get_id("List.new") {
///     // Emit CALL_NAT with native_id
/// }
/// ```

use std::collections::HashMap;
use std::sync::Mutex;

pub struct BigVMNativeRegistry {
    // Maps function name ("List.new") -> native ID (100, 101, ...)
    registry: HashMap<String, u16>,
    next_id: u16,
}

impl BigVMNativeRegistry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            // Start at 100 to avoid conflicts with existing print functions (1-3)
            // and allow room for future expansion
            next_id: 100,
        }
    }

    /// Register a native function and return its assigned ID.
    ///
    /// If the function is already registered, returns the existing ID.
    ///
    /// # Arguments
    /// * `name` - Fully qualified function name (e.g., "List.new", "HashMap.insert")
    ///
    /// # Returns
    /// The assigned native ID (>= 100)
    pub fn register(&mut self, name: &str) -> u16 {
        if let Some(&id) = self.registry.get(name) {
            return id; // Already registered
        }

        let id = self.next_id;
        self.next_id += 1;
        self.registry.insert(name.to_string(), id);
        id
    }

    /// Get the native ID for a function name.
    ///
    /// # Arguments
    /// * `name` - Fully qualified function name
    ///
    /// # Returns
    /// * `Some(id)` - Function is registered as native
    /// * `None` - Function is not a native function (user-defined)
    pub fn get_id(&self, name: &str) -> Option<u16> {
        self.registry.get(name).copied()
    }

    /// Check if a function is registered as native.
    pub fn contains(&self, name: &str) -> bool {
        self.registry.contains_key(name)
    }

    /// Get the number of registered native functions.
    pub fn len(&self) -> usize {
        self.registry.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }

    /// Get all registered function names (for debugging).
    pub fn get_function_names(&self) -> Vec<String> {
        self.registry.keys().cloned().collect()
    }
}

// Global native registry instance
lazy_static::lazy_static! {
    pub static ref BIGVM_NATIVES: Mutex<BigVMNativeRegistry> =
        Mutex::new(BigVMNativeRegistry::new());
}

/// Register all built-in native functions.
///
/// This should be called during VM initialization to register
/// all standard library functions that have native implementations.
pub fn register_builtin_natives() {
    let mut registry = BIGVM_NATIVES.lock().unwrap();

    // List functions
    registry.register("List.new");
    registry.register("List.push");
    registry.register("List.pop");
    registry.register("List.len");
    registry.register("List.is_empty");
    registry.register("List.clear");
    registry.register("List.get");
    registry.register("List.set");
    registry.register("List.drop");

    // Iterator functions
    registry.register("List.iter");
    registry.register("Iterator.next");
    registry.register("Iterator.map");
    registry.register("Iterator.filter");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_returns_id() {
        let mut registry = BigVMNativeRegistry::new();

        let id1 = registry.register("List.new");
        assert_eq!(id1, 100);

        let id2 = registry.register("List.push");
        assert_eq!(id2, 101);

        let id3 = registry.register("List.len");
        assert_eq!(id3, 102);
    }

    #[test]
    fn test_register_idempotent() {
        let mut registry = BigVMNativeRegistry::new();

        let id1 = registry.register("List.new");
        let id2 = registry.register("List.new");

        assert_eq!(id1, id2);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_get_id() {
        let mut registry = BigVMNativeRegistry::new();

        registry.register("List.new");
        assert_eq!(registry.get_id("List.new"), Some(100));
        assert_eq!(registry.get_id("List.push"), None);
    }

    #[test]
    fn test_contains() {
        let mut registry = BigVMNativeRegistry::new();

        registry.register("List.new");
        assert!(registry.contains("List.new"));
        assert!(!registry.contains("List.push"));
    }

    #[test]
    fn test_global_registry() {
        let id = BIGVM_NATIVES.lock().unwrap().register("Test.func");
        assert!(id >= 100);
        assert!(BIGVM_NATIVES.lock().unwrap().contains("Test.func"));
    }
}
