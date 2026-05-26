// Plan 081 Phase 5: FFI Layer for Cross-Mode Function Calls
//
// This module implements the FFI (Foreign Function Interface) layer that enables
// AutoVM bytecode to call functions from C-transpiled modules and vice versa.
//
// **Architecture**:
// AutoVM bytecode → CALL_NAT → Native Shim → C FFI Bridge → C Library
//
// This enables mixed-mode projects where:
// - Main code runs in AutoVM
// - HAL/low-level code is transpiled to C
// - AutoVM can call C functions directly

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// FFI bridge for C-transpiled modules
///
/// **Plan 081 Phase 5**: Manages loading C libraries and registering their functions
/// so they can be called from AutoVM bytecode via CALL_NAT opcode.
///
/// ## Workflow
///
/// 1. **Compilation Phase**:
///    ```text
///    // hal.at transpiled to C
///    #[c]
///    fn gpio_init(pin int) int;
///    ```
///
/// 2. **Registration Phase**:
///    ```ignore
///    ffi_bridge.register_c_function(
//     "hal",
///     "gpio_init",
///     c_signature!("int(int)"),
///     library_path: "target/hal.dll"
///    );
///    ```
///
/// 3. **Code Generation Phase**:
///    ```ignore
///    // When AutoVM sees extern "c" { gpio_init(pin) }
///    // It generates: CALL_NAT <native_id_for_gpio_init>
///    ```
///
/// 4. **Execution Phase**:
///    - AutoVM executes CALL_NAT
///    - Calls native shim registered with this bridge
///    - Shim converts AutoVM values → C arguments
///    - Calls C function via FFI
///    - Converts C return value → AutoVM value
pub struct CFfiBridge {
    /// Registered C functions
    /// Maps (library, function_name) → native_id
    functions: HashMap<(String, String), u16>,

    /// Loaded C libraries
    /// Maps library name -> library handle (for future libloading support)
    libraries: HashMap<String, libloading::Library>,

    /// Native interface for registering shims
    /// Note: We use NativeInterface directly (not Arc) so we can mutate it
    /// The Arc wrapping happens when it's passed to AutoVM
    native_interface: crate::vm::native::NativeInterface,

    /// Next available native ID for C functions
    /// IDs 200+ reserved for C FFI functions (100-199 for Rust FFI)
    next_native_id: u16,
}

impl CFfiBridge {
    /// Create a new C FFI bridge
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            libraries: HashMap::new(),
            native_interface: crate::vm::native::NativeInterface::new(),
            next_native_id: 200, // Reserve 200+ for C functions
        }
    }

    /// Get the native interface for registering shims
    pub fn native_interface(&self) -> &crate::vm::native::NativeInterface {
        &self.native_interface
    }

    /// Get the native interface as Arc for sharing with AutoVM
    /// Note: This consumes the bridge since we can't clone NativeInterface
    pub fn into_native_interface_arc(self) -> Arc<crate::vm::native::NativeInterface> {
        Arc::new(self.native_interface)
    }

    /// Register a C function from a C-transpiled module
    ///
    /// **Plan 081 Phase 5**: Main entry point for registering C FFI functions
    ///
    /// # Arguments
    /// * `library` - Library name (e.g., "hal", "crypto")
    /// * `function_name` - C function name (e.g., "gpio_init")
    /// * `signature` - Function signature for type checking
    /// * `library_path` - Path to compiled C library (.dll/.so)
    ///
    /// # Returns
    /// * `Ok(native_id)` - Assigned native ID for CALL_NAT
    /// * `Err(VMError)` - Registration failed
    ///
    /// # Example
    /// ```ignore
    /// let bridge = CFfiBridge::new();
    ///
    /// // Register C function
    /// let native_id = bridge.register_c_function(
    ///     "hal",
    ///     "gpio_init",
    ///     CSignature::new()
    ///         .param(CType::Int)
    ///         .returns(CType::Int),
    ///     PathBuf::from("target/hal.dll")
    /// )?;
    ///
    /// // Now AutoVM code can call: CALL_NAT native_id
    /// ```
    pub fn register_c_function(
        &mut self,
        library: &str,
        function_name: &str,
        signature: CSignature,
        library_path: PathBuf,
    ) -> Result<u16, VMError> {
        // Check if function already registered
        let key = (library.to_string(), function_name.to_string());
        if let Some(&id) = self.functions.get(&key) {
            return Ok(id);
        }

        // Load the library (for future libloading support)
        if !self.libraries.contains_key(library) {
            // TODO(Plan-212): Implement real C library loading via libloading
            log::info!("Would load C library from: {}", library_path.display());

            // When libloading is implemented:
            // let lib = libloading::Library::new(&library_path)
            //     .map_err(|e| VMError::FFI(format!("Failed to load {}: {}", library_path, e)))?;
            // self.libraries.insert(library.to_string(), lib);
        }

        // Create native shim for this C function
        let native_id = self.next_native_id;
        self.next_native_id += 1;

        // Register the shim with AutoVM
        // The shim will:
        // 1. Pop arguments from AutoVM stack
        // 2. Convert arguments to C types
        // 3. Call C function
        // 4. Convert return value to AutoVM type
        // 5. Push result onto AutoVM stack
        let shim = self.create_c_shim(library, function_name, signature, library_path.clone());

        // Register the shim with AutoVM
        self.native_interface.register(native_id, shim);
        self.functions.insert(key, native_id);

        log::info!(
            "Registered C function: {}::{} (native_id={})",
            library,
            function_name,
            native_id
        );

        Ok(native_id)
    }

    /// Create a native shim for calling a C function
    // TODO(Plan-212): Implement real C FFI — currently a placeholder
    fn create_c_shim(
        &self,
        _library: &str,
        function_name: &str,
        signature: CSignature,
        _library_path: PathBuf,
    ) -> impl Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static {
        let function_name = function_name.to_string();

        move |task: &mut AutoTask, _vm: &AutoVM| {
            // TODO(Plan-212): Call C function via libloading
            let _ = &signature;
            log::warn!(
                "C FFI not yet implemented for {}",
                function_name,
            );

            task.ram.push_i32(0);

            Ok(())
        }
    }

    /// Get the native ID for a registered function
    pub fn get_function_id(&self, library: &str, function_name: &str) -> Option<u16> {
        self.functions
            .get(&(library.to_string(), function_name.to_string()))
            .copied()
    }

    /// Get all registered functions
    pub fn get_functions(&self) -> &HashMap<(String, String), u16> {
        &self.functions
    }
}

