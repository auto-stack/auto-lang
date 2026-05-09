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
    Map,
}

pub struct AutoVMNativeRegistry {
    // Maps function name ("List.new", "auto.list.new") -> native ID (100, 101, ...)
    // Plan 198: Unified single registry (was split into registry + qualified_registry)
    registry: HashMap<String, u16>,
    // Maps function name -> return type (for codegen type inference)
    return_types: HashMap<String, NativeRetType>,
    next_id: u16,
}

/// Maps short type name prefixes to canonical module path.
/// Handles cases where to_lowercase() produces wrong module segment.
pub const TYPE_CANONICAL_MAP: &[(&str, &str)] = &[
    ("Array", "auto.list"),
    ("List", "auto.list"),
    ("HashMap", "auto.hashmap"),
    ("Map", "auto.hashmap"),
    ("TaskHandle", "auto.task"),
    ("TaskSystem", "auto.task_system"),
    ("Result.Ok", "auto.result"),
    ("Result.Err", "auto.result"),
    ("Result", "auto.result"),
    ("Response", "auto.http.response"),
    ("Http", "auto.http"),
    ("Option", "auto.option"),
    ("String", "auto.str"),
    ("Str", "auto.str"),
    ("File", "auto.file"),
];

impl AutoVMNativeRegistry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            return_types: HashMap::new(),
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
        self.return_types.get(name).copied().or_else(|| {
            if !name.starts_with("auto.") {
                Self::to_canonical(name).and_then(|c| self.return_types.get(&c).copied())
            } else {
                None
            }
        })
    }

    /// Get all return types (for bulk import by codegen).
    pub fn get_all_return_types(&self) -> &HashMap<String, NativeRetType> {
        &self.return_types
    }

    /// Register only the return type for a native function (without registering an ID).
    /// Used when the ID is already registered elsewhere (e.g., in qualified_registry).
    pub fn register_return_type(&mut self, name: &str, ret_type: NativeRetType) {
        self.return_types.insert(name.to_string(), ret_type);
    }

    /// Resolve a qualified name to a native ID.
    ///
    /// Lookup order:
    /// 1. Direct lookup in unified registry
    /// 2. Canonical normalization (e.g., "List.push" → "auto.list.push")
    pub fn resolve_qualified(&self, path: &str) -> Option<u16> {
        // Direct lookup in unified registry
        if let Some(id) = self.registry.get(path).copied() {
            return Some(id);
        }
        // Plan 198: normalize short name to canonical form
        // "str.len" → "auto.str.len", "List.push" → "auto.list.push"
        if !path.starts_with("auto.") && !path.starts_with("rust.") && !path.starts_with("py.") {
            if let Some(canonical) = Self::to_canonical(path) {
                return self.registry.get(&canonical).copied();
            }
        }
        None
    }

    /// Resolve a name to its ID and return the canonical name used.
    /// Returns None if not found.
    pub fn resolve_qualified_to_canonical(&self, path: &str) -> Option<String> {
        // Direct lookup — path is already canonical
        if self.registry.contains_key(path) {
            return Some(path.to_string());
        }
        // Try canonical normalization
        if !path.starts_with("auto.") && !path.starts_with("rust.") && !path.starts_with("py.") {
            if let Some(canonical) = Self::to_canonical(path) {
                if self.registry.contains_key(&canonical) {
                    return Some(canonical);
                }
            }
        }
        None
    }

    /// Convert a short native name to its canonical "auto.X.Y" form.
    ///
    /// - "str.len" → "auto.str.len"
    /// - "List.push" → "auto.list.push"
    /// - "TaskHandle.send" → "auto.task.send" (via TYPE_CANONICAL_MAP)
    /// - "TaskSystem.start" → "auto.task_system.start" (via TYPE_CANONICAL_MAP)
    fn to_canonical(name: &str) -> Option<String> {
        let (prefix, rest) = name.split_once('.')?;
        // Check explicit map first (handles composite/wrong-case type names)
        for &(short, canonical_prefix) in TYPE_CANONICAL_MAP {
            if prefix == short {
                return Some(format!("{}.{}", canonical_prefix, rest));
            }
        }
        // Default: lowercase the prefix
        let lower = prefix.to_lowercase();
        Some(format!("auto.{}.{}", lower, rest))
    }

    /// Plan 198 Phase 2: Enrich registry with return types from #[vm] declarations.
    ///
    /// For each #[vm] function in TypeStore that already has an ID registered,
    /// derive the return type from the declaration and store it. This supplements
    /// the manual `register_with_id_and_type()` calls — functions that already
    /// have return type info are not overridden.
    pub fn enrich_from_type_store(&mut self, type_store: &crate::types::TypeStore) {
        for (name, fn_decl) in type_store.all_fn_decls() {
            if fn_decl.kind != crate::ast::FnKind::VmFunction {
                continue;
            }
            let fn_name = name.to_string();
            let ret_type = match Self::type_to_native_ret(&fn_decl.ret) {
                Some(rt) => rt,
                None => continue,
            };

            if let Some(parent) = &fn_decl.parent {
                let parent_str = parent.to_string();
                // Enrich lowercase: "str.char_at"
                let lower = format!("{}.{}", parent_str.to_lowercase(), fn_name);
                self.enrich_return_type(&lower, ret_type);
                // Enrich TitleCase: "Str.char_at"
                let mut chars = parent_str.chars();
                if let Some(first) = chars.next() {
                    let titled: String = first.to_uppercase().collect::<String>() + chars.as_str();
                    let title_key = format!("{}.{}", titled, fn_name);
                    self.enrich_return_type(&title_key, ret_type);
                }
                // Enrich auto qualified: "auto.str.char_at"
                let qualified = format!("auto.{}.{}", parent_str.to_lowercase(), fn_name);
                if self.registry.contains_key(&qualified) {
                    self.return_types.insert(qualified, ret_type);
                }
            }
            // Standalone name
            self.enrich_return_type(&fn_name, ret_type);
        }
    }

    /// Add return type for an already-registered name (no-op if already has type or not registered).
    fn enrich_return_type(&mut self, name: &str, ret_type: NativeRetType) {
        if self.registry.contains_key(name) && !self.return_types.contains_key(name) {
            self.return_types.insert(name.to_string(), ret_type);
        }
    }

    /// Convert a Type to NativeRetType if possible.
    fn type_to_native_ret(ty: &crate::ast::Type) -> Option<NativeRetType> {
        use crate::ast::Type;
        match ty {
            Type::Void => Some(NativeRetType::Void),
            Type::Int => Some(NativeRetType::Int),
            Type::Float => Some(NativeRetType::Float),
            Type::Bool => Some(NativeRetType::Bool),
            Type::StrFixed(_) => Some(NativeRetType::String),
            Type::I64 => Some(NativeRetType::I64),
            _ => None,
        }
    }

    // Plan 198 Problem B: Auto-assign IDs from #[vm] declarations in stdlib .vm.at files.
    //
    // Scans all stdlib/auto/*.vm.at files, parses #[vm] function declarations,
    // and registers them with auto-assigned sequential IDs. This replaces the
    // manual auto.* registrations in register_builtin_natives().
    //
    // Called BEFORE register_builtin_natives() so hardcoded IDs for shim-bound
    // functions take precedence (register_with_id skips if already registered).
    fn register_vm_declarations(&mut self) {
        use crate::ast::{FnKind, Stmt};
        use crate::parser::Parser;

        let stdlib_dir = std::path::Path::new("stdlib/auto");
        if !stdlib_dir.exists() {
            return;
        }

        let vm_files: Vec<std::path::PathBuf> = std::fs::read_dir(stdlib_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().extension().map_or(false, |ext| ext == "at")
                            && e.file_name()
                                .to_str()
                                .map_or(false, |n| n.contains(".vm."))
                    })
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default();

        let mut sorted_files = vm_files;
        sorted_files.sort();

        for file_path in sorted_files {
            let source = match std::fs::read_to_string(&file_path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Extract module name from filename: "str.vm.at" → "str"
            let module_name = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_suffix(".vm"))
                .unwrap_or("");

            let code = match Parser::from(source.as_str()).parse() {
                Ok(ast) => ast,
                Err(_) => continue,
            };

            for stmt in &code.stmts {
                match stmt {
                    Stmt::Fn(fn_decl) if fn_decl.kind == FnKind::VmFunction => {
                        let canonical = if let Some(parent) = &fn_decl.parent {
                            format!("auto.{}.{}.{}", module_name, parent.to_string().to_lowercase(), fn_decl.name)
                        } else {
                            format!("auto.{}.{}", module_name, fn_decl.name)
                        };

                        // Only register if not already registered (hardcoded IDs take precedence)
                        if !self.registry.contains_key(&canonical)
                        {
                            let id = self.next_id;
                            self.next_id += 1;
                            self.registry.insert(canonical.clone(), id);

                            // Also register return type if derivable
                            if let Some(ret_type) = Self::type_to_native_ret(&fn_decl.ret) {
                                self.return_types.insert(canonical.clone(), ret_type);
                            }
                        }
                    }
                    Stmt::Ext(ext) => {
                        let target_lower = ext.target.to_string().to_lowercase();
                        for method in &ext.methods {
                            if method.kind == FnKind::VmFunction {
                                let canonical =
                                    format!("auto.{}.{}.{}", module_name, target_lower, method.name);

                                if !self.registry.contains_key(&canonical)
                                {
                                    let id = self.next_id;
                                    self.next_id += 1;
                                    self.registry.insert(canonical.clone(), id);

                                    if let Some(ret_type) = Self::type_to_native_ret(&method.ret) {
                                        self.return_types.insert(canonical.clone(), ret_type);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// Global native registry instance
lazy_static::lazy_static! {
    pub static ref BIGVM_NATIVES: Mutex<AutoVMNativeRegistry> =
        Mutex::new(AutoVMNativeRegistry::new());
    // Plan 198 Phase 2: Track whether enrich_from_type_store has been called
    pub static ref NATIVE_REGISTRY_ENRICHED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
}

/// Register all built-in native functions.
///
/// This should be called during VM initialization to register
/// all standard library functions that have native implementations.
pub fn register_builtin_natives() {
    let mut registry = BIGVM_NATIVES.lock().unwrap();

    // Plan 198 Problem B: Auto-scan stdlib .vm.at files for #[vm] declarations.
    // This registers functions NOT already covered by hardcoded IDs below.
    // Hardcoded IDs take precedence (shims are bound to specific IDs).
    registry.register_vm_declarations();

    // =========================================================================
    // Shim-bound canonical IDs (must match NATIVE_* constants in native.rs/stdlib.rs)
    // =========================================================================

    // List functions (IDs 100-110)
    registry.register_with_id("auto.list.new", 100);
    registry.register_with_id("auto.list.push", 101);
    registry.register_with_id("auto.list.pop", 102);
    registry.register_with_id("auto.list.len", 103);
    registry.register_with_id("auto.list.is_empty", 104);
    registry.register_with_id("auto.list.clear", 105);
    registry.register_with_id("auto.list.get", 106);
    registry.register_with_id("auto.list.set", 107);
    registry.register_with_id("auto.list.insert", 108);
    registry.register_with_id("auto.list.remove", 109);
    registry.register_with_id("auto.list.drop", 110);
    registry.register_with_id("auto.list.reserve", 118);
    registry.register_with_id("auto.list.capacity", 205);

    // List higher-order functions (Plan 206)
    registry.register_with_id_and_type("auto.list.map", 2060, NativeRetType::List);
    registry.register_with_id_and_type("auto.list.filter", 2061, NativeRetType::List);
    registry.register_with_id_and_type("auto.list.for_each", 2062, NativeRetType::Void);
    registry.register_with_id_and_type("auto.list.find", 2063, NativeRetType::Void);
    registry.register_with_id_and_type("auto.list.any", 2064, NativeRetType::Bool);
    registry.register_with_id_and_type("auto.list.all", 2065, NativeRetType::Bool);
    registry.register_with_id_and_type("auto.list.reduce", 2066, NativeRetType::Void);
    registry.register_with_id_and_type("auto.list.sort", 2067, NativeRetType::Void);
    registry.register_with_id_and_type("auto.list.sort_by", 2068, NativeRetType::Void);
    registry.register_with_id_and_type("auto.list.contains", 2069, NativeRetType::Bool);

    // Iterator functions (IDs 111-117)
    registry.register_with_id("auto.list.iter", 111);
    registry.register_with_id("auto.iterator.next", 112);
    registry.register_with_id("auto.iterator.map", 113);
    registry.register_with_id("auto.iterator.filter", 114);
    registry.register_with_id_and_type("auto.iterator.collect", 115, NativeRetType::List);
    registry.register_with_id("auto.iterator.reduce", 116);
    registry.register_with_id("auto.iterator.find", 117);
    registry.register_with_id("auto.iterator.enumerate", 118);

    // HashMap functions (IDs 119-128)
    registry.register_with_id("auto.hashmap.new", 119);
    registry.register_with_id_and_type("Map.new", 119, NativeRetType::Map); // Alias for Auto syntax
    registry.register_with_id("HashMap.new", 119); // Alias for Auto syntax
    registry.register_with_id("auto.hashmap.insert_str", 120);
    registry.register_with_id("auto.hashmap.insert_int", 121);
    registry.register_with_id("auto.hashmap.get_str", 122);
    registry.register_with_id("auto.hashmap.get_int", 123);
    registry.register_with_id("auto.hashmap.contains", 124);
    registry.register_with_id("auto.hashmap.remove", 125);
    registry.register_with_id("auto.hashmap.size", 126);
    registry.register_with_id("auto.hashmap.clear", 127);
    registry.register_with_id("auto.hashmap.drop", 128);
    // Unified generic methods
    registry.register_with_id("auto.hashmap.insert", 120);
    registry.register_with_id("auto.hashmap.get", 122);

    // HashSet functions (129-135)
    registry.register_with_id("auto.hashset.new", 129);
    registry.register_with_id("auto.hashset.insert", 130);
    registry.register_with_id("auto.hashset.contains", 131);
    registry.register_with_id("auto.hashset.remove", 132);
    registry.register_with_id("auto.hashset.size", 133);
    registry.register_with_id("auto.hashset.clear", 134);
    registry.register_with_id("auto.hashset.drop", 135);

    // VecDeque functions (136-146)
    registry.register_with_id("auto.vecdeque.new", 136);
    registry.register_with_id("auto.vecdeque.push_back", 137);
    registry.register_with_id("auto.vecdeque.push_front", 138);
    registry.register_with_id("auto.vecdeque.pop_back", 139);
    registry.register_with_id("auto.vecdeque.pop_front", 140);
    registry.register_with_id("auto.vecdeque.front", 141);
    registry.register_with_id("auto.vecdeque.back", 142);
    registry.register_with_id("auto.vecdeque.size", 143);
    registry.register_with_id("auto.vecdeque.is_empty", 144);
    registry.register_with_id("auto.vecdeque.clear", 145);
    registry.register_with_id("auto.vecdeque.drop", 146);

    // BTreeMap functions (147-157)
    registry.register_with_id("auto.btreemap.new", 147);
    registry.register_with_id("auto.btreemap.insert", 148);
    registry.register_with_id("auto.btreemap.get", 149);
    registry.register_with_id("auto.btreemap.contains", 150);
    registry.register_with_id("auto.btreemap.remove", 151);
    registry.register_with_id("auto.btreemap.size", 152);
    registry.register_with_id("auto.btreemap.is_empty", 153);
    registry.register_with_id("auto.btreemap.clear", 154);
    registry.register_with_id("auto.btreemap.first_key", 155);
    registry.register_with_id("auto.btreemap.last_key", 156);
    registry.register_with_id("auto.btreemap.drop", 157);

    // StringBuilder functions (160-167)
    registry.register_with_id("auto.stringbuilder.new", 160);
    registry.register_with_id("auto.stringbuilder.append", 161);
    registry.register_with_id("auto.stringbuilder.append_int", 162);
    registry.register_with_id("auto.stringbuilder.append_char", 163);
    registry.register_with_id("auto.stringbuilder.len", 164);
    registry.register_with_id("auto.stringbuilder.clear", 165);
    registry.register_with_id("auto.stringbuilder.drop", 166);
    registry.register_with_id("auto.stringbuilder.build", 167);

    // Heap / storage functions (Plan 052)
    registry.register_with_id("auto.heap.new", 195);
    registry.register_with_id("auto.heap.capacity", 196);
    registry.register_with_id("auto.heap.try_grow", 197);
    registry.register_with_id("auto.heap.drop", 198);
    registry.register_with_id("auto.inline_int64.new", 199);
    registry.register_with_id("auto.inline_int64.capacity", 200);
    registry.register_with_id("auto.inline_int64.try_grow", 201);
    registry.register_with_id("auto.inline_int64.drop", 202);

    // Memory allocation (Plan 052 Phase 2)
    registry.register_with_id("auto.alloc.array", 190);
    registry.register_with_id("auto.realloc.array", 191);
    registry.register_with_id("auto.free.array", 192);

    // String operations (VM-level, IDs 170-186)
    registry.register_with_id("auto.str.len", 1500);
    registry.register_with_id("auto.str.is_empty", 1501);
    registry.register_with_id("auto.str.char_at", 1502);
    registry.register_with_id("auto.str.substr", 1503);
    registry.register_with_id("auto.str.sub", 1503);
    registry.register_with_id("auto.str.slice", 1503);
    registry.register_with_id("auto.str.contains", 1504);
    registry.register_with_id("auto.str.starts_with", 1505);
    registry.register_with_id("auto.str.ends_with", 1506);
    registry.register_with_id("auto.str.trim", 1507);
    registry.register_with_id("auto.str.split", 1508);
    registry.register_with_id("auto.str.repeat", 1509);
    registry.register_with_id("auto.str.replace", 1510);
    registry.register_with_id("auto.str.to_upper", 1511);
    registry.register_with_id("auto.str.to_lower", 1512);
    registry.register_with_id("auto.str.upper", 175);
    registry.register_with_id("auto.str.lower", 1512);
    registry.register_with_id("auto.str.reverse", 1513);
    registry.register_with_id("auto.str.find", 1514);
    registry.register_with_id("auto.str.lines", 1515);
    registry.register_with_id("auto.str.parse_int", 1516);
    registry.register_with_id("auto.str.to_int", 1516);
    registry.register_with_id("auto.str.parse_float", 1517);
    registry.register_with_id("auto.str.new", 177);
    registry.register_with_id("auto.str.push", 178);
    registry.register_with_id("auto.str.pop", 179);
    registry.register_with_id("auto.str.get", 180);
    registry.register_with_id("auto.str.set", 181);
    registry.register_with_id("auto.str.insert", 182);
    registry.register_with_id("auto.str.remove", 183);
    registry.register_with_id("auto.str.clear", 184);
    registry.register_with_id("auto.str.is_empty", 1501);
    registry.register_with_id("auto.str.reserve", 186);
    registry.register_with_id("auto.str.bytes", 235);

    // Bit operations (Plan 178)
    registry.register_with_id("auto.int.and", 210);
    registry.register_with_id("auto.int.or", 211);
    registry.register_with_id("auto.int.xor", 212);
    registry.register_with_id("auto.int.not", 213);
    registry.register_with_id("auto.int.shl", 214);
    registry.register_with_id("auto.int.shr", 215);
    registry.register_with_id("auto.int.sar", 216);
    registry.register_with_id("auto.int.rol", 217);
    registry.register_with_id("auto.int.ror", 218);
    registry.register_with_id("auto.int.count_ones", 220);
    registry.register_with_id("auto.int.leading_zeros", 221);
    registry.register_with_id("auto.int.trailing_zeros", 222);
    registry.register_with_id("auto.int.flip", 223);
    registry.register_with_id("auto.int.bit_read", 230);
    registry.register_with_id("auto.int.bit_test", 231);
    registry.register_with_id("auto.int.bit_on", 232);
    registry.register_with_id("auto.int.bit_off", 233);
    registry.register_with_id("auto.int.bit_flip", 234);

    // =========================================================================
    // FFI function canonical IDs (1000+)
    // =========================================================================

    // File functions (1000-1012)
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
    registry.register_with_id("auto.file.walk", 1010);
    registry.register_with_id("auto.file.append_text", 1011);
    registry.register_with_id("auto.file.read_lines", 1012);

    // fs module aliases
    registry.register_with_id("auto.fs.read_text", 1000);
    registry.register_with_id("auto.fs.read", 1000);
    registry.register_with_id("auto.fs.write_text", 1001);
    registry.register_with_id("auto.fs.write", 1001);
    registry.register_with_id("auto.fs.append_text", 1011);
    registry.register_with_id("auto.fs.append", 1011);
    registry.register_with_id("auto.fs.exists", 1002);
    registry.register_with_id("auto.fs.delete", 1003);
    registry.register_with_id("auto.fs.create_dir", 1004);
    registry.register_with_id("auto.fs.read_bytes", 1005);
    registry.register_with_id("auto.fs.write_bytes", 1006);
    registry.register_with_id("auto.fs.copy", 1007);
    registry.register_with_id("auto.fs.size", 1008);
    registry.register_with_id("auto.fs.is_dir", 1009);

    // File I/O opaque handles (1010-1013) — Plan 240
    registry.register_with_id("auto.file.create_handle", 1010);
    registry.register_with_id("auto.file.open_handle", 1011);
    registry.register_with_id("auto.file.write_handle", 1012);
    registry.register_with_id("auto.file.try_clone", 1013);

    // Env functions (1100-1103)
    registry.register_with_id("auto.env.get", 1100);
    registry.register_with_id("auto.env.set", 1101);
    registry.register_with_id("auto.env.remove", 1102);
    registry.register_with_id("auto.env.get_or", 1103);
    registry.register_with_id("Env.get_or", 1103); // alias for #[rust_fn]

    // Time functions (1200-1205)
    registry.register_with_id("auto.time.now_ms", 1200);
    registry.register_with_id("auto.time.now_sec", 1201);
    registry.register_with_id("auto.time.sleep_ms", 1202);
    registry.register_with_id("auto.time.instant_now", 1203);
    registry.register_with_id("auto.time.instant_elapsed", 1204);

    // OnceCell functions (2850-2853)
    registry.register_with_id("auto.cell.once_new", 2850);
    registry.register_with_id("auto.cell.once_set", 2851);
    registry.register_with_id("auto.cell.once_get", 2852);

    // Process functions (1300-1304)
    registry.register_with_id("auto.process.exit", 1300);
    registry.register_with_id("auto.process.args", 1301);
    registry.register_with_id("auto.process.current_dir", 1302);
    registry.register_with_id("auto.process.set_current_dir", 1303);
    registry.register_with_id("auto.process.spawn", 1304);
    registry.register_with_id("auto.process.spawn_with_output", 1305);

    // Path functions (1400-1404)
    registry.register_with_id("auto.path.join", 1400);
    registry.register_with_id("auto.path.parent", 1401);
    registry.register_with_id("auto.path.extension", 1402);
    registry.register_with_id("auto.path.filename", 1403);
    registry.register_with_id("auto.path.canonicalize", 1404);

    // Char functions (1600-1606)
    registry.register_with_id("auto.char.is_alpha", 1600);
    registry.register_with_id("auto.char.is_digit", 1601);
    registry.register_with_id("auto.char.is_alphanum", 1602);
    registry.register_with_id("auto.char.is_whitespace", 1603);
    registry.register_with_id("auto.char.is_ident", 1604);
    registry.register_with_id("auto.char.to_lower", 1605);
    registry.register_with_id("auto.char.to_upper", 1606);

    // Log functions (1800-1804)
    registry.register_with_id("auto.log.debug", 1800);
    registry.register_with_id("auto.log.info", 1801);
    registry.register_with_id("auto.log.warn", 1802);
    registry.register_with_id("auto.log.error", 1803);
    registry.register_with_id("auto.log.noop", 1804);
    // Log.* aliases for #[rust_fn] — used by Phase 2.4 #macro routing
    registry.register_with_id("Log.debug", 1800);
    registry.register_with_id("Log.info", 1801);
    registry.register_with_id("Log.warn", 1802);
    registry.register_with_id("Log.error", 1803);

    // Math functions (1700-1725)
    registry.register_with_id("auto.math.abs", 1700);
    registry.register_with_id("auto.math.min", 1701);
    registry.register_with_id("auto.math.max", 1702);
    registry.register_with_id("auto.math.sqrt", 1750);  // Changed from 1703
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
    registry.register_with_id("auto.math.asin", 1726);
    registry.register_with_id("auto.math.acos", 1727);
    registry.register_with_id("auto.math.atan", 1728);
    registry.register_with_id("auto.math.atan2", 1729);
    registry.register_with_id("auto.math.powi", 1730);
    registry.register_with_id("auto.math.powf", 1731);
    registry.register_with_id("auto.math.to_radians", 1732);
    registry.register_with_id("auto.math.to_degrees", 1733);

    // Rand functions (1850-1854) — Plan 212 Phase 2
    registry.register_with_id("auto.rand.thread_rng", 1850);
    registry.register_with_id("auto.rng.gen_range", 1851);
    registry.register_with_id("auto.rng.gen", 1852);
    registry.register_with_id("auto.rng.drop", 1853);
    registry.register_with_id("auto.rand.random", 1854);

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

    // URL functions (2000-2015)
    registry.register_with_id("auto.url.encode", 2000);
    registry.register_with_id("auto.url.decode", 2001);
    registry.register_with_id("auto.url.encode_query", 2002);
    registry.register_with_id("auto.url.decode_query", 2003);
    registry.register_with_id("auto.url.parse", 2006);
    registry.register_with_id("auto.url.scheme", 2007);
    registry.register_with_id("auto.url.host", 2008);
    registry.register_with_id("auto.url.port", 2009);
    registry.register_with_id("auto.url.path", 2010);
    registry.register_with_id("auto.url.query", 2011);
    registry.register_with_id("auto.url.fragment", 2012);
    registry.register_with_id("auto.url.join_path", 2015);

    // Net/TCP functions (2100-2113)
    registry.register_with_id("auto.net.tcp_bind", 2100);
    registry.register_with_id("auto.net.tcp_listener_accept", 2101);
    registry.register_with_id("auto.net.tcp_listener_local_addr", 2102);
    registry.register_with_id("auto.net.tcp_listener_close", 2103);
    registry.register_with_id("auto.net.tcp_connect", 2104);
    registry.register_with_id("auto.net.tcp_stream_read", 2105);
    registry.register_with_id("auto.net.tcp_stream_write", 2106);
    registry.register_with_id("auto.net.tcp_stream_read_all", 2107);
    registry.register_with_id("auto.net.tcp_stream_read_line", 2108);
    registry.register_with_id("auto.net.tcp_stream_write_str", 2109);
    registry.register_with_id("auto.net.tcp_stream_close", 2110);
    registry.register_with_id("auto.net.tcp_stream_peer_addr", 2111);
    registry.register_with_id("auto.net.tcp_stream_set_read_timeout", 2112);
    registry.register_with_id("auto.net.tcp_stream_set_write_timeout", 2113);

    // HTTP server functions (2200-2215)
    registry.register_with_id("auto.http.server", 2200);
    registry.register_with_id("auto.http.server_get", 2201);
    registry.register_with_id("auto.http.server_post", 2202);
    registry.register_with_id("auto.http.server_put", 2203);
    registry.register_with_id("auto.http.server_delete", 2204);
    registry.register_with_id("auto.http.server_static", 2205);
    registry.register_with_id("auto.http.server_listen", 2206);
    registry.register_with_id("auto.http.response", 2210);
    registry.register_with_id("auto.http.response_status", 2211);
    registry.register_with_id("auto.http.response_header", 2212);
    registry.register_with_id("auto.http.response_text", 2213);
    registry.register_with_id("auto.http.response_html", 2214);
    registry.register_with_id("auto.http.response_bytes", 2215);

    // HTTP response access (2216-2218)
    registry.register_with_id("auto.http.response.status_code", 2216);
    registry.register_with_id("auto.http.response.header_get", 2217);
    registry.register_with_id("auto.http.response.body", 2218);

    // HTTP client helpers (2220-2224)
    registry.register_with_id("auto.http.ok", 2220);
    registry.register_with_id("auto.http.created", 2221);
    registry.register_with_id("auto.http.bad_request", 2222);
    registry.register_with_id("auto.http.not_found", 2223);
    registry.register_with_id("auto.http.internal_error", 2224);

    // HTTP client functions (2230-2239)
    registry.register_with_id("auto.http.get", 2230);
    registry.register_with_id("auto.http.post", 2231);
    registry.register_with_id("auto.http.put", 2232);
    registry.register_with_id("auto.http.delete", 2233);
    registry.register_with_id("auto.http.request", 2234);
    registry.register_with_id("auto.http.request_builder_header", 2235);
    registry.register_with_id("auto.http.request_builder_body", 2236);
    registry.register_with_id("auto.http.request_builder_timeout", 2237);
    registry.register_with_id("auto.http.request_builder_json", 2238);
    registry.register_with_id("auto.http.request_builder_send", 2239);

    // HTTP streaming (2240-2255)
    registry.register_with_id("auto.http_stream.get_stream", 2240);
    registry.register_with_id("auto.http_stream.post_stream", 2241);
    registry.register_with_id("auto.http_stream.stream_next", 2242);
    registry.register_with_id("auto.http_stream.stream_is_done", 2243);
    registry.register_with_id("auto.http_stream.stream_close", 2244);
    registry.register_with_id("auto.http.post_stream_with_headers", 2255);

    // Task/Msg functions (2300-2311)
    registry.register_with_id("auto.task.spawn", 2300);
    registry.register_with_id("auto.task.send", 2301);
    registry.register_with_id("auto.task.handle_is_null", 2302);
    registry.register_with_id("auto.task.handle_type", 2303);
    registry.register_with_id("auto.task.handle_id", 2304);
    registry.register_with_id("auto.task.send_await", 2308);
    registry.register_with_id("auto.task.ask", 2309);
    registry.register_with_id("auto.ctx.reply", 2310);
    registry.register_with_id("auto.task.singleton_send", 2311);

    // TaskSystem functions (2305-2307)
    registry.register_with_id("auto.task_system.start", 2305);
    registry.register_with_id("auto.task_system.run", 2306);
    registry.register_with_id("auto.task_system.stop", 2307);

    // Regex functions (2400-2401)
    registry.register_with_id("auto.regex.is_match", 2400);
    registry.register_with_id("auto.regex.find_all", 2401);

    // Regex opaque struct shims (2450-2459) — Plan 212 Phase 2.2
    registry.register_with_id("auto.re_opaque.new", 2450);
    registry.register_with_id("auto.re_opaque.is_match", 2451);
    registry.register_with_id("auto.re_opaque.find", 2452);
    registry.register_with_id("auto.re_opaque.find_all", 2453);
    registry.register_with_id("auto.re_opaque.replace_all", 2454);
    registry.register_with_id("auto.re_opaque.captures", 2455);
    registry.register_with_id("auto.re_opaque.drop", 2459);

    // Url opaque struct shims (2500-2509) — Plan 212 Phase 2.2
    registry.register_with_id("auto.url_opaque.parse", 2500);
    registry.register_with_id("auto.url_opaque.scheme", 2501);
    registry.register_with_id("auto.url_opaque.host_str", 2502);
    registry.register_with_id("auto.url_opaque.path", 2503);
    registry.register_with_id("auto.url_opaque.fragment", 2504);
    registry.register_with_id("auto.url_opaque.port", 2505);
    registry.register_with_id("auto.url_opaque.query_pairs", 2506);
    registry.register_with_id("auto.url_opaque.join", 2507);
    registry.register_with_id("auto.url_opaque.origin", 2508);
    registry.register_with_id("auto.url_opaque.drop", 2509);

    // Semver opaque struct shims (2600-2609) — Plan 212 Phase 2.2
    registry.register_with_id("auto.semver_opaque.parse", 2600);
    registry.register_with_id("auto.semver_opaque.major", 2601);
    registry.register_with_id("auto.semver_opaque.minor", 2602);
    registry.register_with_id("auto.semver_opaque.patch", 2603);
    registry.register_with_id("auto.semver_opaque.pre", 2604);
    registry.register_with_id("auto.semver_opaque.to_string", 2605);
    registry.register_with_id("auto.semver_opaque.cmp_gt", 2606);
    registry.register_with_id("auto.semver_opaque.drop", 2609);

    // chrono opaque struct shims (2700-2709) — Plan 212 Phase 2.3
    registry.register_with_id_and_type("auto.chrono_opaque.local_now", 2700, NativeRetType::Int);
    registry.register_with_id_and_type("auto.chrono_opaque.year", 2701, NativeRetType::Int);
    registry.register_with_id_and_type("auto.chrono_opaque.month", 2702, NativeRetType::Int);
    registry.register_with_id_and_type("auto.chrono_opaque.day", 2703, NativeRetType::Int);
    registry.register_with_id_and_type("auto.chrono_opaque.hour", 2704, NativeRetType::Int);
    registry.register_with_id_and_type("auto.chrono_opaque.minute", 2705, NativeRetType::Int);
    registry.register_with_id_and_type("auto.chrono_opaque.second", 2706, NativeRetType::Int);
    registry.register_with_id_and_type("auto.chrono_opaque.timestamp", 2707, NativeRetType::I64);
    registry.register_with_id_and_type("auto.chrono_opaque.format", 2708, NativeRetType::String);
    registry.register_with_id("auto.chrono_opaque.drop", 2709);

    // base64 pure function shims (2710-2719) — Plan 212 Phase 2.3
    registry.register_with_id_and_type("auto.base64.encode", 2710, NativeRetType::String);
    registry.register_with_id_and_type("auto.base64.decode", 2711, NativeRetType::String);

    // hex pure function shims (2720-2729) — Plan 212 Phase 2.3
    registry.register_with_id_and_type("auto.hex.encode", 2720, NativeRetType::String);
    registry.register_with_id_and_type("auto.hex.decode", 2721, NativeRetType::String);

    // sha2 opaque struct shims (2730-2739) — Plan 212 Phase 2.3
    registry.register_with_id("auto.sha2_opaque.sha256_new", 2730);
    registry.register_with_id("auto.sha2_opaque.update", 2731);
    registry.register_with_id_and_type("auto.sha2_opaque.finalize", 2732, NativeRetType::String);
    registry.register_with_id("auto.sha2_opaque.drop", 2739);

    // mime_guess pure function shim (2740-2749) — Plan 212 Phase 2.3
    registry.register_with_id_and_type("auto.mime.from_path", 2740, NativeRetType::String);

    // Rust stdlib dispatch (3000)
    registry.register_with_id("auto.rust_stdlib.dispatch", 3000);

    // =========================================================================
    // Bare names and non-canonicalizable aliases
    // (to_canonical() cannot resolve these — no dot or multi-segment)
    // =========================================================================

    // Bare function names (no canonical equivalent — used by internal shims)
    registry.register_with_id("sleep", 1202);
    registry.register_with_id("parse_sse", 2250);
    registry.register_with_id("str_new", 172);
    registry.register_with_id("str_append", 173);
    registry.register_with_id("int.str", 174);
    registry.register_with_id("uint.to_hex", 236);
    registry.register_with_id("alloc_array", 190);
    registry.register_with_id("realloc_array", 191);
    registry.register_with_id("free_array", 192);

    // ID-conflicting short names (different ID from canonical, used by legacy shims)
    registry.register_with_id("str.len", 170);
    registry.register_with_id("String.len", 171);
    registry.register_with_id("str.upper", 175);
    registry.register_with_id("String.from", 176);
    registry.register_with_id("String.is_empty", 185);

    // FFI shim name aliases (#[rust_fn] uses these names — needed by build_from_inventory)
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
    registry.register_with_id("File.append_text", 1011);

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
    registry.register_with_id("Str.split_once", 1518);
    registry.register_with_id("Str.match_count", 1519);
    registry.register_with_id("Str.replace_first", 1520);

    registry.register_with_id("Http.ok", 2220);
    registry.register_with_id("Http.created", 2221);
    registry.register_with_id("Http.bad_request", 2222);
    registry.register_with_id("Http.not_found", 2223);
    registry.register_with_id("Http.internal_error", 2224);

    registry.register_with_id("Task.spawn", 2300);
    registry.register_with_id("TaskHandle.send", 2301);
    registry.register_with_id("Task.singleton_send", 2311);
    registry.register_with_id("TaskHandle.send_await", 2308);
    registry.register_with_id("TaskHandle.ask", 2309);
    registry.register_with_id("TaskHandle.is_null", 2302);
    registry.register_with_id("TaskHandle.task_type", 2303);
    registry.register_with_id("TaskHandle.instance_id", 2304);
    registry.register_with_id("TaskSystem.start", 2305);
    registry.register_with_id("TaskSystem.stop", 2307);

    // Option functions (canonical names, resolved via to_canonical)
    registry.register_with_id("auto.option.or", 1550);
    registry.register_with_id("auto.option.unwrap_or", 1551);

    // Result functions (canonical names, resolved via to_canonical)
    registry.register_with_id("auto.result.map_err", 2070);
    registry.register_with_id("auto.result.Ok.map_err", 2070);
    registry.register_with_id("auto.result.Err.map_err", 2070);
    // =========================================================================
    // Return type annotations (for codegen type inference)
    // =========================================================================
    registry.register_return_type("auto.time.now_ms", NativeRetType::I64);
    registry.register_return_type("auto.time.now_sec", NativeRetType::I64);
    registry.register_return_type("auto.time.sleep_ms", NativeRetType::Void);
    registry.register_return_type("auto.str.len", NativeRetType::Int);
    registry.register_return_type("auto.str.is_empty", NativeRetType::Bool);
    registry.register_return_type("auto.str.char_at", NativeRetType::Int);
    registry.register_return_type("auto.str.substr", NativeRetType::String);
    registry.register_return_type("auto.str.contains", NativeRetType::Bool);
    registry.register_return_type("auto.str.starts_with", NativeRetType::Bool);
    registry.register_return_type("auto.str.ends_with", NativeRetType::Bool);
    registry.register_return_type("auto.str.trim", NativeRetType::String);
    registry.register_return_type("auto.str.split", NativeRetType::List);
    registry.register_return_type("auto.str.repeat", NativeRetType::String);
    registry.register_return_type("auto.str.replace", NativeRetType::String);
    registry.register_return_type("auto.str.to_upper", NativeRetType::String);
    registry.register_return_type("auto.str.to_lower", NativeRetType::String);
    registry.register_return_type("auto.str.reverse", NativeRetType::String);
    registry.register_return_type("auto.str.find", NativeRetType::Int);
    registry.register_return_type("auto.str.lines", NativeRetType::List);
    registry.register_return_type("auto.str.parse_int", NativeRetType::Int);
    registry.register_return_type("auto.str.parse_float", NativeRetType::Float);
    registry.register_return_type("auto.math.abs", NativeRetType::Int);
    registry.register_return_type("auto.math.min", NativeRetType::Int);
    registry.register_return_type("auto.math.max", NativeRetType::Int);
    // auto.math.sqrt returns f64 — type inferred by infer_native_return_type as Double
    registry.register_with_id_and_type("str.split_once", 1518, NativeRetType::List);
    registry.register_with_id_and_type("str.match_count", 1519, NativeRetType::Int);
    registry.register_with_id_and_type("str.replace_first", 1520, NativeRetType::String);
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

    #[test]
    fn test_to_canonical_default() {
        assert_eq!(AutoVMNativeRegistry::to_canonical("List.push"), Some("auto.list.push".to_string()));
        assert_eq!(AutoVMNativeRegistry::to_canonical("str.len"), Some("auto.str.len".to_string()));
        assert_eq!(AutoVMNativeRegistry::to_canonical("File.read_text"), Some("auto.file.read_text".to_string()));
    }

    #[test]
    fn test_to_canonical_mapped() {
        assert_eq!(AutoVMNativeRegistry::to_canonical("TaskHandle.send"), Some("auto.task.send".to_string()));
        assert_eq!(AutoVMNativeRegistry::to_canonical("TaskSystem.start"), Some("auto.task_system.start".to_string()));
        assert_eq!(AutoVMNativeRegistry::to_canonical("Response.status_code"), Some("auto.http.response.status_code".to_string()));
        assert_eq!(AutoVMNativeRegistry::to_canonical("Result.map_err"), Some("auto.result.map_err".to_string()));
        assert_eq!(AutoVMNativeRegistry::to_canonical("Http.get"), Some("auto.http.get".to_string()));
        assert_eq!(AutoVMNativeRegistry::to_canonical("Option.or"), Some("auto.option.or".to_string()));
        // Plan 202: List/HashMap/Map canonical mapping
        assert_eq!(AutoVMNativeRegistry::to_canonical("List.push"), Some("auto.list.push".to_string()));
        assert_eq!(AutoVMNativeRegistry::to_canonical("List.join"), Some("auto.list.join".to_string()));
        assert_eq!(AutoVMNativeRegistry::to_canonical("HashMap.insert"), Some("auto.hashmap.insert".to_string()));
        assert_eq!(AutoVMNativeRegistry::to_canonical("Map.new"), Some("auto.hashmap.new".to_string()));
    }

    #[test]
    fn test_to_canonical_bare() {
        assert_eq!(AutoVMNativeRegistry::to_canonical("sleep"), None);
        assert_eq!(AutoVMNativeRegistry::to_canonical("parse_sse"), None);
    }
}
