//! Plan 216 Phase 2: C-FFI Runtime
//!
//! Loads C libraries via libloading and registers functions as native shims.
//! Manifests (JSON files in c_bindings/) describe function signatures so the
//! runtime knows how to marshal arguments between AutoVM and C.

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::native::NativeInterface;
use crate::vm::task::AutoTask;
use auto_bindgen::manifest::{CFunction, CHeaderManifest, CTypeDesc};
use std::collections::HashMap;
use std::sync::Arc;

/// Reserved ID range for C-FFI native shims: 500–999
const CFFI_ID_START: u16 = 500;
const CFFI_ID_END: u16 = 999;

/// C-FFI Runtime: loads manifests and registers C functions as native shims.
pub struct CFfiRuntime {
    /// Loaded platform library
    library: Option<Arc<libloading::Library>>,
    /// Registered functions: function_name -> native_id
    functions: HashMap<String, u16>,
    /// Next available native ID
    next_id: u16,
    /// The native interface shims are registered into
    native_interface: NativeInterface,
}

impl CFfiRuntime {
    pub fn new() -> Self {
        Self {
            library: None,
            functions: HashMap::new(),
            next_id: CFFI_ID_START,
            native_interface: NativeInterface::new(),
        }
    }

    /// Load a platform C library and a header manifest, registering all functions.
    ///
    /// If the library is already loaded, only registers any new functions.
    pub fn load_header(&mut self, manifest: &CHeaderManifest) -> Result<(), VMError> {
        // Load platform library once
        if self.library.is_none() {
            let lib_name = Self::resolve_library_name(&manifest.library);
            let lib = unsafe {
                libloading::Library::new(&lib_name).map_err(|e| {
                    VMError::FFI(format!(
                        "Failed to load C library '{}': {}",
                        lib_name, e
                    ))
                })?
            };
            self.library = Some(Arc::new(lib));
        }

        let lib = self.library.as_ref().unwrap();

        for func in &manifest.functions {
            if self.functions.contains_key(&func.name) {
                continue;
            }
            if func.variadic {
                // Skip variadic functions — they need special handling
                log::warn!("Skipping variadic C function: {}", func.name);
                continue;
            }
            let native_id = self.next_id;
            if native_id >= CFFI_ID_END {
                return Err(VMError::FFI(
                    "C-FFI native ID space exhausted (500-999)".to_string(),
                ));
            }
            self.next_id += 1;

            let shim = create_c_shim(Arc::clone(lib), &func)?;
            self.native_interface.register_static(native_id, shim);
            self.functions.insert(func.name.clone(), native_id);

            log::info!(
                "Registered C-FFI function: {} (native_id={})",
                func.name,
                native_id
            );
        }
        Ok(())
    }

    /// Look up the native ID for a registered C function.
    pub fn get_function_id(&self, name: &str) -> Option<u16> {
        self.functions.get(name).copied()
    }

    /// Get the native interface (to merge into the main VM interface).
    pub fn native_interface(&self) -> &NativeInterface {
        &self.native_interface
    }

    /// Consume self and return the native interface as Arc.
    pub fn into_native_interface_arc(self) -> Arc<NativeInterface> {
        Arc::new(self.native_interface)
    }

    /// Resolve a manifest library name to a platform-specific library file.
    fn resolve_library_name(lib: &str) -> String {
        if cfg!(target_os = "windows") {
            match lib {
                "c" => "ucrtbase.dll".to_string(),
                "m" => "ucrtbase.dll".to_string(), // math is in ucrt on Windows
                _ => format!("{}.dll", lib),
            }
        } else if cfg!(target_os = "macos") {
            match lib {
                "c" | "m" => "libSystem.dylib".to_string(),
                _ => format!("lib{}.dylib", lib),
            }
        } else {
            match lib {
                "c" | "m" => "libc.so.6".to_string(),
                _ => format!("lib{}.so", lib),
            }
        }
    }
}

