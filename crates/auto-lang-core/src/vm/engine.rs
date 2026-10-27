use crate::vm::channel::{AutoChannel, ChannelId};
use crate::vm::codegen::ObjectType;
use crate::vm::heap_object::HeapObject;
use crate::vm::native::NativeInterface;
use crate::vm::opcode::OpCode;
use crate::vm::task::{AutoTask, ResultType, TaskId, TaskStatus};
use crate::vm::task_system::TaskRegistry;
use crate::vm::virt_memory::{
    null_binop_type_error, null_unop_type_error, VirtualFlash, VirtualRAM,
};
use auto_val::AutoStr;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

/// Debug logging macro - only prints when VM debug mode is enabled
/// Plan 423 P5:越界跳转的错误上下文(裸 InvalidOpCode(0xFF) 无从定位
/// mispatched jump —— 带 ip/offset/new_ip/当前函数名)。
fn jump_oob_error(task: &AutoTask, offset: isize, new_ip: isize, flash_len: usize) -> VMError {
    let fn_name = task
        .call_stack
        .last()
        .and_then(|f| f.fn_name.clone())
        .unwrap_or_else(|| "<root>".to_string());
    VMError::RuntimeError(format!(
        "jump out of bounds: ip=0x{:04x} offset={} new_ip=0x{:04x} flash_len=0x{:04x} in {}",
        task.ip, offset, new_ip, flash_len, fn_name
    ))
}

/// Plan 423 P5:runaway 判定参数 —— 一次 handler 调用内的字符串池/堆增量
/// 阈值与检查节拍。正常 UI handler 为百级增量;毒化字节码/失控循环为百万
/// 级(检查本身每 5 万步一次,开销可忽略)。pub 供单测引用。
pub const RUNAWAY_STRINGS_GROWTH: usize = 500_000;
pub const RUNAWAY_HEAP_GROWTH: usize = 500_000;
pub const RUNAWAY_CHECK_EVERY: u64 = 50_000;

/// PLAN-781 T-22（R04 修复轮）：HTTP 段协作式取消探测节拍（步）——连续
/// CPU 段无 park 边界，deadline/RST/关闭终态后在安全指令边界中止段，
/// 不继续业务副作用（非抢占；与 runaway 节拍同量级 ≈ 每 5-10ms CPU）。
pub const HTTP_CANCEL_PROBE_EVERY: u64 = 50_000;

macro_rules! vm_debug {
    ($($arg:tt)*) => {
        if crate::is_vm_debug() {
            eprintln!($($arg)*);
        }
    };
}

// ============================================================================
// Plan 221 Task 5: String tagging helpers
// ============================================================================

/// Result of popping a mixed int/string value from the stack.
/// Strings use NaN-boxed encoding, ints use i32 encoding.
#[derive(Debug, Clone, Copy)]
pub enum StackTag {
    /// A plain integer value
    Int(i32),
    /// A string pool index
    Str(u32),
}

/// Push a string tag onto the stack.
#[inline(always)]
pub fn push_str_tag(ram: &mut VirtualRAM, idx: u32) {
    ram.push_string(idx);
}

/// Plan 403: Format an f64 for display. Whole numbers (e.g. 7.0) render
/// without a decimal part ("7"); others use Rust's default float formatting.
/// Used by the f64 to_string() path so calculator results show cleanly.
#[inline(always)]
fn fmt_f64(f: f64) -> String {
    if f.is_nan() {
        return "NaN".to_string();
    }
    if f.is_infinite() {
        return if f > 0.0 {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    // If the value is (very nearly) a whole number, show it as an int.
    let rounded = f.round();
    if (f - rounded).abs() < 1e-9 && rounded.abs() < 1e15 {
        return format!("{}", rounded as i64);
    }
    format!("{}", f)
}

/// Pop a known-string value, returning the string pool index.
#[inline(always)]
fn pop_str_idx(ram: &mut VirtualRAM) -> usize {
    ram.pop_string() as usize
}

/// Pop a mixed int/string value from the stack.
/// Uses NanoValue type detection.
#[inline(always)]
fn pop_tagged(ram: &mut VirtualRAM) -> StackTag {
    let nv = ram.pop_nv();
    if auto_val::is_string(nv) {
        StackTag::Str(auto_val::decode_string(nv))
    } else if auto_val::is_null(nv) {
        // None (null nanbox) — map to old sentinel for backward compatibility
        StackTag::Int(-1)
    } else {
        StackTag::Int(auto_val::decode_i32(nv))
    }
}

/// Decode a string tag from an i32 variable (non-stack sources).
/// NOTE: Under nanbox, callers should prefer `pop_str_idx()` when reading from the stack.
/// This helper is for sites where the value is already in an i32 variable
/// (e.g., read from bytecode or a non-stack source).
#[inline(always)]
#[allow(dead_code)]
fn decode_str_tag_from_i32(bits: i32) -> usize {
    (-bits - 1) as usize
}

/// List iterator state
#[derive(Debug, Clone)]
pub struct ListIterator {
    pub list_id: u64,
    pub current_index: u32,
}

/// Map iterator state - wraps a source iterator and applies a function
#[derive(Debug, Clone)]
pub struct MapIterator {
    pub source_iterator_id: u32,
    pub func_addr: u32, // Address of the function to call
}

/// Filter iterator state - wraps a source iterator and applies a predicate
#[derive(Debug, Clone)]
pub struct FilterIterator {
    pub source_iterator_id: u32,
    pub func_addr: u32, // Address of the predicate function
}

/// Enumerate iterator state - wraps a source iterator, tracking index
/// Plan 200 Task 3.2: for (i, x) in iter.enumerate()
#[derive(Debug, Clone)]
pub struct EnumerateIterator {
    pub source_iterator_id: u32,
    pub current_index: u32,
}

/// Plan 321: Generator frame — saved execution state for yield/resume.
/// A generator runs on an independent AutoTask; this struct tracks the
/// association and lifecycle. The actual ip/bp/sp live on the task itself.
#[derive(Debug, Clone)]
pub struct GeneratorState {
    /// The AutoTask that owns the generator's execution frame.
    /// Created lazily on first next() call via vm.spawn_task(func_addr).
    pub task_id: Option<u64>,
    /// Function address of the generator body (used for first next()).
    pub func_addr: u32,
    /// Number of arguments the generator function takes.
    pub n_args: u8,
    /// Whether the generator has been started (first next() called).
    pub started: bool,
    /// Whether the generator has finished (function returned).
    pub done: bool,
    /// Plan 321 lazy: saved instruction pointer for resume (after YIELD_VAL).
    /// Set on first next() to func_addr; updated each time YIELD_VAL is hit.
    pub resume_ip: usize,
    /// Plan 321 lazy: saved base pointer for resume.
    pub resume_bp: usize,
    /// Plan 321 lazy: saved stack pointer for resume.
    pub resume_sp: usize,
    /// Plan 321 lazy: saved stack contents (snapshot of local vars + args).
    pub stack_snapshot: Vec<u64>, // Vec<NanoValue> as raw u64
}

/// Plan 321: HTTP stream iterator — wraps an HTTPStream handle so that
/// `for chunk in http_stream { }` works via the standard Iter protocol.
#[derive(Debug, Clone)]
pub struct HttpStreamIterator {
    pub stream_handle: u64,
    pub done: bool,
}

/// Plan 341: Async HTTP stream iterator — wraps an async SSE/stream handle.
/// Events are produced by a tokio::spawn'd future (reqwest async) and pulled
/// via try_recv in shim_iterator_next. `done` is set locally when the channel
/// reports Done or is closed.
#[derive(Debug, Clone)]
pub struct AsyncStreamIterator {
    pub stream_id: u64,
    pub done: bool,
}

/// Unified iterator type
#[derive(Debug, Clone)]
pub enum Iterator {
    List(ListIterator),
    Map(MapIterator),
    Filter(FilterIterator),
    Enumerate(EnumerateIterator),
    /// Plan 321: Generator-based iterator (yield/~Iter<T>)
    Generator(GeneratorState),
    /// Plan 321: HTTP stream iterator (wraps HTTPStream for for-loop consumption)
    HttpStream(HttpStreamIterator),
    /// Plan 341: Async HTTP stream iterator (SSE / async streaming response)
    AsyncHttpStream(AsyncStreamIterator),
}

// ============================================================================
// Closures (Plan 071: Direct Capture)
// ============================================================================

use auto_val::Value;

/// Closure - a function value with directly captured environment (Plan 071: Direct Capture)
#[derive(Debug, Clone)]
pub struct Closure {
    pub func_addr: u32,              // Bytecode address
    pub env: HashMap<String, Value>, // Direct captured values (no upvalues!)
    pub n_args: usize, // Number of parameters (for CALL_CLOSURE to set current_fn_n_args)
    /// Plan 385: 捕获变量的原始栈位置 (creator_bp, slot_offset)。
    /// LOAD/STORE_CAPTURED 通过此映射直接读写创建者的栈帧（by-reference 捕获）。
    /// 若为空（旧字节码兼容），fallback 到 env（by-value）。
    pub capture_slots: HashMap<String, (usize, usize)>,
    /// Plan 454 E5a:函数参数域捕获的绝对槽位(bp-(n_args-idx+1),创建时
    /// 解析——参数不在 bp+1+idx 正域,通用 capture_slots 公式对它全错,
    /// 表现为谓词闭包读外层参数得 ""/0。为空=无参数域捕获。
    pub param_abs: HashMap<String, usize>,
    /// PLAN-667 (F-01): 创建帧身份 (task_id, frame_uid)。帧相对捕获
    /// (capture_slots/param_abs 非空)在读写前按此校验：任务不符=跨任务
    /// 错帧，uid 不在活跃表=创建帧已返回（含同 bp 复用）。None=无帧
    /// 相对捕获（env by-value / 合成闭包），无需校验。
    pub creator_frame: Option<(crate::vm::task::TaskId, u64)>,
}

/// Plan 442 A5: one-shot timer callback form. The event form dispatches a
/// widget handler (the `__toast_tick` pattern — no frame-geometry concerns);
/// the closure form satisfies the auto-down SchedulerTimer contract
/// (`setTimeout(fn, ms) -> handle`) but only supports by-value captures —
/// by-reference captures (capture_slots) read the creator's frame and are
/// meaningless once that frame is gone.
#[derive(Debug, Clone)]
pub enum TimerCallback {
    /// Closure registry id (a closure value on the VM stack is Int(id)).
    Closure(u32),
    /// Widget event name — dispatched like any UI event.
    Event(String),
}

/// Plan 442 A5: a pending one-shot timer.
#[derive(Debug, Clone)]
pub struct TimerEntry {
    pub deadline: std::time::Instant,
    pub callback: TimerCallback,
}

#[derive(Debug)]
pub enum VMError {
    StackOverflow,
    StackUnderflow,
    InvalidOpCode(u8),
    DivisionByZero,
    Halt,
    MissingNative(u16),
    RuntimeError(String),
    /// Plan 092: FFI-related errors (library loading, ABI incompatibility, etc.)
    FFI(String),
    /// Plan 011 (MS3-B): `exit(code)` was called from AutoLang. Carries the
    /// requested exit code. The embedding shell's run loop treats this as a
    /// cooperative stop, not a failure.
    ExitRequested(i32),
}

/// Plan 773 F-08: stable variant-name identity of a VM trap, decided where
/// the typed `VMError` still exists (the run-task loop's error arm). This is
/// the type-derived run-stage code for test-case oracles — never parsed back
/// out of the flattened message text.
pub fn vm_error_code(e: &VMError) -> &'static str {
    match e {
        VMError::StackOverflow => "StackOverflow",
        VMError::StackUnderflow => "StackUnderflow",
        VMError::InvalidOpCode(_) => "InvalidOpCode",
        VMError::DivisionByZero => "DivisionByZero",
        VMError::Halt => "Halt",
        VMError::MissingNative(_) => "MissingNative",
        VMError::RuntimeError(_) => "RuntimeError",
        VMError::FFI(_) => "FFI",
        VMError::ExitRequested(_) => "ExitRequested",
    }
}

/// Result of executing a single VM instruction (used by `run_one_instruction`)
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    /// Continue executing more instructions
    Continue,
    /// Task has terminated (HALT, RET at bp==0, IP past end)
    Terminated,
    /// Task should pause the current batch (YIELD_TASK, SLEEP, blocked JOIN/SEND/RECV)
    Yield,
    /// Plan 224: AWAIT_FUTURE hit a Pending future — caller should handle body execution
    AwaitFuture { future_id: u32, body_offset: u32 },
    /// Plan 321: YIELD_VAL hit — generator yielded a value. The value is on
    /// top of stack. The generator driver (shim_iterator_next Generator branch)
    /// should pop the value and save the task state for next resume.
    GeneratorYield,
}

/// Plan 224: Result of executing a single frame (budget-limited instruction batch)
#[derive(Debug)]
pub enum FrameResult {
    /// Normal continuation (unreachable — budget handles this)
    Continue,
    /// Frame completed: RET/HALT reached
    Return,
    /// AWAIT_FUTURE hit Pending — body_offset needs execution
    AwaitFuture { future_id: u32, body_offset: u32 },
    /// Task yielded (SLEEP, YIELD, blocked channel op)
    Yielded,
    /// Execution error
    Error(VMError),
    /// Instruction budget exhausted (cooperative yielding)
    BudgetExhausted,
}

/// PLAN-702 T-01: outcome of one handler dispatch segment (segment driver for
/// the UI bridge; sync `call_fn_by_name` is the busy-wait specialization).
#[derive(Debug)]
pub enum SegmentOutcome {
    /// Handler finished within this segment (RET reached, or the Plan 321
    /// generator short-circuit fired). `Ok(())` ≡ `call_fn_by_name` Ok —
    /// result value on stack top; `Err` after non-stack state restore, same
    /// as the sync driver's error return.
    Completed(Result<(), VMError>),
    /// Handler yielded with a pending wait (async HTTP request / external
    /// `~{}` future). Task state (ip/bp/stack/closures) is preserved for
    /// [`AutoVM::resume_fn_by_name_segment`]. No busy-wait happened; the
    /// wait's result slot is deliberately NOT dropped — PLAN-027 缺陷 A 的
    /// `drop_async_result` 回收只属于"真正放弃等待"的超时路径，恢复时按
    /// req_id 从 ASYNC_RESULTS 取用。
    Parked {
        wait: ParkedWait,
        seg: ParkedSegment,
    },
    /// PLAN-711 T-11 (M-02/D-2): CPU 预算耗尽——栈完整（ip 停在函数内、
    /// bp≠saved_bp、闭包/异常帧原样），下一轮由 CPU 泵在预算内继续。
    /// **不得**伪装为 `Completed`：预算耗尽假成功（旧实现落入
    /// `Completed(Ok(()))` 无结果值）是本变体要替换的正确性缺陷（AC-13）。
    /// 仅 `DriveBudget::CpuSlice` 驱动产生；legacy 档耗尽返回真错误。
    Runnable { seg: ParkedSegment },
}

/// PLAN-711 T-11 (D-2 冻结初值): UI CPU 片预算。数值为 708-decision §D-2
/// 冻结的提议初值，按实测校准时不得为通过而下调验收指标。
#[derive(Clone, Copy, Debug)]
pub struct CpuSliceBudget {
    /// 每片指令上限（初值 4096）。
    pub max_steps: u64,
    /// 每片墙钟上限（初值 4ms）——至多每 [`Self::clock_check_every`] 步查一次。
    pub max_duration: std::time::Duration,
    /// 查钟间隔（初值 64 步）。
    pub clock_check_every: u64,
    /// 跨片累计指令护栏（初值 10M，与 legacy 单调用预算同量级）——
    /// I/O 等待不计忙时；不让出重置绕过原 runaway 保护。
    pub cumulative_steps: u64,
}

impl CpuSliceBudget {
    pub const fn d2_default() -> Self {
        Self {
            max_steps: CPU_SLICE_MAX_STEPS,
            max_duration: CPU_SLICE_MAX_DURATION,
            clock_check_every: CPU_SLICE_CLOCK_CHECK_EVERY,
            cumulative_steps: CPU_CUMULATIVE_STEP_BUDGET,
        }
    }
}

/// PLAN-711 T-11 (D-2): 每片 4096 指令。
/// **T-04 R-1 实测校准（2026-09-30，D-2 授权的按实测校准面）**：4096 步
/// 在解释器实际速度（~1-3ns/指令）下 ≈ 4-12μs ≪ 4ms 墙钟帽——指令帽成为
/// 实际预算，长 handler 每 ~93ms 帧仅推进 4K 指令（5M 迭代夹具实测
/// 11s+ 未完）。校准为 1M 指令天花板：正常片由 4ms 墙钟帽收口（实测
/// ~0.2-0.5ms 提前让出），1M 只兜病理长指令；累计 10M 护栏不变。
pub const CPU_SLICE_MAX_STEPS: u64 = 1_000_000;
/// PLAN-711 T-11 (D-2): 每片 4ms。
pub const CPU_SLICE_MAX_DURATION: std::time::Duration = std::time::Duration::from_millis(4);
/// PLAN-711 T-11 (D-2): 至多每 64 指令查时钟。
pub const CPU_SLICE_CLOCK_CHECK_EVERY: u64 = 64;
/// PLAN-711 T-11 (D-2): 跨片累计护栏 10M 指令（I/O 等待不计）。
pub const CPU_CUMULATIVE_STEP_BUDGET: u64 = 10_000_000;
/// PLAN-711 T-11 (D-2): 会话宿主每轮 CPU pump 总预算 8ms（消费方=vm_bridge
/// 有界泵；引擎片内不感知轮次）。
pub const CPU_PUMP_ROUND_BUDGET: std::time::Duration = std::time::Duration::from_millis(8);