/// C function signature for FFI
///
/// Describes the parameter and return types of a C function
/// for proper argument marshaling.
#[derive(Debug, Clone, PartialEq)]
pub struct CSignature {
    pub params: Vec<CType>,
    pub returns: CType,
}

impl CSignature {
    /// Create a new C signature
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
            returns: CType::Void,
        }
    }

    /// Add a parameter
    pub fn param(mut self, param_type: CType) -> Self {
        self.params.push(param_type);
        self
    }

    /// Set return type
    pub fn returns(mut self, return_type: CType) -> Self {
        self.returns = return_type;
        self
    }
}

/// C type for FFI marshaling
///
/// **Plan 081 Phase 5**: Types that can be converted between AutoVM and C
#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    /// Signed 32-bit integer
    Int,
    /// 32-bit floating point
    Float,
    /// Null-terminated string
    Str,
    /// Void (no return value)
    Void,
}

/// C value for FFI marshaling
///
/// Represents a value that can be passed to/from C code
#[derive(Debug, Clone, PartialEq)]
pub enum CValue {
    Int(i32),
    Float(f32),
    Str(String),
    Void,
}

/// FFI function registry for managing multiple FFI bridges
///
/// **Plan 081 Phase 5**: Global registry for all FFI operations
///
/// Maintains a singleton registry that can be accessed from anywhere
/// to register or look up FFI functions.
pub struct FfiRegistry {
    bridges: HashMap<String, CFfiBridge>,
}

impl FfiRegistry {
    /// Create a new FFI registry
    pub fn new() -> Self {
        Self {
            bridges: HashMap::new(),
        }
    }

    /// Get or create an FFI bridge for a library
    pub fn get_bridge(&mut self, library: &str) -> &mut CFfiBridge {
        if !self.bridges.contains_key(library) {
            let bridge = CFfiBridge::new();
            self.bridges.insert(library.to_string(), bridge);
        }
        self.bridges.get_mut(library).unwrap()
    }

    /// Register a C function across all bridges
    pub fn register_c_function(
        &mut self,
        library: &str,
        function_name: &str,
        signature: CSignature,
        library_path: PathBuf,
    ) -> Result<u16, VMError> {
        let bridge = self.get_bridge(library);
        bridge.register_c_function(library, function_name, signature, library_path)
    }

    /// Get the native ID for a C function
    pub fn get_function_id(&self, library: &str, function_name: &str) -> Option<u16> {
        self.bridges
            .get(library)
            .and_then(|bridge| bridge.get_function_id(library, function_name))
    }
}

// Global FFI registry instance
lazy_static::lazy_static! {
    pub static ref FFI_REGISTRY: std::sync::Mutex<FfiRegistry> =
        std::sync::Mutex::new(FfiRegistry::new());
}

