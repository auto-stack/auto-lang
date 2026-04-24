//! Plan 214/222: Python FFI Bridge — embed CPython via PyO3
//!
//! Supports multi-type marshalling: int, float, bool, string, list (Plan 222).
//! Mirrors RustFfiBridge pattern: register Python functions as native shims.

use crate::py_ffi_types::{PySignature, PyType};
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::native::NativeInterface;
use crate::vm::task::AutoTask;
use pyo3::prelude::*;
use pyo3::types::{PyFloat, PyList, PyString, PyTuple};
use std::collections::HashMap;

pub struct PyFfiBridge {
    modules: HashMap<String, Py<PyModule>>,
    functions: HashMap<String, u16>,
    next_native_id: u16,
    native_interface: NativeInterface,
}

impl PyFfiBridge {
    pub fn new() -> Result<Self, VMError> {
        Python::with_gil(|_py| {});

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
        signature: PySignature,
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
        let return_type = signature.returns.clone();
        let param_types = signature.params.clone();

        let shim = move |task: &mut AutoTask, vm: &AutoVM| {
            Python::with_gil(|py| {
                let mod_ref = module.bind(py);
                let func = mod_ref.getattr(&func_name).map_err(|e| {
                    VMError::FFI(format!("Python function '{}' not found: {}", func_name, e))
                })?;

                // Build Python argument tuple by popping from stack in reverse
                let n = param_types.len();
                let mut bound_args: Vec<Bound<'_, PyAny>> = Vec::with_capacity(n);
                for pt in param_types.iter().rev() {
                    let py_val = match pt {
                        PyType::Int => {
                            let val = task.ram.pop_i32();
                            val.into_pyobject(py).unwrap().into_any()
                        }
                        PyType::Float => {
                            let val = task.ram.pop_f64();
                            PyFloat::new(py, val).into_any()
                        }
                        PyType::Bool => {
                            let val = task.ram.pop_i32();
                            val.into_pyobject(py).unwrap().into_any()
                        }
                        PyType::String => {
                            let str_idx = task.ram.pop_str_idx();
                            let s = if let Ok(strings) = vm.strings.read() {
                                strings.get(str_idx).cloned().unwrap_or_default()
                            } else {
                                Vec::new()
                            };
                            let s = String::from_utf8_lossy(&s).to_string();
                            PyString::new(py, &s).into_any()
                        }
                        PyType::None => py.None().into_bound(py),
                        _ => py.None().into_bound(py),
                    };
                    bound_args.push(py_val);
                }
                bound_args.reverse();

                // Call with PyTuple
                let args_tuple = PyTuple::new(py, bound_args).map_err(|e| {
                    VMError::FFI(format!("Failed to create Python args tuple: {}", e))
                })?;
                let py_result = func.call1(args_tuple).map_err(|e| {
                    VMError::FFI(format!("Python call {}() failed: {}", func_name, e))
                })?;

                // Marshal return value to VM stack
                match return_type {
                    PyType::Int => {
                        let val: i32 = py_result.extract().map_err(|e| {
                            VMError::FFI(format!("Python return not int: {}", e))
                        })?;
                        task.ram.push_i32(val);
                    }
                    PyType::Float => {
                        let val: f64 = py_result.extract().map_err(|e| {
                            VMError::FFI(format!("Python return not float: {}", e))
                        })?;
                        task.ram.push_f64(val);
                    }
                    PyType::Bool => {
                        let val: bool = py_result.extract().map_err(|e| {
                            VMError::FFI(format!("Python return not bool: {}", e))
                        })?;
                        task.ram.push_i32(if val { 1 } else { 0 });
                    }
                    PyType::String => {
                        let val: String = py_result.extract().map_err(|e| {
                            VMError::FFI(format!("Python return not string: {}", e))
                        })?;
                        if let Ok(mut strings) = vm.strings.write() {
                            let idx = strings.len() as u32;
                            strings.push(val.into_bytes());
                            task.ram.push_str_idx(idx);
                        } else {
                            task.ram.push_i32(0);
                        }
                    }
                    PyType::None => {
                        task.ram.push_i32(0);
                    }
                    PyType::List => {
                        let list = py_result.downcast::<PyList>().map_err(|e| {
                            VMError::FFI(format!("Python return not list: {}", e))
                        })?;
                        py_list_to_vm_heap(list, task, vm)?;
                    }
                    PyType::Auto => {
                        py_auto_marshal_return(&py_result, task, vm)?;
                    }
                }
                Ok::<(), VMError>(())
            })?;

            Ok(())
        };
        self.native_interface.register_static(native_id, shim);

        Ok(native_id)
    }

