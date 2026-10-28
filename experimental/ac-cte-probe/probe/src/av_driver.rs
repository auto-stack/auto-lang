//! The AV driver: one real CT call through the merged 798 services
//! (compile → capability audit → lower → verify_mir → ABC roundtrip →
//! verify_abc → `execute_entry`). The 798 module-closure entry contract
//! is zero-param i32, so a parameterized callee is invoked through a
//! synthesized wrapper entry with the constant arguments baked in — the
//! CALLEE still executes on the AV (that is the ct_execute=1 fact), and
//! bool results project through an `if` (no new traps).

use auto_ac_prototype::source::compile_scalar_source;
use auto_av_core::contract::{
    ExecOutcome, Value, ABC_SCHEMA, ABC_SEMANTICS, AV_PROFILE, AV_SEMANTICS, MAX_AV_WALL,
    MIR_SCHEMA, MIR_SEMANTICS,
};

/// The D-798 per-call AV wall cap — an upper bound the request deadline
/// narrows per call (F-817-R6-01: the 10s REQUEST total dominates; the
/// per-call cap is never raised).
pub const MAX_CALL_WALL: std::time::Duration = MAX_AV_WALL;
use auto_av_core::{
    abc_verify, decode_abc, encode_abc, execute, verify_mir, CancelFlag, ExecLimits,
};
use auto_mir_lower::lower_checked;
use auto_av_probe::session::FunctionIncrementalSession;
use auto_cte_contract::CteEngineId;
use auto_cte_contract::{CteAdmissionError, CteCode, CteValue};

/// The wrapper entry name (driver-internal; never a user address). The
/// Atom emitter prefixes def id_texts with `d_`, so the HIR id is
/// `d_cte817_call` (the official av cases use the same convention, e.g.
/// `entry: "d_main"`).
pub const CTE_CALL_ENTRY: &str = "cte817_call";
/// The wrapper's def id_text passed to `lower_checked`.
pub const CTE_CALL_ENTRY_ID: &str = "d_cte817_call";

/// Current engine identity, read from the merged 798 constants at
/// runtime so any future version bump re-keys the cache automatically.
/// `lane` selects the execution axis ("closure" | "snapshot", V-12).
pub fn current_engine_for(lane: &str) -> CteEngineId {
    CteEngineId {
        av_profile: AV_PROFILE.to_string(),
        mir_schema: MIR_SCHEMA.to_string(),
        abc_schema: ABC_SCHEMA.to_string(),
        mir_semantics: MIR_SEMANTICS.0,
        abc_semantics: ABC_SEMANTICS.0,
        av_semantics: AV_SEMANTICS.0,
        exec: lane.to_string(),
    }
}

/// The default (closure-lane) engine identity.
pub fn current_engine() -> CteEngineId {
    current_engine_for("closure")
}

/// A warmed PLAN-828 function-incremental session driving one CTE
/// request's calls (T-11/SD-06): the ONE whole-module compile happens
/// at warm-up; per-call argument changes are incremental edits that
/// recompile ONLY the wrapper function.
pub struct SnapshotLane {
    session: FunctionIncrementalSession,
    /// The wrapper's BODY span — retargets rewrite the whole body so a
    /// callee switch (different function, arity AND return class) is
    /// honored wholesale (F-817-R1-01).
    body_span: (u64, u64),
    /// Lane-level degrade detector (V-12 failure control): the 828
    /// session never silently degrades (its whole_module counters are
    /// structurally 0); the only whole-module path is the EXPLICIT
    /// warm-up, counted here, against the incremental edit count.
    whole_module_events: u64,
    incremental_edits: u64,
}

/// Locate the wrapper's BODY span (between the wrapper's opening brace
/// and its closing brace, exclusive). Retargets rewrite the WHOLE body:
/// a callee switch may change the return class, and the body shape must
/// follow (i32 → `return callee(args)`; bool → the if-projection).
/// PLAN-820 F820-R10-01: fold an AV success value into the CTE value
/// surface. The frozen CTE/scalar lanes keep their I32/Bool semantics
/// byte-for-byte; the shared core's av.data/1 value extension
/// (I64/U64/F32/F64/Record) is a CONTROLLED rejection here — no panic,
/// no unreachable, and no implicit widening of the CTE scalar surface.
fn cte_value_of(value: &Value, is_bool: bool) -> Result<CteValue, CteAdmissionError> {
    match (value, is_bool) {
        (Value::I32(x), true) => Ok(CteValue::Bool(*x != 0)),
        (Value::I32(x), false) => Ok(CteValue::I32(*x)),
        (Value::Bool(b), _) => Ok(CteValue::Bool(*b)),
        (other, _) => Err(CteAdmissionError::new(
            CteCode::CalleeFrontend,
            format!("callee returned a value outside the cte/scalar-query-v1 surface: {other:?}"),
        )),
    }
}

