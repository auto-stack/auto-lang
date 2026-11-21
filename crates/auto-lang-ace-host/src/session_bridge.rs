//! PLAN-970 (T-02/T-03): the reference explicit-ACE **session host** —
//! the executable side of the neutral session contract frozen in
//! `auto_lang_core::ace_session` (`AceSessionHost`/`ace_session_op`).
//!
//! Model (T-01 freeze record): every session owns one normalized program
//! with a FROZEN declaration set —
//!
//! ```text
//! fn main() int { return __repl_eval_int() }      // editable dispatch
//! fn __repl_eval_int() int { <current int eval> } // editable body
//! fn __repl_eval_bool() bool { <current bool eval> }
//! <user items…>                                   // frozen until reshape
//! fn __tail_anchor() int { return 0 }             // alignment anchor
//! ```
//!
//! — over one `FunctionIncrementalSession` (the ACE function-incremental
//! domain: body-only fast-path edits, atomic single publication point,
//! cross-session / stale-base / closed refusals). Evaluations are lane
//! body edits; failed evaluations commit nothing (error-then-continue).
//! Variables persist in a host-side carry table (scalar `int`/`bool`
//! only) via prelude injection + pure capture probes. Async long
//! evaluations run on the PLAN-866 `ResidentRunner` (fuel slices, call-
//! boundary safety points, in-flight frames pin old generations); the
//! runner owns the session for its remaining lifetime once started.
//!
//! All sessions live on ONE owner thread (the ACE session types are
//! single-threaded by design — the legacy daemon's sequential
//! discipline, now structural); the public handle is `Send + Sync` and
//! answers synchronously.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use ace_av::resident::{ResidentConfig, ResidentRunner, ResidentState};
use ace_av::session::{warmup, EditOutcome, FunctionIncrementalSession};
use ace_av_core::av::{CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_core::atom_text::Span;
use ace_core::source::ast::{Block, Item, Program, Stmt, TypeRef, TypeWord};
use ace_core::source::parser::parse_program;

use auto_lang_core::{
    AceSessionDiag, AceSessionHost, AceSessionLimits, AceSessionOp, AceSessionOutcome,
    AceSessionReport, AceSessionValue, AceSessionVar,
};

/// Reserved names of the canonical program shape; a user item with one
/// of these names is a named refusal, never a silent rename.
pub const RESERVED_NAMES: [&str; 3] = ["__repl_eval_int", "__repl_eval_bool", "__tail_anchor"];

const MARK_MAIN: &str = "fn main() int {";
const MARK_INT: &str = "fn __repl_eval_int() int {";
const MARK_BOOL: &str = "fn __repl_eval_bool() bool {";
const DEFAULT_INT_BODY: &str = "return 0";
const DEFAULT_BOOL_BODY: &str = "return true";

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_session_id() -> String {
    // F-970-R-03: the id mixes the process identity into the counter so
    // a restarted process cannot regenerate a restored session's id (the
    // deterministic per-process counter alone collided); the create path
    // additionally skips any id a restored record already owns.
    let n = SESSION_COUNTER.fetch_add(1, Ordering::SeqCst);
    let mix = n.wrapping_mul(0x9E3779B97F4A7C15)
        ^ (std::process::id() as u64)
        ^ (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(0));
    format!("ses_{:04x}{:04x}", n & 0xffff, mix & 0xffff)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lane {
    Int,
    Bool,
}

impl Lane {
    fn marker(self) -> &'static str {
        match self {
            Lane::Int => MARK_INT,
            Lane::Bool => MARK_BOOL,
        }
    }
    fn default_body(self) -> &'static str {
        match self {
            Lane::Int => DEFAULT_INT_BODY,
            Lane::Bool => DEFAULT_BOOL_BODY,
        }
    }
}

fn dispatch_text(lane: Lane) -> &'static str {
    match lane {
        Lane::Int => "fn main() int { return __repl_eval_int() }",
        // The bool lane rides the frozen int entry via if-encode (the
        // entry signature is part of the frozen declaration set); the
        // session report decodes 1/0 back to a bool.
        Lane::Bool => "fn main() int { if __repl_eval_bool() { return 1 }\nreturn 0 }",
    }
}

/// The canonical program text. `body_*` are the EXACT current lane body
/// texts (span computation relies on this equality).
fn canonical(dispatch: &str, body_int: &str, body_bool: &str, items: &[String]) -> String {
    let mut s = String::new();
    s.push_str(dispatch);
    s.push('\n');
    s.push_str(MARK_INT);
    s.push_str(body_int);
    s.push_str("}\n");
    s.push_str(MARK_BOOL);
    s.push_str(body_bool);
    s.push_str("}\n");
    for item in items {
        s.push_str(item);
        s.push('\n');
    }
    s.push_str("fn __tail_anchor() int { return 0 }\n");
    s
}

fn splice(src: &str, a: u64, b: u64, insert: &str) -> String {
    let mut out = String::with_capacity(src.len());
    out.push_str(&src[..a as usize]);
    out.push_str(insert);
    out.push_str(&src[b as usize..]);
    out
}

/// The statement text inside a block (between the braces).
fn body_text(src: &str, block: &Block) -> String {
    let text = &src[block.span.start as usize..block.span.end as usize];
    let open = text.find('{').map(|i| i + 1).unwrap_or(0);
    let close = text.rfind('}').unwrap_or(text.len());
    text[open..close].trim_start_matches('\n').to_string()
}

/// The full item text (span is `kw` through the closing brace).
fn item_text(src: &str, span: Span) -> String {
    src[span.start as usize..span.end as usize].to_string()
}

/// The declaration name of a named item (`None` for `use.rs`).
fn item_name(item: &Item) -> Option<String> {
    match item {
        Item::Fn(f) => Some(f.name.name.clone()),
        Item::Type(t) => Some(t.name.name.clone()),
        Item::Enum(e) => Some(e.name.name.clone()),
        Item::Const(c) => Some(c.name.name.clone()),
        Item::Store(s) => Some(s.name.name.clone()),
        Item::Task(t) => Some(t.name.name.clone()),
        Item::UseRs(_) => None,
        // PLAN-982/PLAN-980：`use py` 的 AST 条目已落地，本桥不留名字面（与 `use rs`
        // 同族），其语义由 FFI 面处理。
        Item::UsePy(_) => None,
        Item::Node(n) => Some(n.name.name.clone()),
    }
}

fn item_span(item: &Item) -> Span {
    match item {
        Item::Fn(f) => f.span,
        Item::Type(t) => t.span,
        Item::Enum(e) => e.span,
        Item::Const(c) => c.span,
        Item::Store(s) => s.span,
        Item::UseRs(u) => u.span,
        Item::Task(t) => t.span,
        Item::UsePy(u) => u.span,
        Item::Node(n) => n.span,
    }
}

/// Normalize a user source into `(body_int, body_bool, items, dispatch)`.
fn normalize(src: &str) -> Result<(String, String, Vec<String>, Lane), (String, String)> {
    if src.trim().is_empty() {
        return Ok((
            DEFAULT_INT_BODY.to_string(),
            DEFAULT_BOOL_BODY.to_string(),
            Vec::new(),
            Lane::Int,
        ));
    }
    let program = parse_program(src).map_err(|d| {
        (
            "session.source/parse".to_string(),
            format!("{}: {}", d.code, d.message),
        )
    })?;
    normalize_program(src, &program)
}

