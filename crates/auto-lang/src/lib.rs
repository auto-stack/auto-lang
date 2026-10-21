// Plan 249: unified native catalog macro expansion needs higher recursion limit
#![recursion_limit = "512"]
pub use auto_lang_core::{
    block_on_autovm_local, bootstrap_lib_source, execute_autovm,
    execute_autovm_with_deadline_and_input, execute_autovm_with_path, extract_undefined_symbol,
    find_project_root, find_store_decl_files, find_use_symbol_span, get_error_limit,
    get_global_runtime, init_py_ffi, init_rust_ffi, is_vm_debug, is_vm_hot_trace, is_vm_trace_ops,
    join_execution, p510_audit, pac_declares_dep, pac_dep_version_violation,
    pac_workspace_member_dir, parse, parse_preserve_error, pool_log_all, pool_log_seq,
    pool_trace_idx, read_auto_lib, resolve_module_path, resolve_use_module, run_abt,
    run_with_capture_and_bytecode, run_with_capture_and_bytecode_with_deadline,
    run_with_capture_and_bytecode_with_input, run_with_capture_and_bytecode_with_meta,
    run_with_capture_and_path_and_bytecode, run_with_capture_and_path_and_bytecode_with_deadline,
    run_with_capture_and_path_and_bytecode_with_input,
    run_with_capture_and_path_and_bytecode_with_meta, script_args, set_error_limit,
    set_external_back_root, set_script_args, set_ui_host_install_hook, set_vm_debug,
    store_name_snake, trans_c, trans_c_with_session, trans_rust, trans_rust_with_session,
    validate_rust_output, vm_thread_stack_size, UseModuleResolution,
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

// PLAN-664 U-4：use 导入两类静默死面的编译期诊断汇聚点（P-15 模块解析
// 失败 / P-16 导入符号与 model 字段撞名）。双出口：stderr 即时可见（消费
// 方 `auto run` 的探针日志可直接 grep `[AUTO-USE-DIAG]`），thread_local
// 汇聚供测试断言（eprintln 无法被进程内单测捕获）。
thread_local! {
    static USE_DIAGS: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
}

/// Emit a use-import compile-time diagnostic (PLAN-664 P-15/P-16): stderr +
/// thread-local sink for test assertions.
pub fn use_diag(msg: String) {
    USE_DIAGS.with(|cell| cell.borrow_mut().push(msg.clone()));
    eprintln!("{}", msg);
}

/// Take (drain) accumulated use-import diagnostics — test entry.
pub fn take_use_diags() -> Vec<String> {
    USE_DIAGS.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// PLAN-664 U-4（review R1 F-1）：该 use 语句是否为非文件系统模块形态
/// ——P-15/P-16 诊断的假阳性卫。三族：
/// - `auto.*` 内建命名空间（jade `use auto.http` 实测——走 natives
///   通道非文件解析，曾被误报 P-15）；
/// - `.web` 表单（ext 适配器通道另行装载，scanner 把整段收进 module
///   字段——plan632 F2 stderr 实证）；
/// - `use.py`（`is_python_import`——Python 侧符号）。
/// 这些形态在本环解析失败是预期路由，不是静默死面，不得告警。
fn is_non_fs_module_use(u: &crate::use_scanner::UseStatement) -> bool {
    u.is_python_import || u.module.starts_with("auto.") || u.module.starts_with(".web")
}

pub use auto_lang_core::api;
pub use auto_lang_core::ast;
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
pub mod back_proxy; // PLAN-658: 单进程多后端宿主（画廊内嵌 demo 后端 session）
pub use auto_lang_core::compile;
pub use auto_lang_core::config;
pub use auto_lang_core::database;
pub use auto_lang_core::dep;
pub use auto_lang_core::error;
pub use auto_lang_core::host; // Plan 011 (MS3-B): ShellHost bridge for system()/exit()/export()
pub use auto_lang_core::http_file_service; // PLAN-729: 服务端文件响应宿主执行（VM/生成 Rust 共用；无 ui 依赖）
pub use auto_lang_core::http_service_config; // PLAN-736: 服务部署配置与纯策略（版本中立；两轨共用）
pub use auto_lang_core::http_service_observability; // PLAN-736: 服务观测（计数器 + 有界 JSONL sink）
pub use auto_lang_core::http_upload_service; // PLAN-730: 服务端上传宿主 executor（VM/生成 Rust 共用；无 axum 类型）
pub use auto_lang_core::mcp; // Plan 265: MCP server for AI agent interaction
pub use auto_lang_core::module_cache;
pub use auto_lang_core::stdlib_assembly; // PLAN-738: stdlib 装配契约（inventory/manifest/provider 目录纯模型层）
                                         // Plan 096 Phase 0: Scenario-based compilation
                                         // Plan 768: AutoDown TestCase adapters (avm/run, a2r/emit) + runner
                                         // orchestration; depends on the dependency-light `auto-test-case` core.
pub use auto_lang_core::session;
#[cfg(feature = "test-cases")]
#[cfg(feature = "test-cases")]
pub use auto_lang_core::test_cases;
// Plan 096 Phase 0: AURA (Auto UI Representation Abstract)
// PLAN-814 T-03 Batch 2：UI 分析/生成域迁 auto-ui-compiler；旧路径经再导出保持。
pub use auto_ui_compiler::aura;
// PLAN-814 R3-02：生成器源指纹公开面（auto-man 管线在 UICache::load 前
// 注入——Plan 698 SD-01 失效合同时序）。
pub use auto_ui_compiler::generator_fingerprint;
// Plan 217: A2UI Protocol Bridge (AURA ↔ A2UI JSON)
pub use auto_ui_compiler::a2ui;
// Plan 096 Phase 2: UI Backend Generators (Vue, Rust)
pub use auto_lang_core::design_tokens;
pub use auto_ui_compiler::ui_gen;
// Stage B P-5（PLAN-590）：跨仓解析序定位 auto-os 侧资产（无 feature 门——
// ui_gen/docs 管线与 CLI 的无 ui 构建形态也要消费）。
pub use auto_lang_core::os_paths;
// PLAN-044：用户级状态文件写安全（原子替换 + 跨进程锁；键级合并写在
// vm::ffi::stdlib::storage_persist，锁路径跨仓约定单源在模块头）。
pub use auto_lang_core::state_file;
// Plan 152: Server-Sent Events (SSE) 解析
pub use auto_lang_core::sse;
// Plan 114: Hybrid Routing (Convention + Config)
pub use auto_ui_compiler::route;
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
pub use auto_lang_core::implicit_union; // Plan 125 Phase 3.3: Implicit union generator
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
pub use auto_lang_core::parser;
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
pub use auto_lang_core::vm;
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
use crate::trans::c::CTrans;
use crate::{trans::Sink, trans::Trans};
use auto_val::{AutoPath, Obj};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

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
#[cfg(any(feature = "ui", feature = "ui-dialog"))]
fn install_ui_host_capabilities() {
    // PLAN-814 R4-01：**显式文法装配优先**——本包全部装配入口（run 族
    // 漏斗/旧公开入口包装/本安装器经 SDK 钩子的回调）首先显式注入
    // UI 文法函数表（auto-ui-dialect 单源构造；幂等，首装优先）。
    // inventory 自注册（下方 submit）仅为不经任何装配入口的直达
    // `Parser::from(..).with_session(UI)` 构造兜底——语义边界与
    // UiCompatInstaller 同（见 SDK lib.rs 对账注记/R4-01 附件）。
    let _ = auto_lang_core::set_ui_grammar(auto_ui_dialect::ui_grammar());
    // PLAN-814 R3-01：本安装器同时注册进 SDK 钩子——SDK 侧旧公开入口
    // （create_vm_from_source/abt、eval_config、execute 族等）经
    // auto_lang_core::install_ui_host_capabilities() 回调本包真实装配
    // （PLAN-779 装配合同保持；首次直达调用由下方入口包装保证先注册）。
    auto_lang_core::set_ui_host_install_hook(install_ui_host_capabilities);
    // PLAN-814: i18n lookup 钩子注册（原 shim_i18n_t 的 crate::ui 直连边收口）
    #[cfg(feature = "ui")]
    auto_lang_core::set_i18n_lookup_hook(crate::ui::i18n_lookup::lookup);
    #[cfg(feature = "ui")]
    {
        crate::ui::frame_provider::install_process_default();
        // PLAN-814 R3-04：autodown 文档引擎（feature 同 owner；SDK 侧
        // natives dispatch 槽未设时经宿主钩子回调至此注册）。
        #[cfg(feature = "autodown")]
        crate::ui::autodown_engine::install();
        #[cfg(feature = "ui-clipboard")]
        crate::ui::clipboard_provider::install_process_default();
        #[cfg(all(windows, feature = "native-clipboard"))]
        crate::ui::clipboard_media_provider::install_process_default();
        crate::ui::scroll_host_provider::install_process_default();
        // PLAN-808: terminal 域 adapter（ui 单门控——引擎服务 + UI 输入/
        // 投影/menu/scroll/window/palette 接线恢复；34 native 全族）。
        crate::ui::terminal_provider::install_process_default();
        // PLAN-802: code editor host adapter（code-editor 单门控——editor
        // 注册表接线恢复，22 native 全族）。
        #[cfg(feature = "code-editor")]
        crate::ui::code_editor_provider::install_process_default();
        // PLAN-804: dnd adapter（委托 ui::native_dnd 既有
        // parse_payload_json + win32::start_drag；模块在 ui 门内，此处
        // 双门控即有效三分 windows ∧ native-dnd ∧ ui）。
        #[cfg(all(windows, feature = "native-dnd"))]
        crate::ui::dnd_host_provider::install_process_default();
    }
    // PLAN-804: dialog adapter（host_adapters 落点，ui-dialog 单门控——
    // 与 ui renderer 门无关，ui-dialog-only 形态同装）。
    #[cfg(feature = "ui-dialog")]
    crate::host_adapters::dialog::install_process_default();
}

#[cfg(not(any(feature = "ui", feature = "ui-dialog")))]
fn install_ui_host_capabilities() {
    // PLAN-814 R4-01：无 renderer 构建同样显式装配文法表——旧包
    // `ui-dialect` 默认组合时代（Scenario::UI 内建方言恒可用）与 renderer
    // feature 无关；文法属语言面，宿主能力属 renderer 面。
    let _ = auto_lang_core::set_ui_grammar(auto_ui_dialect::ui_grammar());
    // PLAN-814 R3-01：无 UI 构建同注册（no-op 安装器）——SDK 钩子语义
    // 与纯 SDK 形态一致（无 UI 默认实现）。
    auto_lang_core::set_ui_host_install_hook(install_ui_host_capabilities);
}

// PLAN-814 R3-01/R3-04：上层装配的进程期自注册（SDK 收集点
// UiCompatInstaller——本 crate 被链接即生效；覆盖不经漏斗的直达调用：
// 旧公开入口/native dispatch 的槽未设路径）。非 inventory 依赖添加：
// SDK 侧已依赖，本 crate 经 re-export 面使用。
fn plan814_ui_compat_install() {
    install_ui_host_capabilities();
}
inventory::submit!(auto_lang_core::UiCompatInstaller(plan814_ui_compat_install));

// PLAN-814 R3-03b：UI 文法函数表装配（SDK 收集点 UiGrammarSource——与
// UiCompatInstaller 同机制，本 crate 被链接即生效）。SDK 清单不再引用
// auto-ui-dialect 后，Scenario::UI 内建方言与旧 Parser 公开 UI 门面经该
// 表派发；本提交使旧默认"UI 文法恒可用"行为（含不经 run 族漏斗的直达
// `Parser::from(..).with_session(UI)` 构造）保持不变。
inventory::submit!(auto_lang_core::UiGrammarSource(auto_ui_dialect::ui_grammar));

// ---- PLAN-814 R3-01：旧公开高层入口的兼容装配包装 ----
// 这四个入口在 SDK 侧经钩子回调本包安装器；包装确保消费者首次直达调用
// （未经 run 家族漏斗）也先注册钩子再进 SDK 实现。签名与旧行为逐位一致。

/// 兼容包装：装配 UI 宿主能力后委托 SDK 实现（PLAN-779 装配合同）。
pub fn create_vm_from_source(
    source: &str,
) -> AutoResult<(
    auto_lang_core::vm::engine::AutoVM,
    std::sync::Arc<std::sync::RwLock<String>>,
    usize,
    auto_lang_core::vm::codegen::ObjectType,
)> {
    install_ui_host_capabilities();
    auto_lang_core::create_vm_from_source(source)
}

/// 兼容包装：装配 UI 宿主能力后委托 SDK 实现（PLAN-779 装配合同）。
pub fn create_vm_from_abt(
    abt_text: &str,
) -> AutoResult<(
    auto_lang_core::vm::engine::AutoVM,
    std::sync::Arc<std::sync::RwLock<String>>,
    usize,
)> {
    install_ui_host_capabilities();
    auto_lang_core::create_vm_from_abt(abt_text)
}

/// 兼容包装：装配 UI 宿主能力后委托 SDK 实现（PLAN-779 装配合同）。
pub fn eval_config_with_vm(code: &str, args: &auto_val::Obj) -> AutoResult<auto_val::Value> {
    install_ui_host_capabilities();
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

/// Plan 260: Run all `#[test]` functions in an Auto source file.
/// Synchronous wrapper around test_code.
pub fn test_file(path: &str) -> AutoResult<test_runner::TestResult> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| crate::error::AutoError::Msg(format!("Failed to read file: {}", e)))?;
    let rt = get_global_runtime();
    rt.block_on(test_code(&code))
}

#[cfg(feature = "test-aavm")]
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

#[cfg(feature = "test-aavm")]
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

#[cfg(feature = "test-aavm")]
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

/// Used by CLI to decide whether to run on main thread for iced.
#[cfg(feature = "ui-iced")]
pub fn has_ui_keywords(code: &str) -> bool {
    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("widget ") || trimmed.starts_with("widget{") {
            return true;
        }
        if trimmed.starts_with("app ") || trimmed.starts_with("app{") {
            return true;
        }
    }
    false
}

/// Return the declared name of an importable top-level statement
/// (`fn`/`type`/`enum`/`ext`), used to filter `use module: items` imports.
// Plan 330: un-gated — these helpers (stmt_symbol_name, resolve_module_path,
// collect_module_imports) are pure path/parsing utilities with no iced dep, so
// they compile in any feature config and tests can call them without ui-iced.
fn stmt_symbol_name(stmt: &crate::ast::Stmt) -> Option<String> {
    use crate::ast::Stmt;
    match stmt {
        Stmt::Fn(f) => Some(f.name.to_string()),
        Stmt::TypeDecl(t) => Some(t.name.to_string()),
        Stmt::EnumDecl(e) => Some(e.name.to_string()),
        Stmt::Ext(e) => Some(e.target.to_string()),
        _ => None,
    }
}