/// PLAN-711 T-11: 段驱动预算档。Legacy 保持既有 10M 步预算与逐字节同的
/// 同步语义（非 UI 契约）；CpuSlice 为 UI 专用可续跑片。
/// PLAN-781 T-15: `LegacyHttp(cap)` = HTTP 入口族的请求级累计预算档
/// （决策报告 781-budget-decision.md D-1/D-4）——段内 10M 帽不变，另加
/// **跨段累计**上限 `cap`（park/resume、嵌套 fn 不重置，不变量 2）；
/// 逐指令精确检查（V-16 的 N-1/N/N+1 可控性）。
#[derive(Clone, Copy, Debug)]
pub enum DriveBudget {
    Legacy,
    /// 请求级累计上限（`AUTO_HTTP_STEP_BUDGET` 解析产物）。
    LegacyHttp(u64),
    CpuSlice(CpuSliceBudget),
}

/// What a parked segment is waiting on (readiness credential for the UI
/// resume pump, PLAN-702 T-03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParkedWait {
    /// Async HTTP request — ready when ASYNC_RESULTS[req_id] holds a result;
    /// the shim's re-entry branch consumes it when the re-driven CALL_NAT
    /// re-fires (Plan 349 step 7 re-entry protocol).
    HttpRequest(u64),
    /// External `~{}` future — ready when the future leaves Pending; the
    /// wake itself happens in `resume_fn_by_name_segment` (mirror of wake
    /// source 6, Plan 394 Phase A).
    Future(u32),
    /// PLAN-707 T-05: HTTP/SSE stream — ready when the stream has a queued
    /// item or reached a terminal state (`http_stream::stream_ready`);
    /// the shim's re-entry branch consumes it when the re-driven CALL_NAT
    /// re-fires (same protocol as HttpRequest).
    HttpStream(u64),
    /// PLAN-711 T-11 (D-2): CPU 预算耗尽的 continuation 凭据——非 I/O 等待，
    /// 就绪性由 CPU 泵的轮次预算/公平策略裁决（tick 泵不拾取；见 vm_bridge
    /// `parked_wait_ready` CpuRunnable 臂与 `resume_cpu_slices`）。只增不改
    /// 707 三凭据（HttpRequest/Future/HttpStream 语义原样）。
    CpuRunnable,
}

/// Resume context captured at park time; immutable across repeated
/// park→resume cycles of the same call (saved_bp / saved_fn_n_args /
/// fn_name are fixed for the lifetime of the dispatch).
#[derive(Debug, Clone)]
pub struct ParkedSegment {
    pub fn_name: String,
    /// Caller-frame bp at dispatch time (completion detection: `task.bp ==
    /// saved_bp` after RET restores it).
    pub saved_bp: usize,
    /// `current_fn_n_args` to restore once the handler returns / errors.
    pub saved_fn_n_args: usize,
}

/// PLAN-776 R1-F03：VM 实例唯一号分配器（凭据 owner 身份的来源侧）。
static VM_INSTANCE_ID_GEN: AtomicU64 = AtomicU64::new(1);

pub struct AutoVM {
    pub flash: Arc<VirtualFlash>,
    pub native_interface: Arc<NativeInterface>,
    /// String constant pool (Plan 073: Made mutable for runtime string field access)
    pub strings: Arc<RwLock<Vec<Vec<u8>>>>,
    /// 运行时字符串池去重索引(2026-08-22,见 add_string 注释):
    /// 内容 → 池索引。池索引在 natives 侧以 u16 截断,只增不减的池
    /// 溢出后会回绕串写;去重令重复内容复用索引,池稳定在低位。
    pub string_dedup: std::sync::Mutex<std::collections::HashMap<Vec<u8>, usize>>,

    pub tasks: DashMap<TaskId, Arc<Mutex<AutoTask>>>,
    pub id_gen: AtomicU64,
    // Plan 744: VM SQLite handles belong to their VM instance and opening
    // task. HTTP request task cleanup releases its connections instead of
    // retaining one process-global entry per request.
    sqlite_handles: std::sync::Mutex<HashMap<i64, (TaskId, a2r_std::sqlite::SqliteDb)>>,

    // Channel Registry
    pub channels: DashMap<ChannelId, Arc<AutoChannel>>,
    pub channel_id_gen: AtomicU64,

    // Plan 077 Phase 6: Legacy list registry removed - all lists now use unified heap_objects registry

    // Iterator Registry
    pub iterators: DashMap<u32, Iterator>,
    pub iterator_id_gen: AtomicU32,

    // Closure Registry (Plan 071: Direct Capture, no upvalues)
    pub closures: DashMap<u32, Closure>,
    pub closure_id_gen: AtomicU32,

    // Plan 077 Phase 4: Unified Object Registry
    // Single registry for all heap-allocated objects (lists, maps, sets, nodes,
    // ObjectData, GenericInstance, etc.). Plan 390 §15 H3: the standalone
    // `objects`/`arrays`/`nodes` registries were folded in here (H1/H3a/H3b).
    pub heap_objects: DashMap<u64, Arc<RwLock<dyn HeapObject>>>,
    pub heap_object_id_gen: AtomicU64,
    /// PLAN-776 R1-F03：VM 实例唯一号（进程级 AtomicU64 分配）。heap id 是
    /// per-VM 局部编号（两 VM 同号必然），HTTP 返回资源凭据的 id 段编入
    /// owner（见 http_return_resource::credential_id），解析侧 id 层校验——
    /// "每 VM 一张表"不构成来源证明。
    pub instance_id: u64,
    /// PLAN-776 R1-F03：HTTP 凭据专用 per-VM local 计数（凭据 id 不走通用
    /// heap_object_id_gen——id 段编 owner，跨 VM 同号不可能）。
    pub http_credential_gen: std::sync::atomic::AtomicU64,
    /// PLAN-781 T-15：HTTP handler 请求级累计步预算快照（宿主 env
    /// `AUTO_HTTP_STEP_BUDGET`；None = 默认 10M Legacy 语义）。构造期一次
    /// 读取，运行期不可变（每 VM 实例一份——同进程多实例共享同一 env，
    /// 跨进程天然隔离，决策报告 D-1）。
    pub http_step_budget: Option<u64>,

    // Plan 419 Phase 1: 精确引用计数(heap_rc)+ 毒化 canary(tombstones)。
    // heap_rc: id → 计数(owned slot 数);insert_heap_object 不建条目,
    // 首次 rc_push/rc_retain 建条目。归零 → remove_heap_object 递归释放。
    pub heap_rc: DashMap<u64, std::sync::atomic::AtomicU32>,
    #[cfg(debug_assertions)]
    pub tombstones: DashMap<u64, std::time::Instant>,
    pub rc_created_total: AtomicU64,
    pub rc_freed_total: AtomicU64,
    pub rc_traffic: AtomicU64,
    // PLAN-062: 状态突变序号——堆/状态**写点**统一 bump（insert_heap_object/
    // SET_FIELD/list 元素突变/字符串池新内容），读与 dedup 命中不 bump。
    // 用途：fire_timer 派发前后对比判定"本拍是否零状态写"（空转拍不置脏，
    // 不触发整树重建）。保守方向：多 bump 至多多一次重建，漏 bump 才丢更新。
    pub state_mutation_seq: AtomicU64,
    // PLAN-047 T-03（档 C SD-09）: per-path 版本表——写点字段归因的定点
    // bump 面。键 = (堆对象 id, path)；path 为字段名（A 类带名写）或 "*"
    // （B 类容器/未知字段写）。全局 [`Self::state_mutation_seq`] 语义零
    // 变化（[`Self::bump_path`] 内仍同步 bump），此表纯增量消费者（memo
    // 门 version_fast 判定）。键用原生 (u64, String) 元组——vm 模块不得
    // 依赖 ui 门控类型（memo_deps DepKey 由桥侧转换）。
    pub path_versions: dashmap::DashMap<(u64, String), u64>,
    // PLAN-047 T-04（档 C SD-08）: 依赖录制槽——ui 侧 memo 门/computed
    // 信号求值期经 guard 激活，VM 读臂（GET_FIELD/GET_GENERIC_FIELD/
    // GET_ELEM/LIST_GET_INT）把堆读记入 [`Self::record_heap_read`]。
    // 快速门 = AtomicBool（未激活读臂一次原子 load 直落——非 memo 零开销
    // 红线）；槽 = Mutex<Option<Arc<Mutex<RecState>>>>（bridge guard 持
    // Arc，finish 取走收编）。顺序纪律：**先填槽后开旗（Release）、先关
    // 旗后清槽（Acquire）**——读臂见旗即见已填槽。同线程串行（渲染/桥
    // 调用），旗与槽无并发写竞争。
    pub dep_recorder_slot:
        std::sync::Mutex<Option<std::sync::Arc<std::sync::Mutex<crate::vm::dep_track::RecState>>>>,
    pub dep_rec_active: std::sync::atomic::AtomicBool,
    // PLAN-062 T12: 宽限窗延迟回收——rc 归零对象先进 dying 队列（记录入队
    // 时的解释步计数），DYING_GRACE_STEPS 步后续期仍未被复活才真回收。
    // 帧内/近邻的 raw 别名（无份额拷贝）在窗口内安全；长期零累积（窗口
    // 有界）。复活（窗口内再 retain）即出队。canary 的 tombstone 只在真
    // 回收后落——窗口内 ACCESS 探测仍见活对象。
    pub dying_heap: std::sync::Mutex<Vec<(u64, u64)>>,
    pub interp_steps: AtomicU64,
    // Plan 419 Phase 2: 字符串池并行状态(rc/pinned/freelist,见 rc.rs)。
    pub pool_state: std::sync::RwLock<crate::vm::rc::PoolState>,

    // Plan 121: Task/Msg Registry for Actor model
    // Manages singleton tasks and task instances
    pub task_registry: Arc<TaskRegistry>,

    // Plan 127: Task Handler Registry for message routing
    // Maps task type names to their handler tables
    pub task_handler_registry: crate::vm::task_handler::TaskHandlerRegistry,

    // Plan 124: Future Registry for async/await
    // Stores pending futures with their body code offsets
    pub futures: DashMap<u32, Arc<RwLock<FutureValue>>>,
    pub future_id_gen: AtomicU32,

    // Plan 442 A5: one-shot timer registry (set_timeout/clear_timeout). The
    // render loop polls due timers on its subscription tick and dispatches
    // the callbacks (closure via call_closure, event via handler dispatch).
    pub timers: DashMap<u32, TimerEntry>,
    pub timer_id_gen: AtomicU32,

    // Plan 317 Phase 1: Per-task message queues for message-loop tasks.
    // Keyed by AutoVM task id (== the handle id returned by Task.spawn).
    // DashMap avoids the tokio Mutex blocking_lock panic that would occur if
    // pending_messages lived on AutoTask (which is behind tokio::sync::Mutex).
    pub task_mailboxes: DashMap<u64, std::sync::Mutex<Vec<auto_val::Value>>>,

    // Plan 317: Global variables (module-level `var x = ...`).
    // String-keyed so any function can read/write them via LOAD_GLOBAL/
    // STORE_GLOBAL. DashMap for thread-safety (concurrent HTTP handlers).
    pub globals: DashMap<String, auto_val::NanoValue>,

    // Plan 197 Task 9: Generic registry for field name lookup at runtime
    pub generic_registry: crate::vm::generic_registry::GenericRegistry,

    // Plan 177: Optional stdout capture buffer for testing
    pub output_buffer: Option<Arc<RwLock<String>>>,

    // Plan 199: Debugger controller (NoOpController for normal execution)
    pub debugger: Arc<std::sync::Mutex<Box<dyn crate::vm::debugger::DebuggerController>>>,

    // Plan 199: Execution trace collector (None = tracing disabled)
    pub trace: Arc<std::sync::Mutex<Option<crate::vm::trace::TraceCollector>>>,

    // Plan 011 (MS3-B): optional shell host bridge. When set, the
    // system()/system_status()/export()/exit() natives forward to it.
    // None for pure-AutoLang use (backward compatible).
    pub host: Option<crate::host::SharedHost>,

    // PLAN-746 (PG-MEM-1): optional cooperative execution deadline. When
    // set, run_task_loop terminates all tasks once the Instant passes and
    // surfaces a synthetic `last_error` ("ExecutionTimeout...") so callers
    // get a clean error instead of a runaway execution. None = unlimited
    // (default; zero behavior change for existing callers).
    pub deadline: Option<std::time::Instant>,
    /// Request-scoped stdin for hosted executions. `None` preserves the
    /// process stdin used by the CLI; `Some(reader)` is isolated to this VM.
    request_input: std::sync::Mutex<Option<std::io::BufReader<std::io::Cursor<Vec<u8>>>>>,
}

// Plan 124: Future value for async operations
#[derive(Debug, Clone)]
pub struct FutureValue {
    /// Bytecode offset of the async block body
    pub body_offset: u32,
    /// Current state of the future
    pub state: FutureState,
    /// Result value when ready
    pub result: Option<auto_val::Value>,
    /// Task ID that owns this future (for suspension/resumption)
    pub owner_task_id: TaskId,
    /// Plan 348 Task 22: Captured free-variable environment for `~{ ... }`.
    /// For SPAWN_GO the spawned task gets its own RAM, so captures cannot be
    /// read from the caller's stack. Instead we stash them here (keyed by the
    /// variable name) and expose them to the body via LOAD_CAPTURED by
    /// installing a synthetic Closure entry before the body runs.
    pub captures: HashMap<String, auto_val::Value>,
    /// Plan 394: Internal = `~{}` bytecode body (sync inline). External =
    /// native-spawned async source resolved via register/resolve API.
    pub kind: FutureKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FutureState {
    Pending,
    Ready,
    Failed,
}

/// Plan 394: distinguishes `~{}` body futures from native external sources.
#[derive(Debug, Clone, PartialEq)]
pub enum FutureKind {
    /// Body offset points at bytecode; `execute_future_body` runs it inline.
    Internal,
    /// Worker thread writes `result`/`state`; AWAIT_FUTURE suspends the task.
    External,
}

/// Convert a single-slot nanboxed value to f64 for mixed-type arithmetic.
/// Handles i32 → f64 and f32 → f64 promotion.
#[inline(always)]
fn nanbox_single_to_f64(nv: auto_val::NanoValue) -> f64 {
    if auto_val::is_f32(nv) {
        auto_val::decode_f32(nv) as f64
    } else {
        auto_val::decode_i32(nv) as f64
    }
}

/// Plan 576: 单槽 nanbox 值按 tag 解码为 f32（f32 按位、int 按值）——
/// ADD/SUB/MUL/DIV 的 f32×int 混算臂用（nanbox_single_to_f64 的 f32 域
/// 形态，pop_f32_operand 同哲学）。原 else 臂 decode_i32 把整值/任意
/// f32 位型当整数（240.0 + 1 → 1131413505，DEBTS 043 家族）。
#[inline(always)]
fn nanbox_single_to_f32(nv: auto_val::NanoValue) -> f32 {
    if auto_val::is_f32(nv) {
        auto_val::decode_f32(nv)
    } else {
        auto_val::decode_i32(nv) as f32
    }
}

/// Plan 437: f32 算术的 tag 驱动操作数弹出——动态来源（Index/Dot/Call 等
/// 推断不可知处）的 int 操作数按**值**转 f32，f32/f64 按 tag 解码。
/// 盲 pop_f32 会把 int 位模式重解释成 denormal（int 80 → 1.12e-43），
/// 与 extract_autovm_result 的 tag-first 修复同一哲学。
/// PLAN-063 T-04b（auto-down 转介）：_D 族算术/比较的标签驱动弹栈。
/// `pop_f64` 是严格 `decode_f64`——而 demo/iced 派发路（call_handler_for →
/// `push_value`）对 `Value::Float` 推的是 **f32 编码槽**（`push_f32`），
/// f64 严格解码得全零位型（恰为 int 0），f64（Type::Double）参数参与任何
/// 二元运算/比较即坍缩为 0/恒假（实机：OnLeftScroll 比例级联、自绘滚动条
/// Move 守卫 `.scrollHeight > .clientHeight` 全哑）。Plan 437
/// `pop_f32_operand` 同款口径：按实际 tag 解码并归一到 f64。
#[inline(always)]
fn pop_f64_operand(task: &mut AutoTask) -> f64 {
    let nv = task.ram.pop_nv();
    if auto_val::is_f64(nv) {
        auto_val::decode_f64(nv)
    } else if auto_val::is_f32(nv) {
        auto_val::decode_f32(nv) as f64
    } else if auto_val::is_i32(nv) {
        auto_val::decode_i32(nv) as f64
    } else {
        auto_val::decode_f64(nv)
    }
}
fn pop_f32_operand(task: &mut AutoTask) -> f32 {
    let nv = task.ram.pop_nv();
    if auto_val::is_f32(nv) {
        auto_val::decode_f32(nv)
    } else if auto_val::is_f64(nv) {
        auto_val::decode_f64(nv) as f32
    } else if auto_val::is_i32(nv) {
        auto_val::decode_i32(nv) as f32
    } else {
        auto_val::decode_f32(nv)
    }
}

/// Plan 550 T03: 定点算术槽（_F/_D/_U64/MOD 族）弹栈前的 null 窥视守卫。
/// 这些臂的弹栈助手按 tag/位模式解码（TAG_NULL → 垃圾整数或 NaN 静默
/// 传播），统一在弹栈前窥视拦截。peek(0)=右操作数，peek(1)=左操作数。
#[inline(always)]
fn null_guard_peek_pair(task: &AutoTask, op: &str) -> Result<(), VMError> {
    let b_nv = task.ram.peek_nv(0);
    let a_nv = task.ram.peek_nv(1);
    if auto_val::is_null(a_nv) || auto_val::is_null(b_nv) {
        return Err(null_binop_type_error(
            op,
            (a_nv, !auto_val::is_nanboxed(a_nv)),
            (b_nv, !auto_val::is_nanboxed(b_nv)),
        ));
    }
    Ok(())
}

/// Plan 550 T03: 一元定点算术槽（NEG_F/NEG_D）弹栈前的 null 窥视守卫。
#[inline(always)]
fn null_guard_peek_unary(task: &AutoTask, op: &str) -> Result<(), VMError> {
    if auto_val::is_null(task.ram.peek_nv(0)) {
        return Err(null_unop_type_error(op));
    }
    Ok(())
}

/// Plan 550 T05: GET_ELEM 越界消息（对标 Python IndexError；携带原始
/// 索引值，负索引越界同报原值）。原先静默 push_i32(0) 哨兵（p8 探针）。
#[cold]
fn index_out_of_range_error(index: i32) -> VMError {
    VMError::RuntimeError(format!("IndexError: index {} out of range", index))
}

/// Plan 406: unified condition truthiness. Tagged bools (TAG_BOOL) are
/// authoritative; null is falsy; everything else falls back to legacy i32
/// truthiness (0 / i32::MIN+1 sentinels) for values pushed by older code
/// paths that still use push_i32. NOTE: a REAL integer -2147483647 is
/// indistinguishable from the legacy false sentinel at the i32 layer — an
/// accepted, documented limitation until all push_i32-bool sites are gone.
#[inline]
fn nv_truthy(nv: auto_val::NanoValue) -> bool {
    if auto_val::is_bool(nv) {
        auto_val::decode_bool(nv)
    } else if auto_val::is_null(nv) {
        false
    } else {
        let v = auto_val::decode_i32(nv);
        v != 0 && v != -2147483647
    }
}

impl AutoVM {
    pub fn new(flash: VirtualFlash, ram_size: usize) -> Self {
        Self::new_inner(flash, ram_size, true)
    }

