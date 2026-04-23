// Global tokio runtime for VM execution
// Using OnceLock to ensure thread-safe lazy initialization
use std::sync::OnceLock;
use std::sync::Arc;
static GLOBAL_RT: OnceLock<Arc<tokio::runtime::Runtime>> = OnceLock::new();

pub(crate) fn get_global_runtime() -> Arc<tokio::runtime::Runtime> {
    GLOBAL_RT.get_or_init(|| {
        Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime")
        )
    }).clone()
}

pub mod api;
pub mod ast;
pub mod atom;
// a2r Standard Library - Rust implementations of AutoLang standard types
pub mod a2r_std;
// Plan 084: Unified TypeStore for type declaration management
pub mod types;
// Plan 091: AutoVM-based interpreter interface
pub mod interpreter;
// Plan 090: Parser helpers to remove Universe dependency
pub mod parser_helpers;
// Plan 085: Use statement scanner for AIE + AutoCache
pub mod use_scanner;
// Plan 092: Dep statement scanner for Rust FFI
pub mod dep_scanner;
// Plan 085 Phase 5: Module cache for incremental compilation
pub mod atom_error;
pub mod auto_cache;
pub mod autovm_persistent; // Plan 068 Phase 9.6: Persistent AutoVM REPL
pub mod compile;
pub mod config;
pub mod database;
pub mod dep;
pub mod error;
// Plan 096 Phase 0: Scenario-based compilation
pub mod session;
// Plan 096 Phase 0: AURA (Auto UI Representation Abstract)
pub mod aura;
// Plan 217: A2UI Protocol Bridge (AURA ↔ A2UI JSON)
pub mod a2ui;
// Plan 096 Phase 2: UI Backend Generators (Vue, Rust)
pub mod ui_gen;
// Plan 152: Server-Sent Events (SSE) 解析
pub mod sse;
// Plan 114: Hybrid Routing (Convention + Config)
pub mod route;
// Plan 081 Phase 2: Execution mode selection (autovm, evaluator, c, rust)
pub mod mode;
// Plan 081 Phase 4: Multi-mode compilation pipeline
pub mod multi_mode;
// Plan 081 Phase 5: FFI layer for cross-mode function calls
pub mod ffi;
// Plan 214: Python FFI via PyO3 (optional, requires `--features python`)
#[cfg(feature = "python")]
pub mod py_ffi;
// Plan 073 Phase 9.3: Execution engine selection (AutoVM vs Evaluator)
pub mod execution_engine;
pub mod hash;
pub mod implicit_union; // Plan 125 Phase 3.3: Implicit union generator
pub mod infer;
pub use crate::infer::InferenceContext;
pub use crate::type_registry::SharedTypeRegistry;
pub mod indexer;
mod lexer;
pub mod libs;
pub mod macro_;
pub mod maker;
pub mod ownership;
pub mod parser;
pub mod query;
pub use parser::Parser;
// Plan 088 Phase 6: Type checking module for parameter passing modes
pub mod autovm_repl;
pub mod patch;
pub mod typeck;
// Plan 091: repl.rs deleted - use autovm_persistent::AutovmReplSession instead
// Plan 078: ModuleResolver trait for dependency resolution
pub mod resolver;
pub mod runtime;
pub mod scope;
pub mod scope_manager;
pub mod symbols;
pub mod target;
pub mod token;
pub mod trait_checker;
// Plan 087: Type registry for REPL
pub mod trans;
pub mod type_registry;
// Plan 109: AutoDown Document Format
pub mod autodown;
// Plan 091: Extracted from universe.rs
pub mod symbol;
pub mod util;
pub mod vm;
// Plan 095: Compile-Time Execution Engine
pub mod comptime;

// Plan 088: Parameter passing mode tests
#[cfg(test)]
mod plan_088_parser_tests;
#[cfg(test)]
mod plan_088_tests;

pub use atom::{Atom, AtomReader};

// 过程宏 - 支持 AutoLang 语法的内嵌 DSL
// 这些宏接受 AutoLang 代码字符串并解析为 Atom/Node/Value 结构体
pub use auto_macros::{atom, node, value};

// Plan 091: AutoVM-based interpreter (replacement for eval.rs/interp.rs)
pub use interpreter::AutoInterpreter;

use crate::compile::CompileSession;
use crate::trans::c::CTrans;
pub use crate::symbols::SymbolLocation;
use crate::{trans::Sink, trans::Trans};
use auto_val::{AutoPath, Obj, Value, Node, Array};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::error::{AutoError, AutoResult};

/// Global error limit for parser error recovery
static ERROR_LIMIT: AtomicUsize = AtomicUsize::new(20);

/// Global VM debug logging flag
static VM_DEBUG: AtomicBool = AtomicBool::new(false);

/// Set the global error limit for parser error recovery
///
/// This controls how many errors the parser will collect before aborting.
/// Default is 20.
pub fn set_error_limit(limit: usize) {
    ERROR_LIMIT.store(limit, Ordering::SeqCst);
}

/// Get the current global error limit
pub fn get_error_limit() -> usize {
    ERROR_LIMIT.load(Ordering::SeqCst)
}

/// Enable or disable VM debug logging
///
/// When enabled, the VM will print debug messages for operations like
/// task spawning, message handling, and replies.
pub fn set_vm_debug(enabled: bool) {
    VM_DEBUG.store(enabled, Ordering::SeqCst);
}

/// Check if VM debug logging is enabled
pub fn is_vm_debug() -> bool {
    VM_DEBUG.load(Ordering::SeqCst)
}

/// Debug logging macro - only prints when VM debug mode is enabled
macro_rules! vm_debug {
    ($($arg:tt)*) => {
        if is_vm_debug() {
            eprintln!($($arg)*);
        }
    };
}

/// Format a Value for display in object literals
fn format_value_for_display(vm: &crate::vm::engine::AutoVM, val: &Value) -> String {
    match val {
        Value::Int(i) => {
            // Check if it's a tagged string index
            if *i < 0 && *i > -1000000 && *i != -2147483648 && *i != -2147483647 {
                let str_idx = (-i - 1) as usize;
                let strings = vm.strings.read().unwrap();
                if let Some(bytes) = strings.get(str_idx) {
                    return format!("\"{}\"", String::from_utf8_lossy(bytes));
                }
            }
            i.to_string()
        }
        Value::Bool(b) => b.to_string(),
        Value::Str(s) => format!("\"{}\"", s.as_str()),
        Value::Nil => "nil".to_string(),
        Value::VmRef(vm_ref) => {
            // Recursively format VmRef values
            let id = vm_ref.id as u64;
            if let Some(obj_arc) = vm.objects.get(&id) {
                let obj = obj_arc.read().unwrap();
                let fields: Vec<String> = obj.fields.iter().map(|(k, v)| {
                    format!("{}: {}", k, format_value_for_display(vm, v))
                }).collect();
                format!("{{{}}}", fields.join(", "))
            } else if let Some(arr_arc) = vm.arrays.get(&id) {
                let arr = arr_arc.read().unwrap();
                let items: Vec<String> = arr.iter().map(|v| format_value_for_display(vm, v)).collect();
                format!("[{}]", items.join(", "))
            } else {
                format!("<ref:{}>", id)
            }
        }
        Value::Array(arr) => {
            let items: Vec<String> = arr.values.iter().map(|v| format_value_for_display(vm, v)).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Obj(obj) => {
            let fields: Vec<String> = obj.iter().map(|(k, v)| {
                format!("{}: {}", k, format_value_for_display(vm, v))
            }).collect();
            format!("{{{}}}", fields.join(", "))
        }
        _ => val.repr().to_string(),
    }
}

/// Run AutoLang code using the default execution engine
///
/// **Plan 081 Phase 1**: Default engine is AutoVM (faster bytecode VM)
/// Use environment variable `AUTO_EXECUTION_ENGINE=evaluator` to switch to TreeWalker
///
/// # Environment Variable
/// Set `AUTO_EXECUTION_ENGINE=autovm` or `=evaluator` to override
///
/// # Examples
/// ```ignore
/// let result = run("1 + 2").unwrap();  // Returns "3"
/// ```
pub fn run(code: &str) -> AutoResult<String> {
    // Plan 081 Phase 1: AutoVM is now the default (no feature flag required)
    // Use execution engine selector to get the engine (with env override support)
    let engine = execution_engine::ExecutionEngine::get();
    execution_engine::execute_with_engine(engine, code)
}

/// Plan 177: Run AutoLang code with stdout capture for testing
///
/// Returns (result_value, captured_stdout)
pub fn run_with_capture(code: &str) -> AutoResult<(String, String)> {
    let engine = execution_engine::ExecutionEngine::get();
    execution_engine::execute_with_engine_capture(engine, code)
}

/// Run AutoLang code using AutoVM (bytecode VM)
///
/// **Plan 068 Phase 9**: Primary execution engine for AutoLang
///
/// This function compiles AutoLang code to ABC bytecode and executes it
/// on the AutoVM virtual machine. AutoVM is faster than the evaluator and
/// provides consistent behavior across PC and MCU environments.
///
/// # Examples
/// ```ignore
/// let result = run_autovm("1 + 2").unwrap();  // Returns "3"
/// ```
pub fn run_autovm(code: &str) -> AutoResult<String> {
    // Use a dedicated thread with 4MB stack to avoid stack overflow on Windows
    // (default main thread stack is only 1MB on Windows)
    let code = code.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            let rt = get_global_runtime();
            rt.block_on(async { execute_autovm(&code, false).await.map(|(r, _)| r) })
        })
        .expect("Failed to spawn execution thread");
    handle.join().unwrap()
}

