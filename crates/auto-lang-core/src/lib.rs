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
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use crate::compile::CompileSession;
use crate::trans::Sink;
use crate::trans::Trans;
use auto_val::AutoPath;
use auto_val::{Array, Node, Obj, Value};

// ---- T-02 批 1-2：扫描器/前端门面组 ----
pub mod dep_scanner;
pub mod mode;
pub mod os_paths;
pub mod use_scanner;

pub mod ast;
pub mod error;
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
pub mod implicit_union;
pub mod indexer;
pub mod infer;
pub mod ownership;
pub mod parser;
pub mod parser_bridge;
pub mod parser_helpers;
pub mod query;
pub mod resolver;
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
pub mod design_tokens;
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

pub fn get_global_runtime() -> Arc<tokio::runtime::Runtime> {
    GLOBAL_RT
        .get_or_init(|| {
            Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(1)
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime"),
            )
        })
        .clone()
}

/// Drive AutoVM work inside a thread-local executor on the calling thread.
/// The HTTP server uses `spawn_local` to retain an `Rc<AutoVM>` without making
/// the `!Send` VM cross an OS-thread boundary.
pub fn block_on_autovm_local<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    let runtime = get_global_runtime();
    tokio::task::LocalSet::new().block_on(&runtime, future)
}

/// Global error limit for parser error recovery
static ERROR_LIMIT: AtomicUsize = AtomicUsize::new(20);

/// Global VM debug logging flag
static VM_DEBUG: AtomicBool = AtomicBool::new(false);

/// Plan 524: CLI 透传参数（`auto <file> [args...]` 直跑形态中 file 之后的
/// 位置参数）。`process.args()` native 读此表拼真实 argv = [程序路径] +
/// 透传参数；`run_file_with_args` 写入。进程级全局（argv 本就是进程概念，
/// 与 VM_DEBUG 同款）。
static SCRIPT_ARGS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// Set the CLI pass-through script args (consumed by `process.args()`).
pub fn set_script_args(args: Vec<String>) {
    match SCRIPT_ARGS.lock() {
        Ok(mut g) => *g = args,
        Err(poisoned) => {
            *poisoned.into_inner() = args;
        }
    }
}

/// Current CLI pass-through script args (empty = run without passthrough).
pub fn script_args() -> Vec<String> {
    match SCRIPT_ARGS.lock() {
        Ok(g) => g.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

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
    // Plan 420: AUTO_VM_DEBUG=1 env as a process-wide opt-in (no CLI flag
    // existed; useful for diagnosing handler aborts in VM-mode UI apps).
    static ENV_DEBUG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    VM_DEBUG.load(Ordering::SeqCst)
        || *ENV_DEBUG.get_or_init(|| {
            std::env::var("AUTO_VM_DEBUG")
                .map(|v| v != "0" && !v.is_empty())
                .unwrap_or(false)
        })
}

/// Plan 423 P5 续修(诊断设施):AUTO_VM_TRACE_OPS=1 —— 每指令 trace
/// (ip/opcode/sp/bp)。与 AUTO_VM_DEBUG(点状埋点)互补,定位 RC 计数
/// 漂移需要完整指令流。OnceLock 缓存,未启用时零成本。
pub fn is_vm_trace_ops() -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENV.get_or_init(|| {
        std::env::var("AUTO_VM_TRACE_OPS")
            .map(|v| v != "0" && !v.is_empty())
            .unwrap_or(false)
    })
}

/// PLAN-026 needs_fix(用户实机反馈):AUTO_VM_TRACE=1 — 热路径成功日志
/// 门控([UI_EVENT]/[VM_HANDLER_CALL]/[VM_EXEC]/[VM_HANDLER_OK])。
/// 挂起根修后 tick 0.7/s→20/s,这四类无条件 eprintln 被放大 ~30 倍
/// (实测 106 行/s ≈ 5.4KB/s 持续写盘,空闲期磁盘噪声;读侧为零)。
/// 缺省静默;失败路径([VM-HANDLER] failed/[CALL_HANDLER_ERR])与
/// 预算超阈 warn 不受本门控(错误面恒出)。
pub fn is_vm_hot_trace() -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENV.get_or_init(|| {
        std::env::var("AUTO_VM_TRACE")
            .map(|v| v != "0" && !v.is_empty())
            .unwrap_or(false)
    })
}

/// Plan 423 P5 续修(诊断设施):P419_TRACE_POOL=<idx> —— 只 trace 指定
/// 字符串池索引的 retain/release/free(定位悬垂池索引的生死链)。
/// 返回 None 表示未启用。
pub fn pool_trace_idx() -> Option<usize> {
    static ENV: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    *ENV.get_or_init(|| {
        std::env::var("P419_TRACE_POOL")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
    })
}

/// Plan 432 D26 临时插桩(修复后移除):P419_POOL_LOG=1 —— 全池事件日志。
/// 每事件带全局序号 + 池索引 + rc 变化 + 内容摘要,intern 路径可见。
pub fn pool_log_all() -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENV.get_or_init(|| {
        std::env::var("P419_POOL_LOG")
            .map(|v| v != "0" && !v.is_empty())
            .unwrap_or(false)
    })
}

pub fn pool_log_seq() -> u64 {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Plan 510 G3:over-release 审计门(P510_AUDIT=1)——首次多扣款
/// (release 时 rc==0)打印双栈,把"哪条路径多扣"从推断变实锤。
pub fn p510_audit() -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENV.get_or_init(|| {
        std::env::var("P510_AUDIT")
            .map(|v| v != "0" && !v.is_empty())
            .unwrap_or(false)
    })
}

/// Debug logging macro - only prints when VM debug mode is enabled
macro_rules! vm_debug {
    ($($arg:tt)*) => {
        if is_vm_debug() {
            eprintln!($($arg)*);
        }
    };
}

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

/// P574 T6:VM 执行线程栈。历史硬编码 4MB 已被 516KB aavm lib 拼合源的
/// 解释栈需求越过(探针定量:4MB 爆/5MB 过,递归有限);且显式 stack_size
/// 会绕过 RUST_MIN_STACK——Plan 423 抬测试线程栈护栏的本意对此失效。
/// 改为 RUST_MIN_STACK 可覆盖(字节),缺省 16MB,下限保底 4MB。
pub fn vm_thread_stack_size() -> usize {
    std::env::var("RUST_MIN_STACK")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n >= 4 * 1024 * 1024)
        .unwrap_or(16 * 1024 * 1024)
}

/// Plan 773 F-08: run failure observed at the production stage boundary,
/// with the typed VM trap code attached when the failure IS a run-stage trap
/// (decided where the typed `VMError` still exists). `run_trap_code: None`
/// means the failure did not come from a VM trap (I/O, missing files,
/// synthetic deadlines, panics) — the test-case adapters treat exactly
/// those as infrastructure ERRORs, never as language negatives.
#[derive(Debug)]
pub struct StagedRunFailure {
    pub error: crate::error::AutoError,
    /// `VMError` variant name from the engine's run-task loop, if any.
    pub run_trap_code: Option<&'static str>,
}

impl std::fmt::Display for StagedRunFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for StagedRunFailure {}

/// Plan 773 F-08: relay for the typed trap code between the result
/// extraction (deep inside the pipeline, where the task object lives) and
/// the staged wrapper below. Every public execution API runs its pipeline
/// on a DEDICATED thread, so the cell is call-scoped: set on the failing
/// extraction, taken by the wrapper on the same thread before join returns.
thread_local! {
    static LAST_RUN_TRAP_CODE: std::cell::RefCell<Option<&'static str>> =
        const { std::cell::RefCell::new(None) };
}

/// Plan 773 F-08: staged twin of [`run_with_capture_and_path`] — identical
/// execution, but a failing run carries the typed trap code so the run-stage
/// oracle channel is type-derived instead of message-text-parsed. The old
/// API keeps its exact signature and behavior.
pub fn run_with_capture_and_path_staged(
    code: &str,
    path: &str,
) -> Result<(String, String), StagedRunFailure> {
    let code = code.to_string();
    let path = path.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(vm_thread_stack_size())
        .spawn(move || {
            let r = block_on_autovm_local(async {
                execute_autovm_with_path(&code, true, Some(&path))
                    .await
                    .map(|(r, stdout, _, _)| (r, stdout))
            });
            let trap = LAST_RUN_TRAP_CODE.with(|c| c.borrow_mut().take());
            r.map_err(|error| StagedRunFailure {
                error,
                run_trap_code: trap,
            })
        })
        .expect("Failed to spawn execution thread");
    match handle.join() {
        Ok(r) => r,
        Err(_) => Err(StagedRunFailure {
            error: crate::error::AutoError::Msg("execution thread panicked".to_string()),
            run_trap_code: None,
        }),
    }
}

/// Run AutoLang code with stdout capture, using the source file path for
/// module resolution (so `use db` resolves `db.at` next to the source file).
pub fn run_with_capture_and_path(code: &str, path: &str) -> AutoResult<(String, String)> {
    run_with_capture_and_path_staged(code, path).map_err(|f| f.error)
}

/// Run AutoLang code with stdout capture and return the disassembled bytecode.
pub fn run_with_capture_and_bytecode(
    code: &str,
) -> AutoResult<(String, String, Vec<crate::vm::disasm::DisasmLine>)> {
    run_with_capture_and_bytecode_with_meta(code).map(|(r, out, bc, _)| (r, out, bc))
}

/// Like [`run_with_capture_and_bytecode`], also returning the symbol tables
/// (strings/functions/natives) for resolving disassembly operand references.
pub fn run_with_capture_and_bytecode_with_meta(
    code: &str,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    run_with_capture_and_bytecode_with_deadline(code, None)
}

/// PLAN-746 (PG-MEM-1): like [`run_with_capture_and_bytecode_with_meta`] but
/// installs a cooperative execution deadline. Playground /api/run uses this
/// to bound runaway programs; `None` keeps unlimited execution.
pub fn run_with_capture_and_bytecode_with_deadline(
    code: &str,
    deadline: Option<std::time::Instant>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    run_with_capture_and_bytecode_with_input(code, deadline, None)
}

/// Like [`run_with_capture_and_bytecode_with_deadline`] with request-local
/// stdin. `Some("")` means immediate EOF; `None` preserves process stdin.
pub fn run_with_capture_and_bytecode_with_input(
    code: &str,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    let code = code.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(vm_thread_stack_size())
        .spawn(move || {
            block_on_autovm_local(async {
                execute_autovm_with_deadline_and_input(&code, true, None, deadline, input).await
            })
        })
        .expect("Failed to spawn execution thread");
    join_execution(handle)
}

/// Run AutoLang code with stdout capture and source path, returning bytecode.
pub fn run_with_capture_and_path_and_bytecode(
    code: &str,
    path: &str,
) -> AutoResult<(String, String, Vec<crate::vm::disasm::DisasmLine>)> {
    run_with_capture_and_path_and_bytecode_with_meta(code, path).map(|(r, out, bc, _)| (r, out, bc))
}

/// Like [`run_with_capture_and_path_and_bytecode`], also returning [`crate::vm::disasm::BytecodeMeta`].
pub fn run_with_capture_and_path_and_bytecode_with_meta(
    code: &str,
    path: &str,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    run_with_capture_and_path_and_bytecode_with_deadline(code, path, None)
}

/// PLAN-746 (PG-MEM-1): path variant of [`run_with_capture_and_bytecode_with_deadline`]
/// (playground project runs — `use db` resolution needs the source path).
pub fn run_with_capture_and_path_and_bytecode_with_deadline(
    code: &str,
    path: &str,
    deadline: Option<std::time::Instant>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    run_with_capture_and_path_and_bytecode_with_input(code, path, deadline, None)
}

/// Path-aware execution with a deadline and request-local stdin.
pub fn run_with_capture_and_path_and_bytecode_with_input(
    code: &str,
    path: &str,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    let code = code.to_string();
    let path = path.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(vm_thread_stack_size())
        .spawn(move || {
            block_on_autovm_local(async {
                execute_autovm_with_deadline_and_input(&code, true, Some(&path), deadline, input)
                    .await
            })
        })
        .expect("Failed to spawn execution thread");
    join_execution(handle)
}

/// PLAN-746 (PG-MEM-1 / SD-04): join an execution thread without panicking.
/// A panic inside the VM execution thread (e.g. a parser bug on hostile
/// input) surfaces as a clean `Err` instead of an `unwrap` panic bubbling
/// up through the caller (previously axum turned that into a bare 500).
pub fn join_execution<T: Send + 'static>(
    handle: std::thread::JoinHandle<AutoResult<T>>,
) -> AutoResult<T> {
    match handle.join() {
        Ok(r) => r,
        Err(_) => Err(crate::error::AutoError::Msg(
            "execution thread panicked".to_string(),
        )),
    }
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
        .stack_size(vm_thread_stack_size())
        .spawn(move || {
            block_on_autovm_local(async {
                execute_autovm(&code, false).await.map(|(r, _, _, _)| r)
            })
        })
        .expect("Failed to spawn execution thread");
    join_execution(handle)
}

