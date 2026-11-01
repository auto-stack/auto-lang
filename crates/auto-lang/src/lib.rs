// Plan 249: unified native catalog macro expansion needs higher recursion limit
#![recursion_limit = "512"]
pub use auto_lang_core::{
    block_on_autovm_local, bootstrap_lib_source, execute_autovm,
    execute_autovm_with_deadline_and_input, execute_autovm_with_path, extract_autovm_result,
    extract_undefined_symbol, find_project_root, find_use_symbol_span, get_error_limit,
    get_global_runtime, init_py_ffi, init_rust_ffi, is_vm_debug, is_vm_hot_trace, is_vm_trace_ops,
    join_execution, p510_audit, pac_declares_dep, pac_dep_version_violation,
    pac_workspace_member_dir, parse, parse_preserve_error, pool_log_all, pool_log_seq,
    pool_trace_idx, read_auto_lib, resolve_module_path, run_abt, run_with_capture_and_bytecode,
    run_with_capture_and_bytecode_with_deadline, run_with_capture_and_bytecode_with_input,
    run_with_capture_and_bytecode_with_meta, run_with_capture_and_path_and_bytecode,
    run_with_capture_and_path_and_bytecode_with_deadline,
    run_with_capture_and_path_and_bytecode_with_input,
    run_with_capture_and_path_and_bytecode_with_meta, script_args, set_error_limit,
    set_external_back_root, set_script_args, set_ui_host_install_hook, set_vm_debug,
    store_name_snake, trans_c, trans_c_with_session, trans_rust, trans_rust_with_session,
    validate_rust_output, vm_thread_stack_size,
};

// Plan 565 P0: counting global allocator for memory attribution — installed
// only with `--features mem-profile`; without the feature the System
// allocator stays in place (zero cost, no extra symbols).
#[cfg(feature = "mem-profile")]
#[global_allocator]
static MEM_PROFILE_ALLOC: mem_profile::CountingAlloc = mem_profile::CountingAlloc;

// Plan 351: store composable files 越带外传递（ui_build_* 返回型不变）——
// PLAN-814 T-03：thread_local 与 drain 单源下沉 SDK（UI 编译器消费），
// 旧路径经再导出保持。
pub use auto_lang_core::{drain_store_extra_files, STORE_EXTRA_FILES};

// PLAN-664 U-4：use 导入诊断汇聚点（use_diag/take_use_diags + USE_DIAGS
// thread_local）。PLAN-814 T-04：随 UI 动态构建管线消费方迁 auto-ui-runtime
// 而下沉 SDK 单源（与 STORE_EXTRA_FILES 同族带外通道）；旧路径经再导出保持。
pub use auto_lang_core::{take_use_diags, use_diag};
// PLAN-814 T-06：#[test] 执行入口族下沉 SDK（纯 Core 面；宿主装配经 SDK
// 钩子分发器），旧 crate 根路径经再导出保持。
pub use auto_lang_core::{
    test_code, test_code_with_deadline_and_input, test_code_with_path_deadline_and_input, test_file,
};