fn normalize_program(
    src: &str,
    program: &Program,
) -> Result<(String, String, Vec<String>, Lane), (String, String)> {
    let mut main_body: Option<(String, Lane)> = None;
    let mut items: Vec<String> = Vec::new();
    for item in &program.items {
        if let Item::Fn(f) = item {
            let name = f.name.name.as_str();
            if RESERVED_NAMES.contains(&name) {
                return Err((
                    "session.source/reserved-name".to_string(),
                    format!("`{name}` is reserved by the session program shape"),
                ));
            }
            if name == "main" {
                // The entry body rides a lane by its declared result type.
                let lane = match &f.ret.kind {
                    TypeRef::Builtin(TypeWord::Int) | TypeRef::Builtin(TypeWord::I32) => Lane::Int,
                    TypeRef::Builtin(TypeWord::Bool) => Lane::Bool,
                    _ => {
                        return Err((
                            "session.source/entry-type".to_string(),
                            "the session lanes support int/i32/bool entry results only".to_string(),
                        ))
                    }
                };
                main_body = Some((body_text(src, &f.body), lane));
                continue;
            }
        }
        items.push(item_text(src, item_span(item)));
    }
    let (body, lane) = main_body.unwrap_or_else(|| (DEFAULT_INT_BODY.to_string(), Lane::Int));
    Ok(match lane {
        Lane::Int => (body, DEFAULT_BOOL_BODY.to_string(), items, Lane::Int),
        Lane::Bool => (DEFAULT_BOOL_BODY.to_string(), body, items, Lane::Bool),
    })
}

/// Parse a CANONICAL session source back into its parts (lane bodies,
/// user items, dispatch lane). Unlike [`normalize`] this accepts the
/// reserved structural names — it is the re-derivation view.
fn lane_bodies(source: &str) -> Option<(String, String, Vec<String>, Lane)> {
    let program = parse_program(source).ok()?;
    let mut body_int: Option<String> = None;
    let mut body_bool: Option<String> = None;
    let mut dispatch = Lane::Int;
    let mut items: Vec<String> = Vec::new();
    for item in &program.items {
        if let Item::Fn(f) = item {
            match f.name.name.as_str() {
                "__repl_eval_int" => {
                    body_int = Some(body_text(source, &f.body));
                    continue;
                }
                "__repl_eval_bool" => {
                    body_bool = Some(body_text(source, &f.body));
                    continue;
                }
                "main" => {
                    dispatch = if body_text(source, &f.body).contains("__repl_eval_bool") {
                        Lane::Bool
                    } else {
                        Lane::Int
                    };
                    continue;
                }
                "__tail_anchor" => continue,
                _ => {}
            }
        }
        items.push(item_text(source, item_span(item)));
    }
    Some((body_int?, body_bool?, items, dispatch))
}

/// The carry-table prelude: one `var` line per tracked variable.
fn prelude(vars: &[AceSessionVar]) -> String {
    let mut s = String::new();
    for v in vars {
        let lit = match v.value {
            AceSessionValue::Int(n) => n.to_string(),
            AceSessionValue::Bool(b) => b.to_string(),
            AceSessionValue::None => continue,
        };
        s.push_str("var ");
        s.push_str(&v.name);
        s.push_str(" = ");
        s.push_str(&lit);
        s.push('\n');
    }
    s
}

/// One binding the capture pass observes after a successful eval.
#[derive(Debug, Clone)]
struct Binding {
    name: String,
    rhs: String,
}

/// Bindings the CURRENT lane body introduced or assigned (name + RHS
/// text). A parse failure yields none — the eval itself already
/// typechecked, so this is a structural probe, not the language gate.
fn bindings_of(source: &str, lane: Lane) -> Vec<Binding> {
    let want = match lane {
        Lane::Int => "__repl_eval_int",
        Lane::Bool => "__repl_eval_bool",
    };
    let Ok(program) = parse_program(source) else {
        return Vec::new();
    };
    let Some(Item::Fn(f)) = program
        .items
        .iter()
        .find(|i| matches!(i, Item::Fn(f) if f.name.name == want))
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let rhs = |e: &ace_core::source::ast::Expr| {
        source[e.span.start as usize..e.span.end as usize].to_string()
    };
    for stmt in &f.body.stmts {
        match stmt {
            Stmt::Let { name, value, .. } => out.push(Binding {
                name: name.name.clone(),
                rhs: rhs(value),
            }),
            Stmt::LetDestruct { names, value, .. } => {
                for n in names {
                    out.push(Binding {
                        name: n.name.clone(),
                        rhs: rhs(value),
                    });
                }
            }
            Stmt::Assign { target, value, .. } => {
                // Plain-name assignments carry; field-path assignments
                // carry record state — outside the scalar carry domain
                // (named boundary, not captured).
                if target.path.is_empty() {
                    out.push(Binding {
                        name: target.base.name.clone(),
                        rhs: rhs(value),
                    });
                }
            }
            _ => {}
        }
    }
    out
}

/// How the eval input was wrapped (drives value/capture semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wrap {
    /// Statement list that carries its own `return` (the returned value
    /// IS the observable value — the explicit-return evaluation form).
    StmtReturn,
    /// Statement list + sink return (no observable value).
    Stmt,
    /// Single expression returned as the value.
    Expr,
}

/// Which engine object currently owns the session's incremental state.
enum Engine {
    /// No async evaluation has run (or none is active): direct access.
    Direct(FunctionIncrementalSession),
    /// A resident runner took ownership (PLAN-866 discipline); the
    /// session stays inside the runner for its remaining lifetime.
    Resident(ResidentRunner),
    /// Transitional state during the synchronous Direct→Resident swap on
    /// the owner thread; never observable outside that statement.
    Taken,
}

/// One live session on the owner thread.
struct HostSession {
    engine: Engine,
    /// Exact text of the current program (equals the FIS source).
    source: String,
    dispatch: Lane,
    /// Exact CURRENT lane bodies in `source` (span computation source).
    cur_body_int: String,
    cur_body_bool: String,
    /// The user-visible lane bodies (capture probes never leak here).
    user_body_int: String,
    user_body_bool: String,
    items: Vec<String>,
    vars: Vec<AceSessionVar>,
    last_active: Instant,
    evals: u64,
    /// Monotone across reshapes: report revision = offset + fis.revision().
    revision_offset: u64,
    /// `Some` while an async evaluation chain is armed/running.
    async_id: Option<u64>,
    /// The persistent op log (T-08): every accepted eval / edit / reset,
    /// replayed verbatim on restore — recovery RE-VERIFIES, it never
    /// trusts persisted values (no deserialized Checked state).
    op_log: Vec<OpRecord>,
    /// Raw edits queued for the async chain's next safety point (FIFO;
    /// spliced into the tracked source as their commits land).
    queued_edits: std::collections::VecDeque<(u64, u64, Vec<u8>)>,
    finished: HashMap<u64, FinishedEval>,
    next_eval_id: u64,
}

struct FinishedEval {
    value: AceSessionValue,
    cancelled: bool,
}

/// One persisted operation (T-08 `ac.session.state/1` op log).
#[derive(Debug, Clone)]
enum OpRecord {
    Eval {
        input: String,
    },
    Edit {
        start: u64,
        end: u64,
        insert_b64: String,
    },
    Reset,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SessionStateRecord {
    schema: String,
    id: String,
    /// Serialized [`OpRecord`]s (tagged form).
    ops: Vec<serde_json::Value>,
    revision: u64,
    saved_at: u64,
}

const STATE_SCHEMA: &str = "ac.session.state/1";

fn refusal_code(out: &EditOutcome) -> String {
    match &out.rejection {
        Some(r) => format!("session.edit/{:?}", r.kind),
        None => "session.edit/refused".to_string(),
    }
}

fn diags_of(out: &EditOutcome) -> Vec<AceSessionDiag> {
    if out.diagnostics.is_empty() {
        vec![AceSessionDiag {
            stage: "session".into(),
            code: refusal_code(out),
            message: out
                .non_fast_path_reason
                .clone()
                .unwrap_or_else(|| "edit refused".into()),
        }]
    } else {
        out.diagnostics
            .iter()
            .map(|d| AceSessionDiag {
                stage: "source".into(),
                code: refusal_code(out),
                message: d.clone(),
            })
            .collect()
    }
}

impl HostSession {
    fn new(source: Option<&str>) -> Result<Self, AceSessionOutcome> {
        let (body_int, body_bool, items, lane) =
            normalize(source.unwrap_or("")).map_err(|(code, message)| {
                if code == "session.source/parse" {
                    AceSessionOutcome::Diagnostic {
                        code: code.clone(),
                        diagnostics: vec![AceSessionDiag {
                            stage: "text".into(),
                            code,
                            message,
                        }],
                    }
                } else {
                    AceSessionOutcome::Denied { code, message }
                }
            })?;
        let src = canonical(dispatch_text(lane), &body_int, &body_bool, &items);
        let fis = warmup(&src, "d_main", &[]).map_err(|e| AceSessionOutcome::Diagnostic {
            code: "session.source/compile".into(),
            diagnostics: vec![AceSessionDiag {
                stage: "compile".into(),
                code: "session.source/compile".into(),
                message: e,
            }],
        })?;
        Ok(HostSession {
            engine: Engine::Direct(fis),
            source: src,
            dispatch: lane,
            cur_body_int: body_int.clone(),
            cur_body_bool: body_bool.clone(),
            user_body_int: body_int,
            user_body_bool: body_bool,
            items,
            vars: Vec::new(),
            last_active: Instant::now(),
            evals: 0,
            revision_offset: 0,
            async_id: None,
            op_log: Vec::new(),
            queued_edits: std::collections::VecDeque::new(),
            finished: HashMap::new(),
            next_eval_id: 1,
        })
    }