/// Helper function to register a C function from AutoVM
///
/// **Plan 081 Phase 5**: Convenience function for codegen to use when
/// it encounters an `extern "c"` declaration.
///
/// # Example
/// ```ignore
/// // In AutoLang code:
/// extern "c" {
///     fn hal_gpio_init(pin int) int;
/// }
///
/// // Codegen would call:
/// let native_id = register_extern_c_function(
///     "hal",
///     "gpio_init",
///     CSignature::new().param(CType::Int).returns(CType::Int),
///     PathBuf::from("target/hal.dll")
/// )?;
/// ```
pub fn register_extern_c_function(
    library: &str,
    function_name: &str,
    signature: CSignature,
    library_path: PathBuf,
) -> Result<u16, VMError> {
    FFI_REGISTRY
        .lock()
        .map_err(|_| VMError::RuntimeError("FFI registry poisoned".to_string()))?
        .register_c_function(library, function_name, signature, library_path)
}

// =============================================================================
// Plan 092: Rust FFI Bridge with Sandbox Integration
// =============================================================================

use auto_cache::{CrateMetadata, CrateRegistry, Sandbox};

/// Rust FFI Bridge with Sandbox integration
///
/// **Plan 092**: Enables loading Rust crates dynamically with ABI safety.
///
/// This bridge:
/// 1. Uses Sandbox to manage compiled Rust crates
/// 2. Verifies ABI compatibility before loading
/// 3. Registers crate functions as native functions
pub struct RustFfiBridge {
    /// The sandbox managing Rust crate compilation
    sandbox: Sandbox,

    /// Crate registry for metadata
    registry: Option<CrateRegistry>,

    /// Loaded libraries (library name -> Library handle)
    /// Wrapped in Arc for sharing with shim closures
    loaded_libraries: HashMap<String, Arc<libloading::Library>>,

    /// Registered functions (crate::function -> native_id)
    functions: HashMap<String, u16>,

    /// Next native ID (use 300+ for dynamically loaded Rust crates)
    next_native_id: u16,

    /// Native interface for registering shims
    native_interface: crate::vm::native::NativeInterface,
}

/// Resolve a cdylib symbol and call it, returning u64 (Phase 3B)
/// Usage: ffi_call!(lib, "name", SymbolType, arg1, arg2, ...)
macro_rules! ffi_call {
    // 0 args
    ($lib:expr, $name:expr, $sym_ty:ty) => {{
        let sym: libloading::Symbol<$sym_ty> = $lib
            .get($name.as_bytes())
            .map_err(|e| VMError::FFI(format!("Symbol {} not found: {}", $name, e)))?;
        sym() as u64
    }};
    // 1 arg
    ($lib:expr, $name:expr, $sym_ty:ty, $a1:expr) => {{
        let sym: libloading::Symbol<$sym_ty> = $lib
            .get($name.as_bytes())
            .map_err(|e| VMError::FFI(format!("Symbol {} not found: {}", $name, e)))?;
        sym($a1) as u64
    }};
    // 2 args
    ($lib:expr, $name:expr, $sym_ty:ty, $a1:expr, $a2:expr) => {{
        let sym: libloading::Symbol<$sym_ty> = $lib
            .get($name.as_bytes())
            .map_err(|e| VMError::FFI(format!("Symbol {} not found: {}", $name, e)))?;
        sym($a1, $a2) as u64
    }};
    // 3 args
    ($lib:expr, $name:expr, $sym_ty:ty, $a1:expr, $a2:expr, $a3:expr) => {{
        let sym: libloading::Symbol<$sym_ty> = $lib
            .get($name.as_bytes())
            .map_err(|e| VMError::FFI(format!("Symbol {} not found: {}", $name, e)))?;
        sym($a1, $a2, $a3) as u64
    }};
}

/// Extract a typed pointer from RustValue (Phase 3B)
fn ptr_arg(val: &RustValue) -> *const () {
    match val {
        RustValue::Ptr(p) => *p,
        _ => std::ptr::null(),
    }
}

impl RustFfiBridge {
    /// Create a new Rust FFI bridge with sandbox
    pub fn new() -> Result<Self, VMError> {
        let sandbox =
            Sandbox::new().map_err(|e| VMError::FFI(format!("Failed to create sandbox: {}", e)))?;

        Ok(Self {
            sandbox,
            registry: None,
            loaded_libraries: HashMap::new(),
            functions: HashMap::new(),
            next_native_id: 300, // Reserve 300+ for dynamic Rust FFI
            native_interface: crate::vm::native::NativeInterface::new(),
        })
    }

    /// Create bridge with registry
    pub fn with_registry(registry_path: &Path) -> Result<Self, VMError> {
        let mut bridge = Self::new()?;
        let registry = CrateRegistry::new(registry_path)
            .map_err(|e| VMError::FFI(format!("Failed to create registry: {}", e)))?;
        bridge.registry = Some(registry);
        Ok(bridge)
    }

    /// Get the sandbox
    pub fn sandbox(&self) -> &Sandbox {
        &self.sandbox
    }

    /// Get the native interface
    pub fn native_interface(&self) -> &crate::vm::native::NativeInterface {
        &self.native_interface
    }

