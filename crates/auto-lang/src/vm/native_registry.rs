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
    // Maps qualified name ("auto.list.new") -> native ID (Plan 203 Phase 1)
    qualified_registry: HashMap<String, u16>,
    // Maps function name -> return type (for codegen type inference)
    return_types: HashMap<String, NativeRetType>,
    next_id: u16,
}

impl AutoVMNativeRegistry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            qualified_registry: HashMap::new(),
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

    /// Plan 198 Phase 5: Register a native with auto-generated TitleCase alias.
    ///
    /// Given a canonical name like "auto.file.read_text", automatically generates:
    /// - "File.read_text" (TitleCase alias for codegen method resolution)
    ///
    /// This eliminates the need for manual duplicate registration of every
    /// `auto.X.Y` + `TitleX.Y` pair. The TitleCase alias is only generated
    /// if it doesn't already exist (explicit registrations take precedence).
    /// Plan 198 Phase 5: DEPRECATED — use register_with_id instead.
    /// Was used to auto-generate TitleCase alias from canonical name.
    /// Now handled by resolve_qualified() canonical normalization.
    #[allow(dead_code)]
    pub fn register_with_aliases(&mut self, canonical: &str, id: u16) {
        self.register_with_id(canonical, id);

        if let Some(rest) = canonical.strip_prefix("auto.") {
            if let Some((module, method)) = rest.split_once('.') {
                let mut chars = module.chars();
                if let Some(first) = chars.next() {
                    let titled: String = first.to_uppercase().collect::<String>() + chars.as_str();
                    let alias = format!("{}.{}", titled, method);
                    if !self.registry.contains_key(&alias) {
                        self.registry.insert(alias, id);
                    }
                }
            }
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

    // =========================================================================
    // Plan 203 Phase 1: Qualified name registry methods
    // =========================================================================

    /// Register a qualified name (e.g., "auto.list.new") pointing to an existing native ID.
    ///
    /// This does NOT create a new ID — it creates an alias from the qualified path
    /// to an already-registered native ID.
    pub fn register_qualified(&mut self, path: &str, id: u16) {
        self.qualified_registry.insert(path.to_string(), id);
    }

    /// Register a qualified name with return type info.
    pub fn register_qualified_with_type(&mut self, path: &str, id: u16, ret_type: NativeRetType) {
        self.register_qualified(path, id);
        self.return_types.insert(path.to_string(), ret_type);
    }

    /// Resolve a qualified name to a native ID.
    ///
    /// Lookup order:
    /// 1. qualified_registry (e.g., "auto.str.len")
    /// 2. short-name registry (e.g., "str.len", "List.push")
    /// 3. canonical normalization (e.g., "str.len" → "auto.str.len" → check both registries)
    pub fn resolve_qualified(&self, path: &str) -> Option<u16> {
        // Direct lookup in qualified registry
        if let Some(id) = self.qualified_registry.get(path).copied() {
            return Some(id);
        }
        // Fallback to short-name registry
        if let Some(id) = self.registry.get(path).copied() {
            return Some(id);
        }
        // Plan 198 Problem A: normalize short name to canonical form
        // "str.len" → "auto.str.len", "List.push" → "auto.list.push"
        if !path.starts_with("auto.") && !path.starts_with("rust.") && !path.starts_with("py.") {
            if let Some(canonical) = Self::to_canonical(path) {
                if let Some(id) = self.qualified_registry.get(&canonical).copied() {
                    return Some(id);
                }
                return self.registry.get(&canonical).copied();
            }
        }
        None
    }

    /// Convert a short native name to its canonical "auto.X.Y" form.
    ///
    /// - "str.len" → "auto.str.len"
    /// - "List.push" → "auto.list.push"
    /// - "File.read_text" → "auto.file.read_text"
    /// - "auto.str.len" → "auto.str.len" (already canonical)
    fn to_canonical(name: &str) -> Option<String> {
        let (prefix, rest) = name.split_once('.')?;
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
                if self.qualified_registry.contains_key(&qualified) {
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
            Type::Str(_) => Some(NativeRetType::String),
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
                            && !self.qualified_registry.contains_key(&canonical)
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
                                    && !self.qualified_registry.contains_key(&canonical)
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

    // List higher-order functions (Plan 206)
    registry.register_with_id("List.map", 2060);
    registry.register_with_id("List.filter", 2061);
    registry.register_with_id("List.for_each", 2062);
    registry.register_with_id("List.find", 2063);
    registry.register_with_id("List.any", 2064);
    registry.register_with_id("List.all", 2065);
    registry.register_with_id("List.reduce", 2066);

    // Result higher-order functions (Plan 200 Task 3.3)
    registry.register_with_id("Result.map_err", 2070);
    registry.register_with_id("Result.Ok.map_err", 2070);
    registry.register_with_id("Result.Err.map_err", 2070);

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
    // Plan 198: heap.* and InlineInt64.* duplicates removed — to_canonical() resolves

    // Iterator functions (IDs 111-117 aligned with NATIVE_LIST_ITER + NATIVE_ITERATOR_*)
    registry.register_with_id("List.iter", 111);
    registry.register_with_id("Iterator.next", 112);
    registry.register_with_id("Iterator.map", 113);
    registry.register_with_id("Iterator.filter", 114);
    registry.register_with_id("Iterator.collect", 115);
    registry.register_with_id("Iterator.reduce", 116);
    registry.register_with_id("Iterator.find", 117);
    registry.register_with_id("Iterator.enumerate", 118);

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
    // Plan 198: int.* lowercase aliases removed — to_canonical() resolves via auto.int.*

    // Phase 4: Dynamic bitfield views
    // Plan 198: int.bit_* aliases removed — to_canonical() resolves via auto.int.bit_*

    // String/Uint extension functions
    registry.register_with_id("str.bytes", 235);    // str.bytes() → iterator
    registry.register_with_id("uint.to_hex", 236); // uint.to_hex(pad) → hex string

    // =========================================================================
    // FFI Shim Registrations (Plan 094)
    // These map Auto function names to their native IDs
    // Plan 198 Phase 5: register_with_id auto-generates TitleCase aliases
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
    registry.register_with_id("File.append_text", 1011); // no auto.file prefix

    // Plan 200 Task 3.4: fs module aliases → auto.fs.* qualified entries
    // Plan 198: fs.* removed from registry; to_canonical("fs.X") → "auto.fs.X" in qualified_registry
    registry.register_qualified("auto.fs.read_text", 1000);
    registry.register_qualified("auto.fs.read", 1000);
    registry.register_qualified("auto.fs.write_text", 1001);
    registry.register_qualified("auto.fs.write", 1001);
    registry.register_qualified("auto.fs.append_text", 1011);
    registry.register_qualified("auto.fs.append", 1011);
    registry.register_qualified("auto.fs.exists", 1002);
    registry.register_qualified("auto.fs.delete", 1003);
    registry.register_qualified("auto.fs.create_dir", 1004);
    registry.register_qualified("auto.fs.read_bytes", 1005);
    registry.register_qualified("auto.fs.write_bytes", 1006);
    registry.register_qualified("auto.fs.copy", 1007);
    registry.register_qualified("auto.fs.size", 1008);
    registry.register_qualified("auto.fs.is_dir", 1009);

    // Env functions (1100-1102) — TitleCase aliases auto-generated
    registry.register_with_id("auto.env.get", 1100);
    registry.register_with_id("auto.env.set", 1101);
    registry.register_with_id("auto.env.remove", 1102);

    // Time functions (1200-1202) — TitleCase aliases auto-generated
    registry.register_with_id("auto.time.now_ms", 1200);
    registry.register_with_id("auto.time.now_sec", 1201);
    registry.register_with_id("auto.time.sleep_ms", 1202);
    registry.register_with_id("sleep", 1202); // Alias for auto.time.sleep_ms
    // Time functions carry return type info
    registry.register_with_id_and_type("auto.time.now_ms", 1200, NativeRetType::I64);
    registry.register_with_id_and_type("auto.time.now_sec", 1201, NativeRetType::I64);
    registry.register_with_id_and_type("auto.time.sleep_ms", 1202, NativeRetType::Void);

    // Process functions (1300-1304) — TitleCase aliases auto-generated
    registry.register_with_id("auto.process.exit", 1300);
    registry.register_with_id("auto.process.args", 1301);
    registry.register_with_id("auto.process.current_dir", 1302);
    registry.register_with_id("auto.process.set_current_dir", 1303);
    registry.register_with_id("auto.process.spawn", 1304);

    // Path functions (1400-1404) — TitleCase aliases auto-generated
    registry.register_with_id("auto.path.join", 1400);
    registry.register_with_id("auto.path.parent", 1401);
    registry.register_with_id("auto.path.extension", 1402);
    registry.register_with_id("auto.path.filename", 1403);
    registry.register_with_id("auto.path.canonicalize", 1404);

    // String functions (1500-1520) — TitleCase aliases auto-generated
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
    // Extra Str methods not in auto.str prefix
    registry.register_with_id("Str.split_once", 1518);
    registry.register_with_id("Str.match_count", 1519);
    registry.register_with_id("Str.replace_first", 1520);

    // Plan 198: auto.str.upper/lower/sub/slice and Str.slice removed — redundant with auto.str.to_upper/substr

    // Plan 198: Return types under canonical keys (get_return_type("str.X") falls back via to_canonical)
    registry.register_return_type("auto.str.len", NativeRetType::Int);
    registry.register_return_type("auto.str.is_empty", NativeRetType::Bool);
    registry.register_return_type("auto.str.char_at", NativeRetType::String);
    registry.register_return_type("auto.str.substr", NativeRetType::String);
    registry.register_return_type("auto.str.contains", NativeRetType::Bool);
    registry.register_return_type("auto.str.starts_with", NativeRetType::Bool);
    registry.register_return_type("auto.str.ends_with", NativeRetType::Bool);
    registry.register_return_type("auto.str.trim", NativeRetType::String);
    registry.register_return_type("auto.str.split", NativeRetType::String);
    registry.register_return_type("auto.str.repeat", NativeRetType::String);
    registry.register_return_type("auto.str.replace", NativeRetType::String);
    registry.register_return_type("auto.str.to_upper", NativeRetType::String);
    registry.register_return_type("auto.str.to_lower", NativeRetType::String);
    registry.register_return_type("auto.str.reverse", NativeRetType::String);
    registry.register_return_type("auto.str.find", NativeRetType::Int);
    registry.register_return_type("auto.str.lines", NativeRetType::String);
    registry.register_return_type("auto.str.parse_int", NativeRetType::Int);
    registry.register_return_type("auto.str.parse_float", NativeRetType::Float);

    // Plan 198: str.* lowercase aliases removed — to_canonical() resolves via auto.str.*
    // Exception: Str.split_once/match_count/replace_first have no auto.str.* canonical entry,
    // so their lowercase forms are kept here (codegen also constructs "str.split_once" etc.)
    registry.register_with_id_and_type("str.split_once", 1518, NativeRetType::List);
    registry.register_with_id_and_type("str.match_count", 1519, NativeRetType::Int);
    registry.register_with_id_and_type("str.replace_first", 1520, NativeRetType::String);

    // Option functions (1550-1551) — Plan 200 Task 2.4
    registry.register_with_id("Option.or", 1550);
    registry.register_with_id("Option.unwrap_or", 1551);

    // Char functions (1600-1606) — TitleCase aliases auto-generated
    registry.register_with_id("auto.char.is_alpha", 1600);
    registry.register_with_id("auto.char.is_digit", 1601);
    registry.register_with_id("auto.char.is_alphanum", 1602);
    registry.register_with_id("auto.char.is_whitespace", 1603);
    registry.register_with_id("auto.char.is_ident", 1604);
    registry.register_with_id("auto.char.to_lower", 1605);
    registry.register_with_id("auto.char.to_upper", 1606);

    // Math functions (1700-1703, 1710-1725) — TitleCase aliases auto-generated
    registry.register_with_id("auto.math.abs", 1700);
    registry.register_with_id("auto.math.min", 1701);
    registry.register_with_id("auto.math.max", 1702);
    registry.register_with_id("auto.math.sqrt", 1703);
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
    // Math functions carry return type info
    registry.register_with_id_and_type("auto.math.abs", 1700, NativeRetType::Int);
    registry.register_with_id_and_type("auto.math.min", 1701, NativeRetType::Int);
    registry.register_with_id_and_type("auto.math.max", 1702, NativeRetType::Int);
    registry.register_with_id_and_type("auto.math.sqrt", 1703, NativeRetType::Float);

    // JSON functions (1900-1917) — TitleCase aliases auto-generated
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

    // URL functions (2000-2012) — TitleCase aliases auto-generated
    registry.register_with_id("auto.url.encode", 2000);
    registry.register_with_id("auto.url.decode", 2001);
    registry.register_with_id("auto.url.parse", 2006);
    registry.register_with_id("auto.url.scheme", 2007);
    registry.register_with_id("auto.url.host", 2008);
    registry.register_with_id("auto.url.port", 2009);
    registry.register_with_id("auto.url.path", 2010);
    registry.register_with_id("auto.url.query", 2011);
    registry.register_with_id("auto.url.fragment", 2012);

    // Log functions (1800-1803)
    registry.register_with_id("Log.debug", 1800);
    registry.register_with_id("Log.info", 1801);
    registry.register_with_id("Log.warn", 1802);
    registry.register_with_id("Log.error", 1803);

    // URL function aliases
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

    // HTTP Stream functions (Plan 152) - 2240-2244 — TitleCase aliases auto-generated
    registry.register_with_id("auto.http_stream.get_stream", 2240);
    registry.register_with_id("auto.http_stream.post_stream", 2241);
    registry.register_with_id("auto.http_stream.stream_next", 2242);
    registry.register_with_id("auto.http_stream.stream_is_done", 2243);
    registry.register_with_id("auto.http_stream.stream_close", 2244);
    registry.register_with_id("parse_sse", 2250);

    // TaskSystem functions (Plan 127) - 2305-2307 — TitleCase aliases auto-generated
    registry.register_with_id("auto.task_system.start", 2305);
    registry.register_with_id("auto.task_system.run", 2306);
    registry.register_with_id("auto.task_system.stop", 2307);

    // Regex functions (Plan 159) - 2400-2401
    registry.register_with_id("Regex.is_match", 2400);
    registry.register_with_id("Regex.find_all", 2401);

    // Task aliases (for handle.send(), MonitorTask.send())
    // Plan 198: Task.spawn removed — to_canonical() resolves via auto.task.spawn
    registry.register_with_id("TaskHandle.send", 2301);
    registry.register_with_id("Task.send", 2311); // For singleton tasks like MonitorTask.send() - uses NATIVE_TASK_SINGLETON_SEND

    // Plan 192: Method table for Rust stdlib dynamic dispatch
    // When use.rust imports a type, its methods are registered here pointing to NATIVE_RUST_STDLIB_DISPATCH

    // =========================================================================
    // Plan 203 Phase 1: Qualified name registrations
    // Canonical qualified names for all commonly-used natives.
    // These are additive aliases — existing short names remain unchanged.
    // =========================================================================

    // List operations
    registry.register_qualified("auto.list.new", 100);
    registry.register_qualified("auto.list.push", 101);
    registry.register_qualified("auto.list.pop", 102);
    registry.register_qualified("auto.list.len", 103);
    registry.register_qualified("auto.list.is_empty", 104);
    registry.register_qualified("auto.list.clear", 105);
    registry.register_qualified("auto.list.get", 106);
    registry.register_qualified("auto.list.set", 107);
    registry.register_qualified("auto.list.insert", 108);
    registry.register_qualified("auto.list.remove", 109);
    registry.register_qualified("auto.list.drop", 110);
    registry.register_qualified("auto.list.reserve", 118);
    registry.register_qualified("auto.list.capacity", 205);

    // List higher-order functions
    registry.register_qualified("auto.list.map", 2060);
    registry.register_qualified("auto.list.filter", 2061);
    registry.register_qualified("auto.list.for_each", 2062);
    registry.register_qualified("auto.list.find", 2063);
    registry.register_qualified("auto.list.any", 2064);
    registry.register_qualified("auto.list.all", 2065);
    registry.register_qualified("auto.list.reduce", 2066);

    // Iterator operations
    registry.register_qualified("auto.list.iter", 111);
    registry.register_qualified("auto.iterator.next", 112);
    registry.register_qualified("auto.iterator.map", 113);
    registry.register_qualified("auto.iterator.filter", 114);
    registry.register_qualified("auto.iterator.collect", 115);
    registry.register_qualified("auto.iterator.reduce", 116);
    registry.register_qualified("auto.iterator.find", 117);
    registry.register_qualified("auto.iterator.enumerate", 118);

    // HashMap operations
    registry.register_qualified("auto.hashmap.new", 119);
    registry.register_qualified("auto.hashmap.insert", 120);
    registry.register_qualified("auto.hashmap.get", 122);
    registry.register_qualified("auto.hashmap.contains", 124);
    registry.register_qualified("auto.hashmap.remove", 125);
    registry.register_qualified("auto.hashmap.size", 126);
    registry.register_qualified("auto.hashmap.clear", 127);
    registry.register_qualified("auto.hashmap.drop", 128);

    // HashSet operations
    registry.register_qualified("auto.hashset.new", 129);
    registry.register_qualified("auto.hashset.insert", 130);
    registry.register_qualified("auto.hashset.contains", 131);
    registry.register_qualified("auto.hashset.remove", 132);
    registry.register_qualified("auto.hashset.size", 133);
    registry.register_qualified("auto.hashset.clear", 134);
    registry.register_qualified("auto.hashset.drop", 135);

    // VecDeque operations
    registry.register_qualified("auto.vecdeque.new", 136);
    registry.register_qualified("auto.vecdeque.push_back", 137);
    registry.register_qualified("auto.vecdeque.push_front", 138);
    registry.register_qualified("auto.vecdeque.pop_back", 139);
    registry.register_qualified("auto.vecdeque.pop_front", 140);
    registry.register_qualified("auto.vecdeque.front", 141);
    registry.register_qualified("auto.vecdeque.back", 142);
    registry.register_qualified("auto.vecdeque.size", 143);
    registry.register_qualified("auto.vecdeque.is_empty", 144);
    registry.register_qualified("auto.vecdeque.clear", 145);
    registry.register_qualified("auto.vecdeque.drop", 146);

    // BTreeMap operations
    registry.register_qualified("auto.btreemap.new", 147);
    registry.register_qualified("auto.btreemap.insert", 148);
    registry.register_qualified("auto.btreemap.get", 149);
    registry.register_qualified("auto.btreemap.contains", 150);
    registry.register_qualified("auto.btreemap.remove", 151);
    registry.register_qualified("auto.btreemap.size", 152);
    registry.register_qualified("auto.btreemap.is_empty", 153);
    registry.register_qualified("auto.btreemap.clear", 154);
    registry.register_qualified("auto.btreemap.first_key", 155);
    registry.register_qualified("auto.btreemap.last_key", 156);
    registry.register_qualified("auto.btreemap.drop", 157);

    // StringBuilder operations
    registry.register_qualified("auto.stringbuilder.new", 160);
    registry.register_qualified("auto.stringbuilder.append", 161);
    registry.register_qualified("auto.stringbuilder.append_int", 162);
    registry.register_qualified("auto.stringbuilder.append_char", 163);
    registry.register_qualified("auto.stringbuilder.len", 164);
    registry.register_qualified("auto.stringbuilder.clear", 165);
    registry.register_qualified("auto.stringbuilder.drop", 166);
    registry.register_qualified("auto.stringbuilder.build", 167);

    // String operations (VM-level, IDs 170-186)
    registry.register_qualified("auto.str.len", 1500);
    registry.register_qualified("auto.str.upper", 175);
    registry.register_qualified("auto.str.new", 177);
    registry.register_qualified("auto.str.push", 178);
    // Aliases: lower→to_lower, sub/slice→substr
    registry.register_qualified("auto.str.lower", 1512);
    registry.register_qualified("auto.str.sub", 1503);
    registry.register_qualified("auto.str.slice", 1503);
    registry.register_qualified("auto.str.pop", 179);
    registry.register_qualified("auto.str.get", 180);
    registry.register_qualified("auto.str.set", 181);
    registry.register_qualified("auto.str.insert", 182);
    registry.register_qualified("auto.str.remove", 183);
    registry.register_qualified("auto.str.clear", 184);
    registry.register_qualified("auto.str.is_empty", 1501);
    registry.register_qualified("auto.str.reserve", 186);

    // Heap / storage operations
    registry.register_qualified("auto.heap.new", 195);
    registry.register_qualified("auto.heap.capacity", 196);
    registry.register_qualified("auto.heap.try_grow", 197);
    registry.register_qualified("auto.heap.drop", 198);
    registry.register_qualified("auto.inline_int64.new", 199);
    registry.register_qualified("auto.inline_int64.capacity", 200);
    registry.register_qualified("auto.inline_int64.try_grow", 201);
    registry.register_qualified("auto.inline_int64.drop", 202);

    // Bit operations
    registry.register_qualified("auto.int.and", 210);
    registry.register_qualified("auto.int.or", 211);
    registry.register_qualified("auto.int.xor", 212);
    registry.register_qualified("auto.int.not", 213);
    registry.register_qualified("auto.int.shl", 214);
    registry.register_qualified("auto.int.shr", 215);
    registry.register_qualified("auto.int.sar", 216);
    registry.register_qualified("auto.int.rol", 217);
    registry.register_qualified("auto.int.ror", 218);
    registry.register_qualified("auto.int.count_ones", 220);
    registry.register_qualified("auto.int.leading_zeros", 221);
    registry.register_qualified("auto.int.trailing_zeros", 222);
    registry.register_qualified("auto.int.flip", 223);
    registry.register_qualified("auto.int.bit_read", 230);
    registry.register_qualified("auto.int.bit_test", 231);
    registry.register_qualified("auto.int.bit_on", 232);
    registry.register_qualified("auto.int.bit_off", 233);
    registry.register_qualified("auto.int.bit_flip", 234);

    // Memory allocation
    registry.register_qualified("auto.alloc.array", 190);
    registry.register_qualified("auto.realloc.array", 191);
    registry.register_qualified("auto.free.array", 192);

    // Note: auto.file.*, auto.str.*, auto.env.*, auto.time.*, auto.path.*,
    // auto.process.*, auto.char.*, auto.math.*, auto.json.*, auto.url.*,
    // auto.task.*, auto.http_stream.*, and auto.task_system.* are already
    // registered with their qualified names as the primary registration
    // in the registry above, so they are resolved via resolve_qualified()'s
    // fallback to the main registry.
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