    fn fis_mut(&mut self) -> &mut FunctionIncrementalSession {
        match &mut self.engine {
            Engine::Direct(f) => f,
            Engine::Resident(r) => r.session_mut(),
            Engine::Taken => unreachable!("Taken exists only inside the owner-thread swap"),
        }
    }

    fn fis_rev(&self) -> u64 {
        match &self.engine {
            Engine::Direct(f) => f.revision(),
            Engine::Resident(r) => r.session().revision(),
            Engine::Taken => unreachable!("Taken exists only inside the owner-thread swap"),
        }
    }

    fn revision(&self) -> u64 {
        self.revision_offset + self.fis_rev()
    }

    fn body_span(&self, lane: Lane) -> (u64, u64) {
        let marker = lane.marker();
        let at = self.source.find(marker).expect("lane marker present") + marker.len();
        let len = match lane {
            Lane::Int => self.cur_body_int.len(),
            Lane::Bool => self.cur_body_bool.len(),
        };
        (at as u64, (at + len) as u64)
    }

    fn dispatch_span(&self) -> (u64, u64) {
        let at = self.source.find(MARK_MAIN).expect("main marker present");
        let full = dispatch_text(self.dispatch);
        (at as u64, (at + full.len()) as u64)
    }

    /// prepare+commit one lane-body replacement. Zero commits happen on
    /// refusal (the error-then-continue invariant).
    fn commit_body(&mut self, lane: Lane, new_body: &str) -> Result<(), AceSessionOutcome> {
        let (a, b) = self.body_span(lane);
        match self
            .fis_mut()
            .prepare_edit(a, b, new_body.as_bytes().to_vec())
        {
            Err(e) => Err(AceSessionOutcome::HostFailure {
                code: "session.edit/infra".into(),
                message: e,
            }),
            Ok(Err(out)) => Err(AceSessionOutcome::Diagnostic {
                code: refusal_code(&out),
                diagnostics: diags_of(&out),
            }),
            Ok(Ok(prepared)) => {
                let outcome = self.fis_mut().commit(prepared);
                if outcome.ok {
                    self.source = splice(&self.source, a, b, new_body);
                    match lane {
                        Lane::Int => self.cur_body_int = new_body.to_string(),
                        Lane::Bool => self.cur_body_bool = new_body.to_string(),
                    }
                    Ok(())
                } else {
                    Err(AceSessionOutcome::Diagnostic {
                        code: refusal_code(&outcome),
                        diagnostics: diags_of(&outcome),
                    })
                }
            }
        }
    }

    /// Commit the dispatch switch (mechanical; a refusal here is a host
    /// failure because the body already typechecked in that lane).
    fn commit_dispatch(&mut self, lane: Lane) -> Result<(), String> {
        if self.dispatch == lane {
            return Ok(());
        }
        let (a, b) = self.dispatch_span();
        let new_text = dispatch_text(lane);
        match self
            .fis_mut()
            .prepare_edit(a, b, new_text.as_bytes().to_vec())
        {
            Ok(Ok(p)) => {
                if self.fis_mut().commit(p).ok {
                    self.source = splice(&self.source, a, b, new_text);
                    self.dispatch = lane;
                    Ok(())
                } else {
                    Err("dispatch commit refused".into())
                }
            }
            _ => Err("dispatch prepare refused after a typechecked lane body".into()),
        }
    }

    fn limits(wall: Option<Duration>) -> ExecLimits {
        let mut limits = ExecLimits::default();
        if let Some(w) = wall {
            limits.wall = w;
        }
        limits
    }

    /// Run the current program synchronously and map the outcome to the
    /// session value surface (decoding the bool lane's if-encode).
    fn run_now(
        &mut self,
        wall: Option<Duration>,
        cancel: Option<&AtomicBool>,
    ) -> (AceSessionValue, Option<AceSessionOutcome>) {
        if cancel.map(|c| c.load(Ordering::SeqCst)).unwrap_or(false) {
            return (AceSessionValue::None, Some(AceSessionOutcome::Cancelled));
        }
        let limits = Self::limits(wall);
        let flag = CancelFlag::new(false);
        let (out, _) = match &mut self.engine {
            Engine::Direct(f) => f.run(&limits, &flag),
            Engine::Resident(r) => r.session_mut().run(&limits, &flag),
            Engine::Taken => unreachable!("Taken exists only inside the owner-thread swap"),
        };
        self.map_outcome(out)
    }

    fn map_outcome(&self, out: ExecOutcome) -> (AceSessionValue, Option<AceSessionOutcome>) {
        match out {
            ExecOutcome::Success { value, .. } => match value {
                Value::I32(v) => (
                    match self.dispatch {
                        Lane::Int => AceSessionValue::Int(v),
                        Lane::Bool => AceSessionValue::Bool(v != 0),
                    },
                    None,
                ),
                Value::Unit => (AceSessionValue::None, None),
                _ => (
                    AceSessionValue::None,
                    Some(AceSessionOutcome::Denied {
                        code: "session.eval/type-unsupported".into(),
                        message: "the session lanes carry int/bool scalar values only".into(),
                    }),
                ),
            },
            // PLAN-980 HELLO 修复：Exit（进程退出内建）→ session 具名诊断。
            ExecOutcome::Exit { code } => (
                AceSessionValue::None,
                Some(AceSessionOutcome::Diagnostic {
                    code: "session.eval/exit".into(),
                    diagnostics: vec![AceSessionDiag {
                        stage: "run".into(),
                        code: "session.eval/exit".into(),
                        message: format!("the entry requested exit with code {code}"),
                    }],
                }),
            ),
            ExecOutcome::Cancelled => (AceSessionValue::None, Some(AceSessionOutcome::Cancelled)),
            ExecOutcome::Resource { kind, message } => (
                AceSessionValue::None,
                Some(AceSessionOutcome::ResourceDenied {
                    kind: format!("{kind:?}"),
                    message,
                }),
            ),
            ExecOutcome::Error { message, .. } => (
                AceSessionValue::None,
                Some(AceSessionOutcome::Diagnostic {
                    code: "session.eval/runtime-error".into(),
                    diagnostics: vec![AceSessionDiag {
                        stage: "run".into(),
                        code: "session.eval/runtime-error".into(),
                        message,
                    }],
                }),
            ),
            ExecOutcome::Trap { kind, .. } => (
                AceSessionValue::None,
                Some(AceSessionOutcome::Diagnostic {
                    code: "session.eval/trap".into(),
                    diagnostics: vec![AceSessionDiag {
                        stage: "run".into(),
                        code: "session.eval/trap".into(),
                        message: format!("{kind:?}"),
                    }],
                }),
            ),
            ExecOutcome::Infra(m) => (
                AceSessionValue::None,
                Some(AceSessionOutcome::HostFailure {
                    code: "session.eval/infra".into(),
                    message: m,
                }),
            ),
        }
    }