/// Plan 177: Run AutoVM with stdout capture for testing
pub fn run_autovm_capture(code: &str) -> AutoResult<(String, String)> {
    let code = code.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(vm_thread_stack_size())
        .spawn(move || {
            block_on_autovm_local(async {
                execute_autovm(&code, true)
                    .await
                    .map(|(r, stdout, _, _)| (r, stdout))
            })
        })
        .expect("Failed to spawn execution thread");
    join_execution(handle)
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

/// Check if source code contains UI keywords (widget/app).

static EXTERNAL_BACK_ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// 设置外部后端根(宿主编排层在 run_file 前调用;路径须为绝对路径)。
pub fn set_external_back_root(root: std::path::PathBuf) {
    let _ = EXTERNAL_BACK_ROOT.set(root);
}

/// PLAN-635 D4: does this pac.at text declare a dependency with the given
/// name? Matches both `dep "name" { ... }` and `dep name { ... }` forms with
/// a word-boundary check so `dep settings` does not match `dep settings_v2`.
pub fn pac_declares_dep(content: &str, dep_name: &str) -> bool {
    for form in [format!("dep \"{}\"", dep_name), format!("dep {}", dep_name)] {
        if let Some(pos) = content.find(&form) {
            let after = content[pos + form.len()..].chars().next();
            // The name must end at a non-identifier char (whitespace, `{`, `,`, `:`).
            let boundary = after.is_some_and(|c| !(c.is_alphanumeric() || c == '_' || c == '.'));
            if boundary {
                return true;
            }
        }
    }
    false
}

/// PLAN-635 D5: if this pac.at declares a workspace whose members include a
/// directory matching `dep_name` (member entry's last path segment), return
/// that entry's relative path. Members syntax: `members: ["common", "apps/x"]`
/// (single- or multi-line).
pub fn pac_workspace_member_dir(content: &str, dep_name: &str) -> Option<String> {
    let members_pos = content.find("members")?;
    let list_start = content[members_pos..].find('[')? + members_pos;
    let list_end = content.find(']').filter(|e| *e > list_start)?;
    let list = &content[list_start + 1..list_end];
    for entry in list.split(',') {
        let entry = entry.trim().trim_matches(|c| c == '"' || c == '\'');
        if entry.is_empty() {
            continue;
        }
        let last = entry.rsplit(['/']).next().unwrap_or(entry);
        if last == dep_name {
            return Some(entry.to_string());
        }
    }
    None
}

/// PLAN-647 guardrail B (contract Q5): version-plane keys inside a pac.at
/// `dep` declaration are a violation. The version plane is not enabled —
/// blueprints are a single rolling source and cross-repo alignment is
/// git-level — so a `version:`/`pin:`/… key would read like a constraint
/// while being silently ignored. Returns the offending message (points at
/// contract Q5) or None when the declaration is clean.
pub fn pac_dep_version_violation(content: &str, dep_name: &str) -> Option<String> {
    const VERSION_KEYS: [&str; 6] = ["version", "pin", "rev", "tag", "branch", "commit"];
    for form in [format!("dep \"{dep_name}\""), format!("dep {dep_name}")] {
        let Some(pos) = content.find(&form) else {
            continue;
        };
        // Same name-boundary discipline as pac_declares_dep: the matched name
        // must end at a non-identifier char (`bpsx` must not hit `dep bps`).
        let after = content[pos + form.len()..].chars().next();
        if !after.is_some_and(|c| !(c.is_alphanumeric() || c == '_' || c == '.')) {
            continue;
        }
        // Scan only this declaration's `{ ... }` block.
        let Some(brace) = content[pos..].find('{') else {
            continue;
        };
        let brace = pos + brace;
        let Some(block_len) = content[brace..].find('}') else {
            continue;
        };
        let block = &content[brace..brace + block_len];
        for key in VERSION_KEYS {
            for prefix in [format!("{key}:"), format!("{key} =")] {
                let Some(kpos) = block.find(&prefix) else {
                    continue;
                };
                // Word-start check so `rev:` doesn't hit `prev:`.
                let before = block[..kpos].chars().last();
                if !before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    return Some(format!(
                        "pac.at dep `{dep_name}` declares `{key}` — the version plane is \
                         not enabled (single-source ruling: docs/specs/blueprint/contract.md \
                         Q5); declare `path` only, cross-repo alignment is carried by git"
                    ));
                }
            }
        }
    }
    None
}

/// PLAN-649 SD-01 (contract Q5): candidate path forms for a dotted use
/// module — the literal dot→separator conversion first, then hyphen variants
/// of underscore segments. bp packages are kebab-case on disk
/// (`packages/blueprints/feedback/empty-state/`) while use paths are written with
/// underscores (`bps.feedback.empty_state.reference.error`); literal-first
/// keeps co-existing `foo_bar/` vs `foo-bar/` resolution deterministic.
/// Variant combinations enumerate per eligible segment in segment order and
/// cap at 4 (real bp paths carry 1-2 eligible segments; the cap only bounds

fn module_path_candidates(rel: &str) -> Vec<String> {
    const MAX_VARIANTS: usize = 4;
    let mut out = vec![rel.to_string()];
    let segs: Vec<&str> = rel.split(std::path::MAIN_SEPARATOR).collect();
    let eligible: Vec<usize> = (0..segs.len()).filter(|&i| segs[i].contains('_')).collect();
    if eligible.is_empty() || eligible.len() > 16 {
        return out;
    }
    for mask in 1..(1usize << eligible.len()) {
        if out.len() > MAX_VARIANTS {
            break;
        }
        let cand: Vec<String> = segs
            .iter()
            .enumerate()
            .map(|(i, s)| match eligible.iter().position(|&e| e == i) {
                Some(bit) if mask & (1 << bit) != 0 => s.replace('_', "-"),
                _ => s.to_string(),
            })
            .collect();
        out.push(cand.join(std::path::MAIN_SEPARATOR_STR));
    }
    out
}

pub fn resolve_module_path(base_dir: &std::path::Path, module: &str) -> Option<std::path::PathBuf> {
    // Plan 061:外部后端链接 —— `back.X` 优先映射到后端项目根
    // (`back.api` → `<root>/api.at`,`back.shell` → `<root>/shell.at`;
    // mod.at 形态同样支持)。未配置或不命中则回落常规解析(本地 back/)。
    if let Some(root) = EXTERNAL_BACK_ROOT.get() {
        let mapped = if module == "back" {
            "api".to_string()
        } else if let Some(rest) = module.strip_prefix("back.") {
            rest.replace('.', std::path::MAIN_SEPARATOR_STR)
        } else {
            String::new()
        };
        if !mapped.is_empty() {
            for cand in module_path_candidates(&mapped) {
                let as_file = root.join(format!("{}.at", cand));
                if as_file.exists() {
                    return Some(as_file);
                }
                let as_mod = root.join(&cand).join("mod.at");
                if as_mod.exists() {
                    return Some(as_mod);
                }
            }
        }
    }
    let rel = module.replace('.', std::path::MAIN_SEPARATOR_STR);
    // Helper to probe a candidate directory for {rel}.at / {rel}/mod.at —
    // literal form first, then kebab variants of underscore segments
    // (PLAN-649 SD-01; candidates without underscores are literal-only, so
    // existing behavior is untouched).
    let probe = |dir: &std::path::Path| -> Option<std::path::PathBuf> {
        for cand in module_path_candidates(&rel) {
            let as_file = dir.join(format!("{}.at", cand));
            if as_file.exists() {
                return Some(as_file);
            }
            let as_mod = dir.join(&cand).join("mod.at");
            if as_mod.exists() {
                return Some(as_mod);
            }
        }
        None
    };
    // Direct resolution relative to base_dir.
    if let Some(p) = probe(base_dir) {
        return Some(p);
    }
    if let Some(p) = crate::util::find_aavm_module(&module.replace('.', "/"), base_dir) {
        return Some(p);
    }
    // Plan 327 Phase B: fall back to the parent directory so a sibling layout
    // like src/{front,back}/ resolves. `app.at` in src/front/ imports
    // `back.api` (src/back/api.at); base_dir is src/front/, so the direct probe
    // misses it. Trying the parent (src/) finds src/back/api.at. This mirrors
    // the script path, which seeds CompileSession.source_dirs with the source
    // dir's parent.
    if let Some(parent) = base_dir.parent() {
        if let Some(p) = probe(parent) {
            return Some(p);
        }
    }

    // 向上遍历查找 deps 目录 (如 src/front/ -> src/ -> root/deps/) 或 pac.at 中的本地 path 依赖
    let (dep_name, _) = match module.find('.') {
        Some(dot) => (&module[..dot], &module[dot + 1..]),
        None => (module, ""),
    };
    // PLAN-635 D4/D5: deps/ 与 workspace members 的候选探测序列（与本地
    // path 依赖一致）。声明门控（pnpm 式严格隔离）：deps/<name> 仅在 pac.at
    // `dep` 声明（或 workspace member）时可达；物化但未声明 = 幽灵依赖，
    // 阻断解析并给修复指引。
    let probe_pkg = |pkg_root: &std::path::Path| -> Option<std::path::PathBuf> {
        let (_, sub) = match module.find('.') {
            Some(dot) => (&module[..dot], &module[dot + 1..]),
            None => (module, ""),
        };
        if sub.is_empty() {
            let candidates = [
                pkg_root.join("src").join("front").join("app.at"),
                pkg_root.join("src").join("front").join("mod.at"),
                pkg_root.join("src").join("mod.at"),
                pkg_root.join("mod.at"),
                pkg_root.join(format!("{}.at", dep_name)),
            ];
            candidates.into_iter().find(|c| c.exists())
        } else {
            let sub_rel = sub.replace('.', std::path::MAIN_SEPARATOR_STR);
            // PLAN-649 SD-01: literal sub-path candidates first, then kebab
            // variants of underscore segments (`bps.feedback.empty_state...`
            // reaches `feedback/empty-state/`). Layout priority (front/back/
            // src/flat) is preserved within each candidate form.
            for cand in module_path_candidates(&sub_rel) {
                let candidates = [
                    pkg_root
                        .join("src")
                        .join("front")
                        .join(format!("{}.at", cand)),
                    pkg_root
                        .join("src")
                        .join("front")
                        .join(&cand)
                        .join("mod.at"),
                    pkg_root
                        .join("src")
                        .join("back")
                        .join(format!("{}.at", cand)),
                    pkg_root.join("src").join("back").join(&cand).join("mod.at"),
                    pkg_root.join("src").join(format!("{}.at", cand)),
                    pkg_root.join("src").join(&cand).join("mod.at"),
                    pkg_root.join("front").join(format!("{}.at", cand)),
                    pkg_root.join("front").join(&cand).join("mod.at"),
                    pkg_root.join(format!("{}.at", cand)),
                    pkg_root.join(&cand).join("mod.at"),
                ];
                if let Some(hit) = candidates.into_iter().find(|c| c.exists()) {
                    return Some(hit);
                }
            }
            None
        }
    };
    let mut curr_dir = Some(base_dir);
    for _ in 0..4 {
        let Some(d) = curr_dir else { break };
        let pac_content = std::fs::read_to_string(d.join("pac.at")).ok();
        let dep_declared = pac_content
            .as_deref()
            .is_some_and(|c| pac_declares_dep(c, dep_name));
        let member_entry = pac_content
            .as_deref()
            .and_then(|c| pac_workspace_member_dir(c, dep_name));

        // PLAN-647 guardrail B: a version-ish key on this dep's declaration
        // fails the resolution outright (even when `path:` would resolve) —
        // resolving anyway would validate a fake constraint.
        if dep_declared {
            if let Some(pac) = &pac_content {
                if let Some(msg) = pac_dep_version_violation(pac, dep_name) {
                    eprintln!("error: {msg}");
                    return None;
                }
            }
        }

        let deps_candidate = d.join("deps");
        if deps_candidate.is_dir() {
            if dep_declared {
                if let Some(p) = probe_pkg(&deps_candidate.join(dep_name)) {
                    return Some(p);
                }
            } else if !dep_name.is_empty()
                && member_entry.is_none()
                && deps_candidate.join(dep_name).exists()
            {
                // PLAN-659 T-09：空名模块探测不触发幽灵依赖守卫——
                // deps/"" 恒为 deps/ 本体（exists 恒真），画廊 boot 期
                // 空名解析批次曾把该守卫刷成 "dependency ''" 噪音 ×N。
                eprintln!(
                    "error: dependency '{}' is materialized at {} but not declared in pac.at — declare it (`dep \"{}\" {{ path: ... }}`) or add it to workspace members",
                    dep_name,
                    deps_candidate.display(),
                    dep_name
                );
            }
        }

        // PLAN-635 D5: workspace members resolve like deps/<name>.
        if let Some(entry) = &member_entry {
            let member_root = d.join(entry);
            if let Some(p) = probe_pkg(&member_root) {
                return Some(p);
            }
        }

        // pac.at 中的本地 path 依赖（dep "<name>" { path: ... } 声明形态，
        // 天然过门控）。
        if let Some(content) = &pac_content {
            if let Some(pos) = content
                .find(&format!("dep \"{}\"", dep_name))
                .or_else(|| content.find(&format!("dep {}", dep_name)))
            {
                let slice = &content[pos..];
                if let Some(path_pos) = slice.find("path:") {
                    let path_slice = &slice[path_pos + 5..];
                    let path_line = path_slice
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches(|c| c == '"' || c == '\'');
                    if !path_line.is_empty() {
                        let local_dep_dir = d.join(path_line);
                        if local_dep_dir.is_dir() {
                            if let Some(p) = probe_pkg(&local_dep_dir) {
                                return Some(p);
                            }
                        }
                    }
                }
            }
        }
        curr_dir = d.parent();
    }

    None
}

/// Plan 442 A2: outcome of resolving one `use` statement's module in the
/// VM-render loader.
pub enum UseModuleResolution {
    /// Direct module file (`{module}.at` / `{module}/mod.at`).
    Module(std::path::PathBuf),
    /// Legacy `use store: X` facade — resolved StoreDecl files by store name.
    StoreFiles(std::collections::HashMap<String, std::path::PathBuf>),
    /// Nothing resolvable.
    None,
}

/// Resolve a use statement's module for the VM-render loader: direct module
/// path first; on failure, the legacy store facade (`use store: X` — module
/// name is literally "store", StoreDecls live in sibling files). Shared by
/// the root use loop, the `collect_module_imports` recursion, and the
/// plan370/plan442 test support so tests exercise the production logic.
pub fn resolve_use_module(
    base_dir: &std::path::Path,
    use_stmt: &crate::use_scanner::UseStatement,
) -> UseModuleResolution {
    if let Some(p) = resolve_module_path(base_dir, &use_stmt.module) {
        return UseModuleResolution::Module(p);
    }
    if use_stmt.module == "store" && !use_stmt.items.is_empty() {
        return UseModuleResolution::StoreFiles(find_store_decl_files(base_dir, &use_stmt.items));
    }
    // PLAN-632 F2: dep 目录 item 命名文件探测——`use settings: SettingsPopover`
    // → `deps/settings/settings_popover.at`。此前 dep 目录候选只认
    // `{dep}.at`/`mod.at`/`app.at`,item 命名的组件文件永不命中,组件实例
    // 在 VM 渲染目标整体缺席(006 实证)。探测走 resolve_module_path 的
    // dotted-module 形态(`{dep}.{item_snake}` 的 sub 臂已含
    // `deps/{dep}/{sub}.at` 平铺候选),deps 走查/pac.at path dep 零重复。
    // item 优先 snake_case(PascalCase 组件 ↔ 文件命名惯例,与
    // find_store_decl_files 同款);裸 item 兜底。
    if !use_stmt.items.is_empty() {
        for item in &use_stmt.items {
            let mut cands = vec![store_name_snake(item)];
            if *cands.last().unwrap() != *item {
                cands.push(item.clone());
            }
            for cand in cands {
                let dotted = format!("{}.{}", use_stmt.module, cand);
                if let Some(p) = resolve_module_path(base_dir, &dotted) {
                    return UseModuleResolution::Module(p);
                }
            }
        }
    }
    UseModuleResolution::None
}

