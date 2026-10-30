//! 脚本/调试/转译文件入口族（PLAN-816 T-05 自 auto-lang 兼容包迁入）。
//!
//! 单源语言 SDK 入口：CLI 直跑透传（`run_file_with_args`）、REPL 脚本
//! 文件、`--dump-lowered`、GDB 风格调试（交互/JSON agent）、逐语言文件
//! 转译（py/js/gd/tscn/godot/ts/ts-to/rust-merged/r2a）。UI 宿主装配经
//! SDK 钩子（[`crate::install_ui_host_capabilities`]——上层链接 UI 实现
//! 时经 inventory/钩子回调真实装配；纯 SDK 消费为 no-op）。动态 UI
//! （`has_ui_keywords` → iced）分岔属宿主层，不在本模块。

use std::cell::RefCell;
use std::rc::Rc;

use auto_val::AutoPath;

use crate::compile;
use crate::error::AutoResult;
use crate::parser::Parser;
use crate::trans::{Sink, Trans};
use crate::{
    block_on_autovm_local, create_vm_from_source, execute_autovm_with_path, find_use_symbol_span,
    get_global_runtime, init_py_ffi, init_rust_ffi, install_ui_host_capabilities, join_execution,
    set_script_args,
};

pub fn run_file(path: &str) -> AutoResult<String> {
    run_file_with_args(path, Vec::new())
}

/// Plan 555 T07: s2s 改写——Auto 脚本糖源 → 正常模式桥源
/// （W1 规则表空置=identity passthrough；W2 糖批逐条落
/// trans::auto_s2s::builtin_rules）。
pub fn trans_auto_s2s(path: &str) -> AutoResult<String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    crate::trans::auto_s2s::lower_source(&src)
}

/// Plan 555 T08: `--dump-lowered` 渲染——模式头 + 改写产物（run/trans
/// 双入口共用）。W1 产物 = passthrough 源；模式头按扩展名解析
/// （pragma 属解析期信号，离线 dump 按扩展名档位）。
pub fn dump_lowered(path: &str) -> AutoResult<String> {
    let lowered = trans_auto_s2s(path)?;
    let ext = path.rsplit('.').next().unwrap_or("");
    let mode = crate::mode::resolve_script_mode(Some(ext), false, false);
    let header = format!(
        "// lowered (Plan 560 W2 s2s) mode={} src={}
",
        match mode {
            crate::mode::ScriptMode::Script => "script",
            crate::mode::ScriptMode::Normal => "normal",
        },
        path
    );
    Ok(header + &lowered)
}

/// Plan 524: run a script with CLI pass-through args (`auto <file> [args...]`
/// 直跑透传形态)。Args land in the process-wide `process.args()` 表 =
/// [程序路径] + args。每次调用覆盖前值（同进程串行多次运行不残留）。
pub fn run_file_with_args(path: &str, args: Vec<String>) -> AutoResult<String> {
    set_script_args(args);
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // PLAN-816 T-05：动态 UI 分岔（has_ui_keywords → iced 宿主）是宿主层
    // 装配——薄 CLI 经 auto_ui_runtime 在入口侧分派；SDK 入口恒为纯语言
    // 执行（等价旧无 ui-iced 构建形态）。

    // Plan 327: True multi-module loading via CompileSession. The source
    // file's directory is added to source_dirs, so `use db` resolves db.at
    // relative to the source. No string flattening — modules are compiled
    // separately and linked by the Linker.
    run_with_path(&code, path)
}

/// Run code with a source file path for module resolution.
fn run_with_path(code: &str, path: &str) -> AutoResult<String> {
    let launch = crate::vm::host_capabilities::http_service::invocation_launch();
    let code = code.to_string();
    let path = path.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            crate::vm::host_capabilities::http_service::with_http_service_launch(launch, || {
                block_on_autovm_local(async {
                    execute_autovm_with_path(&code, true, Some(&path))
                        .await
                        .map(|(r, stdout, _, _)| {
                            if !stdout.is_empty() {
                                println!("{}", stdout);
                            }
                            r
                        })
                })
            })
        })
        .expect("Failed to spawn execution thread");
    join_execution(handle)
}

/// Plan 199 Phase 5: Debug an Auto program with GDB-style interactive debugger
pub fn debug_file(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let rt = get_global_runtime();
    rt.block_on(debug_autovm(&code))
}