    /// The prelude + input body for one (lane, wrap) attempt.
    fn candidate_body(&self, lane: Lane, wrap: Wrap, input: &str) -> String {
        let mut b = prelude(&self.vars);
        let input = input.trim();
        match wrap {
            Wrap::StmtReturn => {
                // The input carries its own `return`; the returned value
                // is the observable evaluation value.
                b.push_str(input);
                b.push('\n');
            }
            Wrap::Stmt => {
                if !input.is_empty() {
                    b.push_str(input);
                    b.push('\n');
                }
                b.push_str(lane.default_body());
            }
            Wrap::Expr => {
                b.push_str("return (");
                b.push_str(input);
                b.push(')');
            }
        }
        b
    }

    /// The wrap-attempt order (explicit-return statements first, then
    /// statement+sink wraps, then single expressions; at most ONE body
    /// commit happens — the first that typechecks).
    const ATTEMPTS: [(Lane, Wrap); 6] = [
        (Lane::Int, Wrap::StmtReturn),
        (Lane::Bool, Wrap::StmtReturn),
        (Lane::Int, Wrap::Stmt),
        (Lane::Bool, Wrap::Stmt),
        (Lane::Int, Wrap::Expr),
        (Lane::Bool, Wrap::Expr),
    ];

    fn try_wrap(&mut self, input: &str) -> Result<(Lane, Wrap), AceSessionOutcome> {
        let mut last: Option<AceSessionOutcome> = None;
        for (lane, wrap) in Self::ATTEMPTS {
            let body = self.candidate_body(lane, wrap, input);
            match self.commit_body(lane, &body) {
                Ok(()) => return Ok((lane, wrap)),
                Err(hf @ AceSessionOutcome::HostFailure { .. }) => return Err(hf),
                Err(e) => last = Some(e),
            }
        }
        Err(last.unwrap_or(AceSessionOutcome::Diagnostic {
            code: "session.eval/refused".into(),
            diagnostics: Vec::new(),
        }))
    }

    /// Track the user-level lane body after a successful wrap commit.
    fn user_body_update(&mut self, lane: Lane, wrap: Wrap, input: &str) {
        let body = self.candidate_body(lane, wrap, input);
        match lane {
            Lane::Int => self.user_body_int = body,
            Lane::Bool => self.user_body_bool = body,
        }
    }

    /// Capture pass: pure re-evaluation of each changed binding's RHS in
    /// a scalar lane. Bindings whose RHS does not typecheck are not
    /// carried (named diagnostic, the session continues).
    fn capture(&mut self, bindings: Vec<Binding>, var_cap: usize) -> Vec<AceSessionDiag> {
        let mut notes = Vec::new();
        for binding in bindings {
            if self.vars.len() >= var_cap {
                notes.push(AceSessionDiag {
                    stage: "session".into(),
                    code: "session.var/cap-exceeded".into(),
                    message: format!(
                        "`{}` not carried: the variable table is at its cap ({var_cap})",
                        binding.name
                    ),
                });
                continue;
            }
            let mut captured: Option<AceSessionValue> = None;
            for lane in [Lane::Int, Lane::Bool] {
                let probe = format!("{}return ({})", prelude(&self.vars), binding.rhs);
                if self.commit_body(lane, &probe).is_err() {
                    continue; // try the other lane
                }
                if self.commit_dispatch(lane).is_err() {
                    break;
                }
                let (value, failure) = self.run_now(None, None);
                if failure.is_none() {
                    captured = Some(value);
                }
                break;
            }
            match captured {
                Some(value @ (AceSessionValue::Int(_) | AceSessionValue::Bool(_))) => {
                    if let Some(slot) = self.vars.iter_mut().find(|v| v.name == binding.name) {
                        slot.value = value;
                    } else {
                        self.vars.push(AceSessionVar {
                            name: binding.name.clone(),
                            value,
                        });
                    }
                }
                _ => notes.push(AceSessionDiag {
                    stage: "session".into(),
                    code: "session.var/non-carryable".into(),
                    message: format!(
                        "`{}` not carried: its initializer is outside the int/bool carry domain",
                        binding.name
                    ),
                }),
            }
        }
        notes
    }

    /// Rebuild the session (the named reshape path for declaration-set
    /// changes / resets). Vars survive; the revision stays monotone.
    fn reshape(
        &mut self,
        body_int: &str,
        body_bool: &str,
        items: Vec<String>,
        lane: Lane,
    ) -> Result<(), String> {
        let src = canonical(dispatch_text(lane), body_int, body_bool, &items);
        let fis = warmup(&src, "d_main", &[])?;
        self.revision_offset += self.revision() + 1;
        self.engine = Engine::Direct(fis);
        self.source = src;
        self.dispatch = lane;
        self.cur_body_int = body_int.to_string();
        self.cur_body_bool = body_bool.to_string();
        self.user_body_int = body_int.to_string();
        self.user_body_bool = body_bool.to_string();
        self.items = items;
        Ok(())
    }

