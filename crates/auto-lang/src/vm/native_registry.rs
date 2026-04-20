//! AutoVM Native Function Registry
//!
//! Runtime registry for mapping function names (like "List.new", "List.len")
//! to native function IDs used by CALL_NAT opcode.
//!
//! This is the AutoVM equivalent of the linker's symbol table:
//! - Function names are "symbols" (like "printf" in C)
//! - Native IDs are "addresses" (like 0x12345678 in machine code)
//!
//! # Example
//!
//! ```rust,no_run
//! use auto_lang::vm::native_registry::BIGVM_NATIVES;
//!
//! // Register native functions during compilation
//! let id = BIGVM_NATIVES.lock().unwrap().register("List.new");
//! assert!(id >= 100); // IDs start at 100
//!
//! // Look up native ID during codegen
//! if let Some(native_id) = BIGVM_NATIVES.lock().unwrap().get_id("List.new") {
//!     // Emit CALL_NAT with native_id
//! }
//! ```
use std::collections::HashMap;
use std::sync::Mutex;

/// Lightweight return type for native functions (Send + Sync safe).
/// Codegen converts these to full `Type` values during initialization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NativeRetType {
    Void,
    Int,
    Float,
    Bool,
    String,
    I64,
    List,
}

pub struct AutoVMNativeRegistry {
    // Maps function name ("List.new") -> native ID (100, 101, ...)
    registry: HashMap<String, u16>,
    // Maps function name -> return type (for codegen type inference)
    return_types: HashMap<String, NativeRetType>,
    next_id: u16,
}