/// Plan 442 A3 (ext link): load `use.web` ext imports for the VM render
/// target. Previously UseWeb was ignored entirely — pure-Auto helpers
/// imported via `use.web X from "…/port.at"` never got a definition, and any
/// handler calling them left an unresolved CALL reloc that killed the whole
/// VmBridge link ("Undefined symbol"). This function:
///   1. loads `.at` ext sources through the port-adapter chain
///      (X.at → X.vm.at → X.web.at) — their pure-Auto fns become real module
///      symbols, aliased to the adapter module's qualified name (mirroring
///      how scanner-resolved use imports alias their items);
///   2. synthesizes no-op platform stubs for remaining ext symbols (TS/npm
///      sources) with the arity observed at call sites — unless
///      `AUTO_VM_EXT_STUBS=0` restores the strict hard-link-error behavior.
/// Shared with the plan370/plan442 test support so tests exercise the
/// production logic.
pub fn read_auto_lib(project_root: &std::path::Path) -> AutoResult<String> {
    let mut lib_code = String::new();
    for file in AUTO_LIB_FILES {
        let path = project_root.join(file);
        if path.exists() {
            lib_code.push_str(&std::fs::read_to_string(&path)?);
            lib_code.push('\n');
        }
    }
    Ok(lib_code)
}

// PLAN-752: cached AAVM v1 bootstrap lib (AUTO_LIB_FILES full manifest) for the
// playground prepend_lib path. Mirrors the golden is_bootstrap concat
// (run_vm_file_test: `format!("{}\n{}", lib, src)`), single source of truth
// stays the pub(crate) manifest above.
static BOOTSTRAP_LIB: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

/// Concatenated `packages/aavm/lib-legacy/*.at` bootstrap lib (~188KB, dependency
/// order per AUTO_LIB_FILES). `None` when no manifest file resolves (caller
/// treats prepend_lib as unavailable). Result is cached after first call.
pub fn bootstrap_lib_source() -> Option<&'static str> {
    BOOTSTRAP_LIB
        .get_or_init(|| {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let project_root = root
                .parent() // crates/
                .and_then(|p| p.parent()) // repo root
                .unwrap_or(root);
            read_auto_lib(project_root).ok().filter(|s| !s.is_empty())
        })
        .as_deref()
}

/// Find the project root by walking up from a path, looking for `auto/` dir or `Cargo.toml`.
pub fn find_project_root(from: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut dir = from;
    loop {
        if dir.join("packages/aavm").is_dir() || dir.join("Cargo.toml").is_file() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Plan 262: Run a single VM file-based test case.
/// Returns a `FileTestReport` with pass/fail outcome.
fn extract_value_from_vm(
    vm: &crate::vm::engine::AutoVM,
    bits: i32,
    visited: &mut std::collections::HashSet<u64>,
) -> Value {
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

    // 1. Check if it's an object ID (Plan 390 §15 H3b: ObjectData in
    // heap_objects).
    if let Some(obj_ref) = vm.get_heap_object(id) {
        let obj_guard = obj_ref.read().unwrap();
        if let Some(obj_data) = obj_guard
            .as_any()
            .downcast_ref::<crate::vm::types::ObjectData>()
        {
            let mut result_obj = Obj::new();
            for (key, val) in &obj_data.fields {
                let extracted = extract_auto_val_value(vm, val, visited);
                result_obj.set(key.clone(), extracted);
            }
            visited.remove(&id);
            return Value::Obj(Box::new(result_obj));
        }
    }

    // 2. Check if it's a node ID
    // Plan 390 §15 H3a: nodes live in heap_objects — resolve via
    // get_heap_object + downcast instead of the deleted `nodes` registry.
    if let Some(node_ref) = vm.get_heap_object(id) {
        let node_data = node_ref.read().unwrap();
        if let Some(node) = node_data.as_any().downcast_ref::<Node>() {
            let result = Value::node(extract_node_deep(vm, node, visited));
            visited.remove(&id);
            return result;
        }
    }

    // 3. Check if it's an array ID (Plan 390 §15 H3b: ListData<Value> in
    // heap_objects — same storage as CREATE_ARRAY).
    if let Some(list_ref) = vm.get_heap_object(id) {
        let array_data = list_ref.read().unwrap();
        if let Some(list) = array_data
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<Value>>()
        {
            let mut items = Vec::new();
            for val in list.elems.iter() {
                items.push(extract_auto_val_value(vm, val, visited));
            }
            visited.remove(&id);
            return Value::Array(Array::from_vec(items));
        }
    }

    visited.remove(&id);
    // Fallback to integer (for non-heap values)
    Value::Int(bits)
}

fn extract_auto_val_value(
    vm: &crate::vm::engine::AutoVM,
    val: &Value,
    visited: &mut std::collections::HashSet<u64>,
) -> Value {
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
            Value::Obj(Box::new(result_obj))
        }
        Value::Node(node) => Value::node(extract_node_deep(vm, node, visited)),
        _ => val.clone(),
    }
}

fn extract_node_deep(
    vm: &crate::vm::engine::AutoVM,
    node: &Node,
    visited: &mut std::collections::HashSet<u64>,
) -> Node {
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
pub fn eval_config_with_vm(code: &str, args: &Obj) -> AutoResult<Value> {
    crate::install_ui_host_capabilities();
    use crate::parser::CompileDest;
    use crate::vm::codegen::Codegen;
    use crate::vm::engine::AutoVM;
    use crate::vm::virt_memory::VirtualFlash;

    // Note: Plan 091 - Universe parameter removed, AutoVM uses its own state
    // Note: Do NOT preprocess macros here — pac.at is config code, not UI code.
    // The `app` keyword in pac.at means a node definition (app (id: "main") {...}),
    // not a UI macro (which would expand to `type ... is App {...}`).

    // 1. Parse the code
    let mut parser = Parser::from(code);
    // Plan 364: Config mode enables tolerant manifest parsing (bare-name nodes,
    // relaxed undefined checks) and context-variable injection (e.g. `port`).
    parser.compile_dest = CompileDest::Config;
    if let Some(port) = args.get_str("port") {
        parser.define_config_globals(port.as_str());
    }
    let ast = parser.parse()?;

    // 2. Compile to bytecode using the unified Codegen in config_mode.
    // Plan 364 Step 5: replaces the old compile-time-only ConfigCodegen with a
    // runtime-evaluating VM Config mode (accumulation opcodes + Script control
    // flow), so f-string templates, object field access, and for-over-array
    // work at runtime.
    //
    // Plan 013: inject `on`/`off` as built-in config globals (true/false) so
    // manifest toggles can read naturally, e.g. pac.at `shadcn: off`. Without
    // these, a bareword `off`/`on` value aborts the whole config eval with
    // "Undefined variable". Caller-supplied args win (only set when absent).
    let mut args_with_builtins = args.clone();
    if !args_with_builtins.has("on") {
        args_with_builtins.set("on", Value::Bool(true));
    }
    if !args_with_builtins.has("off") {
        args_with_builtins.set("off", Value::Bool(false));
    }
    let mut codegen = Codegen::new_for_config();
    codegen.compile_config_program(&ast, &args_with_builtins)?;

    // 3. Perform linking (resolve function calls)
    let strings = codegen.strings.clone();
    let exports = codegen.exports.clone();
    let relocs = codegen.relocs.clone();

    for reloc in &relocs {
        if let Some(&addr) = exports.get(&reloc.symbol_name) {
            let bytes = addr.to_le_bytes();
            let offset = reloc.offset as usize;
            for (i, b) in bytes.iter().enumerate() {
                codegen.code[offset + i] = *b;
            }
        } else {
            return Err(AutoError::Msg(format!(
                "Undefined symbol in config: {}",
                reloc.symbol_name
            )));
        }
    }

    // 4. Load into VM and execute
    let bytecode = codegen.code.clone();
    let object_keys = codegen.object_keys.clone();
    let object_types = codegen.object_types.clone();
    // Plan 197 Task 9: Extract generic registry for VM
    let generic_registry = std::mem::take(&mut codegen.generic_registry);
    // Plan 327 Phase 1: take task handler registry for VM
    let task_handler_registry = std::mem::take(&mut codegen.task_handler_registry);

    let rt = get_global_runtime();
    rt.block_on(async {
        let flash = VirtualFlash::new_with_code_and_keys(bytecode, object_keys, object_types);
        let mut vm = AutoVM::new(flash, 32768); // 32KB RAM for config (deeply nested manifests)
        vm.load_strings(strings);
        vm.load_generic_registry(generic_registry);
        vm.load_task_handler_registry(task_handler_registry); // Plan 327 Phase 1

        // 5. Execute from entry point (default to 0 for config)
        let entry_point = exports.get("main").copied().unwrap_or(0) as usize;

        let task_id = vm.spawn_task(entry_point, 32768); // 32KB task stack (deeply nested manifests)

        // Run the VM to completion
        vm.run_task_loop().await;

        // 6. Get the result from the VM stack.
        // Plan 364 Step 5: the result is the root Node (POP_ACCUM leaves its
        // id on the stack). No Obj->root conversion is needed anymore.
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

            Ok(materialized_result)
        } else {
            // Task not found - return Nil
            Ok(Value::Nil)
        }
    })
}

/// Transpile AutoLang file to C
///
/// **Plan 091**: Now uses CompileSession internally (no Universe dependency)

// ---- PLAN-814：UI 宿主装配钩子（原 REPL 隐式扇入点之一，具名收口） ----
// 纯 SDK 无 UI 实现：钩子未注册时 no-op；上层 auto-lang 在装配点注册真实
// 安装器（实现留上层）。SDK 不依赖 inventory/linker 偶然性拉 UI。
static UI_HOST_INSTALL_HOOK: OnceLock<fn()> = OnceLock::new();

pub fn set_ui_host_install_hook(install: fn()) {
    let _ = UI_HOST_INSTALL_HOOK.set(install);
}

/// PLAN-814 R3-01/R3-04：上层兼容装配的进程期自注册收集点。语义边界
/// （对照 r2 §2 "不靠 linker inventory 偶然拉入初始化"）：SDK **不提交**
/// 任何安装器——纯 SDK 消费者零 submit、钩子缺席=显式拒绝/回退语义不
/// 变；上层 auto-lang 被链接即其 submit 生效（确定性自注册，覆盖不经
/// run 族漏斗的直达调用——native dispatch/旧公开入口）。
pub struct UiCompatInstaller(pub fn());
inventory::collect!(UiCompatInstaller);

pub fn install_ui_host_capabilities() {
    if let Some(install) = UI_HOST_INSTALL_HOOK.get() {
        install();
        return;
    }
    // 钩子未注册：尝试上层自注册安装器（无 submit 时为空——纯 SDK 形态）。
    for installer in inventory::iter::<UiCompatInstaller> {
        (installer.0)();
    }
}

pub mod heavy_gate;

// ---- PLAN-814：i18n lookup 钩子（原 shim_i18n_t 直连 crate::ui 的残边收口） ----
// 纯 SDK：未注册钩子时返回 None（shim 回退原 key）。上层 auto-lang 在装配点
// 注册 ui::i18n_lookup::lookup，行为与旧 ui-iced 门内实现一致。
static I18N_LOOKUP_HOOK: OnceLock<fn(&str) -> Option<String>> = OnceLock::new();

pub fn set_i18n_lookup_hook(lookup: fn(&str) -> Option<String>) {
    let _ = I18N_LOOKUP_HOOK.set(lookup);
}

pub fn i18n_lookup(key: &str) -> Option<String> {
    I18N_LOOKUP_HOOK.get().and_then(|f| f(key))
}

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

    // Auto-start the HTTP server on the VM's owner thread. The synchronous
    // entry points wrap execution in a LocalSet; each connection task retains
    // an Rc clone, so the !Send VM stays on this thread and outlives the task.
    let routes = crate::vm::ffi::stdlib::get_http_routes();
    if !routes.is_empty() {
        // PLAN-736 AC-01: 显式服务配置面（`auto serve`/run_file 服务装配）——
        // listen 取自解析后的 HttpServiceConfig（默认 loopback），不再读 env；
        // seam 由 serve 入口在 spawn VM 线程前 set、此处 take 一次。
        if let Some(svc) = crate::http_service_config::take_active_service_config() {
            // PLAN-736 T-05/AC-02：业务路由占用 /__auto/* 控制面 = 装配诊断
            // （非零退出；不静默遮蔽控制面）。
            let control_conflicts: Vec<&str> = routes
                .iter()
                .map(|r| r.1.as_str())
                .filter(|p| p.starts_with("/__auto/"))
                .collect();
            if !control_conflicts.is_empty() {
                return Err(crate::error::AutoError::Msg(format!(
                    "http service: business routes conflict with the reserved control plane: {:?}",
                    control_conflicts
                )));
            }
            let addr = format!("{}:{}", svc.listen_addr, svc.listen_port);
            // PLAN-736 AC-03/05: 预算与观测由服务配置驱动（env 不参与该面）。
            crate::http_service_observability::install_log_sink(
                svc.observability.log_sink_capacity,
            );
            crate::http_service_observability::emit_event(serde_json::json!({
                "event": "startup",
                "profile": svc.profile.as_str(),
                "addr": addr,
                "config_hash": format!("{:016x}", svc.effective_config_hash()),
            }));
            eprintln!(
                "[HTTP] Auto-starting server with {} route(s) on {} (profile: {}, config_hash: {:016x})",
                routes.len(), addr, svc.profile.as_str(), svc.effective_config_hash()
            );
            crate::http_service_config::set_service_policy_active();
            let runtime = std::sync::Arc::new(crate::http_service_config::ServiceRuntime::new(svc));
            let transport =
                crate::vm::ffi::http_transport::TransportConfig::from_service_runtime(runtime);
            crate::vm::ffi::http_server::serve_async_with(std::rc::Rc::new(vm), &addr, transport)
                .await;
            if let Some(fatal) = crate::vm::ffi::http_server::serve_fatal_snapshot() {
                // peek 不消费：外层 serve 入口 join 后 take 收割同一条诊断。
                crate::http_service_observability::emit_event(serde_json::json!({
                    "event": "config_error",
                    "reason": fatal,
                }));
                return Err(crate::error::AutoError::Msg(format!(
                    "http service failed to start: {fatal}"
                )));
            }
            return Ok((result, get_stdout(), bytecode_lines, bytecode_meta));
        }
        let port = std::env::var("AUTO_HTTP_PORT").unwrap_or_else(|_| "8080".to_string());
        let addr = format!("0.0.0.0:{}", port);
        eprintln!(
            "[HTTP] Auto-starting server with {} route(s) on {}",
            routes.len(),
            addr
        );

        crate::vm::ffi::http_server::serve_async(std::rc::Rc::new(vm), &addr).await;
    }

    Ok((result, get_stdout(), bytecode_lines, bytecode_meta))
}