pub mod api {
    // PLAN-906: Web 生成器（Axum/TypeScript）不再经语言 facade 转发——四仓
    // 零外部消费者，Man/UI/生成工程均直连 auto_web::api。Target 回归 core
    // 单源（Tauri-only，经下方 glob 再导出，`auto_lang::api::Target` 路径不变）。
    pub use auto_lang_core::api::*;
}
use auto_lang_core::ast;
// PLAN-799: 解析便利层（ParserExt 对 Code/Name/Expr/Is/Arrow/OnEvents 的
// 六类 impl + Parser 耦合测试）——模型已在 auto-ast，解析注入留上层。
// PLAN-814 R3-06/R5：crate 内测试消费面（cfg(test) 门——非测试构建下的
// 未用 import 告警为假信号，R5 口径按消费方条件声明）。
#[cfg(test)]
mod ast_parse_ext {
    pub use auto_lang_core::ast_parse_ext::*;
}
pub use auto_lang_core::atom;
// PLAN-814: 实现随前端门面组迁 auto-lang-core；本 crate 保持私有（Lexer
// 不是 auto-lang 公开面——PLAN-789 合同），消费方 crate::lexer::X 不变
// （cfg(test) 门同上）。
#[cfg(test)]
mod lexer {
    pub use auto_lang_core::lexer::*;
}
// a2r Standard Library - Rust implementations of AutoLang standard types
pub use auto_lang_core::a2r_std;
// Plan 084: Unified TypeStore for type declaration management
pub use auto_lang_core::types;
// Plan 091: AutoVM-based interpreter interface
pub use auto_lang_core::interpreter;
// Plan 090: Parser helpers to remove Universe dependency
pub use auto_lang_core::parser_helpers;
// Plan 085: Use statement scanner for AIE + AutoCache
// PLAN-814: 单一实现迁 crates/auto-lang-core（纯 SDK 首个机制切片）；
// 旧路径 auto_lang::use_scanner::* 经再导出保类型身份（同 crate origin）。
pub use auto_lang_core::use_scanner;
// Plan 092: Dep statement scanner for Rust FFI
pub use auto_lang_core::dep_scanner;
// Plan 085 Phase 5: Module cache for incremental compilation
pub use auto_lang_core::atom_error;
pub use auto_lang_core::autovm_client; // Plan 269: AutoVM client for req command
pub use auto_lang_core::autovm_daemon; // Plan 269: AutoVM daemon (serve/req)
pub use auto_lang_core::autovm_persistent; // Plan 068 Phase 9.6: Persistent AutoVM REPL
pub use auto_lang_core::back_prefix; // PLAN-037: launch 期作用域化模块源前缀化 overlay（桌面后端供给——un-gated，模块读点在本文核心装载链）
pub use auto_lang_core::compile;
pub use auto_lang_core::config;
pub use auto_lang_core::database;
pub use auto_lang_core::dep;
use auto_lang_core::error;
pub use auto_lang_core::host; // Plan 011 (MS3-B): ShellHost bridge for system()/exit()/export()
pub use auto_lang_core::mcp; // Plan 265: MCP server for AI agent interaction
pub use auto_lang_core::module_cache;
pub use auto_lang_core::stdlib_assembly;
// PLAN-906: http_file_service 执行面归 auto_web 直连（本 crate 零消费者；a2r-std 描述符单源不变）。
pub use auto_lang_core::http_service_config; // PLAN-906: 合同单源迁语言仓后改指 core（合同+槽函数不变；RateLimiter/ServiceRuntime 留 auto_web）
// PLAN-906: http_service_observability 归 auto_web 直连（本 crate 零消费者）。
// PLAN-906: http_upload_service 归 auto_web 直连（本 crate 零消费者）。
                                                                                                               // PLAN-738: stdlib 装配契约（inventory/manifest/provider 目录纯模型层）
                                                                                                               // Plan 096 Phase 0: Scenario-based compilation
                                                                                                               // Plan 768: AutoDown TestCase adapters (avm/run, a2r/emit) + runner
                                                                                                               // orchestration; depends on the dependency-light `auto-test-case` core.
