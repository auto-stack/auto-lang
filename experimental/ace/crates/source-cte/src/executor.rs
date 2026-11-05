//! The real-AV execution core of the source-CTE lane (PLAN-936 §5.3).
//!
//! The minimal checked call entry: the callee is invoked through a
//! wrapper FUNCTION appended to the caller's already-parsed program at
//! the AST level — no source text is synthesized, nothing re-parses the
//! user's bodies, and the wrapper is checked by the REAL core
//! resolver/verifier (`resolve_with` → `emit_resolved_parts` →
//! `verify_scalar`) before lowering. This is the "equivalent checked
//! call entry" §5.3 admits, and the explicit replacement of the 817
//! `wrapper_source` text bake:
//!
//! ```text
//! program (already parsed) ──append wrapper FnItem──▶ program'
//! program' ── resolve_with(&[], Scalar) ── emit_resolved_parts ── verify_scalar
//!          ──▶ CheckedModule        (the entry is CHECKED, caps = none)
//! CheckedModule ── lower_checked(d_ct936_call) ── verify_mir
//!          ── encode/decode ABC ── verify_abc ── av-core execute ──▶ Value
//! Value ── STRICT fold vs expected type ──▶ CtValue
//! ```
//!
//! The whole pipeline is the SAME shared tail every consumer end uses
//! (`lower_checked` / `av-core`); this crate contributes no second
//! executor and no AV-frontend session. Capability discipline mirrors
//! the 817 closure lane: the CT closure is compiled with NO granted
//! capabilities, so a closure using a trace/capability intrinsic is
//! refused by the frontend before any AV call.

use crate::identity::{CtValue, ExpectTy};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{
    abc_verify, decode_abc, encode_abc, execute, verify_mir, CancelFlag, ExecLimits,
};
use ace_core::atom_text::Span;
use ace_core::source::ast::{
    Arg, Block, Expr, ExprKind, FnItem, Ident, Item, Param, Program, Receiver, Stmt, TypeAnn,
    TypeRef, TypeWord,
};
use ace_core::source::{emit_resolved_parts, resolve_with, SourceProfile};
use ace_core::verify::{verify_scalar, CheckedModule};
use ace_lowering::lower_checked;

/// The wrapper entry name (driver-internal, never a user address). The
/// emitter prefixes def id_texts with `d_`, so the HIR entry id is
/// `d_ct936_call`.
pub const CT_ENTRY: &str = "ct936_call";
pub const CT_ENTRY_ID: &str = "d_ct936_call";

/// One real AV execution's measured outcome.
#[derive(Clone, Copy, Debug)]
pub struct CtExecOutcome {
    pub value: CtValue,
    pub steps: u64,
    pub wall_ms: u64,
}

/// Why an execution failed. `av_reached` distinguishes the attempt
/// accounting: a failure AFTER the AV was reached (trap/resource/cancel)
/// is still a REAL ct_execute attempt (the §5.8 rule); a failure before
/// it (frontend/lowering/admission) is not.
#[derive(Clone, Debug)]
pub struct CtExecError {
    pub code: String,
    pub message: String,
    pub av_reached: bool,
}

impl CtExecError {
    fn pre(code: &str, message: String) -> Self {
        CtExecError {
            code: code.to_string(),
            message,
            av_reached: false,
        }
    }
    fn at_av(code: &str, message: String) -> Self {
        CtExecError {
            code: code.to_string(),
            message,
            av_reached: true,
        }
    }
    /// The counter-control rejection (817 `set_forbid_av` discipline): a
    /// cache miss cannot complete while AV execution is forbidden.
    pub fn forbidden() -> Self {
        CtExecError::pre(
            "cte.infra/av-forbidden",
            "AV execution is forbidden by the counter-control flag (a cache miss cannot complete)"
                .into(),
        )
    }
}