    /// Get native interface as Arc
    pub fn into_native_interface_arc(self) -> Arc<crate::vm::native::NativeInterface> {
        Arc::new(self.native_interface)
    }

    /// Load a Rust crate through the sandbox
    ///
    /// # Arguments
    /// * `crate_name` - Name of the crate (e.g., "serde", "serde_json")
    /// * `version` - Version string (e.g., "1.0.193") or "latest"
    ///
    /// # Returns
    /// * `Ok(())` - Crate loaded successfully
    /// * `Err(VMError)` - Loading failed
    pub fn load_rust_crate(&mut self, crate_name: &str, version: &str) -> Result<(), VMError> {
        // Check if already loaded
        if self.loaded_libraries.contains_key(crate_name) {
            log::debug!("Crate {} already loaded", crate_name);
            return Ok(());
        }

        // Check registry for metadata (if available)
        if let Some(ref registry) = self.registry {
            if let Some(metadata) = registry
                .lookup(crate_name)
                .map_err(|e| VMError::FFI(format!("Registry error: {}", e)))?
            {
                // Verify ABI compatibility
                self.sandbox
                    .verify_abi(&metadata)
                    .map_err(|e| VMError::FFI(format!("ABI error: {}", e)))?;
            }
        }

        // Load the library
        let library = unsafe {
            self.sandbox
                .load_crate(crate_name, version)
                .map_err(|e| VMError::FFI(format!("Failed to load crate: {}", e)))?
        };

        self.loaded_libraries
            .insert(crate_name.to_string(), Arc::new(library));
        log::info!("Loaded Rust crate: {} v{}", crate_name, version);

        Ok(())
    }

    /// Load a Rust crate from a pre-compiled library file
    ///
    /// This is useful for user libraries that are compiled separately.
    pub fn load_rust_library(
        &mut self,
        crate_name: &str,
        library_path: &Path,
    ) -> Result<(), VMError> {
        // Check if already loaded
        if self.loaded_libraries.contains_key(crate_name) {
            return Ok(());
        }

        let library = unsafe {
            libloading::Library::new(library_path).map_err(|e| {
                VMError::FFI(format!("Failed to load {}: {}", library_path.display(), e))
            })?
        };

        self.loaded_libraries
            .insert(crate_name.to_string(), Arc::new(library));
        log::info!(
            "Loaded Rust library: {} from {}",
            crate_name,
            library_path.display()
        );

        Ok(())
    }

    /// Register a function from a loaded crate
    ///
    /// # Arguments
    /// * `crate_name` - Name of the loaded crate
    /// * `function_name` - Name of the function to register
    /// * `signature` - Function signature for marshaling
    ///
    /// # Returns
    /// * `Ok(native_id)` - Assigned native ID for CALL_NAT
    pub fn register_function(
        &mut self,
        crate_name: &str,
        function_name: &str,
        signature: RustSignature,
    ) -> Result<u16, VMError> {
        let key = format!("{}::{}", crate_name, function_name);

        // Check if already registered
        if let Some(&id) = self.functions.get(&key) {
            return Ok(id);
        }

        // Get the loaded library
        let library = self
            .loaded_libraries
            .get(crate_name)
            .ok_or_else(|| VMError::FFI(format!("Crate {} not loaded", crate_name)))?;

        // Get the symbol from the library (sandbox wrapper uses auto_ prefix)
        let exported_name = format!("auto_{}", function_name);
        let symbol_name = std::ffi::CString::new(exported_name.as_bytes())
            .map_err(|e| VMError::FFI(format!("Invalid function name: {}", e)))?;

        // Get the symbol as a raw pointer (we'll cast it based on signature)
        // Note: library.get() returns a Symbol whose internal pointer IS the function address.
        // We must not dereference it (which reads machine code bytes as a pointer).
        // Instead, extract the raw platform pointer via into_raw() + as_raw_ptr().
        let _symbol_ptr: *const () = unsafe {
            let symbol = library
                .get::<*const ()>(symbol_name.as_bytes())
                .map_err(|e| VMError::FFI(format!("Symbol {} (exported as {}) not found: {}", function_name, exported_name, e)))?;
            let raw = symbol.into_raw();
            let ptr = raw.as_raw_ptr() as *const ();
            ptr
        };

        let native_id = self.next_native_id;
        self.next_native_id += 1;

        // Create shim for this function — pass library + symbol name for lazy resolution
        let exported_name = format!("auto_{}", function_name);
        let shim = self.create_rust_shim_lazy(
            Arc::clone(library),
            crate_name,
            function_name,
            &exported_name,
            signature.clone(),
        );
        self.native_interface.register(native_id, shim);
        self.functions.insert(key.clone(), native_id);

        log::info!(
            "Registered Rust function: {} (native_id={})",
            key,
            native_id
        );

        Ok(native_id)
    }

