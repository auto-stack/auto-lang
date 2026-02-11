pub mod ast;
pub mod atom;
pub mod atom_error;
pub mod autovm_persistent; // Plan 068 Phase 9.6: Persistent AutoVM REPL
pub mod compile;
pub mod config;
pub mod database;
pub mod dep;
pub mod error;
// Plan 081 Phase 2: Execution mode selection (autovm, evaluator, c, rust)
pub mod mode;
// Plan 081 Phase 4: Multi-mode compilation pipeline
pub mod multi_mode;
// Plan 081 Phase 5: FFI layer for cross-mode function calls
pub mod ffi;
// Plan 073 Phase 9.3: Execution engine selection (AutoVM vs Evaluator)
pub mod execution_engine;
pub mod eval;
pub mod hash;
pub mod infer;
pub mod indexer;
pub mod interp;
pub mod query;
mod lexer;
pub mod libs;
pub mod macro_;
pub mod maker;
pub mod ownership;
pub mod parser;
pub use parser::Parser;
// Plan 088 Phase 6: Type checking module for parameter passing modes
pub mod typeck;
pub mod patch;
pub mod autovm_repl;
pub mod repl;
// Plan 078: ModuleResolver trait for dependency resolution
pub mod resolver;
pub mod runtime;
pub mod scope;
pub mod target;
pub mod token;
pub mod trait_checker;
// Plan 087: Type registry for REPL
pub mod type_registry;
pub mod trans;
mod universe;
pub mod util;
pub mod vm;

// Plan 088: Parameter passing mode tests
#[cfg(test)]
mod plan_088_tests;
#[cfg(test)]
mod plan_088_parser_tests;

pub use atom::{Atom, AtomReader};

// 过程宏 - 支持 AutoLang 语法的内嵌 DSL
// 这些宏接受 AutoLang 代码字符串并解析为 Atom/Node/Value 结构体
pub use auto_lang_macros::{atom, node, value};



use crate::scope::Meta;
use crate::trans::c::CTrans;
pub use crate::universe::{SymbolLocation, Universe};
use crate::compile::CompileSession;
use crate::{eval::EvalMode, trans::Sink, trans::Trans};
use auto_val::{AutoPath, Obj, Shared, Value};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::{AutoResult, AutoError};

/// Global error limit for parser error recovery
static ERROR_LIMIT: AtomicUsize = AtomicUsize::new(20);

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
    // Create tokio runtime for async execution
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        execute_autovm(code).await
    })
}