    /// F-785-R2-01：跳过进程默认快照的构造（cfg(test)）。共享 lib 测试
    /// 进程的全局默认槽不可假定缺席（普通程序测试经 run 族隐式 if-absent
    /// 装配），"无 provider"断言锚必须**构造式**成立而非依赖全局时序；
    /// 全局缺席时序合同由外部 fixture 独立进程承担（no-ui 无安装路径）。
    // PLAN-814: 跨 crate 测试支持面
    pub fn new_without_host_snapshot(flash: VirtualFlash, ram_size: usize) -> Self {
        Self::new_inner(flash, ram_size, false)
    }

    fn new_inner(flash: VirtualFlash, _ram_size: usize, snapshot_host_capabilities: bool) -> Self {
        let mut native_interface = NativeInterface::production();

        // PLAN-779: 宿主能力装配——构造期快照进程默认 frame provider 进
        // 本实例（每 VM 独立槽：之后进程默认替换/其他实例注入互不影响，
        // 两实例可持不同 provider 不串线）。core 零 UI 引用；宿主注册点
        // 见 vm::host_capabilities（UI adapter / 外部 consumer）。
        if snapshot_host_capabilities {
            if let Some(provider) = crate::vm::host_capabilities::process_default_frame_provider() {
                native_interface.set_frame_provider(provider);
            }

            // PLAN-785: 同纪律快照文本剪贴板 provider（2925/2926 能力=
            // 注入；缺席快照 = shim 侧 feature-disabled 形 VMError，与
            // no-ui 构建迁移前行为一致）。
            if let Some(provider) =
                crate::vm::host_capabilities::process_default_clipboard_text_provider()
            {
                native_interface.set_clipboard_text_provider(provider);
            }

            // PLAN-790: 同纪律快照文件/图像剪贴板 provider（2934..2937）。
            // 缺席快照 = shim 侧 G3 值降级（空表/false/null），与迁移前
            // 非 Windows / 未开 native-clipboard 面同形。
            if let Some(provider) =
                crate::vm::host_capabilities::process_default_clipboard_media_provider()
            {
                native_interface.set_clipboard_media_provider(provider);
            }

            // PLAN-794: 同纪律快照 scroll host provider（9900..9905/
            // 9922/9923）。缺席快照 = shim 侧 T-01 冻结降级臂（Int 0 /
            // false / 0.0 / "error:unsupported" / "unknown"），与迁移前
            // no-ui 构建同形。
            if let Some(provider) =
                crate::vm::host_capabilities::process_default_scroll_host_provider()
            {
                native_interface.set_scroll_host_provider(provider);
            }

            // PLAN-802: 同纪律快照 code editor provider（22 native 全族）。
            // 缺席快照 = shim 侧 802 T-01 冻结缺席臂（21 项 raise
            // feature-disabled 形、9913 scroll_to 弹参返 false），与迁移前
            // feature-off 构建逐字面值同形。
            if let Some(provider) =
                crate::vm::host_capabilities::process_default_code_editor_host_provider()
            {
                native_interface.set_code_editor_provider(provider);
            }

            // PLAN-804: 同纪律快照 dialog host provider（2927/2928）。
            // 缺席快照 = shim 侧 feature-disabled 形 VMError（T-01 冻结
            // 矩阵，与迁移前 no-ui 构建一致）。
            if let Some(provider) =
                crate::vm::host_capabilities::process_default_dialog_host_provider()
            {
                native_interface.set_dialog_host_provider(provider);
            }

            // PLAN-804: 同纪律快照 dnd host provider（2938）。缺席快照 =
            // shim 侧 false 降级（G5，与迁移前 off 臂同形）。
            if let Some(provider) =
                crate::vm::host_capabilities::process_default_dnd_host_provider()
            {
                native_interface.set_dnd_host_provider(provider);
            }

            // PLAN-807: 同纪律快照 editor snapshot provider（9916 供值）。
            // 缺席快照 = shim 侧"无法找到相应 editor"err envelope（值，
            // 非 raise，与迁移前无编辑器注册同形）。
            if let Some(provider) =
                crate::vm::host_capabilities::process_default_editor_snapshot_provider()
            {
                native_interface.set_editor_snapshot_provider(provider);
            }

            // PLAN-808: 同纪律快照 terminal provider（34 native 全族）。
            // 缺席快照**不是**错误/空值——shim 回落进程共享 engine service
            // （迁移前无 UI 也驱动引擎的行为保留）；UI 宿主安装 adapter 后
            // 才有输入/几何/menu/scroll/window/palette 面。
            if let Some(provider) =
                crate::vm::host_capabilities::process_default_terminal_host_provider()
            {
                native_interface.set_terminal_provider(provider);
            }
        }

        // Plan 216 Phase 2: Merge C-FFI shims from the global CFFI_GLOBAL registry.
        // The codegen's handle_c_import populates CFFI_GLOBAL during compilation;
        // here we merge those shims into the VM's NativeInterface so CALL_NAT can find them.
        {
            let cffi = crate::vm::codegen::CFFI_GLOBAL.lock().unwrap();
            native_interface.merge(cffi.native_interface());
        }

        Self {
            flash: Arc::new(flash),
            native_interface: Arc::new(native_interface),
            strings: Arc::new(RwLock::new(Vec::new())),
            string_dedup: std::sync::Mutex::new(std::collections::HashMap::new()),
            tasks: DashMap::new(),
            id_gen: AtomicU64::new(0),
            sqlite_handles: std::sync::Mutex::new(HashMap::new()),
            channels: DashMap::new(),
            channel_id_gen: AtomicU64::new(0),
            // Plan 077 Phase 6: Legacy list registry removed
            iterators: DashMap::new(),
            iterator_id_gen: AtomicU32::new(0),
            closures: DashMap::new(),
            closure_id_gen: AtomicU32::new(0),
            // Plan 077 Phase 4: Unified object registry (Plan 390 §15 H3:
            // objects/arrays/nodes registries folded in — single id space).
            heap_objects: DashMap::new(),
            heap_object_id_gen: AtomicU64::new(4000000),
            // PLAN-776 R1-F03：VM 实例唯一号（凭据 owner 身份）。
            instance_id: VM_INSTANCE_ID_GEN.fetch_add(1, Ordering::Relaxed),
            http_credential_gen: std::sync::atomic::AtomicU64::new(0),
            http_step_budget: crate::vm::budget_policy::http_step_budget_from_env(), // Plan 419 Phase 1: RC 协议状态
            heap_rc: DashMap::new(),
            #[cfg(debug_assertions)]
            tombstones: DashMap::new(),
            rc_created_total: AtomicU64::new(0),
            rc_freed_total: AtomicU64::new(0),
            rc_traffic: AtomicU64::new(0),
            state_mutation_seq: AtomicU64::new(0),
            path_versions: DashMap::new(),
            dep_recorder_slot: std::sync::Mutex::new(None),
            dep_rec_active: std::sync::atomic::AtomicBool::new(false),
            dying_heap: std::sync::Mutex::new(Vec::new()),
            interp_steps: AtomicU64::new(0),
            pool_state: std::sync::RwLock::new(crate::vm::rc::PoolState::new()),
            // Plan 121: Task/Msg registry for Actor model
            task_registry: Arc::new(TaskRegistry::new()),
            // Plan 127: Task handler registry for message routing
            task_handler_registry: crate::vm::task_handler::TaskHandlerRegistry::new(),
            // Plan 124: Future registry for async/await
            futures: DashMap::new(),
            future_id_gen: AtomicU32::new(1),
            // Plan 442 A5: one-shot timer registry
            timers: DashMap::new(),
            timer_id_gen: AtomicU32::new(1),
            task_mailboxes: DashMap::new(), // Plan 317 Phase 1
            globals: DashMap::new(),        // Plan 317: module-level var storage
            // Plan 197 Task 9: Generic registry for runtime field name lookup
            generic_registry: crate::vm::generic_registry::GenericRegistry::new(),
            // Plan 177: stdout capture (None = normal println)
            output_buffer: None,
            // Plan 199: Debugger controller (NoOpController for normal execution)
            debugger: Arc::new(std::sync::Mutex::new(Box::new(
                crate::vm::debugger::NoOpController,
            ))),
            trace: Arc::new(std::sync::Mutex::new(None)),
            host: None,
            // PLAN-746: no deadline by default (unlimited execution).
            deadline: None,
            request_input: std::sync::Mutex::new(None),
        }
    }

    /// PLAN-746 (PG-MEM-1): install a cooperative execution deadline. The VM
    /// checks it on every run_task_loop sweep; once passed, all tasks are
    /// terminated with a synthetic "ExecutionTimeout" last_error.
    pub fn set_deadline(&mut self, deadline: Option<std::time::Instant>) {
        self.deadline = deadline;
    }

    /// Install finite stdin for one hosted execution. An empty string is an
    /// immediate EOF; blank lines remain readable lines.
    pub fn set_request_input(&self, input: Option<String>) {
        let reader =
            input.map(|text| std::io::BufReader::new(std::io::Cursor::new(text.into_bytes())));
        *self
            .request_input
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = reader;
    }

    /// Return `None` when this VM uses process stdin, otherwise `Some(line)`
    /// or `Some(None)` at the request-input EOF.
    pub fn read_request_input_line(&self) -> Option<Option<String>> {
        use std::io::BufRead;
        let mut request_input = self
            .request_input
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let reader = request_input.as_mut()?;
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => Some(None),
            Ok(_) => {
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Some(Some(line))
            }
            Err(_) => Some(None),
        }
    }

    /// Plan 011 (MS3-B): Install a shell host so that `system()` /
    /// `system_status()` / `export()` / `exit()` natives forward to it.
    pub fn set_host(&mut self, host: crate::host::SharedHost) {
        self.host = Some(host);
    }

    /// Plan 011: Remove the shell host (pure-AutoLang mode).
    pub fn clear_host(&mut self) {
        self.host = None;
    }

    /// Create VM with stdout capture for testing (Plan 177)
    pub fn new_with_capture(flash: VirtualFlash, ram_size: usize) -> (Self, Arc<RwLock<String>>) {
        let mut vm = Self::new(flash, ram_size);
        let buffer = Arc::new(RwLock::new(String::new()));
        vm.output_buffer = Some(buffer.clone());
        (vm, buffer)
    }

    /// Load strings from a module's string constant pool
    pub fn load_strings(&mut self, strings: Vec<Vec<u8>>) {
        // 2026-08-22:池整体替换时同步重建去重索引(见 add_string 注释),
        // 否则旧映射会指向失效索引。
        let mut dedup = self.string_dedup.lock().unwrap();
        dedup.clear();
        for (idx, s) in strings.iter().enumerate() {
            dedup.insert(s.clone(), idx);
        }
        // Plan 419 Phase 2: flash 编译期常量条目 pinned —— 永活永复用、
        // 免计费;字节码立即数只引用 pinned 条目(免重映射不变量,§3.1)。
        let n = strings.len();
        let mut pool = crate::vm::rc::PoolState::new();
        pool.ensure_len(n);
        for i in 0..n {
            pool.pinned[i] = true;
        }
        self.pool_state = std::sync::RwLock::new(pool);
        self.strings = Arc::new(RwLock::new(strings));
    }

    /// Plan 375: intern a byte string into the VM string pool, returning its
    /// index. Used by `inject_value` for Str globals (mirrors codegen's
    /// `add_string` but runs at injection time on an already-built VM).
    fn intern_string(&mut self, bytes: &[u8]) -> usize {
        self.add_string(bytes.to_vec())
    }

    /// Plan 199: Set a custom debugger controller
    pub fn set_debugger(&mut self, controller: Box<dyn crate::vm::debugger::DebuggerController>) {
        self.debugger = Arc::new(std::sync::Mutex::new(controller));
    }

    /// Plan 199: Enable execution trace collection with a max record limit
    pub fn enable_trace(&mut self, max_records: usize) {
        self.trace = Arc::new(std::sync::Mutex::new(Some(
            crate::vm::trace::TraceCollector::new(max_records),
        )));
    }

    /// Plan 199: Get trace output as JSON
    pub fn get_trace_json(&self) -> Option<String> {
        let trace = self.trace.lock().unwrap();
        trace.as_ref().map(|t| t.to_json())
    }

    /// Plan 212b Task 4: Merge additional native shims into this VM
    ///
    /// Rebuilds the internal Arc<NativeInterface> by merging in shims
    /// from another NativeInterface (e.g., from RustFfiBridge).
    pub fn merge_native_interface(&mut self, other: &NativeInterface) {
        let mut ni = (*self.native_interface).clone();
        ni.merge(other);
        self.native_interface = Arc::new(ni);
    }