    /// Register a function with a custom exported symbol name (Phase 3C-v2 sig_code).
    ///
    /// Like `register_function`, but uses `exported_name` instead of `auto_{function_name}`.
    pub fn register_function_with_export(
        &mut self,
        crate_name: &str,
        function_name: &str,
        exported_name: &str,
        signature: RustSignature,
    ) -> Result<u16, VMError> {
        let key = format!("{}::{}", crate_name, function_name);

        // Check if already registered
        if let Some(&id) = self.functions.get(&key) {
            return Ok(id);
        }

        let library = self
            .loaded_libraries
            .get(crate_name)
            .ok_or_else(|| VMError::FFI(format!("Crate {} not loaded", crate_name)))?;

        let symbol_name = std::ffi::CString::new(exported_name.as_bytes())
            .map_err(|e| VMError::FFI(format!("Invalid function name: {}", e)))?;

        // Verify symbol exists
        let _symbol_ptr: *const () = unsafe {
            let symbol = library
                .get::<*const ()>(symbol_name.as_bytes())
                .map_err(|e| VMError::FFI(format!("Symbol {} not found: {}", exported_name, e)))?;
            let raw = symbol.into_raw();
            raw.as_raw_ptr() as *const ()
        };

        let native_id = self.next_native_id;
        self.next_native_id += 1;

        let shim = self.create_rust_shim_lazy(
            Arc::clone(library),
            crate_name,
            function_name,
            exported_name,
            signature.clone(),
        );
        self.native_interface.register(native_id, shim);
        self.functions.insert(key.clone(), native_id);

        log::info!(
            "Registered Rust function: {} (native_id={}, exported={})",
            key, native_id, exported_name
        );

        Ok(native_id)
    }

    /// Try to load the sig manifest from a loaded crate.
    ///
    /// Calls `auto__sig_manifest()` which returns a JSON string
    /// mapping function names to sig_code strings.
    pub fn load_sig_manifest(&self, crate_name: &str) -> Option<String> {
        let library = self.loaded_libraries.get(crate_name)?;

        type ManifestFn = unsafe extern "C" fn() -> *const std::os::raw::c_char;

        let symbol: Result<libloading::Symbol<ManifestFn>, _> = unsafe {
            library.get(b"auto__sig_manifest\0")
        };

        match symbol {
            Ok(sym) => unsafe {
                let ptr = (*sym)();
                if ptr.is_null() {
                    return None;
                }
                let cstr = std::ffi::CStr::from_ptr(ptr);
                let s = cstr.to_str().ok()?.to_string();
                // Free the CString that was leaked by into_raw()
                let _ = std::ffi::CString::from_raw(ptr as *mut std::os::raw::c_char);
                Some(s)
            },
            Err(_) => None,
        }
    }

    /// Get the native ID for a registered function
    pub fn get_function_id(&self, crate_name: &str, function_name: &str) -> Option<u16> {
        let key = format!("{}::{}", crate_name, function_name);
        self.functions.get(&key).copied()
    }

    /// Get all registered functions
    pub fn get_functions(&self) -> &HashMap<String, u16> {
        &self.functions
    }

