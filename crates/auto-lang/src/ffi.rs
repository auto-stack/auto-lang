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
            // TODO: Implement actual library loading
            // For now, just record the path
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

    /// Register a Rust function from a Rust-transpiled module
    ///
    /// **Plan 081 Phase 5**: Same as C FFI but for Rust functions
    ///
    /// Rust FFI is simpler because Rust can expose C-compatible functions directly.
    ///
    /// # Arguments
    /// * `library` - Library name (e.g., "crypto")
    /// * `function_name` - Rust function name
    /// * `signature` - Function signature
    /// * `library_path` - Path to compiled Rust library (.rlib/.dll)
    pub fn register_rust_function(
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

        // Reserve IDs 100-199 for Rust FFI
        let native_id = if self.next_native_id >= 200 {
            return Err(VMError::RuntimeError(
                "Rust FFI ID space exhausted (100-199)".to_string(),
            ));
        } else {
            self.next_native_id
        };

        self.next_native_id += 1;

        // Create native shim for this Rust function
        let shim = self.create_rust_shim(library, function_name, signature, library_path.clone());

        self.native_interface.register(native_id, shim);
        self.functions.insert(key, native_id);

        log::info!(
            "Registered Rust function: {}::{} (native_id={})",
            library,
            function_name,
            native_id
        );

        Ok(native_id)
    }

    /// Create a native shim for calling a C function
    fn create_c_shim(
        &self,
        _library: &str,
        function_name: &str,
        signature: CSignature,
        _library_path: PathBuf,
    ) -> impl Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static {
        let function_name = function_name.to_string();
        let signature = signature.clone();

        move |task: &mut AutoTask, _vm: &AutoVM| {
            // Pop arguments from AutoVM stack based on signature
            let args = Self::pop_arguments(task, &signature.params)?;

            // TODO: Call C function via FFI
            // For now, just return a dummy value
            log::warn!(
                "C FFI not yet implemented for {} (would take {:?} args)",
                function_name,
                args.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>()
            );

            // Push dummy return value
            // In real implementation, this would be the C function's return value
            // converted to AutoVM representation
            task.ram.push_i32(0);

            Ok(())
        }
    }

    /// Create a native shim for calling a Rust function
    fn create_rust_shim(
        &self,
        _library: &str,
        function_name: &str,
        signature: CSignature,
        _library_path: PathBuf,
    ) -> impl Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static {
        let function_name = function_name.to_string();
        let signature = signature.clone();

        move |task: &mut AutoTask, _vm: &AutoVM| {
            // Pop arguments from AutoVM stack based on signature
            let args = Self::pop_arguments(task, &signature.params)?;

            // TODO: Call Rust function via FFI
            // For now, just return a dummy value
            log::warn!(
                "Rust FFI not yet implemented for {} (would take {:?} args)",
                function_name,
                args.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>()
            );

            // Push dummy return value
            task.ram.push_i32(0);

            Ok(())
        }
    }

    /// Pop arguments from AutoVM stack according to parameter types
    fn pop_arguments(_task: &mut AutoTask, signature: &[CType]) -> Result<Vec<CValue>, VMError> {
        let mut args = Vec::new();

        // TODO: Actually pop from task.ram based on signature
        // For now, return dummy values
        for param_type in signature {
            let value = match param_type {
                CType::Int => CValue::Int(0),
                CType::Float => CValue::Float(0.0),
                CType::Str => CValue::Str("".to_string()),
                CType::Void => CValue::Void,
            };
            args.push(value);
        }

        Ok(args)
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

        // Get the symbol from the library
        let symbol_name = std::ffi::CString::new(function_name.as_bytes())
            .map_err(|e| VMError::FFI(format!("Invalid function name: {}", e)))?;

        // Get the symbol as a raw pointer (we'll cast it based on signature)
        let symbol_ptr: *const () = unsafe {
            let symbol: libloading::Symbol<*const ()> = library
                .get(symbol_name.as_bytes())
                .map_err(|e| VMError::FFI(format!("Symbol {} not found: {}", function_name, e)))?;
            *symbol.into_raw()
        };

        let native_id = self.next_native_id;
        self.next_native_id += 1;

        // Create shim for this function
        let shim = self.create_rust_shim_with_ptr(
            Arc::clone(library),
            crate_name,
            function_name,
            signature.clone(),
            symbol_ptr,
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

    /// Get the native ID for a registered function
    pub fn get_function_id(&self, crate_name: &str, function_name: &str) -> Option<u16> {
        let key = format!("{}::{}", crate_name, function_name);
        self.functions.get(&key).copied()
    }

    /// Get all registered functions
    pub fn get_functions(&self) -> &HashMap<String, u16> {
        &self.functions
    }

    /// Create a native shim for calling a Rust function with actual symbol resolution
    fn create_rust_shim_with_ptr(
        &self,
        _library: Arc<libloading::Library>,
        crate_name: &str,
        function_name: &str,
        signature: RustSignature,
        symbol_ptr: *const (),
    ) -> impl Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static {
        let crate_name = crate_name.to_string();
        let function_name = function_name.to_string();
        let signature = signature.clone();
        let symbol_ptr = symbol_ptr as usize; // Store as usize for Send + Sync

        move |task: &mut AutoTask, _vm: &AutoVM| {
            let func_ptr = symbol_ptr as *const ();

            // Pop arguments from stack (in reverse order)
            // Note: Arguments are pushed left-to-right, so we pop right-to-left
            let mut args_i32: Vec<i32> = Vec::new();
            let mut args_i64: Vec<i64> = Vec::new();
            let mut args_f32: Vec<f32> = Vec::new();
            let mut args_f64: Vec<f64> = Vec::new();
            let mut args_ptr: Vec<*const ()> = Vec::new();
            let mut string_pool: Vec<std::ffi::CString> = Vec::new(); // Keep strings alive

            for param_type in signature.params.iter().rev() {
                match param_type {
                    RustType::Int | RustType::Bool => {
                        args_i32.push(task.ram.pop_i32());
                    }
                    RustType::Long => {
                        // 64-bit integer (2 slots)
                        args_i64.push(task.ram.pop_i64());
                    }
                    RustType::Float => {
                        args_f32.push(task.ram.pop_f32());
                    }
                    RustType::Double => {
                        args_f64.push(task.ram.pop_f64());
                    }
                    RustType::String => {
                        // Pop string index from constants pool, convert to C string
                        let str_idx = task.ram.pop_i32() as usize;
                        // Get string from pool (owned copy to avoid lifetime issues)
                        let str_owned: Vec<u8> = if let Ok(strings) = _vm.strings.read() {
                            strings.get(str_idx).cloned().unwrap_or_default()
                        } else {
                            Vec::new()
                        };
                        let c_string = std::ffi::CString::new(str_owned)
                            .unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
                        args_ptr.push(c_string.as_ptr() as *const ());
                        string_pool.push(c_string); // Keep alive for duration of call
                    }
                    RustType::Pointer | RustType::PointerMut => {
                        // Pointers are stored as i64 (2 slots)
                        let ptr_val = task.ram.pop_i64();
                        args_ptr.push(ptr_val as *const ());
                    }
                    RustType::Bytes => {
                        // Bytes: pop pointer and length as separate values
                        let len = task.ram.pop_i32() as usize;
                        let ptr_val = task.ram.pop_i64();
                        args_ptr.push(ptr_val as *const ());
                        args_i32.push(len as i32);
                    }
                    RustType::Callback => {
                        // Callback: pop function pointer as i64
                        let callback_ptr = task.ram.pop_i64();
                        args_ptr.push(callback_ptr as *const ());
                    }
                    RustType::Void => {
                        // Void as param is unusual but handle gracefully
                        log::warn!("Void used as parameter type");
                    }
                }
            }

            // Reverse back to original order
            args_i32.reverse();
            args_i64.reverse();
            args_f32.reverse();
            args_f64.reverse();
            args_ptr.reverse();

            // Call the function based on signature
            // For now, support common patterns:
            // - fn() -> T (no args)
            // - fn(i32) -> T
            // - fn(i32, i32) -> T
            // - fn(f32) -> T
            // - fn(i32, f32) -> T

            let result = unsafe {
                match (signature.params.as_slice(), &signature.returns) {
                    // void fn()
                    (&[], RustType::Void) => {
                        let func: fn() = std::mem::transmute(func_ptr);
                        func();
                        return Ok(());
                    }
                    // i32 fn()
                    (&[], RustType::Int) => {
                        let func: fn() -> i32 = std::mem::transmute(func_ptr);
                        func()
                    }
                    // f32 fn()
                    (&[], RustType::Float) => {
                        let _func: fn() -> f32 = std::mem::transmute(func_ptr);
                        return Ok(()); // Handle separately
                    }
                    // bool fn()
                    (&[], RustType::Bool) => {
                        let func: fn() -> bool = std::mem::transmute(func_ptr);
                        if func() {
                            1
                        } else {
                            0
                        }
                    }
                    // i32 fn(i32)
                    (&[RustType::Int], RustType::Int) => {
                        let func: fn(i32) -> i32 = std::mem::transmute(func_ptr);
                        func(args_i32.get(0).copied().unwrap_or(0))
                    }
                    // i32 fn(i32, i32)
                    (&[RustType::Int, RustType::Int], RustType::Int) => {
                        let func: fn(i32, i32) -> i32 = std::mem::transmute(func_ptr);
                        func(
                            args_i32.get(0).copied().unwrap_or(0),
                            args_i32.get(1).copied().unwrap_or(0),
                        )
                    }
                    // f32 fn(f32)
                    (&[RustType::Float], RustType::Float) => {
                        let func: fn(f32) -> f32 = std::mem::transmute(func_ptr);
                        let r = func(args_f32.get(0).copied().unwrap_or(0.0));
                        task.ram.push_f32(r);
                        return Ok(());
                    }
                    // i32 fn(i32, f32)
                    (&[RustType::Int, RustType::Float], RustType::Int) => {
                        let func: fn(i32, f32) -> i32 = std::mem::transmute(func_ptr);
                        func(
                            args_i32.get(0).copied().unwrap_or(0),
                            args_f32.get(0).copied().unwrap_or(0.0),
                        )
                    }
                    // void fn(i32)
                    (&[RustType::Int], RustType::Void) => {
                        let func: fn(i32) = std::mem::transmute(func_ptr);
                        func(args_i32.get(0).copied().unwrap_or(0));
                        return Ok(());
                    }
                    // void fn(i32, i32)
                    (&[RustType::Int, RustType::Int], RustType::Void) => {
                        let func: fn(i32, i32) = std::mem::transmute(func_ptr);
                        func(
                            args_i32.get(0).copied().unwrap_or(0),
                            args_i32.get(1).copied().unwrap_or(0),
                        );
                        return Ok(());
                    }

                    // === String patterns ===
                    // void fn(*const c_char)
                    (&[RustType::String], RustType::Void) => {
                        let func: fn(*const std::ffi::c_char) = std::mem::transmute(func_ptr);
                        func(args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                            as *const std::ffi::c_char);
                        return Ok(());
                    }
                    // i32 fn(*const c_char) - e.g., strlen, parse_int
                    (&[RustType::String], RustType::Int) => {
                        let func: fn(*const std::ffi::c_char) -> i32 =
                            std::mem::transmute(func_ptr);
                        func(args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                            as *const std::ffi::c_char)
                    }
                    // i32 fn(*const c_char, i32) - e.g., strncmp, substring
                    (&[RustType::String, RustType::Int], RustType::Int) => {
                        let func: fn(*const std::ffi::c_char, i32) -> i32 =
                            std::mem::transmute(func_ptr);
                        func(
                            args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                                as *const std::ffi::c_char,
                            args_i32.get(0).copied().unwrap_or(0),
                        )
                    }
                    // i32 fn(i32, *const c_char) - e.g., fprintf patterns
                    (&[RustType::Int, RustType::String], RustType::Int) => {
                        let func: fn(i32, *const std::ffi::c_char) -> i32 =
                            std::mem::transmute(func_ptr);
                        func(
                            args_i32.get(0).copied().unwrap_or(0),
                            args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                                as *const std::ffi::c_char,
                        )
                    }
                    // void fn(i32, *const c_char) - e.g., config setter
                    (&[RustType::Int, RustType::String], RustType::Void) => {
                        let func: fn(i32, *const std::ffi::c_char) = std::mem::transmute(func_ptr);
                        func(
                            args_i32.get(0).copied().unwrap_or(0),
                            args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                                as *const std::ffi::c_char,
                        );
                        return Ok(());
                    }
                    // i32 fn(*const c_char, *const c_char) - e.g., strcmp
                    (&[RustType::String, RustType::String], RustType::Int) => {
                        let func: fn(*const std::ffi::c_char, *const std::ffi::c_char) -> i32 =
                            std::mem::transmute(func_ptr);
                        func(
                            args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                                as *const std::ffi::c_char,
                            args_ptr.get(1).copied().unwrap_or(std::ptr::null())
                                as *const std::ffi::c_char,
                        )
                    }

                    // === Pointer patterns (for structs) ===
                    // void fn(*mut T)
                    (&[RustType::Pointer], RustType::Void) => {
                        let func: fn(*mut std::ffi::c_void) = std::mem::transmute(func_ptr);
                        func(args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                            as *mut std::ffi::c_void);
                        return Ok(());
                    }
                    // i32 fn(*mut T)
                    (&[RustType::Pointer], RustType::Int) => {
                        let func: fn(*mut std::ffi::c_void) -> i32 = std::mem::transmute(func_ptr);
                        func(args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                            as *mut std::ffi::c_void)
                    }
                    // *mut T fn() - e.g., constructor returning pointer
                    (&[], RustType::Pointer) => {
                        let func: fn() -> *mut std::ffi::c_void = std::mem::transmute(func_ptr);
                        let ptr = func();
                        task.ram.push_i64(ptr as i64);
                        return Ok(());
                    }
                    // *mut T fn(i32) - constructor with size hint
                    (&[RustType::Int], RustType::Pointer) => {
                        let func: fn(i32) -> *mut std::ffi::c_void = std::mem::transmute(func_ptr);
                        let ptr = func(args_i32.get(0).copied().unwrap_or(0));
                        task.ram.push_i64(ptr as i64);
                        return Ok(());
                    }
                    // void fn(*mut T, i32) - method with int param
                    (&[RustType::Pointer, RustType::Int], RustType::Void) => {
                        let func: fn(*mut std::ffi::c_void, i32) = std::mem::transmute(func_ptr);
                        func(
                            args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                                as *mut std::ffi::c_void,
                            args_i32.get(0).copied().unwrap_or(0),
                        );
                        return Ok(());
                    }
                    // i32 fn(*mut T, i32) - method returning int
                    (&[RustType::Pointer, RustType::Int], RustType::Int) => {
                        let func: fn(*mut std::ffi::c_void, i32) -> i32 =
                            std::mem::transmute(func_ptr);
                        func(
                            args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                                as *mut std::ffi::c_void,
                            args_i32.get(0).copied().unwrap_or(0),
                        )
                    }
                    // void fn(*mut T, *const c_char) - method with string param
                    (&[RustType::Pointer, RustType::String], RustType::Void) => {
                        let func: fn(*mut std::ffi::c_void, *const std::ffi::c_char) =
                            std::mem::transmute(func_ptr);
                        func(
                            args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                                as *mut std::ffi::c_void,
                            args_ptr.get(1).copied().unwrap_or(std::ptr::null())
                                as *const std::ffi::c_char,
                        );
                        return Ok(());
                    }
                    // i32 fn(*mut T, *const c_char) - method with string returning int
                    (&[RustType::Pointer, RustType::String], RustType::Int) => {
                        let func: fn(*mut std::ffi::c_void, *const std::ffi::c_char) -> i32 =
                            std::mem::transmute(func_ptr);
                        func(
                            args_ptr.get(0).copied().unwrap_or(std::ptr::null())
                                as *mut std::ffi::c_void,
                            args_ptr.get(1).copied().unwrap_or(std::ptr::null())
                                as *const std::ffi::c_char,
                        )
                    }

                    // === Bytes pattern ===
                    // void fn(*const u8, usize) - e.g., write buffer
                    (&[RustType::Bytes], RustType::Void) => {
                        let func: fn(*const u8, usize) = std::mem::transmute(func_ptr);
                        let len = args_i32.get(0).copied().unwrap_or(0) as usize;
                        let ptr = args_ptr.get(0).copied().unwrap_or(std::ptr::null());
                        func(ptr as *const u8, len);
                        return Ok(());
                    }
                    // i32 fn(*const u8, usize) - e.g., parse buffer
                    (&[RustType::Bytes], RustType::Int) => {
                        let func: fn(*const u8, usize) -> i32 = std::mem::transmute(func_ptr);
                        let len = args_i32.get(0).copied().unwrap_or(0) as usize;
                        let ptr = args_ptr.get(0).copied().unwrap_or(std::ptr::null());
                        func(ptr as *const u8, len)
                    }

                    // === Long (i64) patterns ===
                    // i64 fn()
                    (&[], RustType::Long) => {
                        let func: fn() -> i64 = std::mem::transmute(func_ptr);
                        let val = func();
                        task.ram.push_i64(val);
                        return Ok(());
                    }
                    // i64 fn(i32)
                    (&[RustType::Int], RustType::Long) => {
                        let func: fn(i32) -> i64 = std::mem::transmute(func_ptr);
                        let val = func(args_i32.get(0).copied().unwrap_or(0));
                        task.ram.push_i64(val);
                        return Ok(());
                    }
                    // void fn(i64)
                    (&[RustType::Long], RustType::Void) => {
                        let func: fn(i64) = std::mem::transmute(func_ptr);
                        func(args_i64.get(0).copied().unwrap_or(0));
                        return Ok(());
                    }
                    // i64 fn(i64)
                    (&[RustType::Long], RustType::Long) => {
                        let func: fn(i64) -> i64 = std::mem::transmute(func_ptr);
                        let val = func(args_i64.get(0).copied().unwrap_or(0));
                        task.ram.push_i64(val);
                        return Ok(());
                    }

                    // === Double (f64) patterns ===
                    // f64 fn()
                    (&[], RustType::Double) => {
                        let func: fn() -> f64 = std::mem::transmute(func_ptr);
                        let val = func();
                        task.ram.push_f64(val);
                        return Ok(());
                    }
                    // f64 fn(f64)
                    (&[RustType::Double], RustType::Double) => {
                        let func: fn(f64) -> f64 = std::mem::transmute(func_ptr);
                        let r = func(args_f64.get(0).copied().unwrap_or(0.0));
                        task.ram.push_f64(r);
                        return Ok(());
                    }
                    // f64 fn(i32, f64)
                    (&[RustType::Int, RustType::Double], RustType::Double) => {
                        let func: fn(i32, f64) -> f64 = std::mem::transmute(func_ptr);
                        let r = func(
                            args_i32.get(0).copied().unwrap_or(0),
                            args_f64.get(0).copied().unwrap_or(0.0),
                        );
                        task.ram.push_f64(r);
                        return Ok(());
                    }

                    // Default: log warning and return 0
                    _ => {
                        log::warn!(
                            "Unsupported FFI signature: {:?} -> {:?} for {}::{}",
                            signature.params,
                            signature.returns,
                            crate_name,
                            function_name
                        );
                        0
                    }
                }
            };

            // Push return value (for non-void, non-float returns)
            match &signature.returns {
                RustType::Int | RustType::Bool => task.ram.push_i32(result),
                RustType::Void => {}  // Already returned above
                RustType::Float => {} // Handled separately above
                _ => task.ram.push_i32(0),
            }

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