/// Plan 177: Run AutoVM with stdout capture for testing
pub fn run_autovm_capture(code: &str) -> AutoResult<(String, String)> {
    let code = code.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            let rt = get_global_runtime();
            rt.block_on(async { execute_autovm(&code, true).await })
        })
        .expect("Failed to spawn execution thread");
    handle.join().unwrap()
}

/// Find the source span of a `use ... : symbol` statement in source code
fn find_use_symbol_span(source: &str, error_msg: &str) -> miette::SourceSpan {
    let symbol = error_msg
        .strip_prefix("Undefined symbol: ")
        .and_then(|s| s.split(" in ").next())
        .unwrap_or("");

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") && trimmed.contains(symbol) {
            let line_start = source[..]
                .split('\n')
                .take(line_num)
                .map(|l| l.len() + 1)
                .sum::<usize>();
            return miette::SourceSpan::new(line_start.into(), line.len().into());
        }
    }
    miette::SourceSpan::new(0usize.into(), source.len().min(40).into())
}

fn extract_undefined_symbol(error_msg: &str) -> Option<&str> {
    error_msg
        .strip_prefix("Undefined symbol: ")
        .and_then(|s| s.split(" in ").next())
}

/// Plan 212b Task 4: Initialize Rust FFI bridge at runtime
///
/// Scans the CompileSession for Rust dependencies and imports,
/// loads compiled libraries, and registers functions as native shims.
///
/// Returns a NativeInterface with registered Rust FFI shims, or None
/// if there are no Rust imports.
fn init_rust_ffi(session: &compile::CompileSession) -> Option<crate::vm::native::NativeInterface> {
    let rust_imports = session.rust_imports();
    if rust_imports.is_empty() {
        return None;
    }

    let mut native_interface = crate::vm::native::NativeInterface::new();

    // Create a RustFfiBridge to load and register Rust functions
    let mut bridge = match crate::ffi::RustFfiBridge::new() {
        Ok(b) => b,
        Err(e) => {
            log::warn!("Failed to create RustFfiBridge: {:?}", e);
            return None;
        }
    };

    // Load compiled libraries for each crate
    for (crate_name, functions) in rust_imports {
        // The wrapper library name follows the sandbox naming convention
        let wrapper_name = format!("{}_wrapper", crate_name.replace('-', "_"));

        // Try to load the compiled wrapper library
        if let Some(sandbox) = session.sandbox() {
            let lib_path = sandbox.crate_library_path(&wrapper_name, "1");
            if lib_path.exists() {
                if let Err(e) = bridge.load_rust_library(&crate_name, &lib_path) {
                    log::warn!("Failed to load Rust library {} from {}: {:?}", crate_name, lib_path.display(), e);
                    continue;
                }

                // Register each function as a string→string shim
                for func_name in functions {
                    let signature = crate::ffi::RustSignature::new()
                        .param(crate::ffi::RustType::String)
                        .returns(crate::ffi::RustType::String);

                    match bridge.register_function(&crate_name, func_name, signature) {
                        Ok(native_id) => {
                            log::info!("Registered Rust FFI: {}::{} (native_id={})", crate_name, func_name, native_id);

                            // Also register in BIGVM_NATIVES so codegen can find it
                            let qualified = format!("rust.{}", func_name);
                            if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {
                                registry.register_with_id(&qualified, native_id);
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to register Rust function {}::{}: {:?}", crate_name, func_name, e);
                        }
                    }
                }
            } else {
                log::info!("Wrapper library not found for {} at {}", crate_name, lib_path.display());
            }
        }
    }

    // Merge the bridge's native interface into our result
    native_interface.merge(bridge.native_interface());

    Some(native_interface)
}

/// Plan 214: Initialize Python FFI bridge if there are use.py imports
///
/// Creates a PyFfiBridge, imports the requested Python modules,
/// registers each function as a native shim, and returns the
/// NativeInterface for merging into the VM.
#[cfg(feature = "python")]
fn init_py_ffi(session: &compile::CompileSession) -> Option<crate::vm::native::NativeInterface> {
    let py_imports = session.py_imports();
    if py_imports.is_empty() {
        return None;
    }

    let mut bridge = match crate::py_ffi::PyFfiBridge::new() {
        Ok(b) => b,
        Err(e) => {
            log::warn!("Failed to create PyFfiBridge: {:?}", e);
            return None;
        }
    };

    // Import each Python module and register its functions
    for (module_name, functions) in py_imports {
        if let Err(e) = bridge.import_module(module_name) {
            log::warn!("Failed to import Python module '{}': {:?}", module_name, e);
            continue;
        }

        for func_name in functions {
            match bridge.register_function(module_name, func_name) {
                Ok(native_id) => {
                    log::info!("Registered Python FFI: {}.{} (native_id={})", module_name, func_name, native_id);

                    // Also register in BIGVM_NATIVES so codegen can find it
                    let qualified = format!("py.{}", func_name);
                    if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {
                        registry.register_with_id(&qualified, native_id);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to register Python function {}.{}: {:?}", module_name, func_name, e);
                }
            }
        }
    }

    let mut native_interface = crate::vm::native::NativeInterface::new();
    native_interface.merge(bridge.native_interface());

    Some(native_interface)
}

#[cfg(not(feature = "python"))]
fn init_py_ffi(_session: &compile::CompileSession) -> Option<crate::vm::native::NativeInterface> {
    None
}

/// Internal AutoVM execution function (async)
/// Plan 177: capture parameter enables stdout capture for testing
async fn execute_autovm(code: &str, capture: bool) -> AutoResult<(String, String)> {
    use crate::vm::codegen::Codegen;
    use crate::vm::engine::AutoVM;
    use crate::vm::opcode::OpCode;
    use crate::vm::loader::Linker;
    use crate::vm::virt_memory::VirtualFlash;

    // Plan 085: Pre-process use statements to load dependencies
    let mut session = compile::CompileSession::new();
    session.collect_rust_imports(code)?; // Plan 212b: collect use.rust imports before resolving deps
    session.collect_py_imports(code)?; // Plan 214: collect use.py imports
    session.resolve_deps(code)?; // Plan 212b: resolve dep statements (triggers compile_dep)
    session.resolve_uses(code)?;

    // Plan 212b Task 4: Initialize Rust FFI bridge if there are Rust imports
    let rust_ffi_native_interface = init_rust_ffi(&session);

    // Plan 214: Initialize Python FFI bridge if there are Python imports
    let py_ffi_native_interface = init_py_ffi(&session);

    // 1. Parse the code (with pre-loaded type_store from resolve_uses)
    let mut parser = Parser::new_with_type_store(code, session.type_store());
    let mut ast = parser.parse()?;

    // Plan 095: Run CTEE (Compile-Time Execution Engine) to transform AST
    // This handles #if, #for, #is, #{} constructs
    let mut ctee = crate::comptime::CTEE::new();
    ctee.transform(&mut ast)?;

    // 2. Compile to bytecode
    // Plan 091: Wrap script-level code with FN_PROLOG/RESERVE_STACK for proper local variable support
    // Plan 123: Share TypeStore with Parser so Codegen can access registered types/enums
    let mut codegen = Codegen::new_with_type_store(parser.type_store.clone());
    // Separate type/ext declarations from other statements
    // Type declarations and ext blocks stay at global level, other code goes into script wrapper
    let (type_decls, other_stmts): (Vec<_>, Vec<_>) = ast.stmts.iter().partition(|stmt| {
        matches!(stmt, crate::ast::Stmt::TypeDecl(_) | crate::ast::Stmt::Ext(_) | crate::ast::Stmt::EnumDecl(_))
    });

    // First, compile type declarations at global level
    for stmt in &type_decls {
        codegen.compile_stmt(stmt)?;
    }

    // Then, compile other statements with proper local variable setup
    if !other_stmts.is_empty() {
        if is_vm_debug() {
            vm_debug!("DEBUG: Compiling {} script statements", other_stmts.len());
        }

        // Reserve space for locals (Plan 091)
        let n_locals = 16; // Reserve space for up to 16 locals

        // Emit FN_PROLOG to set up BP
        codegen.emit_op(crate::vm::opcode::OpCode::FN_PROLOG);
        codegen.emit_byte(0); // n_args
        codegen.emit_byte(n_locals as u8); // n_locals

        // Emit RESERVE_STACK to reserve space for locals
        codegen.emit_op(crate::vm::opcode::OpCode::RESERVE_STACK);
        codegen.emit_byte(n_locals as u8);

        // Now compile the statements
        for stmt in other_stmts.iter() {
            codegen.compile_stmt(stmt)?;
        }
    }

    // Add explicit HALT at the end
    codegen.code.push(OpCode::HALT as u8);

    // DEBUG: Print bytecode BEFORE relocation
    vm_debug!("DEBUG: === Bytecode BEFORE relocation (0x00-0x40) ===");
    for i in 0x00..0x40u32 {
        if (i as usize) < codegen.code.len() {
            let op = codegen.code[i as usize];
            vm_debug!("CODE[{:04x}]: {:02x}", i, op);
        }
    }
    vm_debug!("DEBUG: === End of bytecode ===");

    // 3. Perform multi-module linking
    let strings = codegen.strings.clone();

    let mut linker = Linker::new();
    let dep_modules = session.take_compiled_modules();
    for module in dep_modules {
        linker.add_module(module);
    }

    let object_keys = codegen.object_keys.clone();
    let object_types = codegen.object_types.clone();
    let result_type = codegen.last_expr_type.clone();
    // Plan 197 Task 9: Extract generic registry before finish() consumes the codegen
    let generic_registry = std::mem::take(&mut codegen.generic_registry);
    let main_module = codegen.finish("<main>".to_string());
    vm_debug!("DEBUG: Main module exports: {:?}", main_module.exports.keys().collect::<Vec<_>>());
    linker.add_module(main_module);

    let (linked_code, global_symbols) = linker.link().map_err(|e| {
        let span = if let Some(pos) = e.source_pos {
            crate::error::pos_to_span(pos)
        } else {
            find_use_symbol_span(code, &e.message)
        };
        let help = if e.source_pos.is_some() {
            Some(format!("Use a `use` statement to import '{}' from a module, or check for typos", e.symbol))
        } else {
            extract_undefined_symbol(&e.message).map(|s| format!("Check if '{}' is defined and exported in the module", s))
        };
        crate::error::AutoError::MsgWithSource(crate::error::MsgWithSource {
            source: miette::NamedSource::new("<script>", code.to_string()),
            message: e.message.clone(),
            span,
            help,
        })
    })?;

    // Use global_symbols to find main's absolute address in linked code
    // (main module is laid out after all dependency modules, so its offset is adjusted)
    let main_entry = if let Some(&addr) = global_symbols.get("main").or_else(|| global_symbols.get("test")) {
        addr as usize
    } else {
        // No main/test function — start from beginning of main module's code
        // (which is after all dependency modules)
        let dep_size: usize = linker.modules.iter().take(linker.modules.len() - 1)
            .map(|m| m.code.len())
            .sum();
        dep_size
    };

    vm_debug!("DEBUG: Linked code size: {} bytes", linked_code.len());
    vm_debug!("DEBUG: Global symbols: {:?}", global_symbols);

    // 4. Load into VM
    // Plan 073: Include object_keys and object_types for object literal support
    // Pass global_symbols as exports_by_name for CALL_SPEC dynamic dispatch
    let flash = VirtualFlash::from_vec_with_metadata(
        linked_code,
        global_symbols,
        object_keys,
        object_types,
    );
    let (mut vm, output_buffer) = if capture {
        let (vm, buf) = AutoVM::new_with_capture(flash, 1024);
        (vm, Some(buf))
    } else {
        let vm = AutoVM::new(flash, 1024);
        (vm, None)
    };
    vm.load_strings(strings);
    vm.load_generic_registry(generic_registry);

    // Plan 212b Task 4: Merge Rust FFI native interface into VM
    if let Some(rust_ni) = rust_ffi_native_interface {
        vm.merge_native_interface(&rust_ni);
    }

    // Plan 214: Merge Python FFI native interface into VM
    if let Some(py_ni) = py_ffi_native_interface {
        vm.merge_native_interface(&py_ni);
    }

    // Helper to extract stdout from capture buffer
    let get_stdout = || {
        output_buffer
            .as_ref()
            .map(|buf| buf.read().unwrap().clone())
            .unwrap_or_default()
    };

    // Plan 118: Store the codegen's result type for formatting

    // 5. Execute - Find main/test entry point
    let task_id = vm.spawn_task(main_entry, 16384);
    vm.run_task_loop().await;

    // 6. Get result from stack
    let result = if let Some(task_arc) = vm.tasks.get(&task_id).map(|r| r.value().clone()) {
        let mut task = task_arc.lock().await;

        // Plan 118: Check if task had an error
        if let Some(error) = &task.last_error {
            return Err(crate::error::AutoError::Msg(error.clone()));
        }

        if task.ram.sp == 0 {
            "".to_string()
        } else {
            // Plan 117/118: Check result type for proper formatting
            use crate::vm::codegen::ObjectType;
            use crate::vm::task::ResultType;

            // Check VM runtime result type first (set during execution)
            match task.last_result_type {
                ResultType::Float => {
                    let result = task.ram.pop_f32();
                    format!("{}", result)
                }
                _ => {
                    // Then check codegen's compile-time result type
                    match result_type {
                        ObjectType::Float | ObjectType::Double => {
                            let result = task.ram.pop_f32();
                            format!("{}", result)
                        }
                        ObjectType::Byte => {
                            let result = task.ram.pop_i32();
                            format!("0x{:02X}", result as u8)
                        }
                        ObjectType::Uint => {
                            let result = task.ram.pop_i32();
                            format!("{}u", result as u32)
                        }
                        ObjectType::Char => {
                            let result = task.ram.pop_i32();
                            if let Some(ch) = char::from_u32(result as u32) {
                                format!("'{}'", ch)
                            } else {
                                format!("{}", result)
                            }
                        }
                        ObjectType::Bool => {
                            let result = task.ram.pop_i32();
                            if result != 0 { "true".to_string() } else { "false".to_string() }
                        }
                        ObjectType::Void => {
                            let _ = task.ram.pop_i32();
                            "".to_string()
                        }
                        _ => {
                            // Default: pop and format based on value
                            let result = task.ram.pop_i32();
                            let result_u64 = result as u64;

                            // Check arrays registry
                            if let Some(arr_arc) = vm.arrays.get(&result_u64) {
                                let arr = arr_arc.read().unwrap();
                                let strings = vm.strings.read().unwrap();
                                let formatted: Vec<String> = arr.iter().map(|v| {
                                    if let auto_val::Value::Int(bits) = v {
                                        if *bits < 0 && *bits > -1000000 && *bits != -2147483648 && *bits != -2147483647 {
                                            let str_idx = (-bits - 1) as usize;
                                            if let Some(bytes) = strings.get(str_idx) {
                                                return format!("\"{}\"", String::from_utf8_lossy(bytes));
                                            }
                                        }
                                    }
                                    v.repr().to_string()
                                }).collect();
                                format!("[{}]", formatted.join(", "))
                            }
                            // Check heap objects
                            else if let Some(obj_arc) = vm.heap_objects.get(&result_u64) {
                                let obj = obj_arc.read().unwrap();
                                if let Some(list) = obj.as_any().downcast_ref::<crate::vm::types::ListData<i32>>() {
                                    let formatted: Vec<String> = list.elems.iter().map(|e| e.to_string()).collect();
                                    format!("[{}]", formatted.join(", "))
                                } else if let Some(sb) = obj.as_any().downcast_ref::<crate::vm::collections::SpecializedStringBuilder>() {
                                    sb.buffer.clone()
                                } else {
                                    format!("{}", result)
                                }
                            }
                            // Check objects registry
                            else if result >= 1000000 && result < 2000000 {
                                if let Some(obj_arc) = vm.objects.get(&result_u64) {
                                    let obj = obj_arc.read().unwrap();
                                    let mut fields: Vec<(&auto_val::ValueKey, &Value)> = obj.fields.iter().collect();
                                    fields.sort_by(|(k1, _), (k2, _)| k1.to_string().cmp(&k2.to_string()));
                                    let formatted: Vec<String> = fields.iter().map(|(k, v)| {
                                        let key_str = k.to_string();
                                        let val_str = format_value_for_display(&vm, v);
                                        format!("{}: {}", key_str, val_str)
                                    }).collect();
                                    format!("{{{}}}", formatted.join(", "))
                                } else {
                                    format!("{}", result)
                                }
                            }
                            // Check strings pool
                            else if result < 0 && result > -1000000 && result != -2147483648 && result != -2147483647 {
                                let str_idx = (-result - 1) as usize;
                                let strings = vm.strings.read().unwrap();
                                if let Some(bytes) = strings.get(str_idx) {
                                    String::from_utf8_lossy(bytes).to_string()
                                } else {
                                    format!("{}", result)
                                }
                            }
                            // Boolean markers
                            else if result == -2147483648 {
                                "true".to_string()
                            } else if result == -2147483647 {
                                "false".to_string()
                            }
                            // Range markers
                            else if result <= -1000000 && result > -2000000 {
                                let range_id = (result + 1000000) as usize;
                                if range_id < task.ram.ranges.len() {
                                    let (start, end, is_inclusive) = task.ram.ranges[range_id];
                                    if is_inclusive {
                                        format!("{}..={}", start, end)
                                    } else {
                                        format!("{}..{}", start, end)
                                    }
                                } else {
                                    format!("{}", result)
                                }
                            }
                            else {
                                format!("{}", result)
                            }
                        }
                    }
                }
            }
        }
    } else {
        return Err(crate::error::AutoError::Msg(
            "Task not found after execution".to_string(),
        ));
    };

    Ok((result, get_stdout()))
}

// run_with_errors() removed in Plan 091 - use run() with built-in error recovery

/// Run code with a custom scope
///
/// **Deprecated**: This function is deprecated. Use CompileSession instead (see Plan 064).
///
/// **Plan 091**: Simplified to use AutoVM internally
/// Run code with incremental compilation support
///
/// **Phase 2**: Execute code with persistent CompileSession
///
/// This function takes a mutable CompileSession and executes the code,
/// reusing the Database and QueryEngine across multiple calls for
/// incremental compilation benefits.
///
/// # Arguments
///
/// * `session` - Mutable reference to persistent CompileSession
/// * `code` - AutoLang source code to execute
///
/// # Returns
///
/// String representation of the result
///
/// # Example
///
/// ```rust,no_run
/// use auto_lang::{run_with_session, compile::CompileSession};
///
/// let mut session = CompileSession::new();
///
/// // First run - parses and compiles
/// let result1 = run_with_session(&mut session, "fn add(a int, b int) int { a + b }")?;
///
/// // Second run - can reuse cached data from first run
/// let result2 = run_with_session(&mut session, "add(10, 20)")?;
///
/// # Ok::<(), auto_lang::error::AutoError>(())
/// ```
pub fn run_with_session(session: &mut CompileSession, code: &str) -> AutoResult<String> {
    // Phase 2: Compile source with incremental support
    // The CompileSession tracks which files/fragments have changed
    session.compile_source(code, "<repl-input>")?;

    // Plan 091: Use AutoVM instead of deprecated Interpreter
    run(code)
}

/// Run code with incremental compilation and persistent scope support
///
/// **Phase 2**: Execute code with persistent CompileSession and Scope
///
/// This function takes a mutable CompileSession and a persistent scope,
/// reusing both across multiple calls for REPL-style incremental execution.
///
/// # Arguments
///
/// * `session` - Mutable reference to persistent CompileSession
/// * `_scope` - Persistent scope (deprecated, not used with AutoVM)
/// * `code` - AutoLang source code to execute
///
/// # Returns
///
/// String representation of the result, or error message
///
/// **Plan 091**: Now uses AutoVM internally. The scope parameter is deprecated.
#[allow(deprecated)]
pub fn run_with_session_and_scope(
    session: &mut CompileSession,
    // _scope: Shared<Universe>,  // Plan 091: removed

    code: &str,
) -> AutoResult<String> {
    // Plan 091: Use AutoVM instead of deprecated Interpreter
    // Note: The scope parameter is deprecated and ignored
    // AutoVM uses its own state management
    run_with_session(session, code)
}

pub fn parse(code: &str) -> AutoResult<ast::Code> {
    // Plan 091: Universe removed
    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code);
    parser.parse().map_err(|e| e.to_string().into())
}

/// Parse code and return proper AutoError (not converted to string)
/// This is used by the LSP to get detailed error information
pub fn parse_preserve_error(code: &str) -> Result<ast::Code, error::AutoError> {
    // Plan 091: Universe removed
    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code);
    parser.parse()
}