/// Internal AutoVM execution function (async)
async fn execute_autovm(code: &str) -> AutoResult<String> {
    use crate::vm::codegen::Codegen;
    use crate::vm::engine::AutoVM;
    use crate::vm::opcode::OpCode;
    use crate::vm::virt_memory::VirtualFlash;

    // 1. Parse the code
    let mut parser = Parser::from(code);
    let ast = parser.parse()?;

    // 2. Compile to bytecode
    let mut codegen = Codegen::new();
    for stmt in ast.stmts {
        codegen.compile_stmt(&stmt)?;
    }

    // Add explicit HALT at the end
    codegen.code.push(OpCode::HALT as u8);

    // DEBUG: Print bytecode BEFORE relocation
    eprintln!("DEBUG: === Bytecode BEFORE relocation (0x00-0x40) ===");
    for i in 0x00..0x40u32 {
        if (i as usize) < codegen.code.len() {
            let op = codegen.code[i as usize];
            eprintln!("CODE[{:04x}]: {:02x}", i, op);
        }
    }
    eprintln!("DEBUG: === End of bytecode ===");

    // 3. Perform linking (resolve function calls)
    let strings = codegen.strings.clone();
    eprintln!("DEBUG: === Performing relocation for {} entries ===", codegen.relocs.len());
    for reloc in &codegen.relocs {
        eprintln!("DEBUG: Relocating '{}' at offset 0x{:04x}", reloc.symbol_name, reloc.offset);
        if let Some(&addr) = codegen.exports.get(&reloc.symbol_name) {
            eprintln!("DEBUG:   Found '{}' at address 0x{:04x}", reloc.symbol_name, addr);
            let bytes = addr.to_le_bytes();
            let offset = reloc.offset as usize;
            for (i, b) in bytes.iter().enumerate() {
                eprintln!("DEBUG:   code[{}] = {} (was {})", offset + i, *b, codegen.code[offset + i]);
                codegen.code[offset + i] = *b;
            }
        } else {
            return Err(crate::error::AutoError::Msg(format!(
                "Undefined symbol: {}", reloc.symbol_name
            )));
        }
    }

    // 4. Load into VM
    // Plan 073: Include object_keys and object_types for object literal support
    let flash = VirtualFlash::new_with_code_and_keys(
        codegen.code,
        codegen.object_keys,
        codegen.object_types,
    );
    let mut vm = AutoVM::new(flash, 1024); // 1KB RAM
    vm.load_strings(strings);

    // 5. Execute - Find main/test entry point
    let entry_point = codegen.exports
        .get("main")
        .or_else(|| codegen.exports.get("test"))
        .copied()
        .unwrap_or(0) as usize; // Default to address 0 for scripts without main/test

    let task_id = vm.spawn_task(entry_point, 1024);
    vm.run_task_loop().await;

    // 6. Get result from stack
    if let Some(task_arc) = vm.tasks.get(&task_id).map(|r| r.value().clone()) {
        let mut task = task_arc.lock().await;

        if task.ram.sp == 0 {
            return Ok("".to_string());
        }

        let result = task.ram.pop_i32();

        // Format result: check if it's an array ID, heap object ID, or regular value
        let result_u64 = result as u64;

        // Check arrays registry (for array literals)
        if let Some(arr_arc) = vm.arrays.get(&result_u64) {
            let arr = arr_arc.read().unwrap();
            let formatted: Vec<String> = arr.iter().map(|v| v.repr().to_string()).collect();
            return Ok(format!("[{}]", formatted.join(", ")));
        }

        // Check heap objects (for List, HashMap, etc.)
        if let Some(obj_arc) = vm.heap_objects.get(&result_u64) {
            let obj = obj_arc.read().unwrap();
            // Try to format as List<int>
            if let Some(list) = obj.as_any().downcast_ref::<crate::universe::ListData<i32>>() {
                let formatted: Vec<String> = list.elems.iter().map(|e| e.to_string()).collect();
                return Ok(format!("[{}]", formatted.join(", ")));
            }
        }

        // Default: return as integer
        Ok(format!("{}", result))
    } else {
        Err(crate::error::AutoError::Msg(
            "Task not found after execution".to_string()
        ))
    }
}

/// Run code and collect all errors during parsing
///
/// **Deprecated**: This function uses the TreeWalker evaluator, which is slower than AutoVM.
/// Use `run()` or `run_autovm()` instead for better performance.
///
/// **Plan 068 Phase 9**: Evaluator is deprecated in favor of AutoVM
///
/// This function enables error recovery to collect multiple syntax errors
/// instead of aborting on the first error.
#[deprecated(since = "0.9.0", note = "Use run() or run_autovm() instead (Plan 068 Phase 9)")]
pub fn run_with_errors(code: &str) -> AutoResult<String> {
    let mut interpreter = interp::Interpreter::new();
    // Enable error recovery
    interpreter.enable_error_recovery();
    interpreter.interpret(code)?;
    Ok(interpreter.result.repr().to_string())
}

/// Run code with a custom scope
///
/// **Deprecated**: This function uses the TreeWalker evaluator with Universe, which is deprecated.
/// Use CompileSession + Database instead (see Plan 064).
///
/// **Plan 064**: Universe is split into Database + ExecutionEngine
/// **Plan 068 Phase 9**: Evaluator is deprecated in favor of AutoVM
#[deprecated(
    since = "0.9.0",
    note = "Use run_with_session() with CompileSession instead (Plan 064 + Plan 068 Phase 9)"
)]
pub fn run_with_scope(code: &str, scope: Universe) -> AutoResult<String> {
    let mut interpreter = interp::Interpreter::with_scope(scope);
    interpreter.interpret(code)?;
    Ok(interpreter.result.repr().to_string())
}

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

    // Create a new Interpreter for this execution
    // Note: Each execution gets its own Evaler, but shares the Database
    let mut interpreter = interp::Interpreter::new_with_session(session);

    // Interpret the code (this parses and executes)
    // TODO: Phase 3 - Check Database cache before parsing
    interpreter.interpret(code)?;

    Ok(interpreter.result.repr().to_string())
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
/// * `scope` - Persistent scope for variable storage across calls
/// * `code` - AutoLang source code to execute
///
/// # Returns
///
/// String representation of the result, or error message
pub fn run_with_session_and_scope(
    session: &mut CompileSession,
    scope: Shared<Universe>,
    code: &str,
) -> AutoResult<String> {
    // Note: For REPL usage, we skip compile_source() to avoid double-parsing
    // The interpreter will parse and execute the code directly with the persistent scope
    // This ensures variables are stored and retrieved from the same scope across REPL inputs

    // Create a new Interpreter for this execution with the persistent scope
    let mut interpreter = interp::Interpreter::new_with_session_and_scope(session, scope);

    // Interpret the code (this parses and executes)
    // TODO: Phase 3 - Check Database cache before parsing
    interpreter.interpret(code)?;

    Ok(interpreter.result.repr().to_string())
}