impl AutoVMNativeRegistry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            return_types: HashMap::new(),
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

    /// Register a native function with a specific ID and return type.
    pub fn register_with_id_and_type(&mut self, name: &str, id: u16, ret_type: NativeRetType) {
        self.registry.insert(name.to_string(), id);
        self.return_types.insert(name.to_string(), ret_type);
        if id >= self.next_id {
            self.next_id = id + 1;
        }
    }

    /// Get the return type for a native function.
    pub fn get_return_type(&self, name: &str) -> Option<NativeRetType> {
        self.return_types.get(name).copied()
    }

    /// Get all return types (for bulk import by codegen).
    pub fn get_all_return_types(&self) -> &HashMap<String, NativeRetType> {
        &self.return_types
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

    // List functions (IDs 100-110 aligned with NATIVE_LIST_* in native.rs)
    registry.register_with_id("List.new", 100);
    registry.register_with_id("List.push", 101);
    registry.register_with_id("List.pop", 102);
    registry.register_with_id("List.len", 103);
    registry.register_with_id("List.is_empty", 104);
    registry.register_with_id("List.clear", 105);
    registry.register_with_id("List.get", 106);
    registry.register_with_id("List.set", 107);
    registry.register_with_id("List.insert", 108);
    registry.register_with_id("List.remove", 109);
    registry.register_with_id("List.drop", 110);
    registry.register_with_id("List.reserve", 118);  // No hardcoded shim, but reserved ID
    registry.register_with_id("List.capacity", 205);

    // List monomorphic aliases (Plan 194 Task 6)
    // All current List natives operate on ListData<i32>, so all type-suffixed
    // aliases route to the same native. When string/float/bool List shims are
    // added later, these can be redirected to type-specific natives.
    registry.register_with_id("List.push_int", 101);    // reuse List.push
    registry.register_with_id("List.push_uint", 101);
    registry.register_with_id("List.push_float", 101);
    registry.register_with_id("List.push_bool", 101);
    registry.register_with_id("List.push_str", 101);
    registry.register_with_id("List.pop_int", 102);     // reuse List.pop
    registry.register_with_id("List.pop_uint", 102);
    registry.register_with_id("List.pop_float", 102);
    registry.register_with_id("List.pop_bool", 102);
    registry.register_with_id("List.pop_str", 102);
    registry.register_with_id("List.get_int", 106);     // reuse List.get
    registry.register_with_id("List.get_uint", 106);
    registry.register_with_id("List.get_float", 106);
    registry.register_with_id("List.get_bool", 106);
    registry.register_with_id("List.get_str", 106);
    registry.register_with_id("List.set_int", 107);     // reuse List.set
    registry.register_with_id("List.set_uint", 107);
    registry.register_with_id("List.set_float", 107);
    registry.register_with_id("List.set_bool", 107);
    registry.register_with_id("List.set_str", 107);
    registry.register_with_id("List.insert_int", 108);  // reuse List.insert
    registry.register_with_id("List.insert_uint", 108);
    registry.register_with_id("List.insert_float", 108);
    registry.register_with_id("List.insert_bool", 108);
    registry.register_with_id("List.insert_str", 108);
    registry.register_with_id("List.remove_int", 109);  // reuse List.remove
    registry.register_with_id("List.remove_uint", 109);
    registry.register_with_id("List.remove_float", 109);
    registry.register_with_id("List.remove_bool", 109);
    registry.register_with_id("List.remove_str", 109);

    // Memory allocation functions (Plan 052 Phase 2)
    registry.register_with_id("alloc_array", 190);
    registry.register_with_id("realloc_array", 191);
    registry.register_with_id("free_array", 192);

    // Heap storage functions (Plan 052)
    registry.register_with_id("Heap.new", 195);
    registry.register_with_id("Heap.capacity", 196);
    registry.register_with_id("Heap.try_grow", 197);
    registry.register_with_id("Heap.drop", 198);

    // InlineInt64 storage functions
    registry.register_with_id("InlineInt64.new", 199);
    registry.register_with_id("InlineInt64.capacity", 200);
    registry.register_with_id("InlineInt64.try_grow", 201);
    registry.register_with_id("InlineInt64.drop", 202);

    // Instance method aliases (lowercase receiver names used by codegen)
    registry.register_with_id("heap.new", 195);
    registry.register_with_id("heap.capacity", 196);
    registry.register_with_id("heap.try_grow", 197);
    registry.register_with_id("heap.drop", 198);
    registry.register_with_id("InlineInt64.new", 199);
    registry.register_with_id("InlineInt64.capacity", 200);
    registry.register_with_id("InlineInt64.try_grow", 201);
    registry.register_with_id("InlineInt64.drop", 202);

    // Iterator functions (IDs 111-117 aligned with NATIVE_LIST_ITER + NATIVE_ITERATOR_*)
    registry.register_with_id("List.iter", 111);
    registry.register_with_id("Iterator.next", 112);
    registry.register_with_id("Iterator.map", 113);
    registry.register_with_id("Iterator.filter", 114);
    registry.register_with_id("Iterator.collect", 115);
    registry.register_with_id("Iterator.reduce", 116);
    registry.register_with_id("Iterator.find", 117);

    // HashMap functions (IDs 119-128 aligned with NATIVE_HASHMAP_* in native.rs)
    registry.register_with_id("HashMap.new", 119);
    registry.register_with_id("HashMap.insert_str", 120);
    registry.register_with_id("HashMap.insert_int", 121);
    registry.register_with_id("HashMap.get_str", 122);
    registry.register_with_id("HashMap.get_int", 123);
    registry.register_with_id("HashMap.contains", 124);
    registry.register_with_id("HashMap.remove", 125);
    registry.register_with_id("HashMap.size", 126);
    registry.register_with_id("HashMap.clear", 127);
    registry.register_with_id("HashMap.drop", 128);

    // HashMap unified generic methods (Plan 194 Task 4)
    registry.register_with_id("HashMap.insert", 120);  // reuse insert_str
    registry.register_with_id("HashMap.get", 122);     // reuse get_str

    // HashMap monomorphic aliases (Plan 194 Task 2)
    // float/bool reuse the int native (float stored as int bits, bool as 0/1)
    registry.register_with_id("HashMap.insert_float", 121);  // reuse insert_int
    registry.register_with_id("HashMap.insert_bool", 121);   // reuse insert_int
    registry.register_with_id("HashMap.get_float", 123);     // reuse get_int
    registry.register_with_id("HashMap.get_bool", 123);      // reuse get_int
    registry.register_with_id("HashMap.contains_str", 124);  // reuse contains
    registry.register_with_id("HashMap.contains_int", 124);  // reuse contains
    registry.register_with_id("HashMap.contains_float", 124); // reuse contains
    registry.register_with_id("HashMap.contains_bool", 124);  // reuse contains
    registry.register_with_id("HashMap.remove_str", 125);    // reuse remove
    registry.register_with_id("HashMap.remove_int", 125);    // reuse remove
    registry.register_with_id("HashMap.remove_float", 125);  // reuse remove
    registry.register_with_id("HashMap.remove_bool", 125);   // reuse remove

    // HashSet functions (129-135)
    registry.register_with_id("HashSet.new", 129);
    registry.register_with_id("HashSet.insert", 130);
    registry.register_with_id("HashSet.contains", 131);
    registry.register_with_id("HashSet.remove", 132);
    registry.register_with_id("HashSet.size", 133);
    registry.register_with_id("HashSet.clear", 134);
    registry.register_with_id("HashSet.drop", 135);

    // HashSet monomorphic aliases (Plan 194 Task 2)
    // str/int/float/bool type-suffixed names all map to the same native
    registry.register_with_id("HashSet.insert_str", 130);
    registry.register_with_id("HashSet.insert_int", 130);
    registry.register_with_id("HashSet.insert_float", 130);  // reuse int
    registry.register_with_id("HashSet.insert_bool", 130);   // reuse int
    registry.register_with_id("HashSet.contains_str", 131);
    registry.register_with_id("HashSet.contains_int", 131);
    registry.register_with_id("HashSet.contains_float", 131); // reuse int
    registry.register_with_id("HashSet.contains_bool", 131);  // reuse int
    registry.register_with_id("HashSet.remove_str", 132);
    registry.register_with_id("HashSet.remove_int", 132);
    registry.register_with_id("HashSet.remove_float", 132);   // reuse int
    registry.register_with_id("HashSet.remove_bool", 132);    // reuse int

    // VecDeque functions (Plan 085) - 136-146
    registry.register_with_id("VecDeque.new", 136);
    registry.register_with_id("VecDeque.push_back", 137);
    registry.register_with_id("VecDeque.push_front", 138);
    registry.register_with_id("VecDeque.pop_back", 139);
    registry.register_with_id("VecDeque.pop_front", 140);
    registry.register_with_id("VecDeque.front", 141);
    registry.register_with_id("VecDeque.back", 142);
    registry.register_with_id("VecDeque.size", 143);
    registry.register_with_id("VecDeque.is_empty", 144);
    registry.register_with_id("VecDeque.clear", 145);
    registry.register_with_id("VecDeque.drop", 146);

    // BTreeMap functions (Plan 085) - 147-157
    registry.register_with_id("BTreeMap.new", 147);
    registry.register_with_id("BTreeMap.insert", 148);
    registry.register_with_id("BTreeMap.get", 149);
    registry.register_with_id("BTreeMap.contains", 150);
    registry.register_with_id("BTreeMap.remove", 151);
    registry.register_with_id("BTreeMap.size", 152);
    registry.register_with_id("BTreeMap.is_empty", 153);
    registry.register_with_id("BTreeMap.clear", 154);
    registry.register_with_id("BTreeMap.first_key", 155);
    registry.register_with_id("BTreeMap.last_key", 156);
    registry.register_with_id("BTreeMap.drop", 157);

    // StringBuilder functions - 160-167
    registry.register_with_id("StringBuilder.new", 160);
    registry.register_with_id("StringBuilder.append", 161);
    registry.register_with_id("StringBuilder.append_int", 162);
    registry.register_with_id("StringBuilder.append_char", 163);
    registry.register_with_id("StringBuilder.len", 164);
    registry.register_with_id("StringBuilder.clear", 165);
    registry.register_with_id("StringBuilder.drop", 166);
    registry.register_with_id("StringBuilder.build", 167);

    // String functions (for string method calls like "hello".len())
    // Use explicit IDs to match NATIVE_* constants in native.rs
    registry.register_with_id("str.len", 170);    // NATIVE_STR_LEN
    registry.register_with_id("String.len", 171);  // NATIVE_STRING_LEN
    registry.register_with_id("str_new", 172);    // NATIVE_STR_NEW - Plan 118 Phase 4
    registry.register_with_id("str_append", 173); // NATIVE_STR_APPEND - Plan 118 Phase 4
    registry.register_with_id("int.str", 174);    // NATIVE_INT_STR - Plan 118 Phase 4
    registry.register_with_id("str.upper", 175);  // NATIVE_STR_UPPER - Plan 118 Phase 4
    registry.register_with_id("String.from", 176);  // NATIVE_STRING_FROM - Plan 155

    // Mutable String functions (177-186)
    registry.register_with_id("String.new", 177);
    registry.register_with_id("String.push", 178);
    registry.register_with_id("String.pop", 179);
    registry.register_with_id("String.get", 180);
    registry.register_with_id("String.set", 181);
    registry.register_with_id("String.insert", 182);
    registry.register_with_id("String.remove", 183);
    registry.register_with_id("String.clear", 184);
    registry.register_with_id("String.is_empty", 185);
    registry.register_with_id("String.reserve", 186);

    // Plan 178: Bit operation methods on int
    registry.register_with_id("int.and", 210);
    registry.register_with_id("int.or", 211);
    registry.register_with_id("int.xor", 212);
    registry.register_with_id("int.not", 213);
    registry.register_with_id("int.shl", 214);
    registry.register_with_id("int.shr", 215);
    registry.register_with_id("int.sar", 216);
    registry.register_with_id("int.rol", 217);
    registry.register_with_id("int.ror", 218);
    registry.register_with_id("int.count_ones", 220);
    registry.register_with_id("int.leading_zeros", 221);
    registry.register_with_id("int.trailing_zeros", 222);
    registry.register_with_id("int.flip", 223);

    // Phase 4: Dynamic bitfield views
    registry.register_with_id("int.bit_read", 230);
    registry.register_with_id("int.bit_test", 231);
    registry.register_with_id("int.bit_on", 232);
    registry.register_with_id("int.bit_off", 233);
    registry.register_with_id("int.bit_flip", 234);

    // String/Uint extension functions
    registry.register_with_id("str.bytes", 235);    // str.bytes() → iterator
    registry.register_with_id("uint.to_hex", 236); // uint.to_hex(pad) → hex string

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

    // File function aliases (codegen uses File.method names)
    registry.register_with_id("File.read_text", 1000);
    registry.register_with_id("File.write_text", 1001);
    registry.register_with_id("File.exists", 1002);
    registry.register_with_id("File.delete", 1003);
    registry.register_with_id("File.create_dir", 1004);
    registry.register_with_id("File.read_bytes", 1005);
    registry.register_with_id("File.write_bytes", 1006);
    registry.register_with_id("File.copy", 1007);
    registry.register_with_id("File.size", 1008);
    registry.register_with_id("File.is_dir", 1009);

    // Env functions (1100-1102)
    registry.register_with_id("auto.env.get", 1100);
    registry.register_with_id("auto.env.set", 1101);
    registry.register_with_id("auto.env.remove", 1102);

    // Env function aliases
    registry.register_with_id("Env.get", 1100);
    registry.register_with_id("Env.set", 1101);
    registry.register_with_id("Env.remove", 1102);

    // Time functions (1200-1202)
    registry.register_with_id_and_type("auto.time.now_ms", 1200, NativeRetType::I64);
    registry.register_with_id_and_type("auto.time.now_sec", 1201, NativeRetType::I64);
    registry.register_with_id_and_type("auto.time.sleep_ms", 1202, NativeRetType::Void);
    registry.register_with_id("sleep", 1202); // Alias for auto.time.sleep_ms

    // Time function aliases
    registry.register_with_id("Time.now_ms", 1200);
    registry.register_with_id("Time.now_sec", 1201);
    registry.register_with_id("Time.sleep_ms", 1202);

    // Process functions (1300-1304)
    registry.register_with_id("auto.process.exit", 1300);
    registry.register_with_id("auto.process.args", 1301);
    registry.register_with_id("auto.process.current_dir", 1302);
    registry.register_with_id("auto.process.set_current_dir", 1303);
    registry.register_with_id("auto.process.spawn", 1304);

    // Process function aliases
    registry.register_with_id("Process.exit", 1300);
    registry.register_with_id("Process.args", 1301);
    registry.register_with_id("Process.current_dir", 1302);
    registry.register_with_id("Process.set_current_dir", 1303);
    registry.register_with_id("Process.spawn", 1304);

    // Path functions (1400-1404)
    registry.register_with_id("auto.path.join", 1400);
    registry.register_with_id("auto.path.parent", 1401);
    registry.register_with_id("auto.path.extension", 1402);
    registry.register_with_id("auto.path.filename", 1403);
    registry.register_with_id("auto.path.canonicalize", 1404);

    // Path function aliases
    registry.register_with_id("Path.parent", 1401);
    registry.register_with_id("Path.extension", 1402);
    registry.register_with_id("Path.filename", 1403);

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
    registry.register_with_id("auto.str.replace", 1510);
    registry.register_with_id("auto.str.to_upper", 1511);
    registry.register_with_id("auto.str.to_lower", 1512);
    registry.register_with_id("auto.str.reverse", 1513);
    registry.register_with_id("auto.str.find", 1514);
    registry.register_with_id("auto.str.lines", 1515);
    registry.register_with_id("auto.str.parse_int", 1516);
    registry.register_with_id("auto.str.parse_float", 1517);

    // String function aliases (codegen uses Str.method names)
    registry.register_with_id("Str.len", 1500);
    registry.register_with_id("Str.is_empty", 1501);
    registry.register_with_id("Str.char_at", 1502);
    registry.register_with_id("Str.substr", 1503);
    registry.register_with_id("Str.contains", 1504);
    registry.register_with_id("Str.starts_with", 1505);
    registry.register_with_id("Str.ends_with", 1506);
    registry.register_with_id("Str.trim", 1507);
    registry.register_with_id("Str.split", 1508);
    registry.register_with_id("Str.repeat", 1509);
    registry.register_with_id("Str.replace", 1510);
    registry.register_with_id("Str.to_upper", 1511);
    registry.register_with_id("Str.to_lower", 1512);
    registry.register_with_id("Str.reverse", 1513);
    registry.register_with_id("Str.find", 1514);
    registry.register_with_id("Str.lines", 1515);
    registry.register_with_id("Str.parse_int", 1516);
    registry.register_with_id("Str.parse_float", 1517);

    // String function aliases matching str.at method names
    registry.register_with_id("auto.str.upper", 1511);  // alias for to_upper
    registry.register_with_id("auto.str.lower", 1512);  // alias for to_lower
    registry.register_with_id("auto.str.sub", 1503);    // alias for substr
    registry.register_with_id("auto.str.slice", 1503);  // alias for substr (1-arg and 2-arg forms)
    registry.register_with_id("Str.slice", 1503);        // alias for substr (1-arg and 2-arg forms)

    // String function aliases (codegen infer_type_from_var returns lowercase "str")
    // These also carry return type info for codegen type inference
    // NOTE: Use IDs from native.rs (170+), NOT FFI IDs (1500+), because FFI shims
    // are not registered in native_interface. Only override if not already registered.
    registry.register_with_id_and_type("str.len", 170, NativeRetType::Int);
    registry.register_with_id_and_type("str.is_empty", 1501, NativeRetType::Bool);
    registry.register_with_id_and_type("str.char_at", 1502, NativeRetType::String);
    registry.register_with_id_and_type("str.substr", 1503, NativeRetType::String);
    registry.register_with_id_and_type("str.contains", 1504, NativeRetType::Bool);
    registry.register_with_id_and_type("str.starts_with", 1505, NativeRetType::Bool);
    registry.register_with_id_and_type("str.ends_with", 1506, NativeRetType::Bool);
    registry.register_with_id_and_type("str.trim", 1507, NativeRetType::String);
    registry.register_with_id_and_type("str.split", 1508, NativeRetType::String);
    registry.register_with_id_and_type("str.repeat", 1509, NativeRetType::String);
    registry.register_with_id_and_type("str.replace", 1510, NativeRetType::String);
    registry.register_with_id_and_type("str.to_upper", 1511, NativeRetType::String);
    registry.register_with_id_and_type("str.to_lower", 1512, NativeRetType::String);
    registry.register_with_id_and_type("str.reverse", 1513, NativeRetType::String);
    registry.register_with_id_and_type("str.find", 1514, NativeRetType::Int);
    registry.register_with_id_and_type("str.lines", 1515, NativeRetType::String);
    registry.register_with_id_and_type("str.parse_int", 1516, NativeRetType::Int);
    registry.register_with_id_and_type("str.parse_float", 1517, NativeRetType::Float);
    registry.register_with_id("str.upper", 1511);   // alias for to_upper
    registry.register_with_id("str.lower", 1512);   // alias for to_lower
    registry.register_with_id("str.sub", 1503);     // alias for substr
    registry.register_with_id_and_type("str.slice", 1503, NativeRetType::String);  // alias for substr

    // Char functions (1600-1606)
    registry.register_with_id("auto.char.is_alpha", 1600);
    registry.register_with_id("auto.char.is_digit", 1601);
    registry.register_with_id("auto.char.is_alphanum", 1602);
    registry.register_with_id("auto.char.is_whitespace", 1603);
    registry.register_with_id("auto.char.is_ident", 1604);
    registry.register_with_id("auto.char.to_lower", 1605);
    registry.register_with_id("auto.char.to_upper", 1606);

    // Char function aliases
    registry.register_with_id("Char.is_alpha", 1600);
    registry.register_with_id("Char.is_digit", 1601);
    registry.register_with_id("Char.is_alphanum", 1602);
    registry.register_with_id("Char.is_whitespace", 1603);
    registry.register_with_id("Char.to_lower", 1605);
    registry.register_with_id("Char.to_upper", 1606);

    // Math functions (1700-1703, 1710-1725)
    registry.register_with_id_and_type("auto.math.abs", 1700, NativeRetType::Int);
    registry.register_with_id_and_type("auto.math.min", 1701, NativeRetType::Int);
    registry.register_with_id_and_type("auto.math.max", 1702, NativeRetType::Int);
    registry.register_with_id_and_type("auto.math.sqrt", 1703, NativeRetType::Float);
    registry.register_with_id("auto.math.floor", 1710);
    registry.register_with_id("auto.math.ceil", 1711);
    registry.register_with_id("auto.math.round", 1712);
    registry.register_with_id("auto.math.pow", 1713);
    registry.register_with_id("auto.math.min_f", 1714);
    registry.register_with_id("auto.math.max_f", 1715);
    registry.register_with_id("auto.math.sin", 1716);
    registry.register_with_id("auto.math.cos", 1717);
    registry.register_with_id("auto.math.tan", 1718);
    registry.register_with_id("auto.math.exp", 1719);
    registry.register_with_id("auto.math.ln", 1720);
    registry.register_with_id("auto.math.log2", 1721);
    registry.register_with_id("auto.math.log10", 1722);
    registry.register_with_id("auto.math.abs_f", 1723);
    registry.register_with_id("auto.math.signum", 1724);
    registry.register_with_id("auto.math.clamp", 1725);

    // Math function aliases (codegen uses Math.method names)
    registry.register_with_id("Math.floor", 1710);
    registry.register_with_id("Math.ceil", 1711);
    registry.register_with_id("Math.round", 1712);
    registry.register_with_id("Math.pow", 1713);
    registry.register_with_id("Math.min_f", 1714);
    registry.register_with_id("Math.max_f", 1715);
    registry.register_with_id("Math.sin", 1716);
    registry.register_with_id("Math.cos", 1717);
    registry.register_with_id("Math.tan", 1718);
    registry.register_with_id("Math.exp", 1719);
    registry.register_with_id("Math.ln", 1720);
    registry.register_with_id("Math.log2", 1721);
    registry.register_with_id("Math.log10", 1722);
    registry.register_with_id("Math.abs_f", 1723);
    registry.register_with_id("Math.signum", 1724);
    registry.register_with_id("Math.clamp", 1725);

    // JSON functions (1900-1917)
    registry.register_with_id("auto.json.encode", 1900);
    registry.register_with_id("auto.json.decode", 1901);
    registry.register_with_id("auto.json.parse", 1902);
    registry.register_with_id("auto.json.prettify", 1903);
    registry.register_with_id("auto.json.minify", 1904);
    registry.register_with_id("auto.json.is_valid", 1905);
    registry.register_with_id("auto.json.get", 1906);
    registry.register_with_id("auto.json.get_at", 1907);
    registry.register_with_id("auto.json.len", 1908);
    registry.register_with_id("auto.json.type_of", 1909);
    registry.register_with_id("auto.json.as_string", 1910);
    registry.register_with_id("auto.json.as_number", 1911);
    registry.register_with_id("auto.json.as_int", 1912);
    registry.register_with_id("auto.json.as_bool", 1913);
    registry.register_with_id("auto.json.is_null", 1914);
    registry.register_with_id("auto.json.keys", 1915);
    registry.register_with_id("auto.json.has_key", 1917);

    // JSON function aliases (codegen uses Json.method names)
    registry.register_with_id("Json.encode", 1900);
    registry.register_with_id("Json.decode", 1901);
    registry.register_with_id("Json.parse", 1902);
    registry.register_with_id("Json.prettify", 1903);
    registry.register_with_id("Json.minify", 1904);
    registry.register_with_id("Json.is_valid", 1905);
    registry.register_with_id("Json.get", 1906);
    registry.register_with_id("Json.get_at", 1907);
    registry.register_with_id("Json.len", 1908);
    registry.register_with_id("Json.type_of", 1909);
    registry.register_with_id("Json.as_string", 1910);
    registry.register_with_id("Json.as_number", 1911);
    registry.register_with_id("Json.as_int", 1912);
    registry.register_with_id("Json.as_bool", 1913);
    registry.register_with_id("Json.is_null", 1914);
    registry.register_with_id("Json.keys", 1915);
    registry.register_with_id("Json.has_key", 1917);

    // URL functions (2000-2015)
    registry.register_with_id("auto.url.encode", 2000);
    registry.register_with_id("auto.url.decode", 2001);
    registry.register_with_id("auto.url.parse", 2006);
    registry.register_with_id("auto.url.scheme", 2007);
    registry.register_with_id("auto.url.host", 2008);
    registry.register_with_id("auto.url.port", 2009);
    registry.register_with_id("auto.url.path", 2010);
    registry.register_with_id("auto.url.query", 2011);
    registry.register_with_id("auto.url.fragment", 2012);

    // URL function aliases
    registry.register_with_id("Url.encode", 2000);
    registry.register_with_id("Url.decode", 2001);
    registry.register_with_id("Url.scheme", 2007);
    registry.register_with_id("Url.host", 2008);
    registry.register_with_id("Url.port", 2009);
    registry.register_with_id("Url.path", 2010);
    registry.register_with_id("Url.query", 2011);
    registry.register_with_id("Url.fragment", 2012);

    // URL function aliases (codegen uses Url.method names)
    registry.register_with_id("Url.parse", 2006);
    registry.register_with_id("Url.scheme", 2007);
    registry.register_with_id("Url.host", 2008);
    registry.register_with_id("Url.port", 2009);
    registry.register_with_id("Url.path", 2010);
    registry.register_with_id("Url.query", 2011);
    registry.register_with_id("Url.fragment", 2012);

    // Net/TCP functions (2100-2113)
    registry.register_with_id("Net.tcp_bind", 2100);
    registry.register_with_id("Net.tcp_listener_accept", 2101);
    registry.register_with_id("Net.tcp_listener_local_addr", 2102);
    registry.register_with_id("Net.tcp_listener_close", 2103);
    registry.register_with_id("Net.tcp_connect", 2104);
    registry.register_with_id("Net.tcp_stream_read", 2105);
    registry.register_with_id("Net.tcp_stream_write", 2106);
    registry.register_with_id("Net.tcp_stream_read_all", 2107);
    registry.register_with_id("Net.tcp_stream_read_line", 2108);
    registry.register_with_id("Net.tcp_stream_write_str", 2109);
    registry.register_with_id("Net.tcp_stream_close", 2110);
    registry.register_with_id("Net.tcp_stream_peer_addr", 2111);
    registry.register_with_id("Net.tcp_stream_set_read_timeout", 2112);
    registry.register_with_id("Net.tcp_stream_set_write_timeout", 2113);

    // Task/Msg functions (Plan 121) - 2300-2304
    registry.register_with_id("auto.task.spawn", 2300);
    registry.register_with_id("auto.task.send", 2301);
    registry.register_with_id("auto.task.handle_is_null", 2302);
    registry.register_with_id("auto.task.handle_type", 2303);
    registry.register_with_id("auto.task.handle_id", 2304);

    // HTTP Stream functions (Plan 152) - 2240-2250
    registry.register_with_id("auto.http_stream.get_stream", 2240);
    registry.register_with_id("auto.http_stream.post_stream", 2241);
    registry.register_with_id("auto.http_stream.stream_next", 2242);
    registry.register_with_id("auto.http_stream.stream_is_done", 2243);
    registry.register_with_id("auto.http_stream.stream_close", 2244);
    registry.register_with_id("parse_sse", 2250);

    // TaskSystem functions (Plan 127) - 2305-2307
    registry.register_with_id("auto.task_system.start", 2305);
    registry.register_with_id("auto.task_system.run", 2306);
    registry.register_with_id("auto.task_system.stop", 2307);

    // TaskSystem aliases (codegen uses TitleCase names)
    registry.register_with_id("TaskSystem.start", 2305);
    registry.register_with_id("TaskSystem.run", 2306);
    registry.register_with_id("TaskSystem.stop", 2307);

    // Task aliases (for LoggerTask.spawn(), handle.send(), MonitorTask.send())
    registry.register_with_id("Task.spawn", 2300);
    registry.register_with_id("TaskHandle.send", 2301);
    registry.register_with_id("Task.send", 2311); // For singleton tasks like MonitorTask.send() - uses NATIVE_TASK_SINGLETON_SEND

    // Plan 192: Method table for Rust stdlib dynamic dispatch
    // When use.rust imports a type, its methods are registered here pointing to NATIVE_RUST_STDLIB_DISPATCH
}

