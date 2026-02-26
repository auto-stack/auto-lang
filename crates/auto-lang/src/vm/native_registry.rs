/// AutoVM Native Function Registry
///
/// Runtime registry for mapping function names (like "List.new", "List.len")
/// to native function IDs used by CALL_NAT opcode.
///
/// This is the AutoVM equivalent of the linker's symbol table:
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

pub struct AutoVMNativeRegistry {
    // Maps function name ("List.new") -> native ID (100, 101, ...)
    registry: HashMap<String, u16>,
    next_id: u16,
}

impl AutoVMNativeRegistry {
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

    /// Register a native function with a specific ID.
    ///
    /// Use this to align BIGVM_NATIVES IDs with NATIVE_* constants.
    ///
    /// # Arguments
    /// * `name` - Fully qualified function name
    /// * `id` - The specific ID to use (must match NATIVE_* constant)
    pub fn register_with_id(&mut self, name: &str, id: u16) {
        self.registry.insert(name.to_string(), id);
        // Update next_id to avoid conflicts
        if id >= self.next_id {
            self.next_id = id + 1;
        }
    }
}

// Global native registry instance
lazy_static::lazy_static! {
    pub static ref BIGVM_NATIVES: Mutex<AutoVMNativeRegistry> =
        Mutex::new(AutoVMNativeRegistry::new());
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
    registry.register("List.insert");
    registry.register("List.remove");
    registry.register("List.drop");
    registry.register("List.reserve");

    // Iterator functions
    registry.register("List.iter");
    registry.register("Iterator.next");
    registry.register("Iterator.map");
    registry.register("Iterator.filter");
    registry.register("Iterator.collect");
    registry.register("Iterator.reduce");
    registry.register("Iterator.find");

    // HashMap functions
    registry.register("HashMap.new");
    registry.register("HashMap.insert_str");
    registry.register("HashMap.insert_int");
    registry.register("HashMap.get_str");
    registry.register("HashMap.get_int");
    registry.register("HashMap.contains");
    registry.register("HashMap.remove");
    registry.register("HashMap.size");
    registry.register("HashMap.clear");
    registry.register("HashMap.drop");

    // HashSet functions
    registry.register("HashSet.new");
    registry.register("HashSet.insert");
    registry.register("HashSet.contains");
    registry.register("HashSet.remove");
    registry.register("HashSet.size");
    registry.register("HashSet.clear");
    registry.register("HashSet.drop");

    // VecDeque functions (Plan 085)
    registry.register("VecDeque.new");
    registry.register("VecDeque.push_back");
    registry.register("VecDeque.push_front");
    registry.register("VecDeque.pop_back");
    registry.register("VecDeque.pop_front");
    registry.register("VecDeque.front");
    registry.register("VecDeque.back");
    registry.register("VecDeque.size");
    registry.register("VecDeque.is_empty");
    registry.register("VecDeque.clear");
    registry.register("VecDeque.drop");

    // BTreeMap functions (Plan 085)
    registry.register("BTreeMap.new");
    registry.register("BTreeMap.insert");
    registry.register("BTreeMap.get");
    registry.register("BTreeMap.contains");
    registry.register("BTreeMap.remove");
    registry.register("BTreeMap.size");
    registry.register("BTreeMap.is_empty");
    registry.register("BTreeMap.clear");
    registry.register("BTreeMap.first_key");
    registry.register("BTreeMap.last_key");
    registry.register("BTreeMap.drop");

    // String functions (for string method calls like "hello".len())
    // Use explicit IDs to match NATIVE_* constants in native.rs
    registry.register_with_id("str.len", 132);    // NATIVE_STR_LEN
    registry.register_with_id("String.len", 133);  // NATIVE_STRING_LEN

    // =========================================================================
    // FFI Shim Registrations (Plan 094)
    // These map Auto function names to their native IDs
    // =========================================================================

    // File functions (1000-1009)
    registry.register_with_id("auto.file.read_text", 1000);
    registry.register_with_id("auto.file.write_text", 1001);
    registry.register_with_id("auto.file.exists", 1002);
    registry.register_with_id("auto.file.delete", 1003);
    registry.register_with_id("auto.file.create_dir", 1004);
    registry.register_with_id("auto.file.read_bytes", 1005);
    registry.register_with_id("auto.file.write_bytes", 1006);
    registry.register_with_id("auto.file.copy", 1007);
    registry.register_with_id("auto.file.size", 1008);
    registry.register_with_id("auto.file.is_dir", 1009);

    // Env functions (1100-1102)
    registry.register_with_id("auto.env.get", 1100);
    registry.register_with_id("auto.env.set", 1101);
    registry.register_with_id("auto.env.remove", 1102);

    // Time functions (1200-1202)
    registry.register_with_id("auto.time.now_ms", 1200);
    registry.register_with_id("auto.time.now_sec", 1201);
    registry.register_with_id("auto.time.sleep_ms", 1202);

    // Process functions (1300-1304)
    registry.register_with_id("auto.process.exit", 1300);
    registry.register_with_id("auto.process.args", 1301);
    registry.register_with_id("auto.process.current_dir", 1302);
    registry.register_with_id("auto.process.set_current_dir", 1303);
    registry.register_with_id("auto.process.spawn", 1304);

    // Path functions (1400-1404)
    registry.register_with_id("auto.path.join", 1400);
    registry.register_with_id("auto.path.parent", 1401);
    registry.register_with_id("auto.path.extension", 1402);
    registry.register_with_id("auto.path.filename", 1403);
    registry.register_with_id("auto.path.canonicalize", 1404);

    // String functions (1500-1509)
    registry.register_with_id("auto.str.len", 1500);
    registry.register_with_id("auto.str.is_empty", 1501);
    registry.register_with_id("auto.str.char_at", 1502);
    registry.register_with_id("auto.str.substr", 1503);
    registry.register_with_id("auto.str.contains", 1504);
    registry.register_with_id("auto.str.starts_with", 1505);
    registry.register_with_id("auto.str.ends_with", 1506);
    registry.register_with_id("auto.str.trim", 1507);
    registry.register_with_id("auto.str.split", 1508);
    registry.register_with_id("auto.str.repeat", 1509);

    // Char functions (1600-1606)
    registry.register_with_id("auto.char.is_alpha", 1600);
    registry.register_with_id("auto.char.is_digit", 1601);
    registry.register_with_id("auto.char.is_alphanum", 1602);
    registry.register_with_id("auto.char.is_whitespace", 1603);
    registry.register_with_id("auto.char.is_ident", 1604);
    registry.register_with_id("auto.char.to_lower", 1605);
    registry.register_with_id("auto.char.to_upper", 1606);

    // Math functions (1700-1703)
    registry.register_with_id("auto.math.abs", 1700);
    registry.register_with_id("auto.math.min", 1701);
    registry.register_with_id("auto.math.max", 1702);
    registry.register_with_id("auto.math.sqrt", 1703);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_returns_id() {
        let mut registry = AutoVMNativeRegistry::new();

        let id1 = registry.register("List.new");
        assert_eq!(id1, 100);

        let id2 = registry.register("List.push");
        assert_eq!(id2, 101);

        let id3 = registry.register("List.len");
        assert_eq!(id3, 102);
    }

    #[test]
    fn test_register_idempotent() {
        let mut registry = AutoVMNativeRegistry::new();

        let id1 = registry.register("List.new");
        let id2 = registry.register("List.new");

        assert_eq!(id1, id2);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_get_id() {
        let mut registry = AutoVMNativeRegistry::new();

        registry.register("List.new");
        assert_eq!(registry.get_id("List.new"), Some(100));
        assert_eq!(registry.get_id("List.push"), None);
    }

    #[test]
    fn test_contains() {
        let mut registry = AutoVMNativeRegistry::new();

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
