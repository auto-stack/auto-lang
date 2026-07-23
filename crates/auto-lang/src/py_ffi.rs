//! Plan 214/222/300: Python FFI Bridge — embed CPython via PyO3
//!
//! Supports multi-type marshalling: int, float, bool, string, list (Plan 222).
//! Plan 300: Auto-type marshalling via NanoValue tag detection for params and returns.
//! Mirrors RustFfiBridge pattern: register Python functions as native shims.

use crate::py_ffi_types::{PySignature, PyType};
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::native::NativeInterface;
use crate::vm::task::AutoTask;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFloat, PyList, PyString, PyTuple};
use std::collections::HashMap;

pub struct PyFfiBridge {
    modules: HashMap<String, Py<PyModule>>,
    functions: HashMap<String, u16>,
    next_native_id: u16,
    native_interface: NativeInterface,
}

impl PyFfiBridge {
    pub fn new() -> Result<Self, VMError> {
        Python::attach(|_py| {});

        Ok(Self {
            modules: HashMap::new(),
            functions: HashMap::new(),
            next_native_id: 400,
            native_interface: NativeInterface::new(),
        })
    }

    pub fn import_module(&mut self, module_name: &str) -> Result<(), VMError> {
        Python::attach(|py| {
            let module = py.import(module_name).map_err(|e| {
                VMError::FFI(format!("Failed to import Python module '{}': {}", module_name, e))
            })?;
            self.modules.insert(module_name.to_string(), module.into());
            Ok(())
        })
    }