// Plan 091 DEPRECATED: Universe removed
// pub fn parse_with_scope(code: &str, scope: Rc<RefCell<Universe>>) -> AutoResult<ast::Code> {
//     let mut parser = Parser::from(code);
//     parser.parse().map_err(|e| e.to_string().into())
// }

// Functions removed in Plan 091:
// - interpret() - use run() instead
// - interpret_with_scope() - use run_with_session() instead
// - interpret_file() - use run_file() instead
// - eval_template() - TODO: implement in AutoVM
// - eval_config() - use eval_config_with_vm() instead
// - eval_config_with_scope() - use eval_config_with_vm() instead

pub fn run_file(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Plan 088 Phase 4: Use AutoVM instead of deprecated Interpreter
    // This enables smart parameter passing and other AutoVM features
    run(&code)
}

/// Evaluate config code using AutoVM (Plan 081 Phase 2)
///
/// **Replaces** `eval_config_with_scope` which uses the deprecated Interpreter.
/// Uses ConfigCodegen to compile to bytecode, then executes with AutoVM.
///
/// # Arguments
/// * `code` - Configuration source code
/// * `args` - Arguments to pass to the config
/// * `univ` - Universe for variable storage
///
/// # Returns
/// * The config value (typically a Node representing the parsed config)
///
/// # Example
/// ```no_run
/// use auto_lang::{eval_config_with_vm};
/// use auto_val::Obj;
///
/// let config = r#"
/// name: "myapp"
/// version: "0.1.0"
/// "#;
fn extract_value_from_vm(vm: &crate::vm::engine::AutoVM, bits: i32, visited: &mut std::collections::HashSet<u64>) -> Value {
    if bits < 0 {
        // Tagged string index: indices are stored as -(index+1)
        let str_idx = (-bits - 1) as usize;
        let strings = vm.strings.read().unwrap();
        if let Some(str_bytes) = strings.get(str_idx) {
            return Value::Str(String::from_utf8_lossy(str_bytes).to_string().into());
        }
        return Value::Nil;
    }

    let id = bits as u64;

    if !visited.insert(id) {
        // Cycle detected
        return Value::Nil;
    }

    // 1. Check if it's an object ID
    if let Some(obj_ref) = vm.objects.get(&id) {
        let obj_data = obj_ref.value().read().unwrap();
        let mut result_obj = Obj::new();
        for (key, val) in &obj_data.fields {
            let extracted = extract_auto_val_value(vm, val, visited);
            result_obj.set(key.clone(), extracted);
        }
        visited.remove(&id);
        return Value::Obj(result_obj);
    }

    // 2. Check if it's a node ID
    if let Some(node_ref) = vm.nodes.get(&id) {
        let node_data = node_ref.value().read().unwrap();
        let result = Value::Node(extract_node_deep(vm, &node_data, visited));
        visited.remove(&id);
        return result;
    }

    // 3. Check if it's an array ID
    if let Some(array_ref) = vm.arrays.get(&id) {
        let array_data = array_ref.value().read().unwrap();
        let mut items = Vec::new();
        for val in array_data.iter() {
            items.push(extract_auto_val_value(vm, val, visited));
        }
        visited.remove(&id);
        return Value::Array(Array::from_vec(items));
    }

    visited.remove(&id);
    // Fallback to integer (for non-heap values)
    Value::Int(bits)
}

