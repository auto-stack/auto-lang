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
use crate::vm::native_catalog::NATIVE_ID_ENTRIES;

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
    /// 3. Plan 250: Lazy registration from NATIVE_NAME_SET whitelist
    pub fn resolve_qualified(&mut self, path: &str) -> Option<u16> {
        // Direct lookup in unified registry
        if let Some(id) = self.registry.get(path).copied() {
            return Some(id);
        }
        // Plan 198: normalize short name to canonical form
        // "str.len" → "auto.str.len", "List.push" → "auto.list.push"
        if !path.starts_with("auto.") && !path.starts_with("rust.") && !path.starts_with("py.") {
            if let Some(canonical) = Self::to_canonical(path) {
                if let Some(id) = self.registry.get(&canonical).copied() {
                    return Some(id);
                }
            }
        }
        // Plan 250: Lazy registration — check name→ID map and register with fixed ID.
        // Try the path as-is first, then try canonical normalization.
        if let Some(&fixed_id) = NATIVE_ID_MAP.get(path) {
            self.registry.insert(path.to_string(), fixed_id);
            if fixed_id >= self.next_id {
                self.next_id = fixed_id + 1;
            }
            return Some(fixed_id);
        }
        if !path.starts_with("auto.") && !path.starts_with("rust.") && !path.starts_with("py.") {
            if let Some(canonical) = Self::to_canonical(path) {
                if let Some(&fixed_id) = NATIVE_ID_MAP.get(canonical.as_str()) {
                    self.registry.insert(path.to_string(), fixed_id);
                    if fixed_id >= self.next_id {
                        self.next_id = fixed_id + 1;
                    }
                    return Some(fixed_id);
                }
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
    // Plan 250: Known native names → fixed IDs for lazy registration
    pub static ref NATIVE_ID_MAP: HashMap<&'static str, u16> =
        NATIVE_ID_ENTRIES.iter().copied().collect();
}

/// Register all built-in native functions.
///
/// Plan 250: Only registers stdlib #[vm] declarations at startup.
/// All other native functions are lazily registered by resolve_qualified()
/// on first use during codegen, using NATIVE_NAME_SET as whitelist.
pub fn register_builtin_natives() {
    let mut registry = BIGVM_NATIVES.lock().unwrap();

    // Plan 198 Problem B: Auto-scan stdlib .vm.at files for #[vm] declarations.
    // These get fixed IDs from next_id counter (starting at 100).
    registry.register_vm_declarations();

    // Plan 250: No longer eager-registering 491 catalog entries.
    // They are lazily registered by resolve_qualified() when first referenced
    // during codegen. NATIVE_NAME_SET acts as the whitelist.
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
    /// Plan 192/240: Register all known methods for a Rust stdlib type in the native registry.
    /// All methods point to NATIVE_RUST_STDLIB_DISPATCH (3000) for dynamic dispatch.
    pub fn register_rust_type_methods(&mut self, type_name: &str) {
        let dispatch_id: u16 = 3000; // NATIVE_RUST_STDLIB_DISPATCH
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
