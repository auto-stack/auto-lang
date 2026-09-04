//! Plan 214/222/300: Python FFI Bridge — embed CPython via PyO3
//!
//! Supports multi-type marshalling: int, float, bool, string, list (Plan 222).
//! Plan 300: Auto-type marshalling via NanoValue tag detection for params and returns.
//! Mirrors RustFfiBridge pattern: register Python functions as native shims.

use crate::py_ffi_types::{PySignature, PyType};
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::heap_object::{HeapObject, TypeTag};
use crate::vm::native::NativeInterface;
use crate::vm::task::AutoTask;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFloat, PyList, PyString, PyTuple};
use std::collections::HashMap;

// ============================================================================
// Plan 369 Task 12: Opaque Python object handle
// ============================================================================

/// Opaque wrapper around a Python object (`Py<PyAny>`) stored in the VM heap.
///
/// When a py-FFI call returns a Python object that doesn't map to a native VM
/// scalar/list/dict (e.g. `datetime.date`, a custom class instance), we wrap it
/// in `PyObjectHandle` and push its heap ID onto the stack instead of stringifying
/// it. Later `py_call` / `py_getattr` built-ins resolve the handle back to the
/// live Python object and dispatch attribute/method access through CPython.
///
/// # Thread safety
/// `Py<PyAny>` is `Send + Sync` in PyO3 0.29 (the owned, reference-counted handle
/// is independent of the GIL). Accessing the underlying object still requires
/// acquiring the GIL via `Python::attach`, which every shim does.
pub struct PyObjectHandle {
    /// Type name from `type(obj).__name__` (captured at creation for debugging).
    pub type_name: String,
    /// The owned Python object reference. Safe to store across threads; GIL
    /// required to dereference.
    pub obj: Py<PyAny>,
}

impl PyObjectHandle {
    pub fn new(type_name: String, obj: Py<PyAny>) -> Self {
        Self { type_name, obj }
    }
}