impl std::fmt::Display for CtExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// The strict value fold (§5.8.3): an `i32` expectation accepts only an
/// `I32` AV value; a `bool` expectation accepts a `Bool` AV value or the
/// EXPLICIT bool→i32 projection wrapper's output — and of that
/// projection ONLY `0`/`1` (the lane-versioned form §5.8.3 pre-approved;
/// the 817 any-nonzero→true fold does not exist here).
pub fn strict_fold(value: &Value, expect: ExpectTy) -> Result<CtValue, CtExecError> {
    match (value, expect) {
        (Value::I32(x), ExpectTy::I32) => Ok(CtValue::I32(*x)),
        (Value::I32(0), ExpectTy::Bool) => Ok(CtValue::Bool(false)),
        (Value::I32(1), ExpectTy::Bool) => Ok(CtValue::Bool(true)),
        (Value::Bool(b), ExpectTy::Bool) => Ok(CtValue::Bool(*b)),
        (other, expect) => Err(CtExecError::at_av(
            "cte.type/value-tag",
            format!(
                "AV returned {other:?} for a `{}` CT demand — strict tag, no projection",
                expect.as_str()
            ),
        )),
    }
}

/// Resolve + emit + verify one program through the real core authority
/// (shared by the CT wrapper lane and by owner-side backfill compiles).
pub fn checked_module_of(
    program: &Program,
    src: &str,
    module_path: &str,
) -> Result<CheckedModule, CtExecError> {
    let resolved = resolve_with(program, &[], SourceProfile::Scalar)
        .map_err(|ds| CtExecError::pre("cte.frontend/bind", first_diag(&ds)))?;
    let parts = emit_resolved_parts(&resolved, src, module_path, SourceProfile::Scalar)
        .map_err(|ds| CtExecError::pre("cte.frontend/emit", first_diag(&ds)))?;
    verify_scalar(parts.bundle)
        .map_err(|ds| CtExecError::pre("cte.frontend/verify", first_diag(&ds)))
}

/// Measured facts of one real AV execution (never reconstructed).
#[derive(Clone, Copy, Debug)]
pub struct AvExecFacts {
    pub steps: u64,
    pub wall_ms: u64,
    pub trace_events: usize,
    pub peak_slot_bytes: u64,
}

/// The neutral execution port (§2/§5.1): ONE boundary every CT lane
/// funnels through. The default implementation is the COLD shared tail
/// below; a resident lane (the 817 snapshot frontend, PLAN-828) supplies
/// its own implementation probe-side — its frontend types must never
/// enter this crate, and its execution bottoms out at the same
/// `av-core execute` (no second VM).
pub trait CtExecutor {
    /// Execute one checked entry for real. The value is RAW — each lane
    /// folds it under its own contract (this lane: strictly; the 817
    /// probe: its frozen nonzero projection).
    fn execute(
        &mut self,
        checked: &CheckedModule,
        entry_id: &str,
        snapshot: u64,
        limits: &ExecLimits,
        cancel: &CancelFlag,
    ) -> Result<(Value, AvExecFacts), CtExecError>;
}

/// The cold shared-tail executor (lower → verify MIR → ABC roundtrip →
/// ABC admission → av-core execute).
#[derive(Default)]
pub struct ColdAvExecutor;

impl CtExecutor for ColdAvExecutor {
    fn execute(
        &mut self,
        checked: &CheckedModule,
        entry_id: &str,
        snapshot: u64,
        limits: &ExecLimits,
        cancel: &CancelFlag,
    ) -> Result<(Value, AvExecFacts), CtExecError> {
        run_entry_on_av(checked, entry_id, snapshot, limits, cancel)
    }
}