/// Plan 199 Phase 5: Compile and debug AutoLang code with GDB-style interactive debugger
async fn debug_autovm(code: &str) -> AutoResult<String> {
    install_ui_host_capabilities();
    use crate::vm::codegen::Codegen;
    use crate::vm::debugger::GdbController;
    use crate::vm::engine::AutoVM;
    use crate::vm::loader::Linker;
    use crate::vm::opcode::OpCode;
    use crate::vm::virt_memory::VirtualFlash;

    // Plan 442 C2: axum serve adapter state reset (see execute_autovm_with_path).
    crate::vm::ffi::axum_adapter::reset();
    // PLAN-669: #[api] param-sig side channel — same reset discipline.
    let http_metadata_scope =
        crate::vm::host_capabilities::http_service::CompilationMetadataGuard::begin();
    crate::vm::ffi::http_server::clear_api_param_sigs();

    // Same compilation pipeline as execute_autovm
    let mut session = compile::CompileSession::new();
    session.collect_rust_imports(code)?;
    session.collect_py_imports(code)?;
    session.resolve_deps(code)?;
    session.resolve_uses(code)?;

    let rust_ffi_native_interface = init_rust_ffi(&session);
    let py_ffi_native_interface = init_py_ffi(&session);

    let mut parser = Parser::new_with_type_store(code, session.type_store());
    let mut ast = parser.parse()?;

    let mut ctee = crate::comptime::CTEE::new();
    ctee.transform(&mut ast)?;

    let mut codegen = Codegen::new_with_type_store(parser.type_store.clone());
    codegen.assembly_context = session.assembly;
    // PLAN-013 T1: 与脚本路径同款——dep 模块(如 stdlib term.vm.at)声明的
    // #[vm] fn 名播种进根 codegen,裸调用才不会被 Plan 347 影子抑制改道 reloc。
    for name in &session.vm_fn_names {
        codegen.vm_fn_names.insert(name.clone());
    }
    let (type_decls, other_stmts): (Vec<_>, Vec<_>) = ast.stmts.iter().partition(|stmt| {
        matches!(
            stmt,
            crate::ast::Stmt::TypeDecl(_)
                | crate::ast::Stmt::Ext(_)
                | crate::ast::Stmt::EnumDecl(_)
        )
    });

    for stmt in &type_decls {
        codegen.compile_stmt(stmt)?;
    }

    if !other_stmts.is_empty() {
        let n_locals = 16;
        codegen.emit_op(OpCode::FN_PROLOG);
        codegen.emit_byte(0);
        codegen.emit_byte(n_locals as u8);
        codegen.emit_op(OpCode::RESERVE_STACK);
        codegen.emit_byte(n_locals as u8);
        for (i, stmt) in ast.stmts.iter().enumerate() {
            if matches!(
                stmt,
                crate::ast::Stmt::TypeDecl(_)
                    | crate::ast::Stmt::Ext(_)
                    | crate::ast::Stmt::EnumDecl(_)
            ) {
                continue;
            }
            if i < ast.source_lines.len() {
                codegen.emit_source_line(ast.source_lines[i]);
            }
            codegen.compile_stmt(stmt)?;
        }
    }

    codegen.code.push(OpCode::HALT as u8);

    let strings = codegen.strings.clone();
    let mut linker = Linker::new();
    let dep_modules = session.take_compiled_modules();
    for module in dep_modules {
        linker.add_module(module);
    }

    let object_keys = codegen.object_keys.clone();
    let object_types = codegen.object_types.clone();
    let _result_type = codegen.last_expr_type.clone();
    let generic_registry = std::mem::take(&mut codegen.generic_registry);
    // Plan 327 Phase 1: take task handler registry before finish() consumes codegen,
    // so HANDLE_MSG opcode can find `task` `on { }` handlers at runtime.
    let task_handler_registry = std::mem::take(&mut codegen.task_handler_registry);
    // Plan 312: Extract API routes before finish() consumes codegen
    let api_routes = codegen.api_routes.clone();
    session
        .assembly_references
        .extend(codegen.assembly_references.iter().cloned());
    session.freeze_assembly_manifest("vm-batch", std::path::Path::new("<main>"), code.as_ref())?;
    let main_module = codegen.finish("<main>".to_string());
    linker.add_entry_module(main_module);

    let (linked_code, global_symbols) = linker.link().map_err(|e| {
        let span = if let Some(pos) = e.source_pos {
            crate::error::pos_to_span(pos)
        } else {
            find_use_symbol_span(code, &e.message)
        };
        crate::error::AutoError::MsgWithSource(crate::error::MsgWithSource {
            source: miette::NamedSource::new("<script>", code.to_string()),
            message: e.message.clone(),
            span,
            help: None,
        })
    })?;

    let main_entry = if let Some(&addr) = global_symbols
        .get("main")
        .or_else(|| global_symbols.get("test"))
    {
        addr as usize
    } else {
        let dep_size: usize = linker
            .modules
            .iter()
            .take(linker.modules.len() - 1)
            .map(|m| m.code.len())
            .sum();
        dep_size
    };

    // Clone exports before moving global_symbols into flash
    let debug_exports = global_symbols.clone();

    let flash = VirtualFlash::from_vec_with_metadata(
        linked_code.clone(),
        global_symbols,
        object_keys,
        object_types,
    );

    // Create VM without capture (debugger needs real stdout)
    let mut vm = AutoVM::new(flash, 1024);
    vm.load_strings(strings);
    vm.load_generic_registry(generic_registry);
    vm.load_task_handler_registry(task_handler_registry); // Plan 327 Phase 1
                                                          // Plan 312: Register #[api] routes for HTTP server dispatch
    crate::vm::host_capabilities::http_service::VmHttpBridge::bind_program_metadata(
        &mut vm,
        http_metadata_scope.finish(&api_routes),
    );
    crate::vm::ffi::stdlib::register_http_routes(api_routes);

    if let Some(rust_ni) = rust_ffi_native_interface {
        vm.merge_native_interface(&rust_ni);
    }
    if let Some(py_ni) = py_ffi_native_interface {
        vm.merge_native_interface(&py_ni);
    }

    // Inject GDB-style debugger controller
    let source_lines: Vec<String> = code.lines().map(|s| s.to_string()).collect();
    let controller = GdbController::new(source_lines, linked_code, debug_exports);
    vm.set_debugger(Box::new(controller));

    let _task_id = vm.spawn_task(main_entry, 16384);
    vm.run_task_loop().await;

    Ok(String::new())
}