fn locate_body_span(src: &str) -> Result<(u64, u64), CteAdmissionError> {
    let header = format!("fn {CTE_CALL_ENTRY}() int {{");
    let h = src.rfind(&header).ok_or_else(|| {
        CteAdmissionError::new(
            CteCode::Infra("snapshot".into()),
            "wrapper header not found",
        )
    })?;
    let body_start = (h + header.len()) as u64;
    let anchor = src.rfind("fn cte817_anchor").ok_or_else(|| {
        CteAdmissionError::new(CteCode::Infra("snapshot".into()), "anchor not found")
    })?;
    // The closing brace of the wrapper = the last '}' before the anchor.
    let close = src[..anchor].rfind('}').ok_or_else(|| {
        CteAdmissionError::new(CteCode::Infra("snapshot".into()), "wrapper close not found")
    })? as u64;
    Ok((body_start, close))
}

/// The wrapper body for one call, by the callee's return class.
fn wrapper_body(callee: &str, args: &[CteValue], is_bool: bool) -> String {
    let args_txt = args.iter().map(lit).collect::<Vec<_>>().join(", ");
    if is_bool {
        format!(
            "
    if {callee}({args_txt}) {{
        return 1
    }}
    return 0
"
        )
    } else {
        format!(
            "
    return {callee}({args_txt})
"
        )
    }
}

impl SnapshotLane {
    /// Warm the lane over the wrapper source for the FIRST call.
    pub fn warm(
        source: &str,
        callee: &str,
        args: &[CteValue],
        is_bool: bool,
    ) -> Result<SnapshotLane, CteAdmissionError> {
        // The warm source must carry the FIRST callee's return-class shape
        // from the start (F-817-R2-01: a cold bool callee through an i32
        // wrapper was misreported as a LANGUAGE type error).
        let src = wrapper_source(source, callee, args, is_bool);
        let body_span = locate_body_span(&src)?;
        let session =
            auto_av_probe::session::warmup(&src, CTE_CALL_ENTRY_ID, &[]).map_err(|m| {
                // The 828 warm-up front door reports source diagnostics
                // verbatim; classify mark/capability shapes like the
                // closure lane.
                if m.contains("`mark_a`") || m.contains("`mark_b`") {
                    CteAdmissionError::new(
                        CteCode::CapabilityDenied,
                        format!("callee closure uses a trace intrinsic: {m}"),
                    )
                } else {
                    CteAdmissionError::new(CteCode::CalleeFrontend, m)
                }
            })?;
        Ok(SnapshotLane {
            session,
            body_span,
            whole_module_events: 1,
            incremental_edits: 0,
        })
    }

    /// Point the wrapper at a DIFFERENT call — the callee, its arguments
    /// AND the body shape (return-class projection) are replaced
    /// WHOLESALE (F-817-R1-01: replacing only the arguments silently
    /// kept the first callee AND the first return class). Only the
    /// wrapper function recompiles.
    pub fn retarget(
        &mut self,
        callee: &str,
        args: &[CteValue],
        is_bool: bool,
    ) -> Result<(), CteAdmissionError> {
        let bytes = wrapper_body(callee, args, is_bool).into_bytes();
        let out = self
            .session
            .edit(self.body_span.0, self.body_span.1, bytes)
            .map_err(|m| CteAdmissionError::new(CteCode::Infra("snapshot-edit".into()), m))?;
        if !out.ok {
            let mut detail = out.diagnostics.join("; ");
            if let Some(r) = out.non_fast_path_reason {
                if !detail.is_empty() {
                    detail.push_str("; ");
                }
                detail.push_str(&r);
            }
            if detail.is_empty() {
                detail = "retarget rejected".to_string();
            }
            return Err(CteAdmissionError::new(
                CteCode::Infra("snapshot-edit".into()),
                detail,
            ));
        }
        // Refresh the span for the next retarget (length may change).
        self.body_span = locate_body_span(self.session.src())?;
        self.incremental_edits += 1;
        Ok(())
    }