    /// Get a reference to an imported module (for introspection).
    pub fn get_module<'py>(
        &self,
        py: Python<'py>,
        module_name: &str,
    ) -> Option<pyo3::Bound<'py, PyModule>> {
        self.modules.get(module_name).map(|m| m.clone_ref(py).into_bound(py))
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

        let module: Py<PyModule> = Python::attach(|py| {
            self.modules
                .get(module_name)
                .ok_or_else(|| VMError::FFI(format!("Module {} not imported", module_name)))
                .map(|m| m.clone_ref(py))
        })?;
        let func_name = function_name.to_string();
        let return_type = signature.returns.clone();
        let param_types = signature.params.clone();

        let shim = move |task: &mut AutoTask, vm: &AutoVM| {
            Python::attach(|py| {
                let mod_ref = module.bind(py);
                let func = mod_ref.getattr(&func_name).map_err(|e| {
                    VMError::FFI(format!("Python function '{}' not found: {}", func_name, e))
                })?;

                // Build Python argument tuple by popping from stack in reverse.
                // Plan 369 Task 10: use the ACTUAL call-site arg count stashed on
                // the task by the CALL_PY handler, rather than the param_types count
                // baked in at registration. The registration-time count comes from
                // inspect.signature(), which fails for C builtins (datetime.date,
                // struct.pack) and is wrong for variadics (struct.pack). All py-FFI
                // params use Auto-type marshalling (NanoValue tag detection), so each
                // arg is popped via pop_auto_py_arg regardless of the declared type.
                let n = task.pending_native_arg_count as usize;
                // Fallback for shims registered before CALL_PY existed (param count
                // was baked into param_types). Prefer the runtime count when > 0.
                let n = if n > 0 { n } else { param_types.len() };
                let mut bound_args: Vec<Bound<'_, PyAny>> = Vec::with_capacity(n);
                for _ in 0..n {
                    let py_val = pop_auto_py_arg(task, vm, py)?;
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
                        if let Ok(list) = py_result.cast::<PyList>() {
                            py_list_to_vm_heap(list, task, vm)?;
                        } else {
                            return Err(VMError::FFI("Python return not list".to_string()));
                        }
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

    /// Plan 369 Task 11: Register a module-level constant (non-callable attribute)
    /// as a zero-arg native. The emitted shim performs `getattr(module, name)` and
    /// marshals the resulting Python object to the VM stack via the auto path.
    /// Returns the assigned native_id. Pair with codegen that emits CALL_PY with
    /// arg_count=0 for the bare identifier reference.
    pub fn register_constant(
        &mut self,
        module_name: &str,
        const_name: &str,
    ) -> Result<u16, VMError> {
        let native_id = self.next_native_id;
        self.next_native_id += 1;

        let qualified = format!("{}.{}", module_name, const_name);
        self.functions.insert(qualified, native_id);

        let module: Py<PyModule> = Python::attach(|py| {
            self.modules
                .get(module_name)
                .ok_or_else(|| VMError::FFI(format!("Module {} not imported", module_name)))
                .map(|m| m.clone_ref(py))
        })?;
        let const_name = const_name.to_string();

        let shim = move |task: &mut AutoTask, vm: &AutoVM| {
            Python::attach(|py| {
                let mod_ref = module.bind(py);
                let py_val = mod_ref.getattr(&const_name).map_err(|e| {
                    VMError::FFI(format!("Python constant '{}' not found: {}", const_name, e))
                })?;
                // Zero-arg constant: no args to pop. pending_native_arg_count is 0.
                py_auto_marshal_return(&py_val, task, vm)?;
                Ok::<(), VMError>(())
            })?;
            Ok(())
        };
        self.native_interface.register_static(native_id, shim);

        Ok(native_id)
    }

    /// Plan 369 Task 11: Return true if `module.name` is callable (a function/type
    /// with __call__), false if it is a plain constant. Used at registration time
    /// to decide between register_function and register_constant. Returns true on
    /// any introspection failure (preserves prior behavior — only treats genuinely
    /// non-callable attributes as constants).
    pub fn is_callable(&self, module_name: &str, attr_name: &str) -> bool {
        Python::attach(|py| {
            let Some(mod_ref) = self.modules.get(module_name) else {
                return true;
            };
            let Ok(attr) = mod_ref.bind(py).getattr(attr_name) else {
                return true;
            };
            attr.is_callable()
        })
    }

    pub fn native_interface(&self) -> &NativeInterface {
        &self.native_interface
    }

    /// Plan 300: Use Python `inspect.signature()` to get the number of parameters for a function.
    /// Falls back to `default_count` if introspection fails.
    pub fn inspect_param_count(&self, module_name: &str, func_name: &str, default_count: usize) -> usize {
        Python::attach(|py| {
            let Some(mod_ref) = self.modules.get(module_name) else {
                return default_count;
            };
            let Ok(func) = mod_ref.bind(py).getattr(func_name) else {
                return default_count;
            };
            // Count required positional parameters using inspect directly on the function object.
            // Avoid eval() scope issues by using inspect methods directly on the Bound object.
            let Ok(inspect) = py.import("inspect") else {
                return default_count;
            };
            let Ok(sig) = inspect.call_method1("signature", (func,)) else {
                return default_count;
            };
            let Ok(params) = sig.getattr("parameters") else {
                return default_count;
            };
            let Ok(param_empty) = inspect.getattr("_empty") else {
                return default_count;
            };
            // Convert mappingproxy to list and iterate
            let Ok(values_list) = params.call_method0("values")
                .and_then(|v| {
                    let builtins = py.import("builtins")?;
                    builtins.call_method1("list", (v,))
                })
            else {
                return default_count;
            };
            let mut count = 0usize;
            let list_len = values_list.len().unwrap_or(0);
            for i in 0..list_len {
                if let Some(param) = values_list.get_item(i).ok() {
                    // Check if default == Parameter.empty (required param)
                    if let Ok(default_val) = param.getattr("default") {
                        let is_required = default_val.eq(&param_empty).unwrap_or(true);
                        if !is_required {
                            continue;
                        }
                    }
                    // Check kind: POSITIONAL_ONLY(0) or POSITIONAL_OR_KEYWORD(1)
                    if let Ok(kind) = param.getattr("kind") {
                        if let Ok(kind_val) = kind.extract::<i32>() {
                            if kind_val <= 1 {
                                count += 1;
                            }
                        }
                    }
                }
            }
            if count == 0 { default_count } else { count }
        })
    }

    /// Plan 300: Discover all public callable functions in a module.
    /// Returns a list of (func_name, param_count) pairs.
    pub fn discover_module_callables(&self, module_name: &str) -> Vec<(String, usize)> {
        Python::attach(|py| {
            let Some(mod_ref) = self.modules.get(module_name) else {
                return Vec::new();
            };
            let m = mod_ref.bind(py);
            let Ok(builtins) = py.import("builtins") else {
                return Vec::new();
            };
            let Ok(dir_result) = builtins.call_method1("dir", (m,)) else {
                return Vec::new();
            };
            let Ok(names) = dir_result.extract::<Vec<String>>() else {
                return Vec::new();
            };
            let mut callables = Vec::new();
            for name in names {
                if name.starts_with('_') {
                    continue;
                }
                if let Ok(member) = m.getattr(&name as &str) {
                    if member.is_callable() {
                        // Count required positional params using inspect on the member object
                        let param_count = if let Ok(inspect) = py.import("inspect") {
                            if let Ok(sig) = inspect.call_method1("signature", (member,)) {
                                if let Ok(params) = sig.getattr("parameters") {
                                    let param_empty = inspect.getattr("_empty").ok();
                                    if let Ok(values_list) = params.call_method0("values")
                                        .and_then(|v| builtins.call_method1("list", (v,)))
                                    {
                                        let mut c = 0usize;
                                        let list_len = values_list.len().unwrap_or(0);
                                        for i in 0..list_len {
                                            if let Some(p) = values_list.get_item(i).ok() {
                                                let required = if let Some(ref empty) = param_empty {
                                                    p.getattr("default")
                                                        .ok()
                                                        .map_or(true, |d| d.eq(empty).unwrap_or(true))
                                                } else { true };
                                                let kind_ok = p.getattr("kind")
                                                    .ok()
                                                    .and_then(|k| k.extract::<i32>().ok())
                                                    .map_or(true, |k| k <= 1);
                                                if required && kind_ok { c += 1; }
                                            }
                                        }
                                        c
                                    } else { 1 }
                                } else { 1 }
                            } else { 1 }
                        } else { 1 };
                        callables.push((name, param_count));
                    }
                }
            }
            callables
        })
    }
}

/// Pop a single argument from the VM stack and convert to a Python object,
/// using the NanoValue tag to determine the actual type at runtime.
/// Plan 300: Replaces fixed-type popping for Python FFI auto-type marshalling.
fn pop_auto_py_arg<'py>(
    task: &mut AutoTask,
    vm: &AutoVM,
    py: Python<'py>,
) -> Result<Bound<'py, PyAny>, VMError> {
    // Check for 2-slot f64: TOS is null padding, slot below is raw f64 bits.
    // This mirrors pop_arith_operand() in virt_memory.rs.
    let tos = task.ram.peek_nv(0);
    if auto_val::is_null(tos) && task.ram.sp > 1 {
        let below = task.ram.peek_nv(1);
        if !auto_val::is_nanboxed(below) {
            // This is a 2-slot f64
            let val = task.ram.pop_f64();
            return Ok(PyFloat::new(py, val).into_any());
        }
    }

    // Single-slot NanoValue — check the tag
    let nv = task.ram.pop_nv();
    let tag = auto_val::tag_of(nv);

    match tag {
        1 => {
            // TAG_I32
            let val = auto_val::decode_i32(nv);
            Ok(val.into_pyobject(py).unwrap().into_any())
        }
        2 => {
            // TAG_STRING — look up in string pool
            let str_idx = auto_val::decode_string(nv) as usize;
            let s = if let Ok(strings) = vm.strings.read() {
                strings.get(str_idx).cloned().unwrap_or_default()
            } else {
                Vec::new()
            };
            let s = String::from_utf8_lossy(&s).to_string();
            Ok(PyString::new(py, &s).into_any())
        }
        3 => {
            // TAG_BOOL — construct Python bool directly
            let val = auto_val::decode_bool(nv);
            Ok(pyo3::types::PyBool::new(py, val).to_owned().into_any())
        }
        4 => {
            // TAG_NULL
            Ok(py.None().into_bound(py))
        }
        5 => {
            // TAG_OBJECT — heap object, try to convert RustStdlibObject<Obj> to Python dict
            let obj_id = auto_val::decode_object(nv) as u64;
            if let Some(heap_obj) = vm.get_heap_object(obj_id) {
                let guard = heap_obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                    if let Some(obj) = rust_obj.downcast_ref::<auto_val::Obj>() {
                        let dict = PyDict::new(py);
                        for (k, v) in obj.iter() {
                            let py_key = match k {
                                auto_val::ValueKey::Str(s) => s.to_string(),
                                auto_val::ValueKey::Int(i) => i.to_string(),
                                auto_val::ValueKey::Bool(b) => b.to_string(),
                            };
                            let py_val = value_to_py(v, py);
                            dict.set_item(&py_key, py_val).map_err(|e| {
                                VMError::FFI(format!("Failed to set dict item: {}", e))
                            })?;
                        }
                        return Ok(dict.into_any());
                    }
                }
            }
            // Unknown object type — fall back to None
            Ok(py.None().into_bound(py))
        }
        6 => {
            // TAG_LIST — heap list, try to convert to Python list
            let list_id = auto_val::decode_list(nv) as u64;
            if let Some(heap_obj) = vm.get_heap_object(list_id) {
                let guard = heap_obj.read().unwrap();
                use crate::vm::heap_object::downcast;
                if let Some(list_data) = downcast::<crate::vm::types::ListData<auto_val::Value>>(&*guard) {
                    let py_list = PyList::empty(py);
                    for v in &list_data.elems {
                        let py_val = value_to_py(v, py);
                        py_list.append(py_val).map_err(|e| {
                            VMError::FFI(format!("Failed to append to Python list: {}", e))
                        })?;
                    }
                    return Ok(py_list.into_any());
                }
            }
            Ok(py.None().into_bound(py))
        }
        7 => {
            // TAG_F32
            let val = auto_val::decode_f32(nv);
            Ok(PyFloat::new(py, val as f64).into_any())
        }
        _ => {
            // Unknown tag — fall back to None
            Ok(py.None().into_bound(py))
        }
    }
}