use auto_lang_core::session;
pub use auto_lang_core::test_cases;
// PLAN-900：aura/ui_ast/a2ui/design_tokens/ui_gen/generator_fingerprint
// 再导出随去 auto-ui 依赖退役——单源在 auto-ui-compiler（crate 根路径
// auto_ui_compiler::* 自 PLAN-814 起即正式公开面）。
// Stage B P-5（PLAN-590）：跨仓解析序定位 auto-os 侧资产（无 feature 门——
// ui_gen/docs 管线与 CLI 的无 ui 构建形态也要消费）。
pub use auto_lang_core::os_paths;
// PLAN-044：用户级状态文件写安全（原子替换 + 跨进程锁；键级合并写在
// vm::ffi::stdlib::storage_persist，锁路径跨仓约定单源在模块头）。
pub use auto_lang_core::state_file;
// Plan 152: Server-Sent Events (SSE) 解析
pub use auto_lang_core::sse;
// PLAN-900：route 再导出随去 auto-ui 依赖退役（单源 auto-ui-compiler::route）。
// Plan 081 Phase 2: Execution mode selection (autovm, evaluator, c, rust)
pub use auto_lang_core::mode;
// Plan 081 Phase 4: Multi-mode compilation pipeline
pub use auto_lang_core::multi_mode;
// Plan 081 Phase 5: FFI layer for cross-mode function calls
pub use auto_lang_core::ffi;
// Plan 222: Python FFI type definitions (no pyo3 dependency)
pub use auto_lang_core::py_ffi_types;
// Plan 214: Python FFI via PyO3 (optional, requires `--features python`)
#[cfg(feature = "python")]
pub use auto_lang_core::py_ffi;
// Plan 073 Phase 9.3: Execution engine selection (AutoVM vs Evaluator)
pub use crate::infer::InferenceContext;
pub use crate::type_registry::SharedTypeRegistry;
pub use auto_lang_core::execution_engine;
pub use auto_lang_core::hash;
pub use auto_lang_core::indexer;
pub use auto_lang_core::infer;
pub use auto_lang_core::libs;
pub use auto_lang_core::macro_;
pub use auto_lang_core::maker;
// Plan 565 P0: counting allocator + stats (only with mem-profile feature)
#[cfg(feature = "mem-profile")]
pub mod mem_profile;
pub use auto_lang_core::dialect;
pub use auto_lang_core::ownership;
use auto_lang_core::parser;
pub use auto_lang_core::query;
pub use parser::Parser; // 方言扩展体系（PR-1 基建）
                        // PLAN-778: 方言外部化最小桥面 —— 外部 crate 实现 Dialect 时的唯一解析入口。
pub use auto_lang_core::parser_bridge;
pub use dialect::DialectPriority;
pub use parser_bridge::ParserBridge;
// Plan 088 Phase 6: Type checking module for parameter passing modes
pub use auto_lang_core::autovm_repl;
pub use auto_lang_core::patch;
pub use auto_lang_core::typeck;
// Plan 091: repl.rs deleted - use autovm_persistent::AutovmReplSession instead
// Plan 078: ModuleResolver trait for dependency resolution
pub use auto_lang_core::resolver;
pub use auto_lang_core::runtime;
pub use auto_lang_core::scope;
pub use auto_lang_core::scope_manager;
pub use auto_lang_core::symbols;
pub use auto_lang_core::target;
pub use auto_lang_core::token;
pub use auto_lang_core::trait_checker;
// Plan 087: Type registry for REPL
pub use auto_lang_core::trans;
pub use auto_lang_core::type_registry;
// Plan 109: AutoDown Document Format
pub mod autodown;
pub use auto_lang_core::util;
pub mod vm {
    pub use auto_lang_core::vm::*;
    pub mod ffi {
        pub use auto_lang_core::vm::ffi::*;
    // PLAN-906: vm::ffi 的 http_server/http_transport Web 执行再导出移除（零消费者；
    // Core 中立 host port 与 stdlib http 面不变，Web 侧原文不动）。
    }
}
// Plan 095: Compile-Time Execution Engine
pub use auto_lang_core::comptime;
// Plan 260: Test framework
pub use auto_lang_core::test_runner;

pub use atom::{Atom, AtomReader};

// 过程宏 - 支持 AutoLang 语法的内嵌 DSL
// 这些宏接受 AutoLang 代码字符串并解析为 Atom/Node/Value 结构体
pub use auto_macros::{atom, node, value};

// Plan 091: AutoVM-based interpreter (replacement for eval.rs/interp.rs)
pub use interpreter::AutoInterpreter;