    /// The current session source text.
    pub fn src(&self) -> &str {
        self.session.src()
    }

    /// Apply a raw byte-span edit to the session source (dependency
    /// changes and failure controls; byte offsets, end exclusive).
    pub fn edit(&mut self, start: u64, end: u64, insert: Vec<u8>) -> Result<(), String> {
        let out = self.session.edit(start, end, insert)?;
        if out.ok {
            Ok(())
        } else {
            Err(if out.diagnostics.is_empty() {
                "edit rejected".to_string()
            } else {
                out.diagnostics.join("; ")
            })
        }
    }

    /// Run the current snapshot (real AV execution through the unit
    /// view). `is_bool` is the CURRENT callee's return class — retargets
    /// may switch callees, so the projection is per-run, never cached
    /// from the first call (F-817-R1-01).
    pub fn run(
        &self,
        is_bool: bool,
        cancel: &CancelFlag,
        wall_budget: std::time::Duration,
    ) -> Result<AvExecResult, CteAdmissionError> {
        let started = std::time::Instant::now();
        let (outcome, trace) = self.session.run(
            &ExecLimits {
                wall: wall_budget,
                ..Default::default()
            },
            cancel,
        );
        let wall_ms = started.elapsed().as_millis() as u64;
        match outcome {
            ExecOutcome::Success { value, steps } => {
                let cv = cte_value_of(&value, is_bool)?;
                Ok(AvExecResult {
                    value: cv,
                    steps,
                    wall_ms,
                    trace_events: trace.events.len(),
                    peak_slot_bytes: trace.peak_slot_bytes,
                })
            }
            ExecOutcome::Trap { kind, .. } => Err(CteAdmissionError::new(
                CteCode::Trap(kind.code().to_string()),
                format!("compiletime-executed snapshot trapped: {}", kind.code()),
            )),
            ExecOutcome::Resource { kind, message } => Err(CteAdmissionError::new(
                CteCode::Resource(format!("{kind:?}")),
                format!("snapshot resource limit: {message}"),
            )),
            ExecOutcome::Cancelled => Err(CteAdmissionError::new(
                CteCode::Cancelled,
                "snapshot execution cancelled",
            )),
            ExecOutcome::Infra(m) => Err(CteAdmissionError::new(
                CteCode::Infra("snapshot-exec".into()),
                format!("snapshot infra failure: {m}"),
            )),
        }
    }

    /// Observations for the V-12 sub-cases. NOTE: the session addresses
    /// functions by SOURCE name ("cte_sum"); a function's recompilation
    /// is observed at the mir_lower stage (the 828 stage vocabulary).
    pub fn compile_count(&self, function: &str) -> u64 {
        self.session.counters().stage_of("mir_lower", function)
    }
    pub fn whole_module_count(&self) -> u64 {
        // The 828 session's own counters are structurally 0 (silent
        // degradation is designed out); the lane's explicit whole-module
        // events (warm-ups) are the degrade detector's non-zero signal.
        self.whole_module_events
    }

    /// Incremental (non-degrading) edit count.
    pub fn incremental_edit_count(&self) -> u64 {
        self.incremental_edits
    }

    /// EVERY pipeline stage's count for one function (unchanged => all 0).
    pub fn stages_of(&self, function: &str) -> std::collections::BTreeMap<String, u64> {
        self.session.counters().stages_of(function)
    }

    /// The session's per-function artifact IO (bytes; None = the phase
    /// never ran — absent, never fabricated zeros).
    pub fn phase_io_of(&self, function: &str) -> Option<u64> {
        self.session
            .phase_io()
            .get(&format!("persist:{function}"))
            .copied()
    }
    pub fn generation_of(&self, function: &str) -> Option<u64> {
        self.session
            .resident(function)
            .map(|(_, _, c)| c.generation())
    }
}

/// One real AV execution result.
#[derive(Clone, Debug)]
pub struct AvExecResult {
    pub value: CteValue,
    pub steps: u64,
    pub wall_ms: u64,
    pub trace_events: usize,
    pub peak_slot_bytes: u64,
}

fn lit(v: &CteValue) -> String {
    match v {
        CteValue::I32(x) => format!("{x}"),
        CteValue::Bool(b) => format!("{b}"),
    }
}