pub fn store_name_snake(name: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                let next_lower = chars.get(i + 1).map_or(false, |n| n.is_ascii_lowercase());
                if prev.is_ascii_lowercase() || prev.is_ascii_digit() || next_lower {
                    out.push('_');
                }
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

pub fn find_store_decl_files(
    base_dir: &std::path::Path,
    names: &[String],
) -> std::collections::HashMap<String, std::path::PathBuf> {
    use std::collections::HashMap;
    let mut found: HashMap<String, std::path::PathBuf> = HashMap::new();
    let mut missing: Vec<&String> = Vec::new();
    // Convention probe: snake_case(Name).at (covers musk's nine stores) and
    // the `_store` suffix variant (for stores whose decl name has no Store
    // suffix, e.g. `store Notes` in notes_store.at).
    for name in names {
        let snake = store_name_snake(name);
        let mut hit = None;
        for cand in [
            format!("{}.at", snake),
            format!("{}_store.at", snake.trim_end_matches("_store")),
        ] {
            let p = base_dir.join(&cand);
            if p.exists() {
                hit = Some(p);
                break;
            }
        }
        match hit {
            Some(p) => {
                found.insert(name.clone(), p);
            }
            None => missing.push(name),
        }
    }
    if missing.is_empty() {
        return found;
    }
    // Bounded scan fallback: walk the source dir (and its parent, mirroring
    // resolve_module_path's sibling layout) for .at files declaring the
    // missing stores. Heavy dirs (gen output / node_modules / build caches)
    // are skipped; the walk is capped so a stray huge tree cannot stall the
    // loader.
    let mut budget = 400usize;
    let mut scan = |root: &std::path::Path,
                    found: &mut HashMap<String, std::path::PathBuf>,
                    missing: &[&String]| {
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            let mut entries: Vec<_> = entries.flatten().collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let path = entry.path();
                let fname = entry.file_name();
                let fname = fname.to_string_lossy();
                if path.is_dir() {
                    if matches!(
                        fname.as_ref(),
                        "gen" | "node_modules" | "target" | "dist" | ".git" | "build"
                    ) {
                        continue;
                    }
                    stack.push(path);
                } else if fname.ends_with(".at") && budget > 0 {
                    budget -= 1;
                    let Ok(code) = std::fs::read_to_string(&path) else {
                        continue;
                    };
                    let code = crate::back_prefix::apply(&path, code);
                    let session = crate::session::CompilerSession::ui();
                    let mut parser = Parser::from(code.as_str()).with_session(session);
                    let Ok(ast) = parser.parse() else { continue };
                    for stmt in &ast.stmts {
                        if let crate::ast::Stmt::StoreDecl(sd) = stmt {
                            let n = sd.name.as_str().to_string();
                            if missing.iter().any(|m| *m == &n) && !found.contains_key(&n) {
                                found.insert(n, path.clone());
                            }
                        }
                    }
                }
            }
        }
    };
    scan(base_dir, &mut found, &missing);
    let still: Vec<&String> = missing
        .iter()
        .filter(|m| !found.contains_key(m.as_str()))
        .copied()
        .collect();
    if !still.is_empty() {
        if let Some(parent) = base_dir.parent() {
            scan(parent, &mut found, &still);
        }
    }
    found
}

// ---- T-02 终批根项（runners/FFI init/错误定位/remap/结果提取）----
/// Plan 262: Run a single VM file-based test case.
/// Returns a `FileTestReport` with pass/fail outcome.
pub fn run_vm_file_test(case: &test_runner::FileTestCase) -> test_runner::FileTestReport {
    use std::time::Instant;

    let start = Instant::now();
    let case_name = case.name.clone();

    // Read source file
    let src = match std::fs::read_to_string(&case.source_file) {
        Ok(s) => s,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("failed to read source: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Prepend auto/lib for bootstrap tests
    let src = if case.is_bootstrap {
        let project_root = match find_project_root(&case.dir) {
            Some(r) => r,
            None => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(
                        "cannot find project root for bootstrap test".into(),
                    ),
                    duration_ms: start.elapsed().as_millis(),
                    stdout: String::new(),
                };
            }
        };
        match read_auto_lib(&project_root) {
            Ok(lib) => format!("{}\n{}", lib, src),
            Err(e) => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(format!(
                        "failed to read auto lib: {}",
                        e
                    )),
                    duration_ms: start.elapsed().as_millis(),
                    stdout: String::new(),
                };
            }
        }
    } else {
        src
    };

    // Error expectation test
    if case.expected_error.is_some() {
        let result = run(&src);
        let duration_ms = start.elapsed().as_millis();
        return match result {
            Err(_) => test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Passed,
                duration_ms,
                stdout: String::new(),
            },
            Ok(v) => test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!(
                    "expected error but got: {:?}",
                    v
                )),
                duration_ms,
                stdout: String::new(),
            },
        };
    }

    // Normal execution with capture
    // Plan 560 T14：文件测试走带 path 通道（硬化门控对 .at 文件生效、
    // .as 豁免——内联 run_with_capture 不门控，见 execute 段注记）。
    let src_path_str = case.source_file.to_string_lossy().to_string();
    let (result, stdout) = match run_with_capture_and_path(&src, &src_path_str) {
        Ok(r) => r,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("execution error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    let duration_ms = start.elapsed().as_millis();

    // Check stdout against .expected.out
    if let Some(ref expected_path) = case.expected_out {
        let expected = match std::fs::read_to_string(expected_path) {
            Ok(e) => e,
            Err(e) => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(format!(
                        "failed to read expected.out: {}",
                        e
                    )),
                    duration_ms,
                    stdout: stdout.clone(),
                };
            }
        };
        if stdout != expected {
            let wrong_path = expected_path.with_extension("wrong.out");
            let _ = std::fs::write(&wrong_path, &stdout);
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed("stdout mismatch".into()),
                duration_ms,
                stdout,
            };
        }
    }

    // Check return value against .expected.result
    if let Some(ref expected_path) = case.expected_result {
        let expected = match std::fs::read_to_string(expected_path) {
            Ok(e) => e,
            Err(e) => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(format!(
                        "failed to read expected.result: {}",
                        e
                    )),
                    duration_ms,
                    stdout: stdout.clone(),
                };
            }
        };
        if result != expected {
            let wrong_path = expected_path.with_extension("wrong.result");
            let _ = std::fs::write(&wrong_path, &result);
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed("result mismatch".into()),
                duration_ms,
                stdout,
            };
        }
    }

    test_runner::FileTestReport {
        name: case_name,
        outcome: test_runner::TestOutcome::Passed,
        duration_ms,
        stdout,
    }
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

/// Plan 263 Phase 3: Run a single a2ts transpiler file-based test case.
/// Reads .at source, transpiles to TypeScript, compares against .expected.ts.
pub fn run_a2ts_file_test(case: &test_runner::A2tsTestCase) -> test_runner::FileTestReport {
    use std::time::Instant;

    let start = Instant::now();
    let case_name = case.name.clone();

    let stem = case
        .source_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("test");

    // Read source
    let src = match std::fs::read_to_string(&case.source_file) {
        Ok(s) => s,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("failed to read source: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Parse and transpile
    let _scope = crate::scope_manager::ScopeManager::new();
    let mut parser = crate::parser::Parser::from(src.as_str());
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("parse error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    let mut sink = crate::trans::Sink::new(stem.into());
    let mut trans = crate::trans::typescript::TypeScriptTrans::new(stem.into());
    let ts_code = match trans.trans(ast, &mut sink) {
        Ok(_) => match sink.done() {
            Ok(bytes) => bytes.to_vec(),
            Err(e) => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(format!(
                        "sink finalize error: {}",
                        e
                    )),
                    duration_ms: start.elapsed().as_millis(),
                    stdout: String::new(),
                };
            }
        },
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("transpiler error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Read expected
    let expected = match std::fs::read(&case.expected_file) {
        Ok(b) => b,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!(
                    "failed to read expected.ts: {}",
                    e
                )),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    let duration_ms = start.elapsed().as_millis();

    if ts_code != expected {
        let wrong_path = case.expected_file.with_extension("wrong.ts");
        let _ = std::fs::write(&wrong_path, &ts_code);
        test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Failed("transpiler output mismatch".into()),
            duration_ms,
            stdout: String::from_utf8_lossy(&ts_code).to_string(),
        }
    } else {
        test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Passed,
            duration_ms,
            stdout: String::new(),
        }
    }
}

/// Plan 263 Phase 3: Run a single a2c transpiler file-based test case.
/// Reads .at source, transpiles to C, compares against .expected.c and .expected.h.
pub fn run_a2c_file_test(case: &test_runner::A2cTestCase) -> test_runner::FileTestReport {
    use std::time::Instant;

    let start = Instant::now();
    let case_name = case.name.clone();

    let stem = case
        .source_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("test");

    // Read source
    let src = match std::fs::read_to_string(&case.source_file) {
        Ok(s) => s,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("failed to read source: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Check if this is an error test
    if case.expected_error.is_some() {
        let result = crate::trans::c::transpile_c(stem, &src);
        let duration_ms = start.elapsed().as_millis();
        return match result {
            Err(_) => test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Passed,
                duration_ms,
                stdout: String::new(),
            },
            Ok(_) => test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(
                    "expected error but transpilation succeeded".into(),
                ),
                duration_ms,
                stdout: String::new(),
            },
        };
    }

    // Normal transpile
    let mut ccode = match crate::trans::c::transpile_c(stem, &src) {
        Ok(c) => c,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("transpiler error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    let actual_src = match ccode.done() {
        Ok(bytes) => bytes.to_vec(),
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!(
                    "transpiler finalize error: {}",
                    e
                )),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };
    let actual_header = ccode.header;

    let duration_ms = start.elapsed().as_millis();

    // Compare .c output
    let expected_c = std::fs::read(&case.expected_c).unwrap_or_default();
    if actual_src != expected_c {
        let wrong_path = case.expected_c.with_extension("wrong.c");
        let _ = std::fs::write(&wrong_path, &actual_src);
        return test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Failed("C source mismatch".into()),
            duration_ms,
            stdout: String::from_utf8_lossy(&actual_src).to_string(),
        };
    }

    // Compare .h output
    let expected_h = std::fs::read(&case.expected_h).unwrap_or_default();
    if actual_header != expected_h {
        let wrong_path = case.expected_h.with_extension("wrong.h");
        let _ = std::fs::write(&wrong_path, &actual_header);
        return test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Failed("C header mismatch".into()),
            duration_ms,
            stdout: String::from_utf8_lossy(&actual_header).to_string(),
        };
    }

    test_runner::FileTestReport {
        name: case_name,
        outcome: test_runner::TestOutcome::Passed,
        duration_ms,
        stdout: String::new(),
    }
}