    /// Re-derive user bodies / items / dispatch after a raw edit (the
    /// canonical view — structural names accepted).
    fn rederive(&mut self, source: &str) {
        if let Some((body_int, body_bool, items, lane)) = lane_bodies(source) {
            self.user_body_int = body_int.clone();
            self.user_body_bool = body_bool.clone();
            self.cur_body_int = body_int;
            self.cur_body_bool = body_bool;
            self.items = items;
            self.dispatch = lane;
        }
        // The incremental seam refused unparseable sources before the
        // commit, so a parse failure here cannot occur on the fast path;
        // the derived fields keep their last good values otherwise.
    }
}

/// Does the input define new top-level items (the reshape path)? A
/// bare `var x = …` store is NOT a reshape: it evaluates as a statement
/// (the REPL carry discipline).
fn looks_like_items(input: &str) -> bool {
    match parse_program(input) {
        Ok(p) => p.items.iter().any(|i| {
            matches!(
                i,
                Item::Fn(_)
                    | Item::Type(_)
                    | Item::Enum(_)
                    | Item::Const(_)
                    | Item::UseRs(_)
                    | Item::Task(_)
            )
        }),
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Owner thread + public handle
// ---------------------------------------------------------------------------

enum Cmd {
    Create {
        source: Option<String>,
        wall: Option<Duration>,
        reply: mpsc::Sender<AceSessionReport>,
    },
    Op {
        session: String,
        op: OpPayload,
        reply: mpsc::Sender<AceSessionReport>,
    },
    List {
        reply: mpsc::Sender<Vec<String>>,
    },
    Cleanup {
        reply: mpsc::Sender<Vec<String>>,
    },
    Drop,
}

/// The owned op payload (the neutral `AceSessionOp` borrows; the owner
/// thread receives an owned copy).
enum OpPayload {
    Eval {
        input: String,
        wall: Option<Duration>,
        cancel: Option<bool>,
    },
    Edit {
        start: u64,
        end: u64,
        insert: Vec<u8>,
    },
    Inspect,
    Vars,
    Snapshot,
    Reset,
    Close,
    EvalAsync {
        input: String,
        wall: Option<Duration>,
    },
    Result {
        eval_id: u64,
    },
    CancelEval {
        eval_id: u64,
    },
}

struct Owner {
    limits: AceSessionLimits,
    sessions: HashMap<String, HostSession>,
    /// T-08 persistence root: when set, live sessions are flushed to
    /// `ac.session.state/1` records on shutdown and re-verified (op-log
    /// replay) on install; Close/expiry remove the record (a clean end).
    state_dir: Option<std::path::PathBuf>,
}

impl Owner {
    /// Serialize the session's accepted-op log to its record file.
    fn write_record(&self, id: &str, session: &HostSession) {
        let Some(dir) = &self.state_dir else { return };
        let ops: Vec<serde_json::Value> = session
            .op_log
            .iter()
            .map(|op| match op {
                OpRecord::Eval { input } => serde_json::json!({"op": "eval", "input": input}),
                OpRecord::Edit {
                    start,
                    end,
                    insert_b64,
                } => serde_json::json!({
                    "op": "edit", "start": start, "end": end, "insert_b64": insert_b64
                }),
                OpRecord::Reset => serde_json::json!({"op": "reset"}),
            })
            .collect();
        let record = SessionStateRecord {
            schema: STATE_SCHEMA.to_string(),
            id: id.to_string(),
            ops,
            revision: session.revision(),
            saved_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };
        let path = dir.join(format!("{id}.state.json"));
        if let Ok(text) = serde_json::to_string_pretty(&record) {
            let tmp = dir.join(format!("{id}.state.json.tmp"));
            // Atomic replace: a bad upgrade never leaves a half record.
            if std::fs::write(&tmp, text).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
            }
        }
    }

    fn remove_record(&self, id: &str) {
        let Some(dir) = &self.state_dir else { return };
        let _ = std::fs::remove_file(dir.join(format!("{id}.state.json")));
    }

    /// Restore persisted sessions: op-log replay RE-VERIFIES every
    /// operation through the same seams (never a deserialized Checked
    /// state). Unknown schemas are skipped by name and their files left
    /// untouched; legacy `.at` snapshots load as seeded sessions (the
    /// explicit legacy-source-replay path).
    fn restore_records(&mut self, dir: &std::path::Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut records: Vec<(String, Vec<serde_json::Value>, u64)> = Vec::new();
        let mut legacy: Vec<(String, String)> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.ends_with(".state.json") {
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                match serde_json::from_str::<SessionStateRecord>(&text) {
                    Ok(rec) if rec.schema == STATE_SCHEMA => {
                        records.push((rec.id, rec.ops, rec.revision));
                    }
                    Ok(rec) => {
                        // Unknown schema: skip by name, file untouched
                        // (the named bad-upgrade boundary, V-08).
                        eprintln!(
                            "ace-session: skipping state record {} with unknown schema {}",
                            rec.id, rec.schema
                        );
                    }
                    Err(e) => {
                        eprintln!("ace-session: skipping unreadable state record {name}: {e}");
                    }
                }
            } else if name.ends_with(".at") {
                if let Ok(source) = std::fs::read_to_string(&path) {
                    let id = name.trim_end_matches(".at").to_string();
                    legacy.push((id, source));
                }
            }
        }
        for (id, ops, revision) in records {
            if self.sessions.contains_key(&id) {
                continue;
            }
            let Ok(mut session) = HostSession::new(None) else {
                continue;
            };
            session.revision_offset = revision;
            session.op_log = ops
                .iter()
                .filter_map(|v| match v.get("op").and_then(|o| o.as_str()) {
                    Some("eval") => {
                        v.get("input")
                            .and_then(|i| i.as_str())
                            .map(|input| OpRecord::Eval {
                                input: input.to_string(),
                            })
                    }
                    Some("edit") => {
                        let start = v.get("start").and_then(|x| x.as_u64())?;
                        let end = v.get("end").and_then(|x| x.as_u64())?;
                        v.get("insert_b64")
                            .and_then(|x| x.as_str())
                            .map(|b64| OpRecord::Edit {
                                start,
                                end,
                                insert_b64: b64.to_string(),
                            })
                    }
                    Some("reset") => Some(OpRecord::Reset),
                    _ => None,
                })
                .collect();
            // Replay through the same seams; each replayed op re-records
            // itself, so trim back to one copy of the log afterwards.
            let log_len = session.op_log.len();
            self.sessions.insert(id.clone(), session);
            for op in &ops {
                let session = self.sessions.get_mut(&id).expect("just inserted");
                let pre = session.op_log.len();
                match op.get("op").and_then(|o| o.as_str()) {
                    Some("eval") => {
                        if let Some(input) = op.get("input").and_then(|i| i.as_str()) {
                            let _ =
                                Self::eval(self.limits.var_cap, &id, session, input, None, None);
                        }
                    }
                    Some("edit") => {
                        let start = op.get("start").and_then(|x| x.as_u64());
                        let end = op.get("end").and_then(|x| x.as_u64());
                        let b64 = op.get("insert_b64").and_then(|x| x.as_str());
                        if let (Some(start), Some(end), Some(b64)) = (start, end, b64) {
                            if let Ok(insert) = B64.decode(b64) {
                                let _ = Self::edit(&id, session, start, end, insert);
                            }
                        }
                    }
                    Some("reset") => {
                        let _ = Self::op_reset(&id, session);
                    }
                    _ => {}
                }
                let session = self.sessions.get_mut(&id).expect("just inserted");
                session.op_log.truncate(pre);
            }
            let session = self.sessions.get_mut(&id).expect("just inserted");
            session.op_log.truncate(log_len);
        }
        for (id, source) in legacy {
            if !self.sessions.contains_key(&id) {
                if let Ok(mut session) = HostSession::new(Some(&source)) {
                    session.op_log.push(OpRecord::Eval { input: source });
                    self.sessions.insert(id, session);
                }
            }
        }
    }
}

fn report(session: &str, s: &HostSession, outcome: AceSessionOutcome) -> AceSessionReport {
    AceSessionReport {
        session: Some(session.to_string()),
        outcome,
        revision: s.revision(),
        value: AceSessionValue::None,
        eval_id: None,
        vars: Vec::new(),
        snapshot: None,
        rebuilt: false,
        wall_ms: 0.0,
    }
}

fn denied(session: &str, code: &str, message: impl Into<String>) -> AceSessionReport {
    AceSessionReport {
        session: Some(session.to_string()),
        outcome: AceSessionOutcome::Denied {
            code: code.to_string(),
            message: message.into(),
        },
        revision: 0,
        value: AceSessionValue::None,
        eval_id: None,
        vars: Vec::new(),
        snapshot: None,
        rebuilt: false,
        wall_ms: 0.0,
    }
}

impl Owner {
    fn create(&mut self, source: Option<&str>) -> AceSessionReport {
        if self.sessions.len() >= self.limits.max_sessions {
            return AceSessionReport {
                session: None,
                outcome: AceSessionOutcome::ResourceDenied {
                    kind: "session-cap".into(),
                    message: format!("session cap reached ({})", self.limits.max_sessions),
                },
                revision: 0,
                value: AceSessionValue::None,
                eval_id: None,
                vars: Vec::new(),
                snapshot: None,
                rebuilt: false,
                wall_ms: 0.0,
            };
        }
        match HostSession::new(source) {
            Ok(s) => {
                // F-970-R-03: never silently replace a restored id.
                let mut id = next_session_id();
                while self.sessions.contains_key(&id) {
                    id = next_session_id();
                }
                let rev = s.revision();
                self.sessions.insert(id.clone(), s);
                AceSessionReport {
                    session: Some(id),
                    outcome: AceSessionOutcome::Ok,
                    revision: rev,
                    value: AceSessionValue::None,
                    eval_id: None,
                    vars: Vec::new(),
                    snapshot: None,
                    rebuilt: false,
                    wall_ms: 0.0,
                }
            }
            Err(outcome) => AceSessionReport {
                session: None,
                outcome,
                revision: 0,
                value: AceSessionValue::None,
                eval_id: None,
                vars: Vec::new(),
                snapshot: None,
                rebuilt: false,
                wall_ms: 0.0,
            },
        }
    }

    fn op(&mut self, id: &str, payload: OpPayload) -> AceSessionReport {
        let var_cap = self.limits.var_cap;
        let Some(session) = self.sessions.get_mut(id) else {
            return denied(id, "session/unknown", "no such session");
        };
        session.last_active = Instant::now();
        match payload {
            OpPayload::Eval {
                input,
                wall,
                cancel,
            } => Self::eval(var_cap, id, session, &input, wall, cancel),
            OpPayload::Edit { start, end, insert } => Self::edit(id, session, start, end, insert),
            OpPayload::Inspect => {
                let mut r = report(id, session, AceSessionOutcome::Ok);
                r.snapshot = Some(session.source.clone());
                r.vars = session.vars.clone();
                r
            }
            OpPayload::Vars => {
                let mut r = report(id, session, AceSessionOutcome::Ok);
                r.vars = session.vars.clone();
                r
            }
            OpPayload::Snapshot => {
                let mut r = report(id, session, AceSessionOutcome::Ok);
                // The user-visible program (capture probes never leak).
                r.snapshot = Some(canonical(
                    dispatch_text(session.dispatch),
                    &session.user_body_int,
                    &session.user_body_bool,
                    &session.items,
                ));
                r
            }
            OpPayload::Reset => Self::op_reset(id, session),
            OpPayload::Close => {
                if let Engine::Resident(r) = &mut session.engine {
                    r.stop(); // drops the chain, releases pins (V-04)
                }
                session.fis_mut().close();
                self.remove_record(id);
                self.sessions.remove(id);
                AceSessionReport {
                    session: Some(id.to_string()),
                    outcome: AceSessionOutcome::Ok,
                    revision: 0,
                    value: AceSessionValue::None,
                    eval_id: None,
                    vars: Vec::new(),
                    snapshot: None,
                    rebuilt: false,
                    wall_ms: 0.0,
                }
            }
            OpPayload::EvalAsync { input, wall } => Self::eval_async(id, session, &input, wall),
            OpPayload::Result { eval_id } => Self::poll_async(id, session, eval_id),
            OpPayload::CancelEval { eval_id } => Self::cancel_eval(id, session, eval_id),
        }
    }

    /// The reset op (shared by the dispatch arm and record replay).
    fn op_reset(id: &str, session: &mut HostSession) -> AceSessionReport {
        if session.async_id.is_some() {
            return denied(
                id,
                "session.eval/busy-async",
                "an async evaluation is in flight; cancel it first",
            );
        }
        match session.reshape(DEFAULT_INT_BODY, DEFAULT_BOOL_BODY, Vec::new(), Lane::Int) {
            Ok(()) => {
                session.vars.clear();
                session.op_log.push(OpRecord::Reset);
                let mut r = report(id, session, AceSessionOutcome::Ok);
                r.rebuilt = true;
                r
            }
            Err(e) => report(
                id,
                session,
                AceSessionOutcome::HostFailure {
                    code: "session.reset/reshape".into(),
                    message: e,
                },
            ),
        }
    }

    fn eval(
        var_cap: usize,
        id: &str,
        session: &mut HostSession,
        input: &str,
        wall: Option<Duration>,
        cancel: Option<bool>,
    ) -> AceSessionReport {
        if session.async_id.is_some() {
            return denied(
                id,
                "session.eval/busy-async",
                "an async evaluation is in flight; poll or cancel it first",
            );
        }
        // Item-definition inputs reshape the frozen declaration set (the
        // function-incremental domain refuses declaration-set changes by
        // design — the reshape is the named, user-visible rebuild).
        if looks_like_items(input) {
            return match Self::reshape_append(id, session, input) {
                Ok(mut r) => {
                    session.op_log.push(OpRecord::Eval {
                        input: input.to_string(),
                    });
                    r.rebuilt = true;
                    r
                }
                Err(outcome) => report(id, session, outcome),
            };
        }
        let (lane, wrap) = match session.try_wrap(input) {
            Ok(c) => c,
            Err(outcome) => return report(id, session, outcome),
        };
        session.user_body_update(lane, wrap, input);
        if let Err(e) = session.commit_dispatch(lane) {
            return report(
                id,
                session,
                AceSessionOutcome::HostFailure {
                    code: "session.eval/dispatch".into(),
                    message: e,
                },
            );
        }
        // The neutral cancel flag was snapshotted at dispatch; long
        // synchronous evaluations are bounded by `wall`.
        let cancel_flag = cancel.map(|c| AtomicBool::new(c));
        let (mut value, failure) = session.run_now(wall, cancel_flag.as_ref());
        if wrap == Wrap::Stmt {
            // Statement-only evaluations display nothing (the sink return
            // exists to satisfy the profile, not as an observable value).
            value = AceSessionValue::None;
        }
        if let Some(outcome) = failure {
            let mut r = report(id, session, outcome);
            r.value = value;
            return r;
        }
        // Capture pass (statement wraps only — expression wraps carry no
        // binding).
        let mut notes = Vec::new();
        if matches!(wrap, Wrap::Stmt | Wrap::StmtReturn) {
            let bindings = bindings_of(&session.source, lane);
            notes = session.capture(bindings, var_cap);
        }
        session.evals += 1;
        session.op_log.push(OpRecord::Eval {
            input: input.to_string(),
        });
        let mut r = report(id, session, AceSessionOutcome::Ok);
        r.value = value;
        if !notes.is_empty() {
            r.outcome = AceSessionOutcome::Diagnostic {
                code: "session.var/notes".into(),
                diagnostics: notes,
            };
        }
        r
    }

    /// Reshape path for item-definition inputs: the current canonical
    /// program's items, same-named declarations replaced by the input's,
    /// then the input's new items appended.
    fn reshape_append(
        id: &str,
        session: &mut HostSession,
        input: &str,
    ) -> Result<AceSessionReport, AceSessionOutcome> {
        let program = parse_program(input).map_err(|d| AceSessionOutcome::Diagnostic {
            code: "session.source/parse".into(),
            diagnostics: vec![AceSessionDiag {
                stage: "text".into(),
                code: "session.source/parse".into(),
                message: format!("{}: {}", d.code, d.message),
            }],
        })?;
        for item in &program.items {
            if let Item::Fn(f) = item {
                if RESERVED_NAMES.contains(&f.name.name.as_str()) {
                    return Err(AceSessionOutcome::Denied {
                        code: "session.source/reserved-name".into(),
                        message: format!(
                            "`{}` is reserved by the session program shape",
                            f.name.name
                        ),
                    });
                }
            }
        }
        let replaced: Vec<String> = program.items.iter().filter_map(item_name).collect();
        let current =
            parse_program(&session.source).map_err(|d| AceSessionOutcome::HostFailure {
                code: "session.reshape/parse".into(),
                message: format!("canonical source no longer parses: {}", d.message),
            })?;
        let structural = |name: &str| name == "main" || RESERVED_NAMES.contains(&name);
        let mut merged: Vec<String> = Vec::new();
        for item in &current.items {
            if let Some(name) = item_name(item) {
                // Skip the canonical shape's structural items (the
                // template re-adds them) and any item the input replaces.
                if structural(&name) || replaced.contains(&name) {
                    continue;
                }
            }
            merged.push(item_text(&session.source, item_span(item)));
        }
        for item in &program.items {
            merged.push(item_text(input, item_span(item)));
        }
        let dispatch = session.dispatch;
        let body_int = session.user_body_int.clone();
        let body_bool = session.user_body_bool.clone();
        match session.reshape(&body_int, &body_bool, merged, dispatch) {
            Ok(()) => {
                session.evals += 1;
                let mut r = report(id, session, AceSessionOutcome::Ok);
                r.rebuilt = true;
                Ok(r)
            }
            Err(e) => Err(AceSessionOutcome::HostFailure {
                code: "session.reshape/compile".into(),
                message: e,
            }),
        }
    }