    pub fn native_interface(&self) -> &NativeInterface {
        &self.native_interface
    }
}

/// Auto-detect Python return type and marshal to VM stack.
fn py_auto_marshal_return(
    py_val: &Bound<'_, PyAny>,
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    // Order matters: bool before int (bool is int subclass)
    if let Ok(b) = py_val.extract::<bool>() {
        task.ram.push_i32(if b { 1 } else { 0 });
    } else if let Ok(i) = py_val.extract::<i32>() {
        task.ram.push_i32(i);
    } else if let Ok(f) = py_val.extract::<f64>() {
        task.ram.push_f64(f);
    } else if let Ok(s) = py_val.extract::<String>() {
        if let Ok(mut strings) = vm.strings.write() {
            let idx = strings.len() as u32;
            strings.push(s.into_bytes());
            task.ram.push_str_idx(idx);
        } else {
            task.ram.push_i32(0);
        }
    } else if py_val.is_none() {
        task.ram.push_i32(0);
    } else if let Ok(list) = py_val.downcast::<PyList>() {
        py_list_to_vm_heap(list, task, vm)?;
    } else {
        // Fallback: convert to string
        let s = format!("{:?}", py_val);
        if let Ok(mut strings) = vm.strings.write() {
            let idx = strings.len() as u32;
            strings.push(s.into_bytes());
            task.ram.push_str_idx(idx);
        } else {
            task.ram.push_i32(0);
        }
    }
    Ok(())
}

/// Convert a Python list to a VM heap List object and push its ID.
fn py_list_to_vm_heap(
    py_list: &Bound<'_, PyList>,
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    use auto_val::Value;

    let mut values = Vec::new();
    for item in py_list.iter() {
        // Try bool first (int subclass)
        if let Ok(b) = item.extract::<bool>() {
            values.push(Value::Bool(b));
        } else if let Ok(i) = item.extract::<i32>() {
            values.push(Value::Int(i));
        } else if let Ok(f) = item.extract::<f64>() {
            values.push(Value::Double(f));
        } else if let Ok(s) = item.extract::<String>() {
            values.push(Value::Str(s.into()));
        } else {
            values.push(Value::Nil);
        }
    }

    let list = ListData::<Value> {
        elems: values,
        storage: None,
    };
    let id = vm.insert_heap_object(list);
    task.ram.push_i32(id as i32);
    Ok(())
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
        let native_id = bridge.register_function("json", "dumps", PySignature::default_string_string());
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
        let result = bridge.register_function("nonexistent", "func", PySignature::default_string_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_py_signature_int_float() {
        let mut bridge = PyFfiBridge::new().unwrap();
        bridge.import_module("math").unwrap();
        let sig = PySignature::new().param(PyType::Float).returns(PyType::Float);
        let native_id = bridge.register_function("math", "sqrt", sig);
        assert!(native_id.is_ok());
        assert_eq!(native_id.unwrap(), 400);
    }

    #[test]
    fn test_py_signature_auto_return() {
        let mut bridge = PyFfiBridge::new().unwrap();
        bridge.import_module("builtins").unwrap();
        let sig = PySignature::new().param(PyType::String).returns(PyType::Auto);
        let native_id = bridge.register_function("builtins", "len", sig);
        assert!(native_id.is_ok());
    }
}