    /// Create a native shim that resolves the DLL symbol on each call.
    /// This avoids issues with raw function pointer transmutation across DLL boundaries.
    fn create_rust_shim_lazy(
        &self,
        _library: Arc<libloading::Library>,
        crate_name: &str,
        function_name: &str,
        exported_name: &str,
        signature: RustSignature,
    ) -> impl Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static {
        let crate_name = crate_name.to_string();
        let function_name = function_name.to_string();
        let exported_name = exported_name.to_string();
        let signature = signature.clone();
        let _keep_lib_alive = _library; // Keep library Arc alive for the closure's lifetime

        move |task: &mut AutoTask, vm: &AutoVM| {
            // Pop arguments from stack (in reverse order) into unified RustValue
            let mut string_pool: Vec<std::ffi::CString> = Vec::new();
            let mut args: Vec<RustValue> = Vec::new();
            for param_type in signature.params.iter().rev() {
                args.push(param_type.pop_value(task, vm, &mut string_pool));
            }
            args.reverse();

            let result: u64 = unsafe {
                match (signature.params.as_slice(), &signature.returns) {
                    // () -> primitives
                    (&[], RustType::Long) => {
                        ffi_call!(_keep_lib_alive, exported_name, extern "C" fn() -> i64)
                    }
                    (&[], RustType::Double) => {
                        ffi_call!(_keep_lib_alive, exported_name, extern "C" fn() -> f64)
                    }
                    (&[], RustType::Bool) => {
                        ffi_call!(_keep_lib_alive, exported_name, extern "C" fn() -> bool)
                    }
                    (&[], RustType::Int) => {
                        ffi_call!(_keep_lib_alive, exported_name, extern "C" fn() -> i32)
                    }
                    (&[], RustType::String) => {
                        ffi_call!(
                            _keep_lib_alive,
                            exported_name,
                            extern "C" fn() -> *const std::ffi::c_char
                        )
                    }

                    // (String) -> String
                    (&[RustType::String], RustType::String) => {
                        let a1 = ptr_arg(&args[0]) as *const std::ffi::c_char;
                        ffi_call!(
                            _keep_lib_alive,
                            exported_name,
                            extern "C" fn(*const std::ffi::c_char) -> *const std::ffi::c_char,
                            a1
                        )
                    }

                    // (String) -> Long
                    (&[RustType::String], RustType::Long) => {
                        let a1 = ptr_arg(&args[0]) as *const std::ffi::c_char;
                        ffi_call!(
                            _keep_lib_alive,
                            exported_name,
                            extern "C" fn(*const std::ffi::c_char) -> i64,
                            a1
                        )
                    }

                    // (Long, Long) -> Long
                    (&[RustType::Long, RustType::Long], RustType::Long) => {
                        let a1 = match &args[0] {
                            RustValue::Long(v) => *v,
                            _ => 0,
                        };
                        let a2 = match &args[1] {
                            RustValue::Long(v) => *v,
                            _ => 0,
                        };
                        ffi_call!(
                            _keep_lib_alive,
                            exported_name,
                            extern "C" fn(i64, i64) -> i64,
                            a1,
                            a2
                        )
                    }

                    // Unsupported signature
                    _ => {
                        log::warn!(
                            "Unsupported FFI signature: {:?} -> {:?} for {}::{}",
                            signature.params,
                            signature.returns,
                            crate_name,
                            function_name
                        );
                        0u64
                    }
                }
            };

            signature.returns.push_return(result, task, vm);
            Ok(())
        }
    }


    /// Register crate metadata with the registry
    pub fn register_crate(&mut self, metadata: &CrateMetadata) -> Result<(), VMError> {
        if let Some(ref registry) = self.registry {
            registry
                .register(metadata)
                .map_err(|e| VMError::FFI(format!("Registry error: {}", e)))?;
        }
        Ok(())
    }
}

impl Default for RustFfiBridge {
    fn default() -> Self {
        Self::new().expect("Failed to create RustFfiBridge")
    }
}

/// Rust function signature for FFI
#[derive(Debug, Clone)]
pub struct RustSignature {
    pub params: Vec<RustType>,
    pub returns: RustType,
}

impl RustSignature {
    /// Create a new signature
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
            returns: RustType::Void,
        }
    }

    /// Add a parameter
    pub fn param(mut self, param_type: RustType) -> Self {
        self.params.push(param_type);
        self
    }

    /// Set return type
    pub fn returns(mut self, return_type: RustType) -> Self {
        self.returns = return_type;
        self
    }
}

impl Default for RustSignature {
    fn default() -> Self {
        Self::new()
    }
}

/// Decode a sig_code string (from auto-cache's sig_code module) into a RustSignature.
///
/// sig_code format: "{params}_{ret}" where chars are:
///   v=Void, i=i32(Int), l=i64(Long), f=f64(Double), b=bool(Bool), s=String, p=Pointer
///
/// Examples: "s_s" → (String)→String, "_l" → ()→Long, "ll_l" → (Long,Long)→Long
pub fn sig_code_to_signature(sig_code: &str) -> RustSignature {
    let (params_str, ret_str) = sig_code.split_once('_').unwrap_or(("", "s"));
    let params: Vec<RustType> = params_str.chars().map(sig_char_to_rust_type).collect();
    let ret = sig_char_to_rust_type(ret_str.chars().next().unwrap_or('s'));
    RustSignature { params, returns: ret }
}

fn sig_char_to_rust_type(c: char) -> RustType {
    match c {
        'v' => RustType::Void,
        'i' => RustType::Int,
        'l' => RustType::Long,
        'f' => RustType::Double,
        'b' => RustType::Bool,
        's' => RustType::String,
        'p' => RustType::Pointer,
        _ => RustType::String,
    }
}

/// Build the exported function name from func_name and sig_code.
/// ("from_str", "s_s") → "auto_from_str_s_s"
pub fn build_exported_name(func_name: &str, sig_code: &str) -> String {
    format!("auto_{}_{}", func_name, sig_code)
}

/// Parse manifest JSON (flat object of string:string pairs) into a vec of (key, value).
/// Input: {"from_str":"s_s","to_string":"s_s"}
pub fn parse_manifest_json(json: &str) -> Vec<(String, String)> {
    let trimmed = json.trim().trim_start_matches('{').trim_end_matches('}');
    if trimmed.is_empty() {
        return vec![];
    }
    let mut result = vec![];
    for entry in trimmed.split(',') {
        let entry = entry.trim();
        if let Some((key, value)) = entry.split_once(':') {
            let key = key.trim().trim_matches('"').to_string();
            let value = value.trim().trim_matches('"').to_string();
            if !key.is_empty() {
                result.push((key, value));
            }
        }
    }
    result
}

