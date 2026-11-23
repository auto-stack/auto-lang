//! PLAN-1106 T-03/T-04：AV **增量事件会话**——组件面的可保留运行时。
//!
//! 结构（T-01 冻结的候选①）：`compile_component_event_program` 一次编译
//! （字段 cell + init/seed + 每 handler 单事件入口 + 文本叶纯读函数，经同一
//! 共享前门与唯一 verify）；每个会话/实例 = 一次 `link_with_globals`
//! （共享 `Arc<FunctionCode>`，独立全局 cell 状态根）+ init 链每代一次；
//! 每个被接纳的事件 = 恰一次 `execute_program_entry_args` 目标 handler 派发
//! （真执行器、真实参绑定）；读取 = `global_cells()` 纯观察（零派发）；
//! 释放 = drop 快照（cell 随之释放）。
//!
//! 事务性（T-04）：handler 体在编译期已是"locals 先行、末尾统一提交"
//! （业务中途 trap ⇒ 零提交）；会话层再按派发前备份/失败恢复兜底预算边界
//! 的残余半提交。序号纪律：宿主生成的单调 sequence，≤ 水位的重投不重复
//! 执行（返回上次结果），新序号正常执行；同名合法新事件各有各的序号。
//!
//! 观察口（AC-06 的执行层 oracle）：[`SessionAudit`] 全部在生产者/执行器
//! 真实调用边界计数；`entry_seqs` 直接取自执行器进程序号（`EXECUTION_SEQ`），
//! 不是 wrapper 按事件数自增。

use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::function_code::FunctionCode;
use ace_av_core::program_snapshot::ProgramSnapshot;
use ace_av_core::span_map::SpanSegments;
use ace_av_core::{
    execute_program_entry_args, execute_program_http_resumable, EntryArg, HttpEntryOutcome,
};

/// PLAN-108（T-03c M3）：派发结果——`SuspendedHttp`=handler 在 HttpCall
/// 挂起（帧保留在会话，经 [`ComponentEventSession::resume_http`] 续跑）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchState {
    Settled,
    SuspendedHttp,
}
use ace_core::source::age_component::{
    compile_component_event_program, ComponentEventProgram as CoreEventProgram,
};
use ace_core::source::age_h0::LegacyWidgetAlias;
use std::sync::Arc;

// 消费者（UI 桥）经本模块取值域，不必直接依赖 ace-av 的场景观察面。
pub use crate::age_component::ComponentFieldValue;
// PLAN-108（T-03c M2）：宿主 HTTP provider trait 随会话面再出口（UI 桥注入）。
pub use ace_av_core::http_client::{HttpFailure, HttpProvider};

/// 会话级观察口（执行层 oracle；真实调用边界计数）。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SessionAudit {
    /// 被接纳事件的 handler 入口总数（= 真实派发次数）。
    pub handler_entries: u64,
    /// 按消息名的入口计数（oracle 断言用）。
    pub handler_entries_by_msg: Vec<(String, u64)>,
    /// init 链运行次数（每会话代次恰 1）。
    pub init_runs: u64,
    /// seed 覆盖派发次数（≤1）。
    pub seed_calls: u64,
    /// 文本叶求值派发次数（纯读；不属业务 handler）。
    pub text_evals: u64,
    /// 效果前拒绝（未知消息/arity/类型错/执行失败）计数。
    pub rejected: u64,
    /// 同成功事件重投被忽略的次数（不重复执行）。
    pub duplicate_ignored: u64,
    /// 会话已释放（drop 快照）。
    pub released: bool,
    /// 每次真实派发的执行器进程序号（执行层证据）。
    pub entry_seqs: Vec<u64>,
}

/// 单个 handler 的运行时入口描述。
#[derive(Clone, Debug)]
pub struct HandlerEntry {
    pub msg: String,
    pub def_id: String,
    pub payload: Vec<String>,
    /// PLAN-108（T-03c M3）：源级赋值字段清单（handler_assigns 的字段名）——
    /// http 挂起的保留-重播种用（统一提交覆盖并发提交）。
    pub assigns: Vec<String>,
}

/// 一次被接纳事件的结果（差量面由调用方从 [`ComponentEventSession::read_states`] 取）。
#[derive(Debug, PartialEq)]
pub enum EventOutcome {
    Dispatched,
    /// 同序号重投：不重复执行，返回上次已提交状态。
    Duplicate,
}

