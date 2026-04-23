//! Plan 214: Python FFI Bridge — embed CPython via PyO3
//!
//! Mirrors RustFfiBridge pattern: register Python functions as native shims,
//! marshal arguments/returns as strings through the PyO3 GIL boundary.

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::native::NativeInterface;
use crate::vm::task::AutoTask;
use pyo3::prelude::*;
use std::collections::HashMap;

pub struct PyFfiBridge {
    modules: HashMap<String, Py<PyModule>>,
    functions: HashMap<String, u16>,
    next_native_id: u16,
    native_interface: NativeInterface,
}

impl PyFfiBridge {
    pub fn new() -> Result<Self, VMError> {
        // Ensure Python interpreter is initialized (PyO3 auto-initialize handles this)
        Python::with_gil(|_py| {
            // Just verify the interpreter is available
        });

        Ok(Self {
            modules: HashMap::new(),
            functions: HashMap::new(),
            next_native_id: 400,
            native_interface: NativeInterface::new(),
        })
    }

    pub fn import_module(&mut self, module_name: &str) -> Result<(), VMError> {
        Python::with_gil(|py| {
            let module = py.import(module_name).map_err(|e| {
                VMError::FFI(format!("Failed to import Python module '{}': {}", module_name, e))
            })?;
            self.modules.insert(module_name.to_string(), module.into());
            Ok(())
        })
    }

    pub fn register_function(
        &mut self,
        module_name: &str,
        function_name: &str,
    ) -> Result<u16, VMError> {
        let native_id = self.next_native_id;
        self.next_native_id += 1;

        let qualified = format!("{}.{}", module_name, function_name);
        self.functions.insert(qualified, native_id);

        let module: Py<PyModule> = Python::with_gil(|py| {
            self.modules
                .get(module_name)
                .ok_or_else(|| VMError::FFI(format!("Module {} not imported", module_name)))
                .map(|m| m.clone_ref(py))
        })?;
        let func_name = function_name.to_string();

        let shim = move |task: &mut AutoTask, vm: &AutoVM| {
            // Pop string argument from stack (tagged as -(idx+1))
            let raw = task.ram.pop_i32();
            let str_idx = if raw < 0 { (-(raw) - 1) as usize } else { raw as usize };
            let input_string: Vec<u8> = if let Ok(strings) = vm.strings.read() {
                strings.get(str_idx).cloned().unwrap_or_default()
            } else {
                Vec::new()
            };

            // Call Python function via PyO3
            let result_string = Python::with_gil(|py| {
                let mod_ref = module.bind(py);
                let func: Bound<'_, PyAny> = mod_ref.getattr(&func_name).map_err(|e| {
                    VMError::FFI(format!("Python function '{}' not found: {}", func_name, e))
                })?;

                let py_input = pyo3::types::PyString::new(py, &String::from_utf8_lossy(&input_string));
                let py_result = func.call1((py_input,)).map_err(|e| {
                    VMError::FFI(format!("Python call {}() failed: {}", func_name, e))
                })?;

                let result: String = py_result.extract().map_err(|e| {
                    VMError::FFI(format!("Python return value not a string: {}", e))
                })?;

                Ok::<String, VMError>(result)
            })?;

            // Push result string into VM string pool
            if let Ok(mut strings) = vm.strings.write() {
                let idx = strings.len() as u16;
                strings.push(result_string.into_bytes());
                task.ram.push_i32(-(idx as i32) - 1);
            } else {
                task.ram.push_i32(0);
            }

            Ok(())
        };
        self.native_interface.register_static(native_id, shim);

        Ok(native_id)
    }

    pub fn native_interface(&self) -> &NativeInterface {
        &self.native_interface
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_ffi_bridge_creation() {
        let bridge = PyFfiBridge::new();
        assert!(bridge.is_ok());
    }

    #[test]
    fn test_py_ffi_import_builtin_module() {
        let mut bridge = PyFfiBridge::new().unwrap();
        let result = bridge.import_module("json");
        assert!(result.is_ok());
    }

    #[test]
    fn test_py_ffi_import_and_register() {
        let mut bridge = PyFfiBridge::new().unwrap();
        bridge.import_module("json").unwrap();
        let native_id = bridge.register_function("json", "dumps");
        assert!(native_id.is_ok());
        assert_eq!(native_id.unwrap(), 400);
    }

    #[test]
    fn test_py_ffi_nonexistent_module() {
        let mut bridge = PyFfiBridge::new().unwrap();
        let result = bridge.import_module("nonexistent_module_xyz_12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_py_ffi_register_without_import() {
        let mut bridge = PyFfiBridge::new().unwrap();
        let result = bridge.register_function("nonexistent", "func");
        assert!(result.is_err());
    }
}
