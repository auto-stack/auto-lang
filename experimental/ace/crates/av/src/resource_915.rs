//! PLAN-915 R02/R03 scenario fixtures: the allocation-fault sweep and the
//! stale-Place sequence, run through the REAL pipeline
//! (source → checked HIR → shared lowering → `verify_mir` → ABC admit → AV
//! executor/arena) and reported as bounded operational facts.
//!
//! R02: every actual allocation attempt of the frozen corpus programs
//! (L04/L05/V05 through the collections frontend, K02 through the closures
//! frontend) is injected one at a time on the two existing seams —
//! `ExecLimits::fault_after_allocs` (value allocations: slot writes,
//! closure-env materialization, list capacity growth) and
//! `StrArena::set_fault_at` (owned Str/List buffer attempts) — and the
//! post-failure state (nothing committed, partial output reclaimed, the
//! source intact) is observed, not assumed.
//!
//! R03: AutoLang source cannot mint a List Place, so the runtime witness is
//! a typed MIR module that passes the SAME verifier gates as source
//! (`verify_mir` → `encode_abc` → `verify_abc_bytes`) and runs on the real
//! executor: mint a place, read it legally, mutate the list structurally
//! (push/insert/remove/element replace), then use the OLD place — the
//! validated-before-access trap and the gated `PlaceFacts` counters are the
//! evidence that no buffer access happened for the stale use.

use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::{verify_abc_bytes, LoadedModule};
use ace_av_core::av::{execute_with_arena, CancelFlag, ExecLimits, PlaceFacts};
use ace_av_core::contract::{ExecOutcome, Value, MIR_COLLECTIONS_SCHEMA};
use ace_av_core::mir::{
    FnSig, ListTypeRef, MirBlock, MirConst, MirFunction, MirInstr, MirModule, MirSlot, MirType,
    OwnerScope, SlotKind, Terminator,
};
use ace_av_core::mir_verify::verify_mir;
use ace_av_core::owned_text::StrArena;

/// The recipe id the worker and the parent share.
pub const RESOURCE_RECIPE: &str = "ac-av/resource-run-v1";

/// Which real frontend turns the scenario source into checked HIR.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceFrontend {
    /// `compile_collections_source` (L04/L05/V05 domain).
    Collections,
    /// `compile_closures_source` (K02 capture domain).
    Closures,
}

impl SourceFrontend {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "collections" => Some(SourceFrontend::Collections),
            "closures" => Some(SourceFrontend::Closures),
            _ => None,
        }
    }
}

/// Compile one scenario source into an admitted `LoadedModule` through the
/// same gates the formal recipes use: frontend → lowering → `verify_mir` →
/// ABC encode → `verify_abc_bytes`.
pub fn compile_module(
    src: &str,
    entry: &str,
    frontend: SourceFrontend,
) -> Result<LoadedModule, String> {
    let mir = match frontend {
        SourceFrontend::Collections => {
            let compiled = ace_core::source::compile_collections_source(src, "r915.at", &[])
                .map_err(|ds| {
                    ds.iter()
                        .map(|d| d.render("r915.at", src))
                        .collect::<Vec<_>>()
                        .join("\n")
                })?;
            ace_lowering::lower_checked_collections(&compiled.checked, entry, 0)
                .map_err(|e| format!("lower: {e:?}"))?
        }
        SourceFrontend::Closures => {
            let compiled =
                ace_core::source::compile_closures_source(src, "r915.at", &[]).map_err(|ds| {
                    ds.iter()
                        .map(|d| d.render("r915.at", src))
                        .collect::<Vec<_>>()
                        .join("\n")
                })?;
            ace_lowering::lower_checked_closures(&compiled.checked, entry, 0)
                .map_err(|e| format!("lower: {e:?}"))?
        }
    };
    let owner = mir.owner.clone();
    verify_mir(&mir).map_err(|e| format!("verify_mir: {e:?}"))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode: {e:?}"))?;
    verify_abc_bytes(&wire.to_flat(), &owner).map_err(|e| format!("abc admit: {e}"))
}

fn i32_of(out: &ExecOutcome) -> Result<i32, String> {
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => Ok(*v),
        other => Err(format!("expected an i32 success, got {other:?}")),
    }
}

fn live_of(trace: &ace_av_core::av::ExecTrace) -> usize {
    trace.arena.as_ref().map(|a| a.live()).unwrap_or(0)
}