use crate::compile::CompileSession;
pub use crate::symbols::SymbolLocation;
use auto_val::Obj;
use std::path::Path;

use crate::error::AutoResult;

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
/// PLAN-779: 内建宿主能力装配（composition root——crate 根是 ui→vm 单向
/// 接线的合法层；vm core 不引用 ui）。ui 构建下装配 frame_bench adapter
/// 为**兼容默认**（F779-R2-01：仅空槽补缺、同锁原子，不覆盖调用方显式
/// set 的默认）：恢复 779 迁移前"本 crate 全部生产执行路径在 ui 构建中
/// 可观测帧通道"的既有接线（F779-R1-01——程序轨 run 管线、测试/调试/
/// ABT/配置/REPL/解释器/back-proxy 各宿主入口统一经此装配）。
/// no-ui 构建=no-op（缺席=未捕获 0 语义，外部 consumer 自行注入）。
/// PLAN-785: clipboard text adapter 同纪律并入（2925/2926——core shim
/// 已不直引 ui::clipboard；缺席槽=no-ui 迁移前 VMError 语义）。
/// PLAN-790: clipboard media adapter 同纪律并入（2934..2937——core shim
/// 已不直引 ui::clipboard_native；windows+native-clipboard 下恢复 OS 接线，
/// 其余形态缺席槽=G3 值降级，非 Windows 不伪造可用 OS）。
/// PLAN-794: scroll host adapter 同纪律并入（9900..9905 + 9922/9923——
/// core 八 shim 已不直引 ui::scroll；ui 构建恢复 controller 注册表接线，
/// 缺席槽=T-01 冻结降级臂）。
/// PLAN-802: code editor host adapter 同纪律并入（22 native 全族——core
/// shim 已不直引 ui::code_editor；code-editor 构建恢复 editor 注册表
/// 接线，缺席槽=802 T-01 冻结缺席臂）。
/// PLAN-804: 两槽并入（2927/2928 + 2938）——本入口的门同时扩为
/// `any(ui, ui-dialog)`：dialog adapter 走 ui-dialog 单门控，**ui-dialog
/// only（无 ui）构建恢复迁移前高层对话框路径**（不把 OS 实现藏在 ui
/// renderer 门后）；dnd adapter 走 windows+native-dnd 双门控（模块本身
/// 在 ui 门内=三分），缺席槽=false/VMError 同旧。

/// SDK 钩子回调臂（旧=本包安装器自注册）：provider 实装委托 runtime
/// 单点。runtime 无 renderer 构建时其安装器为 no-op，语义与旧 no-op
/// 安装器一致。（PLAN-841 T-03：文法重装臂随函数表退役删除。）


// PLAN-814 R3-01/R3-04：上层装配的进程期自注册（SDK 收集点
// UiCompatInstaller——本 crate 被链接即生效；覆盖不经漏斗的直达调用：
// 旧公开入口/native dispatch 的槽未设路径）。非 inventory 依赖添加：
// SDK 侧已依赖，本 crate 经 re-export 面使用。
// PLAN-815 T-04：注释剥离单源在 SDK——根级路径保持（store-deps 两路径
// 随 PLAN-900 去 auto-ui 依赖退役，单源 auto_ui_compiler::store_deps）。
pub use auto_lang_core::strip_line_block_comments;

// PLAN-900：UiCompatInstaller 进程期收集点随宿主能力装配迁
// auto-ui-runtime（其 install 单点自注册 SDK 钩子——本 crate 零 auto-ui
// 依赖后不再承担装配扇入）。

// PLAN-841 T-01/T-03：UiGrammarSource inventory 收集点随文法函数表退役
// 删除——"UI 文法恒可用"的旧默认行为由显式领域端口（parse_domain）取代；
// Core 同名语义正反控见 auto-lang-core parser 测试（plan841_*）。

