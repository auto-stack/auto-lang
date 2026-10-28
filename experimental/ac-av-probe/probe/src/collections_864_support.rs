//! PLAN-864 support: the source→collections full-chain seam used by the
//! driver/worker and the feasibility tests — compile (frontend + verify) →
//! lower (`lower_checked_collections`) → verify_mir → ABC admit → execute.
//! The 822 fixture's exact gate order: UNTRUSTED builder output →
//! verify_mir → encode_abc → verify_abc_bytes (decode + full re-verify) →
//! execute; the arena rides the ExecTrace.

use auto_ac_prototype::source::compile_collections_source;
use auto_av_core::abc::encode_abc;
use auto_av_core::av::{execute, ExecLimits};
use auto_av_core::contract::Value;
use auto_av_core::verify_mir;
use auto_mir_lower::lower_checked_collections;

/// One execution face for the feasibility tests and the worker's
/// collections source path (PLAN-864).
pub struct CollectionsRun {
    pub value: i32,
    pub live_after: usize,
    pub allocs: usize,
    pub frees: usize,
}

pub fn compile_and_execute(src: &str, entry_name: &str) -> Result<CollectionsRun, String> {
    let compiled = compile_collections_source(src, "probe.at", &[]).map_err(|ds| {
        ds.iter()
            .map(|d| d.render("probe.at", src))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mir = lower_checked_collections(&compiled.checked, entry_name, 0)
        .map_err(|e| format!("lower: {e:?}"))?;
    let _checked = verify_mir(&mir).map_err(|e| format!("verify_mir: {e:?}"))?;
    let abc = encode_abc(&mir).map_err(|e| format!("encode: {e:?}"))?;
    let flat = abc.to_flat();
    let loaded = auto_av_core::abc_verify::verify_abc_bytes(&flat, &mir.owner)
        .map_err(|e| format!("abc admit: {e}"))?;
    let (out, trace) = execute(
        &loaded,
        &ExecLimits::default(),
        &auto_av_core::CancelFlag::new(false),
    );
    match out {
        auto_av_core::contract::ExecOutcome::Success { value, .. } => {
            let v = match value {
                Value::I32(v) => v,
                other => return Err(format!("non-i32 entry result: {other:?}")),
            };
            let live_after = trace.arena.as_ref().map(|a| a.live()).unwrap_or(0);
            let allocs = trace
                .arena
                .as_ref()
                .map(|a| a.alloc_events().len())
                .unwrap_or(0);
            let frees = trace
                .arena
                .as_ref()
                .map(|a| a.free_events().len())
                .unwrap_or(0);
            Ok(CollectionsRun {
                value: v,
                live_after,
                allocs,
                frees,
            })
        }
        auto_av_core::contract::ExecOutcome::Trap { kind, .. } => {
            Err(format!("trap: {}", kind.code()))
        }
        other => Err(format!("non-success outcome: {other:?}")),
    }
}