/// PLAN-915 R02: sweep every value-allocation attempt (1..=N) and every
/// owned-arena attempt (1..=N'), plus the N+1 / N'+1 positive controls and
/// the no-fault control.
pub fn source_fault_sweep(
    src: &str,
    entry: &str,
    frontend: SourceFrontend,
) -> Result<serde_json::Value, String> {
    let loaded = compile_module(src, entry, frontend)?;
    let cancel = CancelFlag::new(false);

    // No-fault control: the frozen result and the clean ledger.
    let clean_arena = StrArena::new();
    let (clean_out, clean_trace) = execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &cancel,
        clean_arena.clone(),
    );
    let clean_value = i32_of(&clean_out)?;
    let clean_live = live_of(&clean_trace);
    let arena_attempts = clean_arena.attempts();
    let arena_success = clean_arena.successful_allocs();
    let clean_frees = clean_arena.free_events().len();
    if clean_live != 0 {
        return Err(format!(
            "the no-fault control leaked {clean_live} live owned buffers"
        ));
    }

    // Discovery of the value-allocation ordinal: arm the seam one past any
    // possible count; the executor reports how many attempts it really made.
    let mut discovery = ExecLimits::default();
    discovery.fault_after_allocs = Some(u64::MAX);
    let (discovery_out, discovery_trace) =
        execute_with_arena(&loaded, &discovery, &cancel, StrArena::new());
    i32_of(&discovery_out)?;
    let value_attempts = discovery_trace.alloc_events;

    // Arena seam sweep: fail attempt k, expect the distinct Infra state,
    // zero live buffers and no half-committed count.
    let mut arena_points = Vec::new();
    for attempt in 1..=arena_attempts {
        let arena = StrArena::new();
        arena.set_fault_at(attempt);
        let (out, trace) = execute_with_arena(&loaded, &ExecLimits::default(), &cancel, arena);
        let live = live_of(&trace);
        let ledger = trace.arena.clone().expect("execute attaches the ledger");
        let (faulted, message) = match &out {
            ExecOutcome::Infra(m) => (m.contains("fault-injected"), Some(m.clone())),
            _ => (false, None),
        };
        arena_points.push(serde_json::json!({
            "attempt": attempt,
            "faulted": faulted,
            "live": live,
            "attempts": ledger.attempts(),
            "successful": ledger.successful_allocs(),
            "frees": ledger.free_events().len(),
            "message": message,
        }));
    }
    // N+1 positive control: one past the last real attempt must succeed with
    // the frozen value and a clean ledger.
    let control_arena = StrArena::new();
    control_arena.set_fault_at(arena_attempts + 1);
    let (control_out, control_trace) = execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &cancel,
        control_arena.clone(),
    );
    let arena_control = serde_json::json!({
        "attempt": arena_attempts + 1,
        "faulted": false,
        "live": live_of(&control_trace),
        "value": i32_of(&control_out).ok(),
    });

    // Value seam sweep (slot writes, closure-env materialization and list
    // capacity growth share the one ordinal).
    let mut value_points = Vec::new();
    for attempt in 1..=value_attempts {
        let mut limits = ExecLimits::default();
        limits.fault_after_allocs = Some(attempt);
        let (out, trace) = execute_with_arena(&loaded, &limits, &cancel, StrArena::new());
        let (faulted, message) = match &out {
            ExecOutcome::Infra(m) => (m.contains("fault-injected"), Some(m.clone())),
            _ => (false, None),
        };
        value_points.push(serde_json::json!({
            "attempt": attempt,
            "faulted": faulted,
            "live": live_of(&trace),
            "message": message,
        }));
    }
    let mut value_control_limits = ExecLimits::default();
    value_control_limits.fault_after_allocs = Some(value_attempts + 1);
    let (value_control_out, value_control_trace) =
        execute_with_arena(&loaded, &value_control_limits, &cancel, StrArena::new());
    let value_control = serde_json::json!({
        "attempt": value_attempts + 1,
        "faulted": false,
        "live": live_of(&value_control_trace),
        "value": i32_of(&value_control_out).ok(),
    });

    // The same session re-runs correctly after the fault is cleared (the
    // failure never cached a PASS or a half product).
    let after_arena = StrArena::new();
    let (after_out, after_trace) =
        execute_with_arena(&loaded, &ExecLimits::default(), &cancel, after_arena);
    let after_value = i32_of(&after_out)?;

    // Recovery is a FULL re-verification, never a cached run result: the
    // source is recompiled through the whole gate chain and re-executed.
    let recompiled = compile_module(src, entry, frontend)?;
    let (recompiled_out, recompiled_trace) = execute_with_arena(
        &recompiled,
        &ExecLimits::default(),
        &cancel,
        StrArena::new(),
    );
    let recompiled_value = i32_of(&recompiled_out)?;

    Ok(serde_json::json!({
        "source_value": clean_value,
        "arena_attempts": arena_attempts,
        "arena_successful": arena_success,
        "arena_frees": clean_frees,
        "value_attempts": value_attempts,
        "arena_points": arena_points,
        "value_points": value_points,
        "arena_control": arena_control,
        "value_control": value_control,
        "after": {
            "value": after_value,
            "live": live_of(&after_trace),
        },
        "after_recompile": {
            "value": recompiled_value,
            "live": live_of(&recompiled_trace),
        },
    }))
}