    fn edit(
        id: &str,
        session: &mut HostSession,
        start: u64,
        end: u64,
        insert: Vec<u8>,
    ) -> AceSessionReport {
        if start > end || end > session.source.len() as u64 {
            return denied(id, "session.edit/bounds", "edit span out of bounds");
        }
        let mut bytes = session.source.as_bytes().to_vec();
        bytes.splice(start as usize..end as usize, insert.iter().copied());
        let Ok(new_source) = String::from_utf8(bytes) else {
            return denied(id, "session.edit/utf8", "edit produces non-UTF-8 source");
        };
        // Edits while an async chain runs queue for the next safety
        // point (V-02: in-flight frames keep the old generation).
        if session.async_id.is_some() {
            let Engine::Resident(runner) = &mut session.engine else {
                unreachable!("async implies a resident runner")
            };
            return match runner.submit_edit(start, end, insert.clone()) {
                Ok(Ok(())) => {
                    session.queued_edits.push_back((start, end, insert));
                    let mut r = report(id, session, AceSessionOutcome::Ok);
                    r.snapshot = Some(
                        "queued: the edit lands at the next call-boundary safety point".into(),
                    );
                    r
                }
                Ok(Err(out)) => report(
                    id,
                    session,
                    AceSessionOutcome::Diagnostic {
                        code: refusal_code(&out),
                        diagnostics: diags_of(&out),
                    },
                ),
                Err(e) => report(
                    id,
                    session,
                    AceSessionOutcome::HostFailure {
                        code: "session.edit/infra".into(),
                        message: e,
                    },
                ),
            };
        }
        match session.fis_mut().prepare_edit(start, end, insert.clone()) {
            Err(e) => report(
                id,
                session,
                AceSessionOutcome::HostFailure {
                    code: "session.edit/infra".into(),
                    message: e,
                },
            ),
            Ok(Err(out)) => report(
                id,
                session,
                AceSessionOutcome::Diagnostic {
                    code: refusal_code(&out),
                    diagnostics: diags_of(&out),
                },
            ),
            Ok(Ok(prepared)) => {
                let outcome = session.fis_mut().commit(prepared);
                if outcome.ok {
                    session.source = new_source.clone();
                    session.rederive(&new_source);
                    session.op_log.push(OpRecord::Edit {
                        start,
                        end,
                        insert_b64: B64.encode(&insert),
                    });
                    report(id, session, AceSessionOutcome::Ok)
                } else {
                    report(
                        id,
                        session,
                        AceSessionOutcome::Diagnostic {
                            code: refusal_code(&outcome),
                            diagnostics: diags_of(&outcome),
                        },
                    )
                }
            }
        }
    }

    fn eval_async(
        id: &str,
        session: &mut HostSession,
        input: &str,
        wall: Option<Duration>,
    ) -> AceSessionReport {
        if session.async_id.is_some() {
            return denied(
                id,
                "session.eval/busy-async",
                "an async evaluation is already in flight",
            );
        }
        if looks_like_items(input) {
            return denied(
                id,
                "session.eval/async-items",
                "item definitions are synchronous reshape inputs; evaluate them with eval",
            );
        }
        let (lane, wrap) = match session.try_wrap(input) {
            Ok(c) => c,
            Err(outcome) => return report(id, session, outcome),
        };
        session.user_body_update(lane, wrap, input);
        if let Err(e) = session.commit_dispatch(lane) {
            return report(
                id,
                session,
                AceSessionOutcome::HostFailure {
                    code: "session.eval/dispatch".into(),
                    message: e,
                },
            );
        }
        let mut config = ResidentConfig::default();
        if let Some(w) = wall {
            config.limits.wall = w;
        }
        let old = std::mem::replace(&mut session.engine, Engine::Taken);
        let fis = match old {
            Engine::Direct(f) => f,
            // F-970-R-01: a previous runner is only present while its
            // async id is live (terminal/cancel restored Direct above);
            // the async gate therefore makes this arm unreachable.
            _ => unreachable!("async gate checked above"),
        };
        session.engine = Engine::Resident(ResidentRunner::start(fis, config));
        let eval_id = session.next_eval_id;
        session.next_eval_id += 1;
        session.async_id = Some(eval_id);
        session.op_log.push(OpRecord::Eval {
            input: input.to_string(),
        });
        let mut r = report(id, session, AceSessionOutcome::Ok);
        r.eval_id = Some(eval_id);
        r
    }

    /// Advance the async chain one poll: queued edits land at safety
    /// points (each committed edit splices into the tracked source); the
    /// chain runs slices until finished or the per-poll budget; a
    /// finished chain reports its (possibly mixed-generation) value on
    /// every later poll from the finished table.
    fn poll_async(id: &str, session: &mut HostSession, eval_id: u64) -> AceSessionReport {
        if let Some(fin) = session.finished.get(&eval_id) {
            let mut r = report(
                id,
                session,
                if fin.cancelled {
                    AceSessionOutcome::Cancelled
                } else {
                    AceSessionOutcome::Ok
                },
            );
            r.value = fin.value;
            r.eval_id = Some(eval_id);
            return r;
        }
        if session.async_id != Some(eval_id) {
            return denied(
                id,
                "session.eval/unknown-id",
                format!("no async evaluation {eval_id}"),
            );
        }
        const POLL_SLICES: usize = 256;
        let terminal: Option<ExecOutcome> = {
            let Engine::Resident(runner) = &mut session.engine else {
                unreachable!("async implies a resident runner")
            };
            let mut terminal = None;
            for _ in 0..POLL_SLICES {
                // Advance FIRST, then land queued edits at a real call
                // boundary: an edit submitted while the chain runs must
                // affect only post-boundary rounds (V-02), never the
                // whole chain from its start.
                let rep = runner.slice();
                if rep.state != ResidentState::Running {
                    terminal = rep.outcome;
                    // F-970-R-01: the chain is done — hand the session
                    // back to direct ownership so a later async can arm a
                    // fresh chain (never the Taken guard).
                    let old_engine = std::mem::replace(&mut session.engine, Engine::Taken);
                    session.engine = match old_engine {
                        Engine::Resident(r) => Engine::Direct(r.into_session()),
                        other => other,
                    };
                    break;
                }
                if !runner.pending().is_empty() {
                    let sp = runner.apply_pending_at_safety_point();
                    if sp.edit.ok {
                        if let Some((start, end, insert)) = session.queued_edits.pop_front() {
                            let text = String::from_utf8_lossy(&insert).into_owned();
                            session.source = splice(&session.source, start, end, &text);
                        }
                    }
                }
            }
            terminal
        };
        match terminal {
            Some(outcome) => {
                let cancelled = matches!(outcome, ExecOutcome::Cancelled);
                let (value, failure) = session.map_outcome(outcome);
                let fin_cancelled =
                    cancelled || matches!(failure, Some(AceSessionOutcome::Cancelled));
                session.async_id = None;
                session.queued_edits.clear();
                session.evals += 1;
                session.finished.insert(
                    eval_id,
                    FinishedEval {
                        value,
                        cancelled: fin_cancelled,
                    },
                );
                let outcome_final = match failure {
                    Some(AceSessionOutcome::Cancelled) => AceSessionOutcome::Cancelled,
                    Some(other) => other,
                    None if cancelled => AceSessionOutcome::Cancelled,
                    None => AceSessionOutcome::Ok,
                };
                let mut r = report(id, session, outcome_final);
                r.value = value;
                r.eval_id = Some(eval_id);
                r
            }
            None => {
                let mut r = report(id, session, AceSessionOutcome::Ok);
                r.eval_id = Some(eval_id);
                r.snapshot = Some("running".into());
                r
            }
        }
    }