    /// PLAN-785: 实例级 clipboard text provider 注入/覆盖（宿主 adapter、
    /// 外部 consumer、测试装配用）。覆盖构造期快照；只影响本 VM，不触碰
    /// 绑定身份（revision/proof 不失效）。读取面=shim 2925/2926。
    pub fn set_clipboard_text_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::ClipboardTextProvider>,
    ) {
        self.native_interface.set_clipboard_text_provider(provider);
    }

    /// PLAN-785: 本 VM 当前 clipboard text provider（None=缺席错误语义）。
    pub fn clipboard_text_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::ClipboardTextProvider>> {
        self.native_interface.clipboard_text_provider()
    }

    /// PLAN-790: 实例级 clipboard media provider 注入/覆盖（宿主 adapter、
    /// 外部 consumer、测试装配用）。覆盖构造期快照；只影响本 VM，不触碰
    /// 绑定身份（revision/proof 不失效）。读取面=shim 2934..2937。
    pub fn set_clipboard_media_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::ClipboardMediaProvider>,
    ) {
        self.native_interface.set_clipboard_media_provider(provider);
    }

    /// PLAN-790: 本 VM 当前 clipboard media provider（None=G3 值降级语义）。
    pub fn clipboard_media_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::ClipboardMediaProvider>> {
        self.native_interface.clipboard_media_provider()
    }

    /// PLAN-794: 实例级 scroll host provider 注入/覆盖（宿主 adapter、
    /// 外部 consumer、测试装配用）。覆盖构造期快照；只影响本 VM，不触碰
    /// 绑定身份（revision/proof 不失效）。读取面=shim 9900..9905/9922/9923。
    pub fn set_scroll_host_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::ScrollHostProvider>,
    ) {
        self.native_interface.set_scroll_host_provider(provider);
    }

    /// PLAN-794: 本 VM 当前 scroll host provider（None=降级臂语义）。
    pub fn scroll_host_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::ScrollHostProvider>> {
        self.native_interface.scroll_host_provider()
    }

    /// PLAN-802: 实例级 code editor provider 注入/覆盖（宿主 adapter、
    /// 外部 consumer、测试装配用）。覆盖构造期快照；只影响本 VM，不触碰
    /// 绑定身份（revision/proof 不失效）。读取面=shim 2910..2915/2919..
    /// 2924/2932/2933/2939/9906/9908..9913。
    pub fn set_code_editor_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::CodeEditorHostProvider>,
    ) {
        self.native_interface.set_code_editor_provider(provider);
    }

    /// PLAN-802: 本 VM 当前 code editor provider（None=T-01 冻结缺席臂）。
    pub fn code_editor_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::CodeEditorHostProvider>> {
        self.native_interface.code_editor_provider()
    }

    /// PLAN-804: 实例级 dialog host provider 注入/覆盖（宿主 adapter、
    /// 外部 consumer、测试装配用）。覆盖构造期快照；只影响本 VM，不触碰
    /// 绑定身份（revision/proof 不失效）。读取面=shim 2927/2928。
    pub fn set_dialog_host_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::DialogHostProvider>,
    ) {
        self.native_interface.set_dialog_host_provider(provider);
    }

    /// PLAN-804: 本 VM 当前 dialog host provider（None=缺席错误语义）。
    pub fn dialog_host_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::DialogHostProvider>> {
        self.native_interface.dialog_host_provider()
    }

    /// PLAN-804: 实例级 dnd host provider 注入/覆盖。读取面=shim 2938。
    pub fn set_dnd_host_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::DndHostProvider>,
    ) {
        self.native_interface.set_dnd_host_provider(provider);
    }

    /// PLAN-804: 本 VM 当前 dnd host provider（None=false 降级语义）。
    pub fn dnd_host_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::DndHostProvider>> {
        self.native_interface.dnd_host_provider()
    }

    /// PLAN-807: 实例级 editor snapshot provider 注入/覆盖（diff_snapshots
    /// 9916 供值；宿主 adapter、外部 consumer、测试装配用）。覆盖构造期
    /// 快照；只影响本 VM，不触碰绑定身份。
    pub fn set_editor_snapshot_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::EditorSnapshotProvider>,
    ) {
        self.native_interface.set_editor_snapshot_provider(provider);
    }

    /// PLAN-807: 本 VM 当前 editor snapshot provider（None="无法找到相应
    /// editor" err envelope 语义）。
    pub fn editor_snapshot_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::EditorSnapshotProvider>> {
        self.native_interface.editor_snapshot_provider()
    }

    /// PLAN-808: 实例级 terminal provider 注入/覆盖（宿主 adapter、
    /// 外部 consumer、测试装配用）。覆盖构造期快照；只影响本 VM，不触碰
    /// 绑定身份（revision/proof 不失效）。读取面=34 项 `auto.term.*`。
    pub fn set_terminal_provider(
        &self,
        provider: std::sync::Arc<dyn auto_terminal_core::TerminalHostProvider>,
    ) {
        self.native_interface.set_terminal_provider(provider);
    }

    /// PLAN-808: 本 VM 当前 terminal provider（None → shim 回落进程共享
    /// engine service，不是禁用引擎）。
    pub fn terminal_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn auto_terminal_core::TerminalHostProvider>> {
        self.native_interface.terminal_provider()
    }

    /// Plan 197 Task 9: Load generic registry from codegen
    pub fn load_generic_registry(
        &mut self,
        registry: crate::vm::generic_registry::GenericRegistry,
    ) {
        self.generic_registry = registry;
    }

    /// Plan 317 Phase 1: Load task handler registry from codegen.
    ///
    /// Without this, `AutoVM.task_handler_registry` stays empty (initialized at
    /// engine.rs:370) and the HANDLE_MSG opcode (engine.rs:5449) can never find
    /// a handler table — so `task` definitions' `on { ... }` handlers never run.
    /// Mirrors `load_generic_registry` above (take from codegen before finish()).
    pub fn load_task_handler_registry(
        &mut self,
        registry: crate::vm::task_handler::TaskHandlerRegistry,
    ) {
        self.task_handler_registry = registry;
    }

    /// Plan 118 Phase 4: Add a new string to the string pool
    /// Returns the index of the newly added string.
    ///
    /// 2026-08-22(字符串池去重):池索引在 natives 侧以 u16 截断
    /// (get_string(u16),上限 65535),而运行时拼接会为每个中间串新增
    /// 池条目 —— ash-gui show 的 `line = line + ch` 式逐字符拼装一次就
    /// 塞 ~1.8 万条,池溢出后 u16 回绕令既有字符串互相串写(实测:会话
    /// cwd 变成单字符 "r"、payload 丢失、进程静默退出)。池只增不减,
    /// 这里按内容去重:重复内容(样式串/标签/空串等运行时高频churn)
    /// 复用既有索引,池规模稳定在低位。根治(索引 u32 化/池 GC)记
    /// 引擎债——池生命周期债登记于
    /// docs/plans/KNOWN-DEBT-AND-RISKS.md(Plan 510 条目;over-release
    /// 记账清偿见 docs/plans/510-vm-pool-over-release.md;旧指针
    /// docs/plans/060 系闭包语法计划,主题不符已接正)。
    pub fn add_string(&self, bytes: Vec<u8>) -> usize {
        let dedup_hit = { self.string_dedup.lock().unwrap().get(&bytes).copied() };
        if let Some(idx) = dedup_hit {
            // dedup 键只在条目存活时存在(释放时删键)→ 命中即活条目。
            // Plan 432 D26 加固:残键指向墓碑槽时不可复活(canary 按
            // tombstone 判定而非 rc)——视作 miss 走 freelist/append 重建。
            // PLAN-053 P-053-8 残键防御:另有一族残键指向**已被复用覆写**
            // 的存活槽(幻影 freelist 条目让槽在 rc≥1 时被 freelist 弹出
            // 复用,内容覆写而旧键未删——POOLLOG 实测 #222→#223:键
            // "8f20…" 命中槽 2348,槽内容已轮转为 "你好",musk 点击会话
            // 实参由 id 漂移成会话名)。命中槽内容必须与请求字节一致,
            // 不一致视为 miss 走重内化——insert 覆盖残键,一键一槽不变量
            // 自愈,对任何注入源免疫。
            let content_matches = {
                let strings = self.strings.read().unwrap();
                strings.get(idx).map(|b| b.as_slice()) == Some(bytes.as_slice())
            };
            if !self.pool_is_tombstone(idx) && content_matches {
                if crate::pool_log_all() {
                    eprintln!(
                        "[POOLLOG #{:>4}] intern-dedup-hit {} content={:?}",
                        crate::pool_log_seq(),
                        idx,
                        String::from_utf8_lossy(&bytes)
                            .chars()
                            .take(12)
                            .collect::<String>()
                    );
                }
                return idx;
            }
            if crate::pool_log_all() && !content_matches && !self.pool_is_tombstone(idx) {
                eprintln!(
                    "[POOLLOG #{:>4}] intern-stale-key {} content={:?} != key {:?} — re-interning",
                    crate::pool_log_seq(),
                    idx,
                    self.strings
                        .read()
                        .unwrap()
                        .get(idx)
                        .map(|b| String::from_utf8_lossy(b)
                            .chars()
                            .take(12)
                            .collect::<String>())
                        .unwrap_or_default(),
                    String::from_utf8_lossy(&bytes)
                        .chars()
                        .take(12)
                        .collect::<String>()
                );
            }
        }
        // Plan 419 Phase 2: freelist 优先复用(池峰值 = 并发不同串数,
        // 而非累计创建数)。复用条目 rc=0,由 push 侧 +1 建立 stake。
        {
            let mut pool = self.pool_state.write().unwrap();
            // PLAN-053 P-053-8: 幻影 freelist 条目清扫——rc>0 的弹出槽是
            // 存活槽(不变量:freelist 槽 rc==0)。丢弃条目以恢复不变量,
            // 绝不复用:复用会覆写活内容+清零 rc,触发孤儿 release 下溢
            // 风暴并自续(实测槽 49299:rc=5 被复用→下溢 4294967295→再
            // free→再入 freelist;musk store 兄弟调用实参读到后落的 404
            // JSON 即此伤害)。首见打签名,注入源治理归 060 RC 债。
            let mut chosen: Option<usize> = None;
            while let Some(slot) = pool.freelist.pop() {
                let live = slot < pool.rc.len()
                    && pool.rc[slot].load(std::sync::atomic::Ordering::Relaxed) > 0;
                if live {
                    if pool.phantom_seen.insert(slot) {
                        eprintln!(
                            "[P053-8] phantom freelist entry dropped: slot {} (live holders, rc={})",
                            slot,
                            pool.rc[slot].load(std::sync::atomic::Ordering::Relaxed)
                        );
                    }
                    // Plan 510 G3:总次数计数(soak 断言通道;首见签名只去重打印)。
                    pool.phantom_drops
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    continue;
                }
                chosen = Some(slot);
                break;
            }
            if let Some(slot) = chosen {
                let mut strings = self.strings.write().unwrap();
                if slot < strings.len() {
                    strings[slot] = bytes.clone();
                    drop(strings);
                    pool.rc[slot] = std::sync::atomic::AtomicU32::new(0);
                    pool.tombstone[slot] = false;
                    drop(pool);
                    self.string_dedup
                        .lock()
                        .unwrap()
                        .insert(bytes.clone(), slot);
                    if crate::pool_log_all() {
                        eprintln!(
                            "[POOLLOG #{:>4}] intern-freelist {} content={:?}",
                            crate::pool_log_seq(),
                            slot,
                            String::from_utf8_lossy(&bytes)
                                .chars()
                                .take(12)
                                .collect::<String>()
                        );
                    }
                    // PLAN-062: 槽位写入新内容 = 状态面突变（dedup 命中不算）。
                    self.state_mutation_seq.fetch_add(1, Ordering::Relaxed);
                    return slot;
                }
                // strings 与 freelist 长度异常时兜底:还回槽位走追加。
                pool.freelist.push(slot);
            }
        }
        let mut strings = self.strings.write().unwrap();
        let idx = strings.len();
        strings.push(bytes.clone());
        drop(strings);
        self.pool_state.write().unwrap().ensure_len(idx + 1);
        self.string_dedup.lock().unwrap().insert(bytes, idx);
        if crate::pool_log_all() {
            eprintln!(
                "[POOLLOG #{:>4}] intern-append {} (pool len {})",
                crate::pool_log_seq(),
                idx,
                idx + 1
            );
        }
        // PLAN-062: 追加新槽 = 状态面突变（dedup 命中不算）。
        self.state_mutation_seq.fetch_add(1, Ordering::Relaxed);
        idx
    }

    /// Plan 423 P5 续修:运行期字符串入池并带份额入栈的咽喉。原各处裸
    /// `strings.push` 不进 pool_state(rc/pinned/tombstone 不覆盖)也不进
    /// dedup —— 条目落在计数数组边界外无份额;后续 add_string 的
    /// ensure_len 扩池后 rc=0 即"可释放",而栈/槽上仍持 nanbox 索引,
    /// 消费释放归零即悬垂(016-calendar `ds == today` EQ canary 实测
    /// engine.rs get_string tombstone)。统一收口到 add_string。
    pub fn intern_runtime_str(&self, task: &mut AutoTask, bytes: Vec<u8>) {
        let idx = self.add_string(bytes);
        self.rc_push_str_idx(task, idx);
    }

    // ============================================================================
    // Plan 077 Phase 4: Unified Object Registry Helper Methods
    // ============================================================================

    /// 累计一次忙等段(waits 次轮询,busy 时长)。
    /// PLAN-053 P-053-2: nil 家族判定 —— tag-null(encode_null，现行
    /// `None`/`null`/`nil` 字面量与 JSON null 的统一编码)或旧编码
    /// i32(-1)(历史 `null` 字面量/CREATE_NONE)或 i32(i32::MIN+1)
    /// (历史 `nil` 字面量)。EQ/NE/NULL_COALESCE 用它在等值/合并语义上
    /// 抹平新旧编码(存量持久化字段兼容)。
    /// Plan 539 W2 (T19): numeric tag test for mixed-type comparisons
    /// (f64/f32/i32 — bools excluded, they have their own bit-compare).
    /// PLAN-026 T-03 配套: 宽整数值归一(i32/i64 tag → i64 按值;算术
    /// 兜底臂与序比较宽整臂共用)。
    // ── PLAN-026 T-04: api.* 忙等预算观测(AUTO_VM_API_BUDGET=1 门控)──
    // 累计 call_fn_by_name 的 Yield 忙等段(5ms-sleep 轮询 HTTP 结果)的
    // 等待次数与忙等时长;handler(call_fn_by_name)退出时输出
    // [VM-API-BUDGET] fn=<name> waits=<n> busy_ms=<t>;busy_ms 超阈
    //(默认 100ms)无条件 warn(不受 env 门控——慢端点观测为防御面)。
    thread_local! {
        static API_BUSY_BUDGET: std::cell::RefCell<(u64, std::time::Duration)> =
            std::cell::RefCell::new((0, std::time::Duration::ZERO));
    }

    fn api_budget_accumulate(waits: u64, busy: std::time::Duration) {
        Self::API_BUSY_BUDGET.with(|b| {
            let mut b = b.borrow_mut();
            b.0 += waits;
            b.1 += busy;
        });
    }

    /// 预算阈值(ms;env AUTO_VM_API_BUDGET_MS 覆盖,缺省 100)。
    fn api_budget_threshold_ms() -> u128 {
        static THRESHOLD: std::sync::OnceLock<u128> = std::sync::OnceLock::new();
        *THRESHOLD.get_or_init(|| {
            std::env::var("AUTO_VM_API_BUDGET_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100)
        })
    }

    /// env 门控(AUTO_VM_API_BUDGET=1 时启用完整日志输出)。
    fn api_budget_enabled() -> bool {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var("AUTO_VM_API_BUDGET").as_deref() == Ok("1"))
    }

    /// handler 退出位报告:重置累计并按门控输出/超阈 warn。
    fn api_budget_report(fn_name: &str) {
        let (waits, busy) = Self::API_BUSY_BUDGET.with(|b| {
            let mut b = b.borrow_mut();
            let snap = (b.0, b.1);
            *b = (0, std::time::Duration::ZERO);
            snap
        });
        let busy_ms = busy.as_millis();
        if Self::api_budget_enabled() {
            eprintln!(
                "[VM-API-BUDGET] fn={} waits={} busy_ms={}",
                fn_name, waits, busy_ms
            );
        }
        let threshold = Self::api_budget_threshold_ms();
        if busy_ms > threshold {
            eprintln!(
                "WARN[VM-API-BUDGET] fn={} busy_ms={} exceeds {}ms (api.* busy-wait budget)",
                fn_name, busy_ms, threshold
            );
        }
    }

    /// Execute a single instruction from the given task's instruction stream.
    ///
    /// Returns `Ok(StepResult::Continue)` if the task should keep running,
    /// `Ok(StepResult::Terminated)` if the task has finished, or
    /// `Ok(StepResult::Yield)` if the task should pause the current batch.
    pub fn run_one_instruction(&self, task: &mut AutoTask) -> Result<StepResult, VMError> {
        let assembly_revision = (
            self.flash.memory.len(),
            self.native_interface.binding_revision(),
        );
        if task.assembly_checked != Some(assembly_revision) {
            crate::stdlib_assembly::reference::verify_linked_native_closure(
                &self.flash,
                task.ip,
                &self.native_interface,
                &self.strings.read().unwrap(),
            )
            .map_err(VMError::RuntimeError)?;
            task.assembly_checked = Some(assembly_revision);
        }
        // 1. Fetch
        if task.ip >= self.flash.memory.len() {
            return Ok(StepResult::Terminated);
        }

        // PLAN-062 T12: 解释步计数 + 周期性收割宽限窗到期的 dying 对象。
        let step = self.interp_steps.fetch_add(1, Ordering::Relaxed);
        if step & 0x3FF == 0 {
            self.reap_dying(step);
        }
        let op_byte = self.flash.read_u8(task.ip);
        task.ip += 1;
        if !OpCode::is_valid(op_byte) {
            return Err(VMError::RuntimeError(format!(
                "Invalid opcode: 0x{:02x} at ip={}",
                op_byte,
                task.ip - 1
            )));
        }
        let op: OpCode = op_byte.into();

        // Plan 423 P5 续修(诊断设施):每指令 trace。定位 RC 计数漂移时
        // 需要完整指令流对齐 retain/release(OnceLock 缓存,默认零成本)。
        if crate::is_vm_trace_ops() {
            eprintln!(
                "[VMOP] ip=0x{:04x} op={:?} sp={} bp={}",
                task.ip - 1,
                op,
                task.ram.sp,
                task.bp
            );
        }

        // Plan 199: Debugger hook — check if we should pause before executing
        {
            let mut dbg = self.debugger.lock().unwrap();
            let ctx = crate::vm::debugger::DebugContext {
                task: &task,
                current_op: op,
                ip: task.ip - 1,
                line: task.current_line,
                call_stack: &task.call_stack,
            };
            if dbg.should_pause(&ctx) {
                let action = dbg.on_pause(&ctx);
                match action {
                    crate::vm::debugger::DebuggerAction::Quit => {
                        return Err(VMError::RuntimeError("Debugger quit".to_string()));
                    }
                    crate::vm::debugger::DebuggerAction::Continue
                    | crate::vm::debugger::DebuggerAction::Step
                    | crate::vm::debugger::DebuggerAction::StepOver
                    | crate::vm::debugger::DebuggerAction::StepOut => {}
                }
            }
        }

        // 2. Decode & Execute
        match op {
            OpCode::NOP => {
                // Do nothing
            }
            OpCode::POP => {
                // Plan 419: 弹出的值若为引用 → 计数 -1(协议 §1 POP 行);
                // 槽位清零防双重释放(嵌套执行外层可能再扫死区)。
                // PLAN-062 T12: 堆份额按影子释放(裸副本零释放),字符串
                // 沿内容;影子随取随清。
                let nv = task.ram.pop_nv();
                let stake = task.ram.take_stake_at(task.ram.sp);
                if stake != 0 {
                    self.rc_release_id(stake);
                }
                if auto_val::is_string(nv) {
                    self.rc_release(nv);
                }
                task.ram.raw_nv[task.ram.sp] = 0;
            }
            OpCode::POP_N => {
                let n = self.flash.read_u8(task.ip);
                task.ip += 1;
                for _ in 0..n {
                    // PLAN-062 T12: 同 POP——影子释放 + 字符串内容释放。
                    let nv = task.ram.pop_nv();
                    let stake = task.ram.take_stake_at(task.ram.sp);
                    if stake != 0 {
                        self.rc_release_id(stake);
                    }
                    if auto_val::is_string(nv) {
                        self.rc_release(nv);
                    }
                    task.ram.raw_nv[task.ram.sp] = 0;
                }
            }
            OpCode::DUP => {
                // Plan 419: DUP 的值是引用时 +1(栈上多一个 owned slot)。
                // PLAN-667 (F-07): 份额出处门——源槽无影子的裸 i32≥4M 是
                // 真整数,裸拷贝零计数(内容性补计=幻影份额来源)。
                if task.ram.sp > 0 {
                    let src = task.ram.sp - 1;
                    let nv = task.ram.raw_nv[src];
                    let src_stake = task.ram.stake_at(src);
                    self.rc_push_slot(task, nv, src_stake);
                }
            }

            // === Constants ===
            OpCode::CONST_I32 => {
                let val = self.flash.read_i32(task.ip);
                task.ip += 4;
                task.ram.push_i32(val);
                vm_debug!(
                    "DEBUG: CONST_I32: sp after push={}, wrote to address {}",
                    task.ram.sp,
                    task.ram.sp - 1
                );
            }
            OpCode::CONST_F32 => {
                // Plan 073: Fixed to use push_f32 instead of push_i32
                let val = self.flash.read_f32(task.ip);
                task.ip += 4;
                task.ram.push_f32(val);
                task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
            }
            OpCode::CONST_F64 => {
                let val = self.flash.read_f64(task.ip);
                task.ip += 8;
                task.ram.push_f64(val);
            }
            OpCode::CONST_I64 => {
                // Plan 073: 64-bit integer constant (Plan 377: heap-aware for >2^47)
                let val = self.flash.read_i64(task.ip);
                task.ip += 8;
                self.push_i64_vm(task, val);
            }
            OpCode::CONST_U64 => {
                // Plan 073: 64-bit unsigned integer constant (Plan 377: heap-aware for >=2^48)
                let val = self.flash.read_u64(task.ip);
                task.ip += 8;
                self.push_u64_vm(task, val);
            }
            OpCode::CONST_0 => {
                task.ram.push_i32(0);
            }
            OpCode::CONST_1 => {
                task.ram.push_i32(1);
            }
            OpCode::LOAD_STR => {
                let str_idx = self.flash.read_u32(task.ip);
                task.ip += 4;
                // Push string reference (NaN-boxed tag)
                // Plan 419 Phase 2: flash 常量 pinned,rc_push 免计费直推。
                self.rc_push(task, auto_val::encode_string(str_idx as u32));
                // Reset result type since this produces a string, not a number
                task.last_result_type = ResultType::default();
            }
            // Plan 073: Node support
            OpCode::CREATE_NODE => {
                let name_idx = self.flash.read_u32(task.ip);
                task.ip += 4;
                let arg_count = self.flash.read_u8(task.ip);
                task.ip += 1;
                let id_idx = self.flash.read_u32(task.ip);
                task.ip += 4;

                // Pop kids_id and props_id first
                let kids_id = task.ram.pop_i32();
                let props_id = task.ram.pop_i32();

                // Pop args (in reverse order)
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    // Inside VM, everything is either a string tag or an int (VmRef id).
                    let val = match pop_tagged(&mut task.ram) {
                        StackTag::Str(str_idx) => {
                            let strings = self.strings.read().unwrap();
                            let v = strings
                                .get(str_idx as usize)
                                .map(|bytes| {
                                    auto_val::Value::Str(
                                        String::from_utf8_lossy(bytes).to_string().into(),
                                    )
                                })
                                .unwrap_or(auto_val::Value::Nil);
                            drop(strings);
                            // Plan 510 G3:物化拷贝后份额配平释放。
                            self.pool_release(str_idx as usize);
                            v
                        }
                        StackTag::Int(bits) => {
                            auto_val::Value::VmRef(auto_val::VmRef { id: bits as usize })
                        }
                    };
                    args.push(val);
                }
                args.reverse();

                // Decode name and id
                let strings = self.strings.read().unwrap();
                let name = if let Some(bytes) = strings.get(name_idx as usize) {
                    String::from_utf8_lossy(bytes).to_string()
                } else {
                    "".to_string()
                };
                let id = if id_idx != 0xFFFF {
                    strings
                        .get(id_idx as usize)
                        .map(|bytes| String::from_utf8_lossy(bytes).to_string())
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                drop(strings);

                let mut node = auto_val::Node::new(&name);
                if !id.is_empty() {
                    node.id = AutoStr::from(id);
                }

                // Assign args
                for arg in args {
                    node.add_arg(auto_val::Arg::Pos(arg));
                }

                // Assign props if available (Plan 390 §15 H3b: ObjectData
                // lives in heap_objects).
                if props_id >= 0 {
                    if let Some(props_ref) = self.get_heap_object(props_id as u64) {
                        let guard = props_ref.read().unwrap();
                        if let Some(props_data) = guard
                            .as_any()
                            .downcast_ref::<crate::vm::types::ObjectData>()
                        {
                            // Clone properties from ObjectData to Node
                            for (key, val) in &props_data.fields {
                                node.set_prop(key.clone(), val.clone());
                            }
                        }
                    }
                }

                // Assign kids if available
                if kids_id >= 0 {
                    // TODO: Implement kids array/list mapping
                }

                // Store node in heap_objects registry (Plan 390 §15 H3a)
                let node_id = self.insert_heap_object(node);
                self.rc_push(task, auto_val::encode_object(node_id as u32));
            }
            // Plan 364 Step 5: Config accumulation opcodes.
            // PUSH_ACCUM name_str_idx:u16, id_str_idx:u16 — push a fresh Node container.
            OpCode::PUSH_ACCUM => {
                let name_idx = self.flash.read_u32(task.ip);
                let id_idx = self.flash.read_u32(task.ip + 4);
                task.ip += 8;
                let (name, id) = {
                    let strings = self.strings.read().unwrap();
                    let name = strings
                        .get(name_idx as usize)
                        .map(|b| String::from_utf8_lossy(b).to_string())
                        .unwrap_or_default();
                    // 0xFFFF sentinel = no id.
                    let id = if id_idx != 0xFFFF {
                        strings
                            .get(id_idx as usize)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    (name, id)
                };
                let container = crate::vm::task::AccumContainer::new_node(
                    auto_val::AutoStr::from(name.as_str()),
                    auto_val::AutoStr::from(id.as_str()),
                );
                task.accum_stack.push(container);
            }
            // ACCUM_PAIR key_str_idx:u16 — pop value, set_prop(key, value) on top container.
            OpCode::ACCUM_PAIR => {
                let key_idx = self.flash.read_u32(task.ip);
                task.ip += 4;
                let value = self.pop_auto_value(task);
                let key = {
                    let strings = self.strings.read().unwrap();
                    strings
                        .get(key_idx as usize)
                        .map(|b| String::from_utf8_lossy(b).to_string())
                        .unwrap_or_default()
                };
                if let Some(container) = task.accum_stack.last_mut() {
                    container.set_field(auto_val::ValueKey::Str(key.into()), value);
                } else {
                    eprintln!("WARNING: ACCUM_PAIR with empty accum_stack");
                }
            }
            // ACCUM_NODE — pop a Node value, append it to top container's kids.
            OpCode::ACCUM_NODE => {
                let value = self.pop_auto_value(task);
                if let auto_val::Value::Node(node) = value {
                    if let Some(container) = task.accum_stack.last_mut() {
                        container.add_kid(*node);
                    } else {
                        eprintln!("WARNING: ACCUM_NODE with empty accum_stack");
                    }
                } else {
                    eprintln!("WARNING: ACCUM_NODE expected Node, got {:?}", value);
                }
            }
            // ACCUM_MERGE — pop an Obj value, merge all its fields into top container.
            OpCode::ACCUM_MERGE => {
                let value = self.pop_auto_value(task);
                if let auto_val::Value::Obj(obj) = value {
                    if let Some(container) = task.accum_stack.last_mut() {
                        container.merge_obj(&obj);
                    } else {
                        eprintln!("WARNING: ACCUM_MERGE with empty accum_stack");
                    }
                } else {
                    eprintln!("WARNING: ACCUM_MERGE expected Obj, got {:?}", value);
                }
            }
            // POP_ACCUM — pop top Node container, store in nodes registry, push id.
            OpCode::POP_ACCUM => {
                if let Some(container) = task.accum_stack.pop() {
                    let mut node = match container.into_value() {
                        auto_val::Value::Node(n) => *n,
                        other => {
                            eprintln!("WARNING: POP_ACCUM got unexpected value {:?}", other);
                            auto_val::Node::new(auto_val::AutoStr::new())
                        }
                    };
                    // Plan 364 Step 5: if the node has no explicit id yet
                    // (PUSH_ACCUM's id operand), but an `id:` Pair was
                    // accumulated into its props (e.g. `dir(id: d)` where d
                    // is a runtime variable), promote that prop to the
                    // node's id field so `node.id()` / `main_arg()` resolve
                    // it. We leave the prop in place for downstream readers.
                    if node.id.is_empty() {
                        let id_val = node.get_prop("id");
                        if matches!(id_val, auto_val::Value::Str(_)) {
                            node.id = id_val.to_astr();
                        }
                    }
                    let id = self.insert_heap_object(node);
                    self.rc_push(task, auto_val::encode_object(id as u32));
                } else {
                    eprintln!("WARNING: POP_ACCUM with empty accum_stack");
                    task.ram.push_i32(0);
                }
            }
            // Plan 073: Object literal support
            OpCode::CREATE_OBJ => {
                let key_index = self.flash.read_u32(task.ip);
                task.ip += 4;
                let field_count = self.flash.read_u8(task.ip);
                task.ip += 1;

                // Get keys from flash metadata
                let keys = &self.flash.object_keys[key_index as usize];
                // Get types from flash metadata
                let types = &self.flash.object_types[key_index as usize];

                // Pop values from stack (in reverse order since last value is on top).
                // Plan 402 §13.10: decode by the value's ACTUAL nanbox tag, not the
                // compile-time ObjectType. infer_object_type is unreliable for
                // variable/field references (e.g. `mine: cell.mine` infers
                // NestedObject instead of Bool), which caused CREATE_OBJ to pop
                // bool/int/string values with the wrong branch (NestedObject does
                // pop_i32() as usize → Value::VmRef with garbage ids). Decoding by
                // tag — the same approach the array push path uses (engine.rs push
                // handler) — is type-inferral-free and always correct.
                let mut values = Vec::with_capacity(field_count as usize);
                for i in 0..field_count {
                    let type_idx = (field_count - 1 - i) as usize;
                    let obj_type = types.get(type_idx).copied().unwrap_or(ObjectType::Int);
                    let nv = task.ram.pop_nv();
                    let value = if auto_val::is_future(nv) {
                        // PLAN-776 R9-02：Future 句柄字段**保真映射**
                        // Value::Future（句柄视图）——原落整数 payload 臂
                        // 把 fid 存成 Int(1)（F776-R9-02 反例 body=1）。
                        auto_val::Value::Future(Box::new(auto_val::FutureData {
                            id: auto_val::decode_future(nv) as usize,
                            state: auto_val::FutureState::Pending,
                            result: None,
                        }))
                    } else if auto_val::is_object(nv) {
                        auto_val::Value::VmRef(auto_val::VmRef {
                            id: auto_val::decode_object(nv) as usize,
                        })
                    } else if auto_val::is_string(nv) {
                        let str_idx = auto_val::decode_string(nv) as usize;
                        let strings = self.strings.read().unwrap();
                        let v = if let Some(str_bytes) = strings.get(str_idx) {
                            auto_val::Value::Str(
                                String::from_utf8_lossy(str_bytes).to_string().into(),
                            )
                        } else {
                            auto_val::Value::Nil
                        };
                        drop(strings);
                        // Plan 419 Phase 2: 对象字段持拷贝,池 stake 死亡。
                        self.pool_release(str_idx);
                        v
                    } else if auto_val::is_bool(nv) {
                        auto_val::Value::Bool(auto_val::decode_bool(nv))
                    } else if auto_val::is_null(nv) {
                        auto_val::Value::Nil
                    } else if auto_val::is_f64(nv) {
                        auto_val::Value::Double(auto_val::decode_f64(nv))
                    } else if auto_val::is_f32(nv) {
                        auto_val::Value::Float(auto_val::decode_f32(nv) as f64)
                    } else {
                        // i32 (and everything else): decode payload. Byte/Uint/Char
                        // are stored as raw i32 too — apply the obj_type narrowing
                        // only for these known-int payload types.
                        let bits = auto_val::decode_i32(nv);
                        match obj_type {
                            ObjectType::Byte => auto_val::Value::Byte(bits as u8),
                            ObjectType::Uint => auto_val::Value::Uint(bits as u32),
                            ObjectType::Char => char::from_u32(bits as u32)
                                .map(auto_val::Value::Char)
                                .unwrap_or(auto_val::Value::Nil),
                            _ => auto_val::Value::Int(bits),
                        }
                    };
                    values.push(value);
                }

                // Create object from key-value pairs
                let mut obj = crate::vm::types::ObjectData::new();
                for (i, key) in keys.iter().enumerate() {
                    // Values were popped in reverse order, so reverse them back
                    let val = &values[field_count as usize - 1 - i];
                    obj.set(key.clone(), val.clone());
                }

                // Store object in heap_objects and get ID (Plan 390 §15 H3b)
                let obj_id = self.insert_heap_object(obj);

                // Plan 390 §15 H2: push object id as a TAG_OBJECT-encoded
                // value so downstream consumers (GET_FIELD/SET_FIELD, send
                // shim's is_object check) recognize it as an object ref,
                // not a scalar i32. This unblocks L3 WithBindings.
                self.rc_push(task, auto_val::encode_object(obj_id as u32));
            }
            // Plan 073: Array literal support
            OpCode::CREATE_ARRAY => {
                let elem_count = self.flash.read_u8(task.ip);
                task.ip += 1;

                let mut elems = Vec::with_capacity(elem_count as usize);
                for _ in 0..elem_count {
                    {
                        let nv = task.ram.pop_nv();
                        let is_nil = nv == auto_val::encode_i32(-2147483647);
                        if !is_nil {
                            let value = if auto_val::is_string(nv) {
                                // Plan 370 (Issue 2 tags): decode the string tag
                                // to its pool index and look up the actual bytes,
                                // mirroring CREATE_OBJ's ObjectType::String arm.
                                // Previously this called decode_i32, which returns
                                // the raw negative tag (e.g. -4) — so []str array
                                // elements rendered as numbers like "-4".
                                let str_idx = auto_val::decode_string(nv) as usize;
                                let strings = self.strings.read().unwrap();
                                let v = if let Some(str_bytes) = strings.get(str_idx) {
                                    let s = String::from_utf8_lossy(str_bytes).to_string();
                                    auto_val::Value::Str(s.into())
                                } else {
                                    auto_val::Value::Nil
                                };
                                drop(strings);
                                // Plan 419 Phase 2: 数组元素持拷贝,池 stake 死亡。
                                self.pool_release(str_idx);
                                v
                            } else if auto_val::is_object(nv) {
                                auto_val::Value::VmRef(auto_val::VmRef {
                                    id: auto_val::decode_object(nv) as usize,
                                })
                            } else if auto_val::is_null(nv) {
                                auto_val::Value::Nil
                            } else if auto_val::is_bool(nv) {
                                auto_val::Value::Bool(auto_val::decode_bool(nv))
                            } else if auto_val::is_f64(nv) {
                                auto_val::Value::Double(auto_val::decode_f64(nv))
                            } else if auto_val::is_f32(nv) {
                                auto_val::Value::Float(auto_val::decode_f32(nv) as f64)
                            } else if auto_val::is_future(nv) {
                                // PLAN-776 R8-01：Future 句柄入数组——保真
                                // 映射 Value::Future（句柄视图）。
                                auto_val::Value::Future(Box::new(auto_val::FutureData {
                                    id: auto_val::decode_future(nv) as usize,
                                    state: auto_val::FutureState::Pending,
                                    result: None,
                                }))
                            } else {
                                auto_val::Value::Int(auto_val::decode_i32(nv))
                            };
                            elems.push(value);
                        }
                    }
                }

                elems.reverse();

                // Plan 390 §15 H3b: arrays live in heap_objects as
                // ListData<Value>, TAG_OBJECT-encoded on the stack.
                let array_id = self.insert_heap_object(crate::vm::types::ListData {
                    elems,
                    storage: None,
                });

                self.rc_push(task, auto_val::encode_object(array_id as u32));
            }
            // Plan 073: Range expression support (0..10, 0..=10)
            OpCode::CREATE_RANGE => {
                // Stack layout: [..., end, start]
                // Pop end first (top of stack), then start
                let end = task.ram.pop_i32();
                let start = task.ram.pop_i32();

                // Store range in ranges registry and push range_id
                let range_id = task.ram.ranges.len() as i32;
                task.ram.ranges.push((start, end, false)); // false = exclusive

                // Use special marker for range: -1000000 + range_id
                task.ram.push_i32(-1000000 + range_id);
            }
            OpCode::CREATE_RANGE_EQ => {
                // Stack layout: [..., end, start]
                // Pop end first (top of stack), then start
                let end = task.ram.pop_i32();
                let start = task.ram.pop_i32();

                // Create RangeEq value (inclusive)
                let _range_value = auto_val::Value::RangeEq(start, end);

                // Store range in ranges registry and push range_id
                let range_id = task.ram.ranges.len() as i32;
                task.ram.ranges.push((start, end, true)); // true = inclusive
                vm_debug!(
                    "DEBUG CREATE_RANGE_EQ: start={}, end={}, range_id={}",
                    start,
                    end,
                    range_id
                );

                // Use special marker for range: -1000000 + range_id
                task.ram.push_i32(-1000000 + range_id);
            }
            OpCode::ARRAY_LEN => {
                // Stack: array_id (raw i32 for legacy arrays, TAG_OBJECT for heap lists)
                {
                    let nv = task.ram.pop_nv();
                    // PLAN-604 T04: 结算弹出槽份额。栈顶引用是 copy-on-load
                    // 的暂存拷贝（rc_push +1 记影子），本 opcode 消费它——
                    // 原实现裸 pop 不结算，+1 永久孤儿（for-in 头部
                    // `dup; arr.len` 形态每拍漏一个列表对象，探针实测
                    // LitPushTick +4040/40=100 实例+1 列表）。DROP:同款
                    // 纪律——堆按影子释放，字符串按内容释放。释放后对象
                    // 入 dying 宽限窗，本 opcode 内 heap 查找仍安全。
                    {
                        let stake = task.ram.take_stake_at(task.ram.sp);
                        if stake != 0 {
                            self.rc_release_id(stake);
                        }
                        if auto_val::is_string(nv) {
                            self.rc_release(nv);
                        }
                    }
                    if auto_val::is_string(nv) {
                        // String .len() fallback — JS .length 语义 = 字符数
                        // （PLAN-055：此前按字节计，for-in over str 对 CJK
                        // 逐字节迭代 ×3 且 GET_ELEM 索引越界——迭代面失真）。
                        let len = self
                            .strings
                            .read()
                            .unwrap()
                            .get(auto_val::decode_string(nv) as usize)
                            .map(|b| String::from_utf8_lossy(b).chars().count() as i32)
                            .unwrap_or(0);
                        task.ram.push_i32(len);
                    } else if auto_val::is_i32(nv) || auto_val::is_object(nv) {
                        // Plan 390 §15 H2: accept both raw i32 (legacy arrays)
                        // and TAG_OBJECT (heap List from CREATE_LIST_*).
                        let array_id = if auto_val::is_object(nv) {
                            auto_val::decode_object(nv) as u64
                        } else {
                            auto_val::decode_i32(nv) as u64
                        };
                        // Plan 390 §15 H3b: arrays live in heap_objects.
                        if let Some(list) = self.heap_objects.get(&array_id) {
                            use crate::vm::types::ListData;
                            let guard = list.read().unwrap();
                            // Plan 539 W0 (DIV-PY-ITER-1): PyObjectHandle —
                            // GIL len() so for-in over sized py objects
                            // (tensors/lists/dicts) iterates like Python.
                            #[cfg(feature = "python")]
                            {
                                #[allow(unused_imports)]
                                use pyo3::prelude::*;
                                let py_len = guard
                                    .as_any()
                                    .downcast_ref::<crate::py_ffi::PyObjectHandle>()
                                    .map(|pyh| {
                                        pyo3::Python::attach(|py| {
                                            pyh.obj
                                                .clone_ref(py)
                                                .into_bound(py)
                                                .len()
                                                .map(|l| l as i32)
                                                .unwrap_or(0)
                                        })
                                    });
                                if let Some(n) = py_len {
                                    task.ram.push_i32(n);
                                    return Ok(StepResult::Continue);
                                }
                            }
                            let len = if let Some(list) =
                                guard.as_any().downcast_ref::<ListData<i32>>()
                            {
                                list.elems.len() as i32
                            } else if let Some(list) =
                                guard.as_any().downcast_ref::<ListData<String>>()
                            {
                                list.elems.len() as i32
                            } else if let Some(list) =
                                guard.as_any().downcast_ref::<ListData<bool>>()
                            {
                                list.elems.len() as i32
                            } else if let Some(list) =
                                guard.as_any().downcast_ref::<ListData<auto_val::Value>>()
                            {
                                list.elems.len() as i32
                            } else {
                                0
                            };
                            task.ram.push_i32(len);
                        } else {
                            task.ram.push_i32(0);
                        }
                    } else if auto_val::is_null(nv) {
                        // Plan 550 T04: null 迭代源守卫。ARRAY_LEN 是
                        // array 通道 for-in 的长度探针（codegen Plan 089），
                        // 现状落此静默 push 0 → 零迭代（p5 探针实证）。
                        // 同臂亦承接 .len() 发射点——null.len() 同翻为
                        // TypeError（Python: None 无 len）。
                        return Err(VMError::RuntimeError(
                            "TypeError: 'NoneType' object is not iterable".to_string(),
                        ));
                    } else {
                        task.ram.push_i32(0);
                    }
                }
            }
            // Plan 073: F-string support (f"hello $name")
            OpCode::BUILD_FSTR => {
                let part_count = self.flash.read_u8(task.ip);
                task.ip += 1;

                // Read type tags for each part: 0=i32, 1=string, 2=f64, 3=f32, 4=u64
                let mut type_tags = Vec::with_capacity(part_count as usize);
                for _ in 0..part_count {
                    type_tags.push(self.flash.read_u8(task.ip));
                    task.ip += 1;
                }

                // Pop parts from stack (in reverse order)
                // Plan 510:守卫不得跨弹栈持有——pop_tagged_rc 的
                // pool_release 归零会走 pool_free_idx 的 strings.write(),
                // 外层 read 守卫未放即同线程自锁(mold 模板测试实挂)。
                let mut parts = Vec::with_capacity(part_count as usize);
                for i in (0..part_count as usize).rev() {
                    let tag = type_tags[i];
                    // Plan 474 待澄清#5: 运行期 tag-first 转换。原实现盲信
                    // 编译期 expr_type_hint 标签——json/unknown 局部落 Int
                    // 提示时，裸 f64（encode_f64=原始位）被按 i32 解码成
                    // 低 32 位位型垃圾（54.16 → -515396076，与 ④ 同族）。
                    // 栈上实际 nv tag 优先，编译期标签仅作兜底（Int 兜底
                    // 中 TAG_BOOL 哨兵特判随之摘除——bool 已被上方 tag 检查
                    // 截走，残留分支只会把真整数 i32::MIN 误显为 true/false）。
                    let s = {
                        let nv = task.ram.peek_nv(0);
                        if auto_val::is_f64(nv) {
                            let val = auto_val::decode_f64(task.ram.pop_nv());
                            format!("{}", val)
                        } else if auto_val::is_i64(nv) {
                            // PLAN-026 T-03 配套: i64 插值串化(tag 8
                            // 此前缺分支,兜底 pop_tagged 的 Int(bits)
                            // ≥4000000 启发式把 i64 位型当 heap id 走
                            // 对象格式化 = 垃圾串/越界;快照消费模型
                            // 字段(px 几何)全 i64,受害面 = 布局串)。
                            let val = auto_val::decode_i64(task.ram.pop_nv());
                            format!("{}", val)
                        } else if auto_val::is_f32(nv) {
                            let val = auto_val::decode_f32(task.ram.pop_nv());
                            format!("{}", val)
                        } else if auto_val::is_bool(nv) {
                            let b = auto_val::decode_bool(task.ram.pop_nv());
                            if b {
                                "true".to_string()
                            } else {
                                "false".to_string()
                            }
                        } else if auto_val::is_null(nv) {
                            task.ram.pop_nv();
                            "None".to_string()
                        } else {
                            match tag {
                                2 => {
                                    let val = task.ram.pop_f64();
                                    format!("{}", val)
                                }
                                3 => {
                                    let val = task.ram.pop_f32();
                                    format!("{}", val)
                                }
                                4 => {
                                    let val = self.pop_u64_vm(task);
                                    format!("{}", val)
                                }
                                1 => {
                                    match pop_tagged(&mut task.ram) {
                                        StackTag::Str(idx) => {
                                            let bytes = {
                                                let strings = self.strings.read().unwrap();
                                                strings.get(idx as usize).cloned()
                                            };
                                            // Plan 510 G3:内容已拷贝,份额配平释放
                                            // (free 清内容,必须先拷贝后释放)。
                                            self.pool_release(idx as usize);
                                            match bytes {
                                                Some(b) => String::from_utf8_lossy(&b).to_string(),
                                                None => format!("<invalid_str_idx:{}>", idx),
                                            }
                                        }
                                        StackTag::Int(bits) => {
                                            if bits == -1 {
                                                "None".to_string()
                                            } else {
                                                bits.to_string()
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    match pop_tagged(&mut task.ram) {
                                        StackTag::Str(idx) => {
                                            let bytes = {
                                                let strings = self.strings.read().unwrap();
                                                strings.get(idx as usize).cloned()
                                            };
                                            // Plan 510 G3:内容已拷贝,份额配平释放
                                            // (free 清内容,必须先拷贝后释放)。
                                            self.pool_release(idx as usize);
                                            match bytes {
                                                Some(b) => String::from_utf8_lossy(&b).to_string(),
                                                None => format!("<invalid_str_idx:{}>", idx),
                                            }
                                        }
                                        StackTag::Int(bits) => {
                                            if bits == -1 {
                                                "None".to_string()
                                            } else if bits >= 4000000 {
                                                // Heap object — format as struct instance
                                                use crate::vm::generic_registry::GenericInstanceData;
                                                use crate::vm::heap_object::TypeTag;
                                                let obj_id = bits as u64;
                                                match self.get_heap_object(obj_id) {
                                                    Some(obj) => {
                                                        let guard = obj.read().unwrap();
                                                        if let TypeTag::RustStdlib(ref name) =
                                                            guard.type_tag()
                                                        {
                                                            // RustStdlibObject — check for string-like values
                                                            if let Some(rust_obj) = guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>() {
                                                            crate::vm::native::format_rust_stdlib_obj(rust_obj)
                                                        } else if let Some(dep_obj) = guard.as_any().downcast_ref::<crate::vm::ffi::dep_methods::DepOpaqueObject>() {
                                                            // PLAN-591 D2(DIV-DEP-8 print 半边):
                                                            // dep 对象 → shim 包 Display 合成 to_string;
                                                            // 无 Display 面维持占位。
                                                            let short_type = dep_obj.short_type.clone();
                                                            let fallback = format!("<{}>", name);
                                                            drop(guard);
                                                            match crate::vm::ffi::dep_methods::display_string(task, self, obj_id, &short_type)? {
                                                                Some(s) => s,
                                                                None => fallback,
                                                            }
                                                        } else {
                                                            format!("<{}>", name)
                                                        }
                                                        } else if let TypeTag::GenericInstance(_) =
                                                            guard.type_tag()
                                                        {
                                                            if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                                                            // Special formatting for Option/Result types
                                                            if inst.mono_name == "Option.None" {
                                                                "None".to_string()
                                                            } else if inst.mono_name.starts_with("Option.Some") {
                                                                if let Some(val) = inst.fields.first() {
                                                                    match val {
                                                                        Value::Int(i) => i.to_string(),
                                                                        Value::Str(s) => s.as_str().to_string(),
                                                                        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                                                                        _ => format!("{:?}", val),
                                                                    }
                                                                } else {
                                                                    "Some".to_string()
                                                                }
                                                            } else if inst.mono_name == "Result.Err" || inst.mono_name.starts_with("Result.Err") {
                                                                if let Some(val) = inst.fields.first() {
                                                                    format!("Err({})", match val {
                                                                        Value::Int(i) => i.to_string(),
                                                                        Value::Str(s) => s.as_str().to_string(),
                                                                        _ => format!("{:?}", val),
                                                                    })
                                                                } else {
                                                                    "Err".to_string()
                                                                }
                                                            } else if inst.mono_name.starts_with("Result.Ok") {
                                                                if let Some(val) = inst.fields.first() {
                                                                    match val {
                                                                        Value::Int(i) => i.to_string(),
                                                                        Value::Str(s) => s.as_str().to_string(),
                                                                        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                                                                        _ => format!("{:?}", val),
                                                                    }
                                                                } else {
                                                                    "Ok".to_string()
                                                                }
                                                            } else {
                                                                let type_name = self.generic_registry
                                                                    .get_type(&inst.mono_name)
                                                                    .map(|ct| ct.base_name().to_string())
                                                                    .unwrap_or_else(|| inst.mono_name.clone());
                                                                let field_strs: Vec<String> = inst.field_names.iter()
                                                                    .zip(inst.fields.iter())
                                                                    .map(|(name, val)| {
                                                                        let val_str = match val {
                                                                            Value::Int(i) => i.to_string(),
                                                                            Value::Uint(u) => u.to_string(),
                                                                            Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                                                                            Value::Float(f) => f.to_string(),
                                                                            Value::Double(d) => d.to_string(),
                                                                            Value::Str(s) => format!("\"{}\"", s.as_str()),
                                                                            Value::VmRef(r) => format!("<heap:{}>", r.id),
                                                                            Value::Nil => "nil".to_string(),
                                                                            _ => format!("{:?}", val),
                                                                        };
                                                                        format!("{}: {}", name, val_str)
                                                                    })
                                                                    .collect();
                                                                format!("{} {{ {} }}", type_name, field_strs.join(", "))
                                                            }
                                                        } else {
                                                            bits.to_string()
                                                        }
                                                        } else {
                                                            bits.to_string()
                                                        }
                                                    }
                                                    None => bits.to_string(),
                                                }
                                            } else {
                                                bits.to_string()
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    };
                    parts.push(s);
                }
                parts.reverse();

                // Join all parts into a single string
                let result = parts.join("");

                // Add to strings pool and push tagged index(dedup:重复
                // 中间串不再膨胀池 —— 2026-08-22 侧栏串写根因治理)
                let result_idx = self.add_string(result.into_bytes());
                self.rc_push_str_idx(task, result_idx as usize);
            }
            OpCode::NULL_COALESCE => {
                {
                    // Pop right expression (default value)
                    let default_nv = task.ram.pop_nv();
                    // Pop left expression (May<T> value)
                    let may_nv = task.ram.pop_nv();

                    // PLAN-053 P-053-2: 判空与 IS_NIL/UNWRAP_SOME 对齐——
                    // 旧编码 i32(-1)/i32(i32::MIN+1) 的 nil 家族值也落
                    // default(原只认 is_null，`null ?? x` 落 -1)。
                    if Self::nv_is_null_family(may_nv) {
                        // Plan 419: None → default 转移回栈,may 死亡。
                        self.rc_release(may_nv);
                        task.ram.push_nv(default_nv);
                    } else if auto_val::is_object(may_nv) {
                        let obj_id = auto_val::decode_object(may_nv) as u64;
                        if self.is_option_none(obj_id) {
                            self.rc_release(may_nv);
                            task.ram.push_nv(default_nv);
                        } else if self.is_option_some(obj_id) {
                            if let Some(field_val) = self.get_option_inner(obj_id) {
                                // Plan 419: 容器 stake 死亡,内值入栈 +1;
                                // default 丢弃 -1。
                                self.rc_release(may_nv);
                                self.rc_release(default_nv);
                                Self::push_value(task, &field_val, self);
                            } else {
                                task.ram.push_nv(may_nv);
                                self.rc_release(default_nv);
                            }
                        } else {
                            // Plan 539 W0 (DIV-PY-EXCEPT-1): Result values
                            // participate in `??` fallback — Err yields the
                            // default, Ok unwraps the inner value (previously
                            // Result objects passed through opaquely).
                            use crate::vm::generic_registry::GenericInstanceData;
                            let result_kind = self.get_heap_object(obj_id).and_then(|obj| {
                                let guard = obj.read().unwrap();
                                guard
                                    .as_any()
                                    .downcast_ref::<GenericInstanceData>()
                                    .and_then(|inst| match inst.mono_name.as_str() {
                                        "Result.Err" => Some(None),
                                        "Result.Ok" => {
                                            inst.fields.first().cloned().map(|f| Some(f))
                                        }
                                        _ => None,
                                    })
                            });
                            match result_kind {
                                // Result.Err — fall back to the default.
                                Some(None) => {
                                    self.rc_release(may_nv);
                                    task.ram.push_nv(default_nv);
                                }
                                // Result.Ok — unwrap the inner value.
                                Some(Some(field_val)) => {
                                    self.rc_release(may_nv);
                                    self.rc_release(default_nv);
                                    Self::push_value(task, &field_val, self);
                                }
                                _ => {
                                    task.ram.push_nv(may_nv);
                                    self.rc_release(default_nv);
                                }
                            }
                        }
                    } else {
                        // Non-None, non-object: push as-is (it's the value itself)
                        task.ram.push_nv(may_nv);
                        self.rc_release(default_nv);
                    }
                }
            }
            // Plan 073: May<T> error propagate operator: expression.?
            // Plan 208: Also handles Result.Ok / Result.Err heap objects with early return
            OpCode::ERROR_PROPAGATE => {
                use crate::vm::generic_registry::GenericInstanceData;
                // Read n_args for potential early return
                let n_args = self.flash.read_u8(task.ip) as usize;
                task.ip += 1;

                // Pop May<T> value from stack
                let may_nv = task.ram.pop_nv();
                let may_bits = auto_val::decode_i32(may_nv);

                // Determine if this is an error case that should propagate
                let should_propagate;
                let mut propagate_value = 0;
                // Plan 567 T08（P560-D2）: Err 值传播可被当前帧的 try
                // handler 拦截（值通道 catch）；None/null 是值不进 catch。
                let mut interceptable_err = false;

                // Plan 197 Task 16: Check if May<T> is Option.None (heap object or old -1)
                let is_none = if auto_val::is_null(may_nv) {
                    true
                } else if may_bits == -1 {
                    true
                } else if may_bits > 0 {
                    self.is_option_none(may_bits as u64)
                } else {
                    false
                };

                if is_none {
                    // Nil case: early return (error propagation)
                    // Push an Option.None sentinel for the caller
                    should_propagate = true;
                    propagate_value = -1;
                    // Plan 419: 弹出的 None 容器 stake 死亡。
                    self.rc_release(may_nv);
                } else if may_bits > 0 {
                    // Positive value: could be heap object (Option.Some, Result.Ok, Result.Err)
                    // or legacy plain positive integer
                    if let Some(obj) = self.get_heap_object(may_bits as u64) {
                        // Plan 419: 先克隆内值再释放容器 —— 容器 RC 归零会级联
                        // 释放子引用,不能在容器借用存续期触碰其字段。
                        enum MayKind {
                            Err,
                            WithInner(auto_val::Value),
                            Bare,
                        }
                        let kind = {
                            let guard = obj.read().unwrap();
                            if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>()
                            {
                                match inst.mono_name.as_str() {
                                    "Result.Err" => MayKind::Err,
                                    "Result.Ok" | "Option.Some" => match inst.fields.first() {
                                        Some(f) => MayKind::WithInner(f.clone()),
                                        None => MayKind::Bare,
                                    },
                                    _ => MayKind::Bare,
                                }
                            } else {
                                MayKind::Bare
                            }
                        };
                        match kind {
                            MayKind::Err => {
                                // Error case: propagate the Result.Err object to
                                // caller via early return (stack stake 转移)。
                                should_propagate = true;
                                propagate_value = may_bits;
                                interceptable_err = true;
                            }
                            MayKind::WithInner(field) => {
                                // Unwrap the inner value (continue execution)。
                                // Plan 419: 容器 stake 死亡;内值入栈 +1
                                // (push_value 的 VmRef 臂已 retain)。
                                should_propagate = false;
                                Self::push_value(task, &field, self);
                                self.rc_release_id(may_bits as u64);
                            }
                            MayKind::Bare => {
                                // Other/non-generic heap object: pass through。
                                should_propagate = false;
                                task.ram.push_i32(may_bits);
                            }
                        }
                    } else {
                        // Legacy: plain positive value (not a heap object)
                        should_propagate = false;
                        task.ram.push_i32(may_bits);
                    }
                } else {
                    // Negative value: legacy Err sentinel or string nanbox (decoded)
                    should_propagate = false;
                    {
                        task.ram.push_nv(may_nv);
                    }
                }

                // Perform early return if propagating error
                if should_propagate {
                    // Plan 567 T08（P560-D2）: 值通道拦截——Err 传播遇到
                    // 当前帧的 try handler 时改跳 catch_pc 绑定载荷（设计
                    // §4.3「catch 拦值传播」——两通道在此汇合；对齐
                    // intercept_error 的异常通道行为：栈回卷到 try 入口 +
                    // 载荷串入栈）。None/null 传播不经此（值不进 catch）。
                    if interceptable_err {
                        if let Some(handler) = task.handler_stack.last().copied() {
                            if handler.bp == task.bp {
                                task.handler_stack.pop();
                                let payload = self.result_err_payload(propagate_value);
                                // 容器 stake 死亡（消费于 catch）。
                                self.rc_release_id(propagate_value as u64);
                                let sp_at_unwind = task.ram.sp;
                                self.rc_release_slot_range(&mut task.ram, handler.sp, sp_at_unwind);
                                task.ram.sp = handler.sp;
                                let idx = self.add_string(payload.into_bytes());
                                self.rc_push_str_idx(task, idx);
                                task.ip = handler.catch_pc;
                                return Ok(StepResult::Continue);
                            }
                        }
                    }
                    if task.bp == 0 {
                        // Plan 567 T10（P560-D2）: 未捕获 Err 传播到 main
                        // 边界 → 带错退出（= Python traceback 等价物）。
                        // 原形态为静默 Terminated（退出码 0）——p7 四套件
                        // 中止时无消息可寻的根因面。None/null 传播维持
                        // 原 May 语义（main 正常结束）。
                        if interceptable_err {
                            let payload = self.result_err_payload(propagate_value);
                            self.rc_release_id(propagate_value as u64);
                            return Err(VMError::RuntimeError(payload));
                        }
                        // Main task: just push the error value and terminate
                        task.ram.push_i32(propagate_value);
                        return Ok(StepResult::Terminated);
                    }
                    // Perform RET-like frame unwinding
                    let old_bp = task.ram.read_i32(task.bp) as usize;
                    let ret_ip = task.ram.read_i32(task.bp - 1) as usize;
                    task.current_closure_id = task.saved_closure_id;
                    let new_sp = task.bp - n_args;
                    // Plan 419: 帧扫描 —— 释放被展开区间内每个引用槽
                    // (含 new_sp-1 槽的旧值;返回值槽由 write 重写)。
                    let new_sp = new_sp.max(1);
                    let sp_before = task.ram.sp;
                    self.rc_release_slot_range(&mut task.ram, new_sp - 1, sp_before);
                    task.ram.write_i32(new_sp - 1, propagate_value);
                    task.bp = old_bp;
                    task.ip = ret_ip;
                    task.ram.sp = new_sp;
                    task.ram.write_i32(new_sp - 1, propagate_value);
                }
            }
            // Plan 162: Type cast opcodes — runtime type conversion
            OpCode::TYPE_CAST_I32 => {
                let v = if task.last_result_type == ResultType::Float {
                    let f = task.ram.pop_f32();
                    f as i32
                } else {
                    task.ram.pop_i32()
                };
                task.ram.push_i32(v);
                task.last_result_type = ResultType::Int;
            }
            OpCode::TYPE_CAST_U32 => {
                let v = if task.last_result_type == ResultType::Float {
                    let f = task.ram.pop_f32();
                    f as u32 as i32
                } else {
                    let v = task.ram.pop_i32();
                    v as u32 as i32
                };
                task.ram.push_i32(v);
                task.last_result_type = ResultType::Uint;
            }
            OpCode::TYPE_CAST_I64 => {
                // PLAN-026 T-03 配套: 宽整入参 sign-extend → i64 TAG 单槽。
                // 原为 no-op(push_i32 回填)——i64 var 持 i32 tag,与 I64
                // 语义链(GET_ELEM 下标/串化/数值比较)全不接;且入参
                // 可能已是 TAG_I64(loop 内赋值位 contains_u64 判定随
                // 作用域波动,此时 pop_i32 位读 = 0x8000_000x 垃圾,
                // i64_probe24 "2147483649u" 实证)。nv_int_as_i64 按 tag
                // 归一,双形态幂等。
                let nv = task.ram.pop_nv();
                let v = Self::nv_int_as_i64(nv).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
                task.ram.push_i64(v as i64);
                task.last_result_type = ResultType::Int;
            }
            OpCode::TYPE_CAST_U64 => {
                // Plan 377: 单槽 u64 —— zero-extend i32 to u64（单 NanoValue）
                let v = task.ram.pop_i32();
                task.ram.push_u64(v as u32 as u64);
            }
            OpCode::PROMOTE_F64 => {
                // Plan 377: f32 与 f64 现在都是单槽，widen 仅是编码转换
                let val_f32 = task.ram.pop_f32();
                task.ram.push_f64(val_f32 as f64);
            }
            _ => return self.ops_construct(task, op, op_byte),
        }

        Ok(StepResult::Continue)
    }

    /// Plan 224: Execute bytecode for a single frame until a boundary condition.
    /// Returns FrameResult indicating what stopped execution.
    /// This can be called recursively for AWAIT_FUTURE body execution.
    pub fn execute_single_frame(&self, task: &mut AutoTask, budget: u32) -> FrameResult {
        for _ in 0..budget {
            let ip_before = task.ip;
            let line_before = task.current_line;
            match self.run_one_instruction(task) {
                Ok(StepResult::Continue) => {
                    self.record_trace(ip_before, line_before, task);
                }
                Ok(StepResult::Terminated) => return FrameResult::Return,
                Ok(StepResult::Yield) => return FrameResult::Yielded,
                Ok(StepResult::GeneratorYield) => {
                    // Plan 321: Generator yield in frame context — yield the frame
                    // so the generator driver can handle it.
                    return FrameResult::Yielded;
                }
                Ok(StepResult::AwaitFuture {
                    future_id,
                    body_offset,
                }) => {
                    return FrameResult::AwaitFuture {
                        future_id,
                        body_offset,
                    };
                }
                Err(e) => {
                    // Plan 010 (MS3-A): try/catch interception.
                    if self.intercept_error(task, &e) {
                        continue;
                    }
                    return FrameResult::Error(e);
                }
            }
        }
        FrameResult::BudgetExhausted
    }

    /// Plan 010 (MS3-A): try/catch interception.
    ///
    /// If the task has a handler frame on its stack, the error is caught:
    /// Plan 567 T08/T10（P560-D2）: Result.Err 载荷串——fields[0] Str
    /// （`PyException <Type>: <msg>`）；非 Str/缺容兜底描述。
    fn result_err_payload(&self, obj_id_bits: i32) -> String {
        use crate::vm::generic_registry::GenericInstanceData;
        self.get_heap_object(obj_id_bits as u64)
            .and_then(|obj| {
                let guard = obj.read().unwrap();
                guard
                    .as_any()
                    .downcast_ref::<GenericInstanceData>()
                    .and_then(|inst| match inst.fields.first() {
                        Some(auto_val::Value::Str(s)) => Some(s.as_str().to_string()),
                        _ => Some(format!("Result.Err {}", obj_id_bits)),
                    })
            })
            .unwrap_or_else(|| format!("Result.Err {}", obj_id_bits))
    }

    /// unwind to the recorded bp/sp, push the error message as a value, and
    /// jump to the catch handler. Returns `true` if caught (caller continues
    /// execution), `false` if the error should propagate (caller returns Err).
    fn intercept_error(&self, task: &mut AutoTask, e: &VMError) -> bool {
        // Plan 011: ExitRequested is a cooperative stop, not a catchable
        // runtime error — it must propagate past try/catch boundaries.
        if matches!(e, VMError::ExitRequested(_) | VMError::Halt) {
            return false;
        }
        let Some(handler) = task.handler_stack.pop() else {
            return false;
        };
        let msg = match e {
            VMError::RuntimeError(m) => m.clone(),
            other => format!("{:?}", other),
        };
        // Unwind the operand stack to the try-entry point. bp is left
        // unchanged because the try/catch runs in the same frame — restoring
        // bp here would fool the function-call loop into thinking the
        // function has returned.
        // Plan 419: 被展开区间 [handler.sp, sp) 内的引用槽逐个 -1(错误路径
        // 不丢帧清理 → 悬空计数)。
        let sp_at_unwind = task.ram.sp;
        self.rc_release_slot_range(&mut task.ram, handler.sp, sp_at_unwind);
        task.ram.sp = handler.sp;
        // PLAN-705 T-04: 跨帧展开——错误可能发生在 try 帧的**被调帧**内
        //（服务端 handler 的 park/resume 恢复路径：try 在外层 fn、除零等
        // 错误在深层被调 fn）。此时必须弃掉被调帧的调用栈账并恢复 try 帧
        // bp，否则 catch 体 RET 弹错帧 → 返回地址错乱（现场 ip 出界
        // Terminated，catch 永不生效）。同帧情形（错误与 try 同帧，既有
        // 702 测试面）无帧可弃，行为逐字节不变。
        // PLAN-781 T-22（F-781-E01 根修）：CallFrame.old_bp 全仓一致语义
        // = **帧自身 bp**（五处压帧点均在 `task.bp = sp-1` 之后捕获）。
        // 原 `>=` 把 try 帧**自己**也弹掉——同帧 catch 后 RET 弹空，
        // current_fn_n_args 不再恢复，调用方 LOAD_LOCAL 参数臂
        // （bp - n_args + param_idx - 1 依赖 task.current_fn_n_args）走
        // null 守卫 → 调用方参数/局部槽位读作 None（跨帧 raise 毒化，
        // 最小复现 maybe_raise/caller 实锤）。改严格 `>` 只弹被调帧。
        // 弹帧时同步恢复 n_args/n_locals 链（最外被弹帧的 old_* 即 try
        // 帧的值）——否则 catch 体内参数寻址仍读被调帧计数。
        let mut last_popped: Option<crate::vm::task::CallFrame> = None;
        while let Some(fr) = task.call_stack.last() {
            if fr.old_bp > handler.bp {
                last_popped = task.call_stack.pop();
            } else {
                break;
            }
        }
        if let Some(fr) = last_popped {
            task.current_fn_n_args = fr.old_fn_n_args;
            task.current_fn_n_locals = fr.old_fn_n_locals;
            task.num_locals = task.current_fn_n_locals;
        }
        task.bp = task.bp.min(handler.bp);
        // Push the error message as a string value for the catch handler to
        // bind to its parameter. The string is added to the pool at runtime
        // and its index pushed as a tag.
        let idx = self.add_string(msg.clone().into_bytes());
        self.rc_push_str_idx(task, idx as usize);
        task.ip = handler.catch_pc;
        // Note: do NOT set task.last_error here — the error was caught and is
        // being handled. extract_autovm_result treats last_error as a fatal
        // failure, which would wrongly fail an otherwise-successful catch.
        true
    }

    /// Execute a chunk of opcodes for a specific task
    fn execute_task(&self, task: &mut AutoTask) -> Result<TaskStatus, VMError> {
        // Plan 394 Phase B: resume suspended `~{}` body (result already pushed).
        if task.waiting_future_id.is_none() {
            if let Some(frame) = task.async_frames.last().copied() {
                return self.resume_suspended_body(task, frame);
            }
        }
        const BUDGET: u32 = 100;
        let result = self.execute_single_frame(task, BUDGET);
        match result {
            FrameResult::Return => Ok(TaskStatus::Terminated),
            FrameResult::Yielded => {
                if matches!(task.status, TaskStatus::Waiting(_)) {
                    Ok(task.status.clone())
                } else {
                    Ok(TaskStatus::Ready)
                }
            }
            FrameResult::AwaitFuture {
                future_id,
                body_offset,
            } => {
                self.handle_await_future(task, future_id, body_offset)?;
                if task.waiting_future_id.is_some() {
                    return Ok(TaskStatus::Waiting("future".into()));
                }
                Ok(TaskStatus::Ready)
            }
            FrameResult::BudgetExhausted => Ok(TaskStatus::Ready),
            FrameResult::Continue => unreachable!(),
            FrameResult::Error(e) => Err(e),
        }
    }

    /// Plan 199: Record a trace entry if tracing is enabled
    fn record_trace(&self, ip: usize, line: u32, task: &AutoTask) {
        let mut trace = self.trace.lock().unwrap();
        if let Some(ref mut collector) = *trace {
            let op_name = if ip < self.flash.memory.len() {
                let byte = self.flash.read_u8(ip);
                if OpCode::is_valid(byte) {
                    OpCode::from(byte).to_mnemonic().to_string()
                } else {
                    format!("0x{:02x}", byte)
                }
            } else {
                "EOF".to_string()
            };
            collector.record(ip, &op_name, line, task.ram.sp, task.call_stack.len());
        }
    }
}

// ============================================================================
// Plan 550 T08: null 家族守卫单测——直推 null 调 opcode，断言 VMError
// 消息格式（539 py_ffi 单测同型：AutoVM::new(VirtualFlash::new_with_code)
// + AutoTask::new，run_one_instruction 单步驱动）。
// ============================================================================
#[cfg(test)]
mod tests_null_guards {
    use super::*;
    use crate::vm::task::AutoTask;
    use crate::vm::virt_memory::VirtualFlash;

    fn vm_with(code: Vec<u8>) -> AutoVM {
        AutoVM::new(VirtualFlash::new_with_code(code), 1024)
    }

    fn runtime_err_of(res: Result<StepResult, VMError>) -> String {
        match res {
            Err(VMError::RuntimeError(msg)) => msg,
            other => panic!("expected RuntimeError, got {:?}", other),
        }
    }

    // ---- T03 算术族 ----

    #[test]
    fn test_null_add_none_left() {
        let vm = vm_with(vec![OpCode::ADD as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_nv(auto_val::encode_null());
        task.ram.push_i32(1);
        let msg = runtime_err_of(vm.run_one_instruction(&mut task));
        assert_eq!(
            msg,
            "TypeError: unsupported operand type(s) for +: 'NoneType' and 'int'"
        );
    }

    #[test]
    fn test_null_add_none_right() {
        let vm = vm_with(vec![OpCode::ADD as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_i32(1);
        task.ram.push_nv(auto_val::encode_null());
        let msg = runtime_err_of(vm.run_one_instruction(&mut task));
        assert_eq!(
            msg,
            "TypeError: unsupported operand type(s) for +: 'int' and 'NoneType'"
        );
    }

    #[test]
    fn test_null_sub_mul_div_mod() {
        for (op, sym) in [
            (OpCode::SUB, "-"),
            (OpCode::MUL, "*"),
            (OpCode::DIV, "/"),
            (OpCode::MOD, "%"),
        ] {
            let vm = vm_with(vec![op as u8]);
            let mut task = AutoTask::new(1, 256, 0);
            task.ram.push_nv(auto_val::encode_null());
            task.ram.push_i32(2);
            let msg = runtime_err_of(vm.run_one_instruction(&mut task));
            assert_eq!(
                msg,
                format!(
                    "TypeError: unsupported operand type(s) for {}: 'NoneType' and 'int'",
                    sym
                )
            );
        }
    }

    #[test]
    fn test_null_neg_unary() {
        let vm = vm_with(vec![OpCode::NEG as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_nv(auto_val::encode_null());
        let msg = runtime_err_of(vm.run_one_instruction(&mut task));
        assert_eq!(msg, "TypeError: bad operand type for unary -: 'NoneType'");
    }

    #[test]
    fn test_null_strcat_concat() {
        // "a" + null：含 str 的 + 经 codegen 静态路由到 STR_CAT（539 探针 2
        // 病灶实际落点）；守卫在解码前拦截，消息报 op='+' 类型 'str'。
        let vm = vm_with(vec![OpCode::STR_CAT as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_nv(auto_val::encode_string(0));
        task.ram.push_nv(auto_val::encode_null());
        let msg = runtime_err_of(vm.run_one_instruction(&mut task));
        assert_eq!(
            msg,
            "TypeError: unsupported operand type(s) for +: 'str' and 'NoneType'"
        );
    }

    #[test]
    fn test_legit_minus_one_arithmetic_unaffected() {
        // 合法 i32(-1) / i32::MIN+1 算术不得被守卫误伤（守卫只拒 TAG_NULL，
        // 历史 i32 哨兵编码与真实整数不可区分，不在守卫范围）。
        let vm = vm_with(vec![OpCode::ADD as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_i32(-1);
        task.ram.push_i32(1);
        vm.run_one_instruction(&mut task).unwrap();
        assert_eq!(task.ram.pop_i32(), 0);

        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_i32(i32::MIN + 1);
        task.ram.push_i32(1);
        vm.run_one_instruction(&mut task).unwrap();
        assert_eq!(task.ram.pop_i32(), i32::MIN + 2);
    }

    // ---- T04 GET_ELEM / CALL_CLOSURE / 迭代源 ----

    #[test]
    fn test_null_getelem_not_subscriptable() {
        let vm = vm_with(vec![OpCode::GET_ELEM as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_nv(auto_val::encode_null());
        task.ram.push_i32(0);
        let msg = runtime_err_of(vm.run_one_instruction(&mut task));
        assert_eq!(msg, "TypeError: 'NoneType' object is not subscriptable");
    }

    #[test]
    fn test_null_callee_not_callable() {
        // 正常模式 null callee 被静态解析在编译期拦下（p4 探针 E0401），
        // .at 探针不可达 VM 路径——CALL_CLOSURE 守卫由此单测钉住。
        let vm = vm_with(vec![OpCode::CALL_CLOSURE as u8, 0]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_nv(auto_val::encode_null());
        let msg = runtime_err_of(vm.run_one_instruction(&mut task));
        assert_eq!(msg, "TypeError: 'NoneType' object is not callable");
    }

    #[test]
    fn test_null_array_len_not_iterable() {
        // array 通道 for-in 的长度探针（p5 探针病灶实际落点）；同臂承接
        // null.len() 发射点。
        let vm = vm_with(vec![OpCode::ARRAY_LEN as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_nv(auto_val::encode_null());
        let msg = runtime_err_of(vm.run_one_instruction(&mut task));
        assert_eq!(msg, "TypeError: 'NoneType' object is not iterable");
    }

    // ---- T05 越界 ----

    #[test]
    fn test_oob_getelem_index_error() {
        use crate::vm::types::ListData;
        let vm = vm_with(vec![OpCode::GET_ELEM as u8]);
        let list_id = vm.insert_heap_object(ListData::<i32> {
            elems: vec![1, 2, 3],
            storage: None,
        });
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_nv(auto_val::encode_object(list_id as u32));
        task.ram.push_i32(999);
        let msg = runtime_err_of(vm.run_one_instruction(&mut task));
        assert_eq!(msg, "IndexError: index 999 out of range");
    }

    // ---- T06 TYPE_TO_I32/F64 翻案 + 合法回归 ----

    #[test]
    fn test_null_to_i32_f64_type_error() {
        let vm = vm_with(vec![OpCode::TYPE_TO_I32 as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_nv(auto_val::encode_null());
        let msg = runtime_err_of(vm.run_one_instruction(&mut task));
        assert_eq!(
            msg,
            "TypeError: int() argument must be a string or a real number, not 'NoneType'"
        );

        let vm = vm_with(vec![OpCode::TYPE_TO_F64 as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_nv(auto_val::encode_null());
        let msg = runtime_err_of(vm.run_one_instruction(&mut task));
        assert_eq!(
            msg,
            "TypeError: float() argument must be a string or a real number, not 'NoneType'"
        );
    }

    #[test]
    fn test_to_i32_f64_legit_paths_unaffected() {
        let vm = vm_with(vec![OpCode::TYPE_TO_I32 as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_i32(7);
        vm.run_one_instruction(&mut task).unwrap();
        assert_eq!(task.ram.pop_i32(), 7);

        let vm = vm_with(vec![OpCode::TYPE_TO_F64 as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_f64(2.5);
        vm.run_one_instruction(&mut task).unwrap();
        assert_eq!(task.ram.pop_f64(), 2.5);
    }

    // ---- T07 TYPE_TO_STR null → "None" ----

    #[test]
    fn test_null_to_str_renders_none() {
        let vm = vm_with(vec![OpCode::TYPE_TO_STR as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        task.ram.push_nv(auto_val::encode_null());
        vm.run_one_instruction(&mut task).unwrap();
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv), "expected string result");
        let idx = auto_val::decode_string(nv) as usize;
        let rendered = vm.strings.read().unwrap().get(idx).cloned().unwrap();
        assert_eq!(rendered, b"None".to_vec());
    }
}

mod tests_add_concat_rc {
    use super::*;
    use crate::vm::task::AutoTask;
    use crate::vm::virt_memory::VirtualFlash;

    fn vm_with(code: Vec<u8>) -> AutoVM {
        AutoVM::new(VirtualFlash::new_with_code(code), 1024)
    }

    fn pop_str(vm: &AutoVM, task: &mut AutoTask) -> String {
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv), "expected string on stack");
        let idx = auto_val::decode_string(nv) as usize;
        String::from_utf8(
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .unwrap_or_default(),
        )
        .unwrap()
    }

    /// Plan 567 T02（P539-D1）: rc=1 操作数 ADD 拼接——先读后放纪律回归。
    /// 病灶形态：release 在读之前 → 中间结果槽 FREE+tombstone 后读归空
    /// （"got d"）；add_string 前的旧第二对 release → rc 下溢回绕 u32::MAX
    /// （P053-8 幻影 freelist 条目）。
    #[test]
    fn test_add_string_concat_rc1_operands() {
        let vm = vm_with(vec![OpCode::ADD as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        let a = vm.add_string(b"a".to_vec());
        let b = vm.add_string(b"b".to_vec());
        vm.rc_push_str_idx(&mut task, a);
        vm.rc_push_str_idx(&mut task, b);
        vm.run_one_instruction(&mut task).unwrap();
        assert_eq!(pop_str(&vm, &mut task), "ab");
        let h = vm.pool_health();
        assert_eq!(h.underflow_events, 0, "no double release (rc underflow)");
        assert_eq!(h.phantom_drops, 0, "no phantom freelist entry");
    }

    /// 左结合链的中间结果必须可读：((a+b)+c) == "abc"——修复前第二跳
    /// 读到已释放的 "ab" 槽（tombstone）归空串，得 "c"（py_list
    /// test_sorted_getitem "got d" 的引擎级最小复现）。
    #[test]
    fn test_add_string_concat_chain_intermediate_survives() {
        let vm = vm_with(vec![OpCode::ADD as u8, OpCode::ADD as u8]);
        let mut task = AutoTask::new(1, 256, 0);
        let a = vm.add_string(b"a".to_vec());
        let b = vm.add_string(b"b".to_vec());
        let c = vm.add_string(b"c".to_vec());
        vm.rc_push_str_idx(&mut task, a);
        vm.rc_push_str_idx(&mut task, b);
        vm.run_one_instruction(&mut task).unwrap();
        vm.rc_push_str_idx(&mut task, c);
        vm.run_one_instruction(&mut task).unwrap();
        assert_eq!(pop_str(&vm, &mut task), "abc");
        let h = vm.pool_health();
        assert_eq!(h.underflow_events, 0);
        assert_eq!(h.phantom_drops, 0);
    }
}

mod tests_p583_pool_child_shares {
    use super::*;
    use crate::vm::ffi::VMConvertible;
    use crate::vm::task::AutoTask;
    use crate::vm::virt_memory::VirtualFlash;

    fn vm_idle() -> AutoVM {
        AutoVM::new(VirtualFlash::new_with_code(vec![OpCode::RET as u8]), 1024)
    }

    /// Plan 583 T4: Vec<String> 入栈必须为每个负哨兵元素建立容器子份额（+1）。
    /// 修复前 add_string 后裸嵌哨兵：元素 rc=0，首个 GET_ELEM 的 +1 被槽位
    /// 释放抵消归零 → FREE 清内容 → 列表悬垂（Plan 579 T9 真实语料扫描
    /// 100% tombstone/静默空串的根因）。
    #[test]
    fn test_vec_string_push_child_shares() {
        let vm = vm_idle();
        let mut task = AutoTask::new(1, 256, 0);
        let payload: Vec<String> = vec!["alpha".to_string(), "beta".to_string()];
        payload.push_to_stack(&mut task, &vm).unwrap();

        // 栈顶是 list id（带影子份额）；列表内每个元素各持 1 份池份额。
        let list_nv = task.ram.pop_nv();
        assert!(
            crate::vm::rc::heap_ref_id(list_nv).is_some(),
            "list id on stack"
        );
        let list_id = crate::vm::rc::heap_ref_id(list_nv).unwrap();
        let obj = vm.get_heap_object(list_id).unwrap();
        let sentinels: Vec<i32> = {
            let g = obj.read().unwrap();
            g.as_any()
                .downcast_ref::<crate::vm::types::ListData<i32>>()
                .unwrap()
                .elems
                .clone()
        };
        assert_eq!(sentinels.len(), 2);
        for &sent in &sentinels {
            assert!(sent < 0, "string sentinel encoding");
            let idx = (-(sent) - 1) as usize;
            assert_eq!(vm.pool_count(idx), 1, "container child share established");
        }

        // 释放列表（rc 归零 + 宽限窗收割）→ 子份额随死亡释放归零、墓碑化。
        vm.rc_release(list_nv);
        vm.reap_all();
        for &sent in &sentinels {
            let idx = (-(sent) - 1) as usize;
            assert_eq!(
                vm.pool_count(idx),
                0,
                "child share released with container death"
            );
            assert!(vm.pool_is_tombstone(idx), "tombstoned after release");
        }
        let h = vm.pool_health();
        assert_eq!(
            h.underflow_events, 0,
            "no over-release (paired retain/release)"
        );
        assert_eq!(h.phantom_drops, 0);
    }

    /// Plan 583 T4: 复活加固——墓碑槽收到 retain（陈旧拷贝症状）时必须
    /// 摘 freelist + 清墓碑，杜绝"存活且在复用队列"的幻影条目
    /// （P-053-8 级联腐化源）。
    #[test]
    fn test_pool_retain_resurrection_unfreelists() {
        let vm = vm_idle();
        let idx = vm.add_string(b"ghost".to_vec());
        vm.pool_retain(idx);
        assert_eq!(vm.pool_count(idx), 1);
        vm.pool_release(idx); // 归零 → 墓碑 + freelist
        assert!(vm.pool_is_tombstone(idx));
        assert!(vm.pool_state.read().unwrap().freelist.contains(&idx));

        vm.pool_retain(idx); // 陈旧拷贝到达 → 复活加固
        assert!(
            !vm.pool_is_tombstone(idx),
            "tombstone cleared on resurrection"
        );
        assert!(
            !vm.pool_state.read().unwrap().freelist.contains(&idx),
            "unfreelisted"
        );
        assert_eq!(vm.pool_count(idx), 1);
        let h = vm.pool_health();
        assert_eq!(h.underflow_events, 0);
        assert_eq!(h.phantom_drops, 0);
    }
}

mod tests_err_value_channel {
    use super::*;
    use crate::vm::generic_registry::GenericInstanceData;
    use crate::vm::task::AutoTask;
    use crate::vm::virt_memory::VirtualFlash;

    fn vm_with(code: Vec<u8>) -> AutoVM {
        AutoVM::new(VirtualFlash::new_with_code(code), 1024)
    }

    fn push_result_instance(vm: &AutoVM, task: &mut AutoTask, kind: &str, field: auto_val::Value) {
        let inst = GenericInstanceData::new(kind.to_string(), vec![field]);
        let id = vm.insert_heap_object(inst);
        vm.rc_push(task, auto_val::encode_object(id as u32));
    }

    /// Plan 567 T08（P560-D2）: Err 值传播被当前帧 try handler 拦截——
    /// 改跳 catch_pc + 载荷串入栈（与 intercept_error 异常通道汇合）。
    /// 布局：PUSH_HANDLER(rel=2) → ERROR_PROPAGATE(n=0) → catch: CONST_I32 77。
    #[test]
    fn test_error_propagate_value_channel_catch_hit() {
        let mut code = Vec::new();
        code.push(OpCode::PUSH_HANDLER as u8);
        code.extend_from_slice(&2i16.to_le_bytes());
        code.push(OpCode::ERROR_PROPAGATE as u8);
        code.push(0u8); // n_args
        code.push(OpCode::CONST_I32 as u8);
        code.extend_from_slice(&77i32.to_le_bytes());

        let vm = vm_with(code);
        let mut task = AutoTask::new(1, 256, 0);
        vm.run_one_instruction(&mut task).unwrap(); // PUSH_HANDLER
        assert_eq!(task.handler_stack.len(), 1);

        push_result_instance(
            &vm,
            &mut task,
            "Result.Err",
            auto_val::Value::Str("PyException AttributeError: boom".to_string().into()),
        );
        vm.run_one_instruction(&mut task).unwrap(); // ERROR_PROPAGATE → intercept

        assert!(task.handler_stack.is_empty(), "handler consumed by catch");
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv), "payload bound for catch");
        let idx = auto_val::decode_string(nv) as usize;
        let payload = vm.strings.read().unwrap().get(idx).cloned().unwrap();
        assert_eq!(payload, b"PyException AttributeError: boom".to_vec());
        assert_eq!(
            task.ip, 5,
            "ip redirected to catch_pc (right after ERROR_PROPAGATE)"
        );

        vm.run_one_instruction(&mut task).unwrap(); // catch body
        assert_eq!(task.ram.pop_i32(), 77);
        let h = vm.pool_health();
        assert_eq!(h.underflow_events, 0);
        assert_eq!(h.phantom_drops, 0);
    }

    /// null 是值不进 catch：None 传播走原 May 帧展开路径（main 帧 = Terminate），
    /// handler 不被消费。
    #[test]
    fn test_error_propagate_null_not_intercepted() {
        let mut code = Vec::new();
        code.push(OpCode::PUSH_HANDLER as u8);
        code.extend_from_slice(&2i16.to_le_bytes());
        code.push(OpCode::ERROR_PROPAGATE as u8);
        code.push(0u8);

        let vm = vm_with(code);
        let mut task = AutoTask::new(1, 256, 0);
        vm.run_one_instruction(&mut task).unwrap(); // PUSH_HANDLER
        task.ram.push_nv(auto_val::encode_null());
        let step = vm.run_one_instruction(&mut task).unwrap();
        assert!(
            matches!(step, StepResult::Terminated),
            "null propagation in main frame terminates, got {:?}",
            step
        );
        assert_eq!(
            task.handler_stack.len(),
            1,
            "handler untouched by null path"
        );
    }

    /// Result.Ok 解包继续执行——不进 catch、handler 保持。
    #[test]
    fn test_error_propagate_ok_unwraps_no_interception() {
        let mut code = Vec::new();
        code.push(OpCode::PUSH_HANDLER as u8);
        code.extend_from_slice(&2i16.to_le_bytes());
        code.push(OpCode::ERROR_PROPAGATE as u8);
        code.push(0u8);

        let vm = vm_with(code);
        let mut task = AutoTask::new(1, 256, 0);
        vm.run_one_instruction(&mut task).unwrap(); // PUSH_HANDLER
        push_result_instance(&vm, &mut task, "Result.Ok", auto_val::Value::Int(42));
        vm.run_one_instruction(&mut task).unwrap(); // ERROR_PROPAGATE → unwrap
        assert_eq!(task.ram.pop_i32(), 42);
        assert_eq!(task.handler_stack.len(), 1, "handler untouched by Ok path");
    }
}

// ---- PLAN-843 结构拆分：语法族主题模块（原单文件热点，纯搬移） ----
mod futures_timers;
mod heap;
mod ops_arith;
mod ops_calls;
mod ops_collections;
mod ops_construct;
mod ops_flow;
mod tasks;
mod values;
