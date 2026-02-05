pub mod ast;
pub mod atom;
pub mod atom_error;
pub mod compile;
pub mod config;
pub mod database;
pub mod dep;
pub mod error;
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
pub mod patch;
pub mod repl;
pub mod runtime;
pub mod scope;
pub mod target;
pub mod token;
pub mod trait_checker;
pub mod trans;
mod universe;
pub mod util;
pub mod vm;

pub use atom::{Atom, AtomReader};

// 过程宏 - 支持 AutoLang 语法的内嵌 DSL
// 这些宏接受 AutoLang 代码字符串并解析为 Atom/Node/Value 结构体
pub use auto_lang_macros::{atom, node, value};



use crate::scope::Meta;
use crate::trans::c::CTrans;
pub use crate::universe::{SymbolLocation, Universe};
use crate::compile::CompileSession;
use crate::{eval::EvalMode, trans::Sink, trans::Trans};
use auto_val::{AutoPath, Obj, Shared};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::AutoResult;

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

pub fn run(code: &str) -> AutoResult<String> {
    let mut interpreter = interp::Interpreter::new();

    // Try to interpret, and attach source code if we get a syntax error
    interpreter.interpret(code)?;
    Ok(interpreter.result.repr().to_string())
}

/// Run code and collect all errors during parsing
///
/// This function enables error recovery to collect multiple syntax errors
/// instead of aborting on the first error.
pub fn run_with_errors(code: &str) -> AutoResult<String> {
    let mut interpreter = interp::Interpreter::new();
    // Enable error recovery
    interpreter.enable_error_recovery();
    interpreter.interpret(code)?;
    Ok(interpreter.result.repr().to_string())
}

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

    let mut interpreter = interp::Interpreter::new();
    interpreter.interpret(&code)?;
    Ok(interpreter.result.repr().to_string())
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

#[cfg(test)]
mod tests;