/// Known methods for each Rust stdlib type.
/// Used by resolve_uses() to auto-register methods when use.rust imports a type.
pub const RUST_STDLIB_METHODS: &[(&str, &[&str])] = &[
    ("Instant", &["now"]),
    ("Duration", &["from_secs", "from_millis", "from_secs_f64"]),
    ("PathBuf", &["from", "join"]),
    ("Arc", &["new"]),
    ("Mutex", &["new"]),
    ("Box", &["new"]),
    ("RefCell", &["new"]),
];

impl AutoVMNativeRegistry {
    /// Plan 192: Register all known methods for a Rust stdlib type in the native registry.
    /// All methods point to NATIVE_RUST_STDLIB_DISPATCH for dynamic dispatch.
    pub fn register_rust_type_methods(&mut self, type_name: &str) {
        let dispatch_id = match type_name {
            "Instant" => 3000,
            "Duration" => 3000,
            "PathBuf" => 3000,
            "Arc" => 3000,
            "Mutex" => 3000,
            "Box" => 3000,
            "RefCell" => 3000,
            _ => return,
        };
        if let Some((_, methods)) = RUST_STDLIB_METHODS.iter().find(|(name, _)| *name == type_name) {
            for method in *methods {
                let full_name = format!("{}.{}", type_name, method);
                self.register_with_id(&full_name, dispatch_id);
            }
        }
    }
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