fn extract_auto_val_value(vm: &crate::vm::engine::AutoVM, val: &Value, visited: &mut std::collections::HashSet<u64>) -> Value {
    match val {
        Value::VmRef(vm_ref) => extract_value_from_vm(vm, vm_ref.id as i32, visited),
        Value::Int(bits) => {
            // Check if this is a tagged string index (negative value)
            if *bits < 0 {
                let str_idx = (-bits - 1) as usize;
                let strings = vm.strings.read().unwrap();
                if let Some(str_bytes) = strings.get(str_idx) {
                    return Value::Str(String::from_utf8_lossy(str_bytes).to_string().into());
                }
            }
            val.clone()
        }
        Value::Array(arr) => {
            let mut items = Vec::new();
            for v in &arr.values {
                items.push(extract_auto_val_value(vm, v, visited));
            }
            Value::Array(Array::from_vec(items))
        }
        Value::Obj(obj) => {
            let mut result_obj = Obj::new();
            for (key, val) in obj.iter() {
                result_obj.set(key.clone(), extract_auto_val_value(vm, val, visited));
            }
            Value::Obj(result_obj)
        }
        Value::Node(node) => Value::Node(extract_node_deep(vm, node, visited)),
        _ => val.clone(),
    }
}

fn extract_node_deep(vm: &crate::vm::engine::AutoVM, node: &Node, visited: &mut std::collections::HashSet<u64>) -> Node {
    let mut result = node.clone();
    // Resolve props
    let props = node.props_clone();
    for (key, val) in props.iter() {
        result.set_prop(key.clone(), extract_auto_val_value(vm, val, visited));
    }
    // TODO: Resolve args and kids if they contain VmRefs
    result
}