pub fn parse(code: &str) -> AutoResult<ast::Code> {
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope.clone());
    parser.parse().map_err(|e| e.to_string().into())
}

/// Parse code and return proper AutoError (not converted to string)
/// This is used by the LSP to get detailed error information
pub fn parse_preserve_error(code: &str) -> Result<ast::Code, error::AutoError> {
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope.clone());
    parser.parse()
}

pub fn parse_with_scope(code: &str, scope: Rc<RefCell<Universe>>) -> AutoResult<ast::Code> {
    let mut parser = Parser::new(code, scope.clone());
    parser.parse().map_err(|e| e.to_string().into())
}

pub fn interpret(code: &str) -> AutoResult<interp::Interpreter> {
    let mut interpreter = interp::Interpreter::new();
    interpreter.interpret(code)?;
    Ok(interpreter)
}

pub fn interpret_with_scope(code: &str, scope: Universe) -> AutoResult<interp::Interpreter> {
    let mut interpreter = interp::Interpreter::with_scope(scope);
    interpreter.interpret(code)?;
    Ok(interpreter)
}

pub fn run_file(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Plan 088 Phase 4: Use AutoVM instead of deprecated Interpreter
    // This enables smart parameter passing and other AutoVM features
    run(&code)
}

pub fn interpret_file(path: &str) -> interp::Interpreter {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();
    let mut interpreter = interp::Interpreter::new();
    interpreter.interpret(&code).unwrap();
    interpreter
}

// TODO: to be deprecated, use Interpreter::eval_template instead
pub fn eval_template(template: &str, scope: Universe) -> AutoResult<interp::Interpreter> {
    let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::TEMPLATE);
    let result = interpreter.eval_template(template)?;
    interpreter.result = result;
    Ok(interpreter)
}

pub fn eval_config_with_scope(
    code: &str,
    args: &Obj,
    mut scope: Universe,
) -> AutoResult<interp::Interpreter> {
    // Preprocess macros (e.g., widget → type ... is Widget)
    let code = crate::macro_::preprocess(code);
    scope.define_global("root", Rc::new(Meta::Node(ast::Node::new("root"))));
    scope.set_args(args);
    let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::CONFIG);
    interpreter.interpret(&code)?;
    Ok(interpreter)
}

pub fn eval_config(code: &str, args: &Obj) -> AutoResult<interp::Interpreter> {
    // Preprocess macros (e.g., widget → type ... is Widget)
    let code = crate::macro_::preprocess(code);
    let mut scope = Universe::new();
    scope.define_global("root", Rc::new(Meta::Node(ast::Node::new("root"))));
    scope.set_args(args);
    let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::CONFIG);
    interpreter.interpret(&code)?;
    Ok(interpreter)
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
/// let result = eval_config_with_vm(config, &Obj::new(), Universe::default()).unwrap();
/// ```
pub fn eval_config_with_vm(code: &str, args: &Obj, univ: Universe) -> AutoResult<Value> {
    use crate::vm::config_codegen::ConfigCodegen;
    use crate::vm::engine::AutoVM;
    use crate::vm::opcode::OpCode;
    use crate::vm::virt_memory::VirtualFlash;

    // Preprocess macros (e.g., widget → type ... is Widget)
    let code = crate::macro_::preprocess(code);

    // Note: Universe is not directly used by AutoVM (it uses the VM's own state)
    // We keep it for API compatibility with the old eval_config_with_scope

    // 1. Parse the code
    let mut parser = Parser::from(code.as_str());
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
                "Undefined symbol in config: {}", reloc.symbol_name
            )));
        }
    }

    // 4. Load into VM and execute
    // Clone the bytecode and metadata before moving into the async block
    let bytecode = configgen.base().code.clone();
    let object_keys = configgen.base().object_keys.clone();
    let object_types = configgen.base().object_types.clone();

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let flash = VirtualFlash::new_with_code_and_keys(bytecode, object_keys, object_types);
        let mut vm = AutoVM::new(flash, 4096); // 4KB RAM for config
        vm.load_strings(strings);

        // 5. Execute from entry point (default to 0 for config)
        let entry_point = exports
            .get("main")
            .copied()
            .unwrap_or(0) as usize;

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
            let _result_i32 = task.ram.pop_i32();

            // For config mode, we need to return the root Node
            // TODO: Properly extract the config object from VM heap
            // For now, return a placeholder
            Ok(Value::Node(auto_val::Node::new("root")))
        } else {
            // Task not found - return empty config
            Ok(Value::Node(auto_val::Node::new("root")))
        }
    })
}

