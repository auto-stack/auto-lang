#![recursion_limit = "512"]

//! auto-lang-core — 纯语言 SDK（PLAN-814）。
//!
//! 从 auto-lang 巨型 crate 迁入的非 UI 语言栈单一来源：use/dep 扫描、
//! 解析/AST/类型、编译会话、AutoVM、多目标转译、基础服务与执行入口族。
//!
//! 边界合同（PLAN-814 §1/AC-01）：normal/build/dev 闭包不含 UI 实现
//! （ui/ui_gen 生成器、renderer、桌面宿主）与 Man；宿主能力经
//! `vm::host_capabilities` provider 端口；旧 `auto_lang::*` 公开路径由上层
//! 兼容包（auto-lang）再导出保类型身份。

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use crate::compile::CompileSession;
use crate::trans::Sink;
use crate::trans::Trans;
use auto_val::AutoPath;
use auto_val::{Array, Node, Obj, Value};

// ---- T-02 批 1-2：扫描器/前端门面组 ----
pub mod dep_scanner;
mod dependency_scan;
pub mod mode;
pub mod os_paths;
pub mod use_scanner;

pub mod ast;
pub mod error;
// PLAN-816 T-05：脚本/调试/转译文件入口族单源（自 auto-lang 兼容包迁入；
// 旧 auto_lang::* 路径经兼容包转发保持）。
pub mod script_entry;
pub use script_entry::{
    debug_file, debug_file_agent, dump_lowered, run_file, run_file_with_args, trans_auto_s2s,
    trans_gdscript, trans_godot, trans_javascript, trans_python, trans_rust_merged, trans_tscn,
    trans_typescript, trans_typescript_to, transpile_r2a_file,
};
pub mod lexer;
pub mod scope;
pub mod scope_manager;
pub mod session;
pub mod symbols;
pub mod token;

// ---- T-02 批 3+4：类型系统与 parser 骨干 ----
pub mod atom;
pub mod atom_error;
pub mod database;
pub mod dialect;
pub mod indexer;
pub mod infer;
pub mod ownership;
pub mod parser;
pub mod parser_bridge;
pub mod parser_helpers;
pub mod query;
pub mod resolver;
mod scene_grammar;
pub mod trait_checker;
pub mod type_registry;
pub mod typeck;
pub mod types;
// PLAN-814: 上层兼容门面需要跨 crate 引用（原 crate 私有模块）
pub mod ast_parse_ext;

// ---- T-02 大迁移：执行/编译/VM/trans/服务全集 ----
pub mod a2r_std;
pub mod api;
pub mod autovm_client;
pub mod autovm_daemon;
pub mod autovm_persistent;
pub mod autovm_repl;
pub mod back_prefix;
pub mod compile;
pub mod comptime;
pub mod config;
pub mod dep;
pub mod execution_engine;
pub mod ffi;
pub mod hash;
pub mod host;
pub mod http_file_service;
pub mod http_service_config;
pub mod http_service_observability;
pub mod http_upload_service;
pub mod interpreter;
pub mod libs;
pub mod macro_;
pub mod maker;
pub mod mcp;
#[cfg(feature = "mem-profile")]
pub mod mem_profile;
pub mod module_cache;
pub mod multi_mode;
pub mod patch;
#[cfg(feature = "python")]
pub mod py_ffi;
pub mod py_ffi_types;
pub mod runtime;
pub mod sse;
pub mod state_file;
pub mod stdlib_assembly;
pub mod target;
#[cfg(feature = "test-cases")]
pub mod test_cases;
pub mod test_runner;
pub mod trans;
pub mod util;
pub mod vm;

// ---- 原 auto-lang lib.rs Core 根项（T-02 拆分，行号精确切割）----
// Global tokio runtime for VM execution
// Using OnceLock to ensure thread-safe lazy initialization
static GLOBAL_RT: OnceLock<Arc<tokio::runtime::Runtime>> = OnceLock::new();

impl std::fmt::Display for StagedRunFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for StagedRunFailure {}

// Plan 773 F-08: relay for the typed trap code between the result
// extraction (deep inside the pipeline, where the task object lives) and
// the staged wrapper below. Every public execution API runs its pipeline
// on a DEDICATED thread, so the cell is call-scoped: set on the failing
/// extraction, taken by the wrapper on the same thread before join returns.
thread_local! {
    static LAST_RUN_TRAP_CODE: std::cell::RefCell<Option<&'static str>> =
        const { std::cell::RefCell::new(None) };
}

pub async fn execute_autovm(
    code: &str,
    capture: bool,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    execute_autovm_with_path(code, capture, None).await
}

/// Plan 346: Remap string pool indices in bytecode after merging string pools.
/// Walks the bytecode one instruction at a time (using a per-opcode operand
/// length table so operand bytes are never misread as opcodes) and remaps the
/// u16 string-pool index operand of every string-indexed opcode.
///
/// Plan 355: this replaces an earlier naive byte-scan that (a) only read 1 byte
/// per index and (b) could mistake operand bytes of other instructions for
/// LOAD_STR/LOAD_GLOBAL/STORE_GLOBAL, silently corrupting dep-module bytecode.
/// The corruption manifested in the sha2 library (large dep module with ~150
/// globals): random instructions got their bytes rewritten, producing wrong
/// hashes. The walker below decodes instruction lengths precisely.

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

