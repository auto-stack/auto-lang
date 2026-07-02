// =============================================================================

// Compile: New compilation API using AIE architecture

// =============================================================================

//

// This module provides the new entry points for compilation using the

// AIE (Auto Incremental Engine) architecture with Database and Indexer.

//

// Phase 1: Demonstrate end-to-end workflow (parse 鈫?index 鈫?query)

// Phase 2: Add incremental compilation (file hashing, dirty tracking)

// Phase 3: Add fine-grained incremental (fragment hashing, patches)



use crate::module_cache::{AutoCache, ModuleCache};

use crate::database::Database;

use crate::dep_scanner::scan_dep_statements;

use crate::error::{AutoError, AutoResult};

use crate::indexer::Indexer;

use crate::parser::Parser;

use crate::scope::{Sid, SID_PATH_GLOBAL};

use crate::types::TypeStore;

use crate::symbols::SymbolLocation;

use crate::use_scanner::{scan_use_statements, UseStatement};

use crate::util::find_std_lib;

use std::collections::HashMap;

use auto_cache::{Sandbox, CrateMetadata, CrateSource};

use auto_val::AutoStr;

use std::rc::Rc;

use std::cell::RefCell;

use std::sync::Arc;

use std::sync::RwLock;


/// Source info for a declared dep (git/path/version)
#[derive(Clone)]
struct DepSourceInfo {
    version: Option<String>,
    git: Option<String>,
    git_ref: Option<String>,
    path: Option<String>,
}



/// Compilation session using the new AIE architecture

///

/// A compilation session manages a Database and provides methods to

/// compile source code with incremental support.

///

/// Phase 4.5: Database is now wrapped in Arc<RwLock<>> for sharing with Evaler

use std::collections::HashSet;



/// Phase 3 (Plan 065): QueryEngine integration complete (now accepts Arc<RwLock<Database>>)

/// Plan 085: Added type_store for module dependency management

/// Plan 085 Phase 5: Added auto_cache for module caching

/// Plan 092: Added sandbox for Rust FFI

pub struct CompileSession {

    db: Arc<RwLock<Database>>,

    query_engine: Option<crate::query::QueryEngine>,

    /// Plan 085: Unified type store for all loaded modules

    type_store: Arc<RwLock<TypeStore>>,

    /// Plan 085 Phase 5: Module cache for incremental compilation

    auto_cache: AutoCache,

    /// Plan 092: Sandbox for Rust FFI

    sandbox: Option<Sandbox>,

    /// Plan 092: Declared crate names (from dep statements)

    declared_crates: HashSet<String>,

    /// Features for declared crates (crate_name → feature list)
    dep_features: HashMap<String, Vec<String>>,

    /// Source info for declared crates (git/path/version)
    dep_sources: HashMap<String, DepSourceInfo>,

    /// Plan 167: Tracks modules currently being loaded (for circular dependency detection)

    loading_stack: Vec<String>,

    /// Cross-module function calls: compiled dependency modules

    compiled_modules: Vec<crate::vm::loader::Module>,

    /// Plan 346: Generic registries from compiled dependency modules.
    /// These are merged into the main module's generic_registry at link time
    /// so that cross-module generic types (e.g. List<Note> in db.at where Note
    /// is defined in api.at) resolve correctly at runtime.
    pub dep_generic_registry: std::cell::RefCell<crate::vm::generic_registry::GenericRegistry>,

    /// Plan 346: Object metadata pools from compiled dependency modules.
    /// Merged into the main module's pools at link time so object literals
    /// in dep modules (e.g. Note { id: 0, ... }) resolve correctly.
    pub dep_object_keys: std::cell::RefCell<Vec<Vec<auto_val::ValueKey>>>,
    pub dep_object_types: std::cell::RefCell<Vec<Vec<crate::vm::codegen::ObjectType>>>,

    /// Directories where source files have been found (for multi-dir module resolution)
    source_dirs: Vec<std::path::PathBuf>,

    /// Paths of modules already compiled to bytecode (to avoid duplicate compilation)
    compiled_module_paths: HashSet<String>,

    /// Plan 212b Task 2: Rust imports collected from use.rust statements

    /// Maps crate_name 鈫?list of imported function names

    rust_imports: std::collections::HashMap<String, Vec<String>>,

    /// Plan 214: Python imports collected from use.py statements

    /// Maps module_name 鈫?list of imported function names

    py_imports: std::collections::HashMap<String, Vec<String>>,

}



impl Clone for CompileSession {

    fn clone(&self) -> Self {

        Self {

            db: self.db.clone(),

            query_engine: None, // QueryEngine is recreated on-demand after clone

            type_store: self.type_store.clone(),

            auto_cache: self.auto_cache.clone(),

            sandbox: None, // Sandbox is recreated on-demand

            declared_crates: self.declared_crates.clone(),

            dep_features: self.dep_features.clone(),

            dep_sources: self.dep_sources.clone(),

            loading_stack: Vec::new(),

            compiled_modules: Vec::new(),
            dep_generic_registry: std::cell::RefCell::new(crate::vm::generic_registry::GenericRegistry::new()),
            dep_object_keys: std::cell::RefCell::new(Vec::new()),
            dep_object_types: std::cell::RefCell::new(Vec::new()),

            source_dirs: self.source_dirs.clone(),

            compiled_module_paths: HashSet::new(),

            rust_imports: self.rust_imports.clone(),

            py_imports: self.py_imports.clone(),

        }

    }

}



impl CompileSession {

    /// Create a new compilation session

    pub fn new() -> Self {

        let db = Arc::new(RwLock::new(Database::new()));

        let type_store = Arc::new(RwLock::new(TypeStore::new()));

        Self {

            db,

            query_engine: None,

            type_store,

            auto_cache: AutoCache::new(),

            sandbox: None,

            declared_crates: HashSet::new(),

            dep_features: HashMap::new(),

            dep_sources: HashMap::new(),

            loading_stack: Vec::new(),

            compiled_modules: Vec::new(),
            dep_generic_registry: std::cell::RefCell::new(crate::vm::generic_registry::GenericRegistry::new()),
            dep_object_keys: std::cell::RefCell::new(Vec::new()),
            dep_object_types: std::cell::RefCell::new(Vec::new()),

            source_dirs: Vec::new(),

            compiled_module_paths: HashSet::new(),

            rust_imports: std::collections::HashMap::new(),

            py_imports: std::collections::HashMap::new(),

        }

    }

    /// Plan 327: Add a source directory to the search path for module resolution.
    /// Called by execute_autovm_with_path to seed the directory of the source file,
    /// so `use db` finds db.at relative to the source.
    pub fn add_source_dir(&mut self, dir: std::path::PathBuf) {
        if !self.source_dirs.contains(&dir) {
            self.source_dirs.push(dir);
        }
    }



    /// Get reference to the type store (Plan 085)

    pub fn type_store(&self) -> Arc<RwLock<TypeStore>> {

        self.type_store.clone()

    }



    /// Get cache statistics (Plan 085 Phase 5)

    pub fn cache_stats(&self) -> crate::module_cache::CacheStats {

        self.auto_cache.stats()

    }



    /// Get number of cached modules (Plan 085 Phase 5)

    /// Take all compiled dependency modules (for cross-module linking)

    pub fn take_compiled_modules(&mut self) -> Vec<crate::vm::loader::Module> {

        std::mem::take(&mut self.compiled_modules)

    }



    pub fn cached_module_count(&self) -> usize {

        self.auto_cache.len()

    }



    /// Get reference to the database (for sharing with Evaler)

    pub fn db(&self) -> Arc<RwLock<Database>> {

        self.db.clone()

    }