// ---- PLAN-814 R3-01：旧公开高层入口的兼容装配包装 ----
// 这四个入口在 SDK 侧经钩子回调本包安装器；包装确保消费者首次直达调用
// （未经 run 家族漏斗）也先注册钩子再进 SDK 实现。签名与旧行为逐位一致。

/// 兼容包装：委托 SDK 实现（PLAN-900 后装配扇入归 auto-ui-runtime install 单点）。
pub fn create_vm_from_source(
    source: &str,
) -> AutoResult<(
    auto_lang_core::vm::engine::AutoVM,
    std::sync::Arc<std::sync::RwLock<String>>,
    usize,
    auto_lang_core::vm::codegen::ObjectType,
)> {
    auto_lang_core::create_vm_from_source(source)
}

/// 兼容包装：委托 SDK 实现（PLAN-900 后装配扇入归 auto-ui-runtime install 单点）。
pub fn create_vm_from_abt(
    abt_text: &str,
) -> AutoResult<(
    auto_lang_core::vm::engine::AutoVM,
    std::sync::Arc<std::sync::RwLock<String>>,
    usize,
)> {
    auto_lang_core::create_vm_from_abt(abt_text)
}

/// 兼容包装：委托 SDK 实现（PLAN-900 后装配扇入归 auto-ui-runtime install 单点）。
pub fn eval_config_with_vm(code: &str, args: &auto_val::Obj) -> AutoResult<auto_val::Value> {
    auto_lang_core::eval_config_with_vm(code, args)
}

/// Plan 327: execute_autovm with an optional source file path.
/// When provided, the source directory is added to CompileSession.source_dirs
/// so that `use db` can resolve db.at relative to the source file (true
/// multi-module loading, not string flattening).
/// PLAN-746 (PG-MEM-1): like [`execute_autovm_with_path`] but installs a
/// cooperative execution deadline on the VM. Once the deadline passes,
/// run_task_loop terminates all tasks with a synthetic "ExecutionTimeout"
/// error instead of letting a runaway program spin forever.

/// Auto library files to prepend for AAVM bootstrap tests.
/// Single source of truth — vm_file_tests.rs reuses this list (plan-429 A3).
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
/// # Ok::<(), auto_lang_core::error::AutoError>(())
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

pub fn run_file(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::run_file(path)
}

/// Plan 555 T07: s2s 改写——Auto 脚本糖源 → 正常模式桥源
/// （W1 规则表空置=identity passthrough；W2 糖批逐条落
/// trans::auto_s2s::builtin_rules）。
pub fn trans_auto_s2s(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::trans_auto_s2s(path)
}

/// Plan 555 T08: `--dump-lowered` 渲染——模式头 + 改写产物（run/trans
/// 双入口共用）。W1 产物 = passthrough 源；模式头按扩展名解析
/// （pragma 属解析期信号，离线 dump 按扩展名档位）。
pub fn dump_lowered(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::dump_lowered(path)
}

/// Plan 524: run a script with CLI pass-through args (`auto <file> [args...]`
/// 直跑透传形态)。Args land in the process-wide `process.args()` 表 =
/// [程序路径] + args。每次调用覆盖前值（同进程串行多次运行不残留）。
pub fn run_file_with_args(path: &str, args: Vec<String>) -> AutoResult<String> {
    // PLAN-900：动态 UI 分岔（has_ui_keywords → iced 宿主直跑）随去
    // auto-ui 依赖退役——UI 动态执行单源在 auto-ui 侧（auto-ui-tools
    // run_vm 同语义复刻）；本入口退化为纯语言执行委托。
    set_script_args(args.clone());
    auto_lang_core::script_entry::run_file_with_args(path, args)
}

// PLAN-816 T-05：私有 helper `run_with_path` 单源迁 auto_lang_core::script_entry（本包公开入口委托）。

/// Plan 199 Phase 5: Debug an Auto program with GDB-style interactive debugger
pub fn debug_file(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::debug_file(path)
}