/// The shared execution tail: lower one entry of a checked module, admit
/// it through MIR/ABC verification, and execute it on the REAL AV.
/// Returns the raw AV value + measured facts; callers fold strictly.
pub fn run_entry_on_av(
    checked: &CheckedModule,
    entry_id: &str,
    snapshot: u64,
    limits: &ExecLimits,
    cancel: &CancelFlag,
) -> Result<(Value, AvExecFacts), CtExecError> {
    let mir = lower_checked(checked, entry_id, snapshot)
        .map_err(|e| CtExecError::pre("cte.infra/mir-lower", format!("lowering failed: {e:?}")))?;
    let _ = verify_mir(&mir).map_err(|e| {
        CtExecError::pre(
            "cte.infra/mir-verify",
            format!("MIR verification failed: {e:?}"),
        )
    })?;
    let abc = encode_abc(&mir)
        .map_err(|e| CtExecError::pre("cte.infra/abc-emit", format!("ABC encode failed: {e:?}")))?;
    let decoded = decode_abc(&abc.to_flat()).map_err(|e| {
        CtExecError::pre("cte.infra/abc-decode", format!("ABC decode failed: {e:?}"))
    })?;
    let owner = &checked.bundle().modules[0].id_text;
    let owner_scope = ace_av_core::mir::OwnerScope {
        schema: ace_av_core::contract::ABC_SCHEMA.to_string(),
        module: owner.clone(),
    };
    let loaded = abc_verify::verify_abc(decoded, &owner_scope).map_err(|e| {
        CtExecError::pre(
            "cte.infra/abc-verify",
            format!("ABC admission failed: {e:?}"),
        )
    })?;
    let started = std::time::Instant::now();
    let (outcome, trace) = execute(&loaded, limits, cancel);
    let wall_ms = started.elapsed().as_millis() as u64;
    match outcome {
        ExecOutcome::Success { value, steps } => Ok((
            value,
            AvExecFacts {
                steps,
                wall_ms,
                trace_events: trace.events.len(),
                peak_slot_bytes: trace.peak_slot_bytes,
            },
        )),
        ExecOutcome::Trap { kind, .. } => Err(CtExecError::at_av(
            &format!("cte.trap/{}", kind.code()),
            format!("compiletime-executed entry trapped: {}", kind.code()),
        )),
        ExecOutcome::Resource { kind, message } => Err(CtExecError::at_av(
            &format!("cte.resource/{kind:?}"),
            format!("AV resource limit: {message}"),
        )),
        ExecOutcome::Cancelled => Err(CtExecError::at_av(
            "cte.cancelled",
            "AV execution cancelled".into(),
        )),
        ExecOutcome::Infra(m) => Err(CtExecError::at_av(
            "cte.infra/exec",
            format!("AV infra failure: {m}"),
        )),
    }
}

/// The execution controls one call runs under (the service holds them;
/// bundling keeps the call surface small).
#[derive(Clone, Copy)]
pub struct ExecCtl<'a> {
    pub forbid_av: bool,
    pub cancel: &'a CancelFlag,
    pub wall_budget: std::time::Duration,
}

/// Execute one CT call for real: build the minimal checked call entry,
/// run it through the shared tail, fold strictly.
///
/// `program` is the OWNER's already-parsed program (never re-parsed);
/// `src` is the owner's original text, used by the emitter for source
/// maps of the USER nodes only — the wrapper carries zero spans and no
/// text is ever re-parsed or synthesized.
pub fn execute_call(
    program: &Program,
    src: &str,
    module_path: &str,
    callee: &str,
    args: &[CtValue],
    expect: ExpectTy,
    ctl: &ExecCtl<'_>,
) -> Result<CtExecOutcome, CtExecError> {
    if ctl.forbid_av {
        return Err(CtExecError::forbidden());
    }
    // Locate the callee's declaration (its declared return type shapes
    // the wrapper's own signature; the RESOLVER does the real binding
    // when the extended program is checked).
    let target = program.items.iter().find_map(|i| match i {
        Item::Fn(f) if f.name.name == callee => Some(f),
        _ => None,
    });
    let Some(target) = target else {
        return Err(CtExecError::pre(
            "cte.query/unknown-callee",
            format!("callee `{callee}` is not a declared function in the current program"),
        ));
    };
    let ret_word = match &target.ret.kind {
        TypeRef::Builtin(w @ (TypeWord::Int | TypeWord::I32)) => Some(*w),
        TypeRef::Builtin(TypeWord::Bool) => Some(TypeWord::Bool),
        _ => None,
    };
    let Some(ret_word) = ret_word else {
        return Err(CtExecError::pre(
            "cte.type/callee-surface",
            format!("callee `{callee}` returns a type outside the ac.source.cte.scalar/v1 surface"),
        ));
    };
    let callee_bool = matches!(ret_word, TypeWord::Bool);
    // Typed admission before any wrapper is built: the demand's expected
    // class must equal the callee's declared return class (§5.8.3 "错tag
    // 拒"). The wrapper's shape follows the CALLEE (an int condition or a
    // bool return would be rejected by the resolver, not folded).
    let callee_class = if callee_bool {
        ExpectTy::Bool
    } else {
        ExpectTy::I32
    };
    if callee_class != expect {
        return Err(CtExecError::pre(
            "cte.type/expect-mismatch",
            format!(
                "callee `{callee}` returns `{}` but the CT demand expects `{}`",
                callee_class.as_str(),
                expect.as_str()
            ),
        ));
    }
    // Wrapper name must not collide with any user declaration (the
    // driver owns this namespace; a collision is suffixed away).
    let mut entry = CT_ENTRY.to_string();
    let mut n = 1;
    while program
        .items
        .iter()
        .any(|i| matches!(i, Item::Fn(f) if f.name.name == entry))
    {
        n += 1;
        entry = format!("{CT_ENTRY}_{n}");
    }
    let entry_id = format!("d_{entry}");
    let wrapper = wrapper_item(&entry, callee, args, callee_bool);
    let mut extended = program.clone();
    extended.items.push(Item::Fn(wrapper));

    let checked = checked_module_of(&extended, src, module_path)?;
    if !checked.required_capabilities().is_empty() {
        return Err(CtExecError::pre(
            "cte.capability/denied",
            format!(
                "callee closure requires capabilities {:?} — the CT profile grants none",
                checked.required_capabilities()
            ),
        ));
    }
    let snapshot = {
        let args_txt = args
            .iter()
            .map(|a| a.canonical())
            .collect::<Vec<_>>()
            .join(",");
        ace_source_query_fnv(format!("{callee}|{args_txt}").as_bytes())
    };
    let (value, facts) = run_entry_on_av(
        &checked,
        &entry_id,
        snapshot,
        &ExecLimits {
            wall: ctl.wall_budget,
            ..Default::default()
        },
        ctl.cancel,
    )?;
    let value = strict_fold(&value, expect)?;
    Ok(CtExecOutcome {
        value,
        steps: facts.steps,
        wall_ms: facts.wall_ms,
    })
}