    /// Get the underlying database (for advanced usage)

    pub fn database(&self) -> std::sync::LockResult<std::sync::RwLockReadGuard<'_, Database>> {

        self.db.read()

    }



    /// Get mutable access to the database (for advanced usage)

    pub fn database_mut(&self) -> std::sync::LockResult<std::sync::RwLockWriteGuard<'_, Database>> {

        self.db.write()

    }



    /// Get or create the QueryEngine for this session

    ///

    /// **Plan 065 Phase 3**: QueryEngine is created on-demand and reused across calls

    pub fn query_engine(&mut self) -> &mut crate::query::QueryEngine {

        if self.query_engine.is_none() {

            self.query_engine = Some(crate::query::QueryEngine::new(self.db.clone()));

        }

        self.query_engine.as_mut().unwrap()

    }



    /// Get the QueryEngine if it exists

    ///

    /// **Plan 065 Phase 3**: Returns None if QueryEngine hasn't been created yet

    pub fn get_query_engine(&self) -> Option<&crate::query::QueryEngine> {

        self.query_engine.as_ref()

    }



    /// Plan 085: 棰勫鐞?use 璇彞

    ///

    /// 鎵弿婧愮爜涓殑鎵€鏈?use 璇彞锛屽苟鍔犺浇渚濊禆妯″潡鍒?type_store銆?

    /// 杩欏簲璇ュ湪 compile_source() 涔嬪墠璋冪敤銆?

    ///

    /// # Arguments

    ///

    /// * `source` - The source code to scan for use statements

    ///

    /// # Returns

    ///

    /// The number of modules that were processed.

    ///

    /// # Example

    ///

    /// ```rust,no_run

    /// use auto_lang::compile::CompileSession;

    ///

    /// let mut session = CompileSession::new();

    /// let source = "use std.io\nuse std.fs: read, write";

    /// session.resolve_uses(source).unwrap();

    /// ```

    pub fn resolve_uses(&mut self, source: &str) -> AutoResult<usize> {

        let use_statements = scan_use_statements(source);

        let mut loaded_count = 0;



        for use_stmt in &use_statements {

            // Skip C imports - they don't have AutoLang types

            if use_stmt.is_c_import {

                continue;

            }



            // Plan 092/190: Handle Rust imports

            if use_stmt.is_rust_import {

                // Extract crate name from module path (first segment)

                let crate_name = use_stmt.module.split("::").next().unwrap_or(&use_stmt.module).to_string();

                // Plan 212 Phase 2.2: Built-in opaque types don't need dep declaration
                const BUILTIN_OPAQUE_CRATES: &[&str] = &[
                    "regex", "url", "semver", "log", "env_logger", "tracing",
                    "rand", "rand_distr", "chrono", "csv", "walkdir", "toml",
                    "serde_json", "percent_encoding", "urlencoding", "base64", "hex",
                    "sha2", "mime_guess", "same_file", "heapless", "clap",
                    "ansi_term", "simplelog", "tar", "flate2", "crossbeam",
                    "anyhow", "serde", "tokio", "num", "ndarray",
                ];
                let is_builtin = BUILTIN_OPAQUE_CRATES.contains(&crate_name.as_str());

                if !is_builtin && !self.is_dep_declared(&crate_name) {

                    return Err(AutoError::Msg(format!(

                        "Crate '{}' not declared. Add `dep {}` before `use.rust`.",

                        crate_name, crate_name

                    )));

                }



                // Plan 212b Task 2: Collect imported function names for compilation

                if !use_stmt.items.is_empty() {

                    self.rust_imports

                        .entry(crate_name.clone())

                        .or_default()

                        .extend(use_stmt.items.iter().cloned());

                }



                // Plan 190: Register imported Rust types in TypeStore

                if let Ok(mut store) = self.type_store.write() {

                    if use_stmt.is_wildcard {

                        log::info!("Rust wildcard import: {}", use_stmt.module);

                    } else if !use_stmt.items.is_empty() {

                        for item in &use_stmt.items {

                            let full_path = format!("{}::{}", use_stmt.module, item);

                            store.register_rust_type(item.as_str(), full_path);

                            // Plan 192: Register methods in VM native registry for runtime dispatch

                            if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {

                                registry.register_rust_type_methods(item.as_str());

                            }

                        }

                    } else {

                        if let Some(short_name) = use_stmt.module.rsplit("::").next() {

                            store.register_rust_type(short_name, use_stmt.module.as_str());

                            if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {

                                registry.register_rust_type_methods(short_name);

                            }

                        }

                    }

                }



                loaded_count += 1;

                continue;

            }



            // Plan 214: Handle Python imports

            if use_stmt.is_python_import {

                #[cfg(feature = "python")]

                {

                    if !use_stmt.items.is_empty() {

                        self.py_imports

                            .entry(use_stmt.module.clone())

                            .or_default()

                            .extend(use_stmt.items.iter().cloned());

                    }

                }

                #[cfg(not(feature = "python"))]

                {

                    return Err(AutoError::Msg(format!(

                        "Python FFI not enabled. Rebuild with `--features python` to use `use.py`."

                    )));

                }

                #[cfg(feature = "python")]

                {

                    loaded_count += 1;

                    continue;

                }

            }



            self.load_module(use_stmt)?;



            loaded_count += 1;

        }



        Ok(loaded_count)

    }



    /// Plan 092: Resolve `dep` statements and register with sandbox/registry

    ///

    /// This scans source code for `dep` statements and registers them

    /// with the sandbox for later use by `use.rust` statements.

    ///

    /// # Example

    ///

    /// ```rust,no_run

    /// use auto_lang::compile::CompileSession;

    ///

    /// let mut session = CompileSession::new();

    /// let source = "dep serde(version: \"1.0\")";

    /// session.resolve_deps(source).unwrap();

    /// ```

    pub fn resolve_deps(&mut self, source: &str) -> AutoResult<usize> {

        let dep_statements = scan_dep_statements(source);
        if dep_statements.is_empty() {
            return Ok(0);
        }
        let mut registered_count = 0;

        // Ensure sandbox is initialized

        if self.sandbox.is_none() {

            match Sandbox::new() {

                Ok(s) => self.sandbox = Some(s),

                Err(e) => {

                    log::warn!("Failed to initialize sandbox: {}", e);

                    // Continue without sandbox - deps won't be usable

                    return Ok(0);

                }

            }

        }



        for dep in &dep_statements {

            // Skip non-Rust deps (for future extensibility)

            if !dep.is_rust {

                continue;

            }



            // Register crate name

            self.declared_crates.insert(dep.name.clone());

            // Store features for compile_dep

            if !dep.features.is_empty() {
                self.dep_features.insert(
                    dep.name.to_string(),
                    dep.features.iter().map(|f| f.to_string()).collect(),
                );
            }

            // Store source info (git/path/version)

            self.dep_sources.insert(
                dep.name.to_string(),
                DepSourceInfo {
                    version: dep.version.as_ref().map(|v| v.to_string()),
                    git: dep.git.as_ref().map(|g| g.to_string()),
                    git_ref: dep.git_ref.as_ref().map(|r| r.to_string()),
                    path: dep.path.as_ref().map(|p| p.to_string()),
                },
            );



            // Log the dependency

            log::info!(

                "Registered dep: {} (version: {:?}, features: {:?})",

                dep.name,

                dep.version,

                dep.features

            );



            // Register with sandbox registry (Plan 092 Phase 6)

            if let Some(ref mut sandbox) = self.sandbox {

                let metadata = CrateMetadata {

                    name: dep.name.to_string(),

                    version: dep.version.as_ref().map(|v| v.to_string()).unwrap_or_default(),

                    rustc_version: sandbox.rustc_version().to_string(),

                    target: sandbox.target().to_string(),

                    dependencies: vec![],

                    abi_hash: String::new(),

                    library_path: std::path::PathBuf::new(),

                    compiled_at: std::time::SystemTime::now()

                        .duration_since(std::time::UNIX_EPOCH)

                        .unwrap_or_default()

                        .as_secs(),

                    source: if dep.is_local() { CrateSource::Local } else { CrateSource::CratesIo },

                };



                if let Err(e) = sandbox.registry().register(&metadata) {

                    log::warn!("Failed to register crate {} in sandbox registry: {}", dep.name, e);

                }

            }



            registered_count += 1;

        }



        // Plan 212b Task 2: Compile deps that have rust imports
        // Phase 2.1: Convert function names to FunctionShim descriptors with signatures
        if let Some(ref sandbox) = self.sandbox {
            use crate::ffi::known_signature;
            use auto_cache::sandbox::{FunctionShim, ShimType};

            for (crate_name, functions) in &self.rust_imports {
                if self.declared_crates.contains(crate_name) {
                    let shims: Vec<FunctionShim> = functions.iter().map(|func| {
                        match known_signature(crate_name, func) {
                            Some(sig) => {
                                let param_types: Vec<ShimType> = sig.params.iter().map(|t| match t {
                                    crate::ffi::RustType::Void => ShimType::Void,
                                    crate::ffi::RustType::Bool => ShimType::Bool,
                                    crate::ffi::RustType::Int => ShimType::I32,
                                    crate::ffi::RustType::Long => ShimType::I64,
                                    crate::ffi::RustType::Float | crate::ffi::RustType::Double => ShimType::F64,
                                    crate::ffi::RustType::String => ShimType::CString,
                                    _ => ShimType::CString,
                                }).collect();
                                let return_type = match sig.returns {
                                    crate::ffi::RustType::Void => ShimType::Void,
                                    crate::ffi::RustType::Bool => ShimType::Bool,
                                    crate::ffi::RustType::Int => ShimType::I32,
                                    crate::ffi::RustType::Long => ShimType::I64,
                                    crate::ffi::RustType::Float | crate::ffi::RustType::Double => ShimType::F64,
                                    crate::ffi::RustType::String => ShimType::CString,
                                    _ => ShimType::CString,
                                };
                                FunctionShim {
                                    name: func.clone(),
                                    param_types,
                                    return_type,
                                    body_override: None,
                                    returns_result: false,
                                }
                            }
                            None => FunctionShim::string_to_string(func),
                        }
                    }).collect();

                    let dep_source = self.build_dep_source(crate_name);

                    match sandbox.compile_dep(crate_name, &shims, &dep_source) {
                        Ok(path) => {
                            log::info!("Compiled dep {} -> {}", crate_name, path.display());
                        }
                        Err(e) => {
                            log::warn!("Failed to compile dep {}: {}", crate_name, e);
                        }
                    }
                }
            }

        }



        Ok(registered_count)

    }



    /// Plan 092: Check if a crate has been declared as a dependency

    ///

    /// Returns true if the crate was declared in a `dep` statement.

    /// Build a DepSource for compile_dep from stored dep info.
    fn build_dep_source(&self, crate_name: &str) -> auto_cache::sandbox::DepSource {
        let features = self.dep_features.get(crate_name)
            .map(|f| f.clone())
            .unwrap_or_default();
        let src = self.dep_sources.get(crate_name);
        auto_cache::sandbox::DepSource {
            version: src.and_then(|s| s.version.clone()),
            features,
            git: src.and_then(|s| s.git.clone()),
            git_ref: src.and_then(|s| s.git_ref.clone()),
            path: src.and_then(|s| s.path.clone()),
        }
    }

    pub fn is_dep_declared(&self, crate_name: &str) -> bool {

        if self.declared_crates.contains(crate_name) {

            return true;

        }

        // Plan 190: Rust built-in crates are always available

        matches!(crate_name, "std" | "core" | "alloc" | "proc_macro")

    }



    /// Plan 212b Task 2: Collect Rust imports from source code

    ///

    /// Scans for `use.rust` statements and collects the function names

    /// per crate. This should be called after `resolve_deps()` to ensure

    /// the crates have been declared.

    pub fn collect_rust_imports(&mut self, source: &str) -> AutoResult<()> {

        let use_statements = scan_use_statements(source);

        for use_stmt in &use_statements {

            if !use_stmt.is_rust_import || use_stmt.items.is_empty() {

                continue;

            }

            let crate_name = use_stmt.module.split("::").next().unwrap_or(&use_stmt.module).to_string();

            self.rust_imports

                .entry(crate_name)

                .or_default()

                .extend(use_stmt.items.iter().cloned());

        }

        Ok(())

    }



    /// Plan 212b Task 2: Get collected Rust imports

    ///

    /// Returns a map of crate_name 鈫?list of function names imported via use.rust.

    pub fn rust_imports(&self) -> &std::collections::HashMap<String, Vec<String>> {

        &self.rust_imports

    }



    /// Plan 214: Collect Python imports from source code

    ///

    /// Scans for `use.py` statements and collects the function names

    /// per Python module.

    pub fn collect_py_imports(&mut self, source: &str) -> AutoResult<()> {

        let use_statements = scan_use_statements(source);

        for use_stmt in &use_statements {

            if !use_stmt.is_python_import {

                continue;

            }

            self.py_imports

                .entry(use_stmt.module.clone())

                .or_default()

                .extend(use_stmt.items.iter().cloned());

        }

        Ok(())

    }



    /// Plan 214: Get collected Python imports

    pub fn py_imports(&self) -> &std::collections::HashMap<String, Vec<String>> {

        &self.py_imports

    }



    /// Plan 092: Get the sandbox (for FFI bridge integration)

    pub fn sandbox(&self) -> Option<&Sandbox> {

        self.sandbox.as_ref()

    }



    /// Plan 092 Phase 6: Create a RustFfiBridge for loading Rust crates

    ///

    /// Returns a RustFfiBridge that can be used to:

    /// 1. Load compiled Rust crates dynamically

    /// 2. Register Rust functions as native functions

    /// 3. Call Rust functions from AutoVM bytecode

    ///

    /// # Example

    /// ```ignore

    /// let session = CompileSession::new();

    /// let bridge = session.create_rust_ffi_bridge()?;

    /// bridge.load_rust_crate("serde", "1.0.193")?;

    /// bridge.register_function("serde", "from_str", signature)?;

    /// ```

    pub fn create_rust_ffi_bridge(&self) -> Result<crate::ffi::RustFfiBridge, AutoError> {

        crate::ffi::RustFfiBridge::new()

            .map_err(|e| AutoError::Msg(format!("Failed to create Rust FFI bridge: {:?}", e)))

    }



    /// Plan 092 Phase 6: Get list of declared crates

    ///

    /// Returns the names of all crates declared via `dep` statements.

    pub fn get_declared_crates(&self) -> &HashSet<String> {

        &self.declared_crates

    }



    /// Plan 085: 鍔犺浇妯″潡鍒?type_store

    ///

    /// 鏍规嵁妯″潡璺緞鏌ユ壘骞跺姞杞芥ā鍧楋紝灏嗙鍙峰悎骞跺埌 type_store銆?

    /// Plan 085 Phase 5: 鏀寔妯″潡缂撳瓨锛岄伩鍏嶉噸澶嶈В鏋愩€?

    /// Plan 094: 鍚屾椂鍔犺浇 .at (root) 鍜?.vm.at (context) 鏂囦欢锛屽悎骞跺鐞嗐€?

    fn load_module(&mut self, use_stmt: &UseStatement) -> AutoResult<()> {
        // Plan 327: If the module is already compiled (from a previous load or
        // a different entry into a circular dependency), skip — don't recompile.
        // This allows legitimate circular deps (db use api: Note + api use db)
        // where types and functions cross-reference.
        if self.compiled_modules.iter().any(|m| m.name == use_stmt.module) {
            return Ok(());
        }

        // Plan 167: Circular dependency detection — only error if the module
        // is currently being loaded (true cycle, not already-resolved).
        if self.loading_stack.contains(&use_stmt.module) {
            // Instead of erroring, skip (the module is being loaded higher in
            // the stack; its types/functions will be available once it finishes).
            return Ok(());
        }

        self.loading_stack.push(use_stmt.module.clone());
        let result = self.load_module_inner(use_stmt);
        self.loading_stack.pop();
        result

    }



    /// Inner implementation of load_module (called after cycle check)

    fn load_module_inner(&mut self, use_stmt: &UseStatement) -> AutoResult<()> {

        // Phase 5: 妫€鏌?AutoCache

        if self.auto_cache.is_cached_and_valid(&use_stmt.module) {

            // 浣跨敤缂撳瓨鐨?type_store

            if let Some(cached) = self.auto_cache.get(&use_stmt.module) {

                let mut store = self.type_store.write().unwrap();

                if use_stmt.is_wildcard {

                    store.merge(&cached.type_store);

                } else if !use_stmt.items.is_empty() {

                    store.import_items(&cached.type_store, &use_stmt.items);

                } else {

                    store.merge(&cached.type_store);

                }

                return Ok(());

            }

        }



        // 灏嗘ā鍧楄矾寰勮浆鎹负鏂囦欢璺緞

        let raw_module_path = use_stmt.module.replace(".", "/");

        // Handle super/super2/super3/super4 prefix -- resolve relative to parent directories
        let (module_path, _parent_dirs): (String, Vec<std::path::PathBuf>) = {
            let path = raw_module_path.as_str();
            let (super_count, rest) = if path.starts_with("super4/") {
                (4, &path[7..])
            } else if path.starts_with("super3/") {
                (3, &path[7..])
            } else if path.starts_with("super2/") {
                (2, &path[7..])
            } else if path.starts_with("super/") {
                (1, &path[6..])
            } else {
                (0, path)
            };
            if super_count > 0 {
                let parents: Vec<std::path::PathBuf> = self.source_dirs.iter()
                    .filter_map(|d| {
                        let mut p = d.to_path_buf();
                        for _ in 0..super_count {
                            p = p.parent()?.to_path_buf();
                        }
                        Some(p)
                    })
                    .collect();
                (rest.to_string(), parents)
            } else {
                (raw_module_path.clone(), vec![])
            }
        };



        let extensions = [".at", ".au", ".auto"];

        let mut found_path: Option<std::path::PathBuf> = None;



        // Resolve stdlib root via find_std_lib (searches CARGO_MANIFEST_DIR, ~/.auto/libs/, system paths)

        let stdlib_base = find_std_lib()

            .map(|s| std::path::PathBuf::from(s.as_str()))

            .unwrap_or_else(|_| std::path::PathBuf::from("stdlib/auto"));



        for ext in &extensions {

            // 1. Try relative to current working directory (local modules)

            let path = std::path::Path::new(&module_path).with_extension(&ext[1..]);

            if path.exists() {

                found_path = Some(path);

                break;

            }

            // 2. Try stdlib path

            // For "auto.io", module_path is "auto/io", but stdlib file is time.at

            // Strip the "auto/" prefix when building stdlib path

            let stdlib_relative = if module_path.starts_with("auto/") {

                &module_path[5..] // strip "auto/"

            } else {

                &module_path

            };

            let stdlib_path = stdlib_base.join(stdlib_relative).with_extension(&ext[1..]);

            if stdlib_path.exists() {

                found_path = Some(stdlib_path);

                break;

            }

        }



        // 3. Try directory module pattern: tools/ → tools/mod.at
        if found_path.is_none() {
            for ext in &extensions {
                let dir_mod_path = std::path::Path::new(&module_path).join(format!("mod{}", ext));
                if dir_mod_path.exists() {
                    found_path = Some(dir_mod_path);
                    break;
                }
                // Also try stdlib directory module
                let stdlib_relative = if module_path.starts_with("auto/") {
                    &module_path[5..]
                } else {
                    &module_path
                };
                let stdlib_dir_mod = stdlib_base.join(stdlib_relative).join(format!("mod{}", ext));
                if stdlib_dir_mod.exists() {
                    found_path = Some(stdlib_dir_mod);
                    break;
                }
            }
        }



        // 4. Try searching in all known source directories (handles cross-dir imports
        // like agent.at's `use registry` resolving to tools/registry.at)
        // Also search parent dirs for `super.xxx` resolution
        if found_path.is_none() {
            let mut all_search_dirs: Vec<&std::path::Path> = self.source_dirs.iter()
                .map(|d| d.as_path()).collect();
            for pd in &_parent_dirs {
                if !all_search_dirs.iter().any(|d| *d == pd) {
                    all_search_dirs.push(pd);
                }
            }
            for ext in &extensions {
                for src_dir in &all_search_dirs {
                    let path = src_dir.join(format!("{}{}", module_path, ext));
                    if path.exists() {
                        found_path = Some(path);
                        break;
                    }
                    // Also try directory module in source dirs
                    let dir_mod = src_dir.join(&module_path).join(format!("mod{}", ext));
                    if dir_mod.exists() {
                        found_path = Some(dir_mod);
                        break;
                    }
                }
                if found_path.is_some() { break; }
            }
        }



        let root_path = found_path.ok_or_else(|| {

            AutoError::Msg(format!("Module not found: {} (module_path={}, parent_dirs={:?})", use_stmt.module, module_path, _parent_dirs))

        })?;

        // Record the directory of this module for future lookups
        // Use canonical (absolute) path so that parent() calculations work correctly
        if let Some(parent) = root_path.parent() {
            let abs_parent = if parent.as_os_str().is_empty() {
                std::path::PathBuf::from(".")
            } else {
                std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf())
            };
            if !self.source_dirs.iter().any(|d| *d == abs_parent) {
                self.source_dirs.push(abs_parent);
            }
        }



        // 璇诲彇妯″潡鏍规枃浠?

        let mut module_source = std::fs::read_to_string(&root_path)

            .map_err(|e| AutoError::Io(format!("Failed to read module {}: {}", root_path.display(), e)))?;



        // Plan 094: 灏濊瘯鍔犺浇涓婁笅鏂囨枃浠?(.vm.at)

        // 鏍规嵁缂栬瘧寮曟搸绫诲瀷閫夋嫨涓婁笅鏂囨枃浠跺悗缂€

        let context_ext = ".vm.at"; // AutoVM 浣跨敤 .vm.at

        let context_path = root_path.with_file_name({

            let name = root_path.file_name().unwrap().to_str().unwrap();

            format!("{}{}", name.strip_suffix(".at").unwrap_or(name), context_ext)

        });



        // 妫€鏌ヤ笂涓嬫枃鏂囦欢鏄惁瀛樺湪

        let full_context_path = if context_path.exists() {

            Some(context_path.clone())

        } else {

            // 涔熷皾璇?stdlib/auto 璺緞

            let stdlib_context = std::path::Path::new("stdlib/auto").join(&context_path);

            if stdlib_context.exists() {

                Some(stdlib_context)

            } else {

                None

            }

        };



        // 濡傛灉涓婁笅鏂囨枃浠跺瓨鍦紝璇诲彇骞跺悎骞?

        if let Some(ctx_path) = full_context_path {

            let context_source = std::fs::read_to_string(&ctx_path)

                .map_err(|e| AutoError::Io(format!("Failed to read context file {}: {}", ctx_path.display(), e)))?;



            // 鍚堝苟涓や釜鏂囦欢鐨勫唴瀹癸紙鐢ㄦ崲琛屽垎闅旓級

            module_source.push('\n');

            module_source.push_str(&context_source);

        }



        // DEBUG: Print module source being parsed

        // Recursively resolve use statements inside this module FIRST
        // so that dependencies (e.g. agent.at's `use permission`) are loaded
        // into the session TypeStore before parse_module_to_type_store needs them
        self.resolve_uses(&module_source)?;

        // 瑙ｆ瀽鍚堝苟鍚庣殑妯″潡鑾峰彇 type_store
        let module_type_store = self.parse_module_to_type_store(&module_source, &root_path.to_string_lossy())?;

        // Cross-module function calls: compile module to bytecode
        // Skip if already compiled (avoid duplicate symbols)
        let path_key = root_path.canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| root_path.to_string_lossy().to_string());
        if !self.compiled_module_paths.contains(&path_key) {
            let module_code = self.compile_module_to_bytecode(&module_source, &root_path.to_string_lossy())?;
            if !module_code.exports.is_empty() {
                self.compiled_modules.push(module_code);
            }
            self.compiled_module_paths.insert(path_key);
        }



        // Phase 5: 瀛樺叆 AutoCache

        let cache_entry = ModuleCache::with_file(

            &use_stmt.module,

            module_type_store.clone(),

            root_path.to_string_lossy(),

            &module_source,

        );

        self.auto_cache.store(&use_stmt.module, cache_entry);



        // 鍚堝苟鍒颁富 type_store

        {

            let mut store = self.type_store.write().unwrap();

            if use_stmt.is_wildcard {

                // 閫氶厤绗﹀鍏ワ細鍚堝苟鎵€鏈夌鍙?

                store.merge(&module_type_store);

            } else if !use_stmt.items.is_empty() {

                // 閫夋嫨鎬у鍏ワ細鍙鍏ユ寚瀹氶」

                store.import_items(&module_type_store, &use_stmt.items);

            } else {

                // 榛樿瀵煎叆鏁翠釜妯″潡

                store.merge(&module_type_store);

            }

        }



        Ok(())

    }



    /// Plan 085: 瑙ｆ瀽妯″潡骞舵彁鍙?type_store

    ///

    /// 瑙ｆ瀽妯″潡婧愮爜锛屾彁鍙栨墍鏈夌被鍨嬨€佸嚱鏁般€乻pec 澹版槑鍒?TypeStore銆?

    fn parse_module_to_type_store(&self, source: &str, path: &str) -> AutoResult<TypeStore> {

        let mut type_store = TypeStore::new();



        // 浣跨敤 Parser 瑙ｆ瀽婧愮爜锛屼紶鍏ュ叏灞?type_store 浠ヤ究瑙ｆ瀽 use 瀵煎叆
        let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
        let mut parser = Parser::new_with_type_store(source, self.type_store.clone());

        let ast = parser.parse()

            .map_err(|e| crate::error::attach_source(e, path.to_string(), source.to_string()))?;



        // 浠?AST 鎻愬彇澹版槑

        for stmt in &ast.stmts {

            match stmt {

                crate::ast::Stmt::Fn(fn_decl) => {

                    type_store.register_fn_decl(fn_decl);

                }

                crate::ast::Stmt::TypeDecl(type_decl) => {

                    type_store.register_type_decl(type_decl);

                }

                crate::ast::Stmt::SpecDecl(spec_decl) => {

                    type_store.register_spec_decl(spec_decl);

                }

                crate::ast::Stmt::Ext(ext) => {

                    type_store.register_ext_methods(ext);

                }

                crate::ast::Stmt::EnumDecl(enum_decl) => {

                    type_store.register_enum_decl(enum_decl.clone());

                }

                _ => {}

            }

        }



        Ok(type_store)

    }



    /// Compile a module's source code to bytecode (for cross-module function calls)

    fn compile_module_to_bytecode(

        &self,

        source: &str,

        path: &str,

    ) -> AutoResult<crate::vm::loader::Module> {

        use crate::vm::codegen::Codegen;

        use crate::vm::opcode::OpCode;



        let mut parser = Parser::new_with_type_store(source, self.type_store.clone());

        let ast = parser.parse()

            .map_err(|e| crate::error::attach_source(e, path.to_string(), source.to_string()))?;

        // Plan 345: compute module name early (needed for global qualification).
        let module_name = path.replace('\\', "/")

            .rsplit('/').next().unwrap_or("unknown")

            .trim_end_matches(".at")

            .trim_end_matches(".auto")

            .to_string();

        let mut codegen = Codegen::new_with_type_store(self.type_store.clone());

        // Plan 345: set module context so global variables get qualified keys
        // (e.g. "db.notes" instead of "notes"), providing cross-module isolation.
        codegen.current_module = module_name.clone();

        // Plan 346 revert: compile ALL statements in declaration order (Type,
        // Store, Fn, Use) as normal top-level bytecode — do NOT wrap Store
        // into __module_init. The __module_init approach broke object literal
        // indices (db module's Note{} literals referenced wrong object_keys
        // pool indices after string/pool merge). By compiling in-place, all
        // pools (strings, object_keys, object_types) are self-consistent within
        // the module. The module's top-level var init code runs when __module_init
        // is called from execute_autovm_with_path (which spawns a task at the
        // module's code start address — Store code is at the top before Fn code).
        for stmt in &ast.stmts {
            match stmt {
                crate::ast::Stmt::TypeDecl(_)
                | crate::ast::Stmt::EnumDecl(_) => {
                    codegen.compile_stmt(stmt)?;
                }
                _ => {}
            }
        }
        // Compile Store (var) statements — these go into the module's top-level
        // bytecode, before function definitions. They use STORE_GLOBAL with
        // module-qualified keys (e.g. "db.notes").
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::Store(_) = stmt {
                codegen.compile_stmt(stmt)?;
            }
        }
        // Compile Fn, Ext, Use.
        for stmt in &ast.stmts {
            match stmt {
                crate::ast::Stmt::Fn(_)
                | crate::ast::Stmt::Ext(_)
                | crate::ast::Stmt::Use(_) => {
                    codegen.compile_stmt(stmt)?;
                }
                _ => {}
            }
        }



        codegen.code.push(OpCode::HALT as u8);

        // Plan 346: Merge this module's generic_registry + object pools into
        // the session so they can be combined with the main module's at link time.
        self.dep_generic_registry.borrow_mut().merge(&codegen.generic_registry);
        self.dep_object_keys.borrow_mut().extend(codegen.object_keys.iter().cloned());
        self.dep_object_types.borrow_mut().extend(codegen.object_types.iter().cloned());

        Ok(codegen.finish(module_name))

    }



    /// Compile source code and index it into the database

    ///

    /// This is the main entry point for the new architecture.

    /// It parses the source code and indexes all declarations into the database.

    ///

    /// # Arguments

    ///

    /// * `source` - The AutoLang source code to compile

    /// * `path` - The file path (for error reporting and identification)

    ///

    /// # Returns

    ///

    /// A list of fragment IDs that were created.

    ///

    /// # Example

    ///

    /// ```rust,no_run

    /// use auto_lang::compile::CompileSession;

    ///

    /// let mut session = CompileSession::new();

    /// let source = r#"

    ///     fn add(a int, b int) int {

    ///         a + b

    ///     }

    ///

    ///     fn main() int {

    ///         add(10, 20)

    ///     }

    /// "#;

    ///

    /// let frag_ids = session.compile_source(source, "test.at").unwrap();

    /// println!("Indexed {} fragments", frag_ids.len());

    /// ```

    pub fn compile_source(

        &mut self,

        source: &str,

        path: &str,

    ) -> AutoResult<Vec<crate::database::FragId>> {

        // Insert source into database

        let file_id = self.db.write().unwrap().insert_source(path, AutoStr::from(source));



        // Parse source code to AST

        // Note: Phase 1.2 will make parser pure, but for now we use existing parser

        let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));

        let mut parser = Parser::from(source);

        let ast = parser.parse()

            .map_err(|e| crate::error::attach_source(e, path.to_string(), source.to_string()))?;



        // Index AST into database

        let mut db = self.db.write().unwrap();

        let mut indexer = Indexer::new(&mut db);

        let frag_ids = indexer.index_ast(&ast, file_id)

            .map_err(|e| AutoError::Msg(format!("Index error: {}", e)))?;



        Ok(frag_ids)

    }



    /// Compile a file and index it into the database

    ///

    /// # Arguments

    ///

    /// * `path` - Path to the .at file to compile

    ///

    /// # Returns

    ///

    /// A list of fragment IDs that were created.

    pub fn compile_file(

        &mut self,

        path: &str,

    ) -> AutoResult<Vec<crate::database::FragId>> {

        // Read file

        let source = std::fs::read_to_string(path)

            .map_err(|e| AutoError::Io(format!("Failed to read file: {}", e)))?;



        self.compile_source(&source, path)

    }



    /// Get a fragment by name

    ///

    /// This demonstrates querying the database for a specific function.

    ///

    /// # Arguments

    ///

    /// * `name` - The name of the function/fragment to find

    ///

    /// # Returns

    ///

    /// The fragment AST if found.

    pub fn get_fragment_by_name(

        &self,

        name: &str,

    ) -> Option<Arc<crate::ast::Fn>> {

        // Search all fragments for one with matching name

        let db = self.db.read().unwrap();

        for file_id in db.get_files() {

            for frag_id in db.get_fragments_in_file(file_id) {

                if let Some(meta) = db.get_fragment_meta(&frag_id) {

                    if meta.name.as_ref() == name {

                        return db.get_fragment(&frag_id);

                    }

                }

            }

        }

        None

    }



    /// Get symbol location for a function (for LSP support)

    ///

    /// # Arguments

    ///

    /// * `name` - The name of the symbol

    ///

    /// # Returns

    ///

    /// The symbol location if found.

    pub fn get_symbol_location(&self, name: &str) -> Option<SymbolLocation> {

        let symbol_id = Sid::kid_of(&SID_PATH_GLOBAL, name);

        self.db.read().unwrap().get_symbol_location(&symbol_id).cloned()

    }



    /// List all functions in the database

    ///

    /// # Returns

    ///

    /// A list of function names.

    pub fn list_functions(&self) -> Vec<String> {

        let mut functions = Vec::new();



        let db = self.db.read().unwrap();

        for file_id in db.get_files() {

            for frag_id in db.get_fragments_in_file(file_id) {

                if let Some(meta) = db.get_fragment_meta(&frag_id) {

                    if matches!(meta.kind, crate::database::FragKind::Function) {

                        functions.push(meta.name.to_string());

                    }

                }

            }

        }



        functions.sort();

        functions.dedup();

        functions

    }



    /// Clear all data from the database

    ///

    /// **Plan 065 Phase 3**: Also resets QueryEngine to clear cache

    pub fn clear(&mut self) {

        self.db = Arc::new(RwLock::new(Database::new()));

        self.query_engine = None; // Reset QueryEngine to clear cache

        self.type_store = Arc::new(RwLock::new(TypeStore::new())); // Plan 085

        self.auto_cache.clear(); // Plan 085 Phase 5

    }



    /// Get statistics about the database

    pub fn stats(&self) -> CompileStats {

        let mut total_frags = 0;

        let mut total_functions = 0;

        let mut total_specs = 0;



        let db = self.db.read().unwrap();

        for file_id in db.get_files() {

            let frags = db.get_fragments_in_file(file_id);

            total_frags += frags.len();



            for frag_id in &frags {

                if let Some(meta) = db.get_fragment_meta(frag_id) {

                    match meta.kind {

                        crate::database::FragKind::Function => total_functions += 1,

                        crate::database::FragKind::Spec => total_specs += 1,

                        _ => {}

                    }

                }

            }

        }



        CompileStats {

            total_files: db.get_files().len(),

            total_frags,

            total_functions,

            total_specs,

        }

    }



    /// Re-index a file with new source content (incremental compilation)

    ///

    /// This method updates a file's content in the database and re-indexes it.

    /// If the file hash hasn't changed, no recompilation occurs (empty result).

    ///

    /// # Arguments

    ///

    /// * `path` - The file path to re-index

    /// * `source` - The new source content

    ///

    /// # Returns

    ///

    /// A list of new fragment IDs if recompiled, empty if unchanged.

    ///

    /// # Example

    ///

    /// ```rust,no_run

    /// use auto_lang::compile::CompileSession;

    ///

    /// let mut session = CompileSession::new();

    /// session.compile_source("fn main() int { 42 }", "test.at").unwrap();

    ///

    /// // Re-index with same content (no recompilation)

    /// let frags = session.reindex_source("test.at", "fn main() int { 42 }").unwrap();

    /// assert!(frags.is_empty());

    ///

    /// // Re-index with changed content (recompiles)

    /// let frags = session.reindex_source("test.at", "fn main() int { 100 }").unwrap();

    /// assert_eq!(frags.len(), 1);

    /// ```

    pub fn reindex_source(

        &mut self,

        path: &str,

        source: &str,

    ) -> AutoResult<Vec<crate::database::FragId>> {

        // Update source content (insert_source updates if file exists)

        self.db.write().unwrap().insert_source(path, AutoStr::from(source));



        // Get file ID

        let file_id = self.db.read().unwrap().get_file_id_by_path(path)

            .ok_or_else(|| AutoError::Msg(format!("File not found: {}", path)))?;



        // Re-index using indexer

        let mut db = self.db.write().unwrap();

        let mut indexer = Indexer::new(&mut db);

        let frag_ids = indexer.reindex_file(file_id, source)

            .map_err(|e| AutoError::Msg(format!("Reindex error: {}", e)))?;



        Ok(frag_ids)

    }

}