/// let result = eval_config_with_vm(config, &Obj::new()).unwrap();
/// ```
pub fn eval_config_with_vm(code: &str, _args: &Obj) -> AutoResult<Value> {
    use crate::vm::config_codegen::ConfigCodegen;
    use crate::vm::engine::AutoVM;
    use crate::vm::opcode::OpCode;
    use crate::vm::virt_memory::VirtualFlash;

    // Note: Plan 091 - Universe parameter removed, AutoVM uses its own state
    // Note: Do NOT preprocess macros here — pac.at is config code, not UI code.
    // The `app` keyword in pac.at means a node definition (app (id: "main") {...}),
    // not a UI macro (which would expand to `type ... is App {...}`).

    // 1. Parse the code
    let mut parser = Parser::from(code);
    let ast = parser.parse()?;

    // 2. Compile to bytecode using ConfigCodegen
    let mut configgen = ConfigCodegen::new();
    configgen.compile_config(&ast)?;

    // Add explicit RET at the end (ConfigCodegen already adds this, but ensure it)
    if configgen.base().code.last() != Some(&(OpCode::RET as u8)) {
        configgen.base().code.push(OpCode::RET as u8);
    }

    // 3. Perform linking (resolve function calls)
    let strings = configgen.base().strings.clone();
    let exports = configgen.base().exports.clone();
    let relocs = configgen.base().relocs.clone();

    for reloc in &relocs {
        if let Some(&addr) = exports.get(&reloc.symbol_name) {
            let bytes = addr.to_le_bytes();
            let offset = reloc.offset as usize;
            for (i, b) in bytes.iter().enumerate() {
                configgen.base().code[offset + i] = *b;
            }
        } else {
            return Err(AutoError::Msg(format!(
                "Undefined symbol in config: {}",
                reloc.symbol_name
            )));
        }
    }

    // 4. Load into VM and execute
    // Clone the bytecode and metadata before moving into the async block
    let bytecode = configgen.base().code.clone();
    let object_keys = configgen.base().object_keys.clone();
    let object_types = configgen.base().object_types.clone();
    // Plan 197 Task 9: Extract generic registry for VM
    let generic_registry = std::mem::take(&mut configgen.base().generic_registry);

    let rt = get_global_runtime();
    rt.block_on(async {
        let flash = VirtualFlash::new_with_code_and_keys(bytecode, object_keys, object_types);
        let mut vm = AutoVM::new(flash, 4096); // 4KB RAM for config
        vm.load_strings(strings);
        vm.load_generic_registry(generic_registry);

        // 5. Execute from entry point (default to 0 for config)
        let entry_point = exports.get("main").copied().unwrap_or(0) as usize;

        let task_id = vm.spawn_task(entry_point, 4096);

        // Run the VM to completion
        vm.run_task_loop().await;

        // 6. Get the result from the VM stack
        // ConfigCodegen compiles to a single object that should be on the stack
        if let Some(task_arc) = vm.tasks.get(&task_id).map(|r| r.value().clone()) {
            let mut task = task_arc.lock().await;

            if task.ram.sp == 0 {
                // No return value - return Nil
                return Ok(Value::Nil);
            }

            // Pop the result value from the stack
            let result_i32 = task.ram.pop_i32();

            // Materialize the result from the VM heap
            let mut visited = std::collections::HashSet::new();
            let materialized_result = extract_value_from_vm(&vm, result_i32, &mut visited);

            // For config mode, we often expect a Node. 
            // If the result is an Obj, convert it to a "root" Node.
            match materialized_result {
                Value::Obj(obj) => {
                    let mut root = Node::new("root");
                    for (k, v) in obj.iter() {
                        if k.to_string().starts_with("_expr") {
                            if let Value::Node(n) = v {
                                root.add_kid(n.clone());
                                continue;
                            }
                        }
                        root.set_prop(k.clone(), v.clone());
                    }
                    Ok(Value::Node(root))
                }
                _ => Ok(materialized_result),
            }
        } else {
            // Task not found - return Nil
            Ok(Value::Nil)
        }
    })
}

/// Transpile AutoLang file to C
///
/// **Plan 091**: Now uses CompileSession internally (no Universe dependency)
pub fn trans_c(path: &str) -> AutoResult<String> {
    let mut session = CompileSession::new();
    trans_c_with_session(&mut session, path)
}

/// Transpile AutoLang file to Rust
///
/// **Plan 091**: Now uses CompileSession internally (no Universe dependency)
pub fn trans_rust(path: &str) -> AutoResult<String> {
    let mut session = CompileSession::new();
    trans_rust_with_session(&mut session, path)
}

