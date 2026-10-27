//! PLAN-843 结构拆分：全局旗标/运行时/脚本参数（自 lib.rs 原样搬移）。
use super::*;

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
pub(crate) use vm_debug;
