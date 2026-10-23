//! The AV driver: one real CT call through the merged 798 services
//! (compile → capability audit → lower → verify_mir → ABC roundtrip →
//! verify_abc → `execute_entry`). The 798 module-closure entry contract
//! is zero-param i32, so a parameterized callee is invoked through a
//! synthesized wrapper entry with the constant arguments baked in — the
//! CALLEE still executes on the AV (that is the ct_execute=1 fact), and
//! bool results project through an `if` (no new traps).

use auto_ac_prototype::source::compile_scalar_source;
use auto_av_core::contract::{
    ExecOutcome, ABC_SCHEMA, ABC_SEMANTICS, AV_PROFILE, AV_SEMANTICS, MIR_SCHEMA, MIR_SEMANTICS,
};
use auto_av_core::{
    abc_verify, decode_abc, encode_abc, execute, verify_mir, CancelFlag, ExecLimits,
};
use auto_av_probe::hir_adapter::lower_checked;
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
pub fn current_engine() -> CteEngineId {
    CteEngineId {
        av_profile: AV_PROFILE.to_string(),
        mir_schema: MIR_SCHEMA.to_string(),
        abc_schema: ABC_SCHEMA.to_string(),
        mir_semantics: MIR_SEMANTICS.0,
        abc_semantics: ABC_SEMANTICS.0,
        av_semantics: AV_SEMANTICS.0,
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
fn wrapper_source(source: &str, callee: &str, args: &[CteValue], returns_bool: bool) -> String {
    let args_txt = args.iter().map(lit).collect::<Vec<_>>().join(", ");
    let body = if returns_bool {
        format!("    if {callee}({args_txt}) {{\n        return 1\n    }}\n    return 0\n")
    } else {
        format!("    return {callee}({args_txt})\n")
    };
    format!("{source}fn {CTE_CALL_ENTRY}() int {{\n{body}}}\n")
}

/// Whether the callee's declared return is bool, per the projected 796
/// input (`input` is the projection of the ORIGINAL source).
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
    let (outcome, trace) = execute(&loaded, &ExecLimits::default(), &cancel);
    let wall_ms = started.elapsed().as_millis() as u64;
    match outcome {
        ExecOutcome::Success { value, steps } => {
            let cv = match (&value, is_bool) {
                (auto_av_core::Value::I32(x), true) => CteValue::Bool(*x != 0),
                (auto_av_core::Value::I32(x), false) => CteValue::I32(*x),
                (auto_av_core::Value::Bool(b), _) => CteValue::Bool(*b),
            };
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