/// 编译一次、可开多个独立会话的组件事件程序。
pub struct ComponentEventSessionProgram {
    src: String,
    module_path: String,
    codes: Vec<Arc<FunctionCode>>,
    globals: Vec<ace_av_core::mir::MirGlobal>,
    fields: Vec<(String, String)>,
    handlers: Vec<HandlerEntry>,
    /// 字段 i → 全局 cell 下标（按 cell 名映射，不假设声明序）。
    cell_of_field: Vec<usize>,
    init_def_id: String,
    seed_def_id: String,
    text_def_ids: Vec<String>,
    /// 文本叶求值函数是否可用（states-only 边界内为 false，原因留痕）。
    pub texts_available: bool,
    pub texts_unavailable_reason: Option<String>,
    /// 真实编译次数（含 states-only 重试；oracle：热事件不得增长）。
    pub compile_count: u64,
    links: std::sync::atomic::AtomicU64,
    next_generation: std::sync::atomic::AtomicU64,
}

/// lower+admit 的中间产物（会话程序构建用）。
struct Built {
    codes: Vec<Arc<FunctionCode>>,
    globals: Vec<ace_av_core::mir::MirGlobal>,
    cell_of_field: Vec<usize>,
}

fn format_diags(ds: &[ace_core::atom_text::Diagnostic]) -> String {
    ds.iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect::<Vec<_>>()
        .join("; ")
}

/// 值域 → 载荷类型拼写（先验核对用；声明拼写是权威）。
fn value_matches_spelling(v: &ComponentFieldValue, spelling: &str) -> bool {
    match (v, spelling) {
        (ComponentFieldValue::Int(_), "int" | "i32") => true,
        (ComponentFieldValue::Bool(_), "bool") => true,
        (ComponentFieldValue::Str(_), "str" | "Str") => true,
        (ComponentFieldValue::F32(_), "f32" | "float") => true,
        _ => false,
    }
}

pub(crate) fn field_value_to_arg(v: &ComponentFieldValue) -> EntryArg<'static> {
    match v {
        ComponentFieldValue::Int(i) => EntryArg::Value(Value::I32(*i)),
        ComponentFieldValue::Bool(b) => EntryArg::Value(Value::Bool(*b)),
        ComponentFieldValue::F32(f) => EntryArg::Value(Value::F32(*f)),
        // owned str 不是合法入口实参：按文本车道（借用 StrView，本执行 arena）。
        ComponentFieldValue::Str(s) => {
            EntryArg::Text(std::borrow::Cow::Owned(s.as_bytes().to_vec()))
        }
    }
}

fn cell_value_to_field(v: &Value) -> Result<ComponentFieldValue, String> {
    Ok(match v {
        Value::I32(i) => ComponentFieldValue::Int(*i),
        Value::Bool(b) => ComponentFieldValue::Bool(*b),
        Value::F32(f) => ComponentFieldValue::F32(*f),
        Value::Str(h) => ComponentFieldValue::Str(
            String::from_utf8(h.bytes().to_vec())
                .map_err(|e| format!("cell text not UTF-8: {e}"))?,
        ),
        other => {
            return Err(format!(
                "global cell carries value kind {} outside the component field domain",
                other.type_name()
            ))
        }
    })
}

fn cell_value_to_arg(v: &Value) -> EntryArg<'static> {
    match v {
        Value::Str(h) => EntryArg::Text(std::borrow::Cow::Owned(h.bytes().to_vec())),
        other => EntryArg::Value(other.clone()),
    }
}

impl ComponentEventSessionProgram {
    /// 编译（一次）：先带文本叶；lowering 失败（f-string 叶在 av.owned/1 的
    /// 既有边界）时按 states-only 重试并把原因留痕——与场景车道的
    /// states-only 边界同一形状，不是静默 fallback。
    /// PLAN-117 诊断：函数码（abc 解码单元）只读访问（MIR 层 trap-edge
    /// 勘察用；不属交付 API 面，随本计划的探针生命周期）。
    pub fn codes(&self) -> &[std::sync::Arc<FunctionCode>] {
        &self.codes
    }