/// Plan 263: Run a single a2r transpiler file-based test case.
/// Reads .at source, transpiles to Rust, compares against .expected.rs.
pub fn run_a2r_file_test(case: &test_runner::A2rTestCase) -> test_runner::FileTestReport {
    use std::time::Instant;

    let start = Instant::now();
    let case_name = case.name.clone();

    // Extract stem from source file name for transpile_rust
    let stem = case
        .source_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("test");

    // Read source
    let src = match std::fs::read_to_string(&case.source_file) {
        Ok(s) => s,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("failed to read source: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Transpile
    // Plan 610 ⑥: relative use.c JSON manifests resolve against the case dir.
    let actual = match crate::trans::rust::transpile_rust_with_source_dir(
        case.source_file
            .parent()
            .unwrap_or(std::path::Path::new(".")),
        stem,
        &src,
    ) {
        Ok(mut rcode) => match rcode.done() {
            Ok(bytes) => bytes.to_vec(),
            Err(e) => {
                return test_runner::FileTestReport {
                    name: case_name,
                    outcome: test_runner::TestOutcome::Failed(format!(
                        "transpiler finalize error: {}",
                        e
                    )),
                    duration_ms: start.elapsed().as_millis(),
                    stdout: String::new(),
                };
            }
        },
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!("transpiler error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    // Read expected
    let expected = match std::fs::read(&case.expected_file) {
        Ok(b) => b,
        Err(e) => {
            return test_runner::FileTestReport {
                name: case_name,
                outcome: test_runner::TestOutcome::Failed(format!(
                    "failed to read expected.rs: {}",
                    e
                )),
                duration_ms: start.elapsed().as_millis(),
                stdout: String::new(),
            };
        }
    };

    let duration_ms = start.elapsed().as_millis();

    if actual != expected {
        let wrong_path = case.expected_file.with_extension("wrong.rs");
        let _ = std::fs::write(&wrong_path, &actual);
        test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Failed("transpiler output mismatch".into()),
            duration_ms,
            stdout: String::from_utf8_lossy(&actual).to_string(),
        }
    } else {
        test_runner::FileTestReport {
            name: case_name,
            outcome: test_runner::TestOutcome::Passed,
            duration_ms,
            stdout: String::new(),
        }
    }
}

/// Plan 359 B1: Remap CREATE_OBJ `key_index` (u16) operands in bytecode after
/// the object_keys/object_types pools are merged across modules.
///
/// When dep modules' object pools are appended to the main module's pool, every
/// CREATE_OBJ instruction in a dep module still holds its original 0-based
/// `key_index`, which now points into the MAIN module's portion of the merged
/// pool (wrong keys/types → corrupt objects, or field_count/types length
/// mismatches that overflow at runtime). This walker decodes instruction
/// lengths precisely (mirroring `remap_string_indices`) and rewrites the u16
/// operand of each CREATE_OBJ using `obj_remap` (old_index → new_index).
pub(crate) fn remap_obj_indices(code: &mut Vec<u8>, obj_remap: &[u32]) {
    use crate::vm::opcode::OpCode;
    if obj_remap.is_empty() {
        return;
    }
    let mut last_const_i32: i32 = 0;
    let mut i = 0;
    while i < code.len() {
        let op_byte = code[i];
        if !OpCode::is_valid(op_byte) {
            i += 1;
            continue;
        }
        let op = OpCode::from(op_byte);
        let operand_pos = i + 1;
        if let OpCode::CONST_I32 = op {
            if operand_pos + 4 <= code.len() {
                last_const_i32 = i32::from_le_bytes([
                    code[operand_pos],
                    code[operand_pos + 1],
                    code[operand_pos + 2],
                    code[operand_pos + 3],
                ]);
            }
        }
        let (operand_len, idxs) =
            operand_span_with_pool_indices(op, code, operand_pos, last_const_i32);
        for (off, space) in idxs {
            if space == IdxSpace::Obj {
                let pos = operand_pos + off;
                if pos + 4 <= code.len() {
                    let old = u32::from_le_bytes([
                        code[pos],
                        code[pos + 1],
                        code[pos + 2],
                        code[pos + 3],
                    ]) as usize;
                    if old < obj_remap.len() {
                        let bytes = obj_remap[old].to_le_bytes();
                        code[pos] = bytes[0];
                        code[pos + 1] = bytes[1];
                        code[pos + 2] = bytes[2];
                        code[pos + 3] = bytes[3];
                    }
                }
            }
        }
        i += 1 + operand_len;
    }
}

/// Plan 355: dep 模块字符串池并入主池后,重写字节码中的池索引。
/// remap 为 u32(池 u32 化);跨度模型与 remap_obj_indices 共享。
pub(crate) fn remap_string_indices(code: &mut Vec<u8>, remap: &[u32]) {
    use crate::vm::opcode::OpCode;
    let mut last_const_i32: i32 = 0;
    let mut i = 0;
    while i < code.len() {
        let op_byte = code[i];
        if !OpCode::is_valid(op_byte) {
            i += 1;
            continue;
        }
        let op = OpCode::from(op_byte);
        let operand_pos = i + 1;
        if let OpCode::CONST_I32 = op {
            if operand_pos + 4 <= code.len() {
                last_const_i32 = i32::from_le_bytes([
                    code[operand_pos],
                    code[operand_pos + 1],
                    code[operand_pos + 2],
                    code[operand_pos + 3],
                ]);
            }
        }
        let (operand_len, idxs) =
            operand_span_with_pool_indices(op, code, operand_pos, last_const_i32);
        for (off, space) in idxs {
            if space == IdxSpace::Str {
                let pos = operand_pos + off;
                if pos + 4 <= code.len() {
                    let raw = u32::from_le_bytes([
                        code[pos],
                        code[pos + 1],
                        code[pos + 2],
                        code[pos + 3],
                    ]);
                    // 0xFFFF "无 id" 哨兵不重映射(PUSH_ACCUM/CREATE_NODE
                    // 的可选 id 位;吸收自 417-followup 的 apply_u32_opt)。
                    if raw == 0xFFFF {
                        continue;
                    }
                    let old = raw as usize;
                    if old < remap.len() {
                        let bytes = remap[old].to_le_bytes();
                        code[pos] = bytes[0];
                        code[pos + 1] = bytes[1];
                        code[pos + 2] = bytes[2];
                        code[pos + 3] = bytes[3];
                    }
                }
            }
        }
        i += 1 + operand_len;
    }
}

#[cfg(not(feature = "python"))]
pub fn init_py_ffi(
    _session: &compile::CompileSession,
) -> Option<crate::vm::native::NativeInterface> {
    None
}

/// Plan 214/300: Initialize Python FFI bridge if there are use.py imports
///
/// Creates a PyFfiBridge, imports the requested Python modules,
/// registers each function as a native shim with auto-type marshalling,
/// and returns the NativeInterface for merging into the VM.
///
/// Plan 300: Uses Python `inspect.signature()` to detect param count,
/// then registers with `PySignature::all_auto(count)` for runtime
/// NanoValue tag-based type detection instead of hardcoded string→string.
#[cfg(feature = "python")]
pub fn init_py_ffi(
    session: &compile::CompileSession,
) -> Option<crate::vm::native::NativeInterface> {
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

        // Plan 300: For bare module imports (empty items), discover callables via dir()
        if functions.is_empty() {
            let discovered = bridge.discover_module_callables(module_name);
            for (func_name, param_count) in discovered {
                // Plan 567 T17: 裸模块发现面同法灌注定预言机知识。
                let ret_anno = bridge.inspect_return_annotation(module_name, &func_name);
                let sig = {
                    let mut sig = crate::py_ffi_types::PySignature::all_auto(param_count);
                    if let Some(t) = &ret_anno {
                        sig.returns = t.clone();
                    }
                    sig
                };
                if let Some(t) = &ret_anno {
                    crate::py_ffi_types::record_return_annotation(&func_name, t.clone());
                }
                if let Ok(native_id) = bridge.register_function(module_name, &func_name, sig) {
                    // Register with module-qualified name so codegen can find it
                    let qualified = format!("py.{}.{}", module_name, func_name);
                    if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {
                        registry.register_with_id(&qualified, native_id);
                    }
                }
            }
            continue;
        }

        // Named imports — use inspect to get param count for callables; register
        // non-callable attributes (e.g. math.pi) as zero-arg constants (Plan 369 Task 11).
        for func_name in functions {
            // Plan 369 Task 11: constants are non-callable module attributes.
            if !bridge.is_callable(module_name, &func_name) {
                // Plan 567 T17: 常量通道注解（模块级 `x: float = ...`）。
                if let Some(t) = bridge.inspect_constant_annotation(module_name, &func_name) {
                    crate::py_ffi_types::record_return_annotation(&func_name, t);
                }
                match bridge.register_constant(module_name, &func_name) {
                    Ok(native_id) => {
                        log::info!(
                            "Registered Python const: {}.{} (native_id={})",
                            module_name,
                            func_name,
                            native_id
                        );
                        let qualified = format!("py.{}", func_name);
                        if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {
                            registry.register_with_id(&qualified, native_id);
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to register Python const {}.{}: {:?}",
                            module_name,
                            func_name,
                            e
                        );
                    }
                }
                continue;
            }
            let param_count = bridge.inspect_param_count(module_name, &func_name, 1);
            // Plan 567 T17（W3 注解预言机）: 读返回注解——有承诺则签名带
            // 类型（shim 出口 D4 强制），并登记全局知识表（codegen 消费：
            // py_return_types 灌注 + nullable lint）。无注解 = Auto 零变化。
            let ret_anno = bridge.inspect_return_annotation(module_name, &func_name);
            let sig = {
                let mut sig = crate::py_ffi_types::PySignature::all_auto(param_count);
                if let Some(t) = &ret_anno {
                    sig.returns = t.clone();
                }
                sig
            };
            if let Some(t) = &ret_anno {
                crate::py_ffi_types::record_return_annotation(&func_name, t.clone());
                log::info!(
                    "PyOracle: {}.{} -> {:?} (annotated)",
                    module_name,
                    func_name,
                    t
                );
            }
            match bridge.register_function(module_name, func_name, sig) {
                Ok(native_id) => {
                    log::info!(
                        "Registered Python FFI: {}.{} (native_id={}, params={})",
                        module_name,
                        func_name,
                        native_id,
                        param_count
                    );

                    // Also register in BIGVM_NATIVES so codegen can find it
                    let qualified = format!("py.{}", func_name);
                    if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {
                        registry.register_with_id(&qualified, native_id);
                    }
                }
                Err(e) => {
                    log::warn!(
                        "Failed to register Python function {}.{}: {:?}",
                        module_name,
                        func_name,
                        e
                    );
                }
            }
        }
    }

    // Plan 369 Task 12: register py_call / py_getattr built-ins so users can
    // dispatch method calls and attribute access on opaque Python objects
    // returned by earlier py-FFI calls (e.g. datetime.date.isoformat()).
    bridge.register_object_shims();
    if let Ok(mut registry) = crate::vm::native_registry::BIGVM_NATIVES.lock() {
        registry.register_with_id("py.py_call", crate::py_ffi::NATIVE_PY_CALL);
        registry.register_with_id("py.py_getattr", crate::py_ffi::NATIVE_PY_GETATTR);
        // Plan 539 W0 (DIV-PY-KWARGS-1): keyword-argument method-call channel.
        registry.register_with_id("py.py_call_kw", crate::py_ffi::NATIVE_PY_CALL_KW);
        // Plan 539 W0 (DIV-PY-EXCEPT-1): May-valued method-call channel.
        registry.register_with_id("py.py_call_may", crate::py_ffi::NATIVE_PY_CALL_MAY);
        // Plan 539 W0 (DIV-PY-ITER-1): iteration protocol pair.
        registry.register_with_id("py.py_iter", crate::py_ffi::NATIVE_PY_ITER);
        registry.register_with_id("py.py_next", crate::py_ffi::NATIVE_PY_NEXT);
        // Plan 539 W1 (T11-T14): inference idiom builtins.
        registry.register_with_id("py.py_matmul", crate::py_ffi::NATIVE_PY_MATMUL);
        registry.register_with_id("py.py_getitem", crate::py_ffi::NATIVE_PY_GETITEM);
        registry.register_with_id("py.py_setitem", crate::py_ffi::NATIVE_PY_SETITEM);
        registry.register_with_id("py.py_slice", crate::py_ffi::NATIVE_PY_SLICE);
        registry.register_with_id("py.py_call0", crate::py_ffi::NATIVE_PY_CALL0);
        registry.register_with_id("py.py_with", crate::py_ffi::NATIVE_PY_WITH);
        registry.register_with_id("py.py_enter", crate::py_ffi::NATIVE_PY_ENTER);
        registry.register_with_id("py.py_exit", crate::py_ffi::NATIVE_PY_EXIT);
        registry.register_with_id("py.py_item_kw", crate::py_ffi::NATIVE_PY_ITEM_KW);
        registry.register_with_id("py.py_float", crate::py_ffi::NATIVE_PY_FLOAT);
        // Plan 539 W3 (T21): Auto closure as Python callable.
        registry.register_with_id("py.py_callable", crate::py_ffi::NATIVE_PY_CALLABLE);
        // Plan 555 T04: 分发组合子配套三桥（B2 桥半/B6/D8）。
        registry.register_with_id("py.py_setattr", crate::py_ffi::NATIVE_PY_SETATTR);
        registry.register_with_id("py.py_len", crate::py_ffi::NATIVE_PY_LEN);
        registry.register_with_id("py.py_type_name", crate::py_ffi::NATIVE_PY_TYPE_NAME);
        // Plan 560 T04: B7 contains + B8 裸模块句柄。
        registry.register_with_id("py.py_contains", crate::py_ffi::NATIVE_PY_CONTAINS);
        registry.register_with_id("py.py_module", crate::py_ffi::NATIVE_PY_MODULE);
        // Plan 560 T06 (D7): GIL str()。
        registry.register_with_id("py.py_str", crate::py_ffi::NATIVE_PY_STR);
        // Plan 560 T07 (C6)：幂。
        registry.register_with_id("py.py_pow", crate::py_ffi::NATIVE_PY_POW);
        // Plan 560 T08 (C3/C4/C7)。
        registry.register_with_id("py.py_truthy", crate::py_ffi::NATIVE_PY_TRUTHY);
        registry.register_with_id("py.py_is", crate::py_ffi::NATIVE_PY_IS);
        // Plan 567 T06 (P560-D2): may 值通道变体。
        registry.register_with_id("py.py_getattr_may", crate::py_ffi::NATIVE_PY_GETATTR_MAY);
        registry.register_with_id("py.py_getitem_may", crate::py_ffi::NATIVE_PY_GETITEM_MAY);
        // Plan 567 T07 (P539-D5): kwargs×may 组合。
        registry.register_with_id("py.py_call_kw_may", crate::py_ffi::NATIVE_PY_CALL_KW_MAY);
        // Plan 567 T12 (P560-D1): with-as catch 臂再抛通道。
        registry.register_with_id("py.py_raise", crate::py_ffi::NATIVE_PY_RAISE);
        // Plan 567 T18 (W3 D4): GIL int() 显式标量提取。
        registry.register_with_id("py.py_int", crate::py_ffi::NATIVE_PY_INT);
        // Plan 602 (D2): Python 类派生工厂（exec 模板 + 回调方法挂载）。
        registry.register_with_id("py.py_subclass", crate::py_ffi::NATIVE_PY_SUBCLASS);
    }

    let mut native_interface = crate::vm::native::NativeInterface::new();
    native_interface.merge(bridge.native_interface());

    Some(native_interface)
}

/// Plan 212b Task 4: Initialize Rust FFI bridge at runtime
///
/// Scans the CompileSession for Rust dependencies and imports,
/// loads compiled libraries, and registers functions as native shims.
///
/// Returns a NativeInterface with registered Rust FFI shims, or None
/// if there are no Rust imports.
pub fn init_rust_ffi(
    session: &compile::CompileSession,
) -> Option<crate::vm::native::NativeInterface> {
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
        // PLAN-592: 过滤类型导入(首字母大写)——类型走方法 shim 包(dispatch 3000),
        // 不进自由函数 wrapper;不过滤会以计数错配 wrapper 版本键(v3_{count}),
        // 并对 auto_<Type> 符号产生必败的 GetProcAddress 告警。同时去重:
        // collect_rust_imports/resolve_uses 两阶段可能重复登记同一导入名,
        // 计数漂移会使版本键探错 wrapper。
        let mut seen_fns = std::collections::HashSet::new();
        let free_fns: Vec<&String> = functions
            .iter()
            .filter(|f| !f.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
            .filter(|f| seen_fns.insert((*f).clone()))
            .collect();
        // The wrapper library name follows the sandbox naming convention
        let wrapper_name = format!("{}_wrapper", crate_name.replace('-', "_"));

        // Try to load the compiled wrapper library
        if let Some(sandbox) = session.sandbox() {
            // Plan 430 C2: 方法 shim 包(独立于自由函数 wrapper;early-continue 之前加载)
            if let Some((methods_lib, _manifest_json)) = sandbox.find_methods_pack(&crate_name) {
                match unsafe { libloading::Library::new(&methods_lib) } {
                    Ok(lib) => {
                        let lib = std::sync::Arc::new(lib);
                        match crate::vm::ffi::dep_methods::read_shim_manifest(&lib) {
                            Ok(manifest) => {
                                crate::vm::ffi::dep_methods::register_pack(
                                    &crate_name,
                                    lib,
                                    &manifest,
                                );
                            }
                            Err(e) => {
                                log::warn!(
                                    "plan430: shim manifest read failed for {}: {:?}",
                                    crate_name,
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "plan430: methods pack load failed for {} from {}: {}",
                            crate_name,
                            methods_lib.display(),
                            e
                        );
                    }
                }
            }

            // Try v3 cache first (Phase 3C-v2 with sig_code), then v2, then v1。
            // PLAN-592 修复:版本键两侧统一为**导入自由函数数**。
            // PLAN-596 T4 再修:mono 实例使 compile 侧 shim 数(基名+实例)与本侧
            // 导入名数恒差——计数键失效。改为**覆盖校验扫描**:v3_* 候选按
            // mtime 降序逐个装载,读 sig manifest 校验"每个导入名有裸条目或
            // __<label> 实例条目",全覆盖才采用(592 的错变体陷阱由覆盖校验
            // 排除);无覆盖候选再落 v2/v1 旧键。
            let sig_hash_len = free_fns.len();
            let v2_path = sandbox.crate_library_path(&wrapper_name, &format!("v2_{sig_hash_len}"));
            let v1_path = sandbox.crate_library_path(&wrapper_name, "1");
            let lib_ext = if cfg!(target_os = "windows") {
                ".dll"
            } else if cfg!(target_os = "macos") {
                ".dylib"
            } else {
                ".so"
            };
            let v3_prefix = format!("{}-v3_", wrapper_name.replace('-', "_"));
            let mut v3_candidates: Vec<(std::path::PathBuf, std::time::SystemTime)> = Vec::new();
            if let Ok(rd) = std::fs::read_dir(sandbox.crates_path()) {
                for entry in rd.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.starts_with(&v3_prefix) && name.ends_with(lib_ext) {
                        let mtime = entry
                            .metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                        v3_candidates.push((entry.path(), mtime));
                    }
                }
            }
            v3_candidates.sort_by(|a, b| b.1.cmp(&a.1)); // 新→旧

            // 旁路校验:独立 libloading 读候选的 auto__sig_manifest(不进
            // bridge——bridge 对同名 crate 只允许一次装载,先载后验会卡死
            // 扫描;胜者确定后 bridge 一次性装载)。读后立即卸载候选探针。
            let mut lib_path: Option<(std::path::PathBuf, u8)> = None;
            let mut manifest: Option<String> = None;
            'scan: for (cand, _) in &v3_candidates {
                let Ok(probe) = (unsafe { libloading::Library::new(cand) }) else {
                    continue;
                };
                let mjson = (|| -> Option<String> {
                    unsafe {
                        let sym: Result<
                            libloading::Symbol<
                                unsafe extern "C" fn() -> *const std::os::raw::c_char,
                            >,
                            _,
                        > = probe.get(b"auto__sig_manifest");
                        let f = sym.ok()?;
                        let p = f();
                        if p.is_null() {
                            return None;
                        }
                        let c = std::ffi::CStr::from_ptr(p);
                        Some(c.to_string_lossy().into_owned())
                    }
                })();
                drop(probe);
                let Some(mjson) = mjson else { continue };
                let sig_map: std::collections::HashMap<String, String> =
                    crate::ffi::parse_manifest_json(&mjson)
                        .into_iter()
                        .collect();
                let covered = free_fns.iter().all(|f| {
                    sig_map.contains_key(f.as_str())
                        || sig_map.keys().any(|k| k.starts_with(&format!("{}__", f)))
                });
                if covered {
                    lib_path = Some((cand.clone(), 3));
                    manifest = Some(mjson);
                    break 'scan;
                }
            }
            let (lib_path, cache_gen) = match lib_path {
                Some(p) => p,
                None => {
                    // 无覆盖的 v3 候选:退回 592 的精确计数键与 v2/v1(非 mono 场景)
                    let v3_path =
                        sandbox.crate_library_path(&wrapper_name, &format!("v3_{sig_hash_len}"));
                    if v3_path.exists() {
                        (v3_path, 3)
                    } else if v2_path.exists() {
                        (v2_path, 2)
                    } else if v1_path.exists() {
                        (v1_path, 1)
                    } else {
                        log::info!(
                            "Wrapper library not found for {} (no covering v3_* candidate; tried v3_{}/v2/v1)",
                            crate_name,
                            sig_hash_len
                        );
                        continue;
                    }
                }
            };

            // 胜者(或 fallback 精确键)交 bridge 一次性装载——旁路探针不进
            // bridge,此处无论 manifest 是否已取到都必须装载。
            if let Err(e) = bridge.load_rust_library(&crate_name, &lib_path) {
                log::warn!(
                    "Failed to load Rust library {} from {}: {:?}",
                    crate_name,
                    lib_path.display(),
                    e
                );
                continue;
            }
            if cache_gen >= 3 && manifest.is_none() {
                manifest = bridge.load_sig_manifest(&crate_name);
            }
            let manifest = if cache_gen >= 3 { manifest } else { None };

            if let Some(ref manifest_json) = manifest {
                // Parse manifest: {"func_name":"sig_code",...}
                let parsed = crate::ffi::parse_manifest_json(manifest_json);
                let sig_map: std::collections::HashMap<String, String> =
                    parsed.into_iter().collect();

                for func_name in free_fns.iter() {
                    // PLAN-596 T4:mono 实例注册——裸名无条目时枚举
                    // "name__<label>" 实例变体逐个注册(rust.{inst},call 侧
                    // codegen 按实参形态路由到实例名)。
                    if !sig_map.contains_key(func_name.as_str()) {
                        let inst_names: Vec<String> = sig_map
                            .keys()
                            .filter(|k| k.starts_with(&format!("{}__", func_name)))
                            .cloned()
                            .collect();
                        if !inst_names.is_empty() {
                            for inst in inst_names {
                                let Some(sig_code) = sig_map.get(&inst) else {
                                    continue;
                                };
                                let exported = crate::ffi::build_exported_name(&inst, sig_code);
                                let sig = crate::ffi::sig_code_to_signature(sig_code);
                                match bridge.register_function_with_export(
                                    crate_name,
                                    &inst,
                                    &exported,
                                    sig.clone(),
                                ) {
                                    Ok(native_id) => {
                                        log::info!(
                                            "Registered Rust FFI (mono inst): {}::{} (native_id={}, sig={:?})",
                                            crate_name, inst, native_id, sig
                                        );
                                        let qualified = format!("rust.{inst}");
                                        if let Ok(mut registry) =
                                            crate::vm::native_registry::BIGVM_NATIVES.lock()
                                        {
                                            registry.register_with_id(&qualified, native_id);
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!(
                                            "Failed to register mono instance {}::{}: {:?}",
                                            crate_name,
                                            inst,
                                            e
                                        );
                                    }
                                }
                            }
                            continue;
                        }
                    }
                    let (exported_name, signature) =
                        if let Some(sig_code) = sig_map.get(func_name.as_str()) {
                            let exported = crate::ffi::build_exported_name(func_name, sig_code);
                            let sig = crate::ffi::sig_code_to_signature(sig_code);
                            (exported, sig)
                        } else {
                            // Fallback to known_signature or default String→String
                            let sig = crate::ffi::resolve_signature(crate_name, func_name)
                                .unwrap_or_else(|| {
                                    crate::ffi::RustSignature::new()
                                        .param(crate::ffi::RustType::String)
                                        .returns(crate::ffi::RustType::String)
                                });
                            (format!("auto_{}", func_name), sig)
                        };

                    match bridge.register_function_with_export(
                        crate_name,
                        func_name,
                        &exported_name,
                        signature.clone(),
                    ) {
                        Ok(native_id) => {
                            log::info!(
                                "Registered Rust FFI: {}::{} (native_id={}, sig={:?})",
                                crate_name,
                                func_name,
                                native_id,
                                signature
                            );

                            let qualified = format!("rust.{}", func_name);
                            if let Ok(mut registry) =
                                crate::vm::native_registry::BIGVM_NATIVES.lock()
                            {
                                registry.register_with_id(&qualified, native_id);
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to register Rust function {}::{}: {:?}",
                                crate_name,
                                func_name,
                                e
                            );
                        }
                    }
                }
            } else {
                // No manifest — use known_signature or default (legacy v1/v2 path)
                for func_name in free_fns.iter() {
                    let signature = crate::ffi::resolve_signature(crate_name, func_name)
                        .unwrap_or_else(|| {
                            crate::ffi::RustSignature::new()
                                .param(crate::ffi::RustType::String)
                                .returns(crate::ffi::RustType::String)
                        });

                    match bridge.register_function(crate_name, func_name, signature.clone()) {
                        Ok(native_id) => {
                            log::info!(
                                "Registered Rust FFI: {}::{} (native_id={}, sig={:?})",
                                crate_name,
                                func_name,
                                native_id,
                                signature
                            );

                            let qualified = format!("rust.{}", func_name);
                            if let Ok(mut registry) =
                                crate::vm::native_registry::BIGVM_NATIVES.lock()
                            {
                                registry.register_with_id(&qualified, native_id);
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to register Rust function {}::{}: {:?}",
                                crate_name,
                                func_name,
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    // Merge the bridge's native interface into our result
    native_interface.merge(bridge.native_interface());

    Some(native_interface)
}

pub fn extract_undefined_symbol(error_msg: &str) -> Option<&str> {
    error_msg
        .strip_prefix("Undefined symbol: ")
        .and_then(|s| s.split(" in ").next())
}

/// Find the source span of a `use ... : symbol` statement in source code
pub fn find_use_symbol_span(source: &str, error_msg: &str) -> miette::SourceSpan {
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

/// Plan 431 E2:AAVM v2(packages/aavm/lib/,纯 Rust 模式)前置拼接清单——单一事实源。
/// v2 文件由 plan-432 按依赖 topo 序逐个写入,写一个登记一个
/// (token → lexer → ast → parser → typeinfo → opcode → codegen → engine)。
pub(crate) const AUTO_LIB_FILES_V2: &[&str] = &[
    // plan-432 S1/S2: token → lexer → parser(依赖序,见 docs/specs/aavm/file-mapping.md)
    "packages/aavm/lib/token.at",
    "packages/aavm/lib/lexer.at",
    "packages/aavm/lib/parser.at",
    "packages/aavm/lib/typeinfo.at",
    "packages/aavm/lib/codegen.at",
    "packages/aavm/lib/engine.at",
    // plan-434: AA2R(Auto 版 a2r 核心子集;含 434 扩展后的 parser/lexer 见 D38)
    "packages/aavm/lib/a2r.at",
];

/// Plan 517 W2 双轨剥离:读 AUTO_LIB_FILES_V2 拼接并剥除 `use auto.lib.*`
/// 行(拼接消费面专用——行为与模块化前逐字节等价:符号同居一程序;真 use
/// 解析只走 auto.exe 直跑/auto build 模块路径。剥离规则单一事实源,
/// scripts/gen-aavm2-unit.py 镜像之)。
pub(crate) fn aavm2_lib_source(project_root: &std::path::Path) -> AutoResult<String> {
    let mut out = String::new();
    for file in AUTO_LIB_FILES_V2 {
        let content = std::fs::read_to_string(project_root.join(file))?;
        for line in content.lines() {
            if line.trim_start().starts_with("use auto.lib.") {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
    }
    Ok(out)
}

/// 链接器重映射共用的操作数跨度模型(2026-08-22 池 u32 化重写)。
/// 返回 (操作数字节长度, [(池索引偏移, 索引空间)])。
/// 两份重映射 walker(字符串池 / object_keys 池)共用本表,消灭此前
/// 双拷贝各自漂移的病灶(历史上 CALL_SPEC/CREATE_OBJ 宽度就漂过)。
/// 变长操作数(CLOSURE/CREATE_FUTURE/BUILD_FSTR/IS_VARIANT/NEW_INSTANCE)
/// 在此统一解析。
pub(crate) fn operand_span_with_pool_indices(
    op: crate::vm::opcode::OpCode,
    code: &[u8],
    pos: usize,
    last_const_i32: i32,
) -> (usize, Vec<(usize, IdxSpace)>) {
    use crate::vm::opcode::OpCode;
    let none: Vec<(usize, IdxSpace)> = Vec::new();
    match op {
        // 单枚 u32 字符串池索引
        OpCode::LOAD_STR
        | OpCode::LOAD_GLOBAL
        | OpCode::STORE_GLOBAL
        | OpCode::GET_FIELD
        | OpCode::CAPTURE_VAR
        | OpCode::LOAD_CAPTURED
        | OpCode::STORE_CAPTURED
        | OpCode::ACCUM_PAIR => (4, vec![(0, IdxSpace::Str)]),
        // 两枚 u32 字符串池索引(name, id;id 可为 0xFFFF 哨兵)
        OpCode::PUSH_ACCUM => (8, vec![(0, IdxSpace::Str), (4, IdxSpace::Str)]),
        // u32 method_name + u8 argc
        OpCode::CALL_SPEC => (5, vec![(0, IdxSpace::Str)]),
        // u32 name_idx + u8 argc + u32 id_idx(两枚池索引)
        OpCode::CREATE_NODE => (9, vec![(0, IdxSpace::Str), (5, IdxSpace::Str)]),
        // u32 key_index(object_keys 池)+ u8 field_count
        OpCode::CREATE_OBJ => (5, vec![(0, IdxSpace::Obj)]),
        // CALL_NAT: u16 native id —— 池合并后 native id 不变,只跳过
        OpCode::CALL_NAT => (2, none),
        // 1 字节定长
        OpCode::CONST_U8
        | OpCode::PUSH_BOOL
        | OpCode::POP_N
        | OpCode::RESERVE_STACK
        | OpCode::RET
        | OpCode::ERROR_PROPAGATE
        | OpCode::LOAD_LOCAL
        | OpCode::STORE_LOCAL
        | OpCode::LOAD_STATE_FIELD
        | OpCode::STORE_STATE_FIELD
        | OpCode::CREATE_ARRAY
        | OpCode::CREATE_TUPLE
        | OpCode::CREATE_OK
        | OpCode::GET_GENERIC_FIELD
        | OpCode::SET_GENERIC_FIELD
        | OpCode::GET_TUPLE_FIELD => (1, none),
        OpCode::FN_PROLOG => (2, none),
        // 4 字节定长
        OpCode::CONST_I32
        | OpCode::CONST_F32
        | OpCode::JMP_FAR
        | OpCode::JMP_L
        | OpCode::CALL
        | OpCode::LOAD_REF
        | OpCode::STORE_REF
        | OpCode::LOAD_MUT_REF
        | OpCode::STORE_MUT_REF => (4, none),
        OpCode::CONST_I64 | OpCode::CONST_U64 | OpCode::CONST_F64 => (8, none),
        OpCode::SPAWN | OpCode::CREATE_GENERATOR => (5, none),
        OpCode::SLEEP | OpCode::JOIN | OpCode::SEND => (4, none),
        OpCode::SOURCE_LINE => (2, none),
        OpCode::JMP | OpCode::JMP_IF_Z | OpCode::JMP_IF_NZ | OpCode::PUSH_HANDLER => (2, none),
        // CLOSURE: u32 body_addr + u8 capture_count + u8 n_args
        //          + capture_count × (u32 名索引 + u16 slot)
        OpCode::CLOSURE => {
            let mut idxs = Vec::new();
            if pos + 5 < code.len() {
                let count = code[pos + 4] as usize;
                for k in 0..count {
                    idxs.push((6 + k * 6, IdxSpace::Str));
                }
                (6 + count * 6, idxs)
            } else {
                (6, idxs)
            }
        }
        // CREATE_FUTURE: u32 body_offset + u8 capture_count + count × u32 名索引
        OpCode::CREATE_FUTURE => {
            let mut idxs = Vec::new();
            if pos + 4 < code.len() {
                let count = code[pos + 4] as usize;
                for k in 0..count {
                    idxs.push((5 + k * 4, IdxSpace::Str));
                }
                (5 + count * 4, idxs)
            } else {
                (5, idxs)
            }
        }
        // 变长:count 字节 + count 个 tag
        OpCode::BUILD_FSTR => {
            let parts = if pos < code.len() {
                code[pos] as usize
            } else {
                0
            };
            (1 + parts, none)
        }
        // u16 len + len 字节
        OpCode::IS_VARIANT => {
            let len = if pos + 2 <= code.len() {
                u16::from_le_bytes([code[pos], code[pos + 1]]) as usize
            } else {
                0
            };
            (2 + len, none)
        }
        // name_len 来自前一条 CONST_I32
        OpCode::NEW_INSTANCE => (last_const_i32.max(0) as usize, none),
        // 无操作数
        _ => (0, none),
    }
}

/// Internal AutoVM execution function (async)
/// Plan 177: capture parameter enables stdout capture for testing
/// 池索引空间:字符串池 vs object_keys 池(两份重映射各自的关注点)。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum IdxSpace {
    Str,
    Obj,
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
            // Recursively format VmRef values.
            // Plan 390 §15 H3b: objects/arrays/nodes all live in heap_objects.
            let id = vm_ref.id as u64;
            if let Some(obj_arc) = vm.get_heap_object(id) {
                let obj = obj_arc.read().unwrap();
                if let Some(obj) = obj.as_any().downcast_ref::<crate::vm::types::ObjectData>() {
                    let fields: Vec<String> = obj
                        .fields
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, format_value_for_display(vm, v)))
                        .collect();
                    format!("{{{}}}", fields.join(", "))
                } else if let Some(arr) = obj
                    .as_any()
                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                {
                    let items: Vec<String> = arr
                        .elems
                        .iter()
                        .map(|v| format_value_for_display(vm, v))
                        .collect();
                    format!("[{}]", items.join(", "))
                } else {
                    format!("<ref:{}>", id)
                }
            } else {
                format!("<ref:{}>", id)
            }
        }
        Value::Array(arr) => {
            let items: Vec<String> = arr
                .values
                .iter()
                .map(|v| format_value_for_display(vm, v))
                .collect();
            format!("[{}]", items.join(", "))
        }
        Value::Obj(obj) => {
            let fields: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value_for_display(vm, v)))
                .collect();
            format!("{{{}}}", fields.join(", "))
        }
        _ => val.repr().to_string(),
    }
}

// ---- T-02：trans C/Rust 直译腿（纯 trans 后端，无 ui_gen）----
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
    // PLAN-738 T-03 (AC-02): declare the assembly target BEFORE any loading or
    // emission (see trans_c_with_session). The standalone re-parse below uses
    // CompileDest::TransRust — the session target must agree with it, and any
    // VM-layer bytecode preprocessing must never bind as the Rust target's
    // implementation.
    session.set_assembly_target(crate::stdlib_assembly::model::AssemblyTarget::Rust)?;
    let code = std::fs::read_to_string(path)?;
    if let Some(parent) = std::path::Path::new(path).parent() {
        session.add_source_dir(parent.to_path_buf());
    }
    session.resolve_uses(&code)?;

    // Compile source with incremental support (for dirty-tracking / caching)
    let frag_ids = session.compile_source(&code, path)?;

    // Plan 204 Phase 6A: Use .a2r.rs suffix to avoid overwriting .rs files
    let rsname = path.replace(".at", ".a2r.rs");
    if session.layer_selections.iter().any(|selection| {
        selection
            .context_file
            .as_ref()
            .is_some_and(|file| file.ends_with(".rs.at"))
    }) {
        let (output, references) = crate::stdlib_assembly::emission::emit_rust_assembly(
            session,
            &code,
            std::path::Path::new(path),
        )?;
        session.assembly_references.extend(references);
        let manifest =
            session.freeze_assembly_manifest("rust-emission", std::path::Path::new(path), &code)?;
        std::fs::write(&rsname, output)?;
        std::fs::write(
            format!("{rsname}.assembly.json"),
            serde_json::to_vec(&manifest)
                .map_err(|error| crate::error::AutoError::Msg(error.to_string()))?,
        )?;
        return Ok(format!(
            "[trans] {path} -> {rsname} (selected Rust Auto layers)"
        ));
    }

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
    // Plan 610 ⑥: relative `use.c "<file>.json"` manifests resolve against the
    // transpiled file's directory.
    trans.source_dir = std::path::Path::new(path).parent().map(|p| p.to_path_buf());
    // Plan 376U: A2R_CRATE_ROOT=1 marks this file as a crate root (lib.at /
    // */mod.at). Such files emit top-level `use` as `pub use` (re-exports) and
    // the `#![allow(...)]` crate pragma, so they can serve as an auto-generated
    // lib.rs / aggregator without hand-editing.
    if std::env::var("A2R_CRATE_ROOT")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        trans.is_crate_root = true;
        trans.emit_allow_pragma = true;
    }

    // Plan 376D: Build a shared TypeStore from ALL sibling .at files (including
    // subdirectories) so the type inference engine can resolve cross-module types.
    {
        // PLAN-010 T3: only treat the two-levels-up directory as the "crate
        // src root" when it IS literally named `src` (the repo layout this
        // heuristic was built for). For arbitrary CLI paths the old
        // unconditional parent().parent() walked unrelated trees (a sample
        // in $TMP scanned the entire temp dir) — the `auto trans` hang.
        let file_dir = std::path::Path::new(path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let src_root = match file_dir.parent() {
            Some(pp) if pp.file_name().and_then(|n| n.to_str()) == Some("src") => pp.to_path_buf(),
            _ => file_dir.clone(),
        };
        let mut type_store = crate::types::TypeStore::new();
        // Scan the source root recursively for .at files
        // PLAN-010 T3: the scan MUST be bounded. The two-levels-up "crate
        // src root" assumption makes arbitrary CLI paths (e.g. a sample in
        // $TMP) walk enormous unrelated trees parsing every .at file — the
        // `auto trans` multi-minute hang (009 DEBTS obs. #3). Bound depth,
        // file count, and skip build/VC litter.
        const SCAN_MAX_DEPTH: usize = 6;
        const SCAN_MAX_FILES: usize = 400;
        const SCAN_SKIP_DIRS: &[&str] = &[
            ".git",
            "target",
            "node_modules",
            ".wt",
            ".worktrees",
            ".cargo",
            "dist",
            "build",
            ".venv",
            "__pycache__",
        ];
        fn scan_at_files(
            dir: &std::path::Path,
            store: &mut crate::types::TypeStore,
            depth: usize,
            budget: &mut usize,
        ) {
            if depth > SCAN_MAX_DEPTH || *budget == 0 {
                return;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if *budget == 0 {
                        return;
                    }
                    let entry_path = entry.path();
                    let skipped_dir = entry_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| SCAN_SKIP_DIRS.contains(&n))
                        .unwrap_or(false);
                    if entry_path.is_dir() {
                        if !skipped_dir {
                            scan_at_files(&entry_path, store, depth + 1, budget);
                        }
                    } else if entry_path.extension().map(|e| e == "at").unwrap_or(false) {
                        // PLAN-738 T-03 (AC-02): VM/C target layers must not
                        // leak into the Rust emission type context — .vm.at /
                        // .c.at are foreign layers here. .rs.at is the Rust
                        // target's selected (mirror-signature) layer — kept,
                        // alongside the public .at files.
                        let scan_fname = entry_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");
                        if scan_fname.ends_with(".vm.at") || scan_fname.ends_with(".c.at") {
                            continue;
                        }
                        *budget -= 1;
                        if let Ok(code) = std::fs::read_to_string(&entry_path) {
                            let mut p = crate::parser::Parser::from(code.as_str());
                            p.set_dest(crate::parser::CompileDest::TransRust);
                            p.skip_check = true;
                            if let Ok(ast) = p.parse() {
                                for stmt in &ast.stmts {
                                    use crate::ast::Stmt;
                                    match stmt {
                                        Stmt::Fn(fd) => {
                                            store.register_fn_decl(fd);
                                        }
                                        Stmt::TypeDecl(td) => {
                                            store.register_type_decl(td);
                                        }
                                        Stmt::SpecDecl(sd) => {
                                            store.register_spec_decl(sd);
                                        }
                                        Stmt::EnumDecl(ed) => {
                                            store.register_enum_decl(ed.clone());
                                        }
                                        Stmt::Ext(ext) => {
                                            store.register_ext_methods(ext);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        scan_at_files(&src_root, &mut type_store, 0, &mut SCAN_MAX_FILES);
        trans.set_shared_type_store(Some(std::sync::Arc::new(std::sync::RwLock::new(
            type_store,
        ))));
    }

    // Pre-populate struct_fields from sibling .at files in the same directory.
    // This allows cross-file positional struct construction (e.g., Pos(1, 0, 0)
    // in lexer.at when Pos is defined in pos.at) to use actual field names
    // instead of falling back to field0/field1/field2.
    if let Some(parent) = std::path::Path::new(path).parent() {
        // Plan 380: recurse into subdirectories too (e.g. builtin_roles/mod.at
        // defines `load_builtin` used by src/roles.at) and pre-populate
        // fn_ret_types so the spec-bound-ident / .await heuristics see
        // cross-module return types on the single-file CLI path.
        let mut at_files: Vec<std::path::PathBuf> = Vec::new();
        let mut stack: Vec<(std::path::PathBuf, usize)> = vec![(parent.to_path_buf(), 0)];
        // PLAN-010 T3: same bounded-scan policy as the crate-root scan —
        // unbounded recursion over arbitrary CLI paths is the trans hang.
        const SIB_SCAN_MAX_DEPTH: usize = 6;
        const SIB_SCAN_MAX_FILES: usize = 400;
        const SIB_SCAN_SKIP_DIRS: &[&str] = &[
            ".git",
            "target",
            "node_modules",
            ".wt",
            ".worktrees",
            ".cargo",
            "dist",
            "build",
            ".venv",
            "__pycache__",
        ];
        while let Some((dir, depth)) = stack.pop() {
            if depth > SIB_SCAN_MAX_DEPTH || at_files.len() >= SIB_SCAN_MAX_FILES {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let ep = entry.path();
                    let skipped_dir = ep
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| SIB_SCAN_SKIP_DIRS.contains(&n))
                        .unwrap_or(false);
                    if ep.is_dir() {
                        if !skipped_dir {
                            stack.push((ep, depth + 1));
                        }
                    } else if ep.extension().map(|e| e == "at").unwrap_or(false) {
                        at_files.push(ep);
                        if at_files.len() >= SIB_SCAN_MAX_FILES {
                            break;
                        }
                    }
                }
            }
        }
        // PLAN-010 T3: cap the number and size of parsed siblings — parsing
        // hundreds of large stray .at files (e.g. in a temp dir) is the
        // multi-minute `auto trans` hang.
        const SIB_PARSE_MAX: usize = 50;
        const SIB_PARSE_MAX_BYTES: u64 = 1_000_000;
        let mut sib_parsed = 0usize;
        for entry_path in at_files {
            if sib_parsed >= SIB_PARSE_MAX {
                break;
            }
            if entry_path == std::path::Path::new(path) {
                continue;
            }
            let too_big = std::fs::metadata(&entry_path)
                .map(|m| m.len() > SIB_PARSE_MAX_BYTES)
                .unwrap_or(true);
            if too_big {
                continue;
            }
            if let Ok(sibling_code) = std::fs::read_to_string(&entry_path) {
                sib_parsed += 1;
                let mut sib_parser = Parser::from(sibling_code.as_str());
                sib_parser.set_dest(crate::parser::CompileDest::TransRust);
                sib_parser.skip_check = true;
                if let Ok(sib_ast) = sib_parser.parse() {
                    for stmt in &sib_ast.stmts {
                        match stmt {
                            crate::ast::Stmt::TypeDecl(td) => {
                                let field_names: Vec<auto_val::AutoStr> =
                                    td.members.iter().map(|m| m.name.clone()).collect();
                                if !field_names.is_empty()
                                    && !trans.struct_fields().contains_key(&td.name)
                                {
                                    trans
                                        .struct_fields_mut()
                                        .insert(td.name.clone(), field_names);
                                }
                            }
                            // Plan 380: pre-populate fn_ret_types from
                            // sibling fns (unqualified + Type.method) so
                            // the spec-bound-ident / .await heuristics
                            // work on the single-file CLI path.
                            crate::ast::Stmt::Fn(fd) => {
                                let fname: auto_val::AutoStr = fd.name.clone();
                                if !trans.fn_ret_types_mut().contains_key(&fname) {
                                    trans.fn_ret_types_mut().insert(fname, fd.ret.clone());
                                }
                                if let Some(p) = &fd.parent {
                                    let qualified: auto_val::AutoStr =
                                        format!("{}.{}", p, fd.name).into();
                                    if !trans.fn_ret_types_mut().contains_key(&qualified) {
                                        trans.fn_ret_types_mut().insert(qualified, fd.ret.clone());
                                    }
                                }
                            }
                            // Plan 371 (defect A): pre-populate spec names from
                            // sibling .at files so cross-module specs (e.g.
                            // `use role_def: Role` then `role Role`) resolve to
                            // Type::Spec (-> Box<dyn X>) on the single-file path.
                            crate::ast::Stmt::SpecDecl(sd) => {
                                let name = sd.name.clone();
                                if !trans.spec_decls_mut().contains_key(&name) {
                                    trans.spec_decls_mut().insert(name, sd.methods.clone());
                                }
                            }
                            // Plan 372 follow-up: pre-populate enum names from
                            // sibling .at files so cross-module enum errors like
                            // `Err(AgentError::Config(...))` don't get wrongly
                            // Box::new'd (AgentError lives in error.at, not the
                            // file being transpiled).
                            crate::ast::Stmt::EnumDecl(ed) => {
                                trans.known_enum_names_mut().insert(ed.name.clone());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        // Plan 372 follow-up: also scan the grandparent directory (one level up)
        // to catch enums/specs defined in sibling directories (e.g. error.at in
        // src/ when transpiling orchestration/driver.at in src/orchestration/).
        if let Some(grandparent) = parent.parent() {
            // PLAN-010 T3: same parse budget/size guard as the sibling scan.
            const GP_PARSE_MAX: usize = 20;
            const GP_PARSE_MAX_BYTES: u64 = 1_000_000;
            let mut gp_parsed = 0usize;
            if let Ok(entries) = std::fs::read_dir(grandparent) {
                for entry in entries.flatten() {
                    if gp_parsed >= GP_PARSE_MAX {
                        break;
                    }
                    let entry_path = entry.path();
                    if entry_path.extension().map(|e| e == "at").unwrap_or(false) {
                        if entry_path == std::path::Path::new(path) {
                            continue;
                        }
                        let too_big = std::fs::metadata(&entry_path)
                            .map(|m| m.len() > GP_PARSE_MAX_BYTES)
                            .unwrap_or(true);
                        if too_big {
                            continue;
                        }
                        if let Ok(sibling_code) = std::fs::read_to_string(&entry_path) {
                            gp_parsed += 1;
                            let mut sib_parser = Parser::from(sibling_code.as_str());
                            sib_parser.set_dest(crate::parser::CompileDest::TransRust);
                            sib_parser.skip_check = true;
                            if let Ok(sib_ast) = sib_parser.parse() {
                                for stmt in &sib_ast.stmts {
                                    match stmt {
                                        crate::ast::Stmt::EnumDecl(ed) => {
                                            trans.known_enum_names_mut().insert(ed.name.clone());
                                        }
                                        crate::ast::Stmt::SpecDecl(sd) => {
                                            let name = sd.name.clone();
                                            if !trans.spec_decls_mut().contains_key(&name) {
                                                trans
                                                    .spec_decls_mut()
                                                    .insert(name, sd.methods.clone());
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    trans.trans(ast, &mut sink)?;

    // Plan 355: Apply the same post-processing fixes that the in-process
    // `transpile_rust()` path applies. Without this, the `auto trans` CLI
    // emitted raw code with known-correctable defects (e.g. `return None;`
    // in void fns, double semicolons after `static` decls), so a2r test
    // binaries built from CLI-transpiled sources failed to compile while the
    // identical code compiled fine when transpiled via `transpile_rust()`.
    // PLAN-009 T1 (F1): thread the restricted-name set so fix_non_ord_derives
    // never re-widens AST-restricted derives on the CLI path either.
    crate::trans::rust::RustTrans::post_process_with(&mut sink.body, &trans.ord_restricted_names());

    // Write output file
    session
        .assembly_references
        .extend(trans.assembly_references.iter().cloned());
    let manifest =
        session.freeze_assembly_manifest("rust-emission", std::path::Path::new(path), &code)?;
    let source_bytes = sink.done()?;
    if !source_bytes.is_empty() {
        std::fs::write(&rsname, source_bytes)?;
        std::fs::write(
            format!("{rsname}.assembly.json"),
            serde_json::to_vec(&manifest)
                .map_err(|error| crate::error::AutoError::Msg(error.to_string()))?,
        )?;

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
    // PLAN-738 T-03 (AC-02): declare the assembly target BEFORE any loading or
    // emission. This entry has a zero-module-load surface today (decision
    // report E1④); the explicit target guarantees any future resolve_uses
    // consumer on this session selects the C layer — never the VM layer — as
    // its type context.
    session.set_assembly_target(crate::stdlib_assembly::model::AssemblyTarget::C)?;
    let code = std::fs::read_to_string(path)?;
    if let Some(parent) = std::path::Path::new(path).parent() {
        session.add_source_dir(parent.to_path_buf());
    }
    session.resolve_uses(&code)?;

    // Compile source with incremental support
    let frag_ids = session.compile_source(&code, path)?;

    let artifacts = crate::stdlib_assembly::emission::emit_c_assembly(
        session,
        &code,
        std::path::Path::new(path),
    )?;
    let manifest =
        session.freeze_assembly_manifest("c-emission", std::path::Path::new(path), &code)?;
    let output = std::path::Path::new(path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    // All emission/strict checks finish before any generated artifact is written.
    for artifact in &artifacts {
        std::fs::write(
            output.join(format!("{}.c", artifact.stem)),
            &artifact.source,
        )?;
        std::fs::write(
            output.join(format!("{}.h", artifact.stem)),
            &artifact.header,
        )?;
    }
    std::fs::write(
        output.join(format!(
            "{}.assembly.json",
            std::path::Path::new(path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
        )),
        serde_json::to_vec(&manifest)
            .map_err(|error| crate::error::AutoError::Msg(error.to_string()))?,
    )?;
    Ok(format!(
        "[trans] {} -> {} ({} fragments, {} C artifacts)",
        path,
        path.replace(".at", ".c"),
        frag_ids.len(),
        artifacts.len()
    ))
}

/// Transpile AutoLang file to Rust
///
/// **Plan 091**: Now uses CompileSession internally (no Universe dependency)
pub fn trans_rust(path: &str) -> AutoResult<String> {
    let mut session = CompileSession::new();
    trans_rust_with_session(&mut session, path)
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
pub fn trans_c(path: &str) -> AutoResult<String> {
    let mut session = CompileSession::new();
    trans_c_with_session(&mut session, path)
}

// ---- T-02：vm/abt/assembly 测试支持根项 ----
/// Plan 204 Phase 6B: Basic structural validation of transpiled Rust output.
///
/// Checks bracket matching and basic structural validity.
/// Logs warnings if validation fails but does not block writing.
pub fn validate_rust_output(path: &str, content: &str) {
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

/// Plan 226 Phase 6: Create an AutoVM from ABT text, ready to run.
///
/// Parses ABT text, assembles it to bytecode, and constructs a VM with
/// standard native shims (print, assert, etc.) loaded.
///
/// Returns `(vm, stdout_capture, main_entry_point)`.
pub fn create_vm_from_abt(
    abt_text: &str,
) -> AutoResult<(
    crate::vm::engine::AutoVM,
    std::sync::Arc<std::sync::RwLock<String>>,
    usize,
)> {
    crate::install_ui_host_capabilities();
    use crate::vm::abt::{asm, parser};
    use crate::vm::engine::AutoVM;
    use crate::vm::native::NativeInterface;
    use crate::vm::virt_memory::VirtualFlash;

    // 1. Parse ABT text
    let abt_program = parser::parse(abt_text)
        .map_err(|e| crate::error::AutoError::Msg(format!("ABT parse error: {}", e)))?;

    // 2. Assemble to CompiledPackage
    let pkg = asm::assemble(&abt_program)
        .map_err(|e| crate::error::AutoError::Msg(format!("ABT assemble error: {}", e)))?;

    // 3. Create VirtualFlash from CompiledPackage
    let flash = VirtualFlash::from_vec_with_metadata(
        pkg.bytecode,
        pkg.exports.clone(),
        pkg.object_keys,
        pkg.object_types,
    );

    // 4. Ensure built-in natives are registered (normally done by TypeStore/CompileSession)
    crate::vm::native_registry::register_builtin_natives();

    // 5. Create VM with stdout capture
    let (mut vm, output_buffer) = AutoVM::new_with_capture(flash, 8192);

    // 6. Load string pool
    vm.load_strings(pkg.string_pool);

    // 7. Register standard native shims + all stdlib FFI
    let mut ni = NativeInterface::new();
    ni.register_std_shims();
    crate::vm::ffi::stdlib::register_stdlib_ffi(&mut ni);
    vm.merge_native_interface(&ni);

    // 8. Determine entry point (main export, or start of bytecode)
    let main_entry = pkg
        .exports
        .get("main")
        .or_else(|| pkg.exports.get("test"))
        .copied()
        .unwrap_or(0) as usize;

    Ok((vm, output_buffer, main_entry))
}

/// Plan 225: Compile source and create a VM with stdout capture ready for debugging.
/// The caller is responsible for setting the debugger controller, spawning, and running tasks.
/// Returns (vm, stdout_capture_buffer, entry_point, result_type)
pub fn create_vm_from_source(
    code: &str,
) -> AutoResult<(
    crate::vm::engine::AutoVM,
    std::sync::Arc<std::sync::RwLock<String>>,
    usize,
    crate::vm::codegen::ObjectType,
)> {
    crate::install_ui_host_capabilities();
    use crate::vm::codegen::Codegen;
    use crate::vm::engine::AutoVM;
    use crate::vm::loader::Linker;
    use crate::vm::opcode::OpCode;
    use crate::vm::virt_memory::VirtualFlash;

    // Plan 442 C2: axum serve adapter state reset (see execute_autovm_with_path).
    crate::vm::ffi::axum_adapter::reset();
    // PLAN-669: #[api] param-sig side channel — same reset discipline.
    crate::vm::ffi::http_server::clear_api_param_sigs();

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
    let result_type = codegen.last_expr_type.clone();
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

    let flash = VirtualFlash::from_vec_with_metadata(
        linked_code,
        global_symbols,
        object_keys,
        object_types,
    );

    let (mut vm, output_buffer) = AutoVM::new_with_capture(flash, 8192);
    vm.load_strings(strings);
    vm.load_generic_registry(generic_registry);
    vm.load_task_handler_registry(task_handler_registry); // Plan 327 Phase 1
                                                          // Plan 312: Register #[api] routes for HTTP server dispatch
    crate::vm::ffi::stdlib::register_http_routes(api_routes);

    if let Some(rust_ni) = rust_ffi_native_interface {
        vm.merge_native_interface(&rust_ni);
    }
    if let Some(py_ni) = py_ffi_native_interface {
        vm.merge_native_interface(&py_ni);
    }

    Ok((vm, output_buffer, main_entry, result_type))
}

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

/// PLAN-675: 注释剥离（store 依赖启发式扫描的预处理，lib.rs/api.rs 两处
/// 扫描共用）——`//` 行注释与 `/* */` 块注释整体移除，换行保留；双引号
/// 字符串面内的 `//`、`/*` 不触发（.at 字符串字面量双引号定界）。
pub fn strip_line_block_comments(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let mut chars = code.chars().peekable();
    let mut in_str = false;
    let mut str_escape = false;
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if str_escape {
                str_escape = false;
            } else if c == '\\' {
                str_escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                out.push(c);
            }
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                for n in chars.by_ref() {
                    if n == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                while let Some(n) = chars.next() {
                    if n == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                    if n == '\n' {
                        out.push('\n');
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

// Plan 351: thread-local for passing store composable files out-of-band
// (avoids changing the return type of ui_build_* functions). PLAN-814 T-03：
// 单源在此（UI 编译器 api.rs 扫描消费）；auto-lang 旧路径经再导出保持。
thread_local! {
    pub static STORE_EXTRA_FILES: std::cell::RefCell<Vec<(String, String)>> =
        std::cell::RefCell::new(Vec::new());
}

/// Drain and return stashed store composable files (filename, content).
pub fn drain_store_extra_files() -> Vec<(String, String)> {
    let result = STORE_EXTRA_FILES.with(|cell| std::mem::take(&mut *cell.borrow_mut()));
    result
}