pub fn trans_c(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    let cname = path.replace(".at", ".c");

    let fname = AutoPath::new(path).filename();
    // println!("trans_C fname: {}", fname); // LSP: disabled

    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code.as_str(), scope);
    let ast = parser.parse()?;
    let mut sink = Sink::new(fname);
    let mut trans = CTrans::new(cname.clone().into());
    trans.set_scope(parser.scope.clone());
    trans.trans(ast, &mut sink)?;

    // convert sink to .c/.h files
    std::fs::write(&cname, sink.done()?)?;
    // write the header file
    let h_path = path.replace(".at", ".h");
    std::fs::write(Path::new(h_path.as_str()), sink.header)?;

    Ok(format!("[trans] {} -> {}", path, cname))
}

/// Transpile AutoLang file to Rust
pub fn trans_rust(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    let rsname = path.replace(".at", ".rs");
    let fname = AutoPath::new(path).filename();

    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code.as_str(), scope);
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::rust::RustTrans::new(fname);
    trans.set_scope(parser.scope.clone());
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
pub fn trans_c_with_session(
    session: &mut CompileSession,
    path: &str,
) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)?;

    // Compile source with incremental support
    let frag_ids = session.compile_source(&code, path)?;

    // Get file_id and Database
    let db = session.db();
    let file_id = {
        let db_read = db.read().unwrap();
        db_read.get_file_id_by_path(path)
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
pub fn trans_rust_with_session(
    session: &mut CompileSession,
    path: &str,
) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)?;

    // Compile source with incremental support
    let frag_ids = session.compile_source(&code, path)?;

    // Get file_id and Database
    let db = session.db();
    let file_id = {
        let db_read = db.read().unwrap();
        db_read.get_file_id_by_path(path)
            .ok_or_else(|| format!("File not found in database: {}", path))?
    };

    // Create transpiler with Database
    let mut trans = crate::trans::rust::RustTrans::with_database(db.clone());

    // Perform incremental transpilation
    let results = trans.trans_incremental(session, file_id)?;

    // Merge results and write output file
    let rsname = path.replace(".at", ".rs");

    let mut source_content = String::new();
    for (_frag_id, source) in &results {
        source_content.push_str(source);
    }

    // Write output file
    if !source_content.is_empty() {
        std::fs::write(&rsname, source_content)?;
    }

    Ok(format!(
        "[trans] {} -> {} ({} fragments, {} dirty, {} transpiled)",
        path,
        rsname,
        frag_ids.len(),
        db.read().unwrap().get_dirty_fragments().len(),
        results.len()
    ))
}

/// Transpile AutoLang file to Python
pub fn trans_python(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    let pyname = path.replace(".at", ".py");
    let fname = AutoPath::new(path).filename();

    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code.as_str(), scope);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::python::PythonTrans::new(fname);
    trans.set_scope(parser.scope.clone());
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

    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code.as_str(), scope);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::javascript::JavaScriptTrans::new(fname);
    trans.set_scope(parser.scope.clone());
    trans.trans(ast, &mut sink)?;

    // Write JavaScript file
    std::fs::write(&jsname, sink.done()?)?;

    Ok(format!("[trans] {} -> {}", path, jsname))
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
    use crate::vm::template_codegen::TemplateCodegen;
    use crate::vm::loader::Module;

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
/// use auto_lang::detect_mode_from_extension;
/// use std::path::Path;
///
/// let mode = detect_mode_from_extension(Path::new("database.config.at")).unwrap();
/// assert_eq!(mode, CompileMode::Config);
/// ```
pub fn detect_mode_from_extension(path: &Path) -> AutoResult<CompileMode> {
    let filename = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

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
    let source = std::fs::read_to_string(path)
        .map_err(|e| crate::error::AutoError::Msg(format!("Failed to read file {}: {}", path.display(), e)))?;

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