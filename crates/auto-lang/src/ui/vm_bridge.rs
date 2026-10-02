//! # VmBridge - Bridge between AutoVM and the UI system
//!
//! This module provides [`VmBridge`], which links widget handlers compiled by the
//! genuine VM `Codegen` (the same compiler the non-UI `run()` path uses) to the
//! UI rendering backend.
//!
//! ## Architecture (Plan 323 / Option B)
//!
//! ```text
//! AuraWidget (extracted from .at source)
//!    |
//! |  handler_codegen::synthesize_widget_module
//! |  (imports + type AppState + handler fns -> ONE Codegen pass -> Module)
//!    v
//! VmBridge
//!  - Links the module into a VirtualFlash
//!  - Stores widget state as GenericInstanceData on the VM heap
//!  - Dispatches handlers via the segment driver (PLAN-702
//!    `call_fn_by_name_segment`: Yield+wait parks the task into
//!    `parked_tasks` and returns immediately — the iced loop stays
//!    interactive; the `__parked_resume_tick` pump resumes ready segments.
//!    The busy-waiting `call_fn_by_name` is legacy, non-UI callers only)
//!    |
//!    v
//! UI Backend (iced, headless) reads state via read_state()
//! and triggers handlers via call_handler()
//! ```
//!
//! Each handler is a real VM function `fn handler_<Name>(__state AppState, ...)`.
//! State references (`.field`) are AST-rewritten to `__state.field` by
//! `handler_codegen`, which Codegen lowers to `LOAD_LOCAL + GET_FIELD/SET_FIELD`
//! against the state heap object. `call_handler` pushes the state heap id as the
//! first argument and dispatches via the segment driver (`call_fn_by_name_segment`).
//!
//! This replaces the bespoke mini-compiler + AST tree-walker that stalled during
//! the Plan 205 migration: handlers can now use the full language (loops, arrays,
//! objects, cross-module `CALL` like `build_month_grid`).

use std::collections::HashMap;

use crate::ast::Stmt;
use crate::vm::engine::{AutoVM, ParkedSegment, ParkedWait, SegmentOutcome};
use crate::vm::generic_registry::GenericInstanceData;
use crate::vm::loader::Linker;
use crate::vm::task::AutoTask;
use crate::vm::virt_memory::VirtualFlash;
#[allow(unused_imports)] // AuraStateDef/AuraNode used in #[cfg(test)] below
use crate::aura::{AuraWidget, AuraStateDef, AuraNode};
use crate::ast::Expr;
use auto_val::{Op, Value};

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during VmBridge operations.
///
/// UI should never crash from VM errors - all errors are graceful.
#[derive(Debug)]
pub enum VmBridgeError {
    /// Field not found in widget state
    FieldNotFound(String),
    /// Handler not found for the given event name
    HandlerNotFound(String),
    /// VM execution error
    VmError(String),
    /// Invalid state (e.g., corrupt heap object)
    InvalidState(String),
    /// PLAN-711 T-11 (D-2): CPU continuation 存活期的同 App VM 写事件队列
    /// 已满（上限 128）——拒绝入队并给可观察 busy，不无限增容或静默丢动作。
    WriteQueueFull { len: usize },
}

impl std::fmt::Display for VmBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmBridgeError::FieldNotFound(name) => {
                write!(f, "field not found: {}", name)
            }
            VmBridgeError::HandlerNotFound(name) => {
                write!(f, "handler not found: {}", name)
            }
            VmBridgeError::VmError(msg) => {
                write!(f, "VM error: {}", msg)
            }
            VmBridgeError::InvalidState(msg) => {
                write!(f, "invalid state: {}", msg)
            }
            VmBridgeError::WriteQueueFull { len } => {
                write!(f, "vm write queue full ({len}) — handler busy")
            }
        }
    }
}

impl std::error::Error for VmBridgeError {}

pub type Result<T> = std::result::Result<T, VmBridgeError>;

// ============================================================================
// VmBridge
// ============================================================================

/// Bridge between AutoVM and the UI system.
///
/// Holds a VM instance with widget state and handler bytecode.
/// Each widget gets its own VmBridge with an isolated VM.
///
/// # Lifecycle
///
/// 1. `VmBridge::new(widget)` - Create bridge, synthesize handlers, init state
/// 2. `bridge.read_state("count")` - Read state field values for rendering
/// 3. `bridge.call_handler("Inc", &[])` - Execute handler on user interaction
/// 4. `bridge.read_state(...)` - Read updated state for re-rendering
/// Plan 488：宿主注入面的事件载荷（递归记录编码入 VM 堆——见
/// [`VmBridge::call_handler_with_record`]）。
#[derive(Debug, Clone, PartialEq)]
pub enum RecordValue {
    Str(String),
    Int(i32),
    Bool(bool),
    StrList(Vec<String>),
    Record(Vec<(String, RecordValue)>),
    Null,
}

pub struct VmBridge {
    /// AutoVM instance (owned, isolated per widget)
    vm: AutoVM,

    /// Widget state as a VM heap object ID.
    /// The heap object is a `GenericInstanceData` with field names and values.
    state_obj_id: u64,

    /// State field names (ordered, matching GenericInstanceData field order)
    state_field_names: Vec<String>,

    /// Widget name for debugging
    widget_name: String,

    /// Plan 320: child widget state heap object IDs, keyed by widget name.
    /// Each child widget instance gets its own GenericInstanceData on the same
    /// VM heap (single VM, multiple state objects). Uses RefCell for interior
    /// mutability so AuraViewBuilder (which holds &VmBridge) can create/update
    /// child states during rendering.
    child_state_map: std::cell::RefCell<std::collections::HashMap<String, u64>>,

    /// Audit B12(b): declared parameter count per `(widget, event)` handler,
    /// from the on-blocks the module was synthesized from. Dispatch sites use
    /// this to avoid pushing a phantom string argument at a NO-param handler
    /// (mcp submit/type pass the input text as `input_value`; the extra arg
    /// shifted the handler frame — `.todos` writes landed on a garbage object
    /// id in 013-todo's AddTodo).
    /// Audit B12(b) + PLAN-685 G1: declared handler parameter NAMES per
    /// `(widget, event)` handler (arity = vec len; `handler_param_count`
    /// keeps the B12(b) contract). PLAN-685: dispatch-side stripped in-body
    /// callback snapshot evaluation (`eval_stripped_arg`) resolves bare
    /// identifiers against these names paired with the actual dispatch args
    /// — the "param name as string literal" degradation is forbidden.
    handler_param_names: std::collections::HashMap<(String, String), Vec<String>>,

    /// PLAN-062 F2: 帧域 retain 账本（双缓冲）。cur 收本脏帧经
    /// `retain_heap_result` 拿下的宿主份额；`commit_dirty_frame` 在下一
    /// 脏帧"新缓存写回之后"swap 并整体释放 prev——宿主份额由此获得确定
    /// 生存期（KD-051 ⑤"retain 未配对释放"的配平根修）。缓存命中帧不跑
    /// builder、零 retain、不触碰账本。`AUTOUI_FRAME_RC=0` 关闭（退回
    /// v1 永持语义）。
    frame_retains_cur: std::sync::Mutex<Vec<u64>>,
    frame_retains_prev: std::sync::Mutex<Vec<u64>>,

    /// PLAN-051 C3: use.web 引入的 fn 别名 → 模块限定名映射（装载器按文件
    /// stem 限定，如 chatActivePath → forge_helpers.chatActivePath）。computed
    /// 求值面的裸 fn 调用经 [`VmBridge::call_vm_fn`] 按此解析真实导出名。
    import_aliases: std::collections::HashMap<String, String>,

    /// PLAN-642 T-15: 本组件合成期的视图侧 store 别名快照（alias→真名）。
    /// 单组件形态曾依赖 handler_codegen 的线程级 VIEW_STORE_ALIAS_SNAPSHOT
    /// （合成后残留供同线程视图求值）；多组件工程（画廊合并 VM 轨）下该
    /// 快照被后续组件的合成覆盖——`.TodoStore.X` 一类真名限定读在视图层
    /// 解析失败短路（015 "No notes yet"、013 footer 字面模板实锤）。视图
    /// 求值改为查自身 bridge 的存档（[`VmBridge::store_alias_real_name`]）。
    store_alias_snapshot: std::collections::HashMap<String, String>,

    /// PLAN-702 T-02: parked handler 段注册表——handler 派发遇 Yield+等待
    /// （异步 HTTP / 外部 future）时，执行中的 task（ip/bp/栈/闭包态）连
    /// 同引擎恢复上下文登记在此，等恢复泵（`__parked_resume_tick`）在其
    /// wait 就绪后续跑。单 VM 串行：续跑只在 iced update 内发生，同一时刻
    /// 至多一个连续段在执行（架构裁断 1）；同 (widget,handler) 键的重复
    /// 触发在派发前被 `is_handler_parked` 忽略（T-04 重入策略）。
    /// RefCell 与 child_state_map 同款内可变性理由（call_handler_for 是
    /// `&self`）。
    parked_tasks: std::cell::RefCell<Vec<ParkedTask>>,

    /// PLAN-711 T-11 (D-2): 同 App VM 写事件有界串行队列——CPU continuation
    /// 存活期间，input 代写/timer/props 播种/MCP fixture/reload 等经此排队，
    /// 由 `resume_cpu_slices` 在片间/终态后消费，避免与在途 continuation
    /// 交错写共享堆。上限 128（`VM_WRITE_QUEUE_CAP`）：满队列拒绝入队并给
    /// 可观察 busy（`VmBridgeError::WriteQueueFull`），只合并明确可覆盖的
    /// 输入值，不合并点击等副作用事件。宿主滚动/resize/close 不入此队列。
    /// 生产者接线归 T-04（update_inner 各臂）；本任务交付队列原语+泵消费。
    cpu_write_queue: std::cell::RefCell<std::collections::VecDeque<QueuedVmWrite>>,

    /// PLAN-711 T-03 (M-01): Init demand FIFO 队列——渲染路径登记、派发
    /// 驱动消费（`dispatch_pending_inits`）。同代际重复登记不重复入队。
    init_demand_queue: std::cell::RefCell<std::collections::VecDeque<InitDemand>>,

    /// PLAN-711 T-03 (M-01): 每 widget 的 demand 簿记（最近身份/相位/代际）
    /// ——PLAN-536 `child_last_init_identity`"判定即写身份"的替代面：
    /// 登记≠完成，相位由派发驱动真实记账。
    init_demand_records: std::cell::RefCell<std::collections::HashMap<String, InitDemandRecord>>,

    /// PLAN-711 T-03 (M-01): 挂载代际计数器——每次身份变化 +1；A→B→A 的
    /// 第二个 A 携带新代际号（708 设计 §4）。
    init_generation: std::cell::Cell<u64>,



    /// PLAN-702 T-04: `__busy_handlers` 镜像的已写字集——parked 键集无变化
    /// 时跳过堆列表重铸（每 tick 调 sync_busy_flag，稳态零写）。
    busy_flag_names: std::cell::RefCell<Vec<String>>,

    /// PLAN-045: 组件级 memo 缓存宿主——builder 每帧借用临时、桥跨帧持久，
    /// 缓存生命周期随桥（hot-reload 走新建桥 reload，缓存自然弃置）。
    /// RefCell 内可变理由同 parked_tasks（渲染期 `&self`）。类型面依赖
    /// interpreter，同 ui-interpreter 门控。
    #[cfg(feature = "ui-interpreter")]
    memo_cache: std::cell::RefCell<crate::ui::memo_deps::MemoCache>,

    /// PLAN-047 T-01（档 C SD-08）: 依赖录制器宿主——memo 门/computed 信号
    /// 求值期经 [`VmBridge::dep_recording_guard`] 激活，桥读通道把状态读记
    /// 入当前 [`crate::ui::memo_deps::RecState`]；未激活 = `None`，读通道
    /// 零开销直落（非 memo 零行为零开销红线）。RefCell 理由同 memo_cache。
    #[cfg(feature = "ui-interpreter")]
    dep_recorder: std::cell::RefCell<Option<crate::ui::memo_deps::RecState>>,

    /// PLAN-047 T-05: 求值通道探针计数（AC-02 零重解析断言的观测面——
    /// view builder `resolve_expr_to_value` 入口累加，桥级共享、跨帧累计）。
    pub resolve_probe_count: std::sync::atomic::AtomicU64,

    /// PLAN-047 T-06（档 C SD-10）: computed 信号表宿主——(widget, prop)
    /// 键控的信号节点（值缓存 + 动态 dep 基线对）。生命周期随桥（hot-reload
    /// 新建桥自然弃置）。RefCell 理由同 memo_cache（渲染期 `&self`）。
    #[cfg(feature = "ui-interpreter")]
    computed_signals:
        std::cell::RefCell<HashMap<(String, String), ComputedSignal>>,

    /// PLAN-047 T-06: 信号网命中/未命中计数（观测面——AC-04 断言用）。
    pub signal_hits: std::sync::atomic::AtomicU64,
    pub signal_misses: std::sync::atomic::AtomicU64,
}

/// PLAN-047 T-01: 依赖录制 guard。首选显式 [`DepRecGuard::finish`] 取走
/// 本次录制集；未 finish 即 drop（`?` 早退/panic 展开）走同一恢复语义——
/// 外层恢复与并集吸收不因退出路径缺失（正确性面不允许静默吞外层）。
#[cfg(feature = "ui-interpreter")]
pub struct DepRecGuard<'a> {
    bridge: &'a VmBridge,
    outer: Option<Box<crate::ui::memo_deps::RecState>>,
    done: bool,
}

#[cfg(feature = "ui-interpreter")]
impl<'a> DepRecGuard<'a> {
    /// 结束录制：取走本次 [`crate::ui::memo_deps::RecState`]（含 overflow
    /// 旗标），恢复外层并集吸收。
    pub fn finish(mut self) -> crate::ui::memo_deps::RecState {
        self.done = true;
        self.take_and_restore()
    }

    fn take_and_restore(&mut self) -> crate::ui::memo_deps::RecState {
        let mut cur = self
            .bridge
            .dep_recorder
            .borrow_mut()
            .take()
            .unwrap_or_default();
        // PLAN-047 T-04: 收编引擎影子集（VM fn 执行期读臂录制面）——并集
        // 记账沿 record 语义（预算/overflow 同规）。
        let shadow = self.bridge.vm.take_dep_recorder();
        for k in shadow.deps {
            cur.record(k);
        }
        cur.overflow |= shadow.overflow;
        if let Some(mut outer) = self.outer.take() {
            outer.absorb(&cur);
            *self.bridge.dep_recorder.borrow_mut() = Some(*outer);
        }
        cur
    }
}

#[cfg(feature = "ui-interpreter")]
impl<'a> Drop for DepRecGuard<'a> {
    fn drop(&mut self) {
        if !self.done {
            self.take_and_restore();
        }
    }
}

/// PLAN-047 T-06（档 C SD-10）: computed 信号节点——inline 表达式与 block
/// 体隐藏 VM fn 双通道共载。`deps` = 最近一次求值的动态依赖基线对；版本
/// 全同 → 值缓存复用。脏传播 pull 式：写点定点 bump → 版本比对 miss →
/// 重求值（不建主动订阅图——漏传播只落重算，不陈旧）。
#[cfg(feature = "ui-interpreter")]
pub struct ComputedSignal {
    pub cached: Value,
    pub deps: Vec<(crate::ui::memo_deps::DepKey, u64)>,
}

/// PLAN-706 r2 T-10: 值是否携带堆身份（信号网准入检查——携带者不入网）。
/// 直接载体 = `VmRef` / ≥4M 整数约定形（`record_dep_value_heap` 同阈值）/
/// `ValueRef`；容器递归（Array/Obj/Pair/Some/Ok）；不可证净的复合变体
/// （Node/Widget/View 等内嵌 prop 值）保守按携带处理。标量/字符串纯值
/// （AutoStr 为自有拷贝，非池索引）放行。
#[cfg(feature = "ui-interpreter")]
fn value_carries_heap_identity(v: &Value) -> bool {
    match v {
        Value::VmRef(_) | Value::ValueRef(_) => true,
        Value::Int(i) => *i >= 4_000_000,
        Value::Uint(u) => *u as u64 >= 4_000_000,
        Value::USize(s) => *s as u64 >= 4_000_000,
        Value::I64(l) => *l >= 4_000_000,
        Value::Array(a) | Value::Block(a) => a.values.iter().any(value_carries_heap_identity),
        Value::Obj(o) => o.iter().any(|(_, v)| value_carries_heap_identity(v)),
        Value::Pair(_, inner) => value_carries_heap_identity(inner),
        Value::Some(inner) | Value::Ok(inner) => value_carries_heap_identity(inner),
        // 复合载体不可证净（内嵌 prop/字段值可能含堆身份）——保守不入网。
        Value::Node(_)
        | Value::Widget(_)
        | Value::Model(_)
        | Value::View(_)
        | Value::Meta(_)
        | Value::Method(_)
        | Value::Instance(_)
        | Value::Args(_)
        | Value::Grid(_)
        | Value::Closure(_)
        | Value::Future(_)
        | Value::Fn(_)
        | Value::ExtFn(_)
        | Value::Type(_) => true,
        _ => false,
    }
}

/// PLAN-702 T-04: root-state busy 镜像字段名——List&lt;str&gt;（namespaced
/// handler fn 名集），.at 可查询（如 `.store.__busy_handlers.len() > 0`）。
const BUSY_STATE_FIELD: &str = "__busy_handlers";

/// PLAN-702 T-03: outcome of one resume-pump pass.
#[derive(Default)]
pub struct ResumeReport {
    /// Segments that ran to completion this pass (model writes landed).
    pub completed: usize,
    /// Segments whose resumed execution failed uncaught: (fn_name, error).
    pub failed: Vec<(String, String)>,
}

/// PLAN-711 T-11 (D-2): CPU slice 有界泵一轮的报告（测试/诊断/AC-11 计数）。
#[derive(Default)]
pub struct CpuPumpReport {
    /// 本轮实际驱动的 CPU 片数（含排队写事件的首片）。
    pub slices_run: usize,
    /// 本轮跑到终态的 continuation 数。
    pub completed: usize,
    /// 本轮续跑失败：(fn_name, error)。
    pub failed: Vec<(String, String)>,
    /// 本轮消费的排队写事件数。
    pub queue_drained: usize,
    /// 本轮墙钟耗时（对照 `CPU_PUMP_ROUND_BUDGET`）。
    pub round_elapsed: std::time::Duration,
}

/// PLAN-711 T-11 (D-2): 排队的同 App VM 写事件——一个待派发 handler
/// （input 代写/timer/props 播种/MCP fixture/reload 的统一落点）。
#[derive(Debug, Clone)]
pub struct QueuedVmWrite {
    pub widget_name: String,
    pub event_name: String,
    pub state_obj_id: u64,
    pub args: Vec<Value>,
    /// 该事件是否可与队尾同键事件合并（明确可覆盖的输入值，如滚动位置/
    /// 文本草稿；点击等副作用事件必须 false）。
    pub overwritable: bool,
}

/// PLAN-711 T-11 (D-2): 队列上限——满即拒，可观察 busy，不无限增容。
pub const VM_WRITE_QUEUE_CAP: usize = 128;

/// PLAN-711 T-03 (M-01): 一条待派发的 Init demand——view 渲染路径只
/// 登记（判定≠完成），真实派发由 [`VmBridge::dispatch_pending_inits`] 驱动。
#[derive(Debug, Clone)]
pub struct InitDemand {
    pub widget_name: String,
    /// 挂载身份（组件名 + 调用位 `key:` prop，PLAN-536/os-016 语义原样）。
    pub identity: String,
    pub state_obj_id: u64,
    /// 登记时的代际号（每次身份变化前进；取消判定与诊断用）。
    pub generation: u64,
    /// PLAN-711 T-03: 登记时的 props 快照（resolve 后值）。**统一根态约束**：
    /// 子件 props 以同名共享字段形态落在根态（ensure_child_state 返回
    /// root_id），后渲染组件的播种会覆盖前者——旧同步路径"播种后立刻派发"
    /// 是该架构下的正确性要求。延迟派发必须携带快照、派发前重播种，等价
    /// 复刻旧交错语义（donut 除零实录：慢派发读到后写者/空 data）。
    pub props: Vec<(String, auto_val::Value)>,
}

/// PLAN-711 T-03 (M-01): demand 生命周期相位。
/// ```text
/// Registered → Queued（FIFO 等待派发）→ InFlight（段已 park）
///            → Done / Failed（终态；Cancelled 由身份变化触发）
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitDemandPhase {
    /// 已登记排队，未派发。
    Queued,
    /// 段已派发且 park（CPU/IO 凭据在 parked 注册表）。
    InFlight,
    /// 真正完成（Completed 或 Missing 静默）。
    Done,
    /// 派发/续跑失败（错误已走 `[VM-HANDLER]` 通道；不无限重试）。
    Failed,
}

/// PLAN-711 T-03: 每 (widget) 的 demand 状态簿记——`child_init_should_fire`
/// 的"判定即写身份"重排为"登记≠完成"：身份表记录**最近登记**的身份与
/// 相位，完成与否由派发驱动真实记账。
#[derive(Debug, Clone)]
struct InitDemandRecord {
    identity: String,
    phase: InitDemandPhase,
    generation: u64,
}

/// PLAN-711 T-03: [`VmBridge::register_init_demand`] 的判定结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitDemandDecision {
    /// 新 demand 入队（首次登记或身份变化=新代际）。
    Queued,
    /// 同代际重复登记（重复 view/MCP 构建）——只确认簿记，不二次入队。
    AlreadyKnown,
    /// 前序 demand 仍在途（未完成）——同代际不重复派发。
    InFlightKnown,
}

/// PLAN-711 T-03: 一轮 Init demand 派发驱动的报告。
#[derive(Default)]
pub struct InitDispatchReport {
    /// 本轮真实派发的 demand 数。
    pub dispatched: usize,
    /// Missing（未声明导出的 Init）——静默记 Done。
    pub missing: usize,
    /// 派发即失败（VM error）。
    pub failed: Vec<(String, String)>,
    /// 同步跑完（无等待短 Init）。
    pub completed: usize,
    /// PLAN-712 r2 T-16：InFlight→Done / Missing 的**静默终态翻转**数——
    /// 翻转本身不派发不跑段，但要求一轮重渲染（outlet 的 Loading… 占位
    /// 依赖它清场；018 书架实机实证）。
    pub silent_done_transitions: usize,
    /// park 在途（CPU/IO），本轮停止派发后继 demand（依赖序）。
    pub in_flight: usize,
}

/// PLAN-702 T-02: one parked handler segment.
pub struct ParkedTask {
    /// Mid-execution task — ip/bp/ram/call_stack/闭包上下文原样保留，恢复
    /// 时交还引擎续跑。栈上的 rc 份额随 task 存活（不得中途
    /// rc_release_task_stack——完成/失败时按来源纪律清账）。
    pub task: AutoTask,
    /// 引擎恢复上下文（saved_bp/saved_fn_n_args/fn_name，跨段不变）。
    pub seg: ParkedSegment,
    /// 就绪凭据（HTTP req_id / future id），恢复泵按此探测。
    pub wait: ParkedWait,
    /// 派发时解析出的 namespaced fn 名（重入键；编码了 widget+handler）。
    pub fn_name: String,
    /// 事件显示名（日志/诊断用，如 "PickFolder"）。
    pub event_name: String,
    /// 完成时是否清整任务栈（call_handler_for 来源=true，与其同步路径的
    /// rc_release_task_stack 纪律对齐；call_handler/with_record 来源=false，
    /// 保持其既有弃栈行为不变）。
    pub release_stack_on_complete: bool,
    pub parked_at: std::time::Instant,
}

/// PLAN-051 C3: 栈顶 nanbox → Value（call_vm_fn 返回值解码；与
/// vm/native.rs nv_to_value 同构——该函数私有，此处本地副本）。字符串以
/// 池索引形态返回 Int（调用方按需另行解码；列表/对象为 id/VmRef 原样）。
fn nv_to_pub_value(nv: auto_val::NanoValue) -> Value {
    if auto_val::is_i32(nv) {
        Value::Int(auto_val::decode_i32(nv))
    } else if auto_val::is_i64(nv) {
        // PLAN-701 T-06 增量：TAG_I64 返回值此前落入末位 decode_i32 兜底
        // → 桥面垃圾值（epoch 毫秒探针 -699152978 实证）。Value::I64 变体
        // 在档——补臂（Plan 522 T4 f64 臂 / P-053-6 字符串臂同族桥面缺口）。
        Value::I64(auto_val::decode_i64(nv))
    } else if auto_val::is_f64(nv) {
        // Plan 522 T4: f64 返回值此前落入末位 i32 解码 → Int(0) 桥面丢失
        // (call_vm_fn 的 float helper 返回值全 0;引擎内部 decode_tagged_nv
        // 一直正确,仅桥返回路径缺浮点臂)。
        Value::Double(auto_val::decode_f64(nv))
    } else if auto_val::is_f32(nv) {
        Value::Float(auto_val::decode_f32(nv) as f64)
    } else if auto_val::is_list(nv) {
        Value::Int(auto_val::decode_list(nv) as i32)
    } else if auto_val::is_object(nv) {
        Value::VmRef(auto_val::VmRef { id: auto_val::decode_object(nv) as usize })
    } else if auto_val::is_string(nv) {
        Value::Int(auto_val::decode_string(nv) as i32)
    } else if auto_val::is_null(nv) {
        Value::Nil
    } else if auto_val::is_bool(nv) {
        Value::Bool(auto_val::decode_bool(nv))
    } else {
        Value::Int(auto_val::decode_i32(nv))
    }
}

/// Audit B12(b): collect `(widget, event) -> declared param names` from an
/// AuraWidget's handler_params map (patterns like `.Inc` / `SelectNote`).
fn collect_param_names_from_widget(
    widget: &AuraWidget,
    out: &mut std::collections::HashMap<(String, String), Vec<String>>,
) {
    for (pattern, params) in &widget.handler_params {
        let ev = pattern.trim_start_matches('.');
        out.insert((widget.name.clone(), ev.to_string()), params.clone());
    }
}

/// Audit B12(b): same collection from a WidgetDecl's on-block (decl-based
/// synthesis path — `new_from_decls`).
fn collect_param_names_from_decl(
    decl: &crate::ast::ui::WidgetDecl,
    out: &mut std::collections::HashMap<(String, String), Vec<String>>,
) {
    for on in &decl.on {
        for h in &on.handlers {
            let ev = h.pattern.trim_start_matches('.');
            out.insert((decl.name.to_string(), ev.to_string()), h.params.clone());
        }
    }
}

/// PLAN-062 F2: 帧账本总开关——默认开；`AUTOUI_FRAME_RC=0` 逃生门
/// （退回 v1 永持语义，泄漏复现/对照用）。进程级 OnceLock，热路径零
/// env 查询。
fn frame_rc_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("AUTOUI_FRAME_RC").map(|v| v != "0").unwrap_or(true)
    })
}

impl VmBridge {
    /// Audit B12(b): declared parameter count per `(widget, event)` handler.
    /// Dispatch sites use this to avoid pushing a phantom string argument at
    /// a NO-param handler (mcp submit/type pass the input text as
    /// `input_value`; the extra arg shifted the handler frame — `.todos`
    /// writes landed on a garbage object id in 013-todo's AddTodo).
    pub fn handler_param_count(&self, widget_name: &str, event_name: &str) -> Option<usize> {
        let ev = event_name.trim_start_matches('.');
        self.handler_param_names
            .get(&(widget_name.to_string(), ev.to_string()))
            .map(|v| v.len())
    }

    /// PLAN-685 G1: declared parameter NAMES per `(widget, event)` handler.
    /// Stripped in-body callback args (child_emit STRIPPED table) are TEXT
    /// snapshots evaluated outside the VM where handler locals don't exist;
    /// dispatch pairs these names with the actual dispatch args to give the
    /// snapshot evaluator the handler's lexical param bindings.
    pub fn handler_param_names(&self, widget_name: &str, event_name: &str) -> Option<Vec<String>> {
        let ev = event_name.trim_start_matches('.');
        self.handler_param_names
            .get(&(widget_name.to_string(), ev.to_string()))
            .cloned()
    }

    /// Create a new VmBridge for a given AuraWidget, with no imported symbols.
    ///
    /// Delegates to [`VmBridge::new_with_imports`] with an empty import list.
    /// Use `new_with_imports` when the widget's `use`-imported functions/types
    /// must be available to its handlers (e.g. `build_month_grid`).
    pub fn new(widget: &AuraWidget) -> Result<Self> {
        Self::new_with_imports(widget, Vec::new())
    }

    /// Create a new VmBridge, compiling imports + state type + handlers in one
    /// `Codegen` pass so cross-references resolve within a single module.
    ///
    /// # Arguments
    ///
    /// * `widget` - The AuraWidget to create a bridge for
    /// * `import_stmts` - `Stmt::Fn` / `Stmt::TypeDecl` / `Stmt::EnumDecl` /
    ///   `Stmt::Ext` collected from the widget's `use`-imported modules (the
    ///   helpers it calls, like `build_month_grid`)
    ///
    /// # Errors
    ///
    /// Returns an error if handler synthesis or VM initialization fails.
    pub fn new_with_imports(widget: &AuraWidget, import_stmts: Vec<Stmt>) -> Result<Self> {
        let empty = std::collections::HashMap::new();
        Self::new_with_children(widget, &[], import_stmts, &empty, false)
    }

    /// Plan 320: create a VmBridge compiling root widget + child widgets into
    /// ONE VM module (single VM widget tree). Child handlers get namespaced fn
    /// names (handler_<Widget>_<Event>) so they coexist in one module.
    pub fn new_with_children(
        widget: &AuraWidget,
        child_widgets: &[crate::aura::AuraWidget],
        import_stmts: Vec<Stmt>,
        import_aliases: &std::collections::HashMap<String, String>,
        api_over_http: bool,
    ) -> Result<Self> {
        // Plan 446 批一 (C1-3): 任一被引用模块 parse 失败 → 桥接构造致命。
        // 此前 parse 失败仅 WARN,模块符号静默消失,应用以空壳/半壳渲染,
        // 极难定位(os-config C1/J1 现场共同诉求:显式报错优于静默)。
        {
            let failures = crate::ui_module_parse_failures().lock().unwrap();
            if !failures.is_empty() {
                return Err(VmBridgeError::InvalidState(format!(
                    "{} module(s) failed to parse (plan-446 C1-3, see stderr): [{}]",
                    failures.len(),
                    failures.join(" | ")
                )));
            }
        }

        // Ensure BIGVM_NATIVES is populated before AutoVM::new()
        crate::vm::native_registry::register_builtin_natives();

        let widget_name = widget.name.clone();

        // 1. Synthesize imports + ALL widgets' state types + handlers into ONE
        //    Module via the genuine VM Codegen. Plan 320: single VM.
        let (module, registry) = crate::ui::handler_codegen::synthesize_widget_module(widget, child_widgets, import_stmts, import_aliases, api_over_http)
            .map_err(|e| VmBridgeError::InvalidState(format!(
                "handler synthesis failed for '{}': {}", widget_name, e
            )))?;
        // PLAN-642 T-15: 合成同线程紧邻捕获视图侧 store 别名快照（线程级
        // 快照会被多组件工程的后续合成覆盖，须随 bridge 存档）。
        let store_alias_snapshot =
            crate::ui::handler_codegen::capture_view_store_alias_snapshot();

        // Metadata we still need after handing the module to the linker.
        let object_keys = module.object_keys.clone();
        let object_types = module.object_types.clone();
        let strings = module.strings.clone();

        // 2. Link (single module → no cross-module relocation).
        let mut linker = Linker::new();
        linker.add_entry_module(module);
        let (code, exports) = linker.link().map_err(|e| VmBridgeError::InvalidState(
            format!("link failed for '{}': {}", widget_name, e)
        ))?;

        // 3. Build flash + VM with unified metadata tables.
        let flash = VirtualFlash::from_vec_with_metadata(code, exports, object_keys, object_types);
        let mut vm = AutoVM::new(flash, 4096);
        // Plan 318: load the codegen's generic_registry so CONSTRUCT_INSTANCE can
        // resolve struct field names (Note.title). Without this, field_names fall
        // back to "_unknown" and struct field access in handler bodies / for-loop
        // bindings fails.
        vm.load_generic_registry(registry);
        vm.load_strings(strings);

        // 4. Build state fields and default values, then create the state
        //    GenericInstanceData on the VM heap. This is the same object the
        //    handler's `__state.field` GET_FIELD/SET_FIELD opcodes touch.
        let mut field_names = Vec::with_capacity(widget.state_vars.len());
        let mut field_values = Vec::with_capacity(widget.state_vars.len());
        for state_var in &widget.state_vars {
            field_names.push(state_var.name.clone());
            field_values.push(eval_expr_to_value(&state_var.initial, &mut vm));
        }

        let mono_name = format!("{}_State", widget_name);
        let instance = GenericInstanceData::new_with_names(
            mono_name,
            field_values,
            field_names.clone(),
        );
        let state_obj_id = vm.insert_heap_object(instance);
        // Plan 419: widget state 的永久 stake(bridges 与 VM 同寿命;每次
        // call_handler 的 push/RET-sweep 在此之上配平)。
        vm.rc_retain_id(state_obj_id);

        // Audit B12(b): record declared handler arities (root + children) so
        // dispatch can skip phantom string args at no-param handlers.
        // PLAN-685: names (not just counts) — stripped in-body callback
        // snapshot evaluation resolves bare identifiers against them.
        let mut handler_param_names = std::collections::HashMap::new();
        collect_param_names_from_widget(widget, &mut handler_param_names);
        for child in child_widgets {
            collect_param_names_from_widget(child, &mut handler_param_names);
        }

        Ok(Self {
            vm,
            state_obj_id,
            state_field_names: field_names,
            widget_name,
            child_state_map: std::cell::RefCell::new(std::collections::HashMap::new()),
            handler_param_names,
            import_aliases: import_aliases.clone(),
            store_alias_snapshot,
            parked_tasks: std::cell::RefCell::new(Vec::new()),
            cpu_write_queue: std::cell::RefCell::new(std::collections::VecDeque::new()),
            init_demand_queue: std::cell::RefCell::new(std::collections::VecDeque::new()),
            init_demand_records: std::cell::RefCell::new(std::collections::HashMap::new()),
            init_generation: std::cell::Cell::new(0),
            busy_flag_names: std::cell::RefCell::new(Vec::new()),
            #[cfg(feature = "ui-interpreter")]
            memo_cache: std::cell::RefCell::new(crate::ui::memo_deps::MemoCache::new()),
            #[cfg(feature = "ui-interpreter")]
            dep_recorder: std::cell::RefCell::new(None),
            resolve_probe_count: std::sync::atomic::AtomicU64::new(0),
            #[cfg(feature = "ui-interpreter")]
            computed_signals: std::cell::RefCell::new(HashMap::new()),
            signal_hits: std::sync::atomic::AtomicU64::new(0),
            signal_misses: std::sync::atomic::AtomicU64::new(0),
            frame_retains_cur: std::sync::Mutex::new(Vec::new()),
            frame_retains_prev: std::sync::Mutex::new(Vec::new()),
        })
    }

    /// PR-3b Step 2: create a VmBridge directly from a `WidgetDecl` (and child
    /// `WidgetDecl`s), bypassing the AuraWidget intermediate representation for
    /// the logic/handler-synthesis half. Parallel to [`VmBridge::new_with_children`].
    ///
    /// State initialization reads `decl.model.fields` instead of
    /// `widget.state_vars`, converting each field's `init` (base `Expr`) to a
    /// runtime `Value` via `eval_expr_to_value`. When the
    /// widget has a `.Tick` handler, the synthesized "interval" field is filtered
    /// out of the state object (it is consumed by the tick scheduler, not a
    /// `ref()` state field).
    pub fn new_from_decls(
        decl: &crate::ast::WidgetDecl,
        child_decls: &[crate::ast::WidgetDecl],
        import_stmts: Vec<Stmt>,
        import_aliases: &std::collections::HashMap<String, String>,
        api_over_http: bool,
    ) -> Result<Self> {
        // Ensure BIGVM_NATIVES is populated before AutoVM::new().
        crate::vm::native_registry::register_builtin_natives();

        let widget_name = decl.name.to_string();

        // 1. Synthesize imports + ALL widgets' state types + handlers into ONE
        //    Module via the genuine VM Codegen, reading directly from WidgetDecl.
        let (module, registry) = crate::ui::handler_codegen::synthesize_from_decl(
            decl,
            child_decls,
            import_stmts,
            import_aliases,
            api_over_http,
        )
        .map_err(|e| VmBridgeError::InvalidState(format!(
            "handler synthesis failed for '{}': {}",
            widget_name, e
        )))?;
        // PLAN-642 T-15: 同 new_with_children——快照随 bridge 存档
        // （线程级快照会被多组件工程的后续合成覆盖）。
        let store_alias_snapshot =
            crate::ui::handler_codegen::capture_view_store_alias_snapshot();

        // Metadata we still need after handing the module to the linker.
        let object_keys = module.object_keys.clone();
        let object_types = module.object_types.clone();
        let strings = module.strings.clone();

        // 2. Link (single module → no cross-module relocation).
        let mut linker = Linker::new();
        linker.add_entry_module(module);
        let (code, exports) = linker.link().map_err(|e| VmBridgeError::InvalidState(
            format!("link failed for '{}': {}", widget_name, e)
        ))?;

        // 3. Build flash + VM with unified metadata tables.
        let flash = VirtualFlash::from_vec_with_metadata(code, exports, object_keys, object_types);
        let mut vm = AutoVM::new(flash, 4096);
        vm.load_generic_registry(registry);
        vm.load_strings(strings);

        // 4. Build state fields and default values from the decl's model fields.
        //    PR-3b: if the widget has a `.Tick` handler, skip the "interval"
        //    field (it is consumed by the tick scheduler, not a ref() state).
        let tick_interval = crate::ui::handler_codegen::extract_tick_interval_from_decl(decl);

        let model_fields: Vec<&crate::ast::ModelField> = decl
            .model
            .as_ref()
            .map(|m| {
                m.fields
                    .iter()
                    .filter(|f| !(tick_interval.is_some() && f.name.as_str() == "interval"))
                    .collect()
            })
            .unwrap_or_default();

        let mut field_names = Vec::with_capacity(model_fields.len());
        let mut field_values = Vec::with_capacity(model_fields.len());
        for field in model_fields {
            field_names.push(field.name.to_string());
            field_values.push(eval_expr_to_value(&field.init, &mut vm));
        }

        // Plan 370 D-GAP-4: merge store state fields into root state object.
        // Store fields are merged so that `store.X` (rewritten to `__state.X`)
        // resolves correctly at runtime.
        //
        // Plan 370 (Issue 2): ALSO merge model fields from view-having child
        // widgets (e.g. EditorPanel's `editing`, `edit_title`). In VM merged
        // mode there is a single unified state object — `ensure_child_state`
        // writes props into ROOT state and returns root_id, so child widget
        // model vars must live there too, or `read_state("editing")` fails
        // and `eval_condition_with` short-circuits to false (rendering the
        // wrong branch). The `contains` guard deduplicates against root +
        // store fields.
        for child in child_decls {
            if let Some(child_model) = &child.model {
                for sf in &child_model.fields {
                    let name = sf.name.to_string();
                    if !field_names.contains(&name) {
                        field_names.push(name.clone());
                        field_values.push(eval_expr_to_value(&sf.init, &mut vm));
                    }
                }
            }
        }

        // Plan 401/VM-routing: inject the router state fields so handlers can
        // read/write __current_route (set by router.push) and __route_params
        // (populated by the outlet/sync_route_params from the matched route).
        // Only when the root widget declares a routes {} block.
        if decl.routes.is_some() {
            if !field_names.iter().any(|n| n == "__current_route") {
                field_names.push("__current_route".to_string());
                field_values.push(auto_val::Value::str(
                    decl.routes.as_ref().and_then(|r| r.routes.first()).map(|r| r.path.as_str()).unwrap_or("/")
                ));
            }
            if !field_names.iter().any(|n| n == "__route_params") {
                field_names.push("__route_params".to_string());
                // PLAN-712 T-17：与 sync_route_params 写侧同律——字段持堆引用
                //（裸 Value::Obj 读侧被 GET_FIELD 兜底吞 0）。
                let mut od = crate::vm::types::ObjectData::new();
                let seeded = vm.insert_heap_object(od);
                // 实例常驻字段对堆值的永久 stake（与 state_obj_id 自身的
                // rc_retain 同寿命语义；write_state 路径的对称面见
                // stake/release_state_value）。
                vm.rc_retain_id(seeded as u64);
                field_values.push(auto_val::Value::VmRef(auto_val::VmRef { id: seeded as usize }));
            }
        }

        // Plan 412 续(toast VM 化):注入 __toast 挂载字段。handler 里的
        // toast()/toast.success() 调用被 handler_codegen 重写为对它的赋值
        // (编码 "kindmsgpositionduration_ms"),dynamic_view
        // 取走后渲染窗口级悬浮层并清空。
        if !field_names.iter().any(|n| n == "__toast") {
            field_names.push("__toast".to_string());
            field_values.push(auto_val::Value::str(""));
        }

        // Plan 576 (D3): 子件体内引号 emit 的挂载字段对。handler 里的
        // `."msg"(v)` 调用被 handler_codegen 重写为 __emit_<W>_<msg> 桥函数
        // （体内写 msg 名 + 载荷值），on_with_input_for 在子 handler 返回后
        // 读出并清账，经 child_emit ROUTES 派发到父 on<msg> 绑定（__toast
        // 同型"handler 写状态、update 侧消费"管线）。
        if !field_names.iter().any(|n| n == "__emit_msg") {
            field_names.push("__emit_msg".to_string());
            field_values.push(auto_val::Value::str(""));
        }
        if !field_names.iter().any(|n| n == "__emit_payload") {
            field_names.push("__emit_payload".to_string());
            field_values.push(auto_val::Value::Nil);
        }

        let mono_name = format!("{}_State", widget_name);
        let instance = GenericInstanceData::new_with_names(
            mono_name,
            field_values,
            field_names.clone(),
        );
        let state_obj_id = vm.insert_heap_object(instance);
        // Plan 419: widget state 的永久 stake(bridges 与 VM 同寿命;每次
        // call_handler 的 push/RET-sweep 在此之上配平)。
        vm.rc_retain_id(state_obj_id);

        // Audit B12(b): record declared handler arities (root + children) so
        // dispatch can skip phantom string args at no-param handlers.
        // PLAN-685: names (not just counts) — stripped in-body callback
        // snapshot evaluation resolves bare identifiers against them.
        let mut handler_param_names = std::collections::HashMap::new();
        collect_param_names_from_decl(decl, &mut handler_param_names);
        for child in child_decls {
            collect_param_names_from_decl(child, &mut handler_param_names);
        }

        Ok(Self {
            vm,
            state_obj_id,
            state_field_names: field_names,
            widget_name,
            child_state_map: std::cell::RefCell::new(std::collections::HashMap::new()),
            handler_param_names,
            import_aliases: import_aliases.clone(),
            store_alias_snapshot,
            parked_tasks: std::cell::RefCell::new(Vec::new()),
            cpu_write_queue: std::cell::RefCell::new(std::collections::VecDeque::new()),
            init_demand_queue: std::cell::RefCell::new(std::collections::VecDeque::new()),
            init_demand_records: std::cell::RefCell::new(std::collections::HashMap::new()),
            init_generation: std::cell::Cell::new(0),
            busy_flag_names: std::cell::RefCell::new(Vec::new()),
            #[cfg(feature = "ui-interpreter")]
            memo_cache: std::cell::RefCell::new(crate::ui::memo_deps::MemoCache::new()),
            #[cfg(feature = "ui-interpreter")]
            dep_recorder: std::cell::RefCell::new(None),
            resolve_probe_count: std::sync::atomic::AtomicU64::new(0),
            #[cfg(feature = "ui-interpreter")]
            computed_signals: std::cell::RefCell::new(HashMap::new()),
            signal_hits: std::sync::atomic::AtomicU64::new(0),
            signal_misses: std::sync::atomic::AtomicU64::new(0),
            frame_retains_cur: std::sync::Mutex::new(Vec::new()),
            frame_retains_prev: std::sync::Mutex::new(Vec::new()),
        })
    }

    /// Create a new VmBridge with a pre-configured AutoVM instance (legacy path).
    ///
    /// Plan 323: the VM is always rebuilt from the synthesized module, so the
    /// passed `vm` is intentionally ignored. This entry point is retained for
    /// API compatibility; prefer [`VmBridge::new`] / [`VmBridge::new_with_imports`].
    pub fn new_with_vm(_vm: AutoVM, widget: &AuraWidget) -> Result<Self> {
        Self::new_with_imports(widget, Vec::new())
    }

    /// Read a state field value from the VM.
    ///
    /// Accesses the state heap object and returns the field value by name.
    ///
    /// # Arguments
    ///
    /// * `field_name` - Name of the state field (e.g., "count")
    ///
    /// # Returns
    ///
    /// The current value of the field, or an error if the field doesn't exist.
    pub fn read_state(&self, field_name: &str) -> Result<Value> {
        // Find field index by name — prefer the cached field list, but fall
        // back to a name-based lookup on the live heap object. Plan 049:
        // `ensure_child_state` seeds child model-var defaults (e.g. BlockItem's
        // `collapsed`) into the root heap object AFTER the bridge is built, so
        // those fields are missing from `state_field_names` and the cached
        // index lookup alone would wrongly report FieldNotFound.
        let obj = self.vm.get_heap_object(self.state_obj_id)
            .ok_or_else(|| VmBridgeError::InvalidState(
                format!("state heap object {} not found", self.state_obj_id)
            ))?;

        let field_index = {
            let guard = obj.read().unwrap();
            let instance = guard.as_any().downcast_ref::<GenericInstanceData>()
                .ok_or_else(|| VmBridgeError::InvalidState(
                    "state object is not a GenericInstanceData".to_string()
                ))?;
            self.state_field_names.iter().position(|name| name == field_name)
                .or_else(|| instance.field_names.iter().position(|n| n == field_name))
                .ok_or_else(|| VmBridgeError::FieldNotFound(field_name.to_string()))?
        };

        let guard = obj.read().unwrap();
        let instance = guard.as_any().downcast_ref::<GenericInstanceData>()
            .ok_or_else(|| VmBridgeError::InvalidState(
                "state object is not a GenericInstanceData".to_string()
            ))?;

        // PLAN-047 T-01: 读通道录制——成功的具名字段读 = 一条依赖边。
        match instance.get_field(field_index).cloned() {
            Some(v) => {
                self.record_dep_read(self.state_obj_id, field_name);
                Ok(v)
            }
            None => Err(VmBridgeError::FieldNotFound(field_name.to_string())),
        }
    }

    /// Write a state field value to the VM.
    ///
    /// Updates the state heap object field by name.
    ///
    /// # Arguments
    ///
    /// * `field_name` - Name of the state field
    /// * `value` - New value for the field
    pub fn write_state(&mut self, field_name: &str, value: Value) -> Result<()> {
        // Find field index by name — same cached-list-first + live-object
        // fallback as read_state (Plan 049 child model-var defaults).
        let obj = self.vm.get_heap_object(self.state_obj_id)
            .ok_or_else(|| VmBridgeError::InvalidState(
                format!("state heap object {} not found", self.state_obj_id)
            ))?;
        let field_index = {
            let guard = obj.read().unwrap();
            let instance = guard.as_any().downcast_ref::<GenericInstanceData>()
                .ok_or_else(|| VmBridgeError::InvalidState(
                    "state object is not a GenericInstanceData".to_string()
                ))?;
            self.state_field_names.iter().position(|name| name == field_name)
                .or_else(|| instance.field_names.iter().position(|n| n == field_name))
                .ok_or_else(|| VmBridgeError::FieldNotFound(field_name.to_string()))?
        };

        // Access the heap object with write access
        let obj = self.vm.get_heap_object_mut(self.state_obj_id)
            .ok_or_else(|| VmBridgeError::InvalidState(
                format!("state heap object {} not found", self.state_obj_id)
            ))?;

        let mut guard = obj.write().unwrap();
        let instance = guard.as_any_mut().downcast_mut::<GenericInstanceData>()
            .ok_or_else(|| VmBridgeError::InvalidState(
                "state object is not a GenericInstanceData".to_string()
            ))?;

        // PLAN-062 T12: Rust 侧直写绕过 VM 栈——容器字段获得持有时必须
        // 显式记账（rc.rs §2.3 协议）：新值（含 Array 内层 VmRef）+1，
        // 旧值对称 -1。旧内容保活模型的最后一块拼图（pkg 子态 props
        // 种子路径 UAF 根因）。
        let old = instance.get_field(field_index).cloned();
        if let Some(old_v) = old {
            self.release_state_value(&old_v);
        }
        self.stake_state_value(&value);
        instance.set_field(field_index, value)
            .map_err(|e| VmBridgeError::InvalidState(e))?;
        // PLAN-045 T-02：Rust 侧直写绕过 engine 突变臂，全局 state_mutation_seq
        // 原本不动——`set_route` 等桥写通道因此对 memo 快速路径不可见（陈旧
        // 误命中）。与 engine 突变臂同口径在此补 bump（PLAN-045 决策注记②）。
        // PLAN-047 T-03: A 类定点归因（bump_path 内含全局 bump，语义不变）。
        self.vm.bump_path(self.state_obj_id, Some(field_name));
        Ok(())
    }

    /// PLAN-654 阶段 B: 写 state；字段不存在时在根态对象上追加（框架注入面）。
    /// 与 `ensure_child_state` 同款「可缺字段」语义，供 `__clock_*` 使用。
    pub fn write_or_insert_state(&mut self, field_name: &str, value: Value) -> Result<()> {
        match self.write_state(field_name, value.clone()) {
            Ok(()) => Ok(()),
            Err(VmBridgeError::FieldNotFound(_)) => {
                let obj = self.vm.get_heap_object_mut(self.state_obj_id)
                    .ok_or_else(|| VmBridgeError::InvalidState(
                        format!("state heap object {} not found", self.state_obj_id)
                    ))?;
                let mut guard = obj.write().unwrap();
                let instance = guard.as_any_mut().downcast_mut::<GenericInstanceData>()
                    .ok_or_else(|| VmBridgeError::InvalidState(
                        "state object is not a GenericInstanceData".to_string()
                    ))?;
                instance.field_names.push(field_name.to_string());
                instance.fields.push(value);
                self.state_field_names.push(field_name.to_string());
                // PLAN-045 T-02：新增字段同属状态面突变，与 write_state 同口径
                // 补 bump（memo 快速路径可见性）。PLAN-047 T-03: A 类定点归因。
                self.vm.bump_path(self.state_obj_id, Some(field_name));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// PLAN-062 T12: state 值获得持有——顶层堆引用 + Array 内层各 +1。
    fn stake_state_value(&self, v: &Value) {
        match v {
            Value::Int(id) if *id >= 4_000_000 => {
                self.vm.rc_retain_id(*id as u64);
            }
            Value::VmRef(r) => {
                self.vm.rc_retain_id(r.id as u64);
            }
            Value::Array(arr) => {
                for el in arr.values.iter() {
                    if let Value::VmRef(r) = el {
                        self.vm.rc_retain_id(r.id as u64);
                    }
                    if let Value::Int(id) = el {
                        if *id >= 4_000_000 {
                            self.vm.rc_retain_id(*id as u64);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// PLAN-062 T12: state 值释放持有——stake 的对称面。
    fn release_state_value(&self, v: &Value) {
        match v {
            Value::Int(id) if *id >= 4_000_000 => {
                self.vm.rc_release_id(*id as u64);
            }
            Value::VmRef(r) => {
                self.vm.rc_release_id(r.id as u64);
            }
            Value::Array(arr) => {
                for el in arr.values.iter() {
                    if let Value::VmRef(r) = el {
                        self.vm.rc_release_id(r.id as u64);
                    }
                    if let Value::Int(id) = el {
                        if *id >= 4_000_000 {
                            self.vm.rc_release_id(*id as u64);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// PLAN-712 T-17 第二层：把 `auto_val::Obj` 物化为堆 `ObjectData` 并以
    /// `Value::VmRef` 返回——**状态字段的持有约定是堆引用**（对象字面量/
    /// JSON 解析都落堆 id：Plan 390 H3b / Plan 057 Bug 2）。GET_FIELD 实例
    /// 臂的值分发只认 Int≥堆基/VmRef/标量，**裸 `Value::Obj` 落
    /// `_ => push_i32(0)` 兜底**——`router.param` 曾因此恒 0：
    /// `sync_route_params` 直写裸 Obj，详情页读回 book_id=0 → 0 entries
    /// （桌面 + 独立 VM 双轨实机同证，2026-10-01）。
    /// 持有记账不在此处——`write_state` 的 stake/release 对称面负责。
    pub fn materialize_obj_to_heap(&self, obj: auto_val::Obj) -> Value {
        let mut od = crate::vm::types::ObjectData::new();
        for (k, v) in obj.iter() {
            if let Some(name) = k.name() {
                od.set(auto_val::ValueKey::Str(name.into()), v.clone());
            }
        }
        let id = self.vm.insert_heap_object(od);
        auto_val::Value::VmRef(auto_val::VmRef { id: id as usize })
    }

    /// Read a state field that holds an array_id and return the actual Vec<Value>.
    ///
    /// When the state field is `Value::Array`, returns its inner Vec directly.
    /// When it's `Value::Int(id)` / `Value::VmRef(id)` (array ref from a `[...]`
    /// literal or `List<T>.new`), reads from heap_objects (ListData<Value> /
    /// ListData<i32>, Plan 390 §15 H3b — the legacy arrays registry is gone).
    /// This unblocks 015-notes vm rendering where `list_notes()` →
    /// `notes.to_array()` returns a VmRef to the struct list.
    pub fn read_state_as_vec(&self, field_name: &str) -> Result<Vec<Value>> {
        let val = self.read_state(field_name)?;
        match val {
            Value::Array(arr) => Ok(arr.values),
            // Plan 390 §15 H3b: array literals are ListData<Value> in
            // heap_objects (probe instead of the old 2M id-range guard).
            Value::Int(id) => {
                let arr_id = id as u64;
                if let Some(obj) = self.vm.get_heap_object(arr_id) {
                    // PLAN-047 T-01: 容器内容读 = (heap_id, "*") 粗粒度依赖
                    // （原地突变无字段归因面的保守兜底）。
                    self.record_dep_any(arr_id);
                    let guard = obj.read().unwrap();
                    use crate::vm::types::ListData;
                    if let Some(list) = guard.as_any().downcast_ref::<ListData<Value>>() {
                        Ok(list.elems.clone())
                    } else if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                        Ok(list.elems.iter().map(|i| Value::Int(*i)).collect())
                    } else {
                        Err(VmBridgeError::InvalidState(
                            format!("array_id {} is not a readable list", arr_id)
                        ))
                    }
                } else {
                    Err(VmBridgeError::InvalidState(
                        format!("array_id {} not found in heap_objects", arr_id)
                    ))
                }
            }
            Value::VmRef(r) => self.vmref_to_vec(r.id),
            other => Err(VmBridgeError::InvalidState(
                format!("Expected array for field '{}', got {:?}", field_name, other)
            )),
        }
    }

    /// Plan 335: dereference a `VmRef` holding a list into `Vec<Value>`. Tries
    /// heap_objects first (ListData<Value> from `List<T>.new` of structs, or
    /// ListData<i32>). Plan 390 §15 H3b: array literals are ListData<Value> in
    /// heap_objects too.
    /// conventions (4000000 heap / 2000000 arrays) so it stays correct if the
    /// generators' start values change.
    /// Plan 318: index into a list value held as a `VmRef` (heap id) or array_id,
    /// returning the element at `i`. Used by the view builder's Index expr
    /// (e.g. `.notes[.active_id]`) to dereference a List<Note> element.
    pub fn index_list(&self, id: usize, i: i32) -> Option<Value> {
        if let Ok(elems) = self.vmref_to_vec(id) {
            let idx = i as usize;
            if idx < elems.len() {
                return Some(elems[idx].clone());
            }
        }
        None
    }

    /// Plan 370 (Issue 2): return ALL elements of a heap array (ListData),
    /// used by `for` loops over a dotted prop path like `.note.tags`.
    /// PLAN-013 T1: 为 props-feed 消费方(terminal)解码 List<str> 值。
    /// 覆盖三形态:内联 Value::Array(元素为负字符串表哨兵或已解码串)、
    /// 堆 ListData<i32>(负哨兵)、堆 ListData<Value>。负哨兵按
    /// `-(idx)-1` 编码回读字符串表(ffi/convert.rs Vec<String> 同款)。
    pub fn read_str_list_value(&self, val: &Value) -> Vec<String> {
        let items: Vec<Value> = match val {
            Value::Array(arr) => arr.values.clone(),
            Value::Int(id) if *id >= 4_000_000 => self.index_list_all(*id as usize),
            Value::VmRef(r) => self.index_list_all(r.id),
            _ => Vec::new(),
        };
        items
            .iter()
            .map(|v| match v {
                Value::Str(sv) => sv.to_string(),
                Value::Int(n) if *n < 0 => {
                    let idx = (-n - 1) as usize;
                    let strings = self.vm.strings.read().unwrap();
                    strings
                        .get(idx)
                        .map(|b| String::from_utf8_lossy(b).into_owned())
                        .unwrap_or_default()
                }
                other => other.repr().to_string(),
            })
            .collect()
    }

    pub fn index_list_all(&self, id: usize) -> Vec<Value> {
        let r = self.vmref_to_vec(id);
        if std::env::var("AUTO_DEBUG_EMIT").is_ok() {
            if let Err(e) = &r {
                if let Some(obj) = self.vm.get_heap_object(id as u64) {
                    let g = obj.read().unwrap();
                    eprintln!("[VM-IDX] id={} not-list err={}", id, e);
                } else {
                    eprintln!("[VM-IDX] id={} no-heap-object err={}", id, e);
                }
            }
        }
        r.unwrap_or_default()
    }

    pub fn vmref_to_vec(&self, id: usize) -> Result<Vec<Value>> {
        // Path 1: heap_objects — ListData<Value> (array literals / struct lists,
        // Plan 390 §15 H3b) or ListData<i32>.
        if let Some(obj) = self.vm.get_heap_object(id as u64) {
            // PLAN-047 T-01: 列表解引用读通道（read_state_as_vec 的 VmRef 臂
            // / for 迭代 / Index 表达式共用）——容器内容读粗粒度依赖。
            self.record_dep_any(id as u64);
            let guard = obj.read().unwrap();
            use crate::vm::types::ListData;
            if let Some(list) = guard.as_any().downcast_ref::<ListData<Value>>() {
                return Ok(list.elems.clone());
            }
            if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                return Ok(list.elems.iter().map(|i| Value::Int(*i)).collect());
            }
        }
        Err(VmBridgeError::InvalidState(format!(
            "VmRef {} is not a readable list (not in heap_objects as ListData)", id
        )))
    }

    /// Write a Vec<Value> back to a state field that holds an array reference.
    ///
    /// When the state field is `Value::Array`, writes back as Value::Array.
    /// When it's `Value::Int(id)` (array ref from a `[...]` literal), writes to
    /// the ListData<Value> in heap_objects (Plan 390 §15 H3b).
    pub fn write_state_vec(&mut self, field_name: &str, values: Vec<Value>) -> Result<()> {
        let val = self.read_state(field_name)?;
        match val {
            Value::Array(_) => {
                self.write_state(field_name, Value::Array(auto_val::Array { values }))
            }
            // Plan 390 §15 H3b: array literals are ListData<Value> in
            // heap_objects (probe instead of the old 2M id-range guard).
            Value::Int(id) => {
                let arr_id = id as u64;
                if let Some(obj) = self.vm.get_heap_object(arr_id) {
                    let mut guard = obj.write().unwrap();
                    if let Some(list) = guard.as_any_mut().downcast_mut::<crate::vm::types::ListData<Value>>() {
                        // PLAN-062 T12: 元素整体替换——旧元素释放、新元素
                        // 获得（容器持有协议,同 write_state）。
                        for old_el in list.elems.iter() {
                            self.release_state_value(old_el);
                        }
                        let values_ref = &values;
                        for new_el in values_ref.iter() {
                            self.stake_state_value(new_el);
                        }
                        list.elems = values;
                        // PLAN-045 T-02：容器原地替换 = 状态面突变（memo
                        // 快速路径可见性），与 write_state 同口径补 bump。
                        // PLAN-047 T-03: B 类定点归因（堆列表 wildcard）。
                        self.vm.bump_path(arr_id, None);
                        Ok(())
                    } else {
                        Err(VmBridgeError::InvalidState(
                            format!("array_id {} is not a writable list", arr_id)
                        ))
                    }
                } else {
                    Err(VmBridgeError::InvalidState(
                        format!("array_id {} not found in heap_objects", arr_id)
                    ))
                }
            }
            // EDGE-16: List<T>.new([]) 物化成 VmRef 指向 ListData<Value> 堆对象。
            // write_state_vec 需把元素写进该堆对象(同 Int(id) 路径,只是 id 来自 VmRef)。
            Value::VmRef(r) => {
                let arr_id = r.id as u64;
                if let Some(obj) = self.vm.get_heap_object(arr_id) {
                    let mut guard = obj.write().unwrap();
                    if let Some(list) = guard.as_any_mut().downcast_mut::<crate::vm::types::ListData<Value>>() {
                        // PLAN-062 T12: 同 Int 臂——旧释放新获得。
                        for old_el in list.elems.iter() {
                            self.release_state_value(old_el);
                        }
                        let values_ref = &values;
                        for new_el in values_ref.iter() {
                            self.stake_state_value(new_el);
                        }
                        list.elems = values;
                        // PLAN-045 T-02：容器原地替换补 bump（同 Int 臂）。
                        // PLAN-047 T-03: B 类定点归因（同 Int 臂）。
                        self.vm.bump_path(arr_id, None);
                        Ok(())
                    } else {
                        Err(VmBridgeError::InvalidState(
                            format!("VmRef id {} is not a writable list", arr_id)
                        ))
                    }
                } else {
                    Err(VmBridgeError::InvalidState(
                        format!("VmRef id {} not found in heap_objects", arr_id)
                    ))
                }
            }
            other => Err(VmBridgeError::InvalidState(
                format!("Expected array for field '{}', got {:?}", field_name, other)
            )),
        }
    }

    /// Materialize a heap object reference into an inline `Value::Obj` so the
    /// view builder's `Value::Obj`-based field resolvers can read its fields.
    ///
    /// An Auto Obj literal (`{ label: ..., ... }`) is compiled to `CREATE_OBJ`,
    /// which stores an `ObjectData` in the VM `objects` registry and leaves its
    /// id on the stack as a plain `Value::Int`. When such a value is iterated as
    /// a loop item (e.g. `for cell in .days`), the binding is a bare
    /// `Value::Int(obj_id)` and `cell.label` cannot resolve — the view builder's
    /// resolvers only handle `Value::Obj`. This derefs the id (via
    /// `heap_objects`, Plan 390 §15 H3b — ObjectData and GenericInstanceData
    /// both live there) into a `Value::Obj` whose string-keyed fields mirror
    /// the stored data. Non-object values (real ints, already-inline
    /// `Value::Obj`, etc.) pass through unchanged.
    pub fn materialize_obj_ref(&self, v: &Value) -> Value {
        match v {
            Value::Int(id) => {
                // Plan 390 §15 H3b: CREATE_OBJ → ObjectData and
                // CONSTRUCT_INSTANCE → GenericInstanceData both in
                // heap_objects — single probe + downcast.
                if let Some(obj) = self.vm.get_heap_object(*id as u64) {
                    // PLAN-047 T-01: 堆结构展开读 = 粗粒度依赖（展开产物含
                    // 全部字段——任一字段原地变都须失效，记 " *" 面）。
                    self.record_dep_any(*id as u64);
                    let guard = obj.read().unwrap();
                    if let Some(od) = guard.as_any().downcast_ref::<crate::vm::types::ObjectData>() {
                        let mut out = auto_val::Obj::new();
                        for (key, val) in od.fields.iter() {
                            if let auto_val::ValueKey::Str(s) = key {
                                out.set(s.clone(), val.clone());
                            }
                        }
                        return Value::Obj(Box::new(out));
                    }
                    // Plan 318: GenericInstanceData structs. List<Note>.new(
                    // [Note{...}]) stores Note instances as bare Int(heap_id)
                    // elements; without this arm, note.title in a for-loop body
                    // can't resolve.
                    if let Some(inst) = guard.as_any().downcast_ref::<crate::vm::generic_registry::GenericInstanceData>() {
                        let mut out = auto_val::Obj::new();
                        for (val, name) in inst.fields.iter().zip(inst.field_names.iter()) {
                            if name != "_unknown" {
                                out.set(name.clone(), val.clone());
                            }
                        }
                        return Value::Obj(Box::new(out));
                    }
                }
                v.clone()
            }
            // Plan 335: VmRef (heap id from other paths) — deref to Value::Obj.
            // Plan 402: ObjectData downcast added (Obj literals like `{ x, y, ... }`
            // compile to ObjectData, not GenericInstanceData; without this arm, a
            // `for cell in board` whose cells are Obj literals fails to resolve
            // `cell.x` in onclick params — materialize returns the raw VmRef and
            // resolve_binding_path only matches Value::Obj).
            Value::VmRef(r) => {
                if let Some(obj) = self.vm.get_heap_object(r.id as u64) {
                    // PLAN-047 T-01: 同 Int 臂——堆结构展开读粗粒度依赖。
                    self.record_dep_any(r.id as u64);
                    let guard = obj.read().unwrap();
                    if let Some(od) = guard.as_any().downcast_ref::<crate::vm::types::ObjectData>() {
                        let mut out = auto_val::Obj::new();
                        for (key, val) in od.fields.iter() {
                            if let auto_val::ValueKey::Str(s) = key {
                                out.set(s.clone(), val.clone());
                            }
                        }
                        return Value::Obj(Box::new(out));
                    }
                    if let Some(inst) = guard.as_any().downcast_ref::<crate::vm::generic_registry::GenericInstanceData>() {
                        let mut out = auto_val::Obj::new();
                        for (val, name) in inst.fields.iter().zip(inst.field_names.iter()) {
                            if name != "_unknown" {
                                out.set(name.clone(), val.clone());
                            }
                        }
                        return Value::Obj(Box::new(out));
                    }
                }
                v.clone()
            }
            _ => v.clone(),
        }
    }

    /// Read all state fields as a name -> value map.
    ///
    /// Useful for bulk state reads during rendering.
    pub fn read_all_state(&self) -> HashMap<String, Value> {
        let mut result = HashMap::new();

        let obj = match self.vm.get_heap_object(self.state_obj_id) {
            Some(o) => o,
            None => return result,
        };

        let guard = obj.read().unwrap();
        let instance = match guard.as_any().downcast_ref::<GenericInstanceData>() {
            Some(i) => i,
            None => return result,
        };

        for (i, name) in self.state_field_names.iter().enumerate() {
            if let Some(value) = instance.get_field(i) {
                result.insert(name.clone(), value.clone());
            }
        }

        result
    }

    /// Like [`read_all_state`], but materializes `Value::VmRef` list fields into
    /// inline `Value::Array` so snapshot/inspect tools (which only receive a
    /// `HashMap<String, Value>` with no VM heap access) can expand `for` loops
    /// and compare `.len()` against the actual element count.
    ///
    /// Plan 370 D-GAP-4: store fields like `notes` are produced by VM handlers
    /// (`list_notes()` → `db.all_notes()`) as `Value::VmRef` heap ids. Without
    /// materialization, the MCP snapshot's `for i, note in .store.notes` loop
    /// cannot expand (it sees a VmRef, not an Array), so the note list renders
    /// empty even though the data exists.
    ///
    /// Audit B10(a): the compiled `List<T>.new` / `[...]`-literal paths store
    /// the heap array id as a plain `Value::Int` (Plan 289 convention), which
    /// the VmRef-only materialization missed — autoui_state rendered 015's
    /// `notes: 4000014 (int)` instead of the array and desktop_mcp's element
    /// count assertions went blind. Probe Int values through the same
    /// `vmref_to_vec` heap lookup; a genuine Int either has no heap object or
    /// one that isn't a ListData, and stays untouched.
    pub fn read_all_state_materialized(&self) -> HashMap<String, Value> {
        let mut result = self.read_all_state();
        for (name, val) in result.iter_mut() {
            let handle = match val {
                Value::VmRef(r) => Some(r.id),
                Value::Int(id) if *id >= 4_000_000 => Some(*id as usize),
                _ => None,
            };
            if let Some(id) = handle {
                if let Ok(elems) = self.vmref_to_vec(id) {
                    *val = Value::Array(auto_val::Array { values: elems });
                }
            }
        }
        result
    }

    /// Plan 333: run the synthesized `__module_init` fn, which initializes
    /// imported module-level globals (`var notes = ...` etc.). Must be called
    /// once before `Init` (and before any handler that reads those globals).
    /// No-op (returns Ok) if the fn isn't present (no module-level stores).
    /// PLAN-702 T-02: 段派发（E1 证据链 :1114 调用点即此）——模块级 store
    /// 初始化若内含异步等待（api 拉数）即 park 入注册表、由
    /// `__parked_resume_tick` 泵续跑；同步初始化行为不变（Completed 等价）。
    pub fn run_module_init(&mut self) -> Result<()> {
        let fn_name = crate::ui::handler_codegen::MODULE_INIT_FN;
        if !self.vm.flash.exports_by_name.contains_key(fn_name) {
            return Ok(());
        }
        let mut task = AutoTask::new(0, 4096, 0);
        match self.vm.call_fn_by_name_segment(&mut task, fn_name, 0) {
            SegmentOutcome::Completed(res) => {
                res.map_err(|e| VmBridgeError::VmError(format!("{:?}", e)))
            }
            SegmentOutcome::Parked { wait, seg } => {
                self.register_parked(task, seg, wait, "__module_init".to_string(), false);
                Ok(())
            }
            SegmentOutcome::Runnable { .. } => {
                // PLAN-711 T-11: legacy 段入口不产生 Runnable（防御臂）。
                self.vm.rc_release_task_stack(&mut task);
                Err(VmBridgeError::VmError(
                    "internal: cpu-slice outcome escaped module init dispatch".into(),
                ))
            }
        }
    }

    /// PLAN-642 T-15: 本组件的视图侧 store 别名解析（`.Store.X` 展平判定）。
    /// 语义同 handler_codegen::view_store_alias_real_name，但读**本组件**
    /// 合成期存档的快照——多组件工程下线程级快照被后续合成覆盖，视图
    /// 求值若查全局会拿到他件的映射（015 "No notes yet"/013 footer 字面
    /// 模板实锤）。空表语义与全局版一致（无 store 工程不展平）。
    pub fn store_alias_real_name(&self, alias: &str) -> Option<String> {
        if let Some(real) = self.store_alias_snapshot.get(alias) {
            return Some(real.clone());
        }
        if !self.store_alias_snapshot.is_empty() && alias == "store" {
            return Some("store".to_string());
        }
        None
    }

    /// Call a handler by name with arguments.
    ///
    /// Looks up the synthesized `handler_<Name>` function in the module exports,
    /// pushes the state heap id as the first argument (`__state`) followed by the
    /// caller-supplied args, then dispatches via `call_fn_by_name`. Any state
    /// mutation the handler performs is written through to the state heap object
    /// in place, so a subsequent `read_state` reflects it.
    ///
    /// # Arguments
    ///
    /// * `event_name` - Handler name (e.g., "Inc", "Init", "PrevMonth")
    /// * `args` - Arguments to pass to the handler (after `__state`)
    ///
    /// # Errors
    ///
    /// Returns an error if the handler is not found or VM execution fails.
    /// UI should handle errors gracefully (log and continue).
    /// Get the state object heap ID (for event routing).
    pub fn state_obj_id(&self) -> u64 {
        self.state_obj_id
    }

    /// Plan 320: get a child widget's state object heap ID (if it exists).
    pub fn get_child_state_id(&self, widget_name: &str) -> Option<u64> {
        self.child_state_map.borrow().get(widget_name).copied()
    }

    /// Plan 320: ensure a child widget's state object exists on the VM heap,
    /// and update its prop fields. Returns the child state heap id.
    /// Called by render_child_widget (which no longer creates a new VM).
    /// Plan 320: write child widget's prop values directly into the ROOT state
    /// object. Since there is only ONE VM with ONE unified state, all widget
    /// fields (App's model + child props like `note`) live in the same
    /// GenericInstanceData. This returns the ROOT state_obj_id so child views
    /// also read from root state.
    pub fn ensure_child_state(
        &self,
        _widget_name: &str,
        _state_field_names: &[String],
        props: &std::collections::HashMap<String, auto_val::Value>,
    ) -> u64 {
        let root_id = self.state_obj_id;

        // Write prop values into the root state object (adding fields if missing).
        if let Some(obj) = self.vm.get_heap_object(root_id) {
            let mut guard = obj.write().unwrap();
            if let Some(inst) = guard.as_any_mut().downcast_mut::<GenericInstanceData>() {
                for (name, val) in props {
                    // Plan 057 (Bug 2): VM bytecode (GET_FIELD / index) expects
                    // array-typed state slots to hold a heap id (Int ≥4M →
                    // ListData), NOT a raw Value::Array — Value::Array can't
                    // nanobox, so bytecode reads popped double garbage
                    // ([GET_FIELD] non-i32 obj_id) and silently aborted the
                    // whole handler (ghost text 的 .history 读取即此)。
                    // Convert Array props to heap-stored ListData here.
                    let storable = match val {
                        auto_val::Value::Array(arr) => {
                            let id = self.vm.insert_heap_object(
                                crate::vm::types::ListData {
                                    elems: arr.values.clone(),
                                    storage: None,
                                },
                            );
                            // Plan 419: 字段持有列表引用。
                            self.vm.rc_retain_id(id);
                            auto_val::Value::Int(id as i32)
                        }
                        other => other.clone(),
                    };
                    if let Some(idx) = inst.field_names.iter().position(|n| n == name) {
                        // PLAN-045 T-02：prop 种子写入根态 = 状态面突变——但
                        // ensure_child_state 由 render_child_widget 每帧调用，
                        // 无条件 bump 会把"每帧 seq 必动"坐实（PLAN-062
                        // fire_timer 空转拍判定失效 + memo 快速路径恒 miss）。
                        // 值变化才计突变；其余状态面突变（SET_FIELD/handler/
                        // write_state）自有通道 bump，不变值重种子不掩盖。
                        let changed = inst.get_field(idx) != Some(&storable);
                        let _ = inst.set_field(idx, storable);
                        if changed {
                            // PLAN-047 T-03: A 类定点归因（值变才 bump 语义不变）。
                            self.vm.bump_path(self.state_obj_id, Some(&name));
                        }
                    } else {
                        // Add new field (prop not yet in root state).
                        inst.field_names.push(name.clone());
                        inst.fields.push(storable);
                        // PLAN-047 T-03: A 类定点归因（同上）。
                        self.vm.bump_path(self.state_obj_id, Some(&name));
                    }
                }
            }
        }

        // Return root state id — all reads/writes go to the unified state.
        root_id
    }

    /// PLAN-536 T3(题2 收敛): 子件 Init 挂载判定——该子件名首次出现返回
    /// true（调用方派发 Init 并记账）,此后每帧重渲染恒 false。挂载语义对齐
    /// vue onMounted（props 响应走 watch/computed,不靠 Init 重放）。
    /// os-config 016: 判定升级为**身份变化**语义——按组件名记录上次触发的
    /// 身份串（组件名 + 调用点 key prop）,身份与上次不同（含首挂）即触发
    /// 并更新。这是 vue 按 key 重挂载的 vm 对应物:同一子件切回先前的 key
    /// （侧栏 A→B→A）也要重挂载重 Init,只记"首次"会让回访页渲染陈旧数据。
    /// 每帧重渲染身份不变,恒 false——536 防重放语义不变。
    /// Plan 320: read a state field from a SPECIFIC child widget's state object
    /// (by heap id), not the root widget's state.
    pub fn read_child_state(&self, child_state_id: u64, field_name: &str) -> Result<auto_val::Value> {
        use crate::vm::generic_registry::GenericInstanceData;
        let obj = self.vm.get_heap_object(child_state_id)
            .ok_or_else(|| VmBridgeError::InvalidState(
                format!("child state heap object {} not found", child_state_id)
            ))?;
        let guard = obj.read().unwrap();
        let inst = guard.as_any().downcast_ref::<GenericInstanceData>()
            .ok_or_else(|| VmBridgeError::InvalidState("not GenericInstanceData".into()))?;
        let idx = inst.field_names.iter().position(|n| n == field_name)
            .ok_or_else(|| VmBridgeError::FieldNotFound(field_name.to_string()))?;
        inst.get_field(idx)
            .cloned()
            .ok_or_else(|| VmBridgeError::FieldNotFound(field_name.to_string()))
    }

    /// PLAN-721 T-4: 全子命名空间 dump（验收/诊断面）——child_state_map 逐
    /// 实例按**其自有 field_names** 展开（子件布局各异，根字段名槽位不适用；
    /// read_child_state 同款解析）。T-19 家族定案用：handler 作用域对象与
    /// 视图绑定对象是否同一（桌面 VM_EXEC 已见 4000002/4000003 分裂）。
    pub fn read_all_child_states(&self) -> Vec<(String, HashMap<String, auto_val::Value>)> {
        use crate::vm::generic_registry::GenericInstanceData;
        let map = self.child_state_map.borrow();
        let mut out: Vec<(String, HashMap<String, auto_val::Value>)> = Vec::new();
        for (name, id) in map.iter() {
            let Some(obj) = self.vm.get_heap_object(*id) else { continue };
            let guard = obj.read().unwrap();
            let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() else { continue };
            let mut fields = HashMap::new();
            for (i, fname) in inst.field_names.iter().enumerate() {
                if let Some(v) = inst.get_field(i) {
                    fields.insert(fname.clone(), v.clone());
                }
            }
            out.push((name.clone(), fields));
        }
        out
    }

    /// PLAN-721 T-4: root 字段持有的**可寻址实例对象**展开——`store` 等字段
    /// 值为 VmRef/堆 id 时，按目标对象自有 field_names dump 其全字段。
    /// T-19 定案用：root 的扁平镜像字段 vs store 实例对象字段是否分裂
    /// （`store.TogglePlay()` 是 VM 内方法调用，翻的是 store Obj；快照的
    /// read_all_state 只见 root 扁平面）。
    pub fn read_root_field_objects(&self) -> Vec<(String, u64, HashMap<String, auto_val::Value>)> {
        use crate::vm::generic_registry::GenericInstanceData;
        let mut out: Vec<(String, u64, HashMap<String, auto_val::Value>)> = Vec::new();
        let root = self.read_all_state();
        for (fname, val) in root.iter() {
            let id = match val {
                auto_val::Value::VmRef(r) => r.id as u64,
                auto_val::Value::Int(id) if *id >= 4_000_000 => *id as u64,
                _ => continue,
            };
            let Some(obj) = self.vm.get_heap_object(id) else { continue };
            let guard = obj.read().unwrap();
            let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() else { continue };
            let mut fields = HashMap::new();
            for (i, n) in inst.field_names.iter().enumerate() {
                if let Some(v) = inst.get_field(i) {
                    fields.insert(n.clone(), v.clone());
                }
            }
            out.push((fname.clone(), id, fields));
        }
        out
    }

    /// Plan 320: write child widget's prop values directly into the ROOT state
    /// object. Since all handlers run against root state (single VM, unified
    /// state), props must live in the same GenericInstanceData as the parent's
    /// model fields. This avoids "Field 'note' not found" when a child handler
    /// reads .note.title. Idempotent — if the field exists, updates it.
    pub fn sync_child_props_to_root(&self, _child_state_id: u64) {
        // No-op placeholder — actual syncing happens in ensure_child_state which
        // writes directly to root state. Kept for API compat.
    }
    pub fn read_child_state_as_vec(&self, child_state_id: u64, field_name: &str) -> Result<Vec<auto_val::Value>> {
        let val = self.read_child_state(child_state_id, field_name)?;
        match val {
            auto_val::Value::Array(arr) => Ok(arr.values),
            // Plan 390 §15 H3b: array literals are ListData<Value> in
            // heap_objects (probe instead of the old 2M id-range guard).
            auto_val::Value::Int(id) => {
                if let Some(obj) = self.vm.get_heap_object(id as u64) {
                    let guard = obj.read().unwrap();
                    use crate::vm::types::ListData;
                    if let Some(list) = guard.as_any().downcast_ref::<ListData<Value>>() {
                        Ok(list.elems.clone())
                    } else if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                        Ok(list.elems.iter().map(|i| auto_val::Value::Int(*i)).collect())
                    } else {
                        Err(VmBridgeError::InvalidState(
                            format!("array_id {} is not a readable list", id)
                        ))
                    }
                } else {
                    Err(VmBridgeError::InvalidState(
                        format!("array_id {} not found in heap_objects", id)
                    ))
                }
            }
            auto_val::Value::VmRef(r) => self.vmref_to_vec(r.id),
            other => Err(VmBridgeError::InvalidState(
                format!("Expected array for '{}', got {:?}", field_name, other)
            )),
        }
    }

    // ==========================================================================
    // PLAN-702 T-02/T-03/T-04: parked handler segments（段执行驱动）
    // ==========================================================================

    /// PLAN-702 T-02: whether any handler segment is parked awaiting resume
    /// (gates the renderer's `__parked_resume_tick` subscription — same
    /// conditional-subscription family as `has_pending_timers`).
    pub fn has_parked_tasks(&self) -> bool {
        !self.parked_tasks.borrow().is_empty()
    }

    /// PLAN-711 T-04 (D-2): 是否有 **I/O 凭据**的 parked 段——tick 泵的
    /// 订阅门。CPU continuation（CpuRunnable 凭据）不计入：tick 泵对其
    /// 恒不就绪，CPU-only 注册表不该吊着 16ms tick 空转（帧通知泵的门=
    /// `has_cpu_continuations` ∪ `has_pending_init_work`）。
    pub fn has_parked_io_tasks(&self) -> bool {
        self.parked_tasks
            .borrow()
            .iter()
            .any(|p| !matches!(p.wait, ParkedWait::CpuRunnable))
    }

    /// PLAN-702 T-02/T-04: parked 段计数（测试与诊断用；重入忽略断言面）。
    pub fn parked_count(&self) -> usize {
        self.parked_tasks.borrow().len()
    }

    /// PLAN-702 T-04: is a segment already parked for this resolved handler
    /// fn (the (widget, handler) reentry key — namespaced fn 名编码二者)?
    pub fn is_handler_parked(&self, fn_name: &str) -> bool {
        self.parked_tasks.borrow().iter().any(|p| p.fn_name == fn_name)
    }

    /// PLAN-702 T-03: probe every parked segment's wait and resume the ready
    /// ones. Resumes run serially inside the calling (iced update) context —
    /// 单 VM 串行一致性的执行点。Completed → 出清注册表；Completed(Err) →
    /// 出清并由调用方走既有 `[VM-HANDLER] ... failed` 通道（.at try/catch
    /// 已在段内经 intercept_error 捕获，未捕获才到这）；仍 Waiting → 再 park。
    pub fn resume_ready_parked(&self) -> ResumeReport {
        let mut report = ResumeReport::default();
        loop {
            let idx = {
                let parked = self.parked_tasks.borrow();
                match parked.iter().position(|p| self.parked_wait_ready(&p.wait)) {
                    Some(i) => i,
                    None => break,
                }
            };
            let mut p = self.parked_tasks.borrow_mut().remove(idx);
            match self.vm.resume_fn_by_name_segment(&mut p.task, &p.seg) {
                SegmentOutcome::Completed(Ok(())) => {
                    if p.release_stack_on_complete {
                        self.vm.rc_release_task_stack(&mut p.task);
                    }
                    eprintln!(
                        "[VM-PARKED] {} resumed to completion (waited {:?})",
                        p.fn_name,
                        p.parked_at.elapsed()
                    );
                    report.completed += 1;
                }
                SegmentOutcome::Completed(Err(e)) => {
                    if p.release_stack_on_complete {
                        self.vm.rc_release_task_stack(&mut p.task);
                    }
                    eprintln!("[VM-PARKED] {} resume FAILED: {:?}", p.fn_name, e);
                    report.failed.push((p.fn_name.clone(), format!("{:?}", e)));
                }
                SegmentOutcome::Parked { wait, .. } => {
                    // Still waiting (chained await / not-ready shim re-entry)
                    // — re-park with the refreshed credential.
                    p.wait = wait;
                    self.parked_tasks.borrow_mut().push(p);
                }
                SegmentOutcome::Runnable { .. } => {
                    // PLAN-711 T-11: legacy resume 不产生 Runnable（Legacy 预算
                    // 耗尽=真错误）；防御臂转挂 CpuRunnable 凭据交 CPU 泵接管，
                    // 不丢弃已存的执行栈份额。
                    p.wait = ParkedWait::CpuRunnable;
                    self.parked_tasks.borrow_mut().push(p);
                }
            }
        }
        self.sync_busy_flag();
        report
    }

    /// Readiness probe per wait credential.
    fn parked_wait_ready(&self, wait: &ParkedWait) -> bool {
        match wait {
            ParkedWait::HttpRequest(req_id) => {
                crate::vm::ffi::stdlib::async_http_result_ready(*req_id)
            }
            ParkedWait::Future(fid) => self
                .vm
                .futures
                .get(fid)
                .map(|f| f.read().unwrap().state != crate::vm::engine::FutureState::Pending)
                .unwrap_or(true), // future gone → wake (engine wake-source-6 parity)
            // PLAN-707 T-05: HTTP/SSE stream — queued item or terminal state.
            ParkedWait::HttpStream(stream_id) => {
                crate::vm::ffi::http_stream::stream_ready(*stream_id)
            }
            // PLAN-711 T-11 (D-2): CPU continuation 对 tick 泵**永不就绪**——
            // 其推进由 `resume_cpu_slices` 的轮次预算/公平策略裁决（帧通知泵
            // 的消费面，T-04 接线），不进 16ms tick 的无预算 drain（否则逐片
            // 续跑 × tick 频率 = 同轮自旋）。tick 泵的 scan 借此臂天然跳过。
            ParkedWait::CpuRunnable => false,
        }
    }

    /// PLAN-702 T-02: register a parked segment and raise the busy mirror.
    fn register_parked(
        &self,
        task: AutoTask,
        seg: ParkedSegment,
        wait: ParkedWait,
        event_name: String,
        release_stack_on_complete: bool,
    ) {
        eprintln!(
            "[VM-PARKED] {} parked (event {}, wait {:?}) — UI stays interactive",
            seg.fn_name, event_name, wait
        );
        let fn_name = seg.fn_name.clone();
        self.parked_tasks.borrow_mut().push(ParkedTask {
            task,
            seg,
            wait,
            fn_name,
            event_name,
            release_stack_on_complete,
            parked_at: std::time::Instant::now(),
        });
        self.sync_busy_flag();
    }

    /// PLAN-702 T-04: maintain the root-state `__busy_handlers` mirror — a
    /// List&lt;str&gt; heap id (.at reads it like any list field; MCP snapshots
    /// materialize it via read_all_state_materialized). Rewritten only when
    /// the parked-key set actually changes.
    fn sync_busy_flag(&self) {
        let mut names: Vec<String> = {
            let parked = self.parked_tasks.borrow();
            parked.iter().map(|p| p.fn_name.clone()).collect()
        };
        names.sort();
        if *self.busy_flag_names.borrow() == names {
            return;
        }
        *self.busy_flag_names.borrow_mut() = names.clone();

        let mut list = crate::vm::types::ListData::<auto_val::Value>::new();
        for n in &names {
            list.push(auto_val::Value::Str(auto_val::AutoStr::from(n.as_str())));
        }
        let id = self.vm.insert_heap_object(list);
        // PLAN-062 纪律：状态字段获得持有时显式 stake（write_state 对称面
        // —— Rust 直写绕过 VM 栈，rc.rs §2.3）。
        self.vm.rc_retain_id(id);
        let value = auto_val::Value::Int(id as i32);
        let Some(obj) = self.vm.get_heap_object_mut(self.state_obj_id) else {
            return;
        };
        let mut guard = obj.write().unwrap();
        let Some(inst) = guard.as_any_mut().downcast_mut::<GenericInstanceData>() else {
            return;
        };
        match inst.field_names.iter().position(|n| n == BUSY_STATE_FIELD) {
            Some(i) => {
                if let Some(old) = inst.get_field(i).cloned() {
                    self.release_state_value(&old);
                }
                let _ = inst.set_field(i, value);
            }
            None => {
                inst.field_names.push(BUSY_STATE_FIELD.to_string());
                inst.fields.push(value);
            }
        }
        // PLAN-045 T-02：`__busy_handlers` 镜像重写 = 状态面突变——memo
        // 快速路径可见性（函数体前段有"集合未变早退"，真写入才到此）。
        // PLAN-047 T-03: A+B 双归因——根态字段 exact（read_state 通道 dep）
        // + 新镜像堆列表 wildcard（read_state_as_vec 容器 dep）。
        self.vm.bump_path(self.state_obj_id, Some(BUSY_STATE_FIELD));
        self.vm.bump_path(id as u64, None);
    }

    // ==========================================================================
    // PLAN-711 T-11: CPU slice 有界泵与同 App 写事件队列（D-2）
    // ==========================================================================

    /// PLAN-711 T-11: 是否存在存活的 CPU continuation（帧通知泵的条件订阅
    /// 门控与 T-04 接线的 gate 面；与 `has_parked_tasks` 的 I/O 语义分开）。
    pub fn has_cpu_continuations(&self) -> bool {
        self.parked_tasks
            .borrow()
            .iter()
            .any(|p| matches!(p.wait, ParkedWait::CpuRunnable))
    }

    /// PLAN-711 T-11: CPU continuation 计数（诊断/测试/资源计数面）。
    pub fn cpu_continuation_count(&self) -> usize {
        self.parked_tasks
            .borrow()
            .iter()
            .filter(|p| matches!(p.wait, ParkedWait::CpuRunnable))
            .count()
    }

    /// PLAN-711 T-11: 排队写事件计数（诊断/测试/AC-11 队满观测面）。
    pub fn vm_write_queue_len(&self) -> usize {
        self.cpu_write_queue.borrow().len()
    }

    /// PLAN-711 T-11 (D-2): 同 App VM 写事件入队——CPU continuation 存活
    /// 期间的 input 代写/timer/props/MCP fixture/reload 统一走此入口（生产
    /// 者接线归 T-04）。可覆盖输入（`overwritable=true`）与队尾同键事件
    /// 合并（新值覆盖旧值）；满队列拒绝并返回
    /// [`VmBridgeError::WriteQueueFull`]（可观察 busy，不无限增容）。
    pub fn enqueue_vm_write(&self, ev: QueuedVmWrite) -> Result<()> {
        let mut q = self.cpu_write_queue.borrow_mut();
        if ev.overwritable {
            if let Some(last) = q.back_mut() {
                if last.widget_name == ev.widget_name
                    && last.event_name == ev.event_name
                    && last.overwritable
                {
                    *last = ev;
                    return Ok(());
                }
            }
        }
        if q.len() >= VM_WRITE_QUEUE_CAP {
            return Err(VmBridgeError::WriteQueueFull { len: q.len() });
        }
        q.push_back(ev);
        Ok(())
    }

    /// PLAN-711 T-11 (D-2): CPU slice 有界泵——CPU continuation 的唯一
    /// 消费点（tick 泵经 `parked_wait_ready` 的 CpuRunnable=false 臂天然
    /// 跳过；帧通知泵的接线归 T-04，消费的也是本方法）。
    ///
    /// 公平与预算（D-2 冻结纪律）：
    /// - 先整体摘出就绪集（CpuRunnable 凭据任务，FIFO 序），本轮**只处理
    ///   快照**——旧 tick 泵"取出-续跑-放回后再取出"的 drain 形态在此
    ///   不复制；本轮让出的任务放回注册表队尾，同轮不再重拾。
    /// - 每轮总预算 [`CPU_PUMP_ROUND_BUDGET`]（8ms）：超时即停，余下任务
    ///   原样放回下一轮继续。
    /// - 片间消费至多一条排队写事件（同 App 写序纪律：写事件只在泵上下文
    ///   串行落堆，不与在途 continuation 交错）；轮末兜底清剩余队列。
    pub fn resume_cpu_slices(
        &self,
        budget: crate::vm::engine::CpuSliceBudget,
    ) -> CpuPumpReport {
        let round_start = std::time::Instant::now();
        let mut report = CpuPumpReport::default();
        let mut ready: Vec<ParkedTask> = {
            let mut parked = self.parked_tasks.borrow_mut();
            let (cpu, rest): (Vec<_>, Vec<_>) = parked
                .drain(..)
                .partition(|p| matches!(p.wait, ParkedWait::CpuRunnable));
            *parked = rest;
            cpu
        };
        for p in ready.drain(..) {
            let mut p = p;
            if round_start.elapsed() >= crate::vm::engine::CPU_PUMP_ROUND_BUDGET {
                // 轮次预算耗尽：余下任务原样放回（仍 CpuRunnable，下一轮）。
                self.parked_tasks.borrow_mut().push(p);
                continue;
            }
            self.drain_one_queued_write(&mut report, budget);
            if round_start.elapsed() >= crate::vm::engine::CPU_PUMP_ROUND_BUDGET {
                self.parked_tasks.borrow_mut().push(p);
                continue;
            }
            report.slices_run += 1;
            match self.vm.resume_fn_by_name_cpu_slice(&mut p.task, &p.seg, budget) {
                SegmentOutcome::Completed(Ok(())) => {
                    if p.release_stack_on_complete {
                        self.vm.rc_release_task_stack(&mut p.task);
                    }
                    eprintln!(
                        "[VM-CPU] {} slice-resumed to completion (parked {:?}, {} total steps)",
                        p.fn_name, p.parked_at.elapsed(), p.task.cpu_steps_total
                    );
                    report.completed += 1;
                }
                SegmentOutcome::Completed(Err(e)) => {
                    if p.release_stack_on_complete {
                        self.vm.rc_release_task_stack(&mut p.task);
                    }
                    eprintln!("[VM-CPU] {} slice-resume FAILED: {:?}", p.fn_name, e);
                    report.failed.push((p.fn_name.clone(), format!("{:?}", e)));
                }
                SegmentOutcome::Runnable { seg } => {
                    // 本轮让出：放回队尾，同轮不重拾（快照外）。
                    p.seg = seg;
                    p.wait = ParkedWait::CpuRunnable;
                    self.parked_tasks.borrow_mut().push(p);
                }
                SegmentOutcome::Parked { wait, seg } => {
                    // continuation 期间遇到 I/O 等待——换凭据交 tick 泵接管
                    //（wait 集合只增不改 707 三凭据，D-3）。
                    p.seg = seg;
                    p.wait = wait;
                    self.parked_tasks.borrow_mut().push(p);
                }
            }
        }
        // 轮末兜底：清剩余排队写事件（continuation 已全部终态/让出后仍排队
        // 的写），仍受轮次预算约束。
        while !self.cpu_write_queue.borrow().is_empty() {
            if round_start.elapsed() >= crate::vm::engine::CPU_PUMP_ROUND_BUDGET {
                break;
            }
            if !self.drain_one_queued_write(&mut report, budget) {
                break;
            }
        }
        self.sync_busy_flag();
        report.round_elapsed = round_start.elapsed();
        report
    }

    /// PLAN-711 T-11: 消费一条排队写事件（经 CPU slice 入口派发——排队
    /// 的 handler 自身也可能长，同样受片预算约束）。返回是否消费了事件。
    fn drain_one_queued_write(
        &self,
        report: &mut CpuPumpReport,
        budget: crate::vm::engine::CpuSliceBudget,
    ) -> bool {
        let Some(ev) = self.cpu_write_queue.borrow_mut().pop_front() else {
            return false;
        };
        report.queue_drained += 1;
        report.slices_run += 1;
        if let Err(e) = self.call_handler_for_cpu_slice(
            &ev.widget_name,
            &ev.event_name,
            ev.state_obj_id,
            &ev.args,
            budget,
        ) {
            eprintln!(
                "[VM-CPU] queued write {}::{} FAILED: {:?}",
                ev.widget_name, ev.event_name, e
            );
            report.failed.push((
                format!("{}::{}", ev.widget_name, ev.event_name),
                format!("{:?}", e),
            ));
        }
        true
    }

    // ==========================================================================
    // PLAN-711 T-03 (M-01): Init demand 登记簿与代际生命周期
    // =========================================================================

    /// PLAN-711 T-03: 身份探测（**不写簿记、不消费**）——该 widget 最近
    /// 登记的身份是否与传入身份不同（或从未登记）。供 memo 路径在"身份
    /// 变化才准备 child state + 登记"的旁路判定用；真实登记仍走
    /// [`Self::register_init_demand`]。
    pub fn init_identity_changed(&self, widget_name: &str, identity: &str) -> bool {
        match self.init_demand_records.borrow().get(widget_name) {
            Some(rec) => rec.identity != identity,
            None => true,
        }
    }

    /// PLAN-711 T-03: 某 widget 的 demand 相位（诊断/测试/骨架态查询）。
    pub fn init_demand_phase(&self, widget_name: &str) -> Option<InitDemandPhase> {
        self.init_demand_records
            .borrow()
            .get(widget_name)
            .map(|r| r.phase)
    }

    /// PLAN-711 T-03 (M-01): 登记 Init demand——渲染路径的**唯一** Init 入口
    /// （替代 `child_init_should_fire` 判定即写身份 + 渲染内同步派发）。
    ///
    /// - 首次登记（或身份变化=新代际）：入队 FIFO，等待派发驱动；
    ///   身份变化时对旧代际做**一次取消**（未执行的丢弃；已 park 的段清栈/
    ///   等待凭据/忙态一次清理，见 [`Self::cancel_parked_by_fn`]）。
    /// - 同代际重复登记（显示/MCP 双 build、脏重建帧）：只确认簿记，
    ///   不二次入队（AC-04）。
    pub fn register_init_demand(
        &self,
        widget_name: &str,
        identity: &str,
        state_obj_id: u64,
        props: Vec<(String, auto_val::Value)>,
    ) -> InitDemandDecision {
        let mut records = self.init_demand_records.borrow_mut();
        let generation = self.init_generation.get();
        match records.get_mut(widget_name) {
            Some(rec) if rec.identity == identity => {
                // 同代际：Queued/InFlight 不重复入队；Done/Failed 终态同样
                // 不重派（Init 挂载语义每代际一次，PLAN-536 语义保持）。
                return match rec.phase {
                    InitDemandPhase::Queued | InitDemandPhase::InFlight => {
                        InitDemandDecision::InFlightKnown
                    }
                    InitDemandPhase::Done | InitDemandPhase::Failed => {
                        InitDemandDecision::AlreadyKnown
                    }
                };
            }
            Some(rec) => {
                // 身份变化 = 新代际：先取消旧代际（未执行丢弃/在途取消一次清理），
                // 再推进代际号登记新 demand（A→B→A 的第二个 A 由此重跑 Init）。
                let old_identity = rec.identity.clone();
                let had_pending = rec.phase == InitDemandPhase::Queued;
                let had_in_flight = rec.phase == InitDemandPhase::InFlight;
                rec.identity = identity.to_string();
                rec.phase = InitDemandPhase::Queued;
                rec.generation = generation + 1;
                self.init_generation.set(generation + 1);
                drop(records);
                if had_pending {
                    self.cancel_queued_init(&old_identity);
                }
                if had_in_flight {
                    let fn_name = crate::ui::handler_codegen::namespaced_handler_fn_name(
                        widget_name,
                        "Init",
                    );
                    self.cancel_parked_by_fn(&fn_name);
                }
            }
            None => {
                records.insert(
                    widget_name.to_string(),
                    InitDemandRecord {
                        identity: identity.to_string(),
                        phase: InitDemandPhase::Queued,
                        generation: generation + 1,
                    },
                );
                self.init_generation.set(generation + 1);
            }
        }
        self.init_demand_queue
            .borrow_mut()
            .push_back(InitDemand {
                widget_name: widget_name.to_string(),
                identity: identity.to_string(),
                state_obj_id,
                generation: self.init_generation.get(),
                props,
            });
        if std::env::var("AUTO_SCHED_DIAG").ok().as_deref() == Some("1") {
            let t0 = crate::ui::dynamic::sched_diag_t0();
            eprintln!(
                "[SCHED-DIAG] init_demand registered widget={} identity={} gen={} queue={} t={}ms",
                widget_name,
                identity,
                self.init_generation.get(),
                self.init_demand_queue.borrow().len(),
                t0.elapsed().as_millis(),
            );
        }
        InitDemandDecision::Queued
    }

    /// PLAN-711 T-03: 是否有未收敛的 Init 工作（排队 demand 或在途段）——
    /// 渲染订阅门与诊断用（`has_parked_tasks` 的 Init 扩展面）。
    pub fn has_pending_init_work(&self) -> bool {
        !self.init_demand_queue.borrow().is_empty()
            || self
                .init_demand_records
                .borrow()
                .values()
                .any(|r| matches!(r.phase, InitDemandPhase::Queued | InitDemandPhase::InFlight))
    }

    /// PLAN-711 T-03: 排队 demand 数（诊断/测试）。
    pub fn pending_init_demand_count(&self) -> usize {
        self.init_demand_queue.borrow().len()
    }

    /// PLAN-711 T-03 (M-01): Init demand 派发驱动——FIFO 消费登记簿，经
    /// [`Self::call_handler_for_cpu_slice`]（T-11 结果接口）派发并按五态
    /// 观察记账：
    /// - `Missing`（Init 未导出）：静默记 Done（正常程序契约）；
    /// - 派发 `Err`：记 Failed，错误走既有 `[VM-HANDLER]` 通道，不无限重试；
    /// - 同步 `Completed`：记 Done；
    /// - park（CPU Runnable / I/O Waiting）：记 InFlight 并**停止本轮后续
    ///   demand 派发**——同代际后继 demand（依赖父/页产物的 child）等前序
    ///   到终态后再派发（根/页 → child 依赖序，渲染登记序即 FIFO 序）。
    ///
    /// 在途段收敛后的再驱动：段完成会出 parked 注册表，本方法的 InFlight
    /// 探测（`is_handler_parked`）发现已不在册即记账 Done 并继续消费队列
    /// （调用方=泵臂，每轮 tick/帧通知都会重入本方法）。
    pub fn dispatch_pending_inits(
        &self,
        budget: crate::vm::engine::CpuSliceBudget,
    ) -> InitDispatchReport {
        let mut report = InitDispatchReport::default();
        loop {
            // 1. 先探测在途段是否已收敛（parked 出册=终态已发生）。
            let inflight_done = {
                let records = self.init_demand_records.borrow();
                records
                    .iter()
                    .find(|(_, r)| r.phase == InitDemandPhase::InFlight)
                    .map(|(w, _)| {
                        let fn_name = crate::ui::handler_codegen::namespaced_handler_fn_name(w, "Init");
                        (w.clone(), !self.is_handler_parked(&fn_name))
                    })
            };
            if let Some((widget, finished)) = inflight_done {
                if finished {
                    let mut records = self.init_demand_records.borrow_mut();
                    if let Some(rec) = records.get_mut(&widget) {
                        if rec.phase == InitDemandPhase::InFlight {
                            rec.phase = InitDemandPhase::Done;
                            // PLAN-712 r2 T-16：静默翻转入账——否则帧泵
                            // progressed=false 不置 dirty 不 bump epoch，
                            // outlet 永久停留「Loading… (页名)」占位
                            //（018 书架/详情实机实证）。
                            report.silent_done_transitions += 1;
                        }
                    }
                } else {
                    // 仍在途：本轮不再派发新 demand（依赖序）。
                    break;
                }
            }
            // 2. FIFO 派发下一条。
            let Some(demand) = self.init_demand_queue.borrow_mut().pop_front() else {
                break;
            };
            report.dispatched += 1;
            let fn_name = crate::ui::handler_codegen::namespaced_handler_fn_name(
                &demand.widget_name,
                "Init",
            );
            // PLAN-711 T-03: 派发时解析**当前** child state id——登记到派发
            // 之间可能发生重建（child state 对象更替/旧对象随帧账本释放），
            // 陈旧 id 的字段读取得空值（donut Init 除零实录：total=0）。
            // PLAN-711 Q-05 修订：改回**登记时 id 直用**（统一根态=root id）
            // ——child_state_map 条目可能是异构陈旧对象（bar 卡实录：读出
            // area 的 fields → Desktop/Mobile/Tablet 系列错谱，grouped 全套
            // tick 8000/4000/2000 消失）；配合 props 快照重播种即为完整
            // 旧"播种即派发"交错等价。
            let state_obj_id = demand.state_obj_id;
            if !self.vm.flash.exports_by_name.contains_key(&fn_name) {
                // Missing：声明了 lifecycle.Init 但未导出——静默（正常程序
                // 契约；异常导出缺失的显式失败面归 T-05 错误态）。
                let mut records = self.init_demand_records.borrow_mut();
                if let Some(rec) = records.get_mut(&demand.widget_name) {
                    rec.phase = InitDemandPhase::Done;
                    // PLAN-712 r2 T-16：Missing 翻转同样入账——「Loading…」
                    // 占位符需要一次重建才能清除（否则未导出 Init 的页面
                    // 永久停留占位符）。
                    report.silent_done_transitions += 1;
                }
                report.missing += 1;
                continue;
            }
            // PLAN-711 T-03: 派发前重播种登记时 props 快照（统一根态约束，
            // 见 InitDemand.props 注）——等价复刻旧同步路径"播种后立即派发"
            // 的交错语义；值变化才 bump（ensure_child_state 既有纪律）。
            if !demand.props.is_empty() {
                let props_map: std::collections::HashMap<String, auto_val::Value> =
                    demand.props.iter().cloned().collect();
                self.ensure_child_state(&demand.widget_name, &[], &props_map);
            }
            match self.call_handler_for_cpu_slice(
                &demand.widget_name,
                "Init",
                state_obj_id,
                &[],
                budget,
            ) {
                Ok(()) => {
                    // Ok = 段已接受（同步完成 / park 在册 / 重入忽略）。
                    if self.is_handler_parked(&fn_name) {
                        let mut records = self.init_demand_records.borrow_mut();
                        if let Some(rec) = records.get_mut(&demand.widget_name) {
                            rec.phase = InitDemandPhase::InFlight;
                        }
                        report.in_flight += 1;
                        break; // 依赖序：后继 demand 等本段终态
                    }
                    let mut records = self.init_demand_records.borrow_mut();
                    if let Some(rec) = records.get_mut(&demand.widget_name) {
                        rec.phase = InitDemandPhase::Done;
                    }
                    report.completed += 1;
                }
                Err(e) => {
                    let mut records = self.init_demand_records.borrow_mut();
                    if let Some(rec) = records.get_mut(&demand.widget_name) {
                        rec.phase = InitDemandPhase::Failed;
                    }
                    eprintln!(
                        "[VM-INIT] {} Init dispatch FAILED: {:?}",
                        demand.widget_name, e
                    );
                    report
                        .failed
                        .push((demand.widget_name.clone(), format!("{:?}", e)));
                }
            }
        }
        self.sync_busy_flag();
        report
    }

    /// PLAN-711 T-03: 丢弃指定身份的排队 demand（代际取消——未执行项直接
    /// 丢弃，无副作用可回滚）。
    fn cancel_queued_init(&self, identity: &str) {
        self.init_demand_queue
            .borrow_mut()
            .retain(|d| d.identity != identity);
    }

    /// PLAN-711 T-03 (M-01): 取消指定 fn 的 parked 段（代际取消——已开始项
    /// 清栈/等待凭据/忙态**一次清理**）。等待凭据逐种映射清理能力（D-3：
    /// 不能只删桥登记项放任生产者存活）：
    /// - `HttpRequest` → `drop_async_result`（PLAN-027 缺陷 A 纪律：迟到
    ///   完成的响应体必须回收）；
    /// - `HttpStream` → `stream_cancel`（上游流收口）；
    /// - `Future` → 无宿主资源可回收（唤醒源在引擎侧，future 槽随 task 弃）；
    /// - `CpuRunnable` → 纯栈份额，随清栈释放。
    /// 取消不回滚已发生副作用（前缀写入保留，M-01 契约）。
    pub fn cancel_parked_by_fn(&self, fn_name: &str) -> bool {
        let pos = self.parked_tasks.borrow().iter().position(|p| p.fn_name == fn_name);
        let Some(idx) = pos else {
            return false;
        };
        let mut p = self.parked_tasks.borrow_mut().remove(idx);
        match &p.wait {
            ParkedWait::HttpRequest(req_id) => {
                crate::vm::ffi::stdlib::drop_async_result(*req_id);
            }
            ParkedWait::HttpStream(stream_id) => {
                crate::vm::ffi::http_stream::stream_cancel(*stream_id);
            }
            ParkedWait::Future(_) | ParkedWait::CpuRunnable => {}
        }
        if p.release_stack_on_complete {
            self.vm.rc_release_task_stack(&mut p.task);
        }
        eprintln!("[VM-INIT] {} cancelled (generation superseded)", p.fn_name);
        self.sync_busy_flag();
        true
    }

    /// Call a handler by name with arguments.
    ///
    /// Looks up the synthesized `handler_<WidgetName>_<EventName>` function in
    /// the module exports (Plan 320: single-VM namespaced handlers). Pushes the
    /// state heap id as the first argument (`__state`) followed by the
    /// caller-supplied args, then dispatches via `call_fn_by_name`.
    ///
    /// # Arguments
    ///
    /// * `widget_name` - Widget that owns the handler (e.g. "App", "EditorPanel")
    /// * `event_name` - Handler name (e.g., "Inc", "Init", "Edit")
    /// * `args` - Arguments to pass to the handler (after `__state`)
    ///
    /// Plan 437 Phase 2: receiver relaxed to `&self` — AutoVM's call chain
    /// (rc_push_id / rc_push_str_idx / add_string / call_fn_by_name) is fully
    /// interior-mutable, and the render phase (AuraViewBuilder holds `&VmBridge`)
    /// needs to fire child-widget Init handlers after seeding props.
    /// PLAN-051 C3: 按名调用 VM 模块级函数并取返回值。use.web 引入的 helper
    /// 别名（computed 体里的链式纯 fn 调用，如 musk filteredMessages =>
    /// chatSearchFilter(chatActivePath(...), ...)）在 VM 的落点由此可达——
    /// 名字解析序：裸名 → import_aliases 限定名。返回值按栈顶 nanbox 解码
    /// （列表为 heap id Int ≥4M，由调用方 index_list_all 展开）。
    pub fn call_vm_fn(&self, name: &str, args: &[Value]) -> Result<Value> {
        let candidates = [Some(name.to_string()), self.import_aliases.get(name).cloned()];
        let fn_name = candidates
            .into_iter()
            .flatten()
            .find(|n| self.vm.flash.exports_by_name.contains_key(n))
            .ok_or_else(|| VmBridgeError::HandlerNotFound(name.to_string()))?;

        let mut task = AutoTask::new(0, 4096, 0);
        for a in args {
            match a {
                Value::Str(s) => {
                    let idx = self.vm.add_string(s.as_bytes().to_vec());
                    self.vm.rc_push_str_idx(&mut task, idx as usize);
                }
                // PLAN-051 C3: 堆引用实参必须按 tag 编码——push_value 对
                // VmRef/大 Int 落 push_i32(0) 占位(注释自述"not passed as
                // scalar args"),列表参数(≥4M ListData id)整个变 0。
                // PLAN-053 P-053-1: 且必须走 rc_push(+1 stake,rc.rs §2.3
                // 审计清单"不得绕过直接 push_nv")——原裸 push_nv 无 stake,
                // 被调 fn RET 弹栈释放时烧掉 state 持有的份额 → 列表对象
                // 回收成悬垂 id,computed 透传链每帧传死引用
                // (musk filteredMessages: [VM-IDX] id no-heap-object ×N,
                // 消息气泡整体空)。
                Value::Int(id) if *id >= 4_000_000 => {
                    self.vm.rc_push(&mut task, auto_val::encode_list(*id as u32));
                }
                Value::VmRef(r) if r.id >= 4_000_000 => {
                    self.vm.rc_push(&mut task, auto_val::encode_list(r.id as u32));
                }
                Value::VmRef(r) => {
                    self.vm.rc_push(&mut task, auto_val::encode_object(r.id as u32));
                }
                // PLAN-053 P-053-6: Obj/Array 实参物化——原落 push_value 的
                // push_i32(0) 占位，helper 收到的 msg 是 Int(0)，`.content`/
                // `.role` 等字段全读 0（musk 消息正文链 text 落 "0" 的终因）。
                // Obj → ObjectData 堆对象；Array → ListData<Value>。
                Value::Obj(o) => {
                    let mut od = crate::vm::types::ObjectData::new();
                    for (k, v) in o.iter() {
                        if let auto_val::Value::VmRef(r) = v {
                            self.vm.rc_retain_id(r.id as u64);
                        }
                        od.set(k.clone(), v.clone());
                    }
                    let id = self.vm.insert_heap_object(od) as u32;
                    self.vm.rc_push(&mut task, auto_val::encode_object(id));
                }
                Value::Array(arr) => {
                    let mut list = crate::vm::types::ListData::<auto_val::Value>::new();
                    for v in arr.iter() {
                        if let auto_val::Value::VmRef(r) = v {
                            self.vm.rc_retain_id(r.id as u64);
                        }
                        list.push(v.clone());
                    }
                    let id = self.vm.insert_heap_object(list) as u32;
                    self.vm.rc_push(&mut task, auto_val::encode_object(id));
                }
                other => push_value(&mut task.ram, other),
            }
        }
        // PLAN-702 T-05: legacy 同步驱动保留位①——view 侧 helper 求值契约
        // 是"本次调用返回 Value"，无 park 形态（视图每帧重算，parked 返回值
        // 无投递面）；视图 fn 含异步等待本属病理（纯度约束另立计划）。
        self.vm
            .call_fn_by_name(&mut task, &fn_name, args.len())
            .map_err(|e| VmBridgeError::VmError(format!("{:?} (crash ip=0x{:x} in {})", e, task.ip, fn_name)))?;
        let nv = task.ram.pop_nv();
        // PLAN-062 T12: 结果槽份额**接管**（取走不释放）——bp==0 主任务
        // RET 不做帧清扫,结果值压栈携带的份额直接转为宿主持有（避免
        // 释放→瞬时归零→retain 复活的 canary 竞态）;裸拷贝（无份额）由
        // takeover_heap_result 退回普通 retain。随后整栈按影子清账。
        let result_stake = task.ram.take_stake_at(task.ram.sp);
        self.vm.rc_release_task_stack(&mut task);
        // PLAN-053 P-053-6: 字符串结果必须解码为 Value::Str——
        // nv_to_pub_value 的 is_string 臂把字符串降格为池索引
        // Value::Int(idx)（低层裸约定），computed/prop 位置的字符串返回值
        // （musk msgTimeLabel/render_mentions_default/html 转义链）落到
        // builder 后被当作整数显示/判空，正文整体丢失。
        let out = self.decode_task_result_nv(nv);
        // PLAN-051 C3: 返回值为堆引用(ListData/VmRef)时 retain（裸拷贝
        // 形态）。PLAN-062 T12: result_stake 接管形态优先（VM 栈份额直接
        // 转为宿主持有,跳过 +1）。份额记入帧账本,由 commit_dirty_frame
        // 在下一脏帧换代配平释放。
        self.takeover_heap_result(&out, result_stake);
        if std::env::var("AUTO_DEBUG_EMIT").is_ok() {
            eprintln!("[VM-CALLFN] {} args={:?} -> {:?}", fn_name, args, out);
        }
        Ok(out)
    }

    /// Decode a popped return NV into a Rust Value with the P-053-6 string
    /// fix (string returns must not degrade to pool-index Ints).
    fn decode_task_result_nv(&self, nv: auto_val::NanoValue) -> Value {
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as u32;
            match self.vm.get_string(idx) {
                Some(bytes) => Value::Str(String::from_utf8_lossy(&bytes).into()),
                None => Value::Str(String::new().into()),
            }
        } else {
            nv_to_pub_value(nv)
        }
    }

    /// PLAN-051 C3: retain heap-referenced results (RET pops the caller's
    /// stake; an unretained id would be RC-freed into a dangling ref).
    /// PLAN-062 F2: retain 同时记入帧账本（cur），由 commit_dirty_frame
    /// 在下一脏帧换代时配平释放。
    fn retain_heap_result(&self, out: &Value) {
        match out {
            Value::Int(id) if *id >= 4_000_000 => {
                self.vm.rc_retain_id(*id as u64);
                self.record_frame_retain(*id as u64);
            }
            Value::VmRef(r) => {
                self.vm.rc_retain_id(r.id as u64);
                self.record_frame_retain(r.id as u64);
            }
            _ => {}
        }
    }

    /// PLAN-062 T12: 接管形态的 retain——result_stake 即该 id 时跳过 +1
    /// （份额从 VM 栈转移到宿主），只入帧账本；不匹配（裸拷贝/异值）时
    /// 退回普通 retain（并归还错配的接管份额）。
    fn takeover_heap_result(&self, out: &Value, result_stake: u64) {
        let id = match out {
            Value::Int(id) if *id >= 4_000_000 => *id as u64,
            Value::VmRef(r) => r.id as u64,
            _ => {
                if result_stake != 0 {
                    self.vm.rc_release_id(result_stake);
                }
                return;
            }
        };
        if result_stake == id {
            self.record_frame_retain(id);
        } else {
            if result_stake != 0 {
                self.vm.rc_release_id(result_stake);
            }
            self.vm.rc_retain_id(id);
            self.record_frame_retain(id);
        }
    }

    /// PLAN-062 F2: 帧账本记账（开关关时零开销直返）。
    fn record_frame_retain(&self, id: u64) {
        if !frame_rc_enabled() { return; }
        if let Ok(mut cur) = self.frame_retains_cur.lock() {
            cur.push(id);
        }
    }

    /// PLAN-062 F2: 脏帧换代提交——renderer dirty 分支"新缓存写回之后"
    /// 调用：prev 账本整体 `rc_release_id`（上一帧宿主份额归还），cur
    /// 内容晋升为 prev。时序保证帧 N 的 retain 结果在帧 N+1 构建全程
    /// 存活；旧缓存树与新账本同帧换代，无悬挂窗口。
    pub fn commit_dirty_frame(&self) {
        if !frame_rc_enabled() { return; }
        let released: Vec<u64> =
            match (self.frame_retains_cur.lock(), self.frame_retains_prev.lock()) {
                (Ok(mut cur), Ok(mut prev)) => {
                    std::mem::replace(&mut *prev, std::mem::take(&mut *cur))
                }
                _ => return,
            };
        for id in released {
            self.vm.rc_release_id(id);
        }
    }

    /// PLAN-062: 只读 VM 堆水位（泄漏 soak 断言通道——live_heap 以
    /// heap_objects.len() 为准，见 rc.rs rc_stats 注释）。
    /// PLAN-667 (F-02): rc_stats 已纯化；本通道为测试静止点观测，先显式
    /// drain dying 队列再读数（统计与回收动作分离；调用面全部为 soak
    /// 断言，无生产 per-frame 调用者）。
    pub fn heap_live_objects(&self) -> usize {
        self.vm.reap_all();
        self.vm.rc_stats().live_heap
    }

    /// PLAN-062: 状态突变序号只读透传（fire_timer 空转拍判定）。
    pub fn state_mutation_seq(&self) -> u64 {
        self.vm.state_mutation_seq()
    }

    /// PLAN-045: memo 缓存快照口（渲染期 builder 走 `&self`，RefCell 内可变）。
    #[cfg(feature = "ui-interpreter")]
    pub fn with_memo_cache<R>(&self, f: impl FnOnce(&mut crate::ui::memo_deps::MemoCache) -> R) -> R {
        f(&mut self.memo_cache.borrow_mut())
    }

    // ─────────────────────────────────────────────────────────────────
    // PLAN-047 T-01（档 C SD-08）: 依赖录制器
    // ─────────────────────────────────────────────────────────────────

    /// 开启依赖录制（返回 guard）。激活期内桥读通道把状态读记入新
    /// RecState；[`DepRecGuard::finish`] 或 drop 时恢复外层并把本次集
    /// **并集吸收**进外层（外层条目因此覆盖内层 computed/子求值的全部
    /// 依赖），overflow 同向传播——外层集不完整即整体弃用（宁缺勿错）。
    /// PLAN-047 T-04: 同时激活 AutoVM 录制槽——guard 作用域内的 VM fn
    /// 执行（block computed `call_computed_fn`/handler）读臂依赖由引擎
    /// 侧影子集承载，finish 时与桥侧集并集收编。
    #[cfg(feature = "ui-interpreter")]
    pub fn dep_recording_guard(&self) -> DepRecGuard<'_> {
        let outer = self
            .dep_recorder
            .borrow_mut()
            .replace(crate::ui::memo_deps::RecState::default());
        // 引擎影子槽与本 guard 生命周期同绑（同线程串行，无竞争窗口）。
        self.vm
            .set_dep_recorder(std::sync::Arc::new(std::sync::Mutex::new(
                crate::ui::memo_deps::RecState::default(),
            )));
        DepRecGuard {
            bridge: self,
            outer: outer.map(Box::new),
            done: false,
        }
    }

    /// 诊断/测试面：窥探当前录制状态快照（不改变激活态）。
    #[cfg(feature = "ui-interpreter")]
    pub fn peek_dep_recorder(&self) -> Option<crate::ui::memo_deps::RecState> {
        self.dep_recorder.borrow().clone()
    }

    /// 读通道录制：具名字段读（未激活 = 零开销分支直落）。
    #[cfg(feature = "ui-interpreter")]
    fn record_dep_read(&self, heap_id: u64, path: &str) {
        if let Some(rec) = self.dep_recorder.borrow_mut().as_mut() {
            rec.record(crate::ui::memo_deps::DepKey::field(heap_id, path));
        }
    }

    /// 读通道录制：容器/结构体整体展开（粗粒度 `DEP_PATH_ANY` 保守面）。
    #[cfg(feature = "ui-interpreter")]
    fn record_dep_any(&self, heap_id: u64) {
        if let Some(rec) = self.dep_recorder.borrow_mut().as_mut() {
            rec.record(crate::ui::memo_deps::DepKey::any(heap_id));
        }
    }

    /// 非 ui-interpreter 构建的零伤 stub（vm_bridge 模块本身不门控）。
    #[cfg(not(feature = "ui-interpreter"))]
    fn record_dep_read(&self, _heap_id: u64, _path: &str) {}

    /// 非 ui-interpreter 构建的零伤 stub。
    #[cfg(not(feature = "ui-interpreter"))]
    fn record_dep_any(&self, _heap_id: u64) {}

    /// 诊断/测试面：flash 导出名在册查询（block computed 合成通道预检）。
    pub fn vm_flash_exports_contains(&self, fn_name: &str) -> bool {
        self.vm.flash.exports_by_name.contains_key(fn_name)
    }

    // ─────────────────────────────────────────────────────────────────
    // PLAN-047 T-05（档 C SD-11）: version_fast 判定面
    // ─────────────────────────────────────────────────────────────────

    /// 动态 dep 集版本全同判定（check 三级判定的第二级）。全同 → 零重解析
    /// 命中（fill 后任何归因写都会前进对应 path 版本——A 类 exact+wildcard
    /// 双 bump / B 类 wildcard，见 SD-09；C 类（字符串池/对象出世）不动既有
    /// 内容，version_fast 命中安全）。
    #[cfg(feature = "ui-interpreter")]
    pub fn deps_unchanged(&self, pairs: &[(crate::ui::memo_deps::DepKey, u64)]) -> bool {
        pairs.iter().all(|(k, v0)| {
            // PLAN-706 r2 T-10: 死对象 dep 键 = 版本冻结键（path_versions
            // 条目留存而对象已摘除，永不 bump）→ 比对恒等伪命中 → 缓存值
            // 陈旧/悬垂（脏档切换弧 UAF/弹层滞留根因）。死键按 miss（重求
            // 值），宁缺勿错。
            if !self.vm.heap_dep_key_alive(k.heap_id) {
                return false;
            }
            self.vm.path_version(k.heap_id, &k.path) == *v0
        })
    }

    /// 录制集 → (dep, 当前版本) 基线对（fill 入条目 / fp_slow 命中刷新）。
    #[cfg(feature = "ui-interpreter")]
    pub fn dep_pairs(
        &self,
        deps: &std::collections::BTreeSet<crate::ui::memo_deps::DepKey>,
    ) -> Vec<(crate::ui::memo_deps::DepKey, u64)> {
        deps.iter()
            .map(|k| {
                let v = self.vm.path_version(k.heap_id, &k.path);
                (k.clone(), v)
            })
            .collect()
    }

    /// PLAN-047 T-05: 值承载的堆身份录制（keyed-for iterable 注入面——
    /// VmRef/Int≥4M → any 粗粒度；其余值无堆身份不录）。
    #[cfg(feature = "ui-interpreter")]
    pub fn record_dep_value_heap(&self, v: &Value) {
        match v {
            Value::VmRef(r) => self.record_dep_any(r.id as u64),
            Value::Int(i) if *i >= 4_000_000 => self.record_dep_any(*i as u64),
            _ => {}
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // PLAN-047 T-06（档 C SD-10）: computed 信号网
    // ─────────────────────────────────────────────────────────────────

    /// computed 信号命中查询（deps 版本全同 → 值缓存复用）。命中时把信号
    /// 节点的 dep 键**吸收进当前录制域**（嵌套 guard 并集）——外层 memo
    /// 条目/外层信号因此覆盖内层信号的失效面（computed 嵌套的 pull 式级
    /// 联闭合：内层 deps 变 → 内层重算 → 外层条目版本比对失效 → 重渲染）。    #[cfg(feature = "ui-interpreter")]
    pub fn computed_signal_hit(&self, widget: &str, prop: &str) -> Option<Value> {
        let hit = {
            let signals = self.computed_signals.borrow();
            signals
                .get(&(widget.to_string(), prop.to_string()))
                .and_then(|sig| {
                    self.deps_unchanged(&sig.deps)
                        .then(|| {
                            if let Some(rec) = self.dep_recorder.borrow_mut().as_mut() {
                                for (k, _) in &sig.deps {
                                    rec.record(k.clone());
                                }
                            }
                            sig.cached.clone()
                        })
                })
        };
        if hit.is_some() {
            self.signal_hits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.signal_misses
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        hit
    }

    /// computed 信号入库（求值期录制集 → 基线对；空集/超预算不入网——
    /// 盲区面退回每帧重算，同档 A/B 行为）。
    ///
    /// PLAN-706 r2 T-10 回归守卫：缓存值**不得携带堆身份**（`VmRef` /
    /// ≥4M 整数约定形 / `ValueRef` / 不可证净的复合变体）。`Value` 的克隆
    /// 是裸 id 复制、零 RC 份额——状态替换（如 tab 切换的 doc/body 交换）
    /// 释放旧堆对象后，信号命中把悬垂 id 送进渲染（`vmref_to_vec` →
    /// `get_heap_object`），debug 被 RC canary 判 UAF panic（jade-edit
    /// vm_matrix [7 tab] 标脏弧 exit=101），release 无 canary 落 id 复用
    /// 静默错读（确认弹层滞留面）。不入网 = 退回每帧重算（pre-706 行为），
    /// 与空集/超预算退档同族。
    #[cfg(feature = "ui-interpreter")]
    pub fn computed_signal_store(
        &self,
        widget: &str,
        prop: &str,
        cached: Value,
        rec: &crate::ui::memo_deps::RecState,
    ) {
        if rec.overflow || rec.deps.is_empty() {
            return;
        }
        if value_carries_heap_identity(&cached) {
            return;
        }
        let deps = self.dep_pairs(&rec.deps);
        self.computed_signals
            .borrow_mut()
            .insert((widget.to_string(), prop.to_string()), ComputedSignal { cached, deps });
    }

    /// PLAN-045: 指纹展开器——堆引用展开一层为纯值（memo 值指纹用）。
    /// ObjectData/GenericInstanceData → `Value::Obj`（materialize 同款）；
    /// ListData → `Value::Array`（materialize 不覆盖的容器面——内容原地
    /// 突变只有展开内容才检得出）；其余原样返回（指纹器按含 id Debug 兜底）。
    pub fn expand_heap_for_fingerprint(&self, v: &Value) -> Value {
        let (id, via_vmref) = match v {
            Value::Int(i) if *i >= 4_000_000 => (*i as u64, false),
            Value::VmRef(r) => (r.id as u64, true),
            _ => return v.clone(),
        };
        let Some(obj) = self.vm.get_heap_object(id) else {
            return v.clone();
        };
        let guard = obj.read().unwrap();
        if let Some(od) = guard.as_any().downcast_ref::<crate::vm::types::ObjectData>() {
            let mut out = auto_val::Obj::new();
            for (key, val) in od.fields.iter() {
                if let auto_val::ValueKey::Str(s) = key {
                    out.set(s.clone(), val.clone());
                }
            }
            return Value::Obj(Box::new(out));
        }
        if let Some(inst) = guard.as_any().downcast_ref::<crate::vm::generic_registry::GenericInstanceData>() {
            let mut out = auto_val::Obj::new();
            for (val, name) in inst.fields.iter().zip(inst.field_names.iter()) {
                if name != "_unknown" {
                    out.set(name.clone(), val.clone());
                }
            }
            return Value::Obj(Box::new(out));
        }
        if let Some(list) = guard.as_any().downcast_ref::<crate::vm::types::ListData<Value>>() {
            return Value::Array(auto_val::Array {
                values: list.elems.clone(),
            });
        }
        let _ = via_vmref;
        v.clone()
    }

    /// Plan 448 H2: execute a block-bodied computed's hidden fn
    /// (`__computed_<Widget>_<Prop>`) against the ROOT state object — the
    /// same receiver convention handler dispatch uses. The fn is synthesized
    /// alongside handlers in `synthesize_widget_module` /
    /// `synthesize_from_decl`; statement semantics (let scoping, sequencing)
    /// come from the VM itself.
    pub fn call_computed_fn(&self, widget_name: &str, computed_name: &str) -> Result<Value> {
        let fn_name = crate::ui::handler_codegen::computed_fn_name(widget_name, computed_name);
        if !self.vm.flash.exports_by_name.contains_key(&fn_name) {
            return Err(VmBridgeError::HandlerNotFound(fn_name));
        }
        let mut task = AutoTask::new(0, 4096, 0);
        self.vm.rc_push_id(&mut task, self.state_obj_id()); // Plan 419
        // PLAN-702 T-05: legacy 同步驱动保留位②——block computed 求值（同
        // call_vm_fn：本次调用返回 Value 契约，无 park 形态）。
        self.vm
            .call_fn_by_name(&mut task, &fn_name, 1)
            .map_err(|e| VmBridgeError::VmError(format!("{:?} (crash ip=0x{:x} in {})", e, task.ip, fn_name)))?;
        let nv = task.ram.pop_nv();
        // PLAN-062 T12: 同 call_vm_fn——结果槽份额接管（取走不释放）。
        let result_stake = task.ram.take_stake_at(task.ram.sp);
        self.vm.rc_release_task_stack(&mut task);
        let out = self.decode_task_result_nv(nv);
        // PLAN-062 T12: 同 call_vm_fn——接管形态 retain。
        self.takeover_heap_result(&out, result_stake);
        Ok(out)
    }

    pub fn call_handler_for(&self, widget_name: &str, event_name: &str, state_obj_id: u64, args: &[Value]) -> Result<()> {
        let Some((fn_name, mut task)) =
            self.prepare_handler_dispatch(widget_name, event_name, state_obj_id, args)?
        else {
            // parked 段在途重入——702 T-04 契约：静默忽略（Ok）。
            return Ok(());
        };

        // Plan 446 批一 (F1): 失败时带上崩点 ip + handler 名 —— VMError 本身
        // 无位置信息,task.ip 在 Err 返回后指向失败指令附近。
        // PLAN-702 T-02: 段派发——Yield+等待即 park（task 入注册表、零忙等，
        // UI 立即恢复事件循环）；从不 yield 的 handler 在本次 update 内跑完，
        // 与同步驱动逐字节同（兼容性论证见计划 §架构方案 4）。
        match self.vm.call_fn_by_name_segment(&mut task, &fn_name, 1 + args.len()) {
            SegmentOutcome::Completed(res) => {
                // PLAN-062 F2 配套: 主任务边界 RET 无帧清扫(见 call_vm_fn 注)——
                // 任务弃前整栈清账。Err 路径同样清(局部/临时槽可能已持 stake,
                // 不清则崩掉的 handler 额外漏一份)。
                self.vm.rc_release_task_stack(&mut task);
                res.map_err(|e| {
                    VmBridgeError::VmError(format!("{:?} (crash ip=0x{:x} in {})", e, task.ip, fn_name))
                })
            }
            SegmentOutcome::Parked { wait, seg } => {
                // parked 段的栈份额随 task 存活——不得中途清账（T-02）。
                self.register_parked(task, seg, wait, event_name.to_string(), true);
                Ok(())
            }
            SegmentOutcome::Runnable { .. } => {
                // PLAN-711 T-11: legacy 段入口不产生 Runnable（防御臂）；真
                // slice 入口见 [`Self::call_handler_for_cpu_slice`]。
                self.vm.rc_release_task_stack(&mut task);
                Err(VmBridgeError::VmError(format!(
                    "internal: cpu-slice outcome escaped legacy handler dispatch in {}",
                    fn_name
                )))
            }
        }
    }

    /// PLAN-711 T-11 (M-02/D-2): UI 专用 CPU slice 派发入口——与
    /// [`Self::call_handler_for`] 同一套导出检查/重入忽略/入参纪律，但驱动
    /// 预算为 [`DriveBudget::CpuSlice`]：片耗尽挂 `ParkedWait::CpuRunnable`
    /// 凭据入注册表（CPU 泵的下一轮在轮次预算内续跑），而非同步占满一次
    /// update。长 Init/handler 的调用方迁移到此入口（T-03/T-04 接线）。
    pub fn call_handler_for_cpu_slice(
        &self,
        widget_name: &str,
        event_name: &str,
        state_obj_id: u64,
        args: &[Value],
        budget: crate::vm::engine::CpuSliceBudget,
    ) -> Result<()> {
        let Some((fn_name, mut task)) =
            self.prepare_handler_dispatch(widget_name, event_name, state_obj_id, args)?
        else {
            // parked 段在途重入——702 T-04 契约：静默忽略（Ok）。
            return Ok(());
        };
        match self
            .vm
            .call_fn_by_name_cpu_slice(&mut task, &fn_name, 1 + args.len(), budget)
        {
            SegmentOutcome::Completed(res) => {
                self.vm.rc_release_task_stack(&mut task);
                res.map_err(|e| {
                    VmBridgeError::VmError(format!("{:?} (crash ip=0x{:x} in {})", e, task.ip, fn_name))
                })
            }
            SegmentOutcome::Parked { wait, seg } => {
                self.register_parked(task, seg, wait, event_name.to_string(), true);
                Ok(())
            }
            SegmentOutcome::Runnable { seg } => {
                // 片耗尽：栈份额随 task 存活（不得中途清账），凭据
                // CpuRunnable——tick 泵不拾取，`resume_cpu_slices` 消费。
                self.register_parked(
                    task,
                    seg,
                    ParkedWait::CpuRunnable,
                    event_name.to_string(),
                    true,
                );
                Ok(())
            }
        }
    }

    /// PLAN-711 T-11: `call_handler_for` 与 slice 入口的共享派发准备——
    /// 导出检查（`__` 探测事件缺失静默降 debug）、parked 在途重入忽略、
    /// 实参入栈（PLAN-053 P-053-8 双 canary 纪律原样）。`Ok(None)` =
    /// 重入被静默忽略（702 T-04 契约，调用方直接返回 `Ok(())`）；其余
    /// 返回 namespaced fn 名与已备好实参的 task。
    fn prepare_handler_dispatch(
        &self,
        widget_name: &str,
        event_name: &str,
        state_obj_id: u64,
        args: &[Value],
    ) -> Result<Option<(String, AutoTask)>> {
        let fn_name = crate::ui::handler_codegen::namespaced_handler_fn_name(widget_name, event_name);
        if !event_name.starts_with("__") {
            if crate::is_vm_hot_trace() {
                eprintln!("[VM_EXEC] fn_name={} state_obj_id={} args={:?}", fn_name, state_obj_id, args);
            }
        }

        // Verify the handler is exported before setting up a call frame.
        if !self.vm.flash.exports_by_name.contains_key(&fn_name) {
            // PLAN-041 T-12：框架探测事件（`__` 前缀）按契约可选——缺失
            // 静默降 debug（否则 __mcp_heartbeat/__scroll_state_read 每
            // 2s 单行刷屏，75s 空闲 28 行实录）；用户代码 handler 原样。
            if event_name.starts_with("__") {
                log::debug!(
                    "[CALL_HANDLER_FOR_NOT_FOUND] fn_name={} not in exports (framework probe)",
                    fn_name
                );
            } else {
                eprintln!("[CALL_HANDLER_FOR_NOT_FOUND] fn_name={} not in exports", fn_name);
            }
            return Err(VmBridgeError::HandlerNotFound(format!("{}.{}", widget_name, event_name)));
        }

        // PLAN-702 T-04: parked 段在途的重入默认忽略（队列/合并策略留待
        // 使用反馈）；`__busy_handlers` 镜像已在 park 时置位，.at 可查询。
        if self.is_handler_parked(&fn_name) {
            eprintln!("[VM-PARKED] {} re-entry ignored (segment in flight)", fn_name);
            return Ok(None);
        }

        let mut task = AutoTask::new(0, 4096, 0);
        self.vm.rc_push_id(&mut task, state_obj_id); // Plan 419
        for a in args {
            if let Value::Str(s) = a {
                // Plan 419/池去重一致性:走 add_string(原为裸 push,池漂移源)。
                let idx = self.vm.add_string(s.as_bytes().to_vec());
                // PLAN-053 P-053-8 canary①: add_string 返回槽的内容必须与
                // 实参一致——dedup 残键/槽复用竞态会让它返回它串的槽（现场：
                // 点击实参 id 变会话名"你好"，UI 层与 VM_HANDLER_CALL 日志均
                // 正确、handler 体内首读即漂移）。触发即坐实内化层腐坏。
                let slot = self.vm.get_string(idx as u32);
                if slot.as_deref() != Some(s.as_bytes()) {
                    eprintln!(
                        "[P053-8] intern drift: arg {:?} -> idx {} content {:?}",
                        s.as_str(),
                        idx,
                        slot.map(|b| String::from_utf8_lossy(&b).to_string())
                    );
                }
                self.vm.rc_push_str_idx(&mut task, idx as usize);
                // PLAN-053 P-053-8 canary②: 入栈后栈顶 NV 解码回的索引及
                // 槽内容必须仍与实参一致——排除 push/编码环节。
                let nv = task.ram.peek_nv(0);
                let pushed_idx = auto_val::decode_string(nv) as u32;
                if pushed_idx as usize != idx {
                    eprintln!(
                        "[P053-8] push drift: arg {:?} idx {} -> pushed idx {}",
                        s.as_str(), idx, pushed_idx
                    );
                }
            } else {
                push_value(&mut task.ram, a);
            }
        }
        Ok(Some((fn_name, task)))
    }

    /// Plan 442 A5: fire every due one-shot timer (set_timeout). Event-form
    /// callbacks dispatch like a UI event on the root widget; closure-form
    /// callbacks run via call_closure on a fresh task (by-value captures only
    /// — by-reference captures read the long-gone creator frame). Returns
    /// the number of callbacks fired; errors are logged, not fatal (a bad
    /// timer must not take down the render loop).
    pub fn poll_timers(&mut self) -> usize {
        let due = self.vm.due_timers();
        let mut fired = 0usize;
        for (_id, callback) in due {
            let result = match callback {
                crate::vm::engine::TimerCallback::Event(event) => {
                    self.call_handler(&event, &[])
                }
                crate::vm::engine::TimerCallback::Closure(closure_id) => {
                    let mut task = AutoTask::new(0, 4096, 0);
                    // call_closure pops closure_id + arg_count args off the stack.
                    task.ram.push_i32(closure_id as i32);
                    self.vm
                        .call_closure(&mut task, closure_id, 0)
                        .map_err(|e| VmBridgeError::VmError(format!("{:?}", e)))
                }
            };
            match result {
                Ok(()) => fired += 1,
                Err(e) => {
                    log::warn!("timer callback failed: {:?}", e);
                }
            }
        }
        fired
    }

    /// Plan 442 A5: whether any one-shot timer is pending (the iced render
    /// loop gates its timer-tick subscription on this).
    pub fn has_pending_timers(&self) -> bool {
        self.vm.has_pending_timers()
    }

    /// Plan 488：宿主注入 Record 载荷的 handler 调用。VM 侧记录是堆
    /// ObjectData（栈上以 id 引用，非内联 Value::Obj——后者字段访问产出
    /// 垃圾值），与 485 clipboard_image_get 返回路径同型；嵌套记录
    /// （如 drop payload 的 image）递归入堆后以 id 引用。
    pub fn call_handler_with_record(
        &mut self,
        event_name: &str,
        fields: Vec<(String, RecordValue)>,
    ) -> Result<()> {
        let fn_name = format!("handler_{}", extract_handler_name(event_name));
        let namespaced = crate::ui::handler_codegen::namespaced_handler_fn_name(
            &self.widget_name,
            event_name,
        );
        let fn_name = if self.vm.flash.exports_by_name.contains_key(&namespaced) {
            namespaced
        } else if self.vm.flash.exports_by_name.contains_key(&fn_name) {
            fn_name
        } else {
            return Err(VmBridgeError::HandlerNotFound(event_name.to_string()));
        };

        let rec_id = self.build_heap_record(fields);
        let mut task = AutoTask::new(0, 4096, 0);
        self.vm.rc_push_id(&mut task, self.state_obj_id); // Plan 419
        self.vm.rc_push_id(&mut task, rec_id);
        // PLAN-702 T-02: 段派发（同 call_handler_for；宿主注入载荷的
        // handler 同样可能内部等待）。
        match self.vm.call_fn_by_name_segment(&mut task, &fn_name, 2) {
            SegmentOutcome::Completed(res) => res.map_err(|e| {
                VmBridgeError::VmError(format!(
                    "{:?} (crash ip=0x{:x} in {})",
                    e, task.ip, fn_name
                ))
            }),
            SegmentOutcome::Parked { wait, seg } => {
                self.register_parked(task, seg, wait, event_name.to_string(), false);
                Ok(())
            }
            SegmentOutcome::Runnable { .. } => {
                // PLAN-711 T-11: legacy 段入口不产生 Runnable（防御臂）。
                self.vm.rc_release_task_stack(&mut task);
                Err(VmBridgeError::VmError(format!(
                    "internal: cpu-slice outcome escaped legacy dispatch in {}",
                    fn_name
                )))
            }
        }
    }

    /// [`RecordValue`] 递归入堆：嵌套记录先铸（子先父后），字段存真
    /// Value（字符串池内化只用于栈编码，字段存储不适用）。
    fn build_heap_record(&mut self, fields: Vec<(String, RecordValue)>) -> u64 {
        use auto_val::{AutoStr, ValueKey};
        let mut rec = crate::vm::types::ObjectData::new();
        for (k, v) in fields {
            let value = match v {
                RecordValue::Str(s) => Value::Str(AutoStr::from(s.as_str())),
                RecordValue::Int(i) => Value::Int(i),
                RecordValue::Bool(b) => Value::Bool(b),
                RecordValue::StrList(items) => {
                    // .at 列表 = 堆 ListData（inline Value::Array 的 len/join
                    // 在 VM 侧产出错误码——043 实证 heap 形态）。
                    let list = crate::vm::types::ListData {
                        elems: items.clone(),
                        ..Default::default()
                    };
                    let id = self.vm.insert_heap_object(list);
                    Value::Int(id as i32)
                }
                RecordValue::Record(nested) => Value::Int(self.build_heap_record(nested) as i32),
                RecordValue::Null => Value::Null,
            };
            rec.set(ValueKey::Str(AutoStr::from(k.as_str())), value);
        }
        self.vm.insert_heap_object(rec)
    }

    /// Legacy call_handler — calls handler on the ROOT widget (widget_name
    /// defaults to this bridge's widget name). Uses self.state_obj_id.
    pub fn call_handler(&mut self, event_name: &str, args: &[Value]) -> Result<()> {
        let fn_name = format!("handler_{}", extract_handler_name(event_name));

        // Plan 320: try namespaced first (handler_<WidgetName>_<Event>),
        // then fall back to legacy (handler_<Event>) for backward compat.
        let namespaced = crate::ui::handler_codegen::namespaced_handler_fn_name(&self.widget_name, event_name);
        let fn_name = if self.vm.flash.exports_by_name.contains_key(&namespaced) {
            namespaced
        } else if self.vm.flash.exports_by_name.contains_key(&fn_name) {
            fn_name
        } else {
            return Err(VmBridgeError::HandlerNotFound(event_name.to_string()));
        };

        eprintln!("[CALL_HANDLER] widget={} event_name={} fn_name={} args={:?}", self.widget_name, event_name, fn_name, args);

        // PLAN-702 T-04: parked 段在途的重入默认忽略（同 call_handler_for）。
        if self.is_handler_parked(&fn_name) {
            eprintln!("[VM-PARKED] {} re-entry ignored (segment in flight)", fn_name);
            return Ok(());
        }

        let mut task = AutoTask::new(0, 4096, 0);

        // Push arguments left-to-right: __state (the state heap id) first, then
        // the handler's declared params. `call_fn_by_name` then sets up the call
        // frame; params are accessed as bp-(n_args+1) .. bp-2 (see LOAD_LOCAL).
        self.vm.rc_push_id(&mut task, self.state_obj_id); // Plan 419
        for a in args {
            // Strings must be interned into the VM strings pool and pushed as
            // their tagged index (the same encoding LOAD_STR / GET_FIELD use),
            // otherwise a payload like `.SelectDay(cell.date)` arrives as 0.
            if let Value::Str(s) = a {
                // Plan 419/池去重一致性:走 add_string(原为裸 push,池漂移源)。
                let idx = self.vm.add_string(s.as_bytes().to_vec());
                self.vm.rc_push_str_idx(&mut task, idx as usize);
            } else {
                push_value(&mut task.ram, a);
            }
        }

        // Plan 446 批一 (F1): 失败时带上崩点 ip + handler 名 —— VMError 本身
        // 无位置信息,task.ip 在 Err 返回后指向失败指令附近。
        // PLAN-702 T-02: 段派发（同 call_handler_for；本入口的弃栈行为
        // 保持既有纪律——Completed 也不 rc_release_task_stack）。
        match self.vm.call_fn_by_name_segment(&mut task, &fn_name, 1 + args.len()) {
            SegmentOutcome::Completed(res) => res.map_err(|e| {
                eprintln!("[CALL_HANDLER_ERR] {} error: {:?} at ip=0x{:x}", fn_name, e, task.ip);
                eprintln!("[CALL_HANDLER_ERR] exports: {:?}", self.vm.flash.exports_by_name.keys().collect::<Vec<_>>());
                let start = task.ip.saturating_sub(40);
                let end = (task.ip + 40).min(self.vm.flash.memory.len());
                eprintln!("[CALL_HANDLER_ERR] code around ip (0x{:x}..0x{:x}): {:?}", start, end, &self.vm.flash.memory[start..end]);
                VmBridgeError::VmError(format!("{:?} (crash ip=0x{:x} in {})", e, task.ip, fn_name))
            }),
            SegmentOutcome::Parked { wait, seg } => {
                self.register_parked(task, seg, wait, event_name.to_string(), false);
                Ok(())
            }
            SegmentOutcome::Runnable { .. } => {
                // PLAN-711 T-11: legacy 段入口不产生 Runnable（防御臂；
                // 本入口弃栈行为保持既有纪律）。
                Err(VmBridgeError::VmError(format!(
                    "internal: cpu-slice outcome escaped legacy call_handler in {}",
                    fn_name
                )))
            }
        }
    }

    /// Get the widget name.
    pub fn widget_name(&self) -> &str {
        &self.widget_name
    }

    /// Get the state field names.
    pub fn state_fields(&self) -> &[String] {
        &self.state_field_names
    }

    /// Get a reference to the underlying AutoVM (for advanced VM integration).
    pub fn vm(&self) -> &AutoVM {
        &self.vm
    }

    /// Get a mutable reference to the underlying AutoVM.
    pub fn vm_mut(&mut self) -> &mut AutoVM {
        &mut self.vm
    }

    /// Check if a handler exists for the given event name.
    ///
    /// A handler exists iff a synthesized handler function is present in the
    /// module exports. Plan 320 synthesizes handlers under *namespaced* names
    /// (`handler_<WidgetName>_<Event>`) so multiple widgets coexist in one VM;
    /// legacy non-namespaced names (`handler_<Event>`) are kept as a fallback.
    /// This must mirror [`call_handler`]'s lookup, otherwise `has_handler`
    /// returns false for handlers that `call_handler` would dispatch fine.
    pub fn has_handler(&self, event_name: &str) -> bool {
        // Plan 320: try namespaced first (handler_<WidgetName>_<Event>),
        // then fall back to legacy (handler_<Event>) for backward compat.
        let namespaced =
            crate::ui::handler_codegen::namespaced_handler_fn_name(&self.widget_name, event_name);
        if self.vm.flash.exports_by_name.contains_key(&namespaced) {
            return true;
        }
        let fn_name = format!("handler_{}", extract_handler_name(event_name));
        self.vm.flash.exports_by_name.contains_key(&fn_name)
    }

    /// PLAN-659 T-05：指定 widget 名下 handler 存在性探测——与
    /// [`call_handler_for`] 的 namespaced 查找同键（不含 legacy 兜底——
    /// widget 限定的派发路径只走 namespaced 形态）。MCP fixture trigger
    /// 派发前直查本表：未命中即响亮报错（此前 `on_with_input_for` 未
    /// 命中静默无操作，AddrGo 实证病灶）。
    pub fn has_handler_for(&self, widget_name: &str, event_name: &str) -> bool {
        let fn_name = crate::ui::handler_codegen::namespaced_handler_fn_name(widget_name, event_name);
        self.vm.flash.exports_by_name.contains_key(&fn_name)
    }

    /// PLAN-659 T-05：裸名 handler 的 widget 归属解析——扫描 exports 的
    /// 全部 namespaced 键（handler_&lt;Widget&gt;_&lt;Event&gt;），取事件名
    /// 后缀匹配的候选 widget 集。嵌入画廊形态下根组件是宿主 App、目标
    /// handler 在子 demo 名空间（如 handler_Demo027FileManager_AddrGo），
    /// 裸名派发必须跨名空间解析。空=未命中；多候选=歧义（回执点名，
    /// 驱动侧改用 widget 形态消歧）。
    pub fn widgets_declaring_handler(&self, event_name: &str) -> Vec<String> {
        let suffix = format!("_{event_name}");
        let mut widgets: Vec<String> = self
            .vm
            .flash
            .exports_by_name
            .keys()
            .filter_map(|k| k.strip_prefix("handler_"))
            .filter(|rest| rest.ends_with(&suffix) && rest.len() > suffix.len())
            .map(|rest| rest[..rest.len() - suffix.len()].to_string())
            .collect();
        widgets.sort();
        widgets.dedup();
        widgets
    }

    /// List all registered handler names (bare names, without the `handler_` prefix).
    pub fn handler_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .vm
            .flash
            .exports_by_name
            .keys()
            .filter_map(|k| k.strip_prefix("handler_"))
            .collect();
        names.sort();
        names
    }
}

/// Push a runtime [`Value`] onto a task's RAM stack using the appropriate
/// encoding for its type (mirrors how GET_FIELD/CREATE_OBJ push values).
fn push_value(ram: &mut crate::vm::virt_memory::VirtualRAM, value: &Value) {
    match value {
        Value::Int(i) => ram.push_i32(*i),
        Value::Uint(u) => ram.push_i32(*u as i32),
        Value::Bool(b) => ram.push_i32(if *b { 1 } else { 0 }),
        Value::Char(c) => ram.push_i32(*c as i32),
        Value::Float(f) => ram.push_f32(*f as f32),
        Value::Double(d) => ram.push_f64(*d),
        Value::Nil => ram.push_i32(0),
        // Heap-referenced / complex values are not passed as scalar args in the
        // current handler surface; push a placeholder so arg arity stays correct.
        _ => ram.push_i32(0),
    }
}

/// Convert a base `crate::ast::Expr` initial value to a runtime `Value`.
///
/// Phase 3: replaces the old `eval_aura_expr_to_value` (which consumed the now-
/// eliminated `AuraExpr`). Handles the common literal types; complex
/// expressions default to Nil.
fn eval_expr_to_value(expr: &Expr, vm: &mut AutoVM) -> Value {
    match expr {
        Expr::Int(i) => Value::Int(*i),
        Expr::I64(i) => Value::Int(*i as i32),
        Expr::Uint(u) => Value::Uint(*u),
        Expr::U64(u) => Value::Uint(*u as u32),
        Expr::Byte(b) => Value::Int(*b as i32),
        Expr::I8(i) => Value::Int(*i as i32),
        Expr::U8(u) => Value::Int(*u as i32),
        Expr::Float(f, _) => Value::Double(*f),
        Expr::Double(f, _) => Value::Double(*f),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Char(c) => Value::Int(*c as i32),
        Expr::Str(s) => Value::Str(s.clone()),
        Expr::CStr(s) => Value::Str(s.clone()),
        Expr::Ident(name) => {
            // State reference: an identifier whose name starts with "." is a
            // state-ref; otherwise treat as an unresolved state reference.
            // Initial values default to zero/null placeholder.
            let _ = name;
            Value::Int(0)
        }
        // Binary expressions in initial values: simple cases only.
        Expr::Bina(_, _, _) => Value::Int(0),
        Expr::Unary(op, operand) => {
            let val = eval_expr_to_value(operand, vm);
            match op {
                Op::Sub => match val {
                    Value::Int(i) => Value::Int(-i),
                    Value::Double(f) => Value::Double(-f),
                    Value::Float(f) => Value::Float(-f),
                    _ => Value::Int(0),
                },
                Op::Not => match val {
                    Value::Bool(b) => Value::Bool(!b),
                    _ => Value::Bool(true),
                },
                _ => Value::Int(0),
            }
        }
        Expr::Array(elements) => {
            // Plan 420: state 列表字面量物化为 VM 原生 ListData 堆对象
            // (Value::VmRef)—— 与 handler 内局部列表同表示,`.len()`/
            // `.push()`/`[i]` 索引与嵌套字段赋值等字节码操作才能作用于
            // state 字段。此前存 Value::Array,渲染读得到、handler 读出 0。
            let mut list = crate::vm::types::ListData::<Value>::new();
            for e in elements {
                list.push(eval_expr_to_value(e, vm));
            }
            let id = vm.insert_heap_object(list);
            // Plan 423 P5 续修(RC 根引用缺口):返回的引用将被 state 字段/
            // 父容器长期持有 —— 按 419 计数语义补上这份持有份额(+1)。
            // 缺它时:GET_ELEM/GET_FIELD 的栈份额在链式写的 receiver stake
            // 死亡时归零 → 对象被释放 → 字段成悬垂 → canary UAF(041
            // ActNew 实测 panic,rc.rs:378)。
            vm.rc_retain_id(id);
            Value::VmRef(auto_val::VmRef { id: id as usize })
        }
        Expr::Object(pairs) => {
            // Plan 420: 对象字面量同理物化为 GenericInstanceData(字段名可
            // 索引),列表元素经此表示后 `.tabs[0].dirty = true` 才能落进
            // 真实字段(handler 字面量构造走 CONSTRUCT_INSTANCE 同型)。
            let mut names = Vec::with_capacity(pairs.len());
            let mut values = Vec::with_capacity(pairs.len());
            for pair in pairs {
                // to_astr() — bare name; Name's Display wraps as "(name key)"
                // which breaks GET_FIELD's by-name lookup (Plan 420 探针实证)。
                names.push(pair.key.to_astr().to_string());
                values.push(eval_expr_to_value(&pair.value, vm));
            }
            let inst = crate::vm::generic_registry::GenericInstanceData::new_with_names(
                "StateObjectLit".to_string(),
                values,
                names,
            );
            let id = vm.insert_heap_object(inst);
            // Plan 423 P5 续修:同上 —— 持有份额 +1(被列表元素/字段持有)。
            vm.rc_retain_id(id);
            Value::VmRef(auto_val::VmRef { id: id as usize })
        }
        // EDGE-04 fix: type literal `Type{ field: val, ... }` (parsed as
        // Expr::Node) must be materialized to a GenericInstanceData on the VM
        // heap and returned as Value::VmRef. Previously fell into _ => Nil,
        // so store model fields like `var git_info PromptContext = PromptContext{...}`
        // were initialized to Nil — downstream `.git_status.staged` then hit
        // "Field index out of bounds for primitive" (GET_GENERIC_FIELD on Nil).
        // Field extraction mirrors vm codegen (codegen.rs:4817-4845): args
        // (Pos/Pair) + body stmts (Expr::Pair). Nested type literals recurse.
        Expr::Node(node) => {
            materialize_type_literal(node, vm)
        }
        // Function-style type constructor Type(...) — also materialize if the
        // callee is a registered type (same as codegen.rs:6562 Expr::Call path).
        Expr::Call(call) => {
            if let crate::ast::Expr::Ident(type_name) = call.name.as_ref() {
                let tn = type_name.to_string();
                if vm.generic_registry.has_template(&tn) {
                    // Build a synthetic Node from the call args and materialize.
                    let synthetic = crate::ast::Node {
                        name: type_name.clone(),
                        id: crate::ast::Name::new(),
                        num_args: call.args.args.len(),
                        args: call.args.clone(),
                        body: crate::ast::Body::new(),
                        typ: auto_val::shared(crate::ast::Type::Unknown),
                        doc: None,
                    };
                    materialize_type_literal(&synthetic, vm)
                } else {
                    Value::Nil
                }
            } else if let crate::ast::Expr::Dot(receiver, method) = call.name.as_ref() {
                // EDGE-16: <Type>.new(...) 构造调用,name 是 Expr::Dot
                // (如 `List<T>.new([])`)。之前落入 Nil,导致 store model 字段
                // 初始化为 Nil,handler 的 push 静默失败。这里物化成堆对象。
                if method == "new" {
                    // receiver 可能是 GenName(泛型,如 List<BlockItem>)或 Ident(普通类型)
                    let type_name_opt = match receiver.as_ref() {
                        crate::ast::Expr::GenName(n) => Some(n.as_str().to_string()),
                        crate::ast::Expr::Ident(n) => Some(n.as_str().to_string()),
                        _ => None,
                    };
                    if let Some(tn) = type_name_opt {
                        // List<T>.new(...) — 物化成空 ListData 堆对象
                        // (忽略 args,等价 List::new;初始 elems 后续由 push 填充)。
                        if tn.starts_with("List") {
                            let id = vm.insert_heap_object(crate::vm::types::ListData::<auto_val::Value> {
                                elems: Vec::new(),
                                storage: None,
                            });
                            // Plan 423 P5 续修(RC 根引用缺口):同 Array/Object
                            // 字面量分支 —— 引用将被 state 字段长期持有,补
                            // 持有份额(+1)。缺它时 Init 的首个 .push 链式写
                            // 即把计数归零 → 字段悬垂 → canary UAF(021
                            // block-static 实测 rc.rs:378)。
                            vm.rc_retain_id(id);
                            return Value::VmRef(auto_val::VmRef { id: id as usize });
                        }
                        // <OtherType>.new() — 若是注册类型,走字面量物化(空字段)。
                        if vm.generic_registry.has_template(&tn) {
                            let synthetic = crate::ast::Node {
                                name: crate::ast::Name::from(tn.as_str()),
                                id: crate::ast::Name::new(),
                                num_args: 0,
                                args: call.args.clone(),
                                body: crate::ast::Body::new(),
                                typ: auto_val::shared(crate::ast::Type::Unknown),
                                doc: None,
                            };
                            return materialize_type_literal(&synthetic, vm);
                        }
                    }
                }
                Value::Nil
            } else {
                Value::Nil
            }
        }
        // Complex expressions default to Nil for safety
        _ => Value::Nil,
    }
}

/// Materialize a type literal (`Type{ field: val, ... }`) into a
/// GenericInstanceData on the VM heap. Used by `eval_expr_to_value` for store
/// model field initialization (EDGE-04). Returns Value::VmRef(heap_id), or
/// Value::Nil if the type isn't registered / has no fields.
fn materialize_type_literal(node: &crate::ast::Node, vm: &mut AutoVM) -> Value {
    use crate::vm::generic_registry::GenericInstanceData;
    let type_name = node.name.to_string();
    // Look up the ClassType to get mono_name + field count + field names.
    let class_type = match vm.generic_registry.get_or_create_type(&type_name, Vec::new()) {
        Ok(ct) => ct,
        Err(_) => return Value::Nil,
    };
    let mono_name = class_type.mono_name.clone();
    let field_defs = class_type.fields();
    let field_count = field_defs.len();

    // Collect field values from args (Pos/Pair) + body stmts (Expr::Pair),
    // mirroring codegen.rs:4817-4845.
    let mut field_exprs: Vec<&crate::ast::Expr> = Vec::new();
    for arg in &node.args.args {
        match arg {
            crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) => field_exprs.push(e),
            crate::ast::Arg::Name(_) => {} // skip; will be filled with default below
        }
    }
    for stmt in &node.body.stmts {
        if let crate::ast::Stmt::Expr(crate::ast::Expr::Pair(pair)) = stmt {
            field_exprs.push(&pair.value);
        }
    }

    // Evaluate each field (recursively materializing nested type literals).
    let mut field_values: Vec<Value> = Vec::with_capacity(field_count);
    for i in 0..field_count {
        if let Some(expr) = field_exprs.get(i) {
            field_values.push(eval_expr_to_value(expr, vm));
        } else {
            // Field not provided in the literal — default by type.
            field_values.push(Value::Int(0));
        }
    }

    let field_names: Vec<String> = field_defs.iter().map(|f| f.name.clone()).collect();
    let instance = if !field_names.is_empty() && field_values.len() == field_names.len() {
        GenericInstanceData::new_with_names(mono_name, field_values, field_names)
    } else {
        GenericInstanceData::new(mono_name, field_values)
    };
    let heap_id = vm.insert_heap_object(instance);
    // Plan 423 P5 续修(RC 根引用缺口):同 Array/Object 字面量分支 ——
    // 引用将被 state 字段长期持有,补持有份额(+1)。缺它时链式写
    // (`.field.x = v`)的 receiver stake 死亡即归零 → 字段悬垂 → canary UAF。
    vm.rc_retain_id(heap_id);
    Value::VmRef(auto_val::VmRef { id: heap_id as usize })
}

/// Extract a clean handler name from an event pattern.
///
/// Patterns can be:
/// - ".Inc" -> "Inc"
/// - "Msg::Inc" -> "Inc"
/// - "Inc" -> "Inc"
/// - ".SelectDay(date)" -> "SelectDay" (Plan 423 P5 续修:剥参数列表,
///   与 handler_codegen::bare_handler_name 同型)
fn extract_handler_name(pattern: &str) -> &str {
    let name = pattern.trim_start_matches('.');
    let name = name.split('(').next().unwrap_or(name);
    if let Some(pos) = name.rfind("::") {
        &name[pos + 2..]
    } else {
        name
    }
}

/// Convert a serde_json::Value to an auto_val::Value.
///
/// Retained for Plan 323 Task 5 (extracting HTTP shims into natives); currently
/// unused after the AST interpreter removal.
#[cfg(feature = "ui-interpreter")]
#[allow(dead_code)]
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i as i32)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Nil
            }
        }
        serde_json::Value::String(s) => Value::str(s.as_str()),
        serde_json::Value::Array(arr) => {
            let items: Vec<Value> = arr.iter().map(json_to_value).collect();
            Value::Array(auto_val::Array::from(items))
        }
        serde_json::Value::Object(map) => {
            let mut obj = auto_val::Obj::new();
            for (key, val) in map {
                obj.set(key.as_str(), json_to_value(val));
            }
            Value::Obj(Box::new(obj))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Type;

    /// Plan 423 P5 加固:runaway 守卫 —— 单次 handler 调用内字符串池增量
    /// 超 RUNAWAY_STRINGS_GROWTH(1M)必须以明确错误中止,而不是无界吃内存
    /// (毒化字节码在垃圾指令里跑满步数预算的实机 20G 内存事故形态)。
    /// l[0] 的每次 GET_ELEM 都会把元素副本压入运行期字符串池 → 110 万次
    /// 循环即越过阈值。
    /// Plan 423 P5 续修回归锁:物化 state 字面量(列表/对象 VmRef)必须带
    /// "被字段/父容器持有"的 RC 份额 —— 否则首个链式写(`.tabs[0].x = v`)
    /// 的 receiver stake 死亡即把对象释放到零,字段悬垂,canary UAF panic
    /// (041 ActNew 实测,rc.rs:378)。此测试在修复前会 panic。
    #[test]
    fn plan423_p5_state_literal_refs_survive_chained_write() {
        use crate::aura::LogicPayload;
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        let mut widget = make_test_widget("RcRootTest", vec![]);
        let model_src = r#"
            var tabs list = [{a: 1, b: 2}, {a: 3, b: 4}]
        "#;
        let session = CompilerSession::ui();
        let mut parser = Parser::from(model_src).with_session(session);
        let ast = parser.parse().expect("parse model");
        let inits: Vec<_> = ast
            .stmts
            .iter()
            .filter_map(|s| match s {
                crate::ast::Stmt::Store(st) => Some(st.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(inits.len(), 1, "expected one var decl");
        widget.state_vars.push(AuraStateDef {
            name: "tabs".to_string(),
            type_info: Type::Unknown,
            initial: inits[0].expr.clone(),
            decorators: vec![],
        });
        let handler_src = r#"
            .tabs[0].a = 99
        "#;
        let mut parser = Parser::from(handler_src).with_session(CompilerSession::ui());
        let ast2 = parser.parse().expect("parse handler");
        widget
            .handlers
            .insert(".Poke".to_string(), LogicPayload::AstStmts(ast2.stmts));
        let mut bridge = VmBridge::new(&widget).expect("bridge");
        // 连续两次链式写:第一次若缺持有份额,receiver stake 死亡即 UAF。
        bridge.call_handler("Poke", &[]).expect("first chained write");
        bridge.call_handler("Poke", &[]).expect("second chained write");
        let tabs = bridge.read_state_as_vec("tabs").expect("tabs");
        let first = bridge.materialize_obj_ref(&tabs[0]);
        match first {
            auto_val::Value::Obj(obj) => {
                let a = obj.get("a").unwrap_or(Value::Int(0));
                assert_eq!(a, Value::Int(99), "chained write must land");
            }
            other => panic!("tabs[0] not an object: {other:?}"),
        }
    }

    /// PLAN-661 T-01: map 括号写往返（078 复现包收编，AC-01）。
    /// `.m[k] = v` 此前在 SET_ELEM 只 downcast ListData →
    /// `RuntimeError("Invalid array ID")` 中止整个 handler（写后语句全不
    /// 执行，错误被 UI 静默吞掉）。修复后：写落在 state map
    /// （StateObjectLit GenericInstanceData 表示）上，读回可见，后续语句
    /// 续行；字面量键与 var 键同形（078 isolation A/D 两形态）。
    #[test]
    fn plan661_t01_map_bracket_write_roundtrip() {
        use crate::aura::LogicPayload;
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        let mut widget = make_test_widget("MapWrite", vec![]);
        let model_src = r#"
            var m map = { a: 1 }
            var count int = 0
            var readback str = "init"
        "#;
        let session = CompilerSession::ui();
        let mut parser = Parser::from(model_src).with_session(session);
        let ast = parser.parse().expect("parse model");
        let inits: Vec<_> = ast
            .stmts
            .iter()
            .filter_map(|s| match s {
                crate::ast::Stmt::Store(st) => Some(st.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(inits.len(), 3, "expected three var decls");
        for (i, name) in ["m", "count", "readback"].iter().enumerate() {
            widget.state_vars.push(AuraStateDef {
                name: name.to_string(),
                type_info: Type::Unknown,
                initial: inits[i].expr.clone(),
                decorators: vec![],
            });
        }
        // 字面量键形态 + 写后语句续行断言（count/readback 在写之后更新）。
        let handler_src = r#"
            .m["b"] = 2
            var cur = .m["b"]
            .readback = f"${cur}"
            .count = .count + 1
        "#;
        let mut parser = Parser::from(handler_src).with_session(CompilerSession::ui());
        let ast2 = parser.parse().expect("parse handler");
        widget
            .handlers
            .insert(".DoIt".to_string(), LogicPayload::AstStmts(ast2.stmts));
        let mut bridge = VmBridge::new(&widget).expect("bridge");
        bridge.call_handler("DoIt", &[]).expect("DoIt must not abort");
        let count = bridge.read_state("count").expect("count");
        assert_eq!(count, Value::Int(1), "statements after the map write must run");
        match bridge.read_state("readback").expect("readback") {
            Value::Str(s) => assert_eq!(s.as_str(), "2", "read-back of written key must observe the write"),
            other => panic!("readback not a str: {other:?}"),
        }
        // var 键形态：既有键覆写 + var 键读回。
        let handler2_src = r#"
            var k str = "a"
            .m[k] = .m[k] + 40
            var cur2 = .m[k]
            .readback = f"${cur2}"
            .count = .count + 1
        "#;
        let mut widget2 = make_test_widget("MapWrite", vec![]);
        widget2.state_vars = widget.state_vars.clone();
        let mut parser = Parser::from(handler2_src).with_session(CompilerSession::ui());
        let ast3 = parser.parse().expect("parse handler2");
        widget2
            .handlers
            .insert(".Poke".to_string(), LogicPayload::AstStmts(ast3.stmts));
        let mut bridge2 = VmBridge::new(&widget2).expect("bridge2");
        bridge2.call_handler("Poke", &[]).expect("Poke must not abort");
        match bridge2.read_state("readback").expect("readback2") {
            Value::Str(s) => assert_eq!(s.as_str(), "41", "var-key overwrite of existing entry must land"),
            other => panic!("readback2 not a str: {other:?}"),
        }
    }

    /// PLAN-626 T-03: CloseRequest 生命周期 handler 的存在性探测与直调。
    /// 声明了 `.CloseRequest` 的 widget 必须被 has_handler 命中（namespaced
    /// 导出），未声明的必须 miss——渲染器关窗臂据此决定拦截还是默认关窗。
    #[test]
    fn plan626_has_handler_close_request_lifecycle() {
        use crate::aura::LogicPayload;
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        let mut widget = make_test_widget("CloseProbe", vec![]);
        let handler_src = r#"
            console_log("close requested")
        "#;
        let mut parser = Parser::from(handler_src).with_session(CompilerSession::ui());
        let ast = parser.parse().expect("parse handler");
        widget
            .handlers
            .insert(".CloseRequest".to_string(), LogicPayload::AstStmts(ast.stmts));
        let bridge = VmBridge::new(&widget).expect("bridge");
        assert!(
            bridge.has_handler("CloseRequest"),
            "declared lifecycle handler must be found (namespaced export)"
        );
        assert!(
            !bridge.has_handler("NoSuchLifecycle"),
            "undeclared handler must be absent"
        );
        // Fire path: CloseRequest dispatches like a regular handler.
        let mut bridge2 = VmBridge::new(&widget).expect("bridge2");
        bridge2
            .call_handler("CloseRequest", &[])
            .expect("CloseRequest dispatch");
    }

    /// PLAN-702 T-02/T-04: 桥接段派发全链——handler 内 Http.post_json 即
    /// park 入注册表（零忙等），重入默认忽略，结果就绪后 resume_ready_parked
    /// 续跑落账（model 写入经 read_state 可见），`__busy_handlers` 镜像随
    /// park/完成翻转。HTTP worker 走真本地 server（受控延迟 300ms，证明
    /// dispatch 立即返回）。
    #[test]
    fn plan702_handler_park_reentry_resume_roundtrip() {
        use crate::ast::Stmt;
        use crate::aura::{AuraStateDef, LogicPayload};
        use crate::parser::Parser;
        use crate::session::CompilerSession;

        let body_for_server = "seeded-body".to_string();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 4096];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            std::thread::sleep(std::time::Duration::from_millis(300));
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body_for_server.len(),
                body_for_server
            );
            let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
        });

        let mut widget = make_test_widget("ParkProbe", vec![]);
        let state_src = r#"var result str = """#;
        let mut parser = Parser::from(state_src).with_session(CompilerSession::ui());
        let ast = parser.parse().expect("parse state");
        let inits: Vec<_> = ast
            .stmts
            .iter()
            .filter_map(|s| match s {
                Stmt::Store(st) => Some(st.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(inits.len(), 1);
        widget.state_vars.push(AuraStateDef {
            name: "result".to_string(),
            type_info: Type::Unknown,
            initial: inits[0].expr.clone(),
            decorators: vec![],
        });
        let handler_src = format!(
            r#"
            var resp = Http.post_json("http://127.0.0.1:{}/seed", "q=1")
            .result = resp
        "#,
            port
        );
        let mut parser = Parser::from(&handler_src).with_session(CompilerSession::ui());
        let ast2 = parser.parse().expect("parse handler");
        widget
            .handlers
            .insert(".Load".to_string(), LogicPayload::AstStmts(ast2.stmts));
        let mut bridge = VmBridge::new(&widget).expect("bridge");

        // 首次派发：立即返回（server 延迟 300ms 内 park 完成 = 零忙等）。
        let started = std::time::Instant::now();
        bridge.call_handler("Load", &[]).expect("dispatch parks");
        assert!(
            started.elapsed() < std::time::Duration::from_millis(250),
            "dispatch 耗时 {:?} —— 忙等回归",
            started.elapsed()
        );
        assert_eq!(bridge.parked_count(), 1, "应恰有一个 parked 段");
        assert!(bridge.has_parked_tasks());

        // T-04 busy 镜像：__busy_handlers 含 namespaced fn 名。读法走
        // read_state 的 live-object 兜底（镜像字段是运行期追加的堆字段，
        // 不在声明字段列表内——read_all_state 只枚举声明字段）。
        let busy_names: Vec<String> = bridge
            .read_state_as_vec("__busy_handlers")
            .expect("busy 镜像字段应存在")
            .iter()
            .map(|v| v.as_str().to_string())
            .collect();
        assert!(
            busy_names.iter().any(|n| n.contains("ParkProbe_Load")),
            "busy 镜像应含 namespaced handler 名：{:?}",
            busy_names
        );

        // T-04 重入：parked 期间重复触发被忽略（不叠执行）。
        bridge.call_handler("Load", &[]).expect("re-entry ignored");
        assert_eq!(bridge.parked_count(), 1, "重入不得新开段");

        // 结果就绪后恢复泵续跑（模拟 __parked_resume_tick 臂）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let report = bridge.resume_ready_parked();
            if report.completed > 0 {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "恢复泵 5s 内未完成——worker 未应答？"
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        server.join().expect("server thread");

        // AC-02: 结果落账——model 值正确、注册表出清、busy 镜像归零。
        assert_eq!(bridge.parked_count(), 0, "完成段应出清");
        match bridge.read_state("result").expect("result") {
            Value::Str(s) => assert_eq!(s.as_str(), "seeded-body"),
            other => panic!("result not a str: {other:?}"),
        }
        let busy_after: Vec<String> = bridge
            .read_state_as_vec("__busy_handlers")
            .expect("busy 镜像字段应仍在")
            .iter()
            .map(|v| v.as_str().to_string())
            .collect();
        assert!(
            busy_after.is_empty(),
            "完成后 busy 镜像应空：{:?}",
            busy_after
        );
    }

    #[test]
    fn plan423_p5_runaway_guard_halts_unbounded_growth() {
        use crate::aura::LogicPayload;
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        let mut widget = make_test_widget("RunawayTest", vec![]);
        // 载体演进:①无副作用的读取会被编译器省略;②Plan 419 的字符串池
        // RC+freelist 让"重复串/拼接串"的活池有界(str_churn_bounded)——
        // 字符串维度已被其关闭。存活增长向量是**堆对象**(列表持有引用
        // 不释放):60 万个对象字面量入列,活堆 +60 万 > 50 万阈值。
        let src = r#"
            var l list = []
            for i in 0..600000 {
                l.push({a: i})
            }
        "#;
        let session = CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse runaway snippet");
        widget
            .handlers
            .insert(".Boom".to_string(), LogicPayload::AstStmts(ast.stmts));
        let mut bridge = VmBridge::new(&widget).expect("bridge");
        let err = bridge
            .call_handler("Boom", &[])
            .expect_err("runaway growth must halt with an error");
        let msg = err.to_string();
        assert!(
            msg.contains("runaway execution halted"),
            "unexpected error: {msg}"
        );
        assert!(msg.contains("strings"), "growth detail in message: {msg}");
    }

    /// Plan 445 M3：滑窗重建（024-charts .Tick 的核心形态）——
    /// **全保真载具**：WidgetDecl 解析 + new_from_decls + call_handler_for
    /// （与 dynamic.rs on_with_input_for 实机 dispatch 逐步一致）。此前用
    /// VmBridge::new（AuraWidget 路径）无法复现实机 `.monthly = out` 不落
    /// 地 → 重算 DivisionByZero 的失败链（ASH_DEBUG_VM_LOG 实机实锤）。
    #[test]
    fn plan445_m3_slide_window_rebuild() {
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        let app_src = r#"
widget SlideTest {
    msg Msg { Init }

    model {
        var data = [{ m: "A", v: 1 }, { m: "B", v: 2 }, { m: "C", v: 3 }]
        var windowLen int = 3
        var tickN int = 0
    }

    on {
        .Init -> { .tickN = 0 }
        .Tick -> {
            .tickN = .tickN + 1
            var nd = 150 + (.tickN * 37) % 120
            var lbl = f"t${.tickN}"
            var out = []
            var first = true
            var over = .data.len() >= .windowLen
            for d in .data {
                if over && first {
                    first = false
                } else {
                    out.push(d)
                }
            }
            out.push({ m: lbl, v: nd })
            .data = out
        }
    }
}
        "#;
        let session = CompilerSession::ui();
        let mut parser = Parser::from(app_src).with_session(session);
        let ast = parser.parse().expect("parse app");
        let decl = ast
            .stmts
            .into_iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let mut bridge = VmBridge::new_from_decls(
            &decl,
            &[],
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("bridge from decls");
        let sid = bridge.state_obj_id();
        // 与实机一致的 Init dispatch。
        bridge
            .call_handler_for("SlideTest", "Init", sid, &[])
            .expect("init");
        bridge
            .call_handler_for("SlideTest", "Tick", sid, &[])
            .expect("tick 1");
        let data = bridge.read_state_as_vec("data").expect("data");
        assert_eq!(data.len(), 3, "满窗时追加必须滑出最旧一点（.data = out 必须落地）");
        let tick_n = bridge.read_state("tickN").expect("tickN");
        assert_eq!(tick_n, Value::Int(1), "tickN 递增");
        bridge
            .call_handler_for("SlideTest", "Tick", sid, &[])
            .expect("tick 2");
        let data2 = bridge.read_state_as_vec("data").expect("data2");
        assert_eq!(data2.len(), 3, "窗口稳定");
    }

    /// Plan 445 M3：svgdoc 动态 props——`d: .p` 绑定在 view build 时经
    /// bindings/state 解析进 SVG 文档（此前仅字面量透传，图表 path 在
    /// VM 轨 svgdoc 通道整体丢失）。全保真载具（decl 路径 + view build）。
    /// 若 resolve 链存在锁重入/死循环，此测试将挂起（对照：实机带改动
    /// 曾冻窗，需以本测试钉死单线程可复现性）。
    #[test]
    fn plan445_m3_svgdoc_dynamic_props() {
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        use crate::ui::aura_view_builder::AuraViewBuilder;
        let app_src = r##"
widget SvgProbe {
    msg Msg { Init }

    model {
        var p str = ""
    }

    on {
        .Init -> { .p = "M 40 260 L 550 20" }
    }

    view {
        col {
            svg (viewBox: "0 0 560 300", style: "w-full h-auto") {
                path (d: .p, fill: "none", stroke: "#2563eb") {}
            }
        }
    }
}
        "##;
        let session = CompilerSession::ui();
        let mut parser = Parser::from(app_src).with_session(session);
        let ast = parser.parse().expect("parse app");
        let (decl, widget) = ast
            .stmts
            .into_iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => {
                    let w = crate::aura::extract_widget_from_decl(&d).expect("extract");
                    Some((d, w))
                }
                _ => None,
            })
            .expect("widget decl");
        let mut bridge = VmBridge::new_from_decls(
            &decl,
            &[],
            vec![],
            &std::collections::HashMap::new(),
            false,
        )
        .expect("bridge from decls");
        let sid = bridge.state_obj_id();
        bridge
            .call_handler_for("SvgProbe", "Init", sid, &[])
            .expect("init");
        let builder = AuraViewBuilder::new(&bridge, "SvgProbe");
        let view = builder.build(&widget.view_tree);
        // 遍历找 View::Image 的 svgdoc src，断言动态 d 已解析进文档。
        fn find_svgdoc(v: &crate::ui::View<crate::DynamicMessage>, out: &mut Vec<String>) {
            use crate::ui::view::View;
            match v {
                View::Image { src, .. } => out.push(src.clone()),
                View::ImageSurface { src, .. } => out.push(src.clone()),
                View::Column { children, .. } | View::Row { children, .. } => {
                    for c in children { find_svgdoc(c, out); }
                }
                _ => {}
            }
        }
        let mut docs = Vec::new();
        find_svgdoc(&view, &mut docs);
        let doc = docs.iter().find(|s| s.starts_with("svgdoc:")).expect("svgdoc image");
        assert!(
            doc.contains("M 40 260 L 550 20"),
            "动态 d 必须解析进 svgdoc（got: {doc}）"
        );
    }

    /// Plan 445 后续诊断：fill-opacity 属性是否进 svgdoc 文档
    /// （VM 实测带 fill-opacity 的 path 整根不渲染；变体探针锁定）。
    #[test]
    fn plan445_diag_fill_opacity_serialization() {
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        use crate::ui::aura_view_builder::AuraViewBuilder;
        let app_src = r##"
widget OpProbe {
    msg Msg { Init }
    model {
        var barD str = ""
        var barM str = ""
        var dVisible bool = true
        var mVisible bool = true
    }
    on {
        .Init -> {
            .barD = "M44 113 h19 v146 h-19 Z "
            .barM = "M72 197 h19 v62 h-19 Z "
        }
    }
    view {
        col {
            svg (viewBox: "0 0 560 300", style: "w-full h-auto") {
                path (d: "M40 80 H550 M40 140 H550", stroke: "#e2e8f0", stroke-width: "1") {}
                if .dVisible {
                    path (d: .barD, fill: "#2563eb", fill-opacity: "0.9") {}
                }
                if .mVisible {
                    path (d: .barM, fill: "#16a34a", fill-opacity: "0.9") {}
                }
            }
        }
    }
}
        "##;
        let _unused = r##"
widget OpProbeOrig {
    msg Msg { Init }
    model { var p str = "" }
    on { .Init -> { .p = "M120 260 h50 v-150 h-50 Z" } }
    view {
        col {
            svg (viewBox: "0 0 400 300") {
                path (d: "M40 260 h50 v-150 h-50 Z", fill: "#16a34a", fill-opacity: "0.9") {}
            }
        }
    }
}
        "##;
        let session = CompilerSession::ui();
        let mut parser = Parser::from(app_src).with_session(session);
        let ast = parser.parse().expect("parse");
        let (decl, widget) = ast.stmts.into_iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => {
                let w = crate::aura::extract_widget_from_decl(&d).expect("extract");
                Some((d, w))
            }
            _ => None,
        }).expect("decl");
        let mut bridge = VmBridge::new_from_decls(&decl, &[], vec![],
            &std::collections::HashMap::new(), false).expect("bridge");
        let sid = bridge.state_obj_id();
        bridge.call_handler_for("OpProbe", "Init", sid, &[]).expect("init");
        let builder = AuraViewBuilder::new(&bridge, "OpProbe");
        let view = builder.build(&widget.view_tree);
        fn find_svgdoc(v: &crate::ui::View<crate::DynamicMessage>, out: &mut Vec<String>) {
            use crate::ui::View;
            match v {
                View::Image { src, .. } => out.push(src.clone()),
                View::ImageSurface { src, .. } => out.push(src.clone()),
                View::Column { children, .. } | View::Row { children, .. } => {
                    for c in children { find_svgdoc(c, out); }
                }
                _ => {}
            }
        }
        let mut docs = Vec::new();
        find_svgdoc(&view, &mut docs);
        let doc = docs.iter().find(|s| s.starts_with("svgdoc:")).expect("doc");
        eprintln!("SVGDOC: {doc}");
        // Plan 445 后续回归钉：Conditional 包裹的 path 必须序列化进 svgdoc
        // （此前 serializer 跳过 Conditional → VM 轨图表区无数据，仅字面量
        // 网格/轴线；runtime SVGDOC 打印实锤）。
        assert!(doc.contains("M44 113 h19 v146 h-19"), "if .dVisible 的 barD 必须进文档: {doc}");
        assert!(doc.contains("M72 197 h19 v62 h-19"), "if .mVisible 的 barM 必须进文档: {doc}");
        assert!(doc.contains("fill-opacity"), "fill-opacity 属性透传: {doc}");
    }

    /// Helper to create a minimal AuraWidget for testing
    fn make_test_widget(name: &str, state_vars: Vec<AuraStateDef>) -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: name.to_string(),
            state_vars,
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            setup: None, // Plan 426 field; test helper default
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: vec![],
            watchers: vec![],
            exposes: vec![],
        }
    }

    #[test]
    fn test_vm_bridge_creation_empty_state() {
        let widget = make_test_widget("EmptyWidget", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        assert_eq!(bridge.widget_name(), "EmptyWidget");
        assert!(bridge.state_fields().is_empty());
        assert!(bridge.handler_names().is_empty());

    }

    // ─────────────────────────────────────────────────────────────────
    // PLAN-706 r2 T-10: 信号网堆身份准入守卫（computed_signal_store）。
    // 回归面 = jade-edit vm_matrix [7 tab] 标脏弧 UAF（缓存 VmRef 悬垂 →
    // rc canary panic）/release 确认弹层滞留。此组测试在修复前红：
    // pre-fix store 照收 VmRef，信号表被悬垂 id 污染。
    // ─────────────────────────────────────────────────────────────────

    fn plan706_signal_rec() -> crate::ui::memo_deps::RecState {
        let mut rec = <crate::ui::memo_deps::RecState as Default>::default();
        rec.record(crate::ui::memo_deps::DepKey::field(1, "rows"));
        rec
    }

    /// 堆身份载体（直载 VmRef / ≥4M 整数约定形 / 嵌套容器内）不得入网。
    #[test]
    fn plan706_signal_store_skips_heap_identity_value() {
        let widget = make_test_widget("SigGuard", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let rec = plan706_signal_rec();

        // 直载 VmRef → 拒收。
        bridge.computed_signal_store(
            "SigGuard",
            "m",
            Value::VmRef(auto_val::VmRef { id: 4_000_555 }),
            &rec,
        );
        assert!(
            bridge.computed_signals.borrow().is_empty(),
            "VmRef 载体必须被拒收（UAF 回归守卫）"
        );

        // 嵌套载体（数组内 VmRef / ≥4M 整数）→ 拒收。
        bridge.computed_signal_store(
            "SigGuard",
            "m",
            Value::Array(auto_val::Array {
                values: vec![Value::Int(3), Value::VmRef(auto_val::VmRef { id: 99 })],
            }),
            &rec,
        );
        assert!(
            bridge.computed_signals.borrow().is_empty(),
            "数组内嵌 VmRef 载体必须被拒收"
        );
        bridge.computed_signals.borrow_mut().clear();

        bridge.computed_signal_store(
            "SigGuard",
            "m",
            Value::Array(auto_val::Array {
                values: vec![Value::Int(4_000_555)],
            }),
            &rec,
        );
        assert!(
            bridge.computed_signals.borrow().is_empty(),
            "≥4M 整数约定形（堆身份）载体必须被拒收"
        );

        // 正控：纯标量放行入网。
        bridge.computed_signal_store("SigGuard", "m", Value::Int(7), &rec);
        assert_eq!(
            bridge.computed_signals.borrow().len(),
            1,
            "纯标量载体应正常入网"
        );
    }

    /// 入库/命中循环幂等 + 状态变化后重填保新鲜（弹层滞留的缓存陈旧面：
    /// 旧值不得跨状态替换被复用）。
    #[test]
    fn plan706_signal_store_hit_cycle_idempotent_and_fresh() {
        let widget = make_test_widget(
            "SigCycle",
            vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: crate::ast::Expr::Int(1),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        let rec = plan706_signal_rec();

        bridge.computed_signal_store("SigCycle", "m", Value::Int(1), &rec);
        bridge.computed_signal_store("SigCycle", "m", Value::Int(1), &rec);
        assert_eq!(bridge.computed_signals.borrow().len(), 1, "同键重入库存幂等（单条目）");

        let hit = bridge.computed_signal_hit("SigCycle", "m");
        assert!(matches!(hit, Some(Value::Int(1))), "命中返回缓存值");

        // 状态替换 → 重填新值 → 命中必须返回新值（不陈旧）。
        bridge.write_state("count", Value::Int(9)).unwrap();
        bridge.computed_signal_store("SigCycle", "m", Value::Int(9), &rec);
        let hit = bridge.computed_signal_hit("SigCycle", "m");
        assert!(matches!(hit, Some(Value::Int(9))), "重填后命中返回新值");
    }

    /// 死对象 dep 键 → miss（版本冻结键伪命中守卫）。已分配但不在
    /// heap_objects = 已释放（id_gen 单调不复用）；未分配合成键按存活
    /// （纯版本比对语义保持——plan047 门测试依赖）。
    #[test]
    fn plan706_deps_unchanged_dead_heap_key_misses() {
        let widget = make_test_widget("SigDeadKey", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        // 已分配但摘除（freed 模拟：占号不 insert）。
        let freed_id = bridge
            .vm
            .heap_object_id_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dead = vec![(crate::ui::memo_deps::DepKey::field(freed_id, "rows"), 0u64)];
        assert!(
            !bridge.deps_unchanged(&dead),
            "死对象 dep 键必须 miss（版本冻结伪命中守卫）"
        );

        // 活对象键（状态根）：版本恒等 → unchanged。
        let root = bridge.state_obj_id();
        let live = vec![(
            crate::ui::memo_deps::DepKey::field(root, "rows"),
            bridge.vm.path_version(root, "rows"),
        )];
        assert!(bridge.deps_unchanged(&live), "活对象版本恒等 → unchanged");

        // 未分配合成键（远超 id_gen）→ 按存活。
        let synthetic = u64::MAX / 2;
        let unknown = vec![(crate::ui::memo_deps::DepKey::field(synthetic, "x"), 0u64)];
        assert!(
            bridge.deps_unchanged(&unknown),
            "未分配 id 按存活处理（plan047 门测试语义保持）"
        );
    }

    // ─────────────────────────────────────────────────────────────────
    // PLAN-047 T-01: plan047_recorder_channel_tests（桥读通道录制 4 条）
    // ─────────────────────────────────────────────────────────────────

    fn plan047_recorder_widget() -> AuraWidget {
        make_test_widget(
            "RecTarget",
            vec![
                AuraStateDef {
                    name: "count".to_string(),
                    type_info: Type::Int,
                    initial: Expr::Int(7),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "items".to_string(),
                    type_info: Type::Unknown,
                    initial: Expr::Array(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)]),
                    decorators: vec![],
                },
            ],
        )
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_recorder_inactive_zero_record() {
        let bridge = VmBridge::new(&plan047_recorder_widget()).expect("bridge");
        bridge.read_state("count").expect("read");
        bridge.read_state_as_vec("items").expect("read vec");
        // 未激活 = None，读通道零录制（非 memo 零开销红线）。
        assert!(bridge.peek_dep_recorder().is_none());
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_recorder_read_state_records_field() {
        let bridge = VmBridge::new(&plan047_recorder_widget()).expect("bridge");
        let guard = bridge.dep_recording_guard();
        bridge.read_state("count").expect("read");
        let rec = guard.finish();
        assert!(!rec.overflow);
        assert_eq!(rec.deps.len(), 1, "单字段读 = 单条依赖边");
        assert!(rec.deps.contains(&crate::ui::memo_deps::DepKey::field(
            bridge.state_obj_id(),
            "count"
        )));
        // finish 后恢复未激活。
        assert!(bridge.peek_dep_recorder().is_none());
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_recorder_container_read_records_any() {
        let bridge = VmBridge::new(&plan047_recorder_widget()).expect("bridge");
        let guard = bridge.dep_recording_guard();
        bridge.read_state("count").expect("read");
        bridge.read_state_as_vec("items").expect("read vec");
        let rec = guard.finish();
        // items 的堆列表 id 现场取出（字面量物化形态随实现 VmRef/Int(id)）。
        let raw = bridge.read_state("items").expect("items");
        let list_id = match raw {
            Value::VmRef(r) => r.id as u64,
            Value::Int(i) => i as u64,
            other => panic!("items not a heap list: {other:?}"),
        };
        assert!(
            rec.deps.contains(&crate::ui::memo_deps::DepKey::any(list_id)),
            "容器内容读 = (heap_id, \"*\") 粗粒度依赖"
        );
        assert!(
            rec.deps.contains(&crate::ui::memo_deps::DepKey::field(
                bridge.state_obj_id(),
                "items"
            ))
        );
        assert_eq!(rec.deps.len(), 3, "count + items 字段 + 列表内容");
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_recorder_materialize_and_nested_union() {
        let bridge = VmBridge::new(&plan047_recorder_widget()).expect("bridge");
        // 外层 guard 内嵌套内层 guard：内层读 + materialize 展开，
        // finish 后外层并集收编（外层条目覆盖内层求值全部依赖）。
        let outer = bridge.dep_recording_guard();
        {
            let inner = bridge.dep_recording_guard();
            bridge.read_state("count").expect("read");
            let items = bridge.read_state_as_vec("items").expect("items");
            let _ = bridge.materialize_obj_ref(&items[0]); // Int(id) 展开 → any(id)
            let inner_rec = inner.finish();
            assert_eq!(inner_rec.deps.len(), 3, "内层自见三条");
        }
        let outer_rec = outer.finish();
        assert_eq!(outer_rec.deps.len(), 3, "外层并集收编内层");
        assert!(bridge.peek_dep_recorder().is_none(), "嵌套恢复未激活");
    }

    // ─────────────────────────────────────────────────────────────────
    // PLAN-047 T-03: plan047_attribution_tests（写点归因 4 条，AC-03）
    // ─────────────────────────────────────────────────────────────────

    /// 解析 handler 源并挂到 widget（plan423 同款通道）。
    fn plan047_attach_handler(widget: &mut AuraWidget, event: &str, src: &str) {
        use crate::aura::LogicPayload;
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        let ast = Parser::from(src)
            .with_session(CompilerSession::ui())
            .parse()
            .expect("parse handler");
        widget
            .handlers
            .insert(format!(".{event}"), LogicPayload::AstStmts(ast.stmts));
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_attribution_set_field_is_path_exact() {
        let mut widget = make_test_widget("AttrRoot", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(0),
                decorators: vec![],
            },
            AuraStateDef {
                name: "label".to_string(),
                type_info: Type::StrFixed(0),
                initial: Expr::Str("L".into()),
                decorators: vec![],
            },
        ]);
        plan047_attach_handler(&mut widget, "Poke", "\n    .count = .count + 1\n");
        let mut bridge = VmBridge::new(&widget).expect("bridge");
        let root = bridge.state_obj_id();
        // 基线：无任何 path 版本。
        assert_eq!(bridge.vm.path_version(root, "count"), 0);
        assert_eq!(bridge.vm.path_version(root, "label"), 0);
        let seq0 = bridge.state_mutation_seq();
        bridge.call_handler("Poke", &[]).expect("handler");
        // AC-03: exact 面定点前进；无关 path 不动；wildcard（同对象任意写）
        // 与全局 seq 同步前进。
        assert_eq!(bridge.vm.path_version(root, "count"), 1, "exact 面前进");
        assert_eq!(bridge.vm.path_version(root, "label"), 0, "无关 path 不动");
        assert_eq!(bridge.vm.path_version(root, "*"), 1, "wildcard 同步");
        assert!(bridge.state_mutation_seq() > seq0, "全局 seq 语义不变");
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_attribution_list_push_wildcard_only() {
        let mut widget = make_test_widget("AttrList", vec![
            AuraStateDef {
                name: "items".to_string(),
                type_info: Type::Unknown,
                initial: Expr::Array(vec![Expr::Int(1), Expr::Int(2)]),
                decorators: vec![],
            },
        ]);
        plan047_attach_handler(&mut widget, "Add", "\n    .items.push(3)\n");
        let mut bridge = VmBridge::new(&widget).expect("bridge");
        let root = bridge.state_obj_id();
        bridge.call_handler("Add", &[]).expect("handler");
        let raw = bridge.read_state("items").expect("items");
        let list_id = match raw {
            Value::VmRef(r) => r.id as u64,
            Value::Int(i) => i as u64,
            other => panic!("items not heap list: {other:?}"),
        };
        // B 类：容器 wildcard 前进；根态字段 exact 不动（槽位未替换）——
        // 无关 path 版本零扰动的精度面。
        assert_eq!(bridge.vm.path_version(list_id, "*"), 1, "列表 wildcard");
        assert_eq!(bridge.vm.path_version(root, "items"), 0, "根态 exact 不动");
        assert_eq!(bridge.vm.path_version(root, "*"), 0, "根态 wildcard 不动");
        // 全局 seq 仍前进（全局语义不变）。
        assert!(bridge.state_mutation_seq() > 0);
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_attribution_bridge_write_state_exact() {
        let widget = make_test_widget("AttrBridge", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(0),
                decorators: vec![],
            },
        ]);
        let mut bridge = VmBridge::new(&widget).expect("bridge");
        let root = bridge.state_obj_id();
        bridge.write_state("count", Value::Int(9)).expect("write");
        assert_eq!(bridge.vm.path_version(root, "count"), 1);
        assert_eq!(bridge.vm.path_version(root, "*"), 1);
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_attribution_c_class_global_only() {
        let widget = make_test_widget("AttrC", vec![]);
        let bridge = VmBridge::new(&widget).expect("bridge");
        let seq0 = bridge.state_mutation_seq();
        // C 类：堆对象出世（insert_heap_object）——全局 seq 前进，per-path
        // 表零条目（新 id 无既有 dep 可指）。
        let new_id = bridge.vm.insert_heap_object(crate::vm::types::ListData::<i32> {
            elems: vec![1, 2],
            storage: None,
        });
        assert!(bridge.state_mutation_seq() > seq0);
        assert_eq!(bridge.vm.path_version(new_id, "*"), 0);
        assert_eq!(bridge.vm.path_version(new_id, "x"), 0);
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_attribution_hashmap_native_gap_closed() {
        // 普查发现闭合实证：auto.hashmap.set（shim_hashmap_insert_str）原
        // 无任何 seq bump——memo 全局快路径陈旧命中窗口。直调 shim 断言
        // 定点归因（exact+k+wildcard）与全局 seq 补齐。
        let widget = make_test_widget("AttrMap", vec![]);
        let bridge = VmBridge::new(&widget).expect("bridge");
        let seq0 = bridge.state_mutation_seq();
        let map_id = bridge.vm.insert_heap_object(
            crate::vm::collections::SpecializedHashMap::new("str"),
        );
        let mut task = crate::vm::task::AutoTask::new(0, 4096, 0);
        // 栈序（shim pop 序 value→key→map）：先 map_id、再 key、后 value。
        task.ram.push_i32(map_id as i32);
        let key_idx = bridge.vm.add_string(b"k".to_vec());
        task.ram.push_string(key_idx as u32);
        task.ram.push_i32(42);
        crate::vm::native::shim_hashmap_insert_str(&mut task, &bridge.vm)
            .expect("shim insert");
        // 全局 seq 补齐（原盲区）+ 定点归因（A-able 按键名，exact+wildcard）。
        assert!(bridge.state_mutation_seq() > seq0, "全局 seq 补 bump");
        assert_eq!(bridge.vm.path_version(map_id, "k"), 1, "按键名 exact");
        assert_eq!(bridge.vm.path_version(map_id, "*"), 1, "wildcard");
    }

    // ─────────────────────────────────────────────────────────────────
    // PLAN-047 T-04: plan047_engine_read_tests（引擎读臂拦截 2 条）
    // ─────────────────────────────────────────────────────────────────

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_engine_recorder_slot_semantics() {
        let widget = make_test_widget("RecSlot", vec![]);
        let bridge = VmBridge::new(&widget).expect("bridge");
        // 未激活：record 口零录制。
        bridge.vm.record_heap_read(1, "f");
        let arc = std::sync::Arc::new(std::sync::Mutex::new(
            crate::ui::memo_deps::RecState::default(),
        ));
        bridge.vm.set_dep_recorder(arc);
        bridge.vm.record_heap_read(7, "count");
        bridge.vm.record_heap_read_any(9);
        let rec = bridge.vm.take_dep_recorder();
        assert_eq!(rec.deps.len(), 2, "激活期两条落账");
        assert!(
            rec.deps.contains(&crate::ui::memo_deps::DepKey::field(7, "count"))
        );
        assert!(rec.deps.contains(&crate::ui::memo_deps::DepKey::any(9)));
        // take 后去激活：再录零账。
        bridge.vm.record_heap_read(1, "g");
        assert!(bridge.vm.take_dep_recorder().deps.is_empty());
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_engine_read_arm_intercepts_handler_reads() {
        // 端到端：录制激活期内 handler VM 执行（`.count` 读经 GET 系读臂
        // 编码）的依赖由引擎影子集承载——集含 (root,"count")。call_handler
        // 是 &mut self，故直挂 vm 槽（guard 的桥侧并集语义由 T-01 套件
        // 承载，此处专证引擎读臂面）。
        let mut widget = make_test_widget("RecEngine", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(0),
                decorators: vec![],
            },
        ]);
        plan047_attach_handler(&mut widget, "Poke", "\n    .count = .count + 1\n");
        let mut bridge = VmBridge::new(&widget).expect("bridge");
        let root = bridge.state_obj_id();
        let arc = std::sync::Arc::new(std::sync::Mutex::new(
            crate::ui::memo_deps::RecState::default(),
        ));
        bridge.vm.set_dep_recorder(arc.clone());
        bridge.call_handler("Poke", &[]).expect("handler");
        bridge.vm.take_dep_recorder();
        let rec = arc.lock().unwrap().clone();
        assert!(
            rec.deps.contains(&crate::ui::memo_deps::DepKey::field(root, "count")),
            "引擎读臂录制面: deps={rec:?}"
        );
        assert!(!rec.overflow);
        // 去激活后 handler 执行零账（写-only 复跑，无 GET 发生）。
        bridge.call_handler("Poke", &[]).expect("handler2");
        assert!(bridge.vm.take_dep_recorder().deps.is_empty());
    }

    #[test]
    #[cfg(feature = "ui-interpreter")]
    fn plan047_recorder_drop_without_finish_restores() {
        let bridge = VmBridge::new(&plan047_recorder_widget()).expect("bridge");
        {
            let _guard = bridge.dep_recording_guard();
            bridge.read_state("count").expect("read");
            // guard 未 finish 即 drop（早退形态）——恢复语义兜底。
        }
        assert!(bridge.peek_dep_recorder().is_none(), "drop 恢复外层(None)");
    }

    #[test]
    fn test_vm_bridge_creation_with_state() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(0),
                decorators: vec![],
            },
            AuraStateDef {
                name: "label".to_string(),
                type_info: Type::StrFixed(0),
                initial: Expr::Str("Hello".into()),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        assert_eq!(bridge.widget_name(), "Counter");
        assert_eq!(bridge.state_fields().len(), 2);
        assert_eq!(bridge.state_fields()[0], "count");
        assert_eq!(bridge.state_fields()[1], "label");
    }

    #[test]
    fn test_read_state_int() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(42),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        let value = bridge.read_state("count").unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[test]
    fn test_read_state_string() {
        let widget = make_test_widget("Greeter", vec![
            AuraStateDef {
                name: "greeting".to_string(),
                type_info: Type::StrFixed(0),
                initial: Expr::Str("Hello World".into()),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        let value = bridge.read_state("greeting").unwrap();
        assert_eq!(value, Value::str("Hello World"));
    }

    #[test]
    fn test_read_state_bool() {
        let widget = make_test_widget("Toggle", vec![
            AuraStateDef {
                name: "active".to_string(),
                type_info: Type::Bool,
                initial: Expr::Bool(true),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        let value = bridge.read_state("active").unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn test_read_state_unary_neg() {
        // This mirrors how `var editing_id int = -1` is parsed:
        // Expr::Unary(Sub, Int(1)). eval_expr_to_value should produce
        // Value::Int(-1), NOT Value::Int(0)
        let widget = make_test_widget("Todo", vec![
            AuraStateDef {
                name: "editing_id".to_string(),
                type_info: Type::Int,
                initial: Expr::Unary(Op::Sub, Box::new(Expr::Int(1))),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();
        let value = bridge.read_state("editing_id").unwrap();
        assert_eq!(value, Value::Int(-1), "expected -1, got {:?}", value);
    }

    #[test]
    fn test_read_state_not_found() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(0),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        let result = bridge.read_state("nonexistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            VmBridgeError::FieldNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("Expected FieldNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_write_state() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut bridge = VmBridge::new(&widget).unwrap();

        // Write new value
        bridge.write_state("count", Value::Int(10)).unwrap();

        // Read back
        let value = bridge.read_state("count").unwrap();
        assert_eq!(value, Value::Int(10));
    }

    #[test]
    fn test_write_state_not_found() {
        let widget = make_test_widget("Counter", vec![]);
        let mut bridge = VmBridge::new(&widget).unwrap();

        let result = bridge.write_state("nope", Value::Int(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_all_state() {
        let widget = make_test_widget("Multi", vec![
            AuraStateDef {
                name: "x".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(1),
                decorators: vec![],
            },
            AuraStateDef {
                name: "y".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(2),
                decorators: vec![],
            },
            AuraStateDef {
                name: "name".to_string(),
                type_info: Type::StrFixed(0),
                initial: Expr::Str("test".into()),
                decorators: vec![],
            },
        ]);

        let bridge = VmBridge::new(&widget).unwrap();

        let state = bridge.read_all_state();
        assert_eq!(state.len(), 3);
        assert_eq!(state.get("x"), Some(&Value::Int(1)));
        assert_eq!(state.get("y"), Some(&Value::Int(2)));
        assert_eq!(state.get("name"), Some(&Value::str("test")));
    }

    /// Audit B10(a): compiled `List<T>.new` / `[...]`-literal state fields hold
    /// the heap array id as a plain `Value::Int` (Plan 289 convention).
    /// read_all_state_materialized must inline them as `Value::Array` —
    /// previously only `Value::VmRef` fields were materialized, so
    /// autoui_state rendered 015's notes as `notes: 4000014 (int)` and
    /// desktop_mcp's element-count assertions went blind.
    #[test]
    fn test_read_all_state_materializes_int_handle_list() {
        let widget = make_test_widget("Store", vec![
            AuraStateDef {
                name: "notes".to_string(),
                type_info: Type::Unknown,
                initial: Expr::Int(0),
                decorators: vec![],
            },
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(7),
                decorators: vec![],
            },
        ]);
        let mut bridge = VmBridge::new(&widget).unwrap();
        let arr_id = bridge.vm.insert_heap_object(
            crate::vm::types::ListData::<Value> {
                elems: vec![Value::Int(1), Value::Int(2)],
                storage: None,
            },
        );
        bridge.write_state("notes", Value::Int(arr_id as i32)).unwrap();

        let state = bridge.read_all_state_materialized();
        assert_eq!(
            state.get("notes"),
            Some(&Value::Array(auto_val::Array { values: vec![Value::Int(1), Value::Int(2)] })),
            "Int-held list handle should materialize to an inline array"
        );
        // A genuine Int with no ListData behind it must stay untouched.
        assert_eq!(state.get("count"), Some(&Value::Int(7)));
    }

    #[test]
    fn test_read_all_state_after_write() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut bridge = VmBridge::new(&widget).unwrap();

        bridge.write_state("count", Value::Int(99)).unwrap();

        let state = bridge.read_all_state();
        assert_eq!(state.get("count"), Some(&Value::Int(99)));
    }

    #[test]
    fn test_call_handler_not_found() {
        let widget = make_test_widget("Counter", vec![]);
        let mut bridge = VmBridge::new(&widget).unwrap();

        let result = bridge.call_handler("NonExistent", &[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            VmBridgeError::HandlerNotFound(name) => assert_eq!(name, "NonExistent"),
            other => panic!("Expected HandlerNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_has_handler_absent() {
        // No handlers synthesized → has_handler is false for any name.
        let widget = make_test_widget("Counter", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        assert!(!bridge.has_handler("Inc"));
        assert!(bridge.handler_names().is_empty());
    }

    #[test]
    fn test_vm_access() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        // Verify VM is accessible
        assert_eq!(bridge.vm().heap_object_count(), 1); // state object
    }

    #[test]
    fn test_state_obj_id() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        // State object ID should be a valid heap object ID
        let id = bridge.state_obj_id();
        assert!(id >= 4000000); // Heap object IDs start at 4000000
    }

    #[test]
    fn test_extract_handler_name() {
        assert_eq!(extract_handler_name(".Inc"), "Inc");
        assert_eq!(extract_handler_name("Msg::Inc"), "Inc");
        assert_eq!(extract_handler_name("Inc"), "Inc");
        assert_eq!(extract_handler_name(".AddItem"), "AddItem");
        assert_eq!(extract_handler_name("Event::Click::Press"), "Press");
        // Plan 423 P5 续修:带参数列表的模式(016-calendar SelectDay 根因)。
        assert_eq!(extract_handler_name(".SelectDay(date)"), "SelectDay");
        assert_eq!(extract_handler_name("Msg::AddTodo(str)"), "AddTodo");
    }

    #[test]
    fn test_eval_expr() {
        let flash = crate::vm::virt_memory::VirtualFlash::new(64);
        let mut vm = AutoVM::new(flash, 64);
        assert_eq!(eval_expr_to_value(&Expr::Int(42), &mut vm), Value::Int(42));
        assert_eq!(eval_expr_to_value(&Expr::Double(3.14, "".into()), &mut vm), Value::Double(3.14f64));
        assert_eq!(eval_expr_to_value(&Expr::Bool(true), &mut vm), Value::Bool(true));
        assert_eq!(eval_expr_to_value(&Expr::Str("hi".into()), &mut vm), Value::str("hi"));
    }

    #[test]
    fn test_eval_expr_array() {
        let flash = crate::vm::virt_memory::VirtualFlash::new(64);
        let mut vm = AutoVM::new(flash, 64);
        let expr = Expr::Array(vec![Expr::Int(1), Expr::Int(2)]);
        let val = eval_expr_to_value(&expr, &mut vm);
        // Plan 420:数组字面量物化为原生 ListData 堆对象(VmRef)—— 与
        // handler 内局部列表同表示,len()/push()/[i] 等字节码操作才能作用
        // 于 state 字段。经 index_list_all 解回元素断言。
        match val {
            Value::VmRef(r) => {
                let elems = {
                    let obj = vm.get_heap_object(r.id as u64).expect("list heap object");
                    let guard = obj.read().unwrap();
                    use crate::vm::types::ListData;
                    match guard.as_any().downcast_ref::<ListData<Value>>() {
                        Some(list) => list.elems.clone(),
                        None => panic!("VmRef does not point at ListData<Value>"),
                    }
                };
                assert_eq!(elems.len(), 2);
                assert_eq!(elems[0], Value::Int(1));
                assert_eq!(elems[1], Value::Int(2));
            }
            other => panic!("Expected VmRef(list), got {:?}", other),
        }
    }

    #[test]
    fn test_multiple_writes() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut bridge = VmBridge::new(&widget).unwrap();

        // Simulate incrementing counter
        for i in 1..=5 {
            bridge.write_state("count", Value::Int(i)).unwrap();
        }

        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(5));
    }

    /// Plan 323 (Option B) end-to-end unit test: a handler synthesized as a REAL
    /// VM function and dispatched via `call_handler` actually mutates widget
    /// state through the heap object. Exercises state-ref rewrite (`.count` →
    /// `__state.count`), Codegen GET_FIELD/SET_FIELD on the state instance, and
    /// `call_fn_by_name` dispatch.
    #[test]
    fn test_handler_counter_increment() {
        use crate::ast::{Expr, Name, Stmt};
        use crate::aura::LogicPayload;
        use auto_val::Op;

        let mut widget = make_test_widget("Counter", vec![AuraStateDef {
            name: "count".to_string(),
            type_info: Type::Int,
            initial: Expr::Int(0),
            decorators: vec![],
        }]);

        // `.Inc -> { .count = .count + 1 }` parses as a single Asn expression stmt.
        let inc_body = vec![Stmt::Expr(Expr::Bina(
            Box::new(Expr::Ident(Name::from("count"))),
            Op::Asn,
            Box::new(Expr::Bina(
                Box::new(Expr::Ident(Name::from("count"))),
                Op::Add,
                Box::new(Expr::Int(1)),
            )),
        ))];
        widget
            .handlers
            .insert(".Inc".to_string(), LogicPayload::AstStmts(inc_body));

        let mut bridge = VmBridge::new(&widget).unwrap();
        assert!(bridge.has_handler("Inc"));

        // Initial state is 0.
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(0));

        // Dispatch the synthesized handler three times via the real VM.
        for _ in 0..3 {
            bridge.call_handler("Inc", &[]).unwrap();
        }
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(3));
    }

    /// Plan 448 B2: compound assignment on widget state inside a
    /// VM-synthesized handler (`.count += 1` / `count -= 2`). handler_codegen
    /// desugars `x op= e` to `x = x op e` before the state rewrite because
    /// VM codegen's compound-assignment path only accepts Ident LHS — before
    /// B2 either form aborted the whole widget's handler synthesis with
    /// "Compound assignment requires a variable on left side".
    #[test]
    fn test_handler_compound_assignment_state() {
        use crate::ast::{Expr, Name, Stmt};
        use crate::aura::LogicPayload;
        use auto_val::Op;

        let mut widget = make_test_widget("Counter", vec![AuraStateDef {
            name: "count".to_string(),
            type_info: Type::Int,
            initial: Expr::Int(10),
            decorators: vec![],
        }]);

        // `.Inc -> { .count += 1 }` — dot-shorthand LHS: Dot(self, count).
        let dot_body = vec![Stmt::Expr(Expr::Bina(
            Box::new(Expr::Dot(
                Box::new(Expr::Ident(Name::from("self"))),
                Name::from("count"),
            )),
            Op::AddEq,
            Box::new(Expr::Int(1)),
        ))];
        widget
            .handlers
            .insert(".Inc".to_string(), LogicPayload::AstStmts(dot_body));

        // `.Dec -> { count -= 2 }` — bare state-field ident LHS.
        let bare_body = vec![Stmt::Expr(Expr::Bina(
            Box::new(Expr::Ident(Name::from("count"))),
            Op::SubEq,
            Box::new(Expr::Int(2)),
        ))];
        widget
            .handlers
            .insert(".Dec".to_string(), LogicPayload::AstStmts(bare_body));

        let mut bridge = VmBridge::new(&widget).unwrap();
        assert!(bridge.has_handler("Inc"), "Inc synthesis must survive += ");
        assert!(bridge.has_handler("Dec"), "Dec synthesis must survive -=");

        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(10));

        for _ in 0..3 {
            bridge.call_handler("Inc", &[]).unwrap();
        }
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(13));

        bridge.call_handler("Dec", &[]).unwrap();
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(11));
    }

    /// Plan 448 B1 follow-up: the REAL iced/VM runtime builds its bridge via
    /// `new_from_decls` (decl-based synthesis — `run_file_dynamic_ui_inner` →
    /// `DynamicComponent::with_registry_and_imports_from_decls`), compiling
    /// handlers straight from the decl's on-block. Inline lambdas are minted
    /// at PARSE time into that on-block, so this path must synthesize
    /// `handler_<W>___evt_*` too — minting only at extraction left the VM
    /// without the functions and clicks silently no-oped (002-counter VM
    /// mode). call_handler_for mirrors the iced update dispatch exactly.
    #[test]
    fn test_inline_lambda_event_decl_based_synthesis() {
        use crate::parser::Parser;

        let src = concat!(
            "widget App {\n",
            "    model { var count int = 0 }\n",
            "    view {\n",
            "        row {\n",
            "            button \"+\" { onclick: () => {.count += 1} }\n",
            "            button \"-\" { onclick: () => {.count -= 1} }\n",
            "        }\n",
            "    }\n",
            "}\n"
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
            _ => None,
        }).expect("widget decl");

        let mut bridge = VmBridge::new_from_decls(&decl, &[], Vec::new(), &Default::default(), false)
            .expect("decl-based bridge");
        let state_obj_id = bridge.state_obj_id();

        assert!(bridge.has_handler("__evt_onclick_1"), "+ handler synthesized from the decl");
        assert!(bridge.has_handler("__evt_onclick_2"), "- handler synthesized from the decl");

        bridge
            .call_handler_for("App", "__evt_onclick_1", state_obj_id, &[])
            .unwrap();
        bridge
            .call_handler_for("App", "__evt_onclick_1", state_obj_id, &[])
            .unwrap();
        bridge
            .call_handler_for("App", "__evt_onclick_2", state_obj_id, &[])
            .unwrap();
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(1));
    }

    /// Plan 448 B1: inline-lambda event through the real pipeline — parse →
    /// extract (mints `__evt_*` handler + variant) → VmBridge synthesis →
    /// dispatch via the real VM. Proves the 002-counter shorthand shape runs
    /// on the VM/iced route end to end (B2's compound-assignment desugar
    /// included).
    #[test]
    fn test_inline_lambda_event_vm_dispatch() {
        use crate::parser::Parser;

        let src = concat!(
            "widget Counter {\n",
            "    model { var count int = 0 }\n",
            "    view {\n",
            "        row {\n",
            "            button \"+\" { onclick: () => {.count += 1} }\n",
            "            button \"-\" { onclick: () => {.count -= 1} }\n",
            "        }\n",
            "    }\n",
            "}\n"
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut bridge = VmBridge::new(&widget).unwrap();
        assert!(bridge.has_handler("__evt_onclick_1"), "+ handler synthesized");
        assert!(bridge.has_handler("__evt_onclick_2"), "- handler synthesized");

        for _ in 0..3 {
            bridge.call_handler("__evt_onclick_1", &[]).unwrap();
        }
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(3));

        bridge.call_handler("__evt_onclick_2", &[]).unwrap();
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(2));
    }

    #[test]
    fn test_converter_vm_dispatch() {
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        let front = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples/ui/003-converter/src/front");
        let app_src = std::fs::read_to_string(front.join("app.at")).expect("read app.at");
        let session = CompilerSession::ui();
        let mut parser = Parser::from(&app_src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut bridge = VmBridge::new(&widget).unwrap();
        println!("handlers: {:?}", bridge.handler_names());
        assert!(bridge.has_handler("__evt_oninput_1"));
        assert!(bridge.has_handler("__evt_oninput_2"));

        bridge.write_state("fahrenheit", Value::Double(323.0)).unwrap();
        bridge.call_handler("__evt_oninput_2", &[]).unwrap();
        let celsius = bridge.read_state("celsius").unwrap();
        match celsius {
            Value::Float(f) | Value::Double(f) => assert!((f - 161.67).abs() < 0.01, "expected ~161.67, got {}", f),
            other => panic!("expected float/double celsius, got {:?}", other),
        }
    }

    /// Plan 323 (Option B) → Plan 522 evolution: 016-calendar now derives its
    /// grid as WIDGET computed (`days => build_month_grid(...)`,
    /// `month_label => month_name(...)`) calling `use calendar_util:` fns;
    /// the store's ×4 .Rebuild recalc chain is deleted. Full-pipeline proof
    /// against the REAL sources through the production dynamic path (stores
    /// merged, module imports + aliases loaded, Init fired): the imported
    /// helpers execute in the VM (bare alias resolves), and month navigation
    /// mutates store state with no rebuild handler (computed re-derives).
    /// View-layer rendering of the computed grid is covered by the live
    /// `auto run -r vm` MCP snapshot check (AuraSnapshotBuilder is
    /// state-only and cannot see computed).
    /// Supersedes the pre-store-refactor harness that built a bare bridge
    /// from the widget alone — its Init handler calls `store.Init()`, which
    /// cannot compile without store context (064115a76 shape).
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn test_calendar_computed_grid_via_use_fns() {
        let mut dc = match crate::plan370_test_support::build_example_component("016-calendar") {
            Some(dc) => dc,
            None => {
                eprintln!("skipping calendar e2e (example sources unreadable)");
                return;
            }
        };

        // Store fields merged into root state after Init (D-GAP-4).
        for required in ["year", "month", "today", "selected_date"] {
            assert!(
                dc.bridge().read_state(required).is_ok(),
                "store field '{required}' missing from root state"
            );
        }

        // Imported helpers execute via their bare import aliases (the same
        // resolution the computed evaluator uses — PLAN-051 C3).
        let june = dc
            .bridge()
            .call_vm_fn("month_name", &[Value::Int(6)])
            .expect("month_name via bare alias");
        assert_eq!(june, Value::str("June"));

        let cells = dc
            .bridge()
            .call_vm_fn(
                "build_month_grid",
                &[Value::Int(2026), Value::Int(6), Value::str("2026-06-17"), Value::str("2026-06-17")],
            )
            .expect("build_month_grid via bare alias");
        let days: Vec<Value> = match &cells {
            Value::Array(arr) => arr.values.clone(),
            Value::Int(id) => dc.bridge().index_list_all(*id as usize),
            Value::VmRef(r) => dc.bridge().index_list_all(r.id),
            other => panic!("grid should be an array, got {other:?}"),
        };
        assert_eq!(days.len(), 42, "grid is 42 cells (6 weeks)");

        // June 2026: the 1st is a Monday → Sunday-first grid starts with the
        // May-31 tail, then June 1..30, then July 1..11.
        let cell_str = |i: usize| -> String {
            let obj = dc.bridge().materialize_obj_ref(&days[i]);
            match obj {
                Value::Obj(o) => o.get_str("label").unwrap_or_default().to_string(),
                other => format!("{other:?}"),
            }
        };
        assert_eq!(cell_str(0), "31", "cell 0 is the May-31 tail");
        assert_eq!(cell_str(1), "1", "cell 1 is June 1");
        assert_eq!(cell_str(30), "30", "cell 30 is June 30");
        assert_eq!(cell_str(31), "1", "cell 31 is the July-1 filler");
        // The per-cell Tailwind class (day_style) materialized correctly:
        // cells[17] = June 17 is BOTH today and selected (both seeds are
        // "2026-06-17") → selected wins by precedence.
        match dc.bridge().materialize_obj_ref(&days[17]) {
            Value::Obj(o) => {
                let class = o.get_str("cell_class").expect("cell_class on today cell");
                assert!(
                    class.contains("bg-primary text-primary-foreground"),
                    "selected cell highlight expected (selected > today), got '{class}'"
                );
            }
            other => panic!("cell 17 should materialize to Obj, got {other:?}"),
        }
        // A plain current-month cell (June 2) keeps the default class.
        match dc.bridge().materialize_obj_ref(&days[2]) {
            Value::Obj(o) => {
                let class = o.get_str("cell_class").expect("cell_class on plain cell");
                assert!(
                    class.contains("hover:bg-accent"),
                    "plain cell default class expected, got '{class}'"
                );
            }
            other => panic!("cell 2 should materialize to Obj, got {other:?}"),
        }

        // Behavior: one PrevMonth dispatch rolls the store month with no
        // rebuild handler (the recalc chain is gone — computed re-derives).
        dc.on_with_input("PrevMonth", None);
        assert_eq!(
            dc.bridge().read_state("month").unwrap(),
            Value::Int(5),
            "PrevMonth must roll 2026-06 → 2026-05"
        );
        let may = dc
            .bridge()
            .call_vm_fn("month_name", &[Value::Int(5)])
            .expect("month_name after roll");
        assert_eq!(may, Value::str("May"));
    }

    /// Plan 522 T4: 024-charts 的 donut 组件经 `use chart_geom: dc, ds`
    /// 导入几何 helper(components/ 包通道)。VM 装载对包组件文件自身的
    /// use 依赖做收集 + 裸名别名(与根文件 use 同规则)——dc/ds 在 VM 内
    /// 可按裸名调用即证明该链路(437 §0.6.E-3 绕过点回正的 VM 半边)。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn test_plan522_024_chart_geom_helpers_in_vm() {
        let dc = match crate::plan370_test_support::build_example_component("024-charts") {
            Some(dc) => dc,
            None => {
                eprintln!("skipping 024 e2e (example sources unreadable)");
                return;
            }
        };
        let raw = dc
            .bridge()
            .call_vm_fn("dc", &[Value::Double(0.0)])
            .expect("dc via bare alias (package component use dep)");
        let cos0 = match raw {
            Value::Float(f) | Value::Double(f) => f,
            Value::Int(i) => i as f64,
            other => panic!("dc(0.0) should be numeric, got {other:?}"),
        };
        assert!(
            (cos0 - 1.0).abs() < 1e-9,
            "dc(0.0) == 1.0, got {cos0:?} (raw {raw:?})"
        );
        let raw_sin = dc
            .bridge()
            .call_vm_fn("ds", &[Value::Double(1.0)])
            .expect("ds via bare alias");
        let sin1 = match raw_sin {
            Value::Float(f) | Value::Double(f) => f,
            Value::Int(i) => i as f64,
            other => panic!("ds(1.0) should be numeric, got {other:?}"),
        };
        assert!(
            (sin1 - 0.8414709848078965).abs() < 1e-9,
            "ds(1.0) == sin(1), got {sin1:?} (raw {raw_sin:?})"
        );
    }

    /// Audit B12(b) reproducer: a store handler that (1) calls an imported fn
    /// with a STRING state-field argument (discarding the result), then
    /// (2) assigns a state field. In 013-todo's AddTodo this exact shape lost
    /// the `.todos = …` write — SET_FIELD's object-id pop received a string
    /// nanbox (negative pool index) instead of the state object id, so the
    /// write silently landed nowhere.
    #[test]
    fn repro_b12_state_write_after_string_arg_call() {
        use crate::ast::Stmt;
        use crate::parser::Parser;
        use crate::session::CompilerSession;

        let src = r#"
type Todo { id int; text str; done bool }

var todos List<Todo> = List<Todo>.new([
    Todo { id: 0, text: "one", done: true },
    Todo { id: 1, text: "two", done: false },
])

var nextid int = 2

fn create_todo(text str) Todo {
    var todo = Todo { id: nextid, text: text, done: false }
    todos.push(todo)
    nextid = nextid + 1
    return todo
}

fn all_todos() []Todo {
    return todos.to_array()
}

fn list_todos() []Todo {
    return all_todos()
}

widget Store {
    msg Msg { Add }
    model {
        var input str = "seed"
        var saved str = "none"
        var list int = 0
        var count int = 0
    }
    view { col { text .saved } }
    on {
        .Add -> {
            if .input != "" {
                create_todo(.input)
                .list = list_todos()
                .saved = "done"
                .count = 7
            }
        }
    }
}
"#;
        let session = CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("repro src should parse");
        let mut widget = None;
        let mut import_stmts: Vec<Stmt> = Vec::new();
        for s in ast.stmts {
            match s {
                Stmt::WidgetDecl(d) => widget = crate::aura::extract_widget_from_decl(&d).ok(),
                Stmt::Fn(_) | Stmt::TypeDecl(_) | Stmt::Store(_) => import_stmts.push(s),
                _ => {}
            }
        }
        let widget = widget.expect("src must declare a widget");
        let mut bridge = VmBridge::new_with_imports(&widget, import_stmts)
            .expect("bridge builds");
        bridge.run_module_init().expect("module init");

        bridge.call_handler("Add", &[]).expect("Add handler runs");

        let saved = bridge.read_state("saved").unwrap();
        assert_eq!(
            saved,
            Value::str("done"),
            "state write after a string-arg call must land (B12: write was lost)"
        );
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(7));
        // The list write must land a real value (handle/int), not stay 0.
        let list = bridge.read_state("list").unwrap();
        assert!(
            !matches!(list, Value::Int(0)),
            "`.list = list_todos()` must assign a value, got {:?}",
            list
        );
        // B12(ii) refinement: the pushed element must keep its GLOBAL-sourced
        // field — `Todo { id: nextid, ... }` with nextid=2 must yield id=2,
        // not 0 (GUI probe: constructed Obj carried id: 0 while nextid was
        // verifiably 4 at LOAD_GLOBAL time). List must hold 3 (2 seeds + 1).
        let elems = bridge.read_state_as_vec("list").expect("list readable as vec");
        assert_eq!(elems.len(), 3, "2 seeds + 1 new, got {}", elems.len());
        let new_id = elems
            .iter()
            .rev()
            .find_map(|e| match e {
                Value::VmRef(r) => bridge
                    .vm
                    .get_heap_object(r.id as u64)
                    .and_then(|o| {
                        let g = o.read().ok()?;
                        let inst = g
                            .as_any()
                            .downcast_ref::<crate::vm::generic_registry::GenericInstanceData>()?;
                        inst.get_field(0).cloned()
                    }),
                Value::Obj(o) => o.get_str("id").map(|s| Value::str(s)),
                _ => None,
            });
        match new_id {
            Some(Value::Int(id)) => assert_eq!(id, 2, "constructed Todo.id must be nextid (2)"),
            other => panic!("expected constructed Todo with id, got {:?}", other),
        }
    }

    /// Repro for the SelectDay panic (bp - actual_offset overflow on param
    /// access). Mirrors the real click dispatch: build via the production
    /// path (stores merged — the bare-bridge form predates the store
    /// refactor and no longer compiles Init), then dispatch SelectDay with a
    /// date-string payload. Plan 522: `.days` left state for a widget
    /// computed, so the assertion set is the store fields the handler writes.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn repro_selectday_panic() {
        let mut dc = match crate::plan370_test_support::build_example_component("016-calendar") {
            Some(dc) => dc,
            None => {
                eprintln!("skipping calendar e2e (example sources unreadable)");
                return;
            }
        };
        // This is what a day-cell click dispatches:
        dc.bridge_mut()
            .call_handler("SelectDay", &[Value::str("2026-06-25")])
            .expect("SelectDay should run without panicking");
        assert_eq!(
            dc.bridge().read_state("selected_date").unwrap(),
            Value::str("2026-06-25"),
            "SelectDay must write the payload through to the store field"
        );
        assert_eq!(
            dc.bridge().read_state("month").unwrap(),
            Value::Int(6),
            "SelectDay must not roll the month"
        );
    }

    /// receives the value dispatched via call_handler's args — both int and
    /// string payloads. This is the contract `onclick: .SelectDay(cell.date)`
    /// depends on (view resolves cell.date → args; on() forwards args).
    #[test]
    fn test_handler_receives_declared_param() {
        use crate::ast::{Expr, Name, Stmt};
        use crate::aura::LogicPayload;
        use auto_val::Op;

        // widget with an int field and a str field
        let mut widget = make_test_widget("App", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(0),
                decorators: vec![],
            },
            AuraStateDef {
                name: "label".to_string(),
                type_info: Type::StrOwned,
                initial: Expr::Str(String::new().into()),
                decorators: vec![],
            },
        ]);

        // `.SetCount(n) -> { .count = n }` — `n` is a handler param (not a state
        // field), so the rewriter must leave it alone and Codegen resolves it as
        // the declared param.
        widget.handlers.insert(
            ".SetCount".to_string(),
            LogicPayload::AstStmts(vec![Stmt::Expr(Expr::Bina(
                Box::new(Expr::Ident(Name::from("count"))),
                Op::Asn,
                Box::new(Expr::Ident(Name::from("n"))),
            ))]),
        );
        widget
            .handler_params
            .insert(".SetCount".to_string(), vec!["n".to_string()]);

        // `.SetLabel(s) -> { .label = s }`
        widget.handlers.insert(
            ".SetLabel".to_string(),
            LogicPayload::AstStmts(vec![Stmt::Expr(Expr::Bina(
                Box::new(Expr::Ident(Name::from("label"))),
                Op::Asn,
                Box::new(Expr::Ident(Name::from("s"))),
            ))]),
        );
        widget
            .handler_params
            .insert(".SetLabel".to_string(), vec!["s".to_string()]);

        let mut bridge = VmBridge::new(&widget).unwrap();
        assert!(bridge.has_handler("SetCount"));
        assert!(bridge.has_handler("SetLabel"));

        // Int payload: n = 42 → count = 42
        bridge
            .call_handler("SetCount", &[Value::Int(42)])
            .expect("SetCount runs");
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(42));

        // String payload: s = "2026-06-17" → label = "2026-06-17"
        bridge
            .call_handler("SetLabel", &[Value::str("2026-06-17")])
            .expect("SetLabel runs");
        match bridge.read_state("label").unwrap() {
            Value::Str(s) => assert_eq!(s.as_str(), "2026-06-17"),
            other => panic!("label should be the dispatched string, got {:?}", other),
        }
    }

    /// Plan 323 (Option B) regression smoke against the REAL 002-counter
    /// source: parse → extract widget → VmBridge → dispatch the inline-lambda
    /// handlers via the real VM → read `.count`. Confirms the canonical
    /// handler-mutation example works end-to-end through genuine
    /// Codegen/AutoVM dispatch (Plan 448 B1: the example is the shorthand
    /// form, so this also covers anonymous-event minting on the VM route).
    #[test]
    fn test_counter_002_handlers_mutate_state() {
        use crate::parser::Parser;
        use crate::session::CompilerSession;

        let app_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples/ui/002-counter/src/front/app.at");
        let app_src = match std::fs::read_to_string(&app_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping 002-counter smoke (app.at unreadable): {}", e);
                return;
            }
        };

        let session = CompilerSession::ui();
        let mut parser = Parser::from(app_src.as_str()).with_session(session);
        let ast = parser.parse().expect("002-counter app.at should parse");
        let widget = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => {
                    crate::aura::extract_widget_from_decl(d).ok()
                }
                _ => None,
            })
            .expect("002-counter must declare a widget");

        let mut bridge = VmBridge::new(&widget).expect("bridge builds");
        // Plan 448 B1: the example now uses inline lambdas
        // (`onclick: () => {.count ± 1}`), so the handlers are the minted
        // anonymous events — declaration order: 1 = "-", 2 = "Reset", 3 = "+".
        assert!(bridge.has_handler("__evt_onclick_1"));
        assert!(bridge.has_handler("__evt_onclick_2"));
        assert!(bridge.has_handler("__evt_onclick_3"));

        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(0));

        bridge.call_handler("__evt_onclick_3", &[]).unwrap();
        bridge.call_handler("__evt_onclick_3", &[]).unwrap();
        bridge.call_handler("__evt_onclick_1", &[]).unwrap();
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(1));

        bridge.call_handler("__evt_onclick_2", &[]).unwrap();
        assert_eq!(bridge.read_state("count").unwrap(), Value::Int(0));
    }

    /// Plan 327 (recursive import loading): drive import collection from the
    /// REAL 016-calendar `use` clause — `use calendar_util: build_month_grid,
    /// month_name, add_months_year, add_months_month` — which does NOT name
    /// `weekday_of`/`days_in_month`/`format_date`/`is_leap`, yet `build_month_grid`
    /// calls them. The recursive collector must pull the whole module (intra-
    /// module callees) so Init links. This is the exact bug that made 016-calendar
    /// fail to start in VM render mode ("Undefined symbol: weekday_of").
    #[test]
    fn test_calendar_imports_resolve_intra_module_callees() {
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        use crate::use_scanner::scan_use_statements;

        let front = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples/ui/016-calendar/src/front");
        let app_src = match std::fs::read_to_string(front.join("app.at")) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping (app.at unreadable): {}", e);
                return;
            }
        };

        // Resolve the calendar_util module exactly as run_file_dynamic_ui does.
        let calendar_util_path = crate::resolve_module_path(&front, "calendar_util")
            .expect("calendar_util.at must resolve");

        // Recursive collection from calendar_util.at (mirrors production).
        let mut visited = std::collections::HashSet::new();
        let mut seen = std::collections::HashSet::new();
        let mut import_stmts: Vec<crate::ast::Stmt> = Vec::new();
        let mut session = crate::compile::CompileSession::new();
        crate::collect_module_imports(
            &calendar_util_path,
            &mut visited,
            &mut import_stmts,
            &mut seen,
            &mut session,
            None, // PR-6: no scenario override in test
        );

        // The non-imported callees MUST be present (the bug was their absence).
        // Plan 339: collected Fns are renamed to `<module>.<name>` (e.g.
        // `calendar_util.build_month_grid`), so match by the qualified suffix
        // rather than the bare name.
        let names: std::collections::HashSet<String> = import_stmts
            .iter()
            .filter_map(|s| crate::stmt_symbol_name(s))
            .collect();
        let module_prefix = "calendar_util.";
        for required in [
            "build_month_grid",
            "weekday_of",
            "days_in_month",
            "format_date",
            "is_leap",
        ] {
            let present = names.iter().any(|n| {
                n == required || n.strip_prefix(module_prefix) == Some(required)
            });
            assert!(
                present,
                "recursive import collection must include `{}` (callee of build_month_grid not named in the use clause). collected = {:?}",
                required, names
            );
        }

        // And the production path (stores + imports + aliases, Plan 522 shape)
        // drives the imported helpers end to end — see
        // test_calendar_computed_grid_via_use_fns for the full grid assertions
        // (42 cells / navigation); here just prove the component builds and
        // the module-qualified fn executes.
        let dc = match crate::plan370_test_support::build_example_component("016-calendar") {
            Some(dc) => dc,
            None => {
                eprintln!("skipping (016 sources unreadable)");
                return;
            }
        };
        let cells = dc
            .bridge()
            .call_vm_fn(
                "build_month_grid",
                &[Value::Int(2026), Value::Int(6), Value::str("2026-06-17"), Value::str("2026-06-17")],
            )
            .expect("build_month_grid executes via the production import chain");
        let count = match &cells {
            Value::Array(arr) => arr.values.len(),
            Value::Int(id) => dc.bridge().index_list_all(*id as usize).len(),
            Value::VmRef(r) => dc.bridge().index_list_all(r.id).len(),
            other => panic!("grid should be an array, got {other:?}"),
        };
        assert_eq!(count, 42);

        // Silence unused warning for scan_use_statements when the body returns early.
        let _ = scan_use_statements;
    }
}

// ── Plan 423 P5(测试债诊断):字节码诊断辅助 ──────────────────────────────

impl VmBridge {
    /// 函数表(name → 地址,按地址排序)+ 指定地址附近的反汇编。
    /// 仅供测试/调试(plan370_015 InvalidOpCode(255) 归因)。
    /// Plan 423 P5:函数表(Vec<(name, addr)>,按地址排序)—— 导出对齐
    /// 不变量测试用(每个导出地址必须是 FN_PROLOG)。
    pub fn debug_fn_table(&self) -> Vec<(String, u32)> {
        let mut names: Vec<(String, u32)> = self
            .vm
            .flash
            .exports_by_name
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        names.sort_by_key(|(_, a)| *a);
        names
    }

    /// 原始字节读取(不变量测试用)。
    pub fn debug_byte(&self, addr: u32) -> u8 {
        self.vm.flash.memory.get(addr as usize).copied().unwrap_or(0xEE)
    }

    /// PLAN-046 (auto-musk T10): total linked bytecode size in bytes.
    pub fn bytecode_len(&self) -> usize {
        self.vm.flash.memory.len()
    }

    /// Plan 423 P5:原始字节十六进制(反汇编起点错位时会误导,字节不会)。
    pub fn debug_raw(&self, addr: u32, len: usize) -> String {
        let mut out = format!("== raw 0x{:04x}..0x{:04x} ==\n", addr, addr as usize + len);
        for i in 0..len {
            let a = addr as usize + i;
            let b = self.vm.flash.memory.get(a).copied().unwrap_or(0xEE);
            out.push_str(&format!("{:02x} ", b));
            if i % 16 == 15 {
                out.push('\n');
            }
        }
        out
    }

    pub fn debug_disasm(&self, addr: u32, before: usize, after: usize) -> String {
        use crate::vm::disasm::Disassembler;
        let mut names: Vec<(String, u32)> = self
            .vm
            .flash
            .exports_by_name
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        names.sort_by_key(|(_, a)| *a);
        let mut out = String::from("== fn table ==\n");
        for (n, a) in &names {
            out.push_str(&format!("  0x{:04x} {}\n", a, n));
        }
        let dis = Disassembler::new(&self.vm.flash);
        let start = (addr as usize).saturating_sub(before);
        let end = (addr as usize) + after;
        out.push_str(&format!("== disasm 0x{:04x}..0x{:04x} ==\n", start, end));
        for l in dis.disassemble_range(start, end) {
            out.push_str(&format!(
                "  0x{:04x} {} {}\n",
                l.offset,
                l.mnemonic,
                l.operands
            ));
        }
        out
    }
}
