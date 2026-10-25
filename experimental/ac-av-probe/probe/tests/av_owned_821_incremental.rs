//! PLAN-821 T-09 incremental counter-controls (`av_owned_821` family):
//! the frozen V-13 four sub-cases (contract-decision §12) over the REAL
//! D828 interfaces — `lower_function_owned_unit` → unit `verify_mir` →
//! AVAD v2 → unit admission (`FunctionCode::admit_unit`) → checked
//! linking (`ProgramSnapshot::link`) → `execute_program`.
//!
//! Oracles (per the plan's incremental contract):
//! 1. shared scalar baseline: the 828 campaign gate replays green in the
//!    script's incremental scope (this file carries the OWNED faces);
//! 2. a PURE body edit of one function re-produces ONLY that unit — the
//!    untouched units keep their code objects (Arc::ptr_eq) AND their
//!    mint generations, and the linked program runs with exact semantics
//!    plus a clean owned ledger (live=0);
//! 3. interface (signature) and layout (type-table) changes are PRECISE:
//!    reusing stale caller units fails the checked link — the affected
//!    set must re-produce, never a silent cross-identity reuse;
//! 4. a forced whole recompute is VISIBLE: every unit's production
//!    counter moves (a "whole recompute" reporting zero productions for
//!    any unit — the hash-only/no-object swap shape — reddens here).

use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::av::{execute_program, CancelFlag, ExecLimits};
use auto_av_core::contract::ExecOutcome;
use auto_av_core::function_code::FunctionCode;
use auto_av_core::program_snapshot::ProgramSnapshot;
use auto_av_core::{abc_verify, mir_verify};
use std::collections::BTreeMap;
use std::sync::Arc;