// PLAN-816 T-05：私有 helper `debug_autovm` 单源迁 auto_lang_core::script_entry（本包公开入口委托）。

/// Plan 199 Phase 8: Debug an Auto program with JSON agent-mode controller.
/// Each pause emits a JSON state line to stdout; the driver reads commands
/// from stdin as simple strings (continue / step / step_over / step_out / stop).
pub fn debug_file_agent(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::debug_file_agent(path)
}

// PLAN-816 T-05：私有 helper `debug_autovm_agent` 单源迁 auto_lang_core::script_entry（本包公开入口委托）。

/// Transpile AutoLang file to C (legacy implementation)

/// Transpile AutoLang file to Rust (legacy implementation)

// =============================================================================
// Phase 066: Incremental Transpilation API (with CompileSession)
// =============================================================================

/// Transpile a multi-file AutoLang project into a single merged Rust file.
///
/// Discovers all .at files starting from `entry_file`, transpiles them
/// into one .rs file with deduplication and merge-specific post-processing.
pub fn trans_rust_merged(entry_file: &str) -> AutoResult<Vec<u8>> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::trans_rust_merged(entry_file)
}

/// Transpile AutoLang file to Python
pub fn trans_python(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::trans_python(path)
}

/// Transpile AutoLang file to JavaScript
pub fn trans_javascript(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::trans_javascript(path)
}

/// Transpile AutoLang file to GDScript (Plan 290: a2gd)
pub fn trans_gdscript(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::trans_gdscript(path)
}

/// Transpile an AutoLang scene declaration to a Godot `.tscn` file (Plan 306).
///
/// Reads `path` (an `.at` file containing a `scene` declaration), parses it,
/// finds the `SceneDecl`, and writes a `.tscn` file next to the source.
pub fn trans_tscn(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::trans_tscn(path)
}

/// Transpile an AutoLang file to a full Godot node: emit BOTH the `.tscn`
/// scene (from any `scene` declaration) and the `.gd` script (from the rest).
///
/// Plan 306 Phase 2b. A single `.at` file may describe a scene tree and the
/// game logic that drives it; this produces the two Godot files it maps to.
/// The scene's `script = "name.gd"` reference ties the generated script to
/// the scene root.
pub fn trans_godot(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::trans_godot(path)
}

/// Transpile AutoLang file to TypeScript (Plan 100: a2js → a2ts)
pub fn trans_typescript(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::trans_typescript(path)
}

/// Transpile AutoLang file to TypeScript with custom output path
pub fn trans_typescript_to(path: &str, output: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::trans_typescript_to(path, output)
}

/// Transpile Rust file to AutoLang (Plan 173: r2a)
pub fn transpile_r2a_file(path: &str) -> AutoResult<String> {
    // PLAN-816 T-05：单源在 SDK（auto_lang_core::script_entry）；本包为兼容转发。
    auto_lang_core::script_entry::transpile_r2a_file(path)
}

// ============================================================================
// PLAN-900：UI 解析/生成管线装配（ui_build 族）整体迁 auto-ui-compiler
// `pipeline` 模块（依赖方向反转，Design 52 §2.1）。旧 auto_lang::ui_build*
// 公开路径随本 crate 去 auto-ui 依赖一并退役；消费方直连
// auto_ui_compiler::pipeline（组合层）或 ui_gen::generate_component_from_file。
// ============================================================================
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
            // Plan 364 Step 5: use the unified Codegen in config_mode
            // (replaces the old compile-time-only ConfigCodegen).
            let mut codegen = Codegen::new_for_config();
            codegen.compile_config_program(&code, &Obj::new())?;
            codegen.finish("config".to_string())
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
pub(crate) mod tests; // PLAN-726 T-02: heavy_gate 机器闸门跨档共用(crate::vm churn 族引用)