/// Resolve a (possibly dotted) module path to its source file under `base_dir`.
///
/// A dotted module path maps to a filesystem path segment by segment:
/// `back.api` → `back/api`, `calendar_util` → `calendar_util`. Per the module
/// rules a module lives at either `{path}.at` or `{path}/mod.at`; the first
/// existing candidate wins. Returns `None` if neither exists.
/// Plan 061:外部后端根(pac.at `back: { project }`)。设置后,`back.*`
/// 模块解析优先映射到该目录(`back.api` → `<root>/api.at`),前端项目
/// 无需本地 back/ 目录 —— 契约与桩由后端项目拥有,链接式引用(零复制)。
/// 进程级全局(`auto run` 单用途进程,生命周期即进程;unset 无需)。
/// pathological inputs). Single-element result for paths without underscore
/// segments — callers loop and behave exactly as before in that case.
#[allow(clippy::too_many_arguments)]
#[cfg(feature = "ui")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn load_ext_imports_for_vm(
    base_dir: &std::path::Path,
    ast: &crate::ast::Code,
    root_decl: &crate::ast::ui::WidgetDecl,
    all_child_decls: &[crate::ast::ui::WidgetDecl],
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
    import_stmts: &mut Vec<crate::ast::Stmt>,
    seen_symbols: &mut std::collections::HashSet<String>,
    import_session: &mut crate::compile::CompileSession,
    override_scenario: Option<&crate::session::CompilerSession>,
    import_aliases: &mut std::collections::HashMap<String, String>,
    // PLAN-051 C4: adapter 链解析到的 .at 目标内、与导入符号同名的 widget
    // 声明（ports renderer.vm.at 的 Markdown 纯文本降级等）——调用方注册进
    // 视图 registry + child_decls，激活 use.web component 的 VM widget 形态。
    ext_widget_decls: &mut Vec<crate::ast::ui::WidgetDecl>,
    // PLAN-633: 解析到的适配器文件路径——调用方补传递子 widget 注册
    // （Demo013Todo 视图内的 TodoList 等 registry miss → Empty 实证）。
    ext_adapter_paths: &mut Vec<std::path::PathBuf>,
) -> Result<(), String> {
    let mut ext_imports = crate::ui::ext_stubs::collect_useweb_imports(&ast.stmts);
    ext_imports.extend(crate::ui::ext_stubs::collect_widget_ext_imports(
        std::slice::from_ref(root_decl),
    ));
    ext_imports.extend(crate::ui::ext_stubs::collect_widget_ext_imports(
        all_child_decls,
    ));
    // Plan 442 B-support: child modules' own top-level `use.web` (e.g.
    // specs_view.at's `use.web spec_next_id from ".../specs_helpers.at"`) —
    // the root-AST walk above misses them; sweep every loaded module file.
    for path in visited.iter() {
        if let Ok(code) = std::fs::read_to_string(path) {
            // PLAN-037: 模块重读统一过前缀化 overlay（visited 集扫）。同下。
            let code = crate::back_prefix::apply(path, code);
            // PLAN-668 A-02：逐模块 stylekit 预注册（同主收集路径；前缀化不触
            // stylekit 导入面，变换后扫等价）。
            let _ = crate::design_tokens::recipe::prepare_style_recipe_imports(
                path.parent().unwrap_or(std::path::Path::new(".")),
                &code,
            );
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(code.as_str()).with_session(session);
            if let Ok(mod_ast) = parser.parse() {
                ext_imports.extend(crate::ui::ext_stubs::collect_useweb_imports(&mod_ast.stmts));
            }
        }
    }
    if ext_imports.is_empty() {
        return Ok(());
    }
    let (loaded_adapters, nested) =
        crate::ui::ext_stubs::load_at_ext_imports(base_dir, &ext_imports, &mut |adapter_path| {
            collect_module_imports(
                adapter_path,
                visited,
                import_stmts,
                seen_symbols,
                import_session,
                override_scenario,
            );
        });
    // PLAN-633: 适配器路径上交调用方（传递子 widget 注册用）。
    for (adapter, _) in &loaded_adapters {
        ext_adapter_paths.push(adapter.clone());
    }
    // Alias each adapter-loaded symbol to ITS adapter's module-qualified
    // name (collect_module_imports renames adapter fns by file stem) so call
    // sites emit resolvable qualified relocs. Pairs come from the loader —
    // aliasing by file, not by "any .at import" (which would mis-attribute
    // symbols to the wrong adapter).
    for (adapter, symbols) in &loaded_adapters {
        let qualifier = adapter
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("ext");
        for sym in symbols {
            import_aliases
                .entry(sym.clone())
                .or_insert_with(|| format!("{}.{}", qualifier, sym));
        }
    }
    ext_imports.extend(nested);

    // PLAN-051 C4 widget 臂：逐 adapter 解析 widget 声明（同名符号命中）+
    // ExplicitFn 导出校验（typo 编译期报错，替代静默落 stub——裸形式不校验，
    // 沿"可能是函数/对象/常量"的含糊语义）。
    let mut adapter_fn_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (adapter, symbols) in &loaded_adapters {
        let Ok(code) = std::fs::read_to_string(adapter) else {
            continue;
        };
        let code = crate::back_prefix::apply(adapter, code);
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let Ok(adapter_ast) = parser.parse() else {
            continue;
        };
        for stmt in &adapter_ast.stmts {
            match stmt {
                crate::ast::Stmt::WidgetDecl(wd) => {
                    if symbols.iter().any(|s| s == wd.name.as_str()) {
                        ext_widget_decls.push(wd.clone());
                    }
                }
                crate::ast::Stmt::Fn(f) => {
                    // 记录裸名供校验（collect_module_imports 重命名前形态）。
                    adapter_fn_names.insert(f.name.to_string());
                }
                _ => {}
            }
        }
    }
    for imp in &ext_imports {
        if imp.kind == crate::ast::ExtImportKind::ExplicitFn && imp.path.ends_with(".at") {
            for sym in &imp.symbols {
                if !adapter_fn_names.contains(sym) {
                    return Err(format!(
                        "use.web.fn '{}': .at 目标 {} 未导出名为 '{}' 的顶层 fn（显式 fn 声明校验，PLAN-051 C4）",
                        sym, imp.path, sym
                    ));
                }
            }
        }
    }

    // PLAN-632 F3: 已装载模块自身 use 链上的 widget 注册。根 use 环的
    // register_transitive_widgets 只覆盖根文件;use.web demo 适配器
    // （AppViewport.vm.at → Demo*.at）自带的 `use <mod>: Component`
    // （006 的 SettingsPopover）此前从不进 registry/child_decls——
    // 组件实例在 VM 渲染目标渲染 Empty/缺席。扫 visited 全集（与上方
    // use.web 收集同集、同重解析模式），按各文件 use items 显式名单
    // 注册 widget 声明进 ext_widget_decls（调用方统一入
    // registry+child_decls，handler 随之编译进单 VM）。P545 语义：
    // 仅具名 items / 显式通配可见，bare use 不触发。
    {
        let mut swept: std::collections::HashSet<String> = ext_widget_decls
            .iter()
            .map(|wd| wd.name.to_string())
            .collect();
        for wd in all_child_decls {
            swept.insert(wd.name.to_string());
        }
        swept.insert(root_decl.name.to_string());
        for path in visited.iter() {
            if path.extension().and_then(|e| e.to_str()) != Some("at") {
                continue;
            }
            let Ok(code) = std::fs::read_to_string(path) else {
                continue;
            };
            let code = crate::back_prefix::apply(path, code);
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(code.as_str()).with_session(session);
            let Ok(mod_ast) = parser.parse() else {
                continue;
            };
            let mod_dir = path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            for use_stmt in crate::use_scanner::scan_use_statements(&code) {
                if use_stmt.is_c_import || use_stmt.is_rust_import {
                    continue;
                }
                if !(use_stmt.is_wildcard || !use_stmt.items.is_empty()) {
                    continue;
                }
                // F2 增强后的解析（dep item 文件命中同路）。
                let module_path = match resolve_use_module(&mod_dir, &use_stmt) {
                    UseModuleResolution::Module(p) => p,
                    _ => continue,
                };
                // PLAN-632 F4: 已装载模块 use 链的符号别名——裸名 → 模块限定名
                // （与根 use 环 lib.rs 别名位同规则）。此前别名只对根文件 use
                // 填充：demo 作为 ext 适配器装载时其 `use calendar_util:
                // month_name` 无别名 → 视图 computed 体 `month_name(...)`
                // call_vm_fn 裸名查 exports miss（fn 实为
                // `calendar_util.month_name`）→ 求值失败回退 raw `${}`。
                // or_insert：根环别名（先填）优先。
                {
                    let qualifier = use_stmt
                        .module
                        .split('.')
                        .last()
                        .unwrap_or(&use_stmt.module);
                    for item in &use_stmt.items {
                        import_aliases
                            .entry(item.clone())
                            .or_insert_with(|| format!("{}.{}", qualifier, item));
                    }
                }
                let Ok(module_code) = std::fs::read_to_string(&module_path) else {
                    continue;
                };
                let module_code = crate::back_prefix::apply(&module_path, module_code);
                let mod_session = crate::session::CompilerSession::ui();
                let mut mod_parser =
                    crate::Parser::from(module_code.as_str()).with_session(mod_session);
                let Ok(target_ast) = mod_parser.parse() else {
                    continue;
                };
                for stmt in &target_ast.stmts {
                    if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                        if swept.contains(decl.name.as_str()) {
                            continue;
                        }
                        if use_stmt.is_wildcard
                            || use_stmt.items.iter().any(|s| s == &decl.name.as_str())
                        {
                            swept.insert(decl.name.to_string());
                            ext_widget_decls.push(decl.clone());
                        }
                    }
                }
            }
        }
    }

    // PLAN-643: 已装载模块自身 widget 级 `use { package: x from "dir" }` 的
    // 装载。demo 适配器（AppViewport.vm.at → demos/<id>.at）内的包导入此前
    // 无任何装载分支——模块链只处理 `use <mod>`/`use.web`，package 仅根组件
    // 动态分支（build_dynamic_component_inner）处理——包组件（024-charts 的
    // LineChart/BarChart/AreaChart/DonutChart）不进 registry/child_decls，
    // 实例 tag 落 schema builtin 空壳桩 → 画布空（P642-D1 根因）。扫 visited
    // 全集（与上方 use.web/632-F3 收集同集），镜像动态分支同语义：
    // load_package → decls 入 ext_widget_decls（调用方统一注册
    // registry+child_decls，handler 随之编译进单 VM）+ 包文件 use 依赖
    // collect_module_imports 入编译单元 + 裸名别名。schema package_origin
    // 分类（PLAN-643 双态归属）保证同名 tag 不再被 builtin 臂截胡。
    {
        let mut swept_pkgs: std::collections::HashSet<String> = ext_widget_decls
            .iter()
            .map(|wd| wd.name.to_string())
            .collect();
        for wd in all_child_decls {
            swept_pkgs.insert(wd.name.to_string());
        }
        swept_pkgs.insert(root_decl.name.to_string());
        let mut seen_pkg_dirs: std::collections::HashSet<std::path::PathBuf> = Default::default();
        // 快照迭代:collect_module_imports 会向 visited 追加包文件自身 use 链,
        // 不能在 visited.iter() 借用下调用(新装载模块的包导入由后续调用方的
        // 下一轮装载自然覆盖——与根环一次性 sweep 语义一致)。
        let visited_snap: Vec<std::path::PathBuf> = visited.iter().cloned().collect();
        for path in visited_snap.iter() {
            if path.extension().and_then(|e| e.to_str()) != Some("at") {
                continue;
            }
            let Ok(code) = std::fs::read_to_string(path) else {
                continue;
            };
            let code = crate::back_prefix::apply(path, code);
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(code.as_str()).with_session(session);
            let Ok(mod_ast) = parser.parse() else {
                continue;
            };
            let mod_dir = path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            for wd in mod_ast.stmts.iter().filter_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(w) => Some(w),
                _ => None,
            }) {
                for imp in &wd.ext_imports {
                    if !matches!(imp.kind, crate::ast::ui::ExtImportKind::Package) {
                        continue;
                    }
                    let dir = std::path::PathBuf::from(imp.path.as_str());
                    if !seen_pkg_dirs.insert(dir.clone()) {
                        continue;
                    }
                    let candidates = [mod_dir.join(&dir), mod_dir.join("pages").join(&dir)];
                    let mut pkg_reg = crate::ui_gen::widget::ComponentRegistry::new();
                    let first = pkg_reg.load_package(&candidates[0], &mod_dir);
                    let (loaded, loaded_dir, load_err) = match first {
                        Ok(p) => (Ok(p), Some(candidates[0].clone()), String::new()),
                        Err(_) => match pkg_reg.load_package(&candidates[1], &mod_dir) {
                            Ok(p) => (Ok(p), Some(candidates[1].clone()), String::new()),
                            Err(e) => (Err(()), None, e),
                        },
                    };
                    if loaded.is_err() {
                        log::warn!(
                            "package `{}` load failed (VM module chain): {} (tried {:?})",
                            imp.path,
                            load_err,
                            candidates
                        );
                        continue;
                    }
                    let pkg = loaded.unwrap();
                    for (d, _aw) in &pkg.full_widgets {
                        if swept_pkgs.insert(d.name.to_string()) {
                            ext_widget_decls.push(d.clone());
                        }
                    }
                    // 包组件文件自身的 use 依赖入编译单元 + 裸名别名
                    //（与动态分支 Plan 522 同规则）。
                    if let Some(pkg_dir) = loaded_dir {
                        if let Ok(entries) = std::fs::read_dir(&pkg_dir) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.extension().map(|e| e != "at").unwrap_or(true) {
                                    continue;
                                }
                                crate::collect_module_imports(
                                    &p,
                                    visited,
                                    import_stmts,
                                    seen_symbols,
                                    import_session,
                                    None,
                                );
                                if let Ok(pc) = std::fs::read_to_string(&p) {
                                    for us in crate::use_scanner::scan_use_statements(&pc) {
                                        let qualifier =
                                            us.module.split('.').last().unwrap_or(&us.module);
                                        for item in &us.items {
                                            import_aliases.entry(item.clone()).or_insert_with(
                                                || format!("{}.{}", qualifier, item),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !crate::ui::ext_stubs::ext_stubs_enabled() {
        return Ok(());
    }
    // Symbols already defined (adapter fns, root/imported fns — qualified or
    // plain) must not be stubbed.
    let mut defined: std::collections::HashSet<String> = std::collections::HashSet::new();
    for stmt in import_stmts.iter() {
        if let crate::ast::Stmt::Fn(f) = stmt {
            defined.insert(f.name.to_string());
            if let Some(pos) = f.name.as_str().rfind('.') {
                defined.insert(f.name.as_str()[pos + 1..].to_string());
            }
        }
    }
    let mut targets = std::collections::HashSet::new();
    for imp in &ext_imports {
        for sym in &imp.symbols {
            if !defined.contains(sym) {
                targets.insert(sym.clone());
            }
        }
    }
    if targets.is_empty() {
        return Ok(());
    }
    let mut arities = std::collections::HashMap::new();
    crate::ui::ext_stubs::scan_call_arities(&ast.stmts, &targets, &mut arities);
    crate::ui::ext_stubs::scan_call_arities(import_stmts, &targets, &mut arities);
    crate::ui::ext_stubs::scan_decl_arities(all_child_decls, &targets, &mut arities);
    let mut names: Vec<&String> = targets.iter().collect();
    names.sort();
    for name in names {
        let arity = arities.get(name).copied().unwrap_or(0);
        log::warn!(
            "ext stub: `{}` has no VM-target implementation (web-ecosystem source) — no-op platform stub, arity {}",
            name,
            arity
        );
        import_stmts.push(crate::ui::ext_stubs::synthesize_stub_fn(name, arity));
    }
    Ok(())
}

/// PascalCase → snake_case (compact local variant of ui_gen::api's helper;
/// `AuthStore` → `auth_store`, `AgentConfigs` → `agent_configs`).
/// StoreDecl → view-less child WidgetDecl（store-as-child：store 与子件
/// 同构、仅缺视图——统一根态模型并入 + handler 编译进单 VM 都走 child
/// 通道）。PLAN-632 F1 抽出公共转换体，供 ext 装载后的补转换位使用。
fn store_decl_as_widget_decl(store_decl: &crate::ast::ui::StoreDecl) -> crate::ast::ui::WidgetDecl {
    crate::ast::ui::WidgetDecl {
        name: store_decl.name.clone(),
        messages: store_decl.messages.clone(),
        model: store_decl.model.clone(),
        computed: store_decl.computed.clone(),
        view: None,
        named_views: Vec::new(),
        on: store_decl.on.clone(),
        bind: None,
        props: Vec::new(),
        routes: None,
        lifecycle: Vec::new(),
        style: None,
        ext_imports: Vec::new(),
        watch: Vec::new(),
        expose: Vec::new(),
        setup: None,
        actions: None,
        timer: store_decl.timer.clone(),
    }
}

/// Plan 442 A2 (store facade): the legacy `use store: Name` form uses the
/// literal module name "store", which never maps to a `store.at` file — the
/// StoreDecl lives in a sibling file named after the store (musk convention:
/// `AuthStore` → `auth_store.at`, `AgentConfigs` → `agent_configs.at`).
/// Locate those files by naming convention first, then a bounded recursive
/// scan of the source dir (parsing each .at and matching StoreDecl names).
/// Returns store name → file path for every name that resolves.
/// Recursively collect ALL declarations (Fn/TypeDecl/EnumDecl/Ext/Store/Use)
/// from a module and its transitive `use` dependencies, for VM-render
/// compilation.
///
/// Why "all declarations" rather than just the `use`-named items: an imported
/// function's body may call sibling helpers in the same module that are NOT
/// listed in the `use` clause (e.g. `use calendar_util: build_month_grid` does
/// not name `weekday_of`/`days_in_month`/`format_date`, yet `build_month_grid`
/// calls them). A module is a compilation unit, so the whole module — plus the
/// modules it itself imports — must be fed to the single Codegen pass for
/// callees to resolve. Dedups by canonical path (cycle guard) and by symbol
/// name (first definition wins).
///
/// Plan 333: a shared `CompileSession` is threaded through the recursion so
/// that `resolve_uses` pre-registers imported types (e.g. `Note` from
/// `use api: Note` in back/db.at) into a `type_store` BEFORE each module is
/// parsed. Without this, db.at's `var notes List<Note> = List<Note>.new([...])`
/// fails to parse with "undefined variable: Note", dropping all of db.at's
/// functions and breaking `db.func()` linking. This mirrors the script path
/// (`execute_autovm_with_path`), which already does this.
/// Plan 446 批一 (C1-3): UI 模块 parse 失败的进程级登记表。收集侧
/// (`collect_module_imports`) 保持非致命(投机解析路径安全),`VmBridge` 构造时
/// 检查本表并升级为致命错误 —— 消除"WARN + 静默空渲染"(os-config C1 现场)。
pub fn ui_module_parse_failures() -> &'static std::sync::Mutex<Vec<String>> {
    static REG: std::sync::OnceLock<std::sync::Mutex<Vec<String>>> = std::sync::OnceLock::new();
    REG.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

/// Plan 446 批一 (C1-2): 把 AutoError 的 miette span 偏移对照源码换算为
/// `line:col` 定位。AutoError 的 Display 链不含位置;`Diagnostic::labels()`
/// 给出字节偏移。MultipleErrors 递归展开(上限 20 条,与 parser 恢复上限一致)。
fn positioned_parse_errors(e: &crate::error::AutoError, source: &str) -> String {
    use miette::Diagnostic;
    fn walk(e: &crate::error::AutoError, source: &str, out: &mut Vec<String>, depth: usize) {
        if out.len() >= 20 || depth > 4 {
            return;
        }
        if let crate::error::AutoError::MultipleErrors { errors, .. } = e {
            for sub in errors {
                walk(sub, source, out, depth + 1);
            }
            return;
        }
        let msg = e.to_string();
        let mut line_col = None;
        if let Some(span) = e.labels().and_then(|mut ls| ls.next()) {
            let mut offset = span.inner().offset();
            // Plan 011(os-config):span offset 可能落在多字节字符内部(如中文
            // 注释/字符串里报错)——直接切片会 panic("not a char boundary"),
            // 把真实语法错误掩盖成崩溃。回退到最近的字符边界。
            while offset > 0 && !source.is_char_boundary(offset) {
                offset -= 1;
            }
            if offset <= source.len() {
                let until = &source[..offset];
                let line = until.matches('\n').count() + 1;
                let col = until
                    .rsplit('\n')
                    .next()
                    .map(|l| l.chars().count())
                    .unwrap_or(0)
                    + 1;
                line_col = Some((line, col));
            }
        }
        out.push(match line_col {
            Some((l, c)) => format!("  error at {l}:{c}: {msg}"),
            None => format!("  error (no span): {msg}"),
        });
    }
    let mut out = Vec::new();
    walk(e, source, &mut out, 0);
    if out.is_empty() {
        e.to_string()
    } else {
        out.join("\n")
    }
}

fn collect_module_imports(
    module_path: &std::path::Path,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
    out: &mut Vec<crate::ast::Stmt>,
    seen: &mut std::collections::HashSet<String>,
    session: &mut crate::compile::CompileSession,
    override_scenario: Option<&crate::session::CompilerSession>,
) {
    let canon = module_path
        .canonicalize()
        .unwrap_or_else(|_| module_path.to_path_buf());
    if !visited.insert(canon) {
        return;
    }
    let code = match std::fs::read_to_string(module_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    // PLAN-037: launch 期作用域化模块源前缀化（未激活时零成本透传）。
    let code = crate::back_prefix::apply(module_path, code);
    // Plan 333: pre-resolve this module's `use` deps into the shared type_store
    // so types like `Note` (from `use api: Note`) are registered before parse.
    // resolve_uses is itself recursive (it follows each module's own uses), so
    // transitive types land in the store. Seed source_dirs with this module's
    // directory AND its parent so `back.api` resolves from `src/front/` and
    // `use api` resolves from `src/back/`.
    if let Some(dir) = module_path.parent() {
        let abs = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        session.add_source_dir(abs);
        if let Some(parent) = dir.parent() {
            let abs_parent = std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
            session.add_source_dir(abs_parent);
        }
    }
    // Tolerate resolve_uses failures (e.g. a rust dep not declared); we still
    // attempt the parse below with whatever types did resolve.
    let _ = session.resolve_uses(&code);

    // Choose parser scenario. Priority (PR-6):
    //   1. override_scenario (from CLI --scene / pac.at scene field)
    //   2. .au extension → UI scenario (弱提示推断)
    //   3. path heuristic: back/ → Core, else UI
    //   4. default: Core
    let parser_session = if let Some(s) = override_scenario {
        s.clone()
    } else if module_path.extension().map_or(false, |e| e == "au") {
        // PR-6: .au files default to UI scenario (weak hint, overridable by --scene).
        crate::session::CompilerSession::ui()
    } else if module_path.components().any(|c| c.as_os_str() == "back") {
        crate::session::CompilerSession::core()
    } else {
        crate::session::CompilerSession::ui()
    };
    let mut parser = Parser::new_with_type_store(code.as_str(), session.type_store())
        .with_session(parser_session);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            // Plan 398: surface parse failures instead of silently dropping the
            // module. Previously a parse error here caused ALL of the module's
            // symbols (e.g. every `api.*` fn) to vanish from the VM module,
            // producing misleading "Undefined symbol: api.X" link errors far
            // downstream. Log so the real .at parse issue is visible.
            //
            // Plan 446 批一 (C1-2/C1-3): (1) 诊断带 文件:行:列 —— Display 链不含
            // 位置,这里从 miette Diagnostic 的 labels 提取 span 偏移,对照手边的
            // 源码换算行列;(2) 记入全局注册表,VmBridge 构造时升级为致命错误
            // (此前 WARN + 静默空渲染,os-config C1 现场"模块丢弃极难定位")。
            // 本函数保持非致命返回(投机解析路径安全),致命化在桥接层统一执行。
            let detail = positioned_parse_errors(&e, &code);
            eprintln!(
                "\n════ module parse failed (fatal at boot, plan-446 C1) ════\n  file: {}\n{}\n════════════════════════════════════════════════════════",
                module_path.display(),
                detail
            );
            log::warn!(
                "collect_module_imports: parse failed for {}: {}",
                module_path.display(),
                e
            );
            ui_module_parse_failures().lock().unwrap().push(format!(
                "{}: {}",
                module_path.display(),
                detail.lines().next().unwrap_or("(no detail)").trim()
            ));
            return;
        }
    };
    // Plan 339: derive module name from file path.
    // back/db.at → "db", back/api.at → "api", front/editor.at → "editor"
    let module_name = module_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // 1. Take every declaration in this module (dedup by qualified name).
    //    Plan 339: prefix functions from back/ modules with the module name
    //    (e.g. db.at's create_note → db.create_note) so db.at and api.at's
    //    functions don't collide. This replaces the last-wins workaround
    //    (Plan 338) since qualified names are unique.
    for stmt in &ast.stmts {
        match stmt {
            crate::ast::Stmt::Fn(_) => {
                if let Some(name) = stmt_symbol_name(stmt) {
                    // Plan 339 Phase 3: ALL Fn declarations from ALL modules get
                    // a module-qualified name (e.g. api.create_note, db.create_note).
                    // TypeDecl/EnumDecl/Ext are NOT qualified — types are shared.
                    let qualified = format!("{}.{}", module_name, name);
                    if seen.insert(qualified.clone()) {
                        let mut s = stmt.clone();
                        if let crate::ast::Stmt::Fn(ref mut f) = s {
                            f.name = crate::ast::Name::from(qualified.as_str());
                        }
                        out.push(s);
                    }
                }
            }
            crate::ast::Stmt::TypeDecl(_)
            | crate::ast::Stmt::EnumDecl(_)
            | crate::ast::Stmt::Ext(_) => {
                if let Some(name) = stmt_symbol_name(stmt) {
                    if seen.insert(name.clone()) {
                        out.push(stmt.clone());
                    }
                }
            }
            // Use statements have no dedup key; emit them as-is. (codegen
            // re-dedups auto_modules registration by name.)
            crate::ast::Stmt::Use(_) => {
                out.push(stmt.clone());
            }
            // Module-level stores (globals). Dedup by the variable name so a
            // transitive re-import doesn't double-declare.
            crate::ast::Stmt::Store(s) => {
                let key = format!("__store:{}", s.name);
                if seen.insert(key) {
                    out.push(stmt.clone());
                }
            }
            // VM multi-store fix: collect StoreDecl (store Name { ... }) from
            // imported modules so stores used only by route pages (not the root
            // widget) are compiled into the VM module. Without this, the store's
            // handlers are undefined at link time. Dedup by store name.
            crate::ast::Stmt::StoreDecl(sd) => {
                // PLAN-733: 登记 store 源文件模块（file stem）——合成该
                // store 的 handler 时作 own-module 裸名绑定域（同文件模块级
                // fn 裸调用不再因跨模块同名歧义悬空）。
                #[cfg(feature = "ui")]
                crate::ui::handler_codegen::register_store_source_module(
                    &sd.name.to_string(),
                    &module_name,
                );
                let key = format!("__storedecl:{}", sd.name);
                if seen.insert(key) {
                    out.push(stmt.clone());
                }
            }
            _ => {}
        }
    }
    // 2. Recurse into this module's own `use` dependencies, each resolved
    //    relative to THIS module's directory (so back/api.at's `use db` finds
    //    back/db.at).
    let module_dir = module_path.parent().unwrap_or(std::path::Path::new("."));
    for dep in crate::use_scanner::scan_use_statements(&code) {
        if dep.is_c_import || dep.is_rust_import {
            continue;
        }
        match resolve_use_module(module_dir, &dep) {
            UseModuleResolution::Module(dep_path) => {
                collect_module_imports(&dep_path, visited, out, seen, session, override_scenario);
            }
            UseModuleResolution::StoreFiles(found) => {
                // Plan 442 A2: transitive `use store: X` (legacy facade form) —
                // module name never maps to store.at; locate the StoreDecl's file
                // by convention/scan so the store (and its own `use` deps)
                // compile into the VM module. Without this, a child widget's
                // store stays uncollected and its handlers fail at link time.
                for (_, path) in found {
                    collect_module_imports(&path, visited, out, seen, session, override_scenario);
                }
            }
            UseModuleResolution::None => {
                // PLAN-664 U-4（P-15）：传递性 use 同告警化（见根 use 环
                // 同名诊断注记）；review R1 F-1 同卫（非文件系统形态）。
                if !is_non_fs_module_use(&dep) {
                    crate::use_diag(format!(
                        "[AUTO-USE-DIAG] P-15 传递 use 模块 `{}` 解析失败（items: {:?}）——基目录 {} 均无对应 .at/mod.at（PLAN-664）。该导入静默跳过。",
                        dep.module,
                        dep.items,
                        module_dir.display()
                    ));
                }
            }
        }
    }
}

/// Run a .at file as dynamic UI with iced backend.
/// MUST be called from the OS main thread (iced requirement).
#[cfg(feature = "ui-iced")]
pub fn run_file_dynamic_ui(code: &str, path: Option<&str>) -> AutoResult<String> {
    // PR-6: 默认 UI 场景（向后兼容）。调用方可通过 run_file_dynamic_ui_with_scenario 覆写。
    run_file_dynamic_ui_inner(code, path, None)
}

/// PR-6: 带场景覆写的 run_file_dynamic_ui。
/// override_scenario 为 None 时默认 UI（向后兼容）。
#[cfg(feature = "ui-iced")]
pub fn run_file_dynamic_ui_with_scenario(
    code: &str,
    path: Option<&str>,
    override_scenario: Option<&crate::session::CompilerSession>,
) -> AutoResult<String> {
    run_file_dynamic_ui_inner(code, path, override_scenario)
}

/// EDGE-16 第四层:递归注册传递性子组件(孙组件)。
///
/// 直接子组件(如 block_list.at 的 BlockList)自己 `use` 的子组件(如
/// block_item.at 的 BlockItem)默认不进 registry(lib.rs 的 widget 收集
/// 注释明确说 "Transitive modules do not register widgets here"),导致
/// BlockList 渲染时 `<BlockItem ...>` 在 registry 找不到 → fallback Empty。
///
/// 此函数从 `module_path` 出发,扫它的 `use` 导入,把传递性 widget 注册进
/// `registry`。用 `visited` 防循环(与 collect_module_imports 共享 visited)。
/// 只注册 AuraWidget(孙组件),不重复处理 store(store 走 Plan 370 路径)。
#[cfg(feature = "ui-iced")]
fn register_transitive_widgets(
    module_path: &std::path::Path,
    registry: &mut crate::ui::widget_registry::WidgetRegistry,
    child_decls: &mut Vec<crate::ast::WidgetDecl>,
) {
    // 独立的 visited(不与 collect_module_imports 共享,避免互相跳过模块)。
    register_transitive_widgets_inner(
        module_path,
        registry,
        child_decls,
        &mut std::collections::HashSet::new(),
    )
}

#[cfg(feature = "ui-iced")]
fn register_transitive_widgets_inner(
    module_path: &std::path::Path,
    registry: &mut crate::ui::widget_registry::WidgetRegistry,
    child_decls: &mut Vec<crate::ast::WidgetDecl>,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
) {
    let module_code = match std::fs::read_to_string(module_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    // PLAN-037: 同上——孙组件模块读点过前缀化 overlay。
    let module_code = crate::back_prefix::apply(module_path, module_code);
    let base_dir = module_path.parent().unwrap_or(std::path::Path::new("."));
    let use_stmts = crate::use_scanner::scan_use_statements(&module_code);
    for use_stmt in &use_stmts {
        if use_stmt.is_c_import || use_stmt.is_rust_import {
            continue;
        }
        let sub_path = match crate::resolve_module_path(base_dir, &use_stmt.module) {
            Some(p) => p,
            None => continue,
        };
        // 防循环:规范化路径后查 visited
        let canon = match sub_path.canonicalize() {
            Ok(c) => c,
            Err(_) => sub_path.clone(),
        };
        if !visited.insert(canon.clone()) {
            continue;
        }
        let sub_code = match std::fs::read_to_string(&sub_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // PLAN-037: 同上——孙组件子模块读点过前缀化 overlay。
        let sub_code = crate::back_prefix::apply(&sub_path, sub_code);

        // PLAN-668 A-02：逐模块 stylekit 预注册（同 4244 主路径——模块级
        // `use stylekit.styles:` 缺预注册则整文件 parse 失败被静默吞）。
        let _ = crate::design_tokens::recipe::prepare_style_recipe_imports(
            sub_path.parent().unwrap_or(std::path::Path::new(".")),
            &sub_code,
        );
        let mod_session = crate::session::CompilerSession::ui();
        let mut mod_parser = crate::Parser::from(sub_code.as_str()).with_session(mod_session);
        if let Ok(sub_ast) = mod_parser.parse() {
            for stmt in &sub_ast.stmts {
                if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                    if let Ok(child_widget) = crate::aura::extract_widget_from_decl(decl) {
                        // 只注册 use 子句明确要的(或通配/裸装载的),且 registry
                        // 还没有的。PLAN-545 回归修复（069 收口转介收回）：
                        // bare `use mod` 的组件发现语义恢复（items.is_empty()
                        // 臂）——545 的"bare=命名空间"针对 fn 符号冲突面；
                        // UI 子件码道收紧后裸 use 孙件 handler 不再合成
                        // （同 demo CustomScrollbar 断裂）。
                        // PLAN-093 G-16（095 T-07 根文件修复的传递装载对应
                        // 面）：同文件兄弟 widget 一并注册——items 过滤只决定
                        // "显式引用谁"，不应把同文件其他 widget 逐出 registry
                        // （musk canvas_panel.at 实证：use 只列
                        // CanvasPanel/CanvasStudioSlot，兄弟 CanvasCanvasColumn
                        // 被 skip → 组件臂 miss → 画布列空渲染）。registry
                        // 已有者仍跳过（先到优先，显式 use 同名覆写沿 095
                        // 口径由后装载覆盖）。
                        if registry.get(&child_widget.name).is_none() {
                            // Plan 049:孙组件 handler 也要编译进 VM 模块,
                            // 否则点击其按钮(如 BlockItem.ToggleCollapse)会
                            // HandlerNotFound 静默失败。
                            child_decls.push(decl.clone());
                            registry.register(child_widget);
                        }
                    }
                }
                // Plan 044 M3: 传递性模块的 view fn 也注册(同类 EDGE-16c,
                // 但 view fn 走 register_view_fragment 而非 widget registry)。
                if let crate::ast::Stmt::ViewFragmentDecl(frag) = stmt {
                    crate::aura::extract::register_view_fragment(frag);
                }
            }
        }
        // 继续递归(孙组件可能还 use 了更深的组件)
        register_transitive_widgets_inner(&sub_path, registry, child_decls, visited);
    }
}

#[cfg(feature = "ui-iced")]
fn build_dynamic_component_inner(
    code: &str,
    path: Option<&str>,
    override_scenario: Option<&crate::session::CompilerSession>,
) -> AutoResult<crate::ui::dynamic::DynamicComponent> {
    use crate::session::CompilerSession;
    use crate::ui::dynamic::DynamicComponent;
    use crate::ui::widget_registry::WidgetRegistry;

    // PLAN-635: pre-register use-imported recipes BEFORE the parse — the
    // parser's symbol checker accepts `style: <name>` idents only when the
    // recipe is in the live registry.
    let recipe_imports = match path {
        Some(p) => {
            let base_dir = std::path::Path::new(p)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            crate::design_tokens::recipe::prepare_style_recipe_imports(&base_dir, code)
                .map_err(|e| e.to_string())?
        }
        None => Vec::new(),
    };

    // 1. Parse with UI scenario (or override if provided — PR-6)
    let session = override_scenario
        .cloned()
        .unwrap_or_else(CompilerSession::ui);
    let mut parser = Parser::from(code).with_session(session);
    let ast = parser.parse()?;

    // 2. Extract first AuraWidget (and keep its WidgetDecl — PR-3b Step 4).
    // Plan 367 P2-3: register the root module's `view fn` fragments before
    // extracting the root widget's view tree, so fragment calls inline-expand.
    crate::aura::extract::clear_view_fragments();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::ViewFragmentDecl(frag) = stmt {
            crate::aura::extract::register_view_fragment(frag);
        }
    }
    // PLAN-607: register and validate style recipes before extracting widget.
    // PLAN-635: replay imported (source-tagged) + local recipes in order.
    crate::design_tokens::recipe::load_and_validate_style_recipes_with_imports(
        &recipe_imports,
        &ast.stmts,
    )
    .map_err(|e| e.to_string())?;
    let mut root_decl: Option<crate::ast::WidgetDecl> = None;
    let mut widget = None;
    // PLAN-095 T-07 (G-11): 同文件兄弟 widget 一并提取注册——此前 break 只取
    // 首个 WidgetDecl，同文件兄弟不进 WidgetRegistry，组件臂 miss 静默降级，
    // 子件整体零节点（093 §8.16 实证）。兄弟先于 use 导入注册（显式 use
    // 稍后装载时同名覆写——显式导入优先）。
    let mut sibling_decls: Vec<crate::ast::WidgetDecl> = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
            if root_decl.is_none() {
                root_decl = Some(decl.clone());
                widget =
                    Some(crate::aura::extract_widget_from_decl(decl).map_err(|e| e.to_string())?);
            } else {
                sibling_decls.push(decl.clone());
            }
        }
    }

    let widget = widget.ok_or("No widget declaration found")?;
    let root_decl = root_decl.ok_or("No widget declaration found")?;

    // PLAN-050 T9 (C7): VM 轨 t() 插值查表——front 根目录 i18n/{lang}.json
    // 存在则装载（默认 zh, AUTO_LOCALE 可覆）；缺席 = 空表静默。
    if let Some(file_path) = path {
        if let Some(base_dir) = std::path::Path::new(file_path).parent() {
            crate::ui::i18n_lookup::load_from_dir(base_dir);
        }
    }

    // Plan 451: root widget 的 `actions {}` 块安装 DSL 动作配置（优先于
    // auto-atom 外挂文件；热重载经 action_config::reload_action_config
    // 重读本源文件）。消费点（键盘回退层/menubar·toolbar 合成/MCP 同源）
    // 全部经 action_config() 取数，零改动。
    // Plan 451 P3 合并优先级：宿主 widget 自带块 > 本文件顶层声明 >
    // use 引入模块的顶层声明（导入序首个）。顶层声明的 handler 引用此时
    // 才有宿主，不做解析期校验（由 from_actions_block 的引用校验兜底）。
    let mut dsl_actions_installed = false;
    if let Some(ref acts) = root_decl.actions {
        crate::ui::action_config::set_dsl_action_config_from_block(acts, path);
        dsl_actions_installed = true;
    }

    // 2b. Load child widgets + imported functions/types from `use` imports
    let mut registry = WidgetRegistry::new();
    // PR-3b Step 4: collect child WidgetDecls alongside child AuraWidgets so the
    // decl-based synthesis path can compile child handlers into the single VM.
    let mut child_decls: Vec<crate::ast::WidgetDecl> = Vec::new();
    // PLAN-095 T-07 (G-11): 同文件兄弟 widget 注册进 registry + child_decls
    //（handler 编入单 VM，与 use 导入子件同面）。
    for decl in &sibling_decls {
        let aura = crate::aura::extract_widget_from_decl(decl).map_err(|e| e.to_string())?;
        registry.register(aura);
        child_decls.push(decl.clone());
    }
    let mut import_stmts: Vec<crate::ast::Stmt> = Vec::new();
    // Visited-module / seen-symbol sets span ALL top-level use stmts so shared
    // transitive deps load once.
    let mut visited = std::collections::HashSet::new();
    let mut seen_symbols = std::collections::HashSet::new();
    // Plan 333: a shared CompileSession carries the type_store across the whole
    // recursive import walk so imported types resolve before each module parse.
    let mut import_session = crate::compile::CompileSession::new();
    // Plan 339 Phase 4: collect use alias mappings (bare name → module-qualified name)
    let mut import_aliases: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    if let Some(file_path) = path {
        let use_stmts = crate::use_scanner::scan_use_statements(code);
        let base_dir = std::path::Path::new(file_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        // PLAN-012 F2：`use` 兄弟模块解析的**仓根回退基**——source path 为
        // 仓根相对形态（ui_desktop 直挂窗：`examples/ui/...`）且 CWD ≠ 仓根
        // 时（desktop.ps1 iced 轨 CWD=os 根），CWD 相对解析永不命中，
        // `use prog_util` 链接死（Undefined symbol——桌面 boot 直挂计算器
        // 实测）。回退基 = 语言仓根（编译期锚，CWD 无关；主检出/worktree
        // 构建各自自洽）。首基（CWD 相对）优先，回退基仅在前者 miss 时参与。
        let mut base_dirs: Vec<std::path::PathBuf> = vec![base_dir.to_path_buf()];
        // PLAN-041 T-02（T-01 决策件推荐臂：装载链补 back 符号注册）：
        // auto-os app 布局 = `src/front/` + `src/back/`，front 的
        // `use back.api` 按既有基目录只探 `front/back/api.at`——miss 后
        // 静默跳过（下方 `if let Ok(read_to_string)`），符号悬空至链接期
        // `Undefined symbol: api.kill_process` 死窗（025-sys-monitor/
        // kanban/017-chat 家族实录）。增补 entry 父目录（= `src/`）为次
        // 基：`back.api → src/back/api.at` 命中，back 模块随 import 束
        // 正常编译链接。首基序不变（front 兄弟优先，无回归面）。
        if let Some(src_dir) = base_dir.parent() {
            if src_dir.file_name().map_or(false, |n| n == "src") {
                base_dirs.push(src_dir.to_path_buf());
            }
        }
        let lang_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .map(|p| p.to_path_buf());
        if let Some(root) = &lang_root {
            let fb = root.join(base_dir);
            if !base_dirs.contains(&fb) {
                base_dirs.push(fb);
            }
        }

        for use_stmt in &use_stmts {
            // Skip non-module imports (c, rust, py)
            if use_stmt.is_c_import || use_stmt.is_rust_import {
                continue;
            }
            // PLAN-664 U-4（P-16）：use 导入符号与 widget model 字段撞名 =
            // link 期 "Undefined symbol: App_State.<field>" 死面（080 T-03
            // 实录：`use panels_graph_fns: graph_stats` × `var graph_stats
            // map`，MCP 不起、零编译期告警）。撞名时名字被状态通道抢占，
            // 此处编译期告警化；硬错化留待确认无合法撞名形态后另议。
            // review R1 F-1：非文件系统形态（auto.*/.web/use.py）跳过——
            // 其 items 是外域符号，撞名检查无意义且会误报。
            if !is_non_fs_module_use(use_stmt) {
                for item in &use_stmt.items {
                    let mut owner: Option<String> = None;
                    if let Some(model) = &root_decl.model {
                        if model.fields.iter().any(|f| f.name.as_str() == item) {
                            owner = Some(root_decl.name.to_string());
                        }
                    }
                    if owner.is_none() {
                        for d in &child_decls {
                            if d.model
                                .as_ref()
                                .map_or(false, |m| m.fields.iter().any(|f| f.name.as_str() == item))
                            {
                                owner = Some(d.name.to_string());
                                break;
                            }
                        }
                    }
                    if let Some(w) = owner {
                        crate::use_diag(format!(
                        "[AUTO-USE-DIAG] P-16 use 导入符号 `{item}` 与 widget `{w}` 的 model 字段撞名——VM 轨该名字被状态通道占用，fn 导入将在 link 期落 `Undefined symbol` 类死面（PLAN-664）。fn 与字段改名错开其一。"
                    ));
                    }
                }
            }
            // Locate the module source file. A dotted module path maps to a
            // filesystem path: `back.api` → `back/api.at`, `calendar_util` →
            // `calendar_util.at`. Per the module rules (CLAUDE.md) a module is
            // either `{path}.at` or `{path}/mod.at`; try both.
            // PLAN-012 F2：逐基尝试（CWD 相对首基优先，仓根回退基兜底）。
            let mut module_path = UseModuleResolution::None;
            for base in &base_dirs {
                module_path = resolve_use_module(base, use_stmt);
                if !matches!(module_path, UseModuleResolution::None) {
                    break;
                }
            }
            let module_path = match module_path {
                UseModuleResolution::Module(p) => p,
                UseModuleResolution::StoreFiles(found) => {
                    // Plan 442 A2 (store facade): legacy `use store: Name` uses
                    // the literal module name "store" — no store.at exists; the
                    // StoreDecl lives in a sibling file named after the store.
                    // Feed each located file through collect_module_imports so
                    // the store's decl (→ view-less child WidgetDecl via the
                    // import conversion below) and its own `use` deps compile
                    // into the VM module. Without this the store context stays
                    // empty and handlers/views referencing `store.X` fail with
                    // "Undefined variable: store" (musk KNOWN-DEBT 028 ③).
                    for (name, path) in &found {
                        log::info!(
                            "store facade: `use store: {}` resolved to {}",
                            name,
                            path.display()
                        );
                        collect_module_imports(
                            path,
                            &mut visited,
                            &mut import_stmts,
                            &mut seen_symbols,
                            &mut import_session,
                            override_scenario,
                        );
                    }
                    for name in &use_stmt.items {
                        if !found.contains_key(name) {
                            log::warn!(
                                "store facade: `use store: {}` — no StoreDecl file found under {} (looked for {}.at)",
                                name,
                                base_dir.display(),
                                store_name_snake(name)
                            );
                        }
                    }
                    continue;
                }
                UseModuleResolution::None => {
                    // PLAN-664 U-4（P-15）：fn/组件 use 模块解析失败此前
                    // 静默跳过——消费侧表现 = 引用符号的视图/handler 落空、
                    // App 空视图零报错（jade 079 E-8 跨项目 fn 导入静默
                    // 死面实录）。编译期告警化：留下可 grep 诊断；硬错化
                    // 留待确认无合法未解析形态后另议。
                    // review R1 F-1：非文件系统形态（auto.*/.web/use.py）
                    // 解析失败是预期路由，不告警（jade use auto.http
                    // 实测假阳性）。
                    if !is_non_fs_module_use(use_stmt) {
                        crate::use_diag(format!(
                            "[AUTO-USE-DIAG] P-15 use 模块 `{}` 解析失败（items: {:?}）——候选基目录均无对应 .at/mod.at（PLAN-664）。该导入静默跳过，引用其符号的面将落空。",
                            use_stmt.module, use_stmt.items
                        ));
                    }
                    continue;
                }
            };

            // Register child widgets declared directly in this top-level module
            // (filtered by the use clause's item list). Transitive modules do
            // not register widgets here.
            // PLAN-668 A-02：逐模块文件 stylekit 预注册——模块自身可携带
            // `use stylekit.styles:`（PLAN-637 起 editor.at 实案），host 级
            // 预注册只覆盖根 app.at；缺本步则模块 parse 落 UndefinedVariable
            // 被 `if let Ok` 静默吞掉 → 子 widget/handler 整件丢失（015
            // EditorPanel：handler_EditorPanel_EditTitle 不在 exports 实证）。
            if let Ok(module_code) = std::fs::read_to_string(&module_path) {
                // PLAN-037: 同上——子模块读点过前缀化 overlay。
                let module_code = crate::back_prefix::apply(&module_path, module_code);
                // PLAN-668 A-02：逐模块文件 stylekit 预注册（同上，变换后扫等价）。
                let _ = crate::design_tokens::recipe::prepare_style_recipe_imports(
                    module_path.parent().unwrap_or(std::path::Path::new(".")),
                    &module_code,
                );
                let mod_session = CompilerSession::ui();
                let mut mod_parser = Parser::from(module_code.as_str()).with_session(mod_session);
                if let Ok(mod_ast) = mod_parser.parse() {
                    // Plan 367 P2-3: register this module's `view fn` fragments
                    // BEFORE extracting any widget view trees below. A widget in
                    // the same module (e.g. NavTree in sidebar.at) calls a
                    // fragment (e.g. NoteItem); extract_widget_from_decl inlines
                    // those calls only if the fragment is in the VIEW_FRAGMENTS
                    // thread_local. Without this the call stays as an unexpanded
                    // component node and renders empty in VM mode.
                    for stmt in &mod_ast.stmts {
                        if let crate::ast::Stmt::ViewFragmentDecl(frag) = stmt {
                            crate::aura::extract::register_view_fragment(frag);
                        }
                    }
                    for stmt in &mod_ast.stmts {
                        if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                            if let Ok(child_widget) = crate::aura::extract_widget_from_decl(decl) {
                                // PLAN-545 回归修复（069 收口转介收回）：bare
                                // use 的**组件发现**语义恢复——`use custom_scrollbar`
                                //（无 items 无通配）装载该模块的子件。545 的
                                // "bare=命名空间"裁定针对 fn 符号冲突面
                                // （compile.rs/linker），此处是 UI 子件收集码道；
                                // 收紧时连带删掉 items.is_empty() 臂令裸 use 子件
                                // 的 handler 不再合成（handler_<Child>_* 导出
                                // 缺席→运行时派发全灭，demo CustomScrollbar
                                // 拖拽死——vm-smoke 组 4(d) 实证）。
                                // PLAN-093 G-16（095 T-07 的导入模块对应面）：
                                // 同文件兄弟 widget 一并注册——use items 过滤
                                // 只决定"显式引用谁"，不应把同文件其他 widget
                                // 逐出 registry（musk canvas_panel.at 实证：
                                // 兄弟 CanvasCanvasColumn 被 skip → 组件臂
                                // miss → 画布列空渲染）。显式 use 同名覆写
                                // 优先沿 095 根文件修复口径（后到覆盖）。
                                if std::env::var_os("AUTO_DEBUG_G16").is_some() {
                                    eprintln!(
                                        "[G16] load-site register: {} from {:?} (matched={})",
                                        child_widget.name,
                                        module_path.display(),
                                        use_stmt.items.iter().any(|s| s == &child_widget.name)
                                    );
                                }
                                child_decls.push(decl.clone());
                                registry.register(child_widget);
                            }
                        } else if let crate::ast::Stmt::StoreDecl(store_decl) = stmt {
                            // Plan 370 D-GAP-4: stores imported via `use module: Store`
                            // (e.g. `use notes_store: NotesStore`) live in their own
                            // source file, NOT the root AST. Convert them to view-less
                            // child WidgetDecls here — mirroring the root-AST store
                            // conversion below — so their state fields get merged into
                            // the root state object and their handlers compile into the
                            // single VM module. Without this, `store.X` fields (notes,
                            // active_id, ...) never exist in `__state`, so handlers
                            // like `.notes = list_notes()` silently no-op and the view
                            // renders an empty list.
                            let name = store_decl.name.clone();
                            // 069 同款回归修复：bare use 亦装载 store（组件
                            // 发现语义，见上方 WidgetDecl 臂注）。
                            if use_stmt.is_wildcard
                                || use_stmt.items.is_empty()
                                || use_stmt.items.iter().any(|s| *s == name.as_str())
                            {
                                child_decls.push(crate::ast::ui::WidgetDecl {
                                    name: name.clone(),
                                    messages: store_decl.messages.clone(),
                                    model: store_decl.model.clone(),
                                    computed: store_decl.computed.clone(),
                                    view: None,
                                    named_views: Vec::new(),
                                    on: store_decl.on.clone(),
                                    bind: None,
                                    props: Vec::new(),
                                    routes: None,
                                    lifecycle: Vec::new(),
                                    style: None,
                                    ext_imports: Vec::new(),
                                    watch: Vec::new(),
                                    expose: Vec::new(),
                                    setup: None,
                                    actions: None,
                                    timer: store_decl.timer.clone(),
                                });
                            }
                        }
                    }
                }
            }

            // EDGE-16 第四层:递归收集孙组件(传递性 widget)。
            // 直接子组件(block_list.at 的 BlockList)自己 use 的子组件
            // (block_item.at 的 BlockItem)不会进 registry,导致 BlockList
            // 渲染时 BlockItem 找不到 → fallback Empty。这里递归扫当前模块的
            // use,把孙组件也注册进同一个 registry,并同步收集其 WidgetDecl,
            // 让孙组件的 handler(model var 读写)也编译进单一 VM 模块。
            // Plan 049:没有后者,BlockItem 的 .ToggleCollapse handler 不存在,
            // 点击折叠静默失败(HandlerNotFound)。
            register_transitive_widgets(&module_path, &mut registry, &mut child_decls);

            // Recursively collect ALL declarations from this module + its
            // transitive `use` deps (full module load so an imported fn's
            // intra-module callees — e.g. build_month_grid → weekday_of — and
            // cross-module deps — e.g. back.api → back.db — both resolve).
            collect_module_imports(
                &module_path,
                &mut visited,
                &mut import_stmts,
                &mut seen_symbols,
                &mut import_session,
                override_scenario,
            );

            // Plan 339 Phase 4: populate import_scope aliases directly from the
            // use_scanner data. This is more reliable than parsing use statements.
            // use back.api: create_note → import_aliases["create_note"] = "api.create_note"
            let module_qualifier = use_stmt
                .module
                .split('.')
                .last()
                .unwrap_or(&use_stmt.module);
            for item in &use_stmt.items {
                let qualified = format!("{}.{}", module_qualifier, item);
                import_aliases.insert(item.clone(), qualified);
            }
        }
    }

    // 2c. Plan 401/VM-routing: load route-target pages (pages/{module}.at).
    // The vue codegen discovers these via `routes { "/" -> use bookshelf }`
    // and emits a router; the VM/iced path instead renders the matched page
    // widget inside `outlet`. But the VM loader (above) only follows top-level
    // `use` lines — `-> use X` inside `routes {}` is invisible to use_scanner.
    // So explicitly load each route's page module + register its widget, the
    // same way top-level `use` modules are handled above. This makes the page
    // widgets available for outlet to render (see aura_view_builder.rs Outlet).
    if let Some(file_path) = path {
        if let Some(ref routes_block) = root_decl.routes {
            let base_dir = std::path::Path::new(file_path)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            for route in &routes_block.routes {
                // Convention: `-> use bookshelf` maps to pages/bookshelf.at
                // (mirrors vue's @/pages/{module}.vue — see ast/route.rs:45-46).
                let page_module = format!("pages/{}", route.module);
                let module_path = resolve_module_path(base_dir, &page_module);
                let Some(module_path) = module_path else {
                    continue;
                };
                if let Ok(module_code) = std::fs::read_to_string(&module_path) {
                    // PLAN-037: 路由页读点同过前缀化 overlay——PLAN-712 T-17：
                    // 此前此处**裸读**（全部模块读点中唯一未包 apply 的），
                    // routes 页（pages/*.at）的相对 `/api/` 字面量不被改写 →
                    // fetch 解析到 app 自身 back origin（死端口）→ get_json
                    // 吞错返空 = 018 详情页「0 entries」（实机实证：顶层
                    // book_store.at 前缀生效、书架 2 REQ 到 proxy；pages/ 页
                    // 零 REQ + 86ms 秒拒）。
                    let module_code = crate::back_prefix::apply(&module_path, module_code);
                    let mod_session = CompilerSession::ui();
                    let mut mod_parser =
                        Parser::from(module_code.as_str()).with_session(mod_session);
                    if let Ok(mod_ast) = mod_parser.parse() {
                        // Register view-fn fragments from the page module too.
                        for stmt in &mod_ast.stmts {
                            if let crate::ast::Stmt::ViewFragmentDecl(frag) = stmt {
                                crate::aura::extract::register_view_fragment(frag);
                            }
                        }
                        for stmt in &mod_ast.stmts {
                            if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                                if let Ok(child_widget) =
                                    crate::aura::extract_widget_from_decl(decl)
                                {
                                    child_decls.push(decl.clone());
                                    // Plan 408: record route.module → widget.name
                                    // mapping so render_outlet can find the page.
                                    // Using a separate alias map (not a registry
                                    // entry) avoids shadowing built-in UI elements
                                    // like <button>, <input>, etc.
                                    registry
                                        .register_route_alias(&route.module, &child_widget.name);
                                    registry.register(child_widget);
                                }
                            }
                            // VM multi-store fix: also collect StoreDecls declared
                            // in the page's own source (e.g. home.at has
                            // `use article_store: ArticleStore` → article_store.at
                            // has the StoreDecl). Without this, stores only used by
                            // route pages never enter all_decls → their handlers
                            // are undefined at link time. This mirrors the root-AST
                            // store conversion (line ~2582).
                            if let crate::ast::Stmt::StoreDecl(store_decl) = stmt {
                                let name = store_decl.name.clone();
                                child_decls.push(crate::ast::ui::WidgetDecl {
                                    name: name.clone(),
                                    messages: store_decl.messages.clone(),
                                    model: store_decl.model.clone(),
                                    computed: store_decl.computed.clone(),
                                    view: None,
                                    named_views: Vec::new(),
                                    on: store_decl.on.clone(),
                                    bind: None,
                                    props: Vec::new(),
                                    routes: None,
                                    lifecycle: Vec::new(),
                                    style: None,
                                    ext_imports: Vec::new(),
                                    watch: Vec::new(),
                                    expose: Vec::new(),
                                    setup: None,
                                    actions: None,
                                    timer: store_decl.timer.clone(),
                                });
                            }
                        }
                        // Pull in the page's own `use` deps (e.g. a page doing
                        // `use store: BooksStore` or `use back.api: list_books`)
                        // so its handlers compile.
                        collect_module_imports(
                            &module_path,
                            &mut visited,
                            &mut import_stmts,
                            &mut seen_symbols,
                            &mut import_session,
                            override_scenario,
                        );
                    }
                }
            }
        }
    }

    // 3. Create DynamicComponent with registry + imported symbols
    // Plan 451 P3: 顶层 actions 回退链（widget 块未安装时）：本文件顶层
    // 声明 → import_stmts 里首个模块声明（use 传递闭包已在此前收齐）。
    if !dsl_actions_installed {
        let fallback = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::ActionsDecl(b) => Some(b.clone()),
                _ => None,
            })
            .or_else(|| {
                import_stmts.iter().find_map(|s| match s {
                    crate::ast::Stmt::ActionsDecl(b) => Some(b.clone()),
                    _ => None,
                })
            });
        if let Some(acts) = fallback {
            crate::ui::action_config::set_dsl_action_config_from_block(&acts, path);
        }
    }

    // Plan 370 D-GAP-4: collect StoreDecls and convert them to child WidgetDecls
    // so their state + handlers get compiled into the single VM module.
    // A store is isomorphic to a child widget minus a view — we give it an
    // empty view so the existing child-widget synthesis machinery handles it.
    let mut store_as_child_decls: Vec<crate::ast::WidgetDecl> = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::StoreDecl(store_decl) = stmt {
            // Convert StoreDecl → WidgetDecl with empty view
            let fake_widget = crate::ast::ui::WidgetDecl {
                name: store_decl.name.clone(),
                messages: store_decl.messages.clone(),
                model: store_decl.model.clone(),
                computed: store_decl.computed.clone(),
                view: None, // stores have no view
                named_views: Vec::new(),
                on: store_decl.on.clone(),
                bind: None,
                props: Vec::new(),
                routes: None,
                lifecycle: Vec::new(),
                style: None,
                ext_imports: Vec::new(),
                watch: Vec::new(),
                expose: Vec::new(),
                setup: None,
                actions: None,
                timer: store_decl.timer.clone(),
            };
            store_as_child_decls.push(fake_widget);
        }
    }
    // VM multi-store fix: also extract StoreDecls collected from route-page
    // imports (import_stmts now carries them via collect_module_imports).
    // Convert each to a WidgetDecl so its handlers compile into the VM module.
    // Dedup by store name against what root already collected.
    let root_store_names: std::collections::HashSet<String> = store_as_child_decls
        .iter()
        .map(|d| d.name.to_string())
        .collect();
    for stmt in &import_stmts {
        if let crate::ast::Stmt::StoreDecl(store_decl) = stmt {
            let sname = store_decl.name.to_string();
            if !root_store_names.contains(&sname) {
                store_as_child_decls.push(crate::ast::ui::WidgetDecl {
                    name: store_decl.name.clone(),
                    messages: store_decl.messages.clone(),
                    model: store_decl.model.clone(),
                    computed: store_decl.computed.clone(),
                    view: None,
                    named_views: Vec::new(),
                    on: store_decl.on.clone(),
                    bind: None,
                    props: Vec::new(),
                    routes: None,
                    lifecycle: Vec::new(),
                    style: None,
                    ext_imports: Vec::new(),
                    watch: Vec::new(),
                    expose: Vec::new(),
                    setup: None,
                    actions: None,
                    timer: store_decl.timer.clone(),
                });
            }
        }
    }
    // Plan 435 P8-6(D13):包组件接入桌面端(iced)—— `use { package: x from
    // "dir" }` 的 widget:视图注册进 WidgetRegistry(fold 桥接见
    // WidgetRegistry::get),decl 并入 child_decls 使 handler 编入单 VM。
    // 与 vue 侧(api.rs SFC 生成)同源同机制;加载失败仅告警不阻塞。
    {
        let base_dir = path
            .map(|p| {
                std::path::Path::new(p)
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
            })
            .unwrap_or(std::path::Path::new("."));
        let mut pkg_reg = crate::ui_gen::widget::ComponentRegistry::new();
        let mut seen_dirs: std::collections::HashSet<std::path::PathBuf> = Default::default();
        let mut sources: Vec<crate::aura::AuraWidget> = vec![widget.clone()];
        sources.extend(registry.all());
        for w in &sources {
            for imp in &w.ext_imports {
                if !matches!(imp.kind, crate::ast::ui::ExtImportKind::Package) {
                    continue;
                }
                let dir = std::path::PathBuf::from(imp.path.as_str());
                if !seen_dirs.insert(dir.clone()) {
                    continue;
                }
                // Plan 437 Phase 2:包路径基准与 vue 侧(api.rs)对齐 —— 页文件
                // 相对声明(`from "../components"`)在 pages/ 下解析;app 相对
                // 仍先行(根 widget 的声明形态)。两个候选逐一试,首个存在者
                // 生效,全部不存在才告警。
                let candidates = [base_dir.join(&dir), base_dir.join("pages").join(&dir)];
                let first = pkg_reg.load_package(&candidates[0], base_dir);
                let (loaded, loaded_dir) = match first {
                    Ok(p) => (Ok(p), Some(candidates[0].clone())),
                    Err(_) => match pkg_reg.load_package(&candidates[1], base_dir) {
                        Ok(p) => (Ok(p), Some(candidates[1].clone())),
                        Err(e) => (Err(e), None),
                    },
                };
                match loaded {
                    Ok(pkg) => {
                        for (d, aw) in &pkg.full_widgets {
                            child_decls.push(d.clone());
                            registry.register(aw.clone());
                        }
                        // Plan 522:包组件文件自身的 `use` 依赖也入编译单元。
                        // 此前仅根文件 use 链被收集——包组件 handler 引用的
                        // helper 模块(如 donut 的 chart_geom dc/ds)在 VM 侧
                        // 无定义;import_aliases 同步补齐(裸名 → 模块限定名,
                        // 与根文件 use 同规则)。
                        if let Some(pkg_dir) = loaded_dir {
                            if let Ok(entries) = std::fs::read_dir(&pkg_dir) {
                                for entry in entries.flatten() {
                                    let p = entry.path();
                                    if !p.extension().map(|e| e == "at").unwrap_or(false) {
                                        continue;
                                    }
                                    crate::collect_module_imports(
                                        &p,
                                        &mut visited,
                                        &mut import_stmts,
                                        &mut seen_symbols,
                                        &mut import_session,
                                        None,
                                    );
                                    if let Ok(code) = std::fs::read_to_string(&p) {
                                        for us in crate::use_scanner::scan_use_statements(&code) {
                                            let qualifier =
                                                us.module.split('.').last().unwrap_or(&us.module);
                                            for item in &us.items {
                                                import_aliases.insert(
                                                    item.clone(),
                                                    format!("{}.{}", qualifier, item),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "package `{}` load failed (VM): {} (tried {:?})",
                            imp.path,
                            e,
                            candidates
                        )
                    }
                }
            }
        }
    }
    // Merge store-as-child decls with actual child widget decls
    let mut all_child_decls = child_decls.clone();
    // Plan 442 A2: also dedup against child decls — a store can reach
    // child_decls directly (unified-form module branch above) AND via
    // import_stmts (legacy store-facade fallback); compiling the same
    // StoreDecl twice would collide in the single VM module.
    let child_decl_names: std::collections::HashSet<String> =
        all_child_decls.iter().map(|d| d.name.to_string()).collect();
    store_as_child_decls.retain(|d| !child_decl_names.contains(&d.name.to_string()));
    all_child_decls.extend(store_as_child_decls);

    // Plan 403: collect the root module's own top-level fn/type/enum
    // declarations so they're compiled into the VM alongside imported ones.
    // Without this, a handler calling a top-level fn (e.g. fn eval_expr in
    // 011-calculator) fails with "Undefined symbol".
    for stmt in &ast.stmts {
        match stmt {
            crate::ast::Stmt::Fn(_)
            | crate::ast::Stmt::TypeDecl(_)
            | crate::ast::Stmt::EnumDecl(_) => {
                import_stmts.push(stmt.clone());
            }
            _ => {}
        }
    }

    // Plan 442 A3 (ext link): `use.web` ext imports on the VM render target
    // (adapter-chain loading + platform stubs). See load_ext_imports_for_vm.
    if let Some(file_path) = path {
        let base_dir = std::path::Path::new(file_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let mut ext_widget_decls: Vec<crate::ast::ui::WidgetDecl> = Vec::new();
        let mut ext_adapter_paths: Vec<std::path::PathBuf> = Vec::new();
        load_ext_imports_for_vm(
            base_dir,
            &ast,
            &root_decl,
            &all_child_decls,
            &mut visited,
            &mut import_stmts,
            &mut seen_symbols,
            &mut import_session,
            override_scenario,
            &mut import_aliases,
            &mut ext_widget_decls,
            &mut ext_adapter_paths,
        )
        .map_err(|e| format!("ext imports (use.web) failed: {}", e))?;
        // PLAN-051 C4: adapter widget 注册——use.web component 的 VM widget
        // 形态（renderer.vm.at Markdown 纯文本降级）进视图 registry +
        // child_decls（handlers 一并编译进单 VM 模块）。
        for wd in &ext_widget_decls {
            if let Ok(w) = crate::aura::extract_widget_from_decl(wd) {
                // PLAN-642 R642-F3: `.Tick` handler + `interval` 模型变量的
                // 子件（027/012/025 惯用法——Init 轻量、重 FS/时钟活延后到
                // Tick）在合并 VM 无 tick 源：根件（画廊宿主）无 tick →
                // view.tick_interval 订阅不存在，子件 Tick 永不派发（027
                // 内嵌永久"正在加载..."实证；chart 的 timer{} 块另有 051 C7
                // 通路不受影响）。按 051 C7 timer 条目形态为该子件合成
                // `Tick (every_ms: interval)` 定时条目，走既有 fire_timer
                // 门控派发（handler_<Widget>_Tick 语义不变）。
                let mut wd_owned = wd.clone();
                if let Some(ms) = w.tick_interval {
                    let has_tick_timer = wd_owned
                        .timer
                        .as_ref()
                        .map(|tb| tb.entries.iter().any(|e| e.event.as_str() == "Tick"))
                        .unwrap_or(false);
                    if !has_tick_timer {
                        let mut tb = crate::ast::ui::TimerBlock {
                            entries: Vec::new(),
                        };
                        tb.entries.push(crate::ast::ui::TimerEntry {
                            event: crate::ast::Name::from("Tick"),
                            every_ms: ms.max(16) as u64,
                            when: None,
                        });
                        wd_owned.timer = Some(tb);
                    }
                }
                all_child_decls.push(wd_owned);
                registry.register(w);
            }
        }
        // PLAN-633: demo 适配器的传递子 widget 注册——Demo013Todo 视图里的
        // TodoList 等经适配器自身的 `use` 引入，C4 只注册适配器本体 widget；
        // 不补传递注册则子件 registry miss → fallback Empty（013 内嵌列表区
        // 空白实证）。stores 由下方 632-F1 晚到补转换覆盖，此处只补 widget。
        {
            let mut tw_visited: std::collections::HashSet<std::path::PathBuf> =
                std::collections::HashSet::new();
            for p in &ext_adapter_paths {
                register_transitive_widgets_inner(
                    p,
                    &mut registry,
                    &mut all_child_decls,
                    &mut tw_visited,
                );
            }
        }
        // PLAN-642 R642-F2: use.web 适配器自带 `use { package: X from "dir" }`
        // 的包组件注册。上方的包装载走查（435 P8-6）只覆盖根件——彼时
        // registry 尚无适配器 widget，适配器的包导入永远错过装载且无任何
        // 告警 → 包内 widget 全部缺注册，实例落 builtin 桩（024 chart 内嵌
        // 空画布实证；独立臂根件自带包导入 → 同语料正常出图）。适配器注册
        // 后补一轮包装载：dir 优先相对适配器文件目录（ext_adapter_paths
        // 同源），回退根 base_dir；包组件 fn 模块链按 435 P8-6 同款收集。
        {
            let mut pkg_seen_dirs: std::collections::HashSet<std::path::PathBuf> =
                Default::default();
            let mut pkg_reg = crate::ui_gen::widget::ComponentRegistry::new();
            for wd in &ext_widget_decls {
                for imp in &wd.ext_imports {
                    if !matches!(imp.kind, crate::ast::ui::ExtImportKind::Package) {
                        continue;
                    }
                    let dir = std::path::PathBuf::from(imp.path.as_str());
                    if !pkg_seen_dirs.insert(dir.clone()) {
                        continue;
                    }
                    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
                    for adapter_path in &ext_adapter_paths {
                        if let Some(adapter_dir) = adapter_path.parent() {
                            candidates.push(adapter_dir.join(&dir));
                        }
                    }
                    candidates.push(base_dir.join(&dir));
                    let mut loaded: Option<(
                        Vec<(crate::ast::ui::WidgetDecl, crate::aura::AuraWidget)>,
                        std::path::PathBuf,
                    )> = None;
                    for cand in &candidates {
                        if cand.is_dir() {
                            if let Ok(pkg) = pkg_reg.load_package(cand, base_dir) {
                                // load_package 返回注册表内缓存引用（&mut self
                                // 借续）——立即抽走所需列表，避免借用跨迭代。
                                loaded = Some((pkg.full_widgets.clone(), cand.clone()));
                                break;
                            }
                        }
                    }
                    match loaded {
                        Some((pkg_full, loaded_dir)) => {
                            for (d, aw) in &pkg_full {
                                all_child_decls.push(d.clone());
                                registry.register(aw.clone());
                            }
                            if let Ok(entries) = std::fs::read_dir(&loaded_dir) {
                                for entry in entries.flatten() {
                                    let p = entry.path();
                                    if !p.extension().map(|e| e == "at").unwrap_or(false) {
                                        continue;
                                    }
                                    crate::collect_module_imports(
                                        &p,
                                        &mut visited,
                                        &mut import_stmts,
                                        &mut seen_symbols,
                                        &mut import_session,
                                        None,
                                    );
                                    if let Ok(code) = std::fs::read_to_string(&p) {
                                        for us in crate::use_scanner::scan_use_statements(&code) {
                                            let qualifier =
                                                us.module.split('.').last().unwrap_or(&us.module);
                                            for item in &us.items {
                                                import_aliases.insert(
                                                    item.clone(),
                                                    format!("{}.{}", qualifier, item),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        None => log::warn!(
                            "package `{}` load failed (VM adapter) (tried {:?})",
                            imp.path,
                            candidates
                        ),
                    }
                }
            }
        }
        // PLAN-632 F1: 装载顺序缺陷收口——use.web demo 适配器链带来的
        // StoreDecl 在 load_ext_imports_for_vm 内才进 import_stmts，晚于
        // 上方 store→child 转换位（Plan 370 D-GAP-4 / 多 store fix：根
        // AST + 当时的 import_stmts）。晚到的 store（016 的 CalendarStore
        // 经 Demo*.at 适配器链）错过转换 → model 不并入统一根态、handler
        // 不编译（synthesize 的 all_decls 只含 decl+child_decls）→
        // `.store.*` 全落空 + `store.X()` 派发失联。此处按名去重补转换。
        {
            let known: std::collections::HashSet<String> =
                all_child_decls.iter().map(|d| d.name.to_string()).collect();
            for stmt in &import_stmts {
                if let crate::ast::Stmt::StoreDecl(store_decl) = stmt {
                    let sname = store_decl.name.to_string();
                    if !known.contains(&sname) {
                        all_child_decls.push(store_decl_as_widget_decl(store_decl));
                    }
                }
            }
        }
    }

    // Plan 340: AUTO_VM_MERGE=0 (i.e. --no-merge) enables API-over-HTTP:
    // cross-module `#[api]` calls are rewritten at codegen time into HTTP
    // requests instead of in-process direct calls. Default (merge) keeps the
    // existing in-process behavior.
    // Plan 057 (ash-gui): AUTO_BACKEND (e.g. http://127.0.0.1:3000) also
    // enables the rewrite — the VM app then talks to a real backend over HTTP
    // (run/complete/history/prompt_context/jobs), not the in-process mock.
    // emit_api_http_call prefers AUTO_BACKEND as the base URL.
    let api_over_http = std::env::var("AUTO_VM_MERGE").as_deref() == Ok("0")
        || std::env::var("AUTO_BACKEND")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
    // PLAN-050 T7 (C5): use.web component 名单注册——builder 图标组件臂的
    // 生产数据源（import_stmts 不含 UseWeb,Plan 442 ext 流单独消费）。每次
    // 装载整体替换。all_child_decls 此处已定型（child 装载循环已收尾）。
    // 来源三路:①根 AST 顶层 use.web（musk app.at 形态）②widget 内嵌
    // ext_imports（decl 携带形态）③子模块文件顶层 use.web——PLAN-051 P2-①
    // 补：chats_view.at/nav_item.at 的模块级声明此前两路均不覆盖 → 图标
    // 组件 unknown fallback → Empty（icon 按钮空白现场）。沿 ext_stubs 的
    // visited 重解析模式（文件小、装载一次）。
    {
        let mut names: Vec<String> = Vec::new();
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::UseWeb(entries) = stmt {
                for e in entries {
                    if matches!(e.kind, crate::ast::ui::ExtImportKind::Component) {
                        names.extend(e.symbols.iter().map(|n| n.to_string()));
                    }
                }
            }
        }
        for decl in std::iter::once(&root_decl).chain(all_child_decls.iter()) {
            for ext in &decl.ext_imports {
                if matches!(ext.kind, crate::ast::ui::ExtImportKind::Component) {
                    names.extend(ext.symbols.iter().map(|n| n.to_string()));
                }
            }
        }
        // ③ 子模块顶层 use.web（visited 为 collect_module_imports 的已装载集）。
        for path in visited.iter() {
            if path.extension().and_then(|e| e.to_str()) != Some("at") {
                continue;
            }
            let Ok(code) = std::fs::read_to_string(path) else {
                continue;
            };
            let code = crate::back_prefix::apply(path, code);
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::Parser::from(code.as_str()).with_session(session);
            let Ok(mod_ast) = parser.parse() else {
                continue;
            };
            for stmt in &mod_ast.stmts {
                if let crate::ast::Stmt::UseWeb(entries) = stmt {
                    for e in entries {
                        if matches!(e.kind, crate::ast::ui::ExtImportKind::Component) {
                            names.extend(e.symbols.iter().map(|n| n.to_string()));
                        }
                    }
                }
            }
        }
        crate::ui::aura_view_builder::register_imported_components(names);
    }
    let mut comp = DynamicComponent::with_registry_and_imports_from_decls(
        &root_decl,
        &all_child_decls,
        &widget,
        registry,
        import_stmts,
        &import_aliases,
        api_over_http,
    )
    .map_err(|e| format!("DynamicComponent init failed: {}", e))?;

    // 3b. Set source path for hot-reload tracking
    if let Some(p) = path {
        comp.set_source_path(p);
    }

    // 3c. Fire .Init lifecycle event before starting Iced loop.
    //     Init is now a regular VM handler (handler_Init), dispatched
    //     unconditionally via call_handler.
    comp.fire_init();

    Ok(comp)
}

/// Plan 459：按 `auto run` 同一管线构造 DynamicComponent（不启动 iced 循环）。
/// 双窗口 demo（examples/ui_dual_app）等宿主复用本入口装配多个 App。
#[cfg(feature = "ui-iced")]
pub fn build_dynamic_component(
    code: &str,
    path: Option<&str>,
) -> AutoResult<crate::ui::dynamic::DynamicComponent> {
    build_dynamic_component_inner(code, path, None)
}

#[cfg(feature = "ui-iced")]
fn run_file_dynamic_ui_inner(
    code: &str,
    path: Option<&str>,
    override_scenario: Option<&crate::session::CompilerSession>,
) -> AutoResult<String> {
    use crate::ui::iced::run_dynamic_iced;
    // PLAN-779: UI 宿主能力装配——`auto run` UI 入口在组件/VM 构造前注册
    // frame provider 进程默认（覆盖库消费者不经 CLI main 的形态；-q
    // rqhost 客户端在 build 之后分岔，同样受益于此处注册）。
    crate::ui::frame_provider::install_process_default();
    // PLAN-785: clipboard text adapter 同点并入（if-absent 兼容默认）。
    #[cfg(feature = "ui-clipboard")]
    crate::ui::clipboard_provider::install_process_default();
    // PLAN-790: clipboard media adapter 同点并入（if-absent 兼容默认；
    // windows+native-clipboard 双门控——非 Windows 不装配）。
    #[cfg(all(windows, feature = "native-clipboard"))]
    crate::ui::clipboard_media_provider::install_process_default();
    // PLAN-794: scroll host adapter 同点并入（if-absent 兼容默认；ui 单
    // 门控——controller 注册表接线恢复，9900..9905/9922/9923）。
    crate::ui::scroll_host_provider::install_process_default();
    // PLAN-808: terminal adapter 同点并入（if-absent 兼容默认；ui 单门控——
    // 34 native 全族的输入/投影/menu/scroll/window/palette 接线恢复）。
    crate::ui::terminal_provider::install_process_default();
    // PLAN-802: code editor host adapter 同点并入（if-absent 兼容默认；
    // code-editor 单门控——editor 注册表接线恢复，22 native 全族）。
    #[cfg(feature = "code-editor")]
    crate::ui::code_editor_provider::install_process_default();
    // PLAN-804: dialog adapter 同点并入（if-absent 兼容默认；ui-dialog
    // 单门控——host_adapters 落点，2927/2928 parented 策略接线恢复）。
    #[cfg(feature = "ui-dialog")]
    crate::host_adapters::dialog::install_process_default();
    // PLAN-804: dnd adapter 同点并入（if-absent 兼容默认；windows+
    // native-dnd 双门控，本函数在 ui-iced 门内=有效三分，2938 接线恢复）。
    #[cfg(all(windows, feature = "native-dnd"))]
    crate::ui::dnd_host_provider::install_process_default();
    let comp = build_dynamic_component_inner(code, path, override_scenario)?;
    // PLAN-031 T-05：-q vm 轨分岔（env 门）——装载链零改：CWD/主题/后端
    // 序由 run_vm_ui 上游照常执行（组件已按同链装载），仅"自开 OS 窗"
    // 换 rqhost 客户端（build_dynamic_component 后、run_dynamic_iced
    // 前——§5.1 D6 定案位）。env 值 = well-known 管道名（rendezvous
    // 采纳内建在 ClientTarget::Rqhost——不预连不烧一次性管道实例）。
    if let Ok(pipe) = std::env::var("AUTO_RQHOST_PIPE") {
        return crate::ui::desktop_protocol::rqhost::run_vm_rqhost_client(comp, &pipe)
            .map_err(crate::error::AutoError::Msg);
    }
    // 4. Run iced (blocks until all windows close; plan-459 daemon semantics)
    run_dynamic_iced(comp).map_err(|e| crate::error::AutoError::Msg(format!("{}", e)))
}

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

    // Plan 227: Detect UI keywords and run with iced backend
    #[cfg(feature = "ui-iced")]
    if has_ui_keywords(&code) {
        return run_file_dynamic_ui(&code, Some(path));
    }

    // Plan 327: True multi-module loading via CompileSession. The source
    // file's directory is added to source_dirs, so `use db` resolves db.at
    // relative to the source. No string flattening — modules are compiled
    // separately and linked by the Linker.
    run_with_path(&code, path)
}

/// Run code with a source file path for module resolution.
fn run_with_path(code: &str, path: &str) -> AutoResult<String> {
    let code = code.to_string();
    let path = path.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
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
    crate::install_ui_host_capabilities();
    use crate::vm::codegen::Codegen;
    use crate::vm::debugger::GdbController;
    use crate::vm::engine::AutoVM;
    use crate::vm::loader::Linker;
    use crate::vm::opcode::OpCode;
    use crate::vm::virt_memory::VirtualFlash;

    // Plan 442 C2: axum serve adapter state reset (see execute_autovm_with_path).
    crate::vm::ffi::axum_adapter::reset();
    // PLAN-669: #[api] param-sig side channel — same reset discipline.
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
    crate::install_ui_host_capabilities();
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

/// Transpile AutoLang file to C (legacy implementation)
#[deprecated(since = "0.1.0", note = "use trans_c instead")]
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
    use crate::ui_gen::{BackendGenerator, JetGenerator, RustGenerator, VueGenerator};

    // Parse scenario
    let session = match scenario {
        "ui" => CompilerSession::ui().with_backend(backend),
        "core" => CompilerSession::default(),
        "shell" => CompilerSession::shell(),
        _ => CompilerSession::ui().with_backend(backend),
    };

    // Read input file
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // PLAN-635: pre-register use-imported recipes before the parse (parser
    // symbol-checker hook), then replay after the parse.
    let base_dir = std::path::Path::new(path)
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let recipe_imports =
        crate::design_tokens::recipe::prepare_style_recipe_imports(&base_dir, &code)
            .map_err(|e| e.to_string())?;

    // Parse with scenario
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session.clone());
    let ast = parser
        .parse()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // Extract AURA widgets from AST
    crate::aura::extract::clear_view_fragments();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::ViewFragmentDecl(frag) = stmt {
            crate::aura::extract::register_view_fragment(frag);
        }
    }
    // PLAN-607: register and validate style recipes before extracting widget.
    // PLAN-635: replay imported (source-tagged) + local recipes in order.
    crate::design_tokens::recipe::load_and_validate_style_recipes_with_imports(
        &recipe_imports,
        &ast.stmts,
    )
    .map_err(|e| e.to_string())?;
    let mut widgets = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            // Convert WidgetDecl to AuraWidget
            let aura_widget =
                crate::aura::extract_widget_from_decl(widget_decl).map_err(|e| e.to_string())?;
            widgets.push(aura_widget);
        }
    }

    if widgets.is_empty() {
        return Err("No widget or store declarations found in input file".into());
    }

    // Generate code based on backend
    // Rust backend uses auto-ui abstract components (Iced handled by auto-ui crate)
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
            let out_path = std::path::Path::new(out_dir).join(format!("{}.{}", widget.name, ext));
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
pub fn ui_build_shadcn(path: &str, output: Option<&str>) -> AutoResult<String> {
    use crate::session::CompilerSession;
    use crate::ui_gen::{BackendGenerator, VueGenerator, VueMode};

    // Read input file
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // PLAN-635: pre-register use-imported recipes before the parse (parser
    // symbol-checker hook), then replay after the parse.
    let base_dir = std::path::Path::new(path)
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let recipe_imports =
        crate::design_tokens::recipe::prepare_style_recipe_imports(&base_dir, &code)
            .map_err(|e| e.to_string())?;

    // Parse with UI scenario
    let session = CompilerSession::ui().with_backend("vue");
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session);
    let ast = parser
        .parse()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // Extract AURA widgets from AST
    crate::aura::extract::clear_view_fragments();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::ViewFragmentDecl(frag) = stmt {
            crate::aura::extract::register_view_fragment(frag);
        }
    }
    // PLAN-607: register and validate style recipes before extracting widget.
    // PLAN-635: replay imported (source-tagged) + local recipes in order.
    crate::design_tokens::recipe::load_and_validate_style_recipes_with_imports(
        &recipe_imports,
        &ast.stmts,
    )
    .map_err(|e| e.to_string())?;
    let mut widgets = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget =
                crate::aura::extract_widget_from_decl(widget_decl).map_err(|e| e.to_string())?;
            widgets.push(aura_widget);
        }
    }

    if widgets.is_empty() {
        return Err("No widget or store declarations found in input file".into());
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
            let out_path = std::path::Path::new(out_dir).join(format!("{}.vue", widget.name));
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
/// Extract API function names from `use back.api: fn1, fn2, ...` statements in an AST
fn extract_api_imports_from_ast(code: &crate::ast::Code) -> Vec<String> {
    let mut api_imports = Vec::new();
    for stmt in &code.stmts {
        if let crate::ast::Stmt::Use(ref use_stmt) = stmt {
            if is_api_use_stmt(use_stmt) {
                api_imports.extend(use_stmt.items.iter().map(|s| s.as_str().to_string()));
            }
        }
    }
    api_imports
}

/// Check if a `use` statement targets `back.api`
fn is_api_use_stmt(use_stmt: &crate::ast::Use) -> bool {
    // Check legacy paths: ["back", "api"]
    if use_stmt.paths.len() == 2
        && use_stmt.paths[0].as_str() == "back"
        && use_stmt.paths[1].as_str() == "api"
    {
        return true;
    }
    // Check module_path: display == "back.api"
    if let Some(ref mp) = use_stmt.module_path {
        if mp.display() == "back.api" {
            return true;
        }
    }
    false
}

// PLAN-815 T-04 R1-F03: store-deps 提取族迁 auto-ui-compiler、注释剥离
// 单源在 SDK——三个历史根级公开路径经转发保持（消费者零改动）。
pub use auto_lang_core::strip_line_block_comments;
pub use auto_ui_compiler::store_deps::{
    extract_store_deps_from_file, extract_store_imports_from_ast,
};

/// // Check if any widget has routes
/// let has_routes = widgets.iter().any(|w| w.routes.is_some());
/// ```
pub fn ui_build_shadcn_with_widgets(
    path: &str,
    output: Option<&str>,
) -> AutoResult<(String, Vec<crate::aura::AuraWidget>)> {
    let (vue_code, widgets, _stores) =
        ui_build_shadcn_with_widgets_and_stores(path, output, None, None, None)?;
    Ok((vue_code, widgets))
}

/// Plan 043 store-codegen: same as `ui_build_shadcn_with_widgets`, but also
/// returns the generated store composable files `(filename, code)`. Use this
/// when compiling multiple .at files in a loop and you need to collect stores
/// explicitly (the `STORE_EXTRA_FILES` thread-local is cleared per call).
///
/// `root_dir` (Plan 043 stream phase): when provided, streaming endpoints
/// (`~Stream<T>`) are resolved from `back/api.at` so the store composable can
/// wire type-driven SSE. Pass the project root (where `src/back/api.at` lives).
///
/// `shadcn` (Plan 013): `None`/`Some(true)` = shadcn-vue mapping (default);
/// `Some(false)` = native HTML elements, no `@/components/ui/*` imports.
///
/// `default_classes` (Plan 014): `None`/`Some(true)` = inject doc-theme
/// default Tailwind classes (default); `Some(false)` = skip them except for
/// structural layout primitives (row/col/grid/...).
pub fn ui_build_shadcn_with_widgets_and_stores(
    path: &str,
    output: Option<&str>,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
) -> AutoResult<(String, Vec<crate::aura::AuraWidget>, Vec<(String, String)>)> {
    use crate::ui_gen::{generate_component_from_file, ComponentGenOptions, VueGenerator, VueMode};

    let at_path = std::path::Path::new(path);
    let stream_endpoints = root_dir.map(crate::ui_gen::api::resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        stream_endpoints,
        shadcn,
        default_classes,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    let store_composables = result.store_composables.clone();

    // Plan 012 Batch A: surface codegen validation warnings on the build path
    // (deduplicated — auto-man pre-scans each file before the real pass).
    crate::ui_gen::validators::print_warnings_once(path, &result.validation_warnings);

    // Write output if specified (legacy behavior)
    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for (name, code) in &result.all_widget_codes {
            let out_path = std::path::Path::new(out_dir).join(format!("{}.vue", name));
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
        // Also write store composables
        for (filename, code) in &result.store_composables {
            let out_path = std::path::Path::new(out_dir).join(filename);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&out_path, code).ok();
        }
    }

    Ok((result.vue_code, result.widgets, store_composables))
}

/// Plan 408 P11 / KNOWN-DEBT pages 写盘：薄包装，返回完整 `GeneratedComponent`
/// （含 `all_widget_codes`——每个 component fn / widget 一个 SFC），让调用方
/// （auto-man `from_workspace` 的 pages 路径）能把 component fn SFC 也写盘到
/// components/，而非只取 first widget 的 vue_code。与
/// `ui_build_shadcn_with_widgets_and_stores` 同语义，只是不丢弃 all_widget_codes。
pub fn ui_build_shadcn_all_widget_codes(
    path: &str,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
) -> AutoResult<crate::ui_gen::GeneratedComponent> {
    ui_build_shadcn_all_widget_codes_with_bound(path, root_dir, shadcn, default_classes, None)
}

/// Plan 443: `ui_build_shadcn_all_widget_codes` + the workspace-aggregated
/// bound-channel map (which model channels some parent actually binds).
pub fn ui_build_shadcn_all_widget_codes_with_bound(
    path: &str,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
    bound_model_channels: Option<std::collections::HashMap<String, Vec<String>>>,
) -> AutoResult<crate::ui_gen::GeneratedComponent> {
    use crate::ui_gen::{generate_component_from_file, ComponentGenOptions};
    let at_path = std::path::Path::new(path);
    let stream_endpoints = root_dir.map(crate::ui_gen::api::resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        stream_endpoints,
        shadcn,
        default_classes,
        bound_model_channels,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    crate::ui_gen::validators::print_warnings_once(path, &result.validation_warnings);
    Ok(result)
}

/// Like `ui_build_shadcn_with_widgets`, but accepts a list of known sub-widget names
/// to avoid naming collisions with shadcn-vue components (e.g. custom "Sidebar" widget
/// should not be mapped to shadcn's Sidebar).
pub fn ui_build_shadcn_with_sub_widgets(
    path: &str,
    output: Option<&str>,
    sub_widget_names: Vec<String>,
) -> AutoResult<(String, Vec<crate::aura::AuraWidget>)> {
    let (vue_code, widgets, _stores) = ui_build_shadcn_with_sub_widgets_and_stores(
        path,
        output,
        sub_widget_names,
        None,
        None,
        None,
    )?;
    Ok((vue_code, widgets))
}

/// Plan 043 store-codegen: same as `ui_build_shadcn_with_sub_widgets`, but also
/// returns the generated store composable files `(filename, code)`. Callers that
/// need to write store files explicitly (e.g. `VueProject::from_workspace`) use
/// this variant instead of relying on the `STORE_EXTRA_FILES` thread-local,
/// which is cleared at the start of every `generate_component_from_file` call
/// and thus loses stores when multiple .at files are compiled in sequence.
///
/// `root_dir` (Plan 043 stream phase): when provided, streaming endpoints
/// (`~Stream<T>`) are resolved from `back/api.at` for type-driven SSE wiring.
///
/// `shadcn` (Plan 013): `None`/`Some(true)` = shadcn-vue mapping (default);
/// `Some(false)` = native HTML elements, no `@/components/ui/*` imports.
///
/// `default_classes` (Plan 014): `None`/`Some(true)` = inject doc-theme
/// default Tailwind classes (default); `Some(false)` = skip them except for
/// structural layout primitives (row/col/grid/...).
/// PLAN-037 T5: `ui_build_shadcn_with_sub_widgets_and_stores` + the
/// cross-file sub-widget model-var map (call-site model addressing). The
/// legacy entry delegates here with `None`.
pub fn ui_build_shadcn_with_sub_widgets_and_stores_full(
    path: &str,
    output: Option<&str>,
    sub_widget_names: Vec<String>,
    sub_widget_models: Option<std::collections::HashMap<String, Vec<String>>>,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
    bound_model_channels: Option<std::collections::HashMap<String, Vec<String>>>,
    sub_widget_msgs: Option<std::collections::HashMap<String, Vec<String>>>,
) -> AutoResult<(
    String,
    Vec<crate::aura::AuraWidget>,
    Vec<(String, String)>,
    Vec<(String, String, String)>,
)> {
    // PLAN-024：第四返回段 = named_view_codes（widget, view, SFC code）——
    // 桌面 vue 宿主 Mini.vue 产物源（仅本 fn 的唯一调用方 auto-man 消费）。
    use crate::ui_gen::{generate_component_from_file, ComponentGenOptions};

    let at_path = std::path::Path::new(path);
    let stream_endpoints = root_dir.map(crate::ui_gen::api::resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        sub_widgets: Some(sub_widget_names),
        sub_widget_models,
        bound_model_channels,
        stream_endpoints,
        shadcn,
        default_classes,
        sub_widget_msgs,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    let store_composables = result.store_composables.clone();

    crate::ui_gen::validators::print_warnings_once(path, &result.validation_warnings);

    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for (name, code) in &result.all_widget_codes {
            let out_path = std::path::Path::new(out_dir).join(format!("{}.vue", name));
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
        for (filename, code) in &result.store_composables {
            let out_path = std::path::Path::new(out_dir).join(filename);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write store file: {}", e))?;
        }
    }

    Ok((
        result.vue_code,
        result.widgets,
        store_composables,
        result.named_view_codes,
    ))
}

pub fn ui_build_shadcn_with_sub_widgets_and_stores(
    path: &str,
    output: Option<&str>,
    sub_widget_names: Vec<String>,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
) -> AutoResult<(String, Vec<crate::aura::AuraWidget>, Vec<(String, String)>)> {
    use crate::ui_gen::{generate_component_from_file, ComponentGenOptions};

    let at_path = std::path::Path::new(path);
    let stream_endpoints = root_dir.map(crate::ui_gen::api::resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        sub_widgets: Some(sub_widget_names),
        stream_endpoints,
        shadcn,
        default_classes,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    let store_composables = result.store_composables.clone();

    // Plan 012 Batch A: surface codegen validation warnings (deduplicated).
    crate::ui_gen::validators::print_warnings_once(path, &result.validation_warnings);

    // Write output if specified (legacy behavior)
    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for (name, code) in &result.all_widget_codes {
            let out_path = std::path::Path::new(out_dir).join(format!("{}.vue", name));
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
        for (filename, code) in &result.store_composables {
            let out_path = std::path::Path::new(out_dir).join(filename);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&out_path, code).ok();
        }
    }

    Ok((result.vue_code, result.widgets, store_composables))
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

// Plan 370 shared test support: builds the real 015-notes DynamicComponent.
// Must precede the plan370_* test modules that depend on it.
// These require the `ui-iced` feature (crate::ui::* imports), so gate them
// to avoid breaking lib-test compilation without that feature.
#[cfg(all(test, feature = "ui-iced"))]
mod plan370_test_support;

// =============================================================================
// Plan 015: AutoUI Core (feature-gated)
// =============================================================================

#[cfg(feature = "ui")]
pub mod ui;

// PLAN-804: 宿主 OS 能力适配层（vm core 与 ui renderer 门之外的第三
// 落点——首员 = ui-dialog 单门控的 OS 文件对话框）。crate 根是宿主接线
// 的合法层；vm core 经 vm::host_capabilities provider 接口消费，不直引
// 本层。
pub mod host_adapters;

// Re-export UI types when feature is enabled
#[cfg(feature = "ui")]
pub use ui::{
    view_to_vtree, App, AppResult, Component, Style, VNode, VNodeId, VNodeKind, VNodeProps, VTree,
    View, ViewBuilder,
};

#[cfg(feature = "ui-interpreter")]
pub use ui::{
    event_router::{EventContext, EventRouter, EventType},
    hot_reload::{HotReloadComponent, UIWatcher},
    interpreter::{DynamicMessage, InterpreterBridge},
};

#[cfg(feature = "ui-iced")]
pub use ui::iced::{ComponentIced, IntoIcedElement};
// =============================================================================
// Plan 446 批一 (C1-2): parse 错误定位换算回归
// =============================================================================
#[cfg(test)]
mod plan446_batch1_tests {
    use super::*;

    #[test]
    fn positioned_parse_errors_reports_line_col() {
        // 第 2 行制造语法错误(`let` 缺名) —— 诊断必须带 2:N 行列,而非裸消息。
        let src = "fn ok() int {\n    let = 1\n    return 1\n}\n";
        let mut cs = crate::compile::CompileSession::new();
        let mut parser = Parser::new_with_type_store(src, cs.type_store())
            .with_session(crate::session::CompilerSession::ui());
        let err = parser.parse().expect_err("broken source must fail");
        let detail = positioned_parse_errors(&err, src);
        assert!(
            detail.contains("at 2:"),
            "expected line-2 position in: {detail}"
        );
    }

    #[test]
    fn positioned_parse_errors_multi_error_cap() {
        // 多错误(MultipleErrors)展开且不超过 20 条。
        let mut src = String::new();
        for i in 0..30 {
            src.push_str(&format!("let broken{i} =\n")); // 每行一个残缺 stmt
        }
        let mut cs = crate::compile::CompileSession::new();
        let mut parser = Parser::new_with_type_store(src.as_str(), cs.type_store())
            .with_session(crate::session::CompilerSession::ui());
        if let Err(err) = parser.parse() {
            let detail = positioned_parse_errors(&err, &src);
            let count = detail.lines().filter(|l| l.contains("error")).count();
            assert!(count >= 1 && count <= 20, "cap violated: {count}\n{detail}");
        }
    }
}

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

// os-007（origin PLAN-577/P534-D4）：avatar 家族渲染探针。
#[cfg(test)]
#[path = "tests/plan577_avatar_tests.rs"]
mod plan577_avatar_tests;

// os-007（origin PLAN-577/P530-D1）：widget 自名折叠环守卫回归。
#[cfg(test)]
#[path = "tests/plan577_breadcrumb_cycle_tests.rs"]
mod plan577_breadcrumb_cycle_tests;

// P499-6 清偿:widgets-gallery 全页可编译冒烟(lib 级,入日常门禁)。
#[cfg(test)]
#[path = "tests/gallery_pages_compile_tests.rs"]
mod gallery_pages_compile_tests;

// PLAN-614 临时探针:VM 轨模块 fn 对 Obj 嵌套 children/List/bool 参数行为
// 隔离(tree 组件族 VM 端 first-light 排障;定位后可退役)。
#[cfg(test)]
#[path = "tests/p614_tree_vm_probe.rs"]
mod p614_tree_vm_probe;

// PLAN-618 探针:VM 轨模块 fn 递归(filter_tree)能力验证。
#[cfg(test)]
#[path = "tests/p618_filter_tree_probe.rs"]
mod p618_filter_tree_probe;

// PLAN-632: 画廊内嵌 demo 的模块组件桥接（store 装配顺序 + dep item
// 文件解析 + 适配器链 widget 注册）回归。
#[cfg(all(test, feature = "ui-iced"))]
#[path = "tests/plan632_demo_bridge_tests.rs"]
mod plan632_demo_bridge_tests;

// PLAN-664 U-4：use 导入静默死面报错化负例（P-15 模块解析失败 /
// P-16 导入符号 × model 字段撞名）。
#[cfg(all(test, feature = "ui-iced"))]
#[path = "tests/plan664_use_diag_tests.rs"]
mod plan664_use_diag_tests;

// PLAN-633: 内嵌全栈 demo 数据面（store → #[api] → db 模块种子/写路径）
// 回归。
#[cfg(all(test, feature = "ui-iced"))]
#[path = "tests/plan633_fullstack_embed_tests.rs"]
mod plan633_fullstack_embed_tests;

// PLAN-658 T-01: back proxy 骨架回归（真 TCP，cargo th 档；测试名带
// http_e2e 前缀以被 `cargo th` 的名字过滤器收录）。
#[cfg(all(test, feature = "test-http-e2e"))]
#[path = "tests/back_proxy_tests.rs"]
mod back_proxy_tests;

// PLAN-707 T-06/T-07: 外部流端到端 relay 与取消生命周期（真 TCP，cargo th
// 串行池；测试名带 http_e2e+plan707 双前缀以被 th 过滤器与交集命令收录）。
#[cfg(all(test, feature = "test-http-e2e"))]
#[path = "tests/plan707_stream_e2e_tests.rs"]
mod plan707_stream_e2e_tests;

// PLAN-042 T-08：ScanProbe 最小复现（D6 腿）——get_json 延续值 = body
// 本体回归钉（生产装载管线 + 进程内真 HTTP 端点）。
#[cfg(test)]
#[path = "tests/plan042_scan_probe_tests.rs"]
mod plan042_scan_probe_tests;

// ---- PLAN-814：run 入口族的上层装配包装（原三处隐式扇入收敛点） ----
// ui/ui-dialog 构建下，run 管线先装兼容宿主能力（if-absent 补缺，不覆盖
// 显式默认——779/785 三级装配契约），再委托 SDK 实现；纯 SDK 消费者无
// UI 装配（install 为 no-op hook 未注册形态）。
#[cfg(any(feature = "ui", feature = "ui-dialog"))]
pub fn run(code: &str) -> auto_lang_core::AutoResult<String> {
    install_ui_host_capabilities();
    auto_lang_core::run(code)
}

#[cfg(any(feature = "ui", feature = "ui-dialog"))]
pub fn run_with_capture(code: &str) -> auto_lang_core::AutoResult<(String, String)> {
    install_ui_host_capabilities();
    auto_lang_core::run_with_capture(code)
}

#[cfg(any(feature = "ui", feature = "ui-dialog"))]
pub fn run_with_capture_and_path(
    code: &str,
    path: &str,
) -> auto_lang_core::AutoResult<(String, String)> {
    install_ui_host_capabilities();
    auto_lang_core::run_with_capture_and_path(code, path)
}

#[cfg(any(feature = "ui", feature = "ui-dialog"))]
pub fn run_with_capture_and_path_staged(
    code: &str,
    path: &str,
) -> Result<(String, String), auto_lang_core::StagedRunFailure> {
    install_ui_host_capabilities();
    auto_lang_core::run_with_capture_and_path_staged(code, path)
}

#[cfg(any(feature = "ui", feature = "ui-dialog"))]
pub fn run_autovm(code: &str) -> auto_lang_core::AutoResult<String> {
    install_ui_host_capabilities();
    auto_lang_core::run_autovm(code)
}

#[cfg(any(feature = "ui", feature = "ui-dialog"))]
pub fn run_autovm_capture(code: &str) -> auto_lang_core::AutoResult<(String, String)> {
    install_ui_host_capabilities();
    auto_lang_core::run_autovm_capture(code)
}

#[cfg(not(any(feature = "ui", feature = "ui-dialog")))]
pub use auto_lang_core::{
    run, run_autovm, run_autovm_capture, run_with_capture, run_with_capture_and_path,
    run_with_capture_and_path_staged,
};