impl Default for CompileSession {

    fn default() -> Self {

        Self::new()

    }

}



/// Statistics about a compilation session

#[derive(Debug, Clone)]

pub struct CompileStats {

    pub total_files: usize,

    pub total_frags: usize,

    pub total_functions: usize,

    pub total_specs: usize,

}



// =============================================================================

// Convenience Functions

// =============================================================================



/// Compile source code in a single call (convenience function)

///

/// This creates a temporary CompileSession, compiles the source,

/// and returns the session for further queries.

///

/// # Example

///

/// ```rust,no_run

/// use auto_lang::compile::compile_once;

///

/// let session = compile_once("fn main() int { 42 }", "test.at").unwrap();

/// let main_fn = session.get_fragment_by_name("main").unwrap();

/// println!("Found main function: {}", main_fn.name);

/// ```

pub fn compile_once(source: &str, path: &str) -> AutoResult<CompileSession> {

    let mut session = CompileSession::new();

    session.compile_source(source, path)?;

    Ok(session)

}



/// Compile a file in a single call (convenience function)

pub fn compile_file_once(path: &str) -> AutoResult<CompileSession> {

    let mut session = CompileSession::new();

    session.compile_file(path)?;

    Ok(session)

}



// =============================================================================

// Tests

// =============================================================================



