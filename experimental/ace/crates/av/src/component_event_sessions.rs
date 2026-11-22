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
use ace_av_core::{execute_program_entry_args, execute_program_entry_args_with_http, EntryArg};
use ace_core::source::age_component::{
    compile_component_event_program, ComponentEventProgram as CoreEventProgram,
};
use ace_core::source::age_h0::LegacyWidgetAlias;
use std::sync::Arc;

// 消费者（UI 桥）经本模块取值域，不必直接依赖 ace-av 的场景观察面。
pub use crate::age_component::ComponentFieldValue;
// PLAN-108（T-03c M2）：宿主 HTTP provider trait 随会话面再出口（UI 桥注入）。
pub use ace_av_core::http_client::HttpProvider;

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
    pub fn compile(src: &str, module_path: &str, alias: LegacyWidgetAlias) -> Result<Self, String> {
        let caps = ["ac.runtime.text.v1".to_string()];
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
        };
        // init 链（首次派发触发；每代一次）。
        let (out, trace) = session.entry(&self.init_def_id, &[]);
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
            let (out, trace) = session.entry(&self.seed_def_id, &args);
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
}

impl ComponentEventSession {
    fn entry(
        &self,
        def_id: &str,
        args: &[EntryArg<'static>],
    ) -> (ExecOutcome, ace_av_core::ExecTrace) {
        let snap = self.snap.as_ref().expect("session is live");
        execute_program_entry_args_with_http(
            snap,
            def_id,
            &ace_av_core::ExecLimits::default(),
            &ace_av_core::CancelFlag::new(false),
            args,
            self.http.clone(),
        )
    }

    /// PLAN-108（T-03c M2）：注入/替换宿主 HTTP provider（UI 桥在打开
    /// 会话后调用；阻塞语义由桥侧 worker 线程承担）。
    pub fn set_http_provider(
        &mut self,
        provider: std::sync::Arc<std::sync::Mutex<dyn ace_av_core::http_client::HttpProvider>>,
    ) {
        self.http = Some(provider);
    }

    /// 会话代次（每次 open 递增；关闭重开必然新代次）。
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// 观察口（执行层 oracle）。
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
        let Some(h) = self.handlers.iter().find(|h| h.msg == msg) else {
            self.audit.rejected += 1;
            return Err(format!(
                "source.age.component-event: no `on {msg}` handler is declared; a directed event must name a declared handler"
            ));
        };
        if args.len() != h.payload.len() {
            self.audit.rejected += 1;
            // 原场景车道的具名诊断（check_event_dispatch）同码同形——消费侧
            // 错误合同（stage/type）保持。
            return Err(format!(
                "source.age.component-event: message `{msg}` declares {} payload slot(s); the directed event carries {} value(s) — a payload event must not run with dropped, padded or swapped values",
                h.payload.len(),
                args.len()
            ));
        }
        for (i, (v, spelling)) in args.iter().zip(h.payload.iter()).enumerate() {
            if !value_matches_spelling(v, spelling) {
                self.audit.rejected += 1;
                // 消费侧错误合同保持旧诊断码（`source.type-mismatch`——旧道
                // 经 verify 的类型诊断；效果前拒同语义）。
                return Err(format!(
                    "source.type-mismatch: payload slot #{i} of `{msg}` declares `{spelling}`, which the value does not match"
                ));
            }
        }
        // 序号水位：已成功事件的重投不重复执行（返回上次结果或具名 duplicate）。
        if let Some(seq) = seq {
            if self.watermark >= Some(seq) {
                self.audit.duplicate_ignored += 1;
                return Ok(EventOutcome::Duplicate);
            }
        }
        let entry_args: Vec<EntryArg<'static>> = args.iter().map(field_value_to_arg).collect();
        // 事务兜底：派发前备份 cell；失败恢复（业务中途 trap 由编译期
        // locals-先行/统一提交承担；这是预算边界的残余半提交兜底）。
        let backup = self.snap.as_ref().expect("session is live").global_cells();
        let (out, trace) = self.entry(&h.def_id, &entry_args);
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
                    None => self.audit.handler_entries_by_msg.push((msg.to_string(), 1)),
                }
                if let Some(seq) = seq {
                    self.watermark = Some(seq);
                }
                self.last_states = self.read_states()?;
                Ok(EventOutcome::Dispatched)
            }
            other => {
                self.snap
                    .as_ref()
                    .expect("session is live")
                    .restore_global_cells(&backup)
                    .map_err(|e| format!("rollback after failed dispatch: {e}"))?;
                self.audit.rejected += 1;
                Err(format!(
                    "event `{msg}` did not succeed (state rolled back): {other:?}"
                ))
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
            let (outcome, trace) = self.entry(def_id, &[]);
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