/// Create a native shim for a C function based on its manifest signature.
///
/// Uses exhaustive match on parameter/return type combinations, the same pattern
/// as `RustFfiBridge::create_rust_shim_with_ptr`.
fn create_c_shim(
    library: Arc<libloading::Library>,
    func: &CFunction,
) -> Result<
    impl Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    VMError,
> {
    let name_cstr = std::ffi::CString::new(func.name.as_bytes())
        .map_err(|e| VMError::FFI(format!("Invalid C function name '{}': {}", func.name, e)))?;

    let symbol: libloading::Symbol<'_, *const ()> = unsafe {
        library
            .get(name_cstr.as_bytes())
            .map_err(|e| VMError::FFI(format!("Symbol '{}' not found: {}", func.name, e)))?
    };
    let func_ptr = unsafe { *symbol.into_raw() } as usize;

    let func_name = func.name.clone();
    let param_types: Vec<CTypeDesc> = func.params.iter().map(|p| p.ty.clone()).collect();
    let ret_type = func.return_type.clone();

    Ok(move |task: &mut AutoTask, _vm: &AutoVM| {
        let fp = func_ptr as *const ();

        // Pop arguments in reverse order
        let mut args_i32: Vec<i32> = Vec::new();
        let mut args_i64: Vec<i64> = Vec::new();
        let mut args_f64: Vec<f64> = Vec::new();
        let mut args_ptr: Vec<*const ()> = Vec::new();
        let mut string_pool: Vec<std::ffi::CString> = Vec::new();

        for pty in param_types.iter().rev() {
            match pty {
                CTypeDesc::Bool | CTypeDesc::Char | CTypeDesc::Int | CTypeDesc::UInt => {
                    args_i32.push(task.ram.pop_i32());
                }
                CTypeDesc::Long | CTypeDesc::ULong | CTypeDesc::Size => {
                    args_i64.push(task.ram.pop_i64());
                }
                CTypeDesc::Float => {
                    // VM stores floats as f64 on stack; convert to f32 at call site
                    let v = task.ram.pop_f64();
                    args_i32.push(v as f32 as i32);
                }
                CTypeDesc::Double => {
                    args_f64.push(task.ram.pop_f64());
                }
                CTypeDesc::CStr => {
                    let str_idx = task.ram.pop_i32();
                    let idx = if str_idx < 0 {
                        (-str_idx - 1) as usize
                    } else {
                        str_idx as usize
                    };
                    let owned: Vec<u8> = if let Ok(strings) = _vm.strings.read() {
                        strings.get(idx).cloned().unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    let cstr = std::ffi::CString::new(owned)
                        .unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
                    args_ptr.push(cstr.as_ptr() as *const ());
                    string_pool.push(cstr);
                }
                CTypeDesc::Ptr | CTypeDesc::PtrMut => {
                    let ptr_val = task.ram.pop_i64();
                    args_ptr.push(ptr_val as *const ());
                }
                CTypeDesc::Void => {}
            }
        }

        // Reverse to restore original parameter order
        args_i32.reverse();
        args_i64.reverse();
        args_f64.reverse();
        args_ptr.reverse();

        // Dispatch based on (param_types, return_type)
        unsafe {
            match (param_types.as_slice(), &ret_type) {
                // --- void fn() ---
                ([], CTypeDesc::Void) => {
                    let f: fn() = std::mem::transmute(fp);
                    f();
                    return Ok(());
                }
                // --- int fn() ---
                ([], CTypeDesc::Int) => {
                    let f: fn() -> i32 = std::mem::transmute(fp);
                    let r = f();
                    task.ram.push_i32(r);
                    return Ok(());
                }
                // --- long fn() ---
                ([], CTypeDesc::Long) | ([], CTypeDesc::Size) => {
                    let f: fn() -> i64 = std::mem::transmute(fp);
                    let r = f();
                    task.ram.push_i64(r);
                    return Ok(());
                }
                // --- double fn() ---
                ([], CTypeDesc::Double) => {
                    let f: fn() -> f64 = std::mem::transmute(fp);
                    let r = f();
                    task.ram.push_f64(r);
                    return Ok(());
                }
                // --- ptr fn() ---
                ([], CTypeDesc::Ptr) => {
                    let f: fn() -> *mut std::ffi::c_void = std::mem::transmute(fp);
                    let r = f();
                    task.ram.push_i64(r as i64);
                    return Ok(());
                }

                // --- int fn(int) ---
                ([CTypeDesc::Int], CTypeDesc::Int) => {
                    let f: fn(i32) -> i32 = std::mem::transmute(fp);
                    let r = f(args_i32.get(0).copied().unwrap_or(0));
                    task.ram.push_i32(r);
                    return Ok(());
                }
                // --- double fn(double) ---
                ([CTypeDesc::Double], CTypeDesc::Double) => {
                    let f: fn(f64) -> f64 = std::mem::transmute(fp);
                    let r = f(args_f64.get(0).copied().unwrap_or(0.0));
                    task.ram.push_f64(r);
                    return Ok(());
                }
                // --- size_t fn(const char*) ---
                ([CTypeDesc::CStr], CTypeDesc::Size) => {
                    let f: fn(*const std::ffi::c_char) -> usize = std::mem::transmute(fp);
                    let r = f(args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char);
                    task.ram.push_i64(r as i64);
                    return Ok(());
                }
                // --- int fn(const char*) ---
                ([CTypeDesc::CStr], CTypeDesc::Int) => {
                    let f: fn(*const std::ffi::c_char) -> i32 = std::mem::transmute(fp);
                    let r = f(args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char);
                    task.ram.push_i32(r);
                    return Ok(());
                }
                // --- double fn(const char*) ---
                ([CTypeDesc::CStr], CTypeDesc::Double) => {
                    let f: fn(*const std::ffi::c_char) -> f64 = std::mem::transmute(fp);
                    let r = f(args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char);
                    task.ram.push_f64(r);
                    return Ok(());
                }
                // --- ptr fn(const char*) ---
                ([CTypeDesc::CStr], CTypeDesc::Ptr) => {
                    let f: fn(*const std::ffi::c_char) -> *mut std::ffi::c_void =
                        std::mem::transmute(fp);
                    let r = f(args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char);
                    task.ram.push_i64(r as i64);
                    return Ok(());
                }
                // --- int fn(const char*, const char*) ---
                ([CTypeDesc::CStr, CTypeDesc::CStr], CTypeDesc::Int) => {
                    let f: fn(*const std::ffi::c_char, *const std::ffi::c_char) -> i32 =
                        std::mem::transmute(fp);
                    let r = f(
                        args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char,
                        args_ptr.get(1).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char,
                    );
                    task.ram.push_i32(r);
                    return Ok(());
                }
                // --- ptr fn(const char*, const char*) ---
                ([CTypeDesc::CStr, CTypeDesc::CStr], CTypeDesc::Ptr) => {
                    let f: fn(*const std::ffi::c_char, *const std::ffi::c_char) -> *mut std::ffi::c_void =
                        std::mem::transmute(fp);
                    let r = f(
                        args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char,
                        args_ptr.get(1).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char,
                    );
                    task.ram.push_i64(r as i64);
                    return Ok(());
                }
                // --- double fn(double, double) ---
                ([CTypeDesc::Double, CTypeDesc::Double], CTypeDesc::Double) => {
                    let f: fn(f64, f64) -> f64 = std::mem::transmute(fp);
                    let r = f(
                        args_f64.get(0).copied().unwrap_or(0.0),
                        args_f64.get(1).copied().unwrap_or(0.0),
                    );
                    task.ram.push_f64(r);
                    return Ok(());
                }
                // --- ptr fn(size_t) ---
                ([CTypeDesc::Size], CTypeDesc::Ptr) => {
                    let f: fn(usize) -> *mut std::ffi::c_void = std::mem::transmute(fp);
                    let sz = args_i64.get(0).copied().unwrap_or(0) as usize;
                    let r = f(sz);
                    task.ram.push_i64(r as i64);
                    return Ok(());
                }
                // --- void fn(ptr) ---
                ([CTypeDesc::Ptr], CTypeDesc::Void) => {
                    let f: fn(*mut std::ffi::c_void) = std::mem::transmute(fp);
                    f(args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *mut std::ffi::c_void);
                    return Ok(());
                }
                // --- void fn(int) ---
                ([CTypeDesc::Int], CTypeDesc::Void) => {
                    let f: fn(i32) = std::mem::transmute(fp);
                    f(args_i32.get(0).copied().unwrap_or(0));
                    return Ok(());
                }
                // --- void fn(uint) ---
                ([CTypeDesc::UInt], CTypeDesc::Void) => {
                    let f: fn(u32) = std::mem::transmute(fp);
                    f(args_i32.get(0).copied().unwrap_or(0) as u32);
                    return Ok(());
                }
                // --- int fn(ptr) ---
                ([CTypeDesc::Ptr], CTypeDesc::Int) => {
                    let f: fn(*mut std::ffi::c_void) -> i32 = std::mem::transmute(fp);
                    let r = f(args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *mut std::ffi::c_void);
                    task.ram.push_i32(r);
                    return Ok(());
                }
                // --- int fn(const char*, ptr)  fputs ---
                ([CTypeDesc::CStr, CTypeDesc::Ptr], CTypeDesc::Int) => {
                    let f: fn(*const std::ffi::c_char, *mut std::ffi::c_void) -> i32 =
                        std::mem::transmute(fp);
                    let r = f(
                        args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char,
                        args_ptr.get(1).copied().unwrap_or(std::ptr::null()) as *mut std::ffi::c_void,
                    );
                    task.ram.push_i32(r);
                    return Ok(());
                }
                // --- ptr fn(ptr_mut, const char*) ---
                ([CTypeDesc::PtrMut, CTypeDesc::CStr], CTypeDesc::Ptr) => {
                    let f: fn(*mut std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void =
                        std::mem::transmute(fp);
                    let r = f(
                        args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *mut std::ffi::c_void,
                        args_ptr.get(1).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char,
                    );
                    task.ram.push_i64(r as i64);
                    return Ok(());
                }
                // --- ptr fn(ptr_mut, int, ptr)  fgets ---
                ([CTypeDesc::PtrMut, CTypeDesc::Int, CTypeDesc::Ptr], CTypeDesc::Ptr) => {
                    let f: fn(*mut std::ffi::c_void, i32, *mut std::ffi::c_void) -> *mut std::ffi::c_void =
                        std::mem::transmute(fp);
                    let r = f(
                        args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *mut std::ffi::c_void,
                        args_i32.get(0).copied().unwrap_or(0),
                        args_ptr.get(1).copied().unwrap_or(std::ptr::null()) as *mut std::ffi::c_void,
                    );
                    task.ram.push_i64(r as i64);
                    return Ok(());
                }
                // --- ptr fn(ptr_mut, int, size_t)  memset ---
                ([CTypeDesc::PtrMut, CTypeDesc::Int, CTypeDesc::Size], CTypeDesc::Ptr) => {
                    let f: fn(*mut std::ffi::c_void, i32, usize) -> *mut std::ffi::c_void =
                        std::mem::transmute(fp);
                    let r = f(
                        args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *mut std::ffi::c_void,
                        args_i32.get(0).copied().unwrap_or(0),
                        args_i64.get(0).copied().unwrap_or(0) as usize,
                    );
                    task.ram.push_i64(r as i64);
                    return Ok(());
                }
                // --- ptr fn(ptr_mut, ptr, size_t)  memcpy ---
                ([CTypeDesc::PtrMut, CTypeDesc::Ptr, CTypeDesc::Size], CTypeDesc::Ptr) => {
                    let f: fn(*mut std::ffi::c_void, *const std::ffi::c_void, usize) -> *mut std::ffi::c_void =
                        std::mem::transmute(fp);
                    let r = f(
                        args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *mut std::ffi::c_void,
                        args_ptr.get(1).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_void,
                        args_i64.get(0).copied().unwrap_or(0) as usize,
                    );
                    task.ram.push_i64(r as i64);
                    return Ok(());
                }
                // --- int fn(const char*, const char*, size_t)  strncmp ---
                ([CTypeDesc::CStr, CTypeDesc::CStr, CTypeDesc::Size], CTypeDesc::Int) => {
                    let f: fn(*const std::ffi::c_char, *const std::ffi::c_char, usize) -> i32 =
                        std::mem::transmute(fp);
                    let r = f(
                        args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char,
                        args_ptr.get(1).copied().unwrap_or(std::ptr::null()) as *const std::ffi::c_char,
                        args_i64.get(0).copied().unwrap_or(0) as usize,
                    );
                    task.ram.push_i32(r);
                    return Ok(());
                }
                // --- long fn(ptr_mut)  time ---
                ([CTypeDesc::PtrMut], CTypeDesc::Long) => {
                    let f: fn(*mut std::ffi::c_void) -> i64 = std::mem::transmute(fp);
                    let r = f(args_ptr.get(0).copied().unwrap_or(std::ptr::null()) as *mut std::ffi::c_void);
                    task.ram.push_i64(r);
                    return Ok(());
                }
                // --- double fn(double, double)  difftime ---
                // Already handled by the Double, Double -> Double arm above.

                _ => {
                    log::warn!(
                        "Unsupported C-FFI signature for {}: {:?} -> {:?}",
                        func_name,
                        param_types,
                        ret_type
                    );
                    // Push a zero return value
                    task.ram.push_i32(0);
                    return Ok(());
                }
            }
        }
    })
}

/// Load a built-in manifest JSON from the embedded c_bindings directory.
///
/// Returns `None` if the header is not recognized.
pub fn load_builtin_manifest(header: &str) -> Option<CHeaderManifest> {
    // First try the extractor (hard-coded definitions)
    if let Some(m) = auto_bindgen::extractor::get_builtin_manifest(header) {
        return Some(m);
    }
    // Strip angle brackets if present
    let clean = header.trim_start_matches('<').trim_end_matches('>');
    auto_bindgen::extractor::get_builtin_manifest(clean)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_library_name() {
        let name = CFfiRuntime::resolve_library_name("c");
        if cfg!(target_os = "windows") {
            assert_eq!(name, "ucrtbase.dll");
        } else if cfg!(target_os = "macos") {
            assert_eq!(name, "libSystem.dylib");
        } else {
            assert_eq!(name, "libc.so.6");
        }
    }

    #[test]
    fn test_resolve_math_library() {
        let name = CFfiRuntime::resolve_library_name("m");
        if cfg!(target_os = "windows") {
            assert_eq!(name, "ucrtbase.dll");
        }
    }

    #[test]
    fn test_load_builtin_manifest() {
        let m = load_builtin_manifest("string.h").unwrap();
        assert_eq!(m.header, "string.h");
        assert!(m.functions.iter().any(|f| f.name == "strlen"));

        let m = load_builtin_manifest("<math.h>").unwrap();
        assert_eq!(m.header, "math.h");
    }

    #[test]
    fn test_cffi_runtime_new() {
        let rt = CFfiRuntime::new();
        assert_eq!(rt.next_id, CFFI_ID_START);
        assert!(rt.functions.is_empty());
        assert!(rt.library.is_none());
    }

    #[test]
    fn test_cffi_runtime_get_function_id_empty() {
        let rt = CFfiRuntime::new();
        assert!(rt.get_function_id("strlen").is_none());
    }
}