/// Transpile AutoLang file to C (legacy implementation)
#[deprecated(
    since = "0.10.0",
    note = "Use trans_c() or trans_c_with_session() instead"
)]
pub fn trans_c_legacy(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    let cname = path.replace(".at", ".c");

    let fname = AutoPath::new(path).filename();

    // Plan 091: Universe removed
    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code.as_str());
    let ast = parser.parse()?;
    let mut sink = Sink::new(fname);
    let mut trans = CTrans::new(cname.clone().into());
    // Plan 091: set_scope removed
    trans.trans(ast, &mut sink)?;

    // convert sink to .c/.h files
    std::fs::write(&cname, sink.done()?)?;
    // write the header file
    let h_path = path.replace(".at", ".h");
    std::fs::write(Path::new(h_path.as_str()), sink.header)?;

    Ok(format!("[trans] {} -> {}", path, cname))
}

/// Transpile AutoLang file to Rust (legacy implementation)
#[deprecated(
    since = "0.10.0",
    note = "Use trans_rust() or trans_rust_with_session() instead"
)]
pub fn trans_rust_legacy(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    // Plan 204 Phase 6A: Use .a2r.rs suffix to avoid overwriting .rs files
    let rsname = path.replace(".at", ".a2r.rs");
    let fname = AutoPath::new(path).filename();

    // Plan 091: Universe removed
    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code.as_str());
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::rust::RustTrans::new(fname);
    // Plan 091: set_scope removed
    trans.trans(ast, &mut sink)?;

    // Write Rust file
    std::fs::write(&rsname, sink.done()?)?;

    Ok(format!("[trans] {} -> {}", path, rsname))
}

// =============================================================================
// Phase 066: Incremental Transpilation API (with CompileSession)
// =============================================================================

/// Transpile to C with incremental compilation support
///
/// This function uses CompileSession to enable incremental compilation,
/// caching results between calls for faster subsequent transpilations.
///
/// # Arguments
/// * `session` - Mutable reference to CompileSession (maintains cache)
/// * `path` - Path to the AutoLang source file
///
/// # Returns
/// Ok(String) with success message indicating transpiled file names
///
/// # Example
/// ```no_run
/// use auto_lang::{trans_c_with_session, compile::CompileSession};
///
/// let mut session = CompileSession::new();
/// let result = trans_c_with_session(&mut session, "test.at").unwrap();
/// println!("{}", result);
/// ```
pub fn trans_c_with_session(session: &mut CompileSession, path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)?;

    // Compile source with incremental support
    let frag_ids = session.compile_source(&code, path)?;

    // Get file_id and Database
    let db = session.db();
    let file_id = {
        let db_read = db.read().unwrap();
        db_read
            .get_file_id_by_path(path)
            .ok_or_else(|| format!("File not found in database: {}", path))?
    };

    // Create transpiler with Database
    let mut trans = CTrans::with_database(db.clone());

    // Perform incremental transpilation
    let results = trans.trans_incremental_c(session, file_id)?;

    // Merge results and write output files
    let cname = path.replace(".at", ".c");
    let hname = path.replace(".at", ".h");

    let mut source_content = String::new();
    let mut header_content = String::new();

    // Merge results
    for (_frag_id, (source, header)) in &results {
        source_content.push_str(source);
        header_content.push_str(header);
    }

    // Write output files
    if !source_content.is_empty() {
        std::fs::write(&cname, source_content)?;
    }
    if !header_content.is_empty() {
        std::fs::write(&hname, header_content)?;
    }

    Ok(format!(
        "[trans] {} -> {} ({} fragments, {} dirty, {} transpiled)",
        path,
        cname,
        frag_ids.len(),
        db.read().unwrap().get_dirty_fragments().len(),
        results.len()
    ))
}

/// Transpile to Rust with incremental compilation support
///
/// This function uses CompileSession to enable incremental compilation,
/// caching results between calls for faster subsequent transpilations.
///
/// # Arguments
/// * `session` - Mutable reference to CompileSession (maintains cache)
/// * `path` - Path to the AutoLang source file
///
/// # Returns
/// Ok(String) with success message indicating transpiled file names
///
/// # Example
/// ```no_run
/// use auto_lang::{trans_rust_with_session, compile::CompileSession};
///
/// let mut session = CompileSession::new();
/// let result = trans_rust_with_session(&mut session, "test.at").unwrap();
/// println!("{}", result);
/// ```
pub fn trans_rust_with_session(session: &mut CompileSession, path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)?;

    // Compile source with incremental support (for dirty-tracking / caching)
    let frag_ids = session.compile_source(&code, path)?;

    // Plan 204 Phase 6A: Use .a2r.rs suffix to avoid overwriting .rs files
    let rsname = path.replace(".at", ".a2r.rs");
    let fname = AutoPath::new(path).filename();

    // Re-parse for transpilation using the full pipeline (not incremental),
    // so that type declarations (structs, enums, etc.) are emitted.
    let mut parser = Parser::from(code.as_str());
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let mut ast = parser.parse()?;

    // Run CTEE (Compile-Time Execution Engine) to transform AST
    let mut ctee = crate::comptime::CTEE::new();
    ctee.transform(&mut ast)?;

    // Full transpilation via RustTrans::trans()
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::rust::RustTrans::new(fname);
    trans.trans(ast, &mut sink)?;

    // Write output file
    let source_bytes = sink.done()?;
    if !source_bytes.is_empty() {
        std::fs::write(&rsname, source_bytes)?;

        // Plan 204 Phase 6B: Basic output validation
        let source_str = String::from_utf8_lossy(source_bytes);
        validate_rust_output(&rsname, &source_str);
    }

    Ok(format!(
        "[trans] {} -> {} ({} fragments tracked)",
        path,
        rsname,
        frag_ids.len()
    ))
}

/// Plan 204 Phase 6B: Basic structural validation of transpiled Rust output.
///
/// Checks bracket matching and basic structural validity.
/// Logs warnings if validation fails but does not block writing.
fn validate_rust_output(path: &str, content: &str) {
    let mut paren_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut in_string = false;
    let mut in_char = false;
    let mut escape_next = false;

    for ch in content.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if ch == '\\' && (in_string || in_char) {
            escape_next = true;
            continue;
        }
        if ch == '"' && !in_char {
            in_string = !in_string;
            continue;
        }
        if ch == '\'' && !in_string {
            in_char = !in_char;
            continue;
        }
        if in_string || in_char {
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            _ => {}
        }
    }

    let mut warnings = Vec::new();
    if paren_depth != 0 {
        warnings.push(format!("unbalanced parentheses (depth: {})", paren_depth));
    }
    if brace_depth != 0 {
        warnings.push(format!("unbalanced braces (depth: {})", brace_depth));
    }
    if bracket_depth != 0 {
        warnings.push(format!("unbalanced brackets (depth: {})", bracket_depth));
    }

    if !warnings.is_empty() {
        eprintln!(
            "[a2r warning] Output validation failed for {}: {}",
            path,
            warnings.join(", ")
        );
    }
}

/// Transpile AutoLang file to Python
pub fn trans_python(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    let pyname = path.replace(".at", ".py");
    let fname = AutoPath::new(path).filename();

    // Plan 091: PythonTrans no longer needs Universe, but Parser still requires it
    // Plan 091: Universe removed
    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code.as_str());
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::python::PythonTrans::new(fname);
    // Note: PythonTrans no longer uses Universe
    trans.trans(ast, &mut sink)?;

    // Write Python file
    std::fs::write(&pyname, sink.done()?)?;

    Ok(format!("[trans] {} -> {}", path, pyname))
}

/// Transpile AutoLang file to JavaScript
pub fn trans_javascript(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    let jsname = path.replace(".at", ".js");
    let fname = AutoPath::new(path).filename();

    // Plan 091: JavaScriptTrans no longer needs Universe, but Parser still requires it
    // Plan 091: Universe removed
    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code.as_str());
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::javascript::JavaScriptTrans::new(fname);
    // Note: JavaScriptTrans no longer uses Universe
    trans.trans(ast, &mut sink)?;

    // Write JavaScript file
    std::fs::write(&jsname, sink.done()?)?;

    Ok(format!("[trans] {} -> {}", path, jsname))
}