    fn cancel_eval(id: &str, session: &mut HostSession, eval_id: u64) -> AceSessionReport {
        if session.async_id != Some(eval_id) {
            return denied(
                id,
                "session.eval/unknown-id",
                format!("no in-flight async evaluation {eval_id}"),
            );
        }
        // F-970-R-01: host stop drops the chain, releases every pin
        // (V-04), and hands the session back to direct ownership so a
        // later async can arm a fresh chain.
        let old_engine = std::mem::replace(&mut session.engine, Engine::Taken);
        session.engine = match old_engine {
            Engine::Resident(r) => Engine::Direct(r.into_session()),
            other => other,
        };
        session.async_id = None;
        session.finished.insert(
            eval_id,
            FinishedEval {
                value: AceSessionValue::None,
                cancelled: true,
            },
        );
        report(id, session, AceSessionOutcome::Cancelled)
    }
}

fn run_owner(
    rx: mpsc::Receiver<Cmd>,
    limits: AceSessionLimits,
    state_dir: Option<std::path::PathBuf>,
) {
    let mut owner = Owner {
        limits,
        sessions: HashMap::new(),
        state_dir: state_dir.clone(),
    };
    if let Some(dir) = &state_dir {
        let _ = std::fs::create_dir_all(dir);
        owner.restore_records(dir);
    }
    while let Ok(cmd) = rx.recv() {
        match cmd {
            Cmd::Create {
                source,
                wall,
                reply,
            } => {
                let _ = wall; // warmup cost is the frozen surface; see T-04 evidence
                let _ = reply.send(owner.create(source.as_deref()));
            }
            Cmd::Op { session, op, reply } => {
                let _ = reply.send(owner.op(&session, op));
            }
            Cmd::List { reply } => {
                let _ = reply.send(owner.sessions.keys().cloned().collect());
            }
            Cmd::Cleanup { reply } => {
                let timeout = Duration::from_secs(owner.limits.idle_timeout_secs);
                let expired: Vec<String> = owner
                    .sessions
                    .iter()
                    .filter(|(_, s)| s.last_active.elapsed() > timeout)
                    .map(|(k, _)| k.clone())
                    .collect();
                for key in &expired {
                    owner.remove_record(key);
                    owner.sessions.remove(key);
                }
                let _ = reply.send(expired);
            }
            Cmd::Drop => break,
        }
    }
    // Shutdown flush: every LIVE session's accepted-op log persists (a
    // restarting daemon restores them; closed/expired sessions already
    // removed their records).
    for (id, session) in &owner.sessions {
        owner.write_record(id, session);
    }
}

/// The registry-side host: a cheap cloneable gate to the one owner
/// thread (the public guard below owns the thread lifecycle).
struct SessionHostShim {
    tx: mpsc::Sender<Cmd>,
}

impl SessionHostShim {
    fn call(&self, make: impl FnOnce(mpsc::Sender<AceSessionReport>) -> Cmd) -> AceSessionReport {
        let (rtx, rrx) = mpsc::channel();
        if self.tx.send(make(rtx)).is_err() {
            return host_gone();
        }
        rrx.recv().unwrap_or_else(|_| host_gone())
    }
}

fn host_gone() -> AceSessionReport {
    AceSessionReport {
        session: None,
        outcome: AceSessionOutcome::HostFailure {
            code: "ace.session/host-gone".into(),
            message: "the session owner thread is gone".into(),
        },
        revision: 0,
        value: AceSessionValue::None,
        eval_id: None,
        vars: Vec::new(),
        snapshot: None,
        rebuilt: false,
        wall_ms: 0.0,
    }
}

impl AceSessionHost for SessionHostShim {
    fn create(&self, source: Option<&str>, wall: Option<Duration>) -> AceSessionReport {
        self.call(|reply| Cmd::Create {
            source: source.map(|s| s.to_string()),
            wall,
            reply,
        })
    }

    fn op(&self, session: &str, op: AceSessionOp<'_>) -> AceSessionReport {
        let payload = match op {
            AceSessionOp::Eval {
                input,
                wall,
                cancel,
            } => OpPayload::Eval {
                input: input.to_string(),
                wall,
                // The neutral contract lends a flag reference; the owned
                // round-trip snapshots its CURRENT value (synchronous
                // evals honor it as a pre-check; async evals are
                // cancelled by id — T-08 surface).
                cancel: cancel.map(|c| c.load(Ordering::SeqCst)),
            },
            AceSessionOp::Edit { start, end, insert } => OpPayload::Edit {
                start,
                end,
                insert: insert.to_vec(),
            },
            AceSessionOp::Inspect => OpPayload::Inspect,
            AceSessionOp::Vars => OpPayload::Vars,
            AceSessionOp::Snapshot => OpPayload::Snapshot,
            AceSessionOp::Reset => OpPayload::Reset,
            AceSessionOp::Close => OpPayload::Close,
            AceSessionOp::EvalAsync { input, wall } => OpPayload::EvalAsync {
                input: input.to_string(),
                wall,
            },
            AceSessionOp::Result { eval_id } => OpPayload::Result { eval_id },
            AceSessionOp::CancelEval { eval_id } => OpPayload::CancelEval { eval_id },
        };
        self.call(|reply| Cmd::Op {
            session: session.to_string(),
            op: payload,
            reply,
        })
    }

    fn list(&self) -> Vec<String> {
        let (rtx, rrx) = mpsc::channel();
        if self.tx.send(Cmd::List { reply: rtx }).is_err() {
            return Vec::new();
        }
        rrx.recv().unwrap_or_default()
    }

    fn cleanup_expired(&self) -> Vec<String> {
        let (rtx, rrx) = mpsc::channel();
        if self.tx.send(Cmd::Cleanup { reply: rtx }).is_err() {
            return Vec::new();
        }
        rrx.recv().unwrap_or_default()
    }
}

/// Guard returned by [`install_session_host`]: dropping it (or calling
/// [`SessionHostGuard::stop`]) stops the owner thread and closes every
/// session. The process-wide registry entry is released with it.
pub struct SessionHostGuard {
    tx: mpsc::Sender<Cmd>,
    join: Option<std::thread::JoinHandle<()>>,
}

impl SessionHostGuard {
    /// Stop the owner thread and release the registry slot.
    pub fn stop(&mut self) {
        let _ = self.tx.send(Cmd::Drop);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
        auto_lang_core::shutdown_ace_session_host();
    }
}

impl Drop for SessionHostGuard {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Install the process-wide ACE session host (single owner; conflicts
/// with an installed host are an error — release it first).
pub fn install_session_host(limits: AceSessionLimits) -> Result<SessionHostGuard, String> {
    install_session_host_inner(limits, None)
}

/// T-08: install with a persistence root — persisted sessions restore by
/// op-log replay at startup and flush on shutdown.
pub fn install_session_host_with_store(
    limits: AceSessionLimits,
    state_dir: std::path::PathBuf,
) -> Result<SessionHostGuard, String> {
    install_session_host_inner(limits, Some(state_dir))
}

fn install_session_host_inner(
    limits: AceSessionLimits,
    state_dir: Option<std::path::PathBuf>,
) -> Result<SessionHostGuard, String> {
    let (tx, rx) = mpsc::channel::<Cmd>();
    let join = std::thread::Builder::new()
        .name("ace-session-host".into())
        .spawn(move || run_owner(rx, limits, state_dir))
        .map_err(|e| format!("session owner thread: {e}"))?;
    match auto_lang_core::install_ace_session_host(Arc::new(SessionHostShim { tx: tx.clone() })) {
        Ok(()) => Ok(SessionHostGuard {
            tx,
            join: Some(join),
        }),
        Err(e) => {
            let _ = tx.send(Cmd::Drop);
            let _ = join.join();
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Structural sanity used by the fixtures: the canonical template's
    /// lane bodies are byte-exact under span computation.
    #[test]
    fn canonical_roundtrip_spans() {
        let src = canonical(
            dispatch_text(Lane::Int),
            "return 2 + 3 * 4",
            "return true",
            &["fn helper() int { return 7 }".to_string()],
        );
        assert!(src.contains("fn __repl_eval_int() int {return 2 + 3 * 4}"));
        assert!(src.ends_with("fn __tail_anchor() int { return 0 }\n"));
        let at = src.find(MARK_INT).unwrap() + MARK_INT.len();
        assert_eq!(&src[at..at + "return 2 + 3 * 4".len()], "return 2 + 3 * 4");
    }
}