#[cfg(test)]

mod tests {

    use super::*;



    #[test]

    fn test_compile_session_new() {

        let session = CompileSession::new();

        assert_eq!(session.stats().total_files, 0);

        assert_eq!(session.stats().total_frags, 0);

    }



    #[test]

    fn test_compile_source_simple() {

        let mut session = CompileSession::new();

        let source = "fn main() int { 42 }";



        let result = session.compile_source(source, "test.at");

        assert!(result.is_ok());



        let frag_ids = result.unwrap();

        assert_eq!(frag_ids.len(), 1);



        let stats = session.stats();

        assert_eq!(stats.total_files, 1);

        assert_eq!(stats.total_functions, 1);

    }



    #[test]

    fn test_get_fragment_by_name() {

        let mut session = CompileSession::new();

        let source = "fn add(a int, b int) int { a + b }\nfn main() int { add(10, 20) }";



        session.compile_source(source, "test.at").unwrap();



        // Should find main function

        let main_fn = session.get_fragment_by_name("main");

        assert!(main_fn.is_some());

        assert_eq!(main_fn.unwrap().name.as_ref(), "main");



        // Should find add function

        let add_fn = session.get_fragment_by_name("add");

        assert!(add_fn.is_some());

        assert_eq!(add_fn.unwrap().name.as_ref(), "add");



        // Should not find non-existent function

        let missing_fn = session.get_fragment_by_name("missing");

        assert!(missing_fn.is_none());

    }