/// Transpile AutoLang file to TypeScript (Plan 100: a2js → a2ts)
pub fn trans_typescript(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let tsname = path.replace(".at", ".ts");
    let fname = AutoPath::new(path).filename();

    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code.as_str());
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::typescript::TypeScriptTrans::new(fname);
    trans.trans(ast, &mut sink)?;

    // Write TypeScript file
    std::fs::write(&tsname, sink.done()?)?;

    // Write runtime file if any runtime symbols were used
    if trans.needs_range || trans.needs_print {
        let runtime_dir = std::path::Path::new(path).parent()
            .unwrap_or(std::path::Path::new("."));
        let runtime_path = runtime_dir.join("runtime.ts");
        std::fs::write(&runtime_path, crate::trans::typescript::ts_runtime::runtime_file_content())?;
    }

    Ok(format!("[trans] {} -> {}", path, tsname))
}

/// Transpile AutoLang file to TypeScript with custom output path
pub fn trans_typescript_to(path: &str, output: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let fname = AutoPath::new(path).filename();

    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code.as_str());
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::typescript::TypeScriptTrans::new(fname);
    trans.trans(ast, &mut sink)?;

    // Write TypeScript file to custom output path
    std::fs::write(output, sink.done()?)?;

    // Write runtime file if any runtime symbols were used
    if trans.needs_range || trans.needs_print {
        let runtime_dir = std::path::Path::new(output).parent()
            .unwrap_or(std::path::Path::new("."));
        let runtime_path = runtime_dir.join("runtime.ts");
        std::fs::write(&runtime_path, crate::trans::typescript::ts_runtime::runtime_file_content())?;
    }

    Ok(format!("[trans] {} -> {}", path, output))
}

/// Transpile Rust file to AutoLang (Plan 173: r2a)
pub fn transpile_r2a_file(path: &str) -> AutoResult<String> {
    let rust_code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");
    crate::trans::r2a::transpile_r2a(name, &rust_code)
}

// ============================================================================
// Plan 096: UI Backend Generators
// ============================================================================

/// Build UI components from Auto files using AURA pipeline
///
/// This is the main entry point for UI scenario compilation.
///
/// # Arguments
/// * `path` - Input file or directory
/// * `scenario` - Compilation scenario (core, ui, shell)
/// * `backend` - Backend target (vue, rust)
/// * `output` - Optional output directory
pub fn ui_build(
    path: &str,
    scenario: &str,
    backend: &str,
    output: Option<&str>,
) -> AutoResult<String> {
    use crate::session::CompilerSession;
    use crate::ui_gen::{BackendGenerator, VueGenerator, RustGenerator, JetGenerator};

    // Parse scenario
    let session = match scenario {
        "ui" => CompilerSession::ui().with_backend(backend),
        "core" => CompilerSession::default(),
        "shell" => CompilerSession::shell(),
        _ => CompilerSession::ui().with_backend(backend),
    };

    // Read input file
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse with scenario
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session.clone());
    let ast = parser.parse().map_err(|e| {
        format!("Parse error: {:?}", e)
    })?;

    // Extract AURA widgets from AST
    let mut widgets = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            // Convert WidgetDecl to AuraWidget
            let aura_widget = crate::aura::extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;
            widgets.push(aura_widget);
        }
    }

    if widgets.is_empty() {
        return Err("No widget declarations found in input file".into());
    }

    // Generate code based on backend
    // Rust backend uses auto-ui abstract components (Iced, GPUI handled by auto-ui crate)
    // Jet backend generates Kotlin/Compose code for Android
    let mut output_code = String::new();
    match backend {
        "vue" => {
            let mut gen = VueGenerator::new();
            for widget in &widgets {
                let code = gen.generate(widget).map_err(|e| e.to_string())?;
                output_code.push_str(&code);
                output_code.push_str("\n\n");
            }
        }
        "rust" => {
            let mut gen = RustGenerator::new();
            for widget in &widgets {
                let code = gen.generate(widget).map_err(|e| e.to_string())?;
                output_code.push_str(&code);
                output_code.push_str("\n\n");
            }
        }
        "jet" => {
            let mut gen = JetGenerator::new();
            for widget in &widgets {
                let code = gen.generate(widget).map_err(|e| e.to_string())?;
                output_code.push_str(&code);
                output_code.push_str("\n\n");
            }
        }
        _ => {
            return Err(format!("Unknown backend: {}. Available: vue, rust, jet", backend).into());
        }
    }

    // Write output if specified
    if let Some(out_dir) = output {
        let ext = match backend {
            "vue" => "vue",
            "rust" => "rs",
            "jet" => "kt",
            _ => "txt",
        };
        std::fs::create_dir_all(out_dir).ok();
        for widget in &widgets {
            let out_path = std::path::Path::new(out_dir)
                .join(format!("{}.{}", widget.name, ext));
            // Generate individual widget code
            let widget_code = match backend {
                "vue" => {
                    let mut gen = VueGenerator::new();
                    gen.generate(widget).map_err(|e| e.to_string())?
                }
                "rust" => {
                    let mut gen = RustGenerator::new();
                    gen.generate(widget).map_err(|e| e.to_string())?
                }
                "jet" => {
                    let mut gen = JetGenerator::new();
                    gen.generate(widget).map_err(|e| e.to_string())?
                }
                _ => output_code.clone(),
            };
            std::fs::write(&out_path, &widget_code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
    }

    Ok(output_code)
}

/// Build UI components with shadcn-vue mode enabled
///
/// This is a convenience function for generating Vue components
/// with shadcn-vue support enabled.
pub fn ui_build_shadcn(
    path: &str,
    output: Option<&str>,
) -> AutoResult<String> {
    use crate::session::CompilerSession;
    use crate::ui_gen::{BackendGenerator, VueGenerator, VueMode};

    // Read input file
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse with UI scenario
    let session = CompilerSession::ui().with_backend("vue");
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session);
    let ast = parser.parse().map_err(|e| {
        format!("Parse error: {:?}", e)
    })?;

    // Extract AURA widgets from AST
    let mut widgets = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = crate::aura::extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;
            widgets.push(aura_widget);
        }
    }

    if widgets.is_empty() {
        return Err("No widget declarations found in input file".into());
    }

    // Generate code with shadcn-vue mode
    let mut gen = VueGenerator::new().with_mode(VueMode::Shadcn);
    let mut output_code = String::new();

    for widget in &widgets {
        let code = gen.generate(widget).map_err(|e| e.to_string())?;
        output_code.push_str(&code);
        output_code.push_str("\n\n");
    }

    // Write output if specified
    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for widget in &widgets {
            let out_path = std::path::Path::new(out_dir)
                .join(format!("{}.vue", widget.name));
            let mut gen = VueGenerator::new().with_mode(VueMode::Shadcn);
            let widget_code = gen.generate(widget).map_err(|e| e.to_string())?;
            std::fs::write(&out_path, &widget_code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
    }

    Ok(output_code)
}