// Plan 370 shared test support（build_component_from_app 等）：实装与全部
// 消费测试随 ui/ 迁 auto-ui-runtime::ui_tests（PLAN-814 T-04）。

// =============================================================================
// Plan 015: AutoUI Core (feature-gated)
// =============================================================================
// PLAN-814 T-04：ui/ 运行域整体迁 auto-ui-runtime；旧路径 auto_lang::ui::*
// 经再导出保类型身份（crate::ui::x 引用零改写）。
// PLAN-900：ui 运行域再导出随去 auto-ui 依赖退役——单源在
// auto-ui-runtime（crate::ui / 根再导出面）。

// PLAN-804: 宿主 OS 能力适配层（vm core 与 ui renderer 门之外的第三
// 落点——首员 = ui-dialog 单门控的 OS 文件对话框）。crate 根是宿主接线
// 的合法层；vm core 经 vm::host_capabilities provider 接口消费，不直引
// 本层。PLAN-814 T-04 随 UI 运行域迁 auto-ui-runtime。
// PLAN-900：host_adapters 再导出退役（单源 auto-ui-runtime::host_adapters）。

// PLAN-814 T-04：UI 动态构建管线兼容再导出（原 lib.rs 根项；实装随迁
// auto-ui-runtime::dynamic_build——run 族漏斗/CLI/examples/测试树旧路径保持）。
// Plan 446 批一 (C1-2) parse 错误定位换算回归：随 positioned_parse_errors
// 实装迁 auto-ui-runtime::dynamic_build（PLAN-814 T-04）。

// Plan 510 / P499-7: native ID 撞号(Log×Shell 1800-1803)回归钉 +
// 字符串池 over-release 记账回归。
#[cfg(test)]
#[path = "tests/plan510_pool_tests.rs"]
mod plan510_pool_tests;

// PLAN-608 T05：CALL_SPEC 内联分发区不可达臂路由守护 + 未知方法兜底
// 行为金样（registry 覆盖回退使死臂复活时先红——push 臂=UAF 面）。
#[cfg(test)]
#[path = "tests/plan608_dispatch_golden_tests.rs"]
mod plan608_dispatch_golden_tests;

// Plan 394：External future 真挂起架构（Phase A 门禁 + B/C 骨架）。
#[cfg(test)]
#[path = "tests/plan394_future_arch_tests.rs"]
mod plan394_future_arch_tests;

// os-007 avatar 家族渲染探针随 PLAN-900 迁 auto-ui（ui::dynamic 域）。
// os-007 breadcrumb 折叠环守卫随 PLAN-900 迁 auto-ui（ui::dynamic 域）。

// PLAN-614 临时探针:VM 轨模块 fn 对 Obj 嵌套 children/List/bool 参数行为
// 隔离(tree 组件族 VM 端 first-light 排障;定位后可退役)。

// PLAN-618 探针:VM 轨模块 fn 递归(filter_tree)能力验证。

// PLAN-906: plan707 真 TCP relay e2e 随 http e2e 家族迁独立探针工程
// test/fixtures/modularity/lang-web-e2e/（facade 退役 auto-web 依赖；四仓
// fixture 门驱动）。

// ---- PLAN-814：run 入口族的上层装配包装（原三处隐式扇入收敛点） ----
// ui/ui-dialog 构建下，run 管线先装兼容宿主能力（if-absent 补缺，不覆盖
// 显式默认——779/785 三级装配契约），再委托 SDK 实现；纯 SDK 消费者无
// PLAN-900：run 族兼容包装（ui 构建先装宿主能力再委托 SDK）随去
// auto-ui 依赖退役——装配扇入已归 auto-ui-runtime install 单点（其
// inventory 收集点被链接即生效），此处无条件 re-export SDK 实现，
// 消费方签名与返回类型不变。
pub use auto_lang_core::{
    run, run_autovm, run_autovm_capture, run_with_capture, run_with_capture_and_path,
    run_with_capture_and_path_staged,
};