/// Synthesize the wrapper entry source for one call. The scalar profile
/// grammar is statement-per-line, no semicolons, `name type` params
/// (matching the landed corpus: `fn score(a int, b int) int { ... }`).
pub fn wrapper_source(source: &str, callee: &str, args: &[CteValue], returns_bool: bool) -> String {
    let args_txt = args.iter().map(lit).collect::<Vec<_>>().join(", ");
    let body = if returns_bool {
        format!("    if {callee}({args_txt}) {{\n        return 1\n    }}\n    return 0\n")
    } else {
        format!("    return {callee}({args_txt})\n")
    };
    // The trailing anchor gives the incremental rescan its alignment
    // runway AFTER the wrapper's argument edits (an edit in the LAST
    // function cannot converge — nothing follows to align against).
    format!(
        "{source}fn {CTE_CALL_ENTRY}() int {{\n{body}}}\n\nfn cte817_anchor() int {{\n    return 0\n}}\n"
    )
}

/// Whether the callee's declared return is bool, per the projected 796
/// input (`input` is the projection of the ORIGINAL source).
pub fn callee_returns_bool(input: &ac_sema_core::SemaInput, callee: &str) -> bool {
    returns_bool(input, callee)
}

fn returns_bool(input: &ac_sema_core::SemaInput, callee: &str) -> bool {
    input
        .fns
        .iter()
        .find(|f| f.name == callee)
        .map(|f| matches!(f.ret, ac_sema_core::Ann::Bool))
        .unwrap_or(false)
}

/// Execute one CT call for real. `module_path` is the deck module id.
pub fn execute_call(
    source: &str,
    module_path: &str,
    input: &ac_sema_core::SemaInput,
    callee: &str,
    args: &[CteValue],
    forbid_av: bool,
    cancel: &CancelFlag,
    wall_budget: std::time::Duration,
) -> Result<AvExecResult, CteAdmissionError> {
    if forbid_av {
        return Err(CteAdmissionError::new(
            CteCode::Infra("av-forbidden".into()),
            "AV execution is forbidden by the counter-control flag (a cache miss cannot complete)",
        ));
    }
    if input.lookup_fn(callee).is_none() {
        return Err(CteAdmissionError::new(
            CteCode::UnknownQueryRef,
            format!("callee `{callee}` is not a declared function in the current input"),
        ));
    }
    let is_bool = returns_bool(input, callee);
    let src = wrapper_source(source, callee, args, is_bool);
    let snapshot = crate::fnv1a64(src.as_bytes());

    // Frontend with NO granted capabilities — the CTE closure must
    // require none (trace/text are refused here, before any AV call).
    let compiled = compile_scalar_source(&src, module_path, &[]).map_err(|ds| {
        let d = ds.first().cloned();
        let (code, msg) = match d {
            Some(x) => (x.code, x.message),
            None => ("source.unknown".into(), "frontend failed".into()),
        };
        if msg.contains("`mark_a`") || msg.contains("`mark_b`") {
            // Trace intrinsics resolve ONLY under `hir.test.trace.v1` —
            // a closure using them requires a capability the CTE profile
            // never grants (refused before any AV call).
            CteAdmissionError::new(
                CteCode::CapabilityDenied,
                format!(
                    "callee closure uses a trace intrinsic ({}): requires hir.test.trace.v1, which the CTE profile never grants",
                    msg
                ),
            )
        } else if code.starts_with("capability") {
            CteAdmissionError::new(
                CteCode::CapabilityDenied,
                format!("callee closure requires a capability the CTE profile never grants: {msg}"),
            )
        } else {
            // A callee-side source failure (parse/type/unknown-callee —
            // the profile has no imports, an undeclared name is exactly
            // this) is a LANGUAGE diagnostic, never infra.
            CteAdmissionError::new(CteCode::CalleeFrontend, format!("{code} {msg}"))
        }
    })?;
    let checked = &compiled.checked;
    if !checked.required_capabilities().is_empty() {
        return Err(CteAdmissionError::new(
            CteCode::CapabilityDenied,
            format!(
                "callee closure requires capabilities {:?} — CTE grants none",
                checked.required_capabilities()
            ),
        ));
    }
    let owner = checked.bundle().modules[0].id_text.clone();

    let mir = lower_checked(checked, CTE_CALL_ENTRY_ID, snapshot).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("mir-lower".into()),
            format!("lowering failed: {e:?}"),
        )
    })?;
    let _checked_mir = verify_mir(&mir).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("mir-verify".into()),
            format!("MIR verification failed: {e:?}"),
        )
    })?;

    let abc = encode_abc(&mir).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("abc-emit".into()),
            format!("ABC encode failed: {e:?}"),
        )
    })?;
    let decoded = decode_abc(&abc.to_flat()).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("abc-decode".into()),
            format!("ABC decode failed: {e:?}"),
        )
    })?;
    let owner_scope = auto_av_core::mir::OwnerScope {
        schema: ABC_SCHEMA.to_string(),
        module: owner,
    };
    let loaded = abc_verify::verify_abc(decoded, &owner_scope).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("abc-verify".into()),
            format!("ABC admission failed: {e:?}"),
        )
    })?;

    let started = std::time::Instant::now();
    let (outcome, trace) = execute(
        &loaded,
        &ExecLimits {
            wall: wall_budget,
            ..Default::default()
        },
        &cancel,
    );
    let wall_ms = started.elapsed().as_millis() as u64;
    match outcome {
        ExecOutcome::Success { value, steps } => {
            let cv = cte_value_of(&value, is_bool)?;
            Ok(AvExecResult {
                value: cv,
                steps,
                wall_ms,
                trace_events: trace.events.len(),
                peak_slot_bytes: trace.peak_slot_bytes,
            })
        }
        ExecOutcome::Trap { kind, .. } => Err(CteAdmissionError::new(
            CteCode::Trap(kind.code().to_string()),
            // The CT call chain's call SOURCE rides the diagnostic (V-02).
            format!(
                "compiletime-executed callee `{callee}`({}) trapped: {}",
                args.iter().map(lit).collect::<Vec<_>>().join(", "),
                kind.code()
            ),
        )),
        ExecOutcome::Resource { kind, message } => Err(CteAdmissionError::new(
            CteCode::Resource(format!("{kind:?}")),
            format!("AV resource limit: {message}"),
        )),
        ExecOutcome::Cancelled => Err(CteAdmissionError::new(
            CteCode::Cancelled,
            "AV execution cancelled",
        )),
        ExecOutcome::Infra(m) => Err(CteAdmissionError::new(
            CteCode::Infra("exec".into()),
            format!("AV infra failure: {m}"),
        )),
    }
}