    pub fn compile(src: &str, module_path: &str, alias: LegacyWidgetAlias) -> Result<Self, String> {
        // PLAN-117（B-05 S1b）：json.value/1 随默认授予集（checked 域 JSON 值
        // 面的组件事件道标准能力；与 http.client/1 的生产授予同纪律——后者
        // 在桥按需授予，这里 http 未入默认集因 provider 按注册生效；json
        // 解析纯本域无宿主依赖，默认集安全）。
        let caps = ["ac.runtime.text.v1".to_string(), "json.value/1".to_string()];
        let attempt = |emit_texts: bool| -> Result<CoreEventProgram, String> {
            let core =
                compile_component_event_program(src, module_path, alias, &caps, None, emit_texts)
                    .map_err(|ds| format_diags(&ds))?;
            // lowering 先探（同一 producer 真跑；失败即本形态不可降落）。
            Self::lower_and_admit(&core).map(|_| core)
        };
        match attempt(true) {
            Ok(core) => Self::build(core, src, module_path, None, 1),
            Err(full_err) => {
                let core = attempt(false).map_err(|states_err| {
                    format!("event program compile failed (full: {full_err}; states-only: {states_err})")
                })?;
                Self::build(core, src, module_path, Some(full_err), 2)
            }
        }
    }