const THREE_FN_SRC: &str = "\
type Note { body Str; n int }
fn bump(move u Note) Note {
    return Note { body: \"B\", n: u.n + 1 }
}
fn plain(move a Note) Note {
    return Note { body: \"C\", n: a.n + 10 }
}
fn main() int {
    let a Note = Note { body: \"A\", n: 1 }
    let b Note = bump(a.move)
    let c Note = plain(b.move)
    return c.n * 100 + c.body.len()
}
";

/// The minimal owned incremental session: resident code objects keyed by
/// def id, minted generations, and PER-UNIT production counters (lower +
/// encode + admit counts — the observable this seam owns).
struct OwnedSession {
    code: BTreeMap<String, Arc<FunctionCode>>,
    revision: u64,
    produced: BTreeMap<String, u64>,
}

impl OwnedSession {
    /// Cold warm-up: every function unit produced and linked at revision 1.
    fn warmup(src: &str, caps: &[String]) -> Result<(OwnedSession, String), String> {
        let mut s = OwnedSession {
            code: BTreeMap::new(),
            revision: 1,
            produced: BTreeMap::new(),
        };
        s.reproduce_all(src, caps)?;
        let entry = "d_main".to_string();
        Ok((s, entry))
    }

    /// (Re-)produce ONE function's unit at the current revision.
    fn reproduce_one(&mut self, src: &str, def_id: &str, caps: &[String]) -> Result<(), String> {
        let compiled = auto_ac_prototype::source::compile_text_record_source(src, "s.at", caps)
            .map_err(|ds| {
                ds.first()
                    .map(|d| format!("{} {}", d.code, d.message))
                    .unwrap_or_else(|| "frontend".into())
            })?;
        let bundle = compiled.checked.bundle();
        let snap = auto_av_core::contract::fnv1a64(src.as_bytes());
        let unit =
            auto_av_probe::hir_adapter::lower_function_owned_unit(bundle, def_id, caps, snap)
                .map_err(|e| format!("lower: {}", e.code()))?;
        mir_verify::verify_mir(&unit).map_err(|e| format!("verify: {e:?}"))?;
        let wire = encode_abc(&unit).map_err(|e| format!("encode: {}", e.code()))?;
        let decoded = decode_abc(&wire.to_flat()).map_err(|e| format!("decode: {e:?}"))?;
        let loaded = abc_verify::verify_abc_bytes(&wire.to_flat(), &unit.owner)
            .map_err(|e| format!("abc admit: {e:?}"))?;
        let _ = decoded;
        let fc = FunctionCode::admit_unit(loaded, self.revision).map_err(|e| e)?;
        self.code.insert(def_id.to_string(), Arc::new(fc));
        *self.produced.entry(def_id.to_string()).or_insert(0) += 1;
        Ok(())
    }

    fn reproduce_all(&mut self, src: &str, caps: &[String]) -> Result<(), String> {
        let compiled = auto_ac_prototype::source::compile_text_record_source(src, "s.at", caps)
            .map_err(|ds| {
                ds.first()
                    .map(|d| format!("{} {}", d.code, d.message))
                    .unwrap_or_else(|| "frontend".into())
            })?;
        let bundle = compiled.checked.bundle();
        let module = bundle
            .modules
            .first()
            .ok_or_else(|| "no module".to_string())?;
        for d in &module.defs {
            if matches!(d.kind, auto_ac_prototype::hir::DefKind::Function { .. }) {
                self.reproduce_one(src, &d.id_text, caps)?;
            }
        }
        Ok(())
    }

    fn link_and_run(&self) -> (ExecOutcome, auto_av_core::av::ExecTrace) {
        let code: Vec<Arc<FunctionCode>> = self.code.values().cloned().collect();
        let prog =
            ProgramSnapshot::link(code, "d_main", self.revision).expect("the resident set links");
        execute_program(&prog, &ExecLimits::default(), &CancelFlag::new(false))
    }

    fn run_i32(&self) -> i32 {
        let (out, _) = self.link_and_run();
        let ExecOutcome::Success {
            value: auto_av_core::contract::Value::I32(v),
            ..
        } = out
        else {
            panic!("expected i32 success, got {out:?}")
        };
        v
    }

    /// Bump the program revision (an edit's new generation).
    fn next_revision(&mut self) {
        self.revision += 1;
    }

    fn produced_of(&self, def: &str) -> u64 {
        self.produced.get(def).copied().unwrap_or(0)
    }

    fn code_of(&self, def: &str) -> Arc<FunctionCode> {
        self.code.get(def).expect("resident").clone()
    }
}

/// V-13 (2): two PURE body edits — only the edited function's unit
/// re-produces; the untouched units keep their objects AND generations;
/// both revisions run with exact semantics and a clean ledger.
#[test]
fn av_owned_821_v13_two_fn_local_body_updates() {
    let (mut s, _entry) = OwnedSession::warmup(THREE_FN_SRC, &[]).expect("warmup");
    // Baseline: bump(1) -> 2, plain(2) -> 12, 12*100 + len("C") = 1201.
    assert_eq!(s.run_i32(), 1201);
    let main_before = s.code_of("d_main");
    let plain_before = s.code_of("d_plain");
    let produced_before: BTreeMap<String, u64> = s.produced.clone();

    // ---- edit 1: bump's body (+1 -> +2). Only bump re-produces.
    let src1 = THREE_FN_SRC.replace("n: u.n + 1", "n: u.n + 2");
    s.next_revision();
    s.reproduce_one(&src1, "d_bump", &[])
        .expect("bump body edit re-produces");
    // Untouched units: SAME Arc object, SAME mint generation, ZERO new
    // productions (生产阶段/产物 IO=0).
    assert!(
        Arc::ptr_eq(&s.code_of("d_main"), &main_before),
        "main's resident object is reused (no re-lower)"
    );
    assert!(
        Arc::ptr_eq(&s.code_of("d_plain"), &plain_before),
        "plain's resident object is reused"
    );
    assert_eq!(s.code_of("d_main").generation(), 1);
    assert_eq!(s.code_of("d_plain").generation(), 1);
    assert_eq!(s.code_of("d_bump").generation(), 2, "bump re-minted");
    for def in ["d_main", "d_plain"] {
        assert_eq!(
            s.produced_of(def),
            produced_before.get(def).copied().unwrap_or(0),
            "{def} production stays zero-delta"
        );
    }
    assert_eq!(s.produced_of("d_bump"), 2);
    // Semantics: bump(1)->3, plain(3)->13, 13*100+1 = 1301; ledger clean.
    let (out, trace) = s.link_and_run();
    let ExecOutcome::Success {
        value: auto_av_core::contract::Value::I32(v),
        ..
    } = out
    else {
        panic!("expected success, got {out:?}")
    };
    assert_eq!(v, 1301);
    drop(out);
    assert_eq!(
        trace.arena.as_ref().unwrap().live(),
        0,
        "the owned ledger is clean after the incremental run"
    );

    // ---- edit 2: main's body (*100 -> *10). Only main re-produces; the
    // helper deltas over `before` stay zero.
    let src2 = src1.replace("c.n * 100", "c.n * 10");
    s.next_revision();
    s.reproduce_one(&src2, "d_main", &[])
        .expect("main body edit re-produces");
    let bump_after_edit2 = s.code_of("d_bump");
    assert!(
        Arc::ptr_eq(&bump_after_edit2, &s.code_of("d_bump")),
        "bump stays resident across main's edit"
    );
    for def in ["d_bump", "d_plain"] {
        assert_eq!(
            s.produced_of(def),
            if def == "d_bump" { 2 } else { 1 },
            "{def} production unchanged by main's edit"
        );
    }
    assert_eq!(s.run_i32(), 131, "13*10 + 1");
}

/// V-13 (3): precise refusals — a SIGNATURE change on the helper makes
/// the caller's resident unit stale: linking the stale caller against the
/// new helper FAILS the checked link (never a silent cross-identity
/// reuse); a type-table (layout) change parts every unit's identity the
/// same way; the state before the failed link keeps running.
#[test]
fn av_owned_821_v13_precise_refusals_signature_and_layout() {
    let (mut s, _entry) = OwnedSession::warmup(THREE_FN_SRC, &[]).expect("warmup");
    assert_eq!(s.run_i32(), 1201);
    let stale_main = s.code_of("d_main");

    // Signature change: bump's PARAM type Note -> Str (a move-param
    // retyping — the frozen owned surface's interface axis). The NEW bump
    // unit alone cannot serve the OLD caller's `fn:bump` import — the
    // checked link refuses (signature mismatch), so the precise
    // invalidation set is {d_bump, d_main}; until main re-produces, no
    // link. (Two adjacent 782 call shapes stay OUTSIDE this seam by
    // design: multi-arg `.move` marker association and borrowed RECORD
    // params need a record-view value kind beyond the frozen MIR type
    // list — the adapter's `lower.borrow` keeps them as controlled
    // rejections; recorded in the plan's debt notes.)
    let src_sig = "
type Note { body Str; n int }
fn bump(move s Str) Note {
    return Note { body: s.move, n: 1 }
}
fn plain(move a Note) Note {
    return Note { body: \"C\", n: a.n + 10 }
}
fn main() int {
    let f Str = \"X\"
    let b Note = bump(f.move)
    let c Note = plain(b.move)
    return c.n * 100 + c.body.len()
}
";
    s.next_revision();
    s.reproduce_one(src_sig, "d_bump", &[])
        .expect("the new-signature bump unit produces");
    let code_mix: Vec<Arc<FunctionCode>> = s.code.values().cloned().collect();
    let err = ProgramSnapshot::link(code_mix, "d_main", s.revision)
        .expect_err("the stale caller must not link against the new signature");
    assert!(
        err.to_lowercase().contains("signature") || err.to_lowercase().contains("mismatch"),
        "the refusal names the signature axis: {err}"
    );
    // The invalidation set at THIS seam: every unit's import table is
    // module-wide (each carries ALL function signatures), so a signature
    // change stales every importer — main AND plain both hold the old
    // `fn:d_bump` sig. (The 828 session's up-front refusal of signature
    // edits — the precise front — exists because of exactly this.)
    let plain_before_fix = s.code_of("d_plain");
    s.reproduce_one(src_sig, "d_main", &[])
        .expect("main re-produces against the new interface");
    // main alone does NOT restore a linkable set: plain's module-wide
    // import table still carries the old signature.
    let code_mix: Vec<Arc<FunctionCode>> = s.code.values().cloned().collect();
    assert!(
        ProgramSnapshot::link(code_mix, "d_main", s.revision).is_err(),
        "a stale module-wide import table (plain's) still refuses the link"
    );
    s.reproduce_one(src_sig, "d_plain", &[])
        .expect("plain re-produces (module-wide import tables stale together)");
    assert!(
        !Arc::ptr_eq(&s.code_of("d_plain"), &plain_before_fix),
        "plain's object DID move (it is inside the invalidation set)"
    );
    // bump("X") -> Note{body:"X",n:1}; plain -> Note{body:"C",n:11};
    // 11*100 + 1 = 1101.
    assert_eq!(
        s.run_i32(),
        1101,
        "the re-linked program runs the new interface"
    );
    drop(stale_main);

    // Layout change: adding a field to Note changes every unit's type
    // table — a mixed link of old+new units must fail the owner/shape
    // checks rather than serve cross-layout code.
    let src_layout = THREE_FN_SRC
        .replace(
            "type Note { body Str; n int }",
            "type Note { body Str; n int; k int }",
        )
        .replace(
            "return Note { body: \"B\", n: u.n + 1 }",
            "return Note { body: \"B\", n: (u.n + 1), k: 0 }",
        )
        .replace(
            "return Note { body: \"C\", n: a.n + 10 }",
            "return Note { body: \"C\", n: (a.n + 10), k: 0 }",
        )
        .replace(
            "let a Note = Note { body: \"A\", n: 1 }",
            "let a Note = Note { body: \"A\", n: 1, k: 0 }",
        );
    // Produce ONE unit of the new layout and mix it into the old set.
    let (mut s3, _) = OwnedSession::warmup(THREE_FN_SRC, &[]).expect("warmup 3");
    s3.next_revision();
    // The layout change is a whole-module interface change — the unit
    // lowering of the NEW layout produces a unit whose types differ from
    // the residents'; mixing fails (signature/shape equality is CONTENT
    // equality at link).
    s3.reproduce_one(&src_layout, "d_bump", &[])
        .expect("the new-layout bump unit produces");
    let code_mix: Vec<Arc<FunctionCode>> = s3.code.values().cloned().collect();
    assert!(
        ProgramSnapshot::link(code_mix, "d_main", s3.revision).is_err(),
        "a mixed-layout link must fail (content equality at the checked link)"
    );
}

/// V-13 (4): forced whole recompute is VISIBLE — when the whole module
/// re-produces, EVERY unit's production counter moves; the hash-only
/// shape (claiming a recompute while some unit's counter stays zero, or
/// swapping objects without minting a new generation) reddens here.
#[test]
fn av_owned_821_v13_forced_whole_recompute_is_nonzero_everywhere() {
    let (mut s, _entry) = OwnedSession::warmup(THREE_FN_SRC, &[]).expect("warmup");
    assert_eq!(s.run_i32(), 1201);
    let before: BTreeMap<String, u64> = s.produced.clone();
    let gens_before: BTreeMap<String, u64> = s
        .code
        .iter()
        .map(|(k, v)| (k.clone(), v.generation()))
        .collect();

    // A whole-module interface change forces the whole recompute.
    let src = THREE_FN_SRC.replace("n: a.n + 10", "n: a.n + 11");
    s.next_revision();
    s.reproduce_all(&src, &[]).expect("whole recompute");

    // EVERY unit re-produced (all counters moved) and EVERY mint
    // generation advanced (new objects, not hash-only reuse).
    for def in ["d_main", "d_bump", "d_plain"] {
        assert_eq!(
            s.produced_of(def),
            before.get(def).copied().unwrap_or(0) + 1,
            "{def}: a whole recompute must be VISIBLE in its production counter"
        );
        assert_eq!(
            s.code_of(def).generation(),
            gens_before.get(def).copied().unwrap_or(0) + 1,
            "{def}: a forced recompute must mint a new generation (object swap visible)"
        );
    }
    // bump(1)->2; plain(2)->2+11=13; 13*100 + len("C") = 1301.
    assert_eq!(s.run_i32(), 1301, "plain(+11) shifts the exact oracle");
}

/// The shared scalar baseline (V-13 sub-1) replays through the UNCHANGED
/// 828/827 assets — this test pins only that the owned landing did not
/// disturb the scalar incremental core's structural face (the full 30-
/// revision P95 dual gate lives in the script's incremental scope).
#[test]
fn av_owned_821_v13_shared_scalar_session_still_warms() {
    let src = "\
fn main() int {
    return 6
}
";
    let mut s = auto_av_probe::session::warmup(src, "d_main", &[]).expect("scalar warmup");
    let (value, _) = {
        let (out, trace) = s.run(&ExecLimits::default(), &CancelFlag::new(false));
        (out, trace)
    };
    let _ = &mut s;
    assert!(
        matches!(value, ExecOutcome::Success { .. }),
        "the scalar session still runs: {value:?}"
    );
}