// ---------------------------------------------------------------------------
// R03: the typed List-Place witness
// ---------------------------------------------------------------------------

const TY_I32: u32 = 0;
const TY_LIST: u32 = 1;
const TY_PLACE: u32 = 2;

/// The structural mutation a stale-place witness performs between the mint
/// and the second use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceMutation {
    Push,
    Insert,
    Remove,
    Replace,
}

impl PlaceMutation {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "push" => Some(PlaceMutation::Push),
            "insert" => Some(PlaceMutation::Insert),
            "remove" => Some(PlaceMutation::Remove),
            "replace" => Some(PlaceMutation::Replace),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            PlaceMutation::Push => "push",
            PlaceMutation::Insert => "insert",
            PlaceMutation::Remove => "remove",
            PlaceMutation::Replace => "replace",
        }
    }
}

struct Builder {
    m: MirModule,
    slots: Vec<MirSlot>,
    instrs: Vec<MirInstr>,
}

impl Builder {
    fn new() -> Self {
        let mut m = MirModule::new_collections(OwnerScope {
            schema: MIR_COLLECTIONS_SCHEMA.to_string(),
            module: "resource915".to_string(),
        });
        m.types = vec![
            MirType::I32,
            MirType::List(ListTypeRef { elem: TY_I32 }),
            MirType::Place(ListTypeRef { elem: TY_I32 }),
        ];
        Builder {
            m,
            slots: Vec::new(),
            instrs: Vec::new(),
        }
    }

    fn slot(&mut self, ty: u32, name: &str) -> u32 {
        self.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty,
            name: Some(name.into()),
        });
        (self.slots.len() - 1) as u32
    }

    fn i(&mut self, instr: MirInstr) {
        self.instrs.push(instr);
    }

    fn iconst(&mut self, dst: u32, value: i32) {
        let c = self.m.intern_const(MirConst::I32(value)) as u32;
        self.i(MirInstr::Const { dst, value: c });
    }

    fn build(mut self, ret: u32) -> MirModule {
        let sig = self.m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: TY_I32,
        }));
        self.m.funcs = vec![MirFunction {
            owner: self.m.owner.clone(),
            name: "main".into(),
            sig,
            slots: self.slots,
            blocks: vec![MirBlock {
                instrs: self.instrs,
                term: Terminator::Return { value: ret },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: Vec::new(),
            env: None,
        }];
        self.m.entry = 0;
        self.m
    }
}

/// Build the typed R03 witness: a legal pre-mutation read of a minted place,
/// the structural mutation, then the same (now stale) place's use.
pub fn place_module(mutation: Option<PlaceMutation>) -> MirModule {
    let mut b = Builder::new();
    let list = b.slot(TY_LIST, "list");
    let v = b.slot(TY_I32, "v");
    let idx = b.slot(TY_I32, "idx");
    let p = b.slot(TY_PLACE, "p");
    let got = b.slot(TY_I32, "got");
    let out = b.slot(TY_I32, "out");

    b.i(MirInstr::ListNew {
        dst: list,
        list_ty: TY_LIST,
        cap: 0,
    });
    b.iconst(v, 10);
    b.i(MirInstr::ListPush {
        root: list,
        value: v,
    });
    b.iconst(v, 20);
    b.i(MirInstr::ListPush {
        root: list,
        value: v,
    });

    // Mint over the element the mutation is about to disturb, then read it
    // once (the legal face — the token really was usable).
    let mint_index = match mutation {
        Some(PlaceMutation::Insert) | Some(PlaceMutation::Remove) => 1,
        _ => 0,
    };
    b.iconst(idx, mint_index);
    b.i(MirInstr::ListPlace {
        dst: p,
        base: list,
        index: idx,
    });
    b.i(MirInstr::PlaceGet { dst: got, src: p });

    match mutation {
        Some(PlaceMutation::Push) => {
            b.iconst(v, 30);
            b.i(MirInstr::ListPush {
                root: list,
                value: v,
            });
        }
        Some(PlaceMutation::Insert) => {
            b.iconst(v, 30);
            b.i(MirInstr::ListInsert {
                root: list,
                index: idx,
                value: v,
            });
        }
        Some(PlaceMutation::Remove) => {
            b.iconst(idx, 0);
            b.i(MirInstr::ListRemove {
                dst: out,
                root: list,
                index: idx,
            });
        }
        Some(PlaceMutation::Replace) => {
            b.iconst(v, 99);
            b.i(MirInstr::ListSet {
                root: list,
                index: idx,
                value: v,
            });
        }
        None => {}
    }

    // The judged use: before any mutation this reads the minted element;
    // after one it must be rejected as stale BEFORE any buffer access.
    b.i(MirInstr::PlaceGet { dst: out, src: p });
    b.build(out)
}