/// Convert an AutoVal Value to a Python object (for passing VM values as Python args).
fn value_to_py<'py>(val: &auto_val::Value, py: Python<'py>) -> Bound<'py, PyAny> {
    match val {
        auto_val::Value::Int(i) => i.into_pyobject(py).unwrap().into_any(),
        auto_val::Value::Uint(u) => (*u as i32).into_pyobject(py).unwrap().into_any(),
        auto_val::Value::Float(f) | auto_val::Value::Double(f) => {
            PyFloat::new(py, *f).into_any()
        }
        auto_val::Value::Bool(b) => pyo3::types::PyBool::new(py, *b).to_owned().into_any(),
        auto_val::Value::Str(s) => PyString::new(py, s.as_str()).into_any(),
        auto_val::Value::Nil | auto_val::Value::Null | auto_val::Value::None => {
            py.None().into_bound(py)
        }
        _ => py.None().into_bound(py),
    }
}

/// Auto-detect Python return type and marshal to VM stack.
/// Plan 300: Enhanced with dict→Obj and nested structure support.
fn py_auto_marshal_return(
    py_val: &Bound<'_, PyAny>,
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    // Order matters: use is_instance_of for bool to avoid float/int confusion
    // In Python, bool is a subclass of int, so extract::<bool>() can succeed for floats
    if py_val.is_instance_of::<pyo3::types::PyBool>() {
        let b: bool = py_val.extract().unwrap_or(false);
        task.ram.push_i32(if b { 1 } else { 0 });
    } else if let Ok(i) = py_val.extract::<i32>() {
        task.ram.push_i32(i);
    } else if let Ok(f) = py_val.extract::<f64>() {
        // Plan 300: Store float as string in string pool because codegen assumes
        // Python FFI returns are single-slot values. The 2-slot f64 encoding
        // breaks `let x = py_func()` (only stores 1 slot = null marker).
        // TODO: When codegen supports multi-slot return types, switch to push_f64.
        let s = if f == f.floor() && f.abs() < 1e15 {
            format!("{}", f as i64)
        } else {
            format!("{}", f)
        };
        if let Ok(mut strings) = vm.strings.write() {
            let idx = strings.len() as u32;
            strings.push(s.into_bytes());
            task.ram.push_str_idx(idx);
        } else {
            task.ram.push_i32(0);
        }
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
    } else if py_val.is_instance_of::<PyDict>() {
        // pyo3 0.29: cast() returns reference, borrow for heap conversion
        let dict = py_val.cast::<PyDict>().map_err(|e| {
            VMError::FFI(format!("Cast to PyDict failed: {}", e))
        })?;
        py_dict_to_vm_heap(dict, task, vm)?;
    } else if py_val.is_instance_of::<PyList>() {
        let list = py_val.cast::<PyList>().map_err(|e| {
            VMError::FFI(format!("Cast to PyList failed: {}", e))
        })?;
        py_list_to_vm_heap(list, task, vm)?;
    } else {
        // Fallback: convert to string repr
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

/// Convert a Python dict to a VM heap object and push its ID.
/// Plan 300: Uses RustStdlibObject to wrap auto_val::Obj for generic storage.
fn py_dict_to_vm_heap(
    py_dict: &Bound<'_, PyDict>,
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    let mut obj = auto_val::Obj::new();
    for (key, value) in py_dict.iter() {
        let key_str: String = key.extract().map_err(|e| {
            VMError::FFI(format!("Dict key not string: {}", e))
        })?;
        let val = py_any_to_value(&value, vm)?;
        obj.set(auto_val::ValueKey::from(key_str.as_str()), val);
    }

    let wrapped = crate::vm::ffi::rust_stdlib::RustStdlibObject::new("PyDict", obj);
    let id = vm.insert_heap_object(wrapped);
    task.ram.push_nv(auto_val::encode_object(id as u32));
    Ok(())
}

/// Convert a Python list to a VM heap List object and push its ID.
fn py_list_to_vm_heap(
    py_list: &Bound<'_, PyList>,
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let mut values = Vec::new();
    for item in py_list.iter() {
        values.push(py_any_to_value(&item, vm)?);
    }

    let list = ListData::<auto_val::Value> {
        elems: values,
        storage: None,
    };
    let id = vm.insert_heap_object(list);
    task.ram.push_nv(auto_val::encode_list(id as u32));
    Ok(())
}

/// Recursively convert a Python value to an AutoVal Value.
/// Handles: bool, int, float, string, None, list, dict (nested).
fn py_any_to_value(
    py_val: &Bound<'_, PyAny>,
    vm: &AutoVM,
) -> Result<auto_val::Value, VMError> {
    // bool before int (bool is int subclass in Python)
    if let Ok(b) = py_val.extract::<bool>() {
        return Ok(auto_val::Value::Bool(b));
    }
    if let Ok(i) = py_val.extract::<i32>() {
        return Ok(auto_val::Value::Int(i));
    }
    if let Ok(f) = py_val.extract::<f64>() {
        return Ok(auto_val::Value::Double(f));
    }
    if let Ok(s) = py_val.extract::<String>() {
        return Ok(auto_val::Value::Str(s.into()));
    }
    if py_val.is_none() {
        return Ok(auto_val::Value::Nil);
    }
    // Nested list
    if let Ok(list) = py_val.cast::<PyList>() {
        let mut values = Vec::new();
        for item in list.iter() {
            values.push(py_any_to_value(&item, vm)?);
        }
        return Ok(auto_val::Value::Array(auto_val::Array::from(values)));
    }
    // Nested dict
    if let Ok(dict) = py_val.cast::<PyDict>() {
        let mut obj = auto_val::Obj::new();
        for (key, value) in dict.iter() {
            let key_str: String = key.extract().unwrap_or_default();
            let val = py_any_to_value(&value, vm)?;
            obj.set(auto_val::ValueKey::from(key_str.as_str()), val);
        }
        return Ok(auto_val::Value::Obj(obj));
    }
    // Fallback: string representation
    let s = format!("{:?}", py_val);
    Ok(auto_val::Value::Str(s.into()))
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

    #[test]
    fn test_py_all_auto_registration() {
        let mut bridge = PyFfiBridge::new().unwrap();
        bridge.import_module("math").unwrap();
        let sig = PySignature::all_auto(1);
        let native_id = bridge.register_function("math", "sqrt", sig);
        assert!(native_id.is_ok());
        assert_eq!(native_id.unwrap(), 400);
    }

    #[test]
    fn test_py_inspect_param_count() {
        // Test that we can get param count via inspect
        let result = Python::attach(|py| {
            let math = py.import("math").unwrap();
            let sqrt = math.getattr("sqrt").unwrap();
            let inspect = py.import("inspect").unwrap();
            let sig = inspect.call_method1("signature", (sqrt,)).unwrap();
            let params = sig.getattr("parameters").unwrap();
            // `parameters` is a mappingproxy, not a plain dict — use len()
            let builtins = py.import("builtins").unwrap();
            let len_result = builtins.call_method1("len", (params,)).unwrap();
            Some(len_result.extract::<usize>().unwrap())
        });
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_py_inspect_multi_param() {
        // Test with multi-param function
        let result = Python::attach(|py| {
            let random = py.import("random").unwrap();
            let randint = random.getattr("randint").unwrap();
            let inspect = py.import("inspect").unwrap();
            let sig = inspect.call_method1("signature", (randint,)).unwrap();
            let params = sig.getattr("parameters").unwrap();
            // `parameters` is a mappingproxy, not a plain dict — use len()
            let builtins = py.import("builtins").unwrap();
            let len_result = builtins.call_method1("len", (params,)).unwrap();
            Some(len_result.extract::<usize>().unwrap())
        });
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_py_value_to_py_roundtrip() {
        // Test AutoVal Value → Python → extract round-trip
        Python::attach(|py| {
            // Int round-trip
            let val = auto_val::Value::Int(42);
            let py_val = value_to_py(&val, py);
            let back: i32 = py_val.extract().unwrap();
            assert_eq!(back, 42);

            // Bool round-trip
            let val = auto_val::Value::Bool(true);
            let py_val = value_to_py(&val, py);
            let back: bool = py_val.extract().unwrap();
            assert!(back);

            // Float round-trip
            let val = auto_val::Value::Double(3.14);
            let py_val = value_to_py(&val, py);
            let back: f64 = py_val.extract().unwrap();
            assert!((back - 3.14).abs() < 0.001);

            // String round-trip
            let val = auto_val::Value::Str("hello".into());
            let py_val = value_to_py(&val, py);
            let back: String = py_val.extract().unwrap();
            assert_eq!(back, "hello");
        });
    }
}