    #[test]

    fn test_list_functions() {

        let mut session = CompileSession::new();

        // Functions must be on separate lines for the parser

        let source = "fn foo() int { 1 }\nfn bar() int { 2 }\nfn baz() int { 3 }";



        let frag_ids = session.compile_source(source, "test.at").unwrap();

        println!("Fragment IDs created: {:?}", frag_ids);

        println!("Stats: {:?}", session.stats());



        let functions = session.list_functions();

        println!("Functions found: {:?}", functions);



        assert_eq!(functions.len(), 3);

        assert!(functions.contains(&"foo".to_string()));

        assert!(functions.contains(&"bar".to_string()));

        assert!(functions.contains(&"baz".to_string()));

    }



    #[test]

    fn test_compile_multiple_files() {

        let mut session = CompileSession::new();



        // Compile first file

        let source1 = "fn foo() int { 1 }";

        session.compile_source(source1, "file1.at").unwrap();



        // Compile second file

        let source2 = "fn bar() int { 2 }";

        session.compile_source(source2, "file2.at").unwrap();



        let stats = session.stats();

        assert_eq!(stats.total_files, 2);

        assert_eq!(stats.total_functions, 2);



        // Both functions should be accessible

        assert!(session.get_fragment_by_name("foo").is_some());

        assert!(session.get_fragment_by_name("bar").is_some());

    }



