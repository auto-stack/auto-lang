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
use std::path::PathBuf;
use std::sync::Arc;

/// FFI bridge for C-transpiled modules
///
/// **Plan 081 Phase 5**: Manages loading C libraries and registering their functions
/// so they can be called from AutoVM bytecode via CALL_NAT opcode.
///
/// ## Workflow
///
/// 1. **Compilation Phase**:
///    ```auto
///    // hal.at transpiled to C
///    #[c]
/// fn gpio_init(pin int) int;
///    ```
///
/// 2. **Registration Phase**:
///    ```rust
///    ffi_bridge.register_c_function(
    //     "hal",
///     "gpio_init",
///     c_signature!("int(int)"),
///     library_path: "target/hal.dll"
///    );
///    ```
///
/// 3. **Code Generation Phase**:
///    ```rust
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
        let shim = self.create_c_shim(
            library,
            function_name,
            signature,
            library_path.clone(),
        );

        // Register the shim with AutoVM
        self.native_interface.register(native_id, shim);
        self.functions.insert(key, native_id);

        log::info!(
            "Registered C function: {}::{} (native_id={})",
            library, function_name, native_id
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
            return Err(VMError::RuntimeError("Rust FFI ID space exhausted (100-199)".to_string()));
        } else {
            self.next_native_id
        };

        self.next_native_id += 1;

        // Create native shim for this Rust function
        let shim = self.create_rust_shim(
            library,
            function_name,
            signature,
            library_path.clone(),
        );

        self.native_interface.register(native_id, shim);
        self.functions.insert(key, native_id);

        log::info!(
            "Registered Rust function: {}::{} (native_id={})",
            library, function_name, native_id
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
        self.functions.get(&(library.to_string(), function_name.to_string())).copied()
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
        assert_eq!(hal_bridge2.next_native_id, 250, "Same bridge should be returned");
    }

    #[test]
    fn test_register_c_function() {
        let mut bridge = CFfiBridge::new();

        let sig = CSignature::new()
            .param(CType::Int)
            .returns(CType::Int);

        let native_id = bridge.register_c_function(
            "hal",
            "gpio_init",
            sig,
            PathBuf::from("target/hal.dll")
        );

        assert!(native_id.is_ok());
        let id = native_id.unwrap();
        assert_eq!(id, 200);
        assert_eq!(bridge.next_native_id, 201);
    }

    #[test]
    fn test_get_function_id() {
        let mut bridge = CFfiBridge::new();

        let sig = CSignature::new()
            .param(CType::Int)
            .returns(CType::Int);

        let _ = bridge.register_c_function(
            "hal",
            "gpio_init",
            sig,
            PathBuf::from("target/hal.dll")
        ).unwrap();

        assert_eq!(
            bridge.get_function_id("hal", "gpio_init"),
            Some(200)
        );
        assert_eq!(bridge.get_function_id("hal", "nonexistent"), None);
    }
}