    fn build(
        core: CoreEventProgram,
        src: &str,
        module_path: &str,
        texts_unavailable_reason: Option<String>,
        compile_count: u64,
    ) -> Result<Self, String> {
        let built = Self::lower_and_admit(&core)?;
        // PLAN-108：源级赋值字段清单（msg → 赋值字段名）——供 http 挂起的
        // 保留-重播种（统一提交模型的并发提交覆盖补齐）。
        let assigns_by_msg: std::collections::HashMap<String, Vec<String>> =
            ace_core::source::age_component::project_component_candidates(src)
                .ok()
                .and_then(|nodes| nodes.into_iter().next())
                .map(|node| {
                    node.handlers
                        .into_iter()
                        .map(|h| {
                            (
                                h.msg,
                                h.assigns.into_iter().map(|a| a.field).collect(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
        Ok(Self {
            src: src.to_string(),
            module_path: module_path.to_string(),
            codes: built.codes,
            globals: built.globals,
            fields: core.fields.clone(),
            handlers: core
                .handlers
                .iter()
                .map(|h| HandlerEntry {
                    msg: h.msg.clone(),
                    def_id: h.def_id.clone(),
                    payload: h.payload.clone(),
                    assigns: assigns_by_msg.get(&h.msg).cloned().unwrap_or_default(),
                })
                .collect(),
            cell_of_field: built.cell_of_field,
            init_def_id: core.init_def_id.clone(),
            seed_def_id: core.seed_def_id.clone(),
            text_def_ids: core.text_def_ids.clone(),
            texts_available: texts_unavailable_reason.is_none() && !core.text_def_ids.is_empty(),
            texts_unavailable_reason,
            compile_count,
            links: std::sync::atomic::AtomicU64::new(0),
            next_generation: std::sync::atomic::AtomicU64::new(1),
        })
    }

    fn lower_and_admit(core: &CoreEventProgram) -> Result<Built, String> {
        let mir = ace_lowering::lower_checked_owned_with_init(
            &core.checked,
            &core.init_def_id,
            0,
            &[core.init_def_id.clone()],
        )
        .map_err(|e| format!("lower event program: {e:?}"))?;
        let globals = mir.globals.clone();
        // 字段 → cell 下标（按名；init 引用全部 cell ⇒ 必在）。
        let mut cell_of_field = Vec::with_capacity(core.cell_names.len());
        for name in &core.cell_names {
            let idx = globals
                .iter()
                .position(|g| &g.name == name)
                .ok_or_else(|| format!("event cell `{name}` missing from lowered globals"))?;
            cell_of_field.push(idx);
        }
        let wire = crate::wire::WireCodec::LegacyAbc;
        let mut codes = Vec::new();
        for unit in crate::cache::split_units(&mir).map_err(|e| format!("split units: {e}"))? {
            let bytes = wire
                .encode(&unit)
                .map_err(|e| format!("encode `{}`: {e:?}", unit.funcs[0].name))?;
            let loaded = wire
                .admit_bytes(&bytes, &unit.owner)
                .map_err(|e| format!("admit `{}`: {e:?}", unit.funcs[0].name))?;
            codes.push(Arc::new(
                FunctionCode::admit_unit(loaded, 0).map_err(|e| format!("admit unit: {e}"))?,
            ));
        }
        Ok(Built {
            codes,
            globals,
            cell_of_field,
        })
    }

    /// 源身份（诊断/证据面）。
    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn module_path(&self) -> &str {
        &self.module_path
    }

    /// 字段（名, 类型拼写）声明序。
    pub fn fields(&self) -> &[(String, String)] {
        &self.fields
    }

    /// handler 入口表（消息名 → def id + 载荷拼写）。
    /// PLAN-108（T-03c M2）：本程序是否含 Http 效应（任一单元 MIR 有
    /// HttpCall）——桥据此把派发路由到 worker 线程（UI 线程不阻塞）。
    pub fn has_http_effects(&self) -> bool {
        self.codes.iter().any(|code| {
            code.unit().module().funcs.iter().any(|f| {
                f.blocks
                    .iter()
                    .any(|b| b.instrs.iter().any(|i| matches!(i, ace_av_core::mir::MirInstr::HttpCall { .. })))
            })
        })
    }

    pub fn handlers(&self) -> &[HandlerEntry] {
        &self.handlers
    }

    /// 已发生的会话链接次数（每开一个会话 +1；热事件不得增长）。
    pub fn link_count(&self) -> u64 {
        self.links.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 打开一个新会话（独立状态根）：link → init（声明初值，每代一次）→
    /// 可选 props 种子（seed 全量覆盖——未覆盖字段回填当前 cell 值）。
    pub fn open_session(
        &self,
        initials: &[(String, ComponentFieldValue)],
    ) -> Result<ComponentEventSession, String> {
        // 先验：种子字段存在 + 类型匹配（不匹配即拒，零链接零初始化）。
        for (name, value) in initials {
            let Some((_, spelling)) = self.fields.iter().find(|(n, _)| n == name) else {
                return Err(format!(
                    "source.age.component-unknown-field: no model field named `{name}` to seed"
                ));
            };
            if !value_matches_spelling(value, spelling) {
                return Err(format!(
                    "source.age.component-initial-type: seed for `{name}` does not match declared type `{spelling}`"
                ));
            }
        }
        let generation = self
            .next_generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.links
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let span_maps = (0..self.codes.len())
            .map(|_| SpanSegments::default())
            .collect();
        let snap = ProgramSnapshot::link_with_globals(
            self.codes.clone(),
            &self.init_def_id,
            generation,
            span_maps,
            self.globals.clone(),
            &[self.init_def_id.clone()],
        )
        .map_err(|e| format!("link event session: {e}"))?;
        let mut session = ComponentEventSession {
            snap: Some(snap),
            generation,
            fields: self.fields.clone(),
            handlers: self.handlers.clone(),
            cell_of_field: self.cell_of_field.clone(),
            text_def_ids: self.text_def_ids.clone(),
            texts_available: self.texts_available,
            watermark: None,
            last_states: Vec::new(),
            audit: SessionAudit::default(),
            http: None,
            http_wall: None,
            suspended: None,
            suspended_preserve: Vec::new(),
            seed_def_id: self.seed_def_id.clone(),
        };
        // init 链（首次派发触发；每代一次）。
        let EntryResult::Done(out, trace) = session.entry(&self.init_def_id, &[]) else {
            return Err("init entry unexpectedly suspended on http".to_string());
        };
        if !matches!(out, ExecOutcome::Success { .. }) {
            return Err(format!("event session init did not succeed: {out:?}"));
        }
        session.audit.init_runs = 1;
        session.audit.entry_seqs.push(trace.execution_seq);
        // props 种子（有覆盖才派发；未覆盖字段回填当前 cell 值）。
        if !initials.is_empty() {
            let snap = session.snap.as_ref().expect("just linked");
            let cells = snap.global_cells();
            let mut args: Vec<EntryArg<'static>> = Vec::with_capacity(self.fields.len());
            for (i, (name, _)) in self.fields.iter().enumerate() {
                match initials.iter().find(|(n, _)| n == name) {
                    Some((_, v)) => args.push(field_value_to_arg(v)),
                    None => {
                        let cell = &cells[self.cell_of_field[i]];
                        args.push(cell_value_to_arg(cell));
                    }
                }
            }
            let EntryResult::Done(out, trace) = session.entry(&self.seed_def_id, &args) else {
                return Err("seed entry unexpectedly suspended on http".to_string());
            };
            if !matches!(out, ExecOutcome::Success { .. }) {
                return Err(format!("event session seed did not succeed: {out:?}"));
            }
            session.audit.seed_calls = 1;
            session.audit.entry_seqs.push(trace.execution_seq);
        }
        session.last_states = session.read_states()?;
        Ok(session)
    }
}

/// 一个增量事件会话（独立状态根；UI 桥每实例一个）。
pub struct ComponentEventSession {
    snap: Option<ProgramSnapshot>,
    generation: u64,
    fields: Vec<(String, String)>,
    handlers: Vec<HandlerEntry>,
    cell_of_field: Vec<usize>,
    text_def_ids: Vec<String>,
    texts_available: bool,
    /// 最近一次成功事件的序号水位（≤ 水位 = 重投，不重复执行）。
    watermark: Option<u64>,
    /// 最近一次已提交状态（重投时返回；Duplicate 分支不重算）。
    last_states: Vec<(String, ComponentFieldValue)>,
    audit: SessionAudit,
    /// PLAN-108（T-03c M2）：宿主 HTTP provider（UI 桥注入；None=HttpCall
    /// 具名 fail-closed）。
    http: Option<std::sync::Arc<std::sync::Mutex<dyn ace_av_core::http_client::HttpProvider>>>,
    /// PLAN-108：http 车道的执行墙上限放宽（默认 wall 短于 65s 慢端点见证；
    /// provider 注入时置 300s；只放大不缩小）。
    http_wall: Option<std::time::Duration>,
    /// PLAN-108（T-03c M3）：Http 挂起态（帧保留；续跑经 resume_http）。
    suspended: Option<ace_av_core::SuspendedHttp>,
    /// PLAN-108（T-03c M3）：挂起 handler **未赋值**字段的下标（挂起时
    /// 定）——续跑**前**取值、成功后重播种，抵消事件程序统一提交对并发
    /// 提交的覆盖（并发提交须胜出；取值须在续跑前、单线程内无交错）。
    suspended_preserve: Vec<usize>,
    /// PLAN-108：seed 入口 def（保留-重播种用；来自程序）。
    seed_def_id: String,
}

/// PLAN-108：entry 执行结果（挂起时帧随 `SuspendedHttp` 保留）。
pub enum EntryResult {
    Done(ExecOutcome, ace_av_core::ExecTrace),
    Suspended(ace_av_core::SuspendedHttp),
}

impl ComponentEventSession {
    fn limits(&self) -> ace_av_core::ExecLimits {
        // PLAN-108：http 车道放宽执行墙（默认 wall 容不下 65s+ 慢端点）。
        let mut limits = ace_av_core::ExecLimits::default();
        if let Some(wall) = self.http_wall {
            limits.wall = limits.wall.max(wall);
        }
        limits
    }

    fn entry(&self, def_id: &str, args: &[EntryArg<'static>]) -> EntryResult {
        let snap = self.snap.as_ref().expect("session is live");
        let limits = self.limits();
        match execute_program_http_resumable(
            snap,
            def_id,
            &limits,
            &ace_av_core::CancelFlag::new(false),
            args,
            self.http.clone(),
            None,
        ) {
            HttpEntryOutcome::Completed(out, trace) => EntryResult::Done(out, trace),
            HttpEntryOutcome::Suspended(susp) => EntryResult::Suspended(susp),
        }
    }

    /// PLAN-108（T-03c M3）：续跑挂起的 Http handler——provider job 完成时
    /// 经桥 poll 调用；`None`=仍在途（挂起态保留）；`Some(Ok(states))`=完成
    /// 落账；`Some(Err)`=执行失败（事务回滚语义同派发臂）。
    pub fn resume_http(
        &mut self,
    ) -> Option<Result<Vec<(String, ComponentFieldValue)>, String>> {
        let susp = self.suspended.take()?;
        // PLAN-108：会话已关（卸载/取消）——挂起态随取随弃，迟到结果拒写
        // （不 panic、不落账）。
        let Some(snap_ref) = self.snap.as_ref() else {
            self.suspended_preserve = Vec::new();
            return Some(Err(
                "session closed before http completion: suspended handler discarded (late result isolated)"
                    .to_string(),
            ));
        };
        let def_id = susp.def_id.clone();
        let snap = snap_ref.clone();
        // 续跑前取未赋值字段的当前值（并发提交胜出源；单线程内此后无交错）。
        let preserved: Vec<(usize, ComponentFieldValue)> = match self.read_states() {
            Ok(states) => self
                .suspended_preserve
                .iter()
                .map(|i| (*i, states[*i].1.clone()))
                .collect(),
            Err(e) => return Some(Err(e)),
        };
        let limits = self.limits();
        match execute_program_http_resumable(
            &snap,
            &def_id,
            &limits,
            &ace_av_core::CancelFlag::new(false),
            &[],
            self.http.clone(),
            Some(susp),
        ) {
            HttpEntryOutcome::Suspended(s2) => {
                self.suspended = Some(s2);
                None
            }
            HttpEntryOutcome::Completed(out, _trace) => match out {
                ExecOutcome::Success { .. } => {
                    self.audit.handler_entries += 1;
                    // 统一提交的过期视图覆盖了挂起期的并发提交——把续跑前
                    // 未赋值字段的值重播种回来（并发者胜出）。
                    if let Err(e) = self.reseed_preserved(&preserved) {
                        return Some(Err(e));
                    }
                    self.suspended_preserve = Vec::new();
                    match self.read_states() {
                        Ok(states) => {
                            self.last_states = states.clone();
                            Some(Ok(states))
                        }
                        Err(e) => Some(Err(e)),
                    }
                }
                other => {
                    self.audit.rejected += 1;
                    Some(Err(format!(
                        "resumed http handler did not succeed (state rolled back): {other:?}"
                    )))
                }
            },
        }
    }

    /// PLAN-108（T-03c M2）：注入/替换宿主 HTTP provider（UI 桥在打开
    /// 会话后调用；阻塞语义由桥侧 worker 线程承担）。
    pub fn set_http_provider(
        &mut self,
        provider: std::sync::Arc<std::sync::Mutex<dyn ace_av_core::http_client::HttpProvider>>,
    ) {
        self.http = Some(provider);
        // 65s+ 慢端点见证的墙上限（只放大）：http 等待是合法长阻塞。
        self.http_wall = Some(std::time::Duration::from_secs(300));
    }

    /// 会话代次（每次 open 递增；关闭重开必然新代次）。
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn audit(&self) -> &SessionAudit {
        &self.audit
    }

    /// 派发一个事件（无序号——不走去重；宿主不需要序号语义时用）。
    pub fn dispatch(
        &mut self,
        msg: &str,
        args: &[ComponentFieldValue],
    ) -> Result<EventOutcome, String> {
        self.dispatch_seq(None, msg, args)
    }

    /// 派发一个事件（带宿主序号）：声明存在性/arity/类型在效果前验证；
    /// ≤ 水位的序号 = 已成功事件的重投——**不重复执行**，返回上次状态；
    /// 执行失败（trap/预算/取消）恢复派发前 cell（不发布半状态）。
    pub fn dispatch_seq(
        &mut self,
        seq: Option<u64>,
        msg: &str,
        args: &[ComponentFieldValue],
    ) -> Result<EventOutcome, String> {
        if self.snap.is_none() {
            return Err("session closed: no further events are accepted".to_string());
        }
        let backup = self
            .snap
            .as_ref()
            .expect("session is live")
            .global_cells();
        let result = self
            .dispatch_seq_state(seq, msg, args)
            .map(|(outcome, _)| outcome);
        if result.is_err() {
            // 事务兜底：失败恢复派发前 cell（不发布半状态）。
            self.snap
                .as_ref()
                .expect("session is live")
                .restore_global_cells(&backup)
                .map_err(|e| format!("rollback after failed dispatch: {e}"))?;
        }
        result
    }

    /// PLAN-108（T-03c M3）：派发（返回是否挂起）——桥据 `SuspendedHttp`
    /// 走 InFlight 车道并在逐步 poll 中 resume。同 msg 重入由桥的
    /// pending 门忽略（对齐 VM 轨 T-04）。
    pub fn dispatch_state(
        &mut self,
        msg: &str,
        args: &[ComponentFieldValue],
    ) -> Result<(EventOutcome, DispatchState), String> {
        let backup = self
            .snap
            .as_ref()
            .expect("session is live")
            .global_cells();
        let result = self.dispatch_seq_state(None, msg, args);
        if result.is_err() {
            self.snap
                .as_ref()
                .expect("session is live")
                .restore_global_cells(&backup)
                .map_err(|e| format!("rollback after failed dispatch: {e}"))?;
        }
        result
    }

    /// PLAN-108（T-03c M3）：handler 的**源级**赋值字段 cell 下标集——
    /// 未赋值字段的值在统一提交下会被挂起 handler 的过期帧视图覆盖，
    /// 须在续跑后保留-重播种。（统一提交 epilogue 对每个字段都发
    /// GlobalSet，MIR 扫描不可用；以 handler_assigns 源级清单为准。）
    fn handler_assigned_cells(&self, msg: &str) -> std::collections::BTreeSet<usize> {
        let Some(h) = self.handlers.iter().find(|h| h.msg == msg) else {
            return std::collections::BTreeSet::new();
        };
        h.assigns
            .iter()
            .filter_map(|name| self.fields.iter().position(|(n, _)| n == name))
            .map(|i| self.cell_of_field[i])
            .collect()
    }

    /// PLAN-108（T-03c M3）：续跑成功后的保留-重播种——把续跑**前**未赋值
    /// 字段的值写回（统一提交覆盖了并发提交，并发者胜出；重播种用全量 seed
    /// 形参，赋值字段取当前值以免冲掉续跑结果）。
    fn reseed_preserved(&mut self, preserved: &[(usize, ComponentFieldValue)]) -> Result<(), String> {
        if preserved.is_empty() {
            return Ok(());
        }
        let snap = self.snap.as_ref().expect("session is live").clone();
        let cells = snap.global_cells();
        let mut args: Vec<EntryArg<'static>> = Vec::with_capacity(self.fields.len());
        for (i, (_, _)) in self.fields.iter().enumerate() {
            let value = match preserved.iter().find(|(fi, _)| *fi == i) {
                Some((_, v)) => v.clone(),
                None => cell_value_to_field(&cells[self.cell_of_field[i]])?,
            };
            args.push(field_value_to_arg(&value));
        }
        match self.entry(&self.seed_def_id.clone(), &args) {
            EntryResult::Done(out, _) if matches!(out, ExecOutcome::Success { .. }) => Ok(()),
            EntryResult::Done(out, _) => Err(format!(
                "reseed of preserved fields did not succeed: {out:?}"
            )),
            EntryResult::Suspended(_) => {
                Err("reseed unexpectedly suspended on http".to_string())
            }
        }
    }

    fn dispatch_seq_state(
        &mut self,
        seq: Option<u64>,
        msg: &str,
        args: &[ComponentFieldValue],
    ) -> Result<(EventOutcome, DispatchState), String> {
        if self.snap.is_none() {
            return Err("session closed: no further events are accepted".to_string());
        }
        let Some(h) = self.handlers.iter().find(|h| h.msg == msg) else {
            self.audit.rejected += 1;
            return Err(format!(
                "source.age.component-event: no `on {msg}` handler is declared; a directed event must name a declared handler"
            ));
        };
        if args.len() != h.payload.len() {
            self.audit.rejected += 1;
            return Err(format!(
                "source.age.component-event: message `{msg}` declares {} payload slot(s); the directed event carries {} value(s) — a payload event must not run with dropped, padded or swapped values",
                h.payload.len(),
                args.len()
            ));
        }
        for (i, (v, spelling)) in args.iter().zip(h.payload.iter()).enumerate() {
            if !value_matches_spelling(v, spelling) {
                self.audit.rejected += 1;
                return Err(format!(
                    "source.type-mismatch: payload slot #{i} of `{msg}` declares `{spelling}`, which the value does not match"
                ));
            }
        }
        if let Some(seq) = seq {
            if self.watermark >= Some(seq) {
                self.audit.duplicate_ignored += 1;
                return Ok((EventOutcome::Duplicate, DispatchState::Settled));
            }
        }
        let entry_args: Vec<EntryArg<'static>> = args.iter().map(field_value_to_arg).collect();
        match self.entry(&h.def_id, &entry_args) {
            EntryResult::Suspended(susp) => {
                // handler 在 HttpCall 挂起——帧随会话保留，本事件不提交；
                // 挂起期间其它派发照常（提交在体尾）。未赋值字段的值此刻
                // 快照（续跑统一提交会覆盖并发提交——保留-重播种）。
                let assigned = self.handler_assigned_cells(msg);
                self.suspended_preserve = (0..self.fields.len())
                    .filter(|i| !assigned.contains(&self.cell_of_field[*i]))
                    .collect();
                self.suspended = Some(susp);
                Ok((EventOutcome::Dispatched, DispatchState::SuspendedHttp))
            }
            EntryResult::Done(out, trace) => {
                self.audit.entry_seqs.push(trace.execution_seq);
                match out {
                    ExecOutcome::Success { .. } => {
                        self.audit.handler_entries += 1;
                        match self
                            .audit
                            .handler_entries_by_msg
                            .iter_mut()
                            .find(|(m, _)| m == msg)
                        {
                            Some((_, c)) => *c += 1,
                            None => self
                                .audit
                                .handler_entries_by_msg
                                .push((msg.to_string(), 1)),
                        }
                        if let Some(seq) = seq {
                            self.watermark = Some(seq);
                        }
                        self.last_states = self.read_states()?;
                        Ok((EventOutcome::Dispatched, DispatchState::Settled))
                    }
                    other => {
                        self.audit.rejected += 1;
                        Err(format!(
                            "event `{msg}` did not succeed (state rolled back): {other:?}"
                        ))
                    }
                }
            }
        }
    }

    /// 状态读取（纯观察：cell 直读，零业务派发）。
    pub fn read_states(&self) -> Result<Vec<(String, ComponentFieldValue)>, String> {
        let snap = self.snap.as_ref().ok_or("session closed")?;
        let cells = snap.global_cells();
        let mut out = Vec::with_capacity(self.fields.len());
        for (i, (name, _)) in self.fields.iter().enumerate() {
            out.push((
                name.clone(),
                cell_value_to_field(&cells[self.cell_of_field[i]])?,
            ));
        }
        Ok(out)
    }

    /// 最近一次已提交状态（重投 `Duplicate` 的返回面；关闭后仍可读最后快照）。
    pub fn last_committed_states(&self) -> &[(String, ComponentFieldValue)] {
        &self.last_states
    }

    /// 单字段读取。
    pub fn read_state(&self, field: &str) -> Result<Option<ComponentFieldValue>, String> {
        Ok(self
            .read_states()?
            .into_iter()
            .find(|(n, _)| n == field)
            .map(|(_, v)| v))
    }

    /// 文本叶求值（纯读：每叶一次派发，不执行业务 handler）。
    pub fn read_texts(&mut self) -> Result<Vec<String>, String> {
        if !self.texts_available {
            return Err(format!(
                "text leaves unavailable on this program (states-only boundary): {}",
                self.texts_unavailable_hint()
            ));
        }
        let mut out = Vec::with_capacity(self.text_def_ids.len());
        for def_id in &self.text_def_ids {
            let EntryResult::Done(outcome, trace) = self.entry(def_id, &[]) else {
                return Err("text leaf evaluation unexpectedly suspended on http".to_string());
            };
            self.audit.entry_seqs.push(trace.execution_seq);
            match outcome {
                ExecOutcome::Success { value, .. } => match value {
                    Value::Str(h) => {
                        out.push(
                            String::from_utf8(h.bytes().to_vec())
                                .map_err(|e| format!("text not UTF-8: {e}"))?,
                        );
                        self.audit.text_evals += 1;
                    }
                    other => {
                        self.audit.rejected += 1;
                        return Err(format!(
                            "text leaf `{def_id}` did not produce a Str: {other:?}"
                        ));
                    }
                },
                other => {
                    self.audit.rejected += 1;
                    return Err(format!("text leaf `{def_id}` did not succeed: {other:?}"));
                }
            }
        }
        Ok(out)
    }

    fn texts_unavailable_hint(&self) -> String {
        "f-string leaves cannot lower in the av.owned/1 lane".to_string()
    }

    /// 关闭：标记释放并 drop 快照（cell 状态根随之释放；后续事件具名拒）。
    pub fn close(&mut self) {
        if self.snap.take().is_some() {
            self.audit.released = true;
        }
    }

    /// 会话是否已关闭。
    pub fn is_closed(&self) -> bool {
        self.snap.is_none()
    }

    /// PLAN-1106 V-08 负控支持：handler 表（消息名 + 载荷拼写）——供宿主
    /// 负控构造"读取意外触发业务派发"的真实实参。
    #[doc(hidden)]
    pub fn program_handlers_for_debug(&self) -> Vec<(String, Vec<String>)> {
        self.handlers
            .iter()
            .map(|h| (h.msg.clone(), h.payload.clone()))
            .collect()
    }
}

impl Drop for ComponentEventSession {
    fn drop(&mut self) {
        // 释放路径与 close 同一语义（最后 owner drop = 状态根释放）。
        self.close();
    }
}