/// Rust type for FFI marshaling
#[derive(Debug, Clone, PartialEq)]
pub enum RustType {
    Void,
    Bool,
    Int,
    /// 64-bit integer (takes 2 slots)
    Long,
    Float,
    Double,
    /// Null-terminated C string (*const c_char)
    String,
    /// Byte array/pointer with length
    Bytes,
    /// Raw pointer type (for struct pointers, etc.)
    Pointer,
    /// Mutable pointer type
    PointerMut,
    /// Callback/function pointer
    Callback,
}

/// Unified value container for FFI argument marshaling (Phase 3B)
enum RustValue {
    Void,
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Ptr(*const ()),
}

impl RustType {
    /// Pop a value of this type from the VM stack into a RustValue
    fn pop_value(
        &self,
        task: &mut AutoTask,
        vm: &AutoVM,
        string_pool: &mut Vec<std::ffi::CString>,
    ) -> RustValue {
        match self {
            RustType::Bool => RustValue::Bool(task.ram.pop_i32() != 0),
            RustType::Int => RustValue::Int(task.ram.pop_i32()),
            RustType::Long => RustValue::Long(task.ram.pop_i64()),
            RustType::Float => RustValue::Float(task.ram.pop_f32()),
            RustType::Double => RustValue::Double(task.ram.pop_f64()),
            RustType::String => {
                let str_idx = task.ram.pop_str_idx();
                let bytes = vm
                    .strings
                    .read()
                    .ok()
                    .and_then(|s| s.get(str_idx).cloned())
                    .unwrap_or_default();
                let cstr = std::ffi::CString::new(bytes)
                    .unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
                let ptr = cstr.as_ptr();
                string_pool.push(cstr);
                RustValue::Ptr(ptr as *const ())
            }
            RustType::Pointer | RustType::PointerMut => {
                RustValue::Ptr(task.ram.pop_i64() as *const ())
            }
            RustType::Bytes => {
                let _len = task.ram.pop_i32();
                let ptr = task.ram.pop_i64() as *const ();
                RustValue::Ptr(ptr)
            }
            RustType::Callback => RustValue::Ptr(task.ram.pop_i64() as *const ()),
            RustType::Void => RustValue::Void,
        }
    }

    /// Push a raw u64 C return value to the VM stack
    fn push_return(&self, raw: u64, task: &mut AutoTask, vm: &AutoVM) {
        use std::ffi::CStr;
        match self {
            RustType::Void => {}
            RustType::Bool => task.ram.push_i32(if raw != 0 { 1 } else { 0 }),
            RustType::Int => task.ram.push_i32(raw as i32),
            RustType::Long => task.ram.push_i64(raw as i64),
            RustType::Float => task.ram.push_f32(f32::from_bits(raw as u32)),
            RustType::Double => task.ram.push_f64(f64::from_bits(raw as u64)),
            RustType::String => {
                let ptr = raw as *const std::ffi::c_char;
                let s = if ptr.is_null() {
                    String::new()
                } else {
                    unsafe {
                        CStr::from_ptr(ptr)
                            .to_str()
                            .unwrap_or("")
                            .to_string()
                    }
                };
                if let Ok(mut strings) = vm.strings.write() {
                    let idx = strings.len() as u16;
                    strings.push(s.into_bytes());
                    task.ram.push_str_idx(idx as u32);
                } else {
                    task.ram.push_i32(0);
                }
            }
            RustType::Pointer | RustType::PointerMut | RustType::Callback => {
                task.ram.push_i64(raw as i64);
            }
            RustType::Bytes => {
                task.ram.push_i64(raw as i64);
            }
        }
    }
}