/// Plan 199 Phase 8: Debug an Auto program with JSON agent-mode controller.
/// Each pause emits a JSON state line to stdout; the driver reads commands
/// from stdin as simple strings (continue / step / step_over / step_out / stop).
pub fn debug_file_agent(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let rt = get_global_runtime();
    rt.block_on(debug_autovm_agent(&code))
}

async fn debug_autovm_agent(code: &str) -> AutoResult<String> {
    install_ui_host_capabilities();
    use crate::vm::debugger::{AgentCommand, AgentDebugState, AgentRegisters, JsonAgentController};

    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<AgentCommand>();
    let (state_tx, state_rx) = std::sync::mpsc::channel::<AgentDebugState>();

    let source = code.to_string();
    std::thread::spawn(move || {
        let (mut vm, _output_buffer, entry_point, _result_type) =
            match create_vm_from_source(&source) {
                Ok(v) => v,
                Err(e) => {
                    let _ = state_tx.send(AgentDebugState {
                        status: "compile_error".to_string(),
                        line: 0,
                        ip: 0,
                        op: format!("{:?}", e),
                        stack: vec![],
                        call_stack: vec![],
                        locals: vec![],
                        registers: AgentRegisters {
                            ip: 0,
                            bp: 0,
                            sp: 0,
                        },
                        stdout: String::new(),
                    });
                    return;
                }
            };

        let controller = JsonAgentController::new(cmd_rx, state_tx);
        vm.set_debugger(Box::new(controller));

        let rt = tokio::runtime::Runtime::new().expect("tokio runtime for agent debug");
        rt.block_on(async move {
            vm.spawn_task(entry_point, 65536);
            vm.run_task_loop().await;
        });
    });

    // JSON I/O loop on the main thread
    loop {
        match state_rx.recv() {
            Ok(state) => {
                match serde_json::to_string(&state) {
                    Ok(json) => println!("{}", json),
                    Err(e) => eprintln!("{{\"error\":\"serialize failed: {}\"}}", e),
                }
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            Err(_) => break,
        }

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let cmd = match input.trim() {
            "continue" | "c" => AgentCommand::Continue,
            "step" | "s" => AgentCommand::Step,
            "step_over" | "next" | "n" => AgentCommand::StepOver,
            "step_out" | "finish" | "fin" => AgentCommand::StepOut,
            "stop" | "quit" | "q" => AgentCommand::Stop,
            "" => continue,
            other => {
                eprintln!("{{\"error\":\"unknown command: {}\"}}", other);
                continue;
            }
        };
        if cmd_tx.send(cmd).is_err() {
            break;
        }
    }

    Ok(String::new())
}

/// Transpile a multi-file AutoLang project into a single merged Rust file.
///
/// Discovers all .at files starting from `entry_file`, transpiles them
/// into one .rs file with deduplication and merge-specific post-processing.
pub fn trans_rust_merged(entry_file: &str) -> AutoResult<Vec<u8>> {
    crate::trans::rust::transpile_rust_project_merged(entry_file)
}

/// Transpile AutoLang file to Python
pub fn trans_python(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    // Plan 560 T12：.as 输入防原地覆写（replace(".at") 对 .as 不命中
    // → pyname==源路径，转译产物写回源文件——py_math 实证）。
    let pyname = if path.ends_with(".at") || path.ends_with(".as") {
        path[..path.len() - 3].to_string() + ".py"
    } else {
        path.replace(".at", ".py")
    };
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

/// Transpile AutoLang file to GDScript (Plan 290: a2gd)
pub fn trans_gdscript(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    let gdname = path.replace(".at", ".gd");
    let fname = AutoPath::new(path).filename();

    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code.as_str());
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::gdscript::GDScriptTrans::new(fname);
    trans.trans(ast, &mut sink)?;

    // Write GDScript file
    std::fs::write(&gdname, sink.done()?)?;

    Ok(format!("[trans] {} -> {}", path, gdname))
}

/// Transpile an AutoLang scene declaration to a Godot `.tscn` file (Plan 306).
///
/// Reads `path` (an `.at` file containing a `scene` declaration), parses it,
/// finds the `SceneDecl`, and writes a `.tscn` file next to the source.
pub fn trans_tscn(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let tscn_name = path.replace(".at", ".tscn");

    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code.as_str());
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let tscn = crate::trans::tscn::generate_from_ast(&ast)
        .ok_or_else(|| format!("No `scene` declaration found in {}", path))?;

    std::fs::write(&tscn_name, tscn)?;
    Ok(format!("[trans] {} -> {}", path, tscn_name))
}