    #[test]

    fn test_clear() {

        let mut session = CompileSession::new();

        session.compile_source("fn test() int { 1 }", "test.at").unwrap();



        assert_eq!(session.stats().total_functions, 1);



        session.clear();



        assert_eq!(session.stats().total_files, 0);

        assert_eq!(session.stats().total_functions, 0);

        assert!(session.get_fragment_by_name("test").is_none());

    }



    #[test]

    fn test_compile_once_convenience() {

        let source = "fn main() int { 42 }";

        let result = compile_once(source, "test.at");



        assert!(result.is_ok());

        let session = result.unwrap();

        assert_eq!(session.stats().total_functions, 1);

        assert!(session.get_fragment_by_name("main").is_some());

    }



    #[test]

    fn test_empty_source() {

        let mut session = CompileSession::new();

        let source = "";



        let result = session.compile_source(source, "test.at");

        assert!(result.is_ok());



        let frag_ids = result.unwrap();

        assert_eq!(frag_ids.len(), 0);

        assert_eq!(session.stats().total_frags, 0);

    }



    #[test]

    fn test_stats() {

        let mut session = CompileSession::new();

        let source = "fn foo() int { 1 }\nspec MySpec { fn test() void }";



        session.compile_source(source, "test.at").unwrap();



        let stats = session.stats();

        assert_eq!(stats.total_files, 1);

        assert_eq!(stats.total_frags, 2);  // 1 function + 1 spec

        assert_eq!(stats.total_functions, 1);

        assert_eq!(stats.total_specs, 1);

    }



    // =============================================================================

    // Phase 2.5: Incremental Compilation Tests

    // =============================================================================



    #[test]

    fn test_file_no_change() {

        // Test: No recompilation if file unchanged

        let mut session = CompileSession::new();

        let source = "fn main() int { 42 }";



        // First compilation

        let frag_ids1 = session.compile_source(source, "test.at").unwrap();

        assert_eq!(frag_ids1.len(), 1);



        // Get file ID and initial hash

        let file_id = session.database().unwrap().get_file_id_by_path("test.at").unwrap();

        let hash1 = session.database_mut().unwrap().hash_file(file_id).unwrap();



        // Re-index same content (should skip)

        let frags = session.reindex_source("test.at", source).unwrap();



        // Should not recompile (no fragments returned)

        assert!(frags.is_empty());



        // Hash should be unchanged

        let hash2 = session.database_mut().unwrap().hash_file(file_id).unwrap();

        assert_eq!(hash1, hash2);



        // File should not be dirty

        assert!(!session.database().unwrap().is_file_dirty(file_id));

    }



    #[test]

    fn test_file_changed() {

        // Test: Only changed file recompiled

        let mut session = CompileSession::new();

        let source1 = "fn main() int { 42 }";



        // First compilation

        let frag_ids1 = session.compile_source(source1, "test.at").unwrap();

        assert_eq!(frag_ids1.len(), 1);



        // Get file ID and initial hash

        let file_id = session.database().unwrap().get_file_id_by_path("test.at").unwrap();

        let hash1 = session.database_mut().unwrap().hash_file(file_id).unwrap();



        // Change source and re-index

        let source2 = "fn main() int { 100 }";

        let frags = session.reindex_source("test.at", source2).unwrap();



        // Should return new fragments (recompiled)

        assert_eq!(frags.len(), 1);



        // Hash should be changed

        let hash2 = session.database_mut().unwrap().hash_file(file_id).unwrap();

        assert_ne!(hash1, hash2);



        // File should not be dirty after re-indexing

        assert!(!session.database().unwrap().is_file_dirty(file_id));

    }



    #[test]

    fn test_import_chain() {

        // Test: A imports B, B changes 鈫?A recompiled

        let mut session = CompileSession::new();



        // Compile B first (dependency)

        let source_b = "fn foo() int { 42 }";

        session.compile_source(source_b, "std/b.at").unwrap();



        // Compile A

        let source_a = "fn main() int { 42 }";

        session.compile_source(source_a, "test.a.at").unwrap();



        // Get file IDs

        let (file_b, file_a) = {

            let db = session.database().unwrap();

            let fb = db.get_file_id_by_path("std/b.at").unwrap();

            let fa = db.get_file_id_by_path("test.a.at").unwrap();

            (fb, fa)

        };



        // Manually add dependency: A imports B

        session.database_mut().unwrap().dep_graph_mut().add_file_import(file_a, vec![file_b]);



        // Check dependency: A imports B

        {

            let db = session.database().unwrap();

            let deps_a = db.dep_graph().get_file_imports(file_a);

            assert_eq!(deps_a.len(), 1);

            assert!(deps_a.contains(&file_b));

        }



        // Modify B

        let source_b_new = "fn foo() int { 100 }";

        session.reindex_source("std/b.at", source_b_new).unwrap();



        // Mark B dirty and propagate

        session.database_mut().unwrap().mark_file_dirty(file_b);

        session.database_mut().unwrap().propagate_dirty_recursive(file_b);



        // A should be dirty (depends on B)

        {

            let db = session.database().unwrap();

            assert!(db.is_file_dirty(file_a));

        }

    }