/// Check if source code contains UI keywords (widget/app).

/// PLAN-649 SD-01 (contract Q5): candidate path forms for a dotted use
/// module — the literal dot→separator conversion first, then hyphen variants
/// of underscore segments. bp packages are kebab-case on disk
/// (`packages/blueprints/feedback/empty-state/`) while use paths are written with
/// underscores (`bps.feedback.empty_state.reference.error`); literal-first
/// keeps co-existing `foo_bar/` vs `foo-bar/` resolution deterministic.
/// Variant combinations enumerate per eligible segment in segment order and
/// cap at 4 (real bp paths carry 1-2 eligible segments; the cap only bounds

/// Transpile AutoLang file to C
///
/// **Plan 091**: Now uses CompileSession internally (no Universe dependency)

inventory::collect!(UiCompatInstaller);

pub mod heavy_gate;

// ---- 根别名（原 auto-lang 根 pub use，类型身份不变） ----
pub use crate::error::{AutoError, AutoResult};
pub use crate::parser::Parser;
pub use std::rc::Rc;

// ---- T-02 补充根项 ----
pub async fn execute_autovm_with_path(
    code: &str,
    capture: bool,
    path: Option<&str>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    execute_autovm_with_deadline_and_input(code, capture, path, None, None).await
}

pub async fn execute_autovm_with_deadline_and_input(
    code: &str,
    capture: bool,
    path: Option<&str>,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    let http_launch = crate::vm::host_capabilities::http_service::invocation_launch();
    let _request_file_root = input
        .is_some()
        .then(|| crate::vm::ffi::stdlib::RequestFileRootGuard::for_source_path(path));
    crate::install_ui_host_capabilities();
    // Plan 560 T03: 脚本模式管线激活——`.as` 源经 s2s lowering（糖→桥，
    // W2 规则表自 T05 起逐条入住）后走正常编译管线；`#[rust]` 行首
    // pragma 预检保留原源（显式压回，555 八格矩阵的压回通道）。
    // 注：pragma 预检为行首前缀启发式（#[rust] 文件级标注形态），
    // 完整解析期判定在下方 session 解析段——罕见面在案（P560 债）。
    let _lowered_owned;
    let code: &str = if path.map(|p| p.ends_with(".as")).unwrap_or(false)
        && !code.lines().any(|l| l.trim_start().starts_with("#[rust]"))
    {
        _lowered_owned = crate::trans::auto_s2s::lower_source(code)?;
        &_lowered_owned
    } else {
        code
    };
    use crate::vm::codegen::Codegen;
    use crate::vm::engine::AutoVM;
    use crate::vm::loader::Linker;
    use crate::vm::opcode::OpCode;
    use crate::vm::virt_memory::VirtualFlash;

    // Plan 442 C2: axum serve adapter — fresh process-global state for this
    // program (routes, param sigs, AppState). Must precede resolve_deps:
    // dep-module Codegen instances publish param sigs during compilation.
    crate::vm::ffi::axum_adapter::reset();
    // PLAN-669: #[api] param-sig side channel — same reset discipline.
    let http_metadata_scope =
        crate::vm::host_capabilities::http_service::CompilationMetadataGuard::begin();
    crate::vm::ffi::http_server::clear_api_param_sigs();

    // Plan 085: Pre-process use statements to load dependencies
    let mut session = compile::CompileSession::new();
    // Plan 327: seed source_dirs with the source file's directory so that
    // `use db` resolves db.at relative to the source, not the CWD.
    if let Some(p) = path {
        if let Some(dir) = std::path::Path::new(p).parent() {
            session.add_source_dir(dir.to_path_buf());
        }
    }
    // Extra module search dirs from AUTO_SOURCE_DIRS (path-list). Split-mode
    // backends (vue+vm) run api.at standalone, which only seeds the source
    // file's own directory; the caller sets this to include sibling dirs
    // (e.g. src/front so `use types` resolves front/types.at).
    if let Ok(extra) = std::env::var("AUTO_SOURCE_DIRS") {
        let sep = if cfg!(windows) { ';' } else { ':' };
        for dir in extra.split(sep) {
            let dir = dir.trim();
            if !dir.is_empty() {
                session.add_source_dir(std::path::PathBuf::from(dir));
            }
        }
    }
    session.collect_rust_imports(code)?; // Plan 212b: collect use.rs imports before resolving deps
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

    // Plan 550 T09: parser 警告通道在 CLI 直跑路径可见化（LSP 侧本就
    // 消费 parser.warnings；此前直跑不显示任何警告）。DeprecatedFeature
    // 按名一次性去重输出到 stderr——'nil' 退役提示等。仅警告不拒绝，
    // 运行行为不受影响（py parity 套件对 stdout TAP 行，stderr 不入对拍）。
    {
        let mut seen = std::collections::HashSet::new();
        for w in &parser.warnings {
            if let crate::error::Warning::DeprecatedFeature { name, message, .. } = w {
                if seen.insert(name.as_str()) {
                    eprintln!("warning: '{}' is deprecated: {}", name, message);
                }
            }
        }
    }

    // Plan 555 T02: 脚本方言模式解析（扩展名 .as ≡ 隐式 #[script] +
    // #[rust] 显式压回，八格矩阵见 mode.rs resolve_script_mode）→
    // session 回填（script_marked 保持 550 派生兼容）。W1 语义
    // passthrough——信号先行，语义激活在 W2 lowering 批。
    let script_mode = crate::mode::resolve_script_mode(
        path.and_then(|p| p.rsplit('.').next())
            .filter(|e| e.eq_ignore_ascii_case("at") || e.eq_ignore_ascii_case("as")),
        parser.script_pragma,
        parser.rust_pragma,
    );
    session.set_script_mode(script_mode);

    // Plan 550 T10 / Plan 560 T14: 生产者门控**硬化**——正常模式（.as
    // 扩展名与 #[script] pragma 豁免；#[rust] 压回）含三信号（use.py /
    // null / nil 字面量）→ **诊断错误**（编译期拒绝）。用户裁定在案
    //（2026-09-05，待澄清#5）：py 套件已全量 .as（19 套件），vm 语料
    // 走 run_with_capture 框架不经本路径（实证 tv 零撞击），CLI 直跑
    // .at 信号的存量面按设计拒绝并指引迁移。
    // 门控仅对**文件上下文**生效（设计 §2 = 文件级信号；内联/eval
    // 源码无扩展名档位，T14 实证：musk null 语义测试与 550 探针族
    // 走 run_with_capture 无 path 通道——VM 语义测试不应被文件门拦）。
    if path.is_some() && !matches!(script_mode, crate::mode::ScriptMode::Script) {
        let py_signal = !session.py_imports().is_empty();
        let null_signal = parser.saw_bare_null;
        if py_signal || null_signal {
            let mut signals = Vec::new();
            if py_signal {
                signals.push("use.py");
            }
            if null_signal {
                signals.push("null/nil 字面量");
            }
            return Err(format!(
                "auto_gate_E5501: 正常模式 .at 含脚本内容信号（{}）——                 改名为 .as 或标注 #[script]（#[rust] 可压回）；                 参见脚本模式设计 script-mode-interop.md §2",
                signals.join(" + ")
            )
            .into());
        }
    }

    // Plan 095: Run CTEE (Compile-Time Execution Engine) to transform AST
    // This handles #if, #for, #is, #{} constructs
    let mut ctee = crate::comptime::CTEE::new();
    ctee.transform(&mut ast)?;

    // 2. Compile to bytecode
    // Plan 091: Wrap script-level code with FN_PROLOG/RESERVE_STACK for proper local variable support
    // Plan 417-E4 (DIV-TRAIT-VM-2): pre-register spec decls into the shared
    // TypeStore BEFORE codegen — the script path compiles TypeDecls before
    // other statements, and default-method synthesis on implementing types
    // looks specs up via the store (order-independent).
    {
        let mut ts = parser.type_store.write().unwrap();
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::SpecDecl(sd) = stmt {
                ts.register_spec_decl(sd);
            }
        }
    }
    // Plan 123: Share TypeStore with Parser so Codegen can access registered types/enums
    let mut codegen = Codegen::new_with_type_store(parser.type_store.clone());
    codegen.assembly_context = session.assembly;
    // PLAN-013 T1: seed the root codegen with the `#[vm]` fn names declared
    // by dep modules (e.g. stdlib term.vm.at) — dep modules compile in their
    // own codegen pass, so the root would otherwise never learn that an
    // imported bare call (use auto.term: engine_spawn) targets a native.
    for name in &session.vm_fn_names {
        codegen.vm_fn_names.insert(name.clone());
    }
    // PLAN-057 T7：注入源文本——web 内建编译期门禁的 `// vm-safe-allow`
    // 行级豁免需要按 current_source_line 回读原始行。
    codegen.source_text = Some(code.to_string());
    // Separate type declarations from other statements.
    // PLAN-093 (G-7) 修正：Ext 不再与 TypeDecl 同区——ext 块（解析器已把
    // ext 合并进 `type X` 的形态除外；未合并的独立 Ext 在此）此前先于
    // other_stmts 的 Use 编码，方法体 import_scope 恒空 → 裸名 reloc
    // 链接失败。Ext 归入 other_stmts 按源序编码（use 在文件头自然先行）；
    // TypeDecl 仍最前（ext 方法依赖类型已注册）。
    let (type_decls, other_stmts): (Vec<_>, Vec<_>) = ast.stmts.iter().partition(|stmt| {
        matches!(
            stmt,
            crate::ast::Stmt::TypeDecl(_) | crate::ast::Stmt::EnumDecl(_)
        )
    });
    // PLAN-093 (G-7) 修正：Use 先于 TypeDecl 编码——解析器把 ext 合并进
    // `type X`，合并方法随 TypeDecl pass 编码；use 在 other_stmts 里会更晚
    // 处理，方法体 import_scope 恒空 → 裸名 reloc 链接失败（fixture 实证）。
    let (uses, other_stmts): (Vec<_>, Vec<_>) = other_stmts
        .into_iter()
        .partition(|stmt| matches!(stmt, crate::ast::Stmt::Use(_)));
    for stmt in &uses {
        codegen.compile_stmt(stmt)?;
    }

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

        // Plan 327: Register module-level `var x = ...` as globals — but ONLY
        // when this program has #[api] routes (HTTP server mode). In server
        // mode, #[api] handlers are separate fns that can't see the script
        // wrapper's locals, so module-level vars must be globals. In normal
        // scripts (no #[api]), top-level vars stay as script-wrapper locals
        // (the existing behavior, which many tests rely on).
        let has_api_routes = other_stmts
            .iter()
            .any(|s| matches!(s, crate::ast::Stmt::Fn(fd) if fd.api_attrs.is_some()));
        if has_api_routes {
            for stmt in other_stmts.iter() {
                if let crate::ast::Stmt::Store(store) = stmt {
                    let name = store.name.to_string();
                    codegen.global_vars.insert(name.clone());
                    codegen.global_inits.push((name, store.expr.clone()));
                }
            }
        }

        // Now compile the statements. Emit SOURCE_LINE per top-level stmt so
        // Run-mode bytecode carries source line info for linking, matching
        // the debug path (create_vm_from_source).
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
    let mut strings = codegen.strings.clone();

    let mut linker = Linker::new();
    let mut dep_modules = session.take_compiled_modules();
    // Plan 345: capture dependency module names (for __module_init lookup).
    let dep_module_names: Vec<String> = dep_modules.iter().map(|m| m.name.clone()).collect();

    // and remap string indices in their bytecode. Without this, LOAD_STR /
    // STORE_GLOBAL / LOAD_GLOBAL instructions in dep modules reference wrong
    // strings (the main pool's indices don't match), causing globals to be
    // stored under empty/wrong keys.
    let _main_string_count = strings.len();
    // Plan 346: object_keys/types declared here so the dep module merge loop
    // can append to them. Initialized from the main module's codegen.
    let mut object_keys = codegen.object_keys.clone();
    let mut object_types = codegen.object_types.clone();
    // 主池内容索引(去重 O(1);2026-08-22 池 u32 化 + O(n×m) 线性 position
    // 去重换 HashMap —— 大模块并池曾达秒级)。
    let mut main_pool_idx: std::collections::HashMap<Vec<u8>, u32> = strings
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i as u32))
        .collect();
    for module in dep_modules.iter_mut() {
        // Plan 355: remap string-pool indices. Done even if the module also has
        // object pools; the two remaps are independent.
        if !module.strings.is_empty() {
            // Build remap table: old index → new index(u32 —— 池 u32 化;
            // 此前 u16 在主池越 65535 后回绕)。
            let mut remap = vec![0u32; module.strings.len()];
            for (old_idx, s) in module.strings.iter().enumerate() {
                if let Some(&existing) = main_pool_idx.get(s) {
                    remap[old_idx] = existing;
                } else {
                    let new_idx = strings.len() as u32;
                    strings.push(s.clone());
                    main_pool_idx.insert(s.clone(), new_idx);
                    remap[old_idx] = new_idx;
                }
            }
            // Remap string indices in bytecode. We scan for opcodes that embed
            // a string pool index (LOAD_STR, STORE_GLOBAL, LOAD_GLOBAL, etc.).
            remap_string_indices(&mut module.code, &remap);
        }

        // Plan 359 B1: Merge object_keys/object_types pools AND remap every
        // CREATE_OBJ key_index (u16) operand in the dep module's bytecode.
        //
        // The dep module's CREATE_OBJ instructions hold 0-based indices into
        // their OWN object_keys pool. After we append that pool to the main
        // pool, those indices must be rebased by the current main-pool length
        // (base-offset remap: dep index i → main_base + i), otherwise they
        // point at the main module's pool entries — producing objects with the
        // wrong field names/types and, when the main pool's entries have a
        // different length, a field_count/values length mismatch that overflows
        // at runtime (engine.rs CREATE_OBJ handler).
        //
        // The earlier "CREATE_OBJ remap is intentionally not applied" comment
        // was incorrect: the indices are NOT self-adjusting. This only appeared
        // to work because most object creation goes through NEW_INSTANCE (typed
        // `Point { ... }`), which bypasses the object_keys pool entirely.
        if !module.object_keys.is_empty() {
            let obj_base = object_keys.len() as u32;
            let obj_count = module.object_keys.len();
            for (i, keys) in module.object_keys.iter().enumerate() {
                object_keys.push(keys.clone());
                // Keep object_types aligned with object_keys entry-for-entry.
                // If a dep module ever lacks a matching types entry, push an
                // empty vec so the two pools never desync.
                if let Some(ty) = module.object_types.get(i) {
                    object_types.push(ty.clone());
                } else {
                    object_types.push(Vec::new());
                }
            }
            // Base-offset remap: dep index i → obj_base + i.
            let obj_remap: Vec<u32> = (0..obj_count)
                .map(|i| obj_base.saturating_add(i as u32))
                .collect();
            remap_obj_indices(&mut module.code, &obj_remap);
        }
    }

    for module in dep_modules {
        linker.add_module(module);
    }

    // Plan 346: object_keys/types already merged in the dep module loop above.
    let result_type = codegen.last_expr_type.clone();
    // Plan 197 Task 9: Extract generic registry before finish() consumes the codegen
    let mut generic_registry = std::mem::take(&mut codegen.generic_registry);
    generic_registry.merge(&session.dep_generic_registry.borrow());
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
    vm_debug!(
        "DEBUG: Main module exports: {:?}",
        main_module.exports.keys().collect::<Vec<_>>()
    );
    linker.add_entry_module(main_module);

    let (linked_code, global_symbols) = linker.link().map_err(|e| {
        let span = if let Some(pos) = e.source_pos {
            crate::error::pos_to_span(pos)
        } else {
            find_use_symbol_span(code, &e.message)
        };
        let help = if e.source_pos.is_some() {
            Some(format!(
                "Use a `use` statement to import '{}' from a module, or check for typos",
                e.symbol
            ))
        } else {
            extract_undefined_symbol(&e.message)
                .map(|s| format!("Check if '{}' is defined and exported in the module", s))
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
    let main_entry = if let Some(&addr) = global_symbols
        .get("main")
        .or_else(|| global_symbols.get("test"))
    {
        addr as usize
    } else {
        // No main/test function — start from beginning of main module's code
        // (which is after all dependency modules)
        let dep_size: usize = linker
            .modules
            .iter()
            .take(linker.modules.len() - 1)
            .map(|m| m.code.len())
            .sum();
        dep_size
    };

    vm_debug!("DEBUG: Linked code size: {} bytes", linked_code.len());
    vm_debug!("DEBUG: Global symbols: {:?}", global_symbols);

    // Plan 346: Capture dep module START addresses (module top-level code).
    // After reverting __module_init, each dep module's var initializers are
    // at the module's code start (before function definitions). We spawn a
    // task at each module's start offset to run the initializers.
    let module_offsets: Vec<u32> = {
        let mut offsets = Vec::new();
        let mut cur = 0u32;
        for m in &linker.modules {
            offsets.push(cur);
            cur += m.code.len() as u32;
        }
        offsets
    };
    // dep modules are all except the last (main) module.
    let module_init_addrs: Vec<(String, usize)> = dep_module_names
        .iter()
        .enumerate()
        .filter_map(|(i, name)| {
            module_offsets
                .get(i)
                .map(|&offset| (name.clone(), offset as usize))
        })
        .collect();

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
        let (vm, buf) = AutoVM::new_with_capture(flash, 8192);
        (vm, Some(buf))
    } else {
        let vm = AutoVM::new(flash, 8192);
        (vm, None)
    };
    // PLAN-746 (PG-MEM-1): cooperative execution deadline (None = unlimited).
    vm.set_deadline(deadline);
    vm.set_request_input(input);
    vm.load_strings(strings);
    vm.load_generic_registry(generic_registry);
    vm.load_task_handler_registry(task_handler_registry); // Plan 327 Phase 1
                                                          // Plan 312: Register #[api] routes for HTTP server dispatch
    crate::vm::host_capabilities::http_service::VmHttpBridge::bind_launch(&mut vm, http_launch);
    crate::vm::host_capabilities::http_service::VmHttpBridge::bind_program_metadata(
        &mut vm,
        http_metadata_scope.finish(&api_routes),
    );
    crate::vm::ffi::stdlib::register_http_routes(api_routes);

    // Capture disassembly before execution mutates runtime state.
    let bytecode_lines = {
        let disasm = crate::vm::disasm::Disassembler::new(&vm.flash);
        disasm.disassemble_range(0, vm.flash.memory.len())
    };

    // PLAN-738 生产合同：标准 shims/stdlib FFI/inventory override 已由
    // `AutoVM::new` 内的 `NativeInterface::production()` + CFFI merge 完成
    // （R2 收口）。此处不再重注册——`register_stdlib_ffi` 的无契约重注册
    // 会按 merge 撤销语义覆盖 production() 声明的核心契约（cb_web_mime
    // 实证：http.get #2230 契约被本块撤销 → SIGNATURE_UNVERIFIED 假红）。
    // rebase 组合恢复的 master 旧块于 merge 修复轮移除。

    // Plan 212b Task 4: Merge Rust FFI native interface into VM
    if let Some(rust_ni) = rust_ffi_native_interface {
        vm.merge_native_interface(&rust_ni);
    }

    // Plan 214: Merge Python FFI native interface into VM
    if let Some(py_ni) = py_ffi_native_interface {
        vm.merge_native_interface(&py_ni);
    }

    // Symbol tables for disassembly tooltips (strings/functions/natives).
    // Collected after all native merges, before execution mutates state.
    let bytecode_meta = crate::vm::disasm::BytecodeMeta::of_vm(&vm);

    // Helper to extract stdout from capture buffer
    let get_stdout = || {
        output_buffer
            .as_ref()
            .map(|buf| buf.read().unwrap().clone())
            .unwrap_or_default()
    };

    // Plan 118: Store the codegen's result type for formatting

    // Plan 345: Execute dependency module initializers.
    // Each dep module's top-level var declarations are at the module's code
    // start (before function definitions). Spawning a task at the module's
    // start offset runs the initializers, populating vm.globals.
    for (_mod_name, init_addr) in &module_init_addrs {
        let init_task_id = vm.spawn_task(*init_addr, 65536);
        vm.run_task_loop().await;
        // Plan 419: init 任务栈无人再读 —— 释放后移除。
        if let Some(arc) = vm.tasks.get(&init_task_id) {
            let mut t = arc.lock().await;
            vm.rc_release_task_stack(&mut t);
        }
        vm.remove_task(init_task_id);
    }

    // 5. Execute - Find main/test entry point
    let task_id = vm.spawn_task(main_entry, 65536);
    vm.run_task_loop().await;

    // 6. Get result from stack
    let result = extract_autovm_result(&vm, task_id, Some(result_type)).await?;

    // Explicit host activation. Ordinary language calls do not require Web;
    // a service needs an installed provider and keeps its !Send VM local.
    let routes = crate::vm::host_capabilities::http_service::VmHttpBridge::program_metadata(&vm)
        .map(|p| {
            p.routes
                .iter()
                .map(|r| (r.method.clone(), r.path.clone(), r.fn_name.clone()))
                .collect()
        })
        .unwrap_or_else(crate::vm::ffi::stdlib::get_http_routes);
    if !routes.is_empty() {
        crate::vm::host_capabilities::http_service::serve_registered(vm, routes).await?;
    }

    Ok((result, get_stdout(), bytecode_lines, bytecode_meta))
}

/// Extract and format the result value from a VM task after execution.
/// Plan 225: Shared between execute_autovm and debug sessions.
pub async fn extract_autovm_result(
    vm: &crate::vm::engine::AutoVM,
    task_id: u64,
    result_type: Option<crate::vm::codegen::ObjectType>,
) -> AutoResult<String> {
    use crate::vm::codegen::ObjectType;
    use crate::vm::task::ResultType;

    // Plan 773 F-08: start each extraction with a clean trap relay.
    LAST_RUN_TRAP_CODE.with(|c| *c.borrow_mut() = None);

    let task_arc = vm
        .tasks
        .get(&task_id)
        .map(|r| r.value().clone())
        .ok_or_else(|| {
            crate::error::AutoError::Msg("Task not found after execution".to_string())
        })?;
    let mut task = task_arc.lock().await;

    // Check if task had an error
    if let Some(error) = task.last_error.clone() {
        // Plan 419: 出错任务的栈不再被读取 —— 整栈释放(含 state_vars)。
        vm.rc_release_task_stack(&mut task);
        // Plan 773 F-08: relay the typed trap identity for the staged API.
        LAST_RUN_TRAP_CODE.with(|c| *c.borrow_mut() = task.last_error_code);
        return Err(crate::error::AutoError::Msg(error));
    }

    if task.ram.sp == 0 {
        return Ok("".to_string());
    }

    // Check VM runtime result type first (set during execution)
    // Plan 437: last_result_type 是粘性运行时状态（如刚向 List<float> push
    // 过 float 参数后未复位）——仅当栈顶确为 float（sp-2 处 f64 或 sp-1 处
    // f32 tag）时才走 Float 格式化；其余 tag（i32/bool/string 负哨兵/对象）
    // 全部落回下方通用路径按 tag 解码。此前的无条件 pop_f32 会把任意值按
    // f32 位重解释（i32 2 → "0.000...003"、bool → "-0"、字符串哨兵 → "-2"）。
    let top_is_f32 = task.ram.sp >= 1 && auto_val::is_f32(task.ram.raw_nv[task.ram.sp - 1]);
    let top2_is_f64 = task.ram.sp >= 2 && auto_val::is_f64(task.ram.raw_nv[task.ram.sp - 2]);
    let result = match task.last_result_type {
        ResultType::Float if top_is_f32 || top2_is_f64 => {
            {
                // f64 may be on stack (2 slots) even when last_result_type is Float.
                // This happens when FFI returns f64 but codegen marked the expression as Float.
                // Format as f32 to match expected behavior (which truncates via pop_f32).
                if top2_is_f64 {
                    let nv = task.ram.raw_nv[task.ram.sp - 2];
                    task.ram.sp -= 2;
                    let result = f64::from_bits(nv) as f32;
                    format!("{}", result)
                } else {
                    let result = task.ram.pop_f32();
                    format!("{}", result)
                }
            }
        }
        _ => {
            // Then check codegen's compile-time result type if provided
            match result_type {
                Some(ObjectType::Float) | Some(ObjectType::Double) => {
                    {
                        // f64 occupies 2 slots (value at sp-2, marker at sp-1).
                        // Check sp-2 for f64 bits first, then sp-1 for f32.
                        if task.ram.sp >= 2 {
                            let nv = task.ram.raw_nv[task.ram.sp - 2];
                            if auto_val::is_f64(nv) {
                                task.ram.sp -= 2; // consume both slots
                                let result = f64::from_bits(nv);
                                format!("{}", result)
                            } else {
                                let result = task.ram.pop_i32();
                                format!("{}", result)
                            }
                        } else {
                            let nv = task.ram.raw_nv[task.ram.sp - 1];
                            if auto_val::is_f32(nv) {
                                let result = task.ram.pop_f32();
                                format!("{}", result)
                            } else {
                                let result = task.ram.pop_i32();
                                format!("{}", result)
                            }
                        }
                    }
                }
                Some(ObjectType::Byte) => {
                    let result = task.ram.pop_i32();
                    format!("0x{:02X}", result as u8)
                }
                Some(ObjectType::Uint) => {
                    let result = task.ram.pop_i32();
                    format!("{}u", result as u32)
                }
                Some(ObjectType::Char) => {
                    let result = task.ram.pop_i32();
                    if let Some(ch) = char::from_u32(result as u32) {
                        format!("'{}'", ch)
                    } else {
                        format!("{}", result)
                    }
                }
                Some(ObjectType::Bool) => {
                    let result = task.ram.pop_i32();
                    if result != 0 {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                Some(ObjectType::Void) => {
                    let _ = task.ram.pop_i32();
                    "".to_string()
                }
                _ => {
                    // Default: pop and format based on value
                    let result = task.ram.pop_i32();
                    let result_u64 = result as u64;

                    // Check heap objects (arrays are ListData<Value> here too,
                    // Plan 390 §15 H3b)
                    if let Some(obj_arc) = vm.heap_objects.get(&result_u64) {
                        let obj = obj_arc.read().unwrap();
                        if let Some(list) = obj
                            .as_any()
                            .downcast_ref::<crate::vm::types::ListData<i32>>()
                        {
                            let formatted: Vec<String> =
                                list.elems.iter().map(|e| e.to_string()).collect();
                            format!("[{}]", formatted.join(", "))
                        } else if let Some(list) = obj
                            .as_any()
                            .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                        {
                            let strings = vm.strings.read().unwrap();
                            let formatted: Vec<String> = list
                                .elems
                                .iter()
                                .map(|v| {
                                    if let auto_val::Value::Int(bits) = v {
                                        if *bits < 0
                                            && *bits > -1000000
                                            && *bits != -2147483648
                                            && *bits != -2147483647
                                        {
                                            let str_idx = (-bits - 1) as usize;
                                            if let Some(bytes) = strings.get(str_idx) {
                                                return format!(
                                                    "\"{}\"",
                                                    String::from_utf8_lossy(bytes)
                                                );
                                            }
                                        }
                                    }
                                    v.repr().to_string()
                                })
                                .collect();
                            format!("[{}]", formatted.join(", "))
                        } else if let Some(sb) =
                            obj.as_any()
                                .downcast_ref::<crate::vm::collections::SpecializedStringBuilder>()
                        {
                            sb.buffer.clone()
                        } else if let Some(od) =
                            obj.as_any().downcast_ref::<crate::vm::types::ObjectData>()
                        {
                            // Plan 390 §15 H3b: obj literals are ObjectData in
                            // heap_objects.
                            let mut fields: Vec<(&auto_val::ValueKey, &Value)> =
                                od.fields.iter().collect();
                            fields.sort_by(|(k1, _), (k2, _)| k1.to_string().cmp(&k2.to_string()));
                            let formatted: Vec<String> = fields
                                .iter()
                                .map(|(k, v)| {
                                    let key_str = k.to_string();
                                    let val_str = format_value_for_display(vm, v);
                                    format!("{}: {}", key_str, val_str)
                                })
                                .collect();
                            format!("{{{}}}", formatted.join(", "))
                        } else {
                            format!("{}", result)
                        }
                    }
                    // Check strings pool
                    else if result < 0
                        && result > -1000000
                        && result != -2147483648
                        && result != -2147483647
                    {
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
                    } else {
                        format!("{}", result)
                    }
                }
            }
        }
    };

    // Plan 419: 结果已提取 —— 释放该任务残余栈(含 state_vars;结果若为
    // 堆 id,格式化已完成,其 stake 随整栈释放)。
    vm.rc_release_task_stack(&mut task);

    Ok(result)
}

// run_with_errors() removed in Plan 091 - use run() with built-in error recovery

// ---- T-02 终批补充 ----
pub(crate) const AUTO_LIB_FILES: &[&str] = &[
    // plan-431 E1:AAVM v1 归档路径(v2 由 432 写入 packages/aavm/lib/,runner 见 AUTO_LIB_FILES_V2)
    "packages/aavm/lib-legacy/pos.at",
    "packages/aavm/lib-legacy/token.at",
    "packages/aavm/lib-legacy/error.at",
    "packages/aavm/lib-legacy/lexer.at",
    "packages/aavm/lib-legacy/ast.at",
    "packages/aavm/lib-legacy/parser.at",
    "packages/aavm/lib-legacy/typeinfer.at",
    "packages/aavm/lib-legacy/codegen.at",
    "packages/aavm/lib-legacy/vm.at",
    "packages/aavm/lib-legacy/a2r.at",
    "packages/aavm/lib-legacy/generics.at",
    "packages/aavm/lib-legacy/eval.at",
];

// ---- T-02 补：ABT 运行入口（vm/abt 测试支持）----
/// Plan 226 Phase 6: Run ABT text directly and return captured stdout.
///
/// Convenience wrapper around `create_vm_from_abt` + spawn + run loop.
pub async fn run_abt(abt_text: &str) -> AutoResult<String> {
    let (vm, output_buffer, main_entry) = create_vm_from_abt(abt_text)?;

    let _task_id = vm.spawn_task(main_entry, 16384);
    vm.run_task_loop().await;

    let result = output_buffer.read().unwrap().clone();
    Ok(result)
}

// ---- PLAN-814 T-03：store 扫描辅助下沉（原 auto-lang lib.rs 根项）----

// Plan 351: thread-local for passing store composable files out-of-band
// (avoids changing the return type of ui_build_* functions). PLAN-814 T-03：
// 单源在此（UI 编译器 api.rs 扫描消费）；auto-lang 旧路径经再导出保持。
thread_local! {
    pub static STORE_EXTRA_FILES: std::cell::RefCell<Vec<(String, String)>> =
        std::cell::RefCell::new(Vec::new());
}

// PLAN-664 U-4：use 导入两类静默死面的编译期诊断汇聚点（P-15 模块解析
// 失败 / P-16 导入符号与 model 字段撞名）。双出口：stderr 即时可见，
// thread_local 汇聚供测试断言。PLAN-814 T-04：随 UI 动态构建管线迁入
// auto-ui-runtime 而下沉 SDK 单源（与 STORE_EXTRA_FILES 同族带外通道）；
// auto-lang 旧路径经再导出保持。
thread_local! {
    static USE_DIAGS: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
}

// ============================================================================
// Plan 260/327/746：#[test] 执行入口族（PLAN-814 T-06 自 auto-lang 上层
// 下沉——纯 Core 执行面；宿主装配经本 crate 钩子分发器（无 UI 实现时
// no-op），auto-lang 旧路径经再导出保持。
// ============================================================================
/// Plan 260: Compile and run all `#[test]` functions in the given code.
/// Returns a TestResult with pass/fail details for each discovered test.
pub async fn test_code(code: &str) -> AutoResult<test_runner::TestResult> {
    test_code_with_options(code, None, None, None).await
}

/// Run all `#[test]` functions with a cooperative deadline and request-local
/// stdin. The existing `test_code` keeps its unlimited, process-stdin behavior.
pub async fn test_code_with_deadline_and_input(
    code: &str,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> AutoResult<test_runner::TestResult> {
    test_code_with_options(code, None, deadline, input).await
}

/// Path-aware test execution for isolated files projects.
pub async fn test_code_with_path_deadline_and_input(
    code: &str,
    path: &str,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> AutoResult<test_runner::TestResult> {
    test_code_with_options(code, Some(path), deadline, input).await
}

async fn test_code_with_options(
    code: &str,
    path: Option<&str>,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> AutoResult<test_runner::TestResult> {
    let http_launch = crate::vm::host_capabilities::http_service::invocation_launch();
    let _request_file_root = input
        .is_some()
        .then(|| crate::vm::ffi::stdlib::RequestFileRootGuard::for_source_path(path));
    crate::install_ui_host_capabilities();
    use crate::vm::codegen::Codegen;
    use crate::vm::engine::AutoVM;
    use crate::vm::loader::Linker;
    use crate::vm::opcode::OpCode;
    use crate::vm::virt_memory::VirtualFlash;
    use std::time::Instant;

    // Plan 442 C2: axum serve adapter state reset (see execute_autovm_with_path).
    crate::vm::ffi::axum_adapter::reset();
    // PLAN-669: #[api] param-sig side channel — same reset discipline.
    let http_metadata_scope =
        crate::vm::host_capabilities::http_service::CompilationMetadataGuard::begin();
    crate::vm::ffi::http_server::clear_api_param_sigs();

    // Compilation pipeline (same as execute_autovm)
    let mut session = compile::CompileSession::new();
    if let Some(path) = path {
        if let Some(dir) = std::path::Path::new(path).parent() {
            session.add_source_dir(dir.to_path_buf());
        }
    }
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

    // Collect tests from AST before compilation
    let registry = test_runner::collect_tests(&ast.stmts, "");

    if registry.tests.is_empty() {
        return Ok(test_runner::TestResult::default());
    }

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
        for stmt in other_stmts.iter() {
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

    let (linked_code, global_symbols) = linker
        .link()
        .map_err(|e| crate::error::AutoError::Msg(e.message.clone()))?;

    // Create VM with capture
    let flash = VirtualFlash::from_vec_with_metadata(
        linked_code,
        global_symbols.clone(),
        object_keys,
        object_types,
    );
    let (mut vm, _output_buffer) = AutoVM::new_with_capture(flash, 8192);
    vm.set_deadline(deadline);
    vm.set_request_input(input);
    vm.load_strings(strings);
    vm.load_generic_registry(generic_registry);
    vm.load_task_handler_registry(task_handler_registry); // Plan 327 Phase 1
                                                          // Plan 312: Register #[api] routes for HTTP server dispatch
    crate::vm::host_capabilities::http_service::VmHttpBridge::bind_launch(&mut vm, http_launch);
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

    // Run each test
    let mut result = test_runner::TestResult::default();
    let _start = Instant::now();

    for test in &registry.tests {
        let report = test_runner::run_test_in_vm(&vm, test, &global_symbols).await;
        result.reports.push(report);
    }

    Ok(result)
}

// ---- PLAN-843 结构拆分：入口/旗标分离（机制在子模块，crate 根保再导出） ----
mod entry;
pub use entry::*;
mod flags;
pub use flags::*;