/// Transpile an AutoLang file to a full Godot node: emit BOTH the `.tscn`
/// scene (from any `scene` declaration) and the `.gd` script (from the rest).
///
/// Plan 306 Phase 2b. A single `.at` file may describe a scene tree and the
/// game logic that drives it; this produces the two Godot files it maps to.
/// The scene's `script = "name.gd"` reference ties the generated script to
/// the scene root.
pub fn trans_godot(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code.as_str());
    let ast = parser.parse().map_err(|e| e.to_string())?;

    let mut messages = Vec::new();

    // 1. Scene -> .tscn (generate before moving `ast` into the GDScript pass).
    if let Some(tscn) = crate::trans::tscn::generate_from_ast(&ast) {
        let tscn_name = path.replace(".at", ".tscn");
        std::fs::write(&tscn_name, tscn)?;
        messages.push(format!("[trans] {} -> {}", path, tscn_name));
    }

    // 2. Functions/logic -> .gd (SceneDecl is skipped, not an error).
    let gdname = path.replace(".at", ".gd");
    let fname = AutoPath::new(path).filename();
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::gdscript::GDScriptTrans::new(fname);
    trans.trans(ast, &mut sink)?;
    std::fs::write(&gdname, sink.done()?)?;
    messages.push(format!("[trans] {} -> {}", path, gdname));

    Ok(messages.join("\n"))
}

/// Transpile AutoLang file to TypeScript (Plan 100: a2js → a2ts)
pub fn trans_typescript(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

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
        let runtime_dir = std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let runtime_path = runtime_dir.join("runtime.ts");
        std::fs::write(
            &runtime_path,
            crate::trans::typescript::ts_runtime::runtime_file_content(),
        )?;
    }

    Ok(format!("[trans] {} -> {}", path, tsname))
}

/// Transpile AutoLang file to TypeScript with custom output path
pub fn trans_typescript_to(path: &str, output: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

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
        let runtime_dir = std::path::Path::new(output)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let runtime_path = runtime_dir.join("runtime.ts");
        std::fs::write(
            &runtime_path,
            crate::trans::typescript::ts_runtime::runtime_file_content(),
        )?;
    }

    Ok(format!("[trans] {} -> {}", path, output))
}

/// Transpile Rust file to AutoLang (Plan 173: r2a)
pub fn transpile_r2a_file(path: &str) -> AutoResult<String> {
    let rust_code =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");
    crate::trans::r2a::transpile_r2a(name, &rust_code)
}