    #[test]

    fn test_import_diamond() {

        // Test: A,B import C, C changes 鈫?A,B recompiled

        let mut session = CompileSession::new();



        // Compile C (shared dependency)

        let source_c = "fn shared() int { 42 }";

        session.compile_source(source_c, "std/c.at").unwrap();



        // Compile A

        let source_a = "fn func_a() int { 42 }";

        session.compile_source(source_a, "test/a.at").unwrap();



        // Compile B

        let source_b = "fn func_b() int { 42 }";

        session.compile_source(source_b, "test/b.at").unwrap();



        // Get file IDs

        let (file_c, file_a, file_b) = {

            let db = session.database().unwrap();

            let fc = db.get_file_id_by_path("std/c.at").unwrap();

            let fa = db.get_file_id_by_path("test/a.at").unwrap();

            let fb = db.get_file_id_by_path("test/b.at").unwrap();

            (fc, fa, fb)

        };



        // Manually add dependencies: A imports C, B imports C

        session.database_mut().unwrap().dep_graph_mut().add_file_import(file_a, vec![file_c]);

        session.database_mut().unwrap().dep_graph_mut().add_file_import(file_b, vec![file_c]);



        // Verify diamond dependencies

        {

            let db = session.database().unwrap();

            let deps_a = db.dep_graph().get_file_imports(file_a);

            let deps_b = db.dep_graph().get_file_imports(file_b);

            assert!(deps_a.contains(&file_c));

            assert!(deps_b.contains(&file_c));

        }



        // Modify C (this will mark C as dirty, propagate to A and B, then clear C's dirty flag)

        let source_c_new = "fn shared() int { 100 }";

        session.reindex_source("std/c.at", source_c_new).unwrap();



        // Both A and B should be dirty (dependents of C)

        {

            let db = session.database().unwrap();

            assert!(db.is_file_dirty(file_a));

            assert!(db.is_file_dirty(file_b));

        }



        // C should not be dirty (cleared after re-index)

        {

            let db = session.database().unwrap();

            assert!(!db.is_file_dirty(file_c));

        }

    }



    // =============================================================================

    // Phase 3.2: Fragment Hash鐔旀柇Tests

    // =============================================================================



    #[test]

    fn test_fragment_iface_hash_storage() {

        // Test: Fragment interface hashes are computed and stored

        let mut session = CompileSession::new();

        let source = "fn add(a int, b int) int { a + b }";



        session.compile_source(source, "test.at").unwrap();



        // Get the fragment

        let frag_id = session.database().unwrap().get_fragments_in_file(

            session.database().unwrap().get_file_id_by_path("test.at").unwrap()

        ).into_iter().next().unwrap();



        // Verify interface hash was computed and stored

        let hash = session.database().unwrap().get_fragment_iface_hash(&frag_id);

        assert!(hash.is_some(), "Interface hash should be stored");

        assert_ne!(hash.unwrap(), 0, "Interface hash should not be zero");

    }



    #[test]

    fn test_interface_hash_unchanged_body_change() {

        // Test鐔旀柇: Function body change doesn't change interface hash

        let mut session = CompileSession::new();



        // Initial version

        let source_v1 = "fn add(a int, b int) int { a + b }";

        session.compile_source(source_v1, "test.at").unwrap();



        // Get initial interface hash

        let file_id = session.database().unwrap().get_file_id_by_path("test.at").unwrap();

        let frag_id_v1 = session.database().unwrap().get_fragments_in_file(file_id)[0].clone();

        let hash_v1 = session.database().unwrap().get_fragment_iface_hash(&frag_id_v1).unwrap();



        // Re-index with changed body but same signature

        let source_v2 = "fn add(a int, b int) int { a + b + 0 }";  // Body changed

        session.reindex_source("test.at", source_v2).unwrap();



        // Get new interface hash

        let frag_id_v2 = session.database().unwrap().get_fragments_in_file(file_id)[0].clone();

        let hash_v2 = session.database().unwrap().get_fragment_iface_hash(&frag_id_v2).unwrap();



        // Interface hash should be UNCHANGED (鐔旀柇!)

        assert_eq!(hash_v1, hash_v2, "Interface hash should be unchanged when only body changes");

    }



    #[test]

    fn test_interface_hash_changed_signature_change() {

        // Test: Signature change DOES change interface hash

        let mut session = CompileSession::new();



        // Initial version

        let source_v1 = "fn add(a int, b int) int { a + b }";

        session.compile_source(source_v1, "test.at").unwrap();



        // Get initial interface hash

        let file_id = session.database().unwrap().get_file_id_by_path("test.at").unwrap();

        let frag_id_v1 = session.database().unwrap().get_fragments_in_file(file_id)[0].clone();

        let hash_v1 = session.database().unwrap().get_fragment_iface_hash(&frag_id_v1).unwrap();



        // Re-index with changed signature

        let source_v2 = "fn add(a int, b int, c int) int { a + b + c }";  // Signature changed!

        session.reindex_source("test.at", source_v2).unwrap();



        // Get new interface hash

        let frag_id_v2 = session.database().unwrap().get_fragments_in_file(file_id)[0].clone();

        let hash_v2 = session.database().unwrap().get_fragment_iface_hash(&frag_id_v2).unwrap();



        // Interface hash should be CHANGED

        assert_ne!(hash_v1, hash_v2, "Interface hash should change when signature changes");

    }



    // =============================================================================

    // Plan 167: Circular Dependency Detection Tests

    // =============================================================================



    #[test]

    fn test_circular_dependency_detected() {

        // Test that loading_stack detects cycles when load_module is called

        // with the same module name while it's already being loaded.

        // This verifies the infrastructure is in place for Phase 4 (recursive loading).

        let mut session = CompileSession::new();



        // Simulate a cycle: push "a" onto the loading stack, then try to load "a"

        session.loading_stack.push("b".to_string());

        session.loading_stack.push("a".to_string());



        let use_a = UseStatement::new("a".to_string());

        let result = session.load_module(&use_a);



        // Plan 327: circular deps are now allowed (skip, not error) —
        // legitimate cross-references (db use api: Note + api use db) need
        // this. The module being loaded ("a") is already in loading_stack,
        // so load_module returns Ok(()) without re-loading.
        assert!(result.is_ok(), "Circular dependency should be skipped (Ok), not error. Got: {:?}", result);
    }



    #[test]

    fn test_no_circular_dependency() {

        let tmp = tempfile::TempDir::new().unwrap();



        // Create module c.at that uses d (no cycle)

        let c_path = tmp.path().join("c.at");

        std::fs::write(&c_path, "fn c_func() int { 1 }").unwrap();



        let d_path = tmp.path().join("d.at");

        std::fs::write(&d_path, "fn d_func() int { 2 }").unwrap();



        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(tmp.path()).unwrap();



        let mut session = CompileSession::new();

        let use_c = UseStatement::new("c".to_string());

        let result = session.load_module(&use_c);



        std::env::set_current_dir(&original_dir).unwrap();



        // Should succeed 鈥?no cycle

        assert!(result.is_ok(), "Expected success, got error: {:?}", result.err());

    }

}