/// A fresh-Place positive control: after the mutation a NEW token over the
/// same list reads the current element (legal neighbours stay usable).
fn fresh_module(mutation: PlaceMutation) -> MirModule {
    let mut b = Builder::new();
    let list = b.slot(TY_LIST, "list");
    let v = b.slot(TY_I32, "v");
    let idx = b.slot(TY_I32, "idx");
    let p = b.slot(TY_PLACE, "p");
    let got = b.slot(TY_I32, "got");

    b.i(MirInstr::ListNew {
        dst: list,
        list_ty: TY_LIST,
        cap: 0,
    });
    b.iconst(v, 10);
    b.i(MirInstr::ListPush {
        root: list,
        value: v,
    });
    b.iconst(v, 20);
    b.i(MirInstr::ListPush {
        root: list,
        value: v,
    });
    match mutation {
        PlaceMutation::Push => {
            b.iconst(v, 30);
            b.i(MirInstr::ListPush {
                root: list,
                value: v,
            });
        }
        PlaceMutation::Insert => {
            b.iconst(v, 30);
            b.iconst(idx, 1);
            b.i(MirInstr::ListInsert {
                root: list,
                index: idx,
                value: v,
            });
        }
        PlaceMutation::Remove => {
            b.iconst(idx, 0);
            b.i(MirInstr::ListRemove {
                dst: got,
                root: list,
                index: idx,
            });
        }
        PlaceMutation::Replace => {
            b.iconst(v, 99);
            b.iconst(idx, 0);
            b.i(MirInstr::ListSet {
                root: list,
                index: idx,
                value: v,
            });
        }
    }
    // The fresh token lands AFTER the structural change and reads the
    // current element at the index the mutation left there.
    let index = match mutation {
        PlaceMutation::Push => 2,
        PlaceMutation::Insert => 1,
        PlaceMutation::Remove => 0,
        PlaceMutation::Replace => 0,
    };
    b.iconst(idx, index);
    b.i(MirInstr::ListPlace {
        dst: p,
        base: list,
        index: idx,
    });
    b.i(MirInstr::PlaceGet { dst: got, src: p });
    b.build(got)
}

fn admit(m: &MirModule) -> Result<LoadedModule, String> {
    let owner = m.owner.clone();
    verify_mir(m).map_err(|e| format!("verify_mir: {e:?}"))?;
    let wire = encode_abc(m).map_err(|e| format!("encode: {e:?}"))?;
    verify_abc_bytes(&wire.to_flat(), &owner).map_err(|e| format!("abc admit: {e}"))
}

fn facts_json(facts: PlaceFacts) -> serde_json::Value {
    serde_json::json!({
        "mints": facts.mints,
        "get_ok": facts.get_ok,
        "get_stale": facts.get_stale,
        "set_ok": facts.set_ok,
        "set_stale": facts.set_stale,
        "buffer_reads": facts.buffer_reads,
    })
}

pub fn run_place_module(m: &MirModule, observe: bool) -> Result<serde_json::Value, String> {
    let loaded = admit(m)?;
    let mut limits = ExecLimits::default();
    limits.observe_places = observe;
    let (out, trace) =
        execute_with_arena(&loaded, &limits, &CancelFlag::new(false), StrArena::new());
    let facts = trace
        .place_facts
        .map(facts_json)
        .unwrap_or(serde_json::Value::Null);
    let live = live_of(&trace);
    Ok(match &out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => serde_json::json!({
            "outcome": "success",
            "value": v,
            "trap": null,
            "facts": facts,
            "live": live,
        }),
        ExecOutcome::Trap { kind, .. } => serde_json::json!({
            "outcome": "trap",
            "value": null,
            "trap": kind.code(),
            "facts": facts,
            "live": live,
        }),
        other => return Err(format!("unexpected place outcome: {other:?}")),
    })
}

/// R03 face: the stale witness for `mutation`, the fresh-token positive
/// control beside it, and the legal no-mutation control.
pub fn place_sequence(mutation: PlaceMutation) -> Result<serde_json::Value, String> {
    let stale = run_place_module(&place_module(Some(mutation)), true)?;
    let fresh = run_place_module(&fresh_module(mutation), true)?;
    let legal = run_place_module(&place_module(None), true)?;
    Ok(serde_json::json!({
        "mutation": mutation.as_str(),
        "stale": stale,
        "fresh": fresh,
        "legal": legal,
    }))
}