/// Plan 212 Phase 2.1: Known function signatures for common crates.
///
/// Maps (crate_name, function_name) → RustSignature for functions that don't
/// follow the default String→String pattern. Used by:
/// - `compile_dep()` wrapper generation (via ShimType conversion)
/// - `init_rust_ffi()` runtime registration
/// - codegen return type inference
///
/// Returns None for unknown functions (caller should default to String→String).
pub fn known_signature(crate_name: &str, func_name: &str) -> Option<RustSignature> {
    match (crate_name, func_name) {
        // rand
        ("rand", "random") => Some(RustSignature::new().returns(RustType::Long)),
        ("rand", "thread_rng") => Some(RustSignature::new().returns(RustType::Pointer)),

        // chrono
        ("chrono", "now") => Some(RustSignature::new().returns(RustType::String)),
        ("chrono", "year") => {
            Some(RustSignature::new().param(RustType::String).returns(RustType::Int))
        }
        ("chrono", "month") => {
            Some(RustSignature::new().param(RustType::String).returns(RustType::Int))
        }
        ("chrono", "day") => {
            Some(RustSignature::new().param(RustType::String).returns(RustType::Int))
        }
        ("chrono", "hour") => {
            Some(RustSignature::new().param(RustType::String).returns(RustType::Int))
        }
        ("chrono", "minute") => {
            Some(RustSignature::new().param(RustType::String).returns(RustType::Int))
        }
        ("chrono", "second") => {
            Some(RustSignature::new().param(RustType::String).returns(RustType::Int))
        }
        ("chrono", "timestamp") => {
            Some(RustSignature::new().param(RustType::String).returns(RustType::Long))
        }

        // url
        ("url", "port") => {
            Some(RustSignature::new().param(RustType::String).returns(RustType::Int))
        }

        // uuid
        ("uuid", "new_v4") => Some(RustSignature::new().returns(RustType::String)),

        // sha2
        ("sha2", "Sha256_new") => Some(RustSignature::new().returns(RustType::Pointer)),
        ("sha2", "Sha256_finalize") => {
            Some(RustSignature::new().param(RustType::Pointer).returns(RustType::String))
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_bridge_creation() {
        let bridge = CFfiBridge::new();
        assert_eq!(bridge.next_native_id, 200);
        assert!(bridge.functions.is_empty());
    }

    #[test]
    fn test_c_signature_creation() {
        let sig = CSignature::new()
            .param(CType::Int)
            .param(CType::Float)
            .returns(CType::Int);

        assert_eq!(sig.params, vec![CType::Int, CType::Float]);
        assert_eq!(sig.returns, CType::Int);
    }

    #[test]
    fn test_ffi_registry() {
        let mut registry = FfiRegistry::new();

        // Get bridge for "hal" library
        let hal_bridge = registry.get_bridge("hal");
        assert_eq!(hal_bridge.next_native_id, 200);

        // Same bridge should be returned (check by comparing next_native_id)
        // We increment the ID in the first bridge, and it should reflect in the second
        hal_bridge.next_native_id = 250;
        let hal_bridge2 = registry.get_bridge("hal");
        assert_eq!(
            hal_bridge2.next_native_id, 250,
            "Same bridge should be returned"
        );
    }

    #[test]
    fn test_register_c_function() {
        let mut bridge = CFfiBridge::new();

        let sig = CSignature::new().param(CType::Int).returns(CType::Int);

        let native_id =
            bridge.register_c_function("hal", "gpio_init", sig, PathBuf::from("target/hal.dll"));

        assert!(native_id.is_ok());
        let id = native_id.unwrap();
        assert_eq!(id, 200);
        assert_eq!(bridge.next_native_id, 201);
    }

    #[test]
    fn test_get_function_id() {
        let mut bridge = CFfiBridge::new();

        let sig = CSignature::new().param(CType::Int).returns(CType::Int);

        let _ = bridge
            .register_c_function("hal", "gpio_init", sig, PathBuf::from("target/hal.dll"))
            .unwrap();

        assert_eq!(bridge.get_function_id("hal", "gpio_init"), Some(200));
        assert_eq!(bridge.get_function_id("hal", "nonexistent"), None);
    }

    // =================================================================
    // Plan 092: Rust FFI Bridge Tests
    // =================================================================

    #[test]
    fn test_rust_ffi_bridge_creation() {
        let bridge = RustFfiBridge::new();
        assert!(bridge.is_ok());

        let bridge = bridge.unwrap();
        assert_eq!(bridge.next_native_id, 300);
        assert!(bridge.functions.is_empty());
        assert!(bridge.loaded_libraries.is_empty());
    }

    #[test]
    fn test_rust_signature_creation() {
        let sig = RustSignature::new()
            .param(RustType::Int)
            .param(RustType::Float)
            .returns(RustType::Bool);

        assert_eq!(sig.params, vec![RustType::Int, RustType::Float]);
        assert_eq!(sig.returns, RustType::Bool);
    }

    #[test]
    fn test_rust_ffi_bridge_sandbox() {
        let bridge = RustFfiBridge::new().unwrap();

        // Should have detected rustc version and target
        let sandbox = bridge.sandbox();
        assert!(!sandbox.rustc_version().is_empty());
        assert!(!sandbox.target().is_empty());

        // Check library name generation
        let lib_name = sandbox.crate_library_path("serde", "1.0.193");
        assert!(lib_name.to_string_lossy().contains("serde"));
    }

    #[test]
    fn test_rust_ffi_get_function_id() {
        let bridge = RustFfiBridge::new().unwrap();

        // Simulate loading a crate (in reality would load actual library)
        // For testing, we just check the function ID lookup logic
        assert_eq!(bridge.get_function_id("nonexistent", "func"), None);
    }
}