impl HeapObject for PyObjectHandle {
    fn type_tag(&self) -> TypeTag {
        TypeTag::RustStdlib(format!("PyObj({})", self.type_name))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Native IDs reserved for the py-object built-in shims. These are assigned out
/// of the PyFfiBridge id space (starting at 400) but are stable constants so the
/// codegen can register them in `BIGVM_NATIVES` without consulting the bridge.
pub const NATIVE_PY_CALL: u16 = 450;
pub const NATIVE_PY_GETATTR: u16 = 451;
/// Plan 539 W0 (DIV-PY-KWARGS-1): keyword-argument method-call channel.
/// Codegen lowers `py_call(obj, "m", pos..., k=v...)` into this shim's fixed
/// 5-slot convention so the CALL_PY arg-count byte stays a plain slot count
/// (no sentinel sniffing inside the variadic py_call convention).
pub const NATIVE_PY_CALL_KW: u16 = 452;

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
                        // Plan 510 G1-2: 走 add_string + 配平入栈
                        // (裸推无 dedup 无 rc,消费侧 POP 即多扣)。
                        let idx = vm.add_string(val.into_bytes());
                        vm.rc_push_str_idx(task, idx);
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

    /// Plan 369 Task 12: Register the `py_call(obj, method_name, ...args)` and
    /// `py_getattr(obj, attr_name)` built-in shims. These enable method calls and
    /// attribute access on opaque Python objects returned by earlier py-FFI calls
    /// (wrapped as `PyObjectHandle` in the VM heap).
    ///
    /// Both shims use runtime arg-count detection (via `pending_native_arg_count`,
    /// same mechanism as `CALL_PY`) so `py_call` can accept a variable number of
    /// positional method args.
    ///
    /// Calling convention (args pushed left-to-right, popped TOS-first):
    ///   py_call:   [obj, method_name, arg1, arg2, ...]
    ///   py_getattr:[obj, attr_name]
    ///
    /// The return value is marshalled via `py_auto_marshal_return`, so a method
    /// returning another Python object stays opaque (chainable).
    pub fn register_object_shims(&mut self) {
        // ---- py_call(obj, method_name, ...args) ----
        let call_shim = move |task: &mut AutoTask, vm: &AutoVM| {
            Python::attach(|py| {
                // Layout (TOS → bottom): argN ... arg1, method_name, obj
                // pending_native_arg_count = total args including obj & method_name.
                let n = task.pending_native_arg_count as usize;
                // Defensive: need at least obj + method_name.
                if n < 2 {
                    return Err(VMError::FFI(format!(
                        "py_call needs at least 2 args (obj, method), got {}", n
                    )));
                }
                let extra = n - 2; // method args between method_name and obj

                // Pop method args (in TOS-first order).
                let mut method_args: Vec<Bound<'_, PyAny>> = Vec::with_capacity(extra);
                for _ in 0..extra {
                    method_args.push(pop_auto_py_arg(task, vm, py)?);
                }
                method_args.reverse();

                // Pop method name (must be a string).
                let method_py = pop_auto_py_arg(task, vm, py)?;
                let method_name: String = method_py.extract().map_err(|e| {
                    VMError::FFI(format!("py_call method name not string: {}", e))
                })?;

                // Pop obj (should be a PyObjectHandle).
                let obj_py = pop_auto_py_arg(task, vm, py)?;

                let result = if method_args.is_empty() {
                    obj_py.call_method0(&method_name).map_err(|e| {
                        VMError::FFI(format!(
                            "Python method {}.{}() failed: {}",
                            safe_type_name(&obj_py), method_name, e
                        ))
                    })?
                } else {
                    // pyo3 0.29: call_method1 takes a PyCallArgs tuple. Build a
                    // PyTuple from the collected args so variadic method calls
                    // (any arity) work without per-arity monomorphization.
                    let args_tuple = PyTuple::new(py, &method_args).map_err(|e| {
                        VMError::FFI(format!("Failed to build method args tuple: {}", e))
                    })?;
                    obj_py
                        .call_method1(&method_name, args_tuple)
                        .map_err(|e| {
                            VMError::FFI(format!(
                                "Python method {}.{}() failed: {}",
                                safe_type_name(&obj_py), method_name, e
                            ))
                        })?
                };

                py_auto_marshal_return(&result, task, vm)?;
                Ok::<(), VMError>(())
            })?;
            Ok(())
        };
        self.native_interface.register_static(NATIVE_PY_CALL, call_shim);

        // ---- py_getattr(obj, attr_name) ----
        let getattr_shim = move |task: &mut AutoTask, vm: &AutoVM| {
            Python::attach(|py| {
                // Layout (TOS → bottom): attr_name, obj
                let n = task.pending_native_arg_count as usize;
                if n < 2 {
                    return Err(VMError::FFI(format!(
                        "py_getattr needs 2 args (obj, attr), got {}", n
                    )));
                }
                // Any surplus args beyond the first two are ignored (defensive).

                // Pop attr name (string).
                let attr_py = pop_auto_py_arg(task, vm, py)?;
                let attr_name: String = attr_py.extract().map_err(|e| {
                    VMError::FFI(format!("py_getattr attr name not string: {}", e))
                })?;

                // Pop obj (PyObjectHandle).
                let obj_py = pop_auto_py_arg(task, vm, py)?;

                let result = obj_py.getattr(&attr_name).map_err(|e| {
                    VMError::FFI(format!(
                        "Python getattr({}.{}) failed: {}",
                        safe_type_name(&obj_py), attr_name, e
                    ))
                })?;

                py_auto_marshal_return(&result, task, vm)?;
                Ok::<(), VMError>(())
            })?;
            Ok(())
        };
        self.native_interface
            .register_static(NATIVE_PY_GETATTR, getattr_shim);

        // ---- py_call_kw(obj, method_name, posargs, kw_names, kw_vals) ----
        // Plan 539 W0 (DIV-PY-KWARGS-1): keyword-argument channel. The codegen
        // rewrites `py_call(obj, "m", pos..., k=v...)` into this fixed 5-slot
        // convention (posargs/kw_names/kw_vals are Auto lists marshalled by
        // pop_auto_py_arg), so Python receives real keyword arguments instead
        // of the historic silent name-drop.
        let call_kw_shim = move |task: &mut AutoTask, vm: &AutoVM| {
            Python::attach(|py| {
                // Layout (TOS → bottom): kw_vals, kw_names, posargs, method_name, obj
                let n = task.pending_native_arg_count as usize;
                if n != 5 {
                    return Err(VMError::FFI(format!(
                        "py_call_kw needs 5 slots (obj, method, posargs, kw_names, kw_vals), got {}",
                        n
                    )));
                }

                let kw_vals_py = pop_auto_py_arg(task, vm, py)?;
                let kw_names_py = pop_auto_py_arg(task, vm, py)?;
                let posargs_py = pop_auto_py_arg(task, vm, py)?;
                let method_py = pop_auto_py_arg(task, vm, py)?;
                let method_name: String = method_py.extract().map_err(|e| {
                    VMError::FFI(format!("py_call_kw method name not string: {}", e))
                })?;
                let obj_py = pop_auto_py_arg(task, vm, py)?;

                let kwargs = build_kwargs(&kw_names_py, &kw_vals_py)
                    .map_err(|e| VMError::FFI(format!("py_call_kw kwargs build failed: {}", e)))?;

                let pos_vec: Vec<Bound<'_, PyAny>> = posargs_py
                    .try_iter()
                    .map_err(|e| VMError::FFI(format!("py_call_kw posargs not iterable: {}", e)))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| VMError::FFI(format!("py_call_kw posargs iteration failed: {}", e)))?;
                let args_tuple = PyTuple::new(py, &pos_vec).map_err(|e| {
                    VMError::FFI(format!("Failed to build method args tuple: {}", e))
                })?;

                let result = obj_py
                    .call_method(&method_name, args_tuple, Some(&kwargs))
                    .map_err(|e| {
                        VMError::FFI(format!(
                            "Python method {}.{}(**kw) failed: {}",
                            safe_type_name(&obj_py), method_name, e
                        ))
                    })?;

                py_auto_marshal_return(&result, task, vm)?;
                Ok::<(), VMError>(())
            })?;
            Ok(())
        };
        self.native_interface
            .register_static(NATIVE_PY_CALL_KW, call_kw_shim);
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
            // TAG_OBJECT — heap object. Could be a RustStdlibObject<Obj> (Auto
            // dict) or a PyObjectHandle (opaque live Python object).
            let obj_id = auto_val::decode_object(nv) as u64;
            if let Some(heap_obj) = vm.get_heap_object(obj_id) {
                let guard = heap_obj.read().unwrap();
                // Plan 369 Task 12: opaque Python object — clone out the owned
                // `Py<PyAny>` and bind it to the current GIL scope.
                if let Some(py_handle) = guard.as_any().downcast_ref::<PyObjectHandle>() {
                    // Clone the Py<PyAny> (increments refcount) and bind under GIL.
                    let owned = py_handle.obj.clone_ref(py);
                    return Ok(owned.into_bound(py));
                }
                if let Some(rust_obj) = guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                    if let Some(obj) = rust_obj.downcast_ref::<auto_val::Obj>() {
                        let dict = PyDict::new(py);
                        for (k, v) in obj.iter() {
                            let py_key = match k {
                                auto_val::ValueKey::Str(s) => s.to_string(),
                                auto_val::ValueKey::Int(i) => i.to_string(),
                                auto_val::ValueKey::Bool(b) => b.to_string(),
                            };
                            let py_val = value_to_py(v, py, vm);
                            dict.set_item(&py_key, py_val).map_err(|e| {
                                VMError::FFI(format!("Failed to set dict item: {}", e))
                            })?;
                        }
                        return Ok(dict.into_any());
                    }
                }
                // Plan 539 W0 (DIV-PY-AUTOLIST-1): array literals are
                // TAG_OBJECT-encoded ListData (CREATE_ARRAY pushes
                // encode_object, not encode_list) — this historic miss
                // marshalled every Auto array argument to a single None.
                // Clone the elements out, release the guard, then marshal so
                // nested VmRef lookups never stack read locks.
                if let Some(list_data) = guard
                    .as_any()
                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                {
                    let elems = list_data.elems.clone();
                    drop(guard);
                    let py_list = PyList::empty(py);
                    for v in &elems {
                        let py_val = value_to_py(v, py, vm);
                        py_list.append(py_val).map_err(|e| {
                            VMError::FFI(format!("Failed to append to Python list: {}", e))
                        })?;
                    }
                    return Ok(py_list.into_any());
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
                        let py_val = value_to_py(v, py, vm);
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

/// Best-effort Python type name for error messages. Returns "?" on failure.
fn safe_type_name(obj: &Bound<'_, PyAny>) -> String {
    obj.get_type()
        .name()
        .map(|n| n.to_string())
        .unwrap_or_else(|_| "?".to_string())
}

/// Plan 539 W0 (DIV-PY-KWARGS-1): zip a names list and a values list (both
/// already marshalled to Python sequences by pop_auto_py_arg) into a PyDict
/// suitable as the kwargs argument of a pyo3 call.
fn build_kwargs<'py>(
    kw_names: &Bound<'py, PyAny>,
    kw_vals: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyDict>> {
    let names: Vec<String> = kw_names
        .try_iter()?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|n: Bound<'_, PyAny>| n.extract())
        .collect::<Result<Vec<_>, _>>()?;
    let vals: Vec<Bound<'py, PyAny>> = kw_vals
        .try_iter()?
        .collect::<Result<Vec<_>, _>>()?;
    let dict = PyDict::new(kw_names.py());
    for (name, val) in names.into_iter().zip(vals.into_iter()) {
        dict.set_item(name.as_str(), val)?;
    }
    Ok(dict)
}

/// Convert an AutoVal Value to a Python object (for passing VM values as Python args).
/// Plan 539 W0 (DIV-PY-AUTOLIST-1): handles nested containers — Array/Block →
/// PyList, Obj → PyDict, VmRef → heap-object resolution (nested arrays inside
/// arrays are stored as VmRef ids by CREATE_ARRAY).
fn value_to_py<'py>(val: &auto_val::Value, py: Python<'py>, vm: &AutoVM) -> Bound<'py, PyAny> {
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
        auto_val::Value::Array(a) | auto_val::Value::Block(a) => {
            let py_list = PyList::empty(py);
            for v in &a.values {
                let py_val = value_to_py(v, py, vm);
                let _ = py_list.append(py_val);
            }
            py_list.into_any()
        }
        auto_val::Value::Obj(o) => {
            let dict = PyDict::new(py);
            for (k, v) in o.iter() {
                let py_key = match k {
                    auto_val::ValueKey::Str(s) => s.to_string(),
                    auto_val::ValueKey::Int(i) => i.to_string(),
                    auto_val::ValueKey::Bool(b) => b.to_string(),
                };
                let py_val = value_to_py(v, py, vm);
                let _ = dict.set_item(py_key, py_val);
            }
            dict.into_any()
        }
        auto_val::Value::VmRef(r) => {
            let Some(heap_obj) = vm.get_heap_object(r.id as u64) else {
                return py.None().into_bound(py);
            };
            // Clone the payload out, release the guard, then recurse so nested
            // VmRef lookups never stack read locks on the heap.
            let guard = heap_obj.read().unwrap();
            if let Some(list_data) = guard
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
            {
                let elems = list_data.elems.clone();
                drop(guard);
                let py_list = PyList::empty(py);
                for v in &elems {
                    let _ = py_list.append(value_to_py(v, py, vm));
                }
                return py_list.into_any();
            }
            if let Some(rust_obj) = guard
                .as_any()
                .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
            {
                if let Some(obj) = rust_obj.downcast_ref::<auto_val::Obj>() {
                    let entries: Vec<(String, auto_val::Value)> = obj
                        .iter()
                        .map(|(k, v)| {
                            let key = match k {
                                auto_val::ValueKey::Str(s) => s.to_string(),
                                auto_val::ValueKey::Int(i) => i.to_string(),
                                auto_val::ValueKey::Bool(b) => b.to_string(),
                            };
                            (key, v.clone())
                        })
                        .collect();
                    drop(guard);
                    let dict = PyDict::new(py);
                    for (k, v) in &entries {
                        let _ = dict.set_item(k, value_to_py(v, py, vm));
                    }
                    return dict.into_any();
                }
            }
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
        // Plan 539 W0 (DIV-PY-FLOAT-1): floats return as a real f64. The
        // historic string-pool round-trip existed because the 2-slot f64
        // encoding broke `let x = py_func()` — obsolete since Plan 377 made
        // f64 single-slot. It dropped ".0" on integral values and silently
        // degraded arithmetic on the stringified result to NaN.
        task.ram.push_f64(f);
    } else if let Ok(s) = py_val.extract::<String>() {
        let idx = vm.add_string(s.into_bytes());
        vm.rc_push_str_idx(task, idx);
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
        // Plan 369 Task 12: opaque Python object. Keep the live Python object
        // in the VM heap as a `PyObjectHandle` so later `py_call` / `py_getattr`
        // can dispatch method/attribute access through CPython. Previously this
        // stringified the object (e.g. `datetime.date(2026, 1, 1)`), making it
        // impossible to call `.isoformat()` or read `.year` afterwards.
        let type_name = py_val
            .get_type()
            .name()
            .map(|n| n.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        // `py_val` is a `Bound<'_, PyAny>` borrow; `unbind()` produces an owned
        // `Py<PyAny>` whose lifetime is independent of the current GIL scope.
        let owned = py_val.clone().unbind();
        let handle = PyObjectHandle::new(type_name, owned);
        let id = vm.insert_heap_object(handle);
        vm.rc_push(task, auto_val::encode_object(id as u32));
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
    vm.rc_push(task, auto_val::encode_object(id as u32));
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
        // Plan 539 W0: value_to_py now takes &AutoVM (VmRef resolution);
        // a minimal empty VM suffices for scalar paths.
        let vm = crate::vm::engine::AutoVM::new(crate::vm::virt_memory::VirtualFlash::new(0), 1024);
        Python::attach(|py| {
            // Int round-trip
            let val = auto_val::Value::Int(42);
            let py_val = value_to_py(&val, py, &vm);
            let back: i32 = py_val.extract().unwrap();
            assert_eq!(back, 42);

            // Bool round-trip
            let val = auto_val::Value::Bool(true);
            let py_val = value_to_py(&val, py, &vm);
            let back: bool = py_val.extract().unwrap();
            assert!(back);

            // Float round-trip
            let val = auto_val::Value::Double(3.14);
            let py_val = value_to_py(&val, py, &vm);
            let back: f64 = py_val.extract().unwrap();
            assert!((back - 3.14).abs() < 0.001);

            // String round-trip
            let val = auto_val::Value::Str("hello".into());
            let py_val = value_to_py(&val, py, &vm);
            let back: String = py_val.extract().unwrap();
            assert_eq!(back, "hello");
        });
    }

    // ========================================================================
    // Plan 539 W0 (DIV-PY-AUTOLIST-1): nested container marshalling
    // ========================================================================

    #[test]
    fn test_value_to_py_nested_array_and_obj() {
        let vm = crate::vm::engine::AutoVM::new(crate::vm::virt_memory::VirtualFlash::new(0), 1024);
        Python::attach(|py| {
            // Nested Array → nested PyList
            let inner = auto_val::Value::Array(auto_val::Array::from(vec![
                auto_val::Value::Int(1),
                auto_val::Value::Double(2.5),
                auto_val::Value::Str("x".into()),
            ]));
            let outer = auto_val::Value::Array(auto_val::Array::from(vec![
                auto_val::Value::Int(0),
                inner,
            ]));
            let py_val = value_to_py(&outer, py, &vm);
            let back: Vec<pyo3::Bound<'_, PyAny>> = py_val.extract().unwrap();
            assert_eq!(back.len(), 2);
            assert_eq!(back[0].extract::<i32>().unwrap(), 0);
            let inner_back: Vec<pyo3::Bound<'_, PyAny>> = back[1].extract().unwrap();
            assert_eq!(inner_back.len(), 3);
            assert_eq!(inner_back[0].extract::<i32>().unwrap(), 1);
            assert_eq!(inner_back[2].extract::<String>().unwrap(), "x");

            // Obj → PyDict with string keys
            let mut obj = auto_val::Obj::new();
            obj.set(
                auto_val::ValueKey::from("bias"),
                auto_val::Value::Bool(false),
            );
            obj.set(
                auto_val::ValueKey::from("lr"),
                auto_val::Value::Double(0.1),
            );
            let py_val = value_to_py(&auto_val::Value::Obj(obj), py, &vm);
            let dict = py_val.cast::<PyDict>().unwrap();
            assert_eq!(dict.len(), 2);
            assert!(!dict
                .get_item("bias")
                .unwrap()
                .expect("bias")
                .extract::<bool>()
                .unwrap());
            assert_eq!(
                dict.get_item("lr").unwrap().expect("lr").extract::<f64>().unwrap(),
                0.1
            );
        });
    }

    #[test]
    fn test_value_to_py_resolves_vmref_list() {
        // A nested list stored as ListData in the heap, referenced by VmRef —
        // the shape CREATE_ARRAY produces for nested array literals.
        let vm = crate::vm::engine::AutoVM::new(crate::vm::virt_memory::VirtualFlash::new(0), 1024);
        let list = crate::vm::types::ListData::<auto_val::Value> {
            elems: vec![
                auto_val::Value::Int(7),
                auto_val::Value::Str("seven".into()),
            ],
            storage: None,
        };
        let id = vm.insert_heap_object(list);
        Python::attach(|py| {
            let val = auto_val::Value::VmRef(auto_val::VmRef { id: id as usize });
            let py_val = value_to_py(&val, py, &vm);
            let back: Vec<pyo3::Bound<'_, PyAny>> = py_val.extract().unwrap();
            assert_eq!(back.len(), 2);
            assert_eq!(back[0].extract::<i32>().unwrap(), 7);
            assert_eq!(back[1].extract::<String>().unwrap(), "seven");
        });
    }

    // ========================================================================
    // Plan 369 Task 12: PyObjectHandle + py_call / py_getattr shim tests
    // ========================================================================

    #[test]
    fn test_py_object_handle_send_sync_storage() {
        // Py<PyAny> is Send + Sync in PyO3 0.29, so PyObjectHandle must satisfy
        // the HeapObject bounds (Send + Sync + 'static). This is a compile-time
        // check: if the bounds ever regress, this test stops compiling.
        fn assert_send_sync<T: Send + Sync + 'static>() {}
        assert_send_sync::<PyObjectHandle>();
    }

    #[test]
    fn test_py_object_handle_wraps_and_recovers_object() {
        // Round-trip a live Python object (datetime.date) through PyObjectHandle:
        // store it, then bind it back under the GIL and verify it's the same
        // object (method dispatch works, attribute access works).
        Python::attach(|py| {
            let datetime = py.import("datetime").unwrap();
            let date = datetime.getattr("date").unwrap();
            let d = date.call1((2026, 1, 1)).unwrap();
            let type_name = d.get_type().name().unwrap().to_string();
            assert_eq!(type_name, "date");

            let owned: Py<PyAny> = d.clone().unbind();
            let handle = PyObjectHandle::new(type_name, owned);

            // Recover via clone_ref + bind.
            let bound = handle.obj.clone_ref(py).into_bound(py);
            let iso: String = bound.call_method0("isoformat").unwrap().extract().unwrap();
            assert_eq!(iso, "2026-01-01");
            let year: i32 = bound.getattr("year").unwrap().extract().unwrap();
            assert_eq!(year, 2026);
        });
    }

    #[test]
    fn test_register_object_shims_assigns_fixed_ids() {
        // register_object_shims must install shims at NATIVE_PY_CALL / NATIVE_PY_GETATTR
        // so the codegen can resolve py.py_call / py.py_getattr without consulting
        // the bridge at codegen time.
        let mut bridge = PyFfiBridge::new().unwrap();
        bridge.register_object_shims();
        let ni = bridge.native_interface();
        assert!(ni.get(NATIVE_PY_CALL).is_some(), "py_call shim not registered");
        assert!(ni.get(NATIVE_PY_GETATTR).is_some(), "py_getattr shim not registered");
    }

    // ========================================================================
    // Plan 539 W0 (DIV-PY-KWARGS-1): py_call_kw keyword-argument channel
    // ========================================================================

    #[test]
    fn test_py_call_kw_registers_fixed_id() {
        let mut bridge = PyFfiBridge::new().unwrap();
        bridge.register_object_shims();
        let ni = bridge.native_interface();
        assert!(ni.get(NATIVE_PY_CALL_KW).is_some(), "py_call_kw shim not registered");
    }

    #[test]
    fn test_build_kwargs_zips_names_and_values() {
        Python::attach(|py| {
            let names = PyList::new(py, ["alpha", "beta"]).unwrap().into_any();
            let vals = PyList::new(py, [1, 2]).unwrap().into_any();
            let kwargs = build_kwargs(&names, &vals).unwrap();
            assert_eq!(kwargs.len(), 2);
            assert_eq!(
                kwargs
                    .get_item("alpha")
                    .unwrap()
                    .expect("alpha missing")
                    .extract::<i32>()
                    .unwrap(),
                1
            );
            assert_eq!(
                kwargs
                    .get_item("beta")
                    .unwrap()
                    .expect("beta missing")
                    .extract::<i32>()
                    .unwrap(),
                2
            );
        });
    }

    #[test]
    fn test_build_kwargs_rejects_length_mismatch_silently_zips() {
        // zip() semantics: a longer names list simply drops the unmatched tail
        // (Python raises TypeError for extra kwargs — our shim's zip keeps the
        // contract "kwargs = zip(names, vals)", documented for the ABI).
        Python::attach(|py| {
            let names = PyList::new(py, ["a", "b", "c"]).unwrap().into_any();
            let vals = PyList::new(py, [1]).unwrap().into_any();
            let kwargs = build_kwargs(&names, &vals).unwrap();
            assert_eq!(kwargs.len(), 1);
            assert!(kwargs
                .get_item("a")
                .unwrap()
                .is_some_and(|v| v.extract::<i32>().unwrap() == 1));
        });
    }

    #[test]
    fn test_pyo3_call_method_with_kwargs() {
        // Pins the pyo3 API the py_call_kw shim relies on: call_method with an
        // args tuple + kwargs dict reaches the Python method as keyword args.
        Python::attach(|py| {
            let code = std::ffi::CString::new(
                "type('KwBox', (), {'combine': lambda self, a, b=0: a + b})()",
            )
            .unwrap();
            let obj = py.eval(&code, None, None).unwrap();
            let kwargs = PyDict::new(py);
            kwargs.set_item("b", 5).unwrap();
            let args = PyTuple::new(py, vec![1]).unwrap();
            let result = obj.call_method("combine", args, Some(&kwargs)).unwrap();
            assert_eq!(result.extract::<i32>().unwrap(), 6);
        });
    }

    // ========================================================================
    // Plan 539 W0 (DIV-PY-FLOAT-1): float returns push a real f64
    // ========================================================================

    #[test]
    fn test_marshal_return_float_pushes_f64() {
        // A Python float return must land on the stack as an f64 NanoValue
        // (single-slot since Plan 377), not a string-pool reference. Also
        // covers the 0-dim-tensor `.item()` channel: extract::<f64> rides
        // the __float__ protocol.
        let vm = crate::vm::engine::AutoVM::new(crate::vm::virt_memory::VirtualFlash::new(0), 1024);
        let mut task = crate::vm::task::AutoTask::new(0, 256, 0);
        Python::attach(|py| {
            let f = PyFloat::new(py, 2.0);
            py_auto_marshal_return(&f, &mut task, &vm).unwrap();
        });
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_f64(nv), "expected f64 tag, got {:?}", nv);
        assert_eq!(auto_val::decode_f64(nv), 2.0);

        // Integral floats keep their f64-ness (no ".0"-dropping stringify).
        Python::attach(|py| {
            let f = PyFloat::new(py, 9.0);
            py_auto_marshal_return(&f, &mut task, &vm).unwrap();
        });
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_f64(nv));
        assert_eq!(auto_val::decode_f64(nv), 9.0);
    }

    #[test]
    fn test_marshal_return_int_and_bool_unchanged() {
        let vm = crate::vm::engine::AutoVM::new(crate::vm::virt_memory::VirtualFlash::new(0), 1024);
        let mut task = crate::vm::task::AutoTask::new(0, 256, 0);
        Python::attach(|py| {
            let i: i64 = 42;
            let iv = i.into_pyobject(py).unwrap();
            py_auto_marshal_return(&iv.into_any(), &mut task, &vm).unwrap();
        });
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_i32(nv));
        assert_eq!(auto_val::decode_i32(nv), 42);
    }
}