#[cfg(test)]
mod value_fold_tests {
    //! F820-R10-01: the shared-core Value extension must not break the
    //! CTE lanes — old semantics byte-identical, data values rejected
    //! in a controlled way (this fold is the single consumption point
    //! both lanes share).
    use super::*;

    #[test]
    fn old_scalar_semantics_are_unchanged() {
        assert_eq!(
            cte_value_of(&Value::I32(0), true).unwrap(),
            CteValue::Bool(false)
        );
        assert_eq!(
            cte_value_of(&Value::I32(0), false).unwrap(),
            CteValue::I32(0)
        );
        assert_eq!(
            cte_value_of(&Value::I32(42), true).unwrap(),
            CteValue::Bool(true)
        );
        assert_eq!(
            cte_value_of(&Value::I32(-1), false).unwrap(),
            CteValue::I32(-1)
        );
        assert_eq!(
            cte_value_of(&Value::Bool(true), true).unwrap(),
            CteValue::Bool(true)
        );
        assert_eq!(
            cte_value_of(&Value::Bool(false), false).unwrap(),
            CteValue::Bool(false)
        );
    }

    #[test]
    fn data_value_extension_is_a_controlled_rejection() {
        for v in [
            Value::I64(i64::MIN),
            Value::U64(u64::MAX),
            Value::F32(1.5),
            Value::F64(-0.0),
        ] {
            let err = cte_value_of(&v, false)
                .expect_err("data values are outside the CTE scalar surface");
            assert!(
                matches!(err.code, CteCode::CalleeFrontend),
                "{:?} -> {err:?}",
                v
            );
            assert!(err
                .message
                .contains("outside the cte/scalar-query-v1 surface"));
        }
        let rec = Value::Record(std::rc::Rc::new(auto_av_core::contract::RecordValue {
            ty: 0,
            fields: vec![],
        }));
        let err = cte_value_of(&rec, true).expect_err("records are rejected");
        assert!(matches!(err.code, CteCode::CalleeFrontend));
    }
}