/// Build UI components with shadcn-vue mode enabled and return parsed widgets
///
/// This function is similar to `ui_build_shadcn` but also returns the parsed
/// AuraWidget structs, allowing callers to inspect widget metadata like routes.
///
/// # Arguments
/// * `path` - Input file path
/// * `output` - Optional output directory for generated files
///
/// # Returns
/// A tuple of (generated_code, widgets) on success
///
/// # Example
/// ```no_run
/// use auto_lang::ui_build_shadcn_with_widgets;
///
/// let (code, widgets) = ui_build_shadcn_with_widgets("app.at", None).unwrap();
///
/// // Check if any widget has routes
/// let has_routes = widgets.iter().any(|w| w.routes.is_some());
/// ```
pub fn ui_build_shadcn_with_widgets(
    path: &str,
    output: Option<&str>,
) -> AutoResult<(String, Vec<crate::aura::AuraWidget>)> {
    use crate::session::CompilerSession;
    use crate::ui_gen::{BackendGenerator, VueGenerator, VueMode};

    // Read input file
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse with UI scenario
    let session = CompilerSession::ui().with_backend("vue");
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session);

    let ast = parser.parse().map_err(|e| {
        format!("Parse error: {:?}", e)
    })?;

    // Extract AURA widgets from AST
    let mut widgets = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = crate::aura::extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;
            widgets.push(aura_widget);
        }
    }

    if widgets.is_empty() {
        return Err("No widget declarations found in input file".into());
    }

    // Generate code with shadcn-vue mode
    let mut gen = VueGenerator::new().with_mode(VueMode::Shadcn);
    let mut output_code = String::new();

    for widget in &widgets {
        let code = gen.generate(widget).map_err(|e| e.to_string())?;
        output_code.push_str(&code);
        output_code.push_str("\n\n");
    }

    // Write output if specified
    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for widget in &widgets {
            let out_path = std::path::Path::new(out_dir)
                .join(format!("{}.vue", widget.name));
            let mut gen = VueGenerator::new().with_mode(VueMode::Shadcn);
            let widget_code = gen.generate(widget).map_err(|e| e.to_string())?;
            std::fs::write(&out_path, &widget_code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
    }

    Ok((output_code, widgets))
}

// ============================================================================
// Plan 075: Unified Compilation API for Multiple Execution Modes
// ============================================================================

/// Compilation mode for AutoLang source files
///
/// AutoLang supports three execution modes:
/// - **Script**: Normal program execution (default)
/// - **Config**: Configuration file compilation (returns unified object)
/// - **Template**: Template file compilation (returns concatenated string)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    Script,
    Config,
    Template,
}

/// Run AutoLang source code with specified compilation mode
///
/// This is the unified entry point for all three execution modes.
/// The mode determines how the source code is compiled.
///
/// # Arguments
/// * `source` - AutoLang source code
/// * `mode` - Compilation mode (Script, Config, or Template)
///
/// # Returns
/// * String representation of the compiled bytecode module
///
/// # Example
/// ```no_run
/// use auto_lang::{run_with_mode, CompileMode};
///
/// // Script mode (default)
/// let result = run_with_mode("1 + 2", CompileMode::Script).unwrap();
///
/// // Config mode
/// let config = r#"
/// server.host = "localhost"
/// server.port = 8080
/// "#;
/// let result = run_with_mode(config, CompileMode::Config).unwrap();
///
/// // Template mode
/// let template = r#""Hello, "
/// "World!""#;
/// let result = run_with_mode(template, CompileMode::Template).unwrap();
/// ```
pub fn run_with_mode(source: &str, mode: CompileMode) -> AutoResult<String> {
    use crate::vm::codegen::Codegen;
    use crate::vm::config_codegen::ConfigCodegen;
    use crate::vm::loader::Module;
    use crate::vm::template_codegen::TemplateCodegen;

    let mut parser = Parser::from(source);
    let code = parser.parse()?;

    let module: Module = match mode {
        CompileMode::Script => {
            let mut codegen = Codegen::new();
            // Compile each statement
            for stmt in &code.stmts {
                codegen.compile_stmt(stmt)?;
            }
            codegen.finish("script".to_string())
        }
        CompileMode::Config => {
            let mut configgen = ConfigCodegen::new();
            configgen.compile_config(&code)?;
            configgen.finish("config".to_string())
        }
        CompileMode::Template => {
            let mut tgen = TemplateCodegen::new();
            tgen.compile_template(&code)?;
            tgen.finish("template".to_string())
        }
    };

    // Return bytecode module info
    Ok(format!(
        "Module: {} bytecode={}, strings={}, exports={}, relocs={}",
        module.name,
        module.code.len(),
        module.strings.len(),
        module.exports.len(),
        module.relocs.len()
    ))
}

/// Detect compilation mode from file extension
///
/// # Arguments
/// * `path` - File path to examine
///
/// # Returns
/// * Detected compilation mode
///
/// # File Extension Mapping
/// - `.config.at` → Config mode
/// - `.template.at` → Template mode
/// - `.at` or other → Script mode (default)
///
/// # Example
/// ```no_run
/// use auto_lang::{detect_mode_from_extension, CompileMode};
/// use std::path::Path;
///
/// let mode = detect_mode_from_extension(Path::new("database.config.at")).unwrap();
/// assert_eq!(mode, CompileMode::Config);
/// ```
pub fn detect_mode_from_extension(path: &Path) -> AutoResult<CompileMode> {
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    // Check for special suffixes before extension
    if filename.ends_with(".config.at") {
        return Ok(CompileMode::Config);
    }
    if filename.ends_with(".template.at") {
        return Ok(CompileMode::Template);
    }

    // Default to script mode
    Ok(CompileMode::Script)
}

/// Run AutoLang file with automatic mode detection from file extension
///
/// This is a convenience function that:
/// 1. Reads the file
/// 2. Detects mode from extension (.config.at, .template.at, or default)
/// 3. Compiles and executes with appropriate mode
///
/// # Arguments
/// * `path` - Path to AutoLang source file
///
/// # Returns
/// * String representation of the execution result
///
/// # Example
/// ```no_run
/// use auto_lang::run_file_with_auto_mode;
///
/// // Automatically uses Config mode
/// let result = run_file_with_auto_mode(std::path::Path::new("database.config.at")).unwrap();
///
/// // Automatically uses Template mode
/// let result = run_file_with_auto_mode(std::path::Path::new("email.template.at")).unwrap();
///
/// // Automatically uses Script mode (default)
/// let result = run_file_with_auto_mode(std::path::Path::new("script.at")).unwrap();
/// ```
pub fn run_file_with_auto_mode(path: &Path) -> AutoResult<String> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        crate::error::AutoError::Msg(format!("Failed to read file {}: {}", path.display(), e))
    })?;

    let mode = detect_mode_from_extension(path)?;
    run_with_mode(&source, mode)
}

#[cfg(test)]
mod test_parser_arrow;

#[cfg(test)]
mod test_float_full;

#[cfg(test)]
mod test_double_lexer;

#[cfg(test)]
mod vm_types_tests;

// Plan 076 Phase 1: Generic type support tests
#[cfg(test)]
mod generic_tests;

// Plan 076 Phase 2: Monomorphization tests
#[cfg(test)]
mod monomorphize_tests;

// Plan 076 Phase 4: Storage strategy tests
#[cfg(test)]
mod storage_strategy_tests;

// Plan 076 Phase 5: Integration tests
#[cfg(test)]
mod bigvm_generic_integration_tests;

// Plan 077 Phase 2: Generic ListData<T> tests
#[cfg(test)]
mod generic_list_data_tests;

// Plan 077 Phase 3: HeapObject implementation tests
#[cfg(test)]
mod listdata_heap_object_tests;

// Plan 077 Phase 4: Unified object registry tests
#[cfg(test)]
mod unified_registry_tests;

// Plan 077 Phase 8: Comprehensive integration tests (TODO: Fix compilation errors)
// #[cfg(test)]
// mod plan077_integration_tests;

#[cfg(test)]
mod tests;

// =============================================================================
// Plan 015: AutoUI Core (feature-gated)
// =============================================================================

#[cfg(feature = "ui")]
pub mod ui;

// Re-export UI types when feature is enabled
#[cfg(feature = "ui")]
pub use ui::{
    Component, View, ViewBuilder,
    VNodeId, VNodeKind, VNode, VNodeProps, VTree,
    view_to_vtree,
    App, AppResult,
    Style,
};

#[cfg(feature = "ui-interpreter")]
pub use ui::{
    interpreter::{InterpreterBridge, DynamicMessage},
    event_router::{EventRouter, EventType, EventContext},
    hot_reload::{HotReloadComponent, UIWatcher},
};

#[cfg(feature = "ui-iced")]
pub use ui::iced::{IntoIcedElement, ComponentIced};

#[cfg(feature = "ui-gpui")]
pub use ui::gpui::{IntoGpuiElement, ComponentGpui, GpuiComponentState, VNodeEntity};