/// Build the wrapper entry as an AST node. The frozen AV entry contract
/// is zero-param **i32**, so both callee classes produce an i32 entry:
///
/// * int callee: `fn <entry>() int { return <callee>(<args>) }` — the
///   value rides unchanged;
/// * bool callee: `fn <entry>() int { if <callee>(<args>) { return 1 }
///   return 0 }` — the EXPLICIT, lane-versioned bool→i32 projection
///   (§5.8.3); the strict fold admits only its 0/1 outputs.
fn wrapper_item(entry: &str, callee: &str, args: &[CtValue], callee_bool: bool) -> FnItem {
    let zero = Span { start: 0, end: 0 };
    let call_args = || {
        args.iter()
            .map(|a| {
                let kind = match a {
                    CtValue::I32(x) => ExprKind::IntLit(*x),
                    CtValue::Bool(b) => ExprKind::BoolLit(*b),
                };
                Arg::Pos(Expr { kind, span: zero })
            })
            .collect()
    };
    let call = |args: Vec<Arg>| Expr {
        kind: ExprKind::Call {
            callee: Ident {
                name: callee.to_string(),
                span: zero,
            },
            args,
        },
        span: zero,
    };
    let stmts = if callee_bool {
        vec![
            Stmt::If {
                cond: call(call_args()),
                then: Block {
                    stmts: vec![Stmt::Return {
                        value: Expr {
                            kind: ExprKind::IntLit(1),
                            span: zero,
                        },
                        span: zero,
                    }],
                    span: zero,
                },
                els: None,
                span: zero,
            },
            Stmt::Return {
                value: Expr {
                    kind: ExprKind::IntLit(0),
                    span: zero,
                },
                span: zero,
            },
        ]
    } else {
        vec![Stmt::Return {
            value: call(call_args()),
            span: zero,
        }]
    };
    FnItem {
        name: Ident {
            name: entry.to_string(),
            span: zero,
        },
        type_params: Vec::new(),
        params: Vec::<Param>::new(),
        ret: TypeAnn {
            kind: TypeRef::Builtin(TypeWord::Int),
            span: zero,
        },
        body: Block { stmts, span: zero },
        span: zero,
        receiver: Receiver::Free,
        owner: None,
        script: false,
    }
}

/// A stable non-cryptographic snapshot id for the lowering adapter
/// (mirrors the 817 fnv1a64 discipline; the identity that matters is the
/// ExecKey, not this snapshot).
fn ace_source_query_fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn first_diag(ds: &[ace_core::atom_text::Diagnostic]) -> String {
    ds.first()
        .map(|d| format!("{}: {}", d.code, d.message))
        .unwrap_or_else(|| "unknown".into())
}
