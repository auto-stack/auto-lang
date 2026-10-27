//! PLAN-821 T-02 first-case tests (`av_owned_821` family).
//!
//! The frozen first case (contract-decision §11): text-record source →
//! 782-checked HIR → owned Typed MIR → AVAD v2 ABC → AV. A record with a
//! NON-EMPTY Str field: read a field view, len-observe it, clone the RHS,
//! replace the field (old buffer freed exactly once), move the whole root
//! out through a move-in helper; the host receives the owned result and
//! releases it — the arena then reports live=0. Oracles are hand-audited
//! ("Auto" = 4 scalars / 4 bytes; n = 4; alloc/free sequences below).

use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{
    ExecOutcome, Value, ABC_OWNED_VERSION, ABC_PROFILE_OWNED, AV_OWNED_PROFILE, MIR_OWNED_SCHEMA,
};
use auto_av_core::mir::*;
use auto_av_core::{abc_verify, mir_verify};

/// The frozen first-case source: every owned operation on one module —
/// record ctor with a Str field, field view, len, deep clone, field
/// replace (RHS via explicit `.move`, 782's owning-RHS rule), scalar
/// field write, whole-root move return, move-in param.
const FIRST_CASE: &str = "\
type Note { body Str; n int }
fn rebody(move u Note) Note {
    let m int = u.body.len()
    let c Str = u.body.clone()
    u.body = c.move
    u.n = m
    return u.move
}
fn main() Note {
    let a Note = Note { body: \"Auto\", n: 1 }
    return rebody(a.move)
}
";

/// The int-returning TWIN for the native cross-check: identical owned
/// operations; only the entry wrapper adapts (the native chain's exit
/// code is an int channel — 782's own native tests use int mains the
/// same way, e.g. `pick(t.move).kind`). Hand-audited expected: 4*100+4.
const FIRST_CASE_INT_TWIN: &str = "\
type Note { body Str; n int }
fn rebody(move u Note) Note {
    let m int = u.body.len()
    let c Str = u.body.clone()
    u.body = c.move
    u.n = m
    return u.move
}
fn main() int {
    let a Note = Note { body: \"Auto\", n: 1 }
    let r Note = rebody(a.move)
    return r.n * 100 + r.body.len()
}
";

fn owned_owner(module: &str) -> OwnerScope {
    OwnerScope {
        schema: MIR_OWNED_SCHEMA.to_string(),
        module: module.to_string(),
    }
}

/// Compile text-record source → owned MIR → AVAD v2 → admitted module.
fn admit_owned_source(src: &str, entry: &str) -> abc_verify::LoadedModule {
    let compiled = auto_ac_prototype::source::compile_text_record_source(src, "p.at", &[])
        .expect("text-record source compiles");
    assert_eq!(
        compiled.checked.domain(),
        auto_ac_prototype::hir::HirDomain::TextRecord
    );
    let mir = auto_av_probe::hir_adapter::lower_checked_owned(&compiled.checked, entry, 0x821)
        .expect("owned lowering");
    assert_eq!(mir.profile, AV_OWNED_PROFILE);
    mir_verify::verify_mir(&mir).expect("owned MIR verifies");
    let wire = encode_abc(&mir).expect("AVAD v2 encode");
    let flat = wire.to_flat();
    assert_eq!(&flat[0..4], b"AVAD", "AVAD magic on the wire");
    let decoded = decode_abc(&flat).expect("decode");
    assert_eq!(decoded.version, ABC_OWNED_VERSION);
    assert_eq!(decoded.profile, ABC_PROFILE_OWNED);
    abc_verify::verify_abc(decoded, &mir.owner).expect("AVAD v2 admits")
}

/// The record payload the first case must return (hand-audited): a clone
/// of "Auto" in `body` and the view's scalar length in `n`.
fn expect_note(value: &Value) {
    let Value::Record(r) = value else {
        panic!("first case must return a record, got {value:?}")
    };
    assert_eq!(r.fields.len(), 2);
    match &r.fields[0] {
        Value::Str(h) => assert_eq!(h.bytes(), b"Auto"),
        other => panic!("body must be an owned Str, got {other:?}"),
    }
    match &r.fields[1] {
        Value::I32(n) => assert_eq!(*n, 4, "len(\"Auto\") = 4 scalars"),
        other => panic!("n must be i32, got {other:?}"),
    }
}

#[test]
fn av_owned_821_first_case_whole_path_and_live_zero() {
    let loaded = admit_owned_source(FIRST_CASE, "d_main");
    assert!(loaded.executable(), "complete owned module is executable");
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Success { value, .. } = out else {
        panic!("first case must succeed, got {out:?}")
    };
    expect_note(&value);

    // Host receives the owned result — the SINGLE release responsibility:
    // while held, the record's clone buffer is live; an explicit drop is
    // the release, after which the arena reports live=0 (no reap_all).
    let arena = trace
        .arena
        .as_ref()
        .expect("owned execution carries the arena");
    assert!(
        arena.live() >= 1,
        "the received record holds its clone buffer"
    );
    let allocs = arena.alloc_events();
    assert_eq!(
        allocs.len(),
        2,
        "exactly two owned allocations: ctor buffer + clone"
    );
    assert_eq!(allocs[0].bytes, 4, "ctor materializes the \"Auto\" literal");
    assert_eq!(allocs[1].bytes, 4, "clone re-buffers independently");
    drop(value);
    assert_eq!(arena.live(), 0, "host release empties the ledger");
    // Free order oracle: the ctor buffer frees at the field replace; the
    // clone frees at the host release (creation-order LIFO).
    let frees = arena.free_events();
    assert_eq!(frees.len(), 2);
    assert_eq!(
        frees[0].id, allocs[0].id,
        "old buffer freed exactly once at replace"
    );
    assert_eq!(frees[1].id, allocs[1].id, "clone freed at the host release");
}

#[test]
fn av_owned_821_first_case_wire_round_trip() {
    let loaded = admit_owned_source(FIRST_CASE, "d_main");
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Success { value, .. } = out else {
        panic!("decoded first case must succeed, got {out:?}")
    };
    expect_note(&value);
    drop(value);
    assert_eq!(trace.arena.as_ref().unwrap().live(), 0);
}

/// V-08 first-case slice: same-store NEW-WORKER restore re-verifies and
/// reproduces the identical result (request_owned persists the AVAD v2
/// module under the owned layer; restore_owned re-runs envelope + decode
/// + the full deep admission before executing).
#[test]
fn av_owned_821_first_case_same_store_restore() {
    let dir = std::env::temp_dir().join(format!(
        "av821-restore-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let outcome = auto_av_probe::owned_driver::request_owned(
        Some(&dir),
        "av-owned-821-first-case",
        FIRST_CASE,
        "d_main",
        &[],
    )
    .expect("owned request");
    let auto_av_core::contract::ExecOutcome::Success { .. } = outcome.av.clone().unwrap() else {
        panic!("request must succeed: {:?}", outcome.av)
    };
    // The persisted artifact's key axes come from the driver itself.
    let restored = auto_av_probe::owned_driver::restore_owned(
        &dir,
        &outcome.owner,
        &outcome.entry,
        outcome.snapshot,
        &{
            let compiled =
                auto_ac_prototype::source::compile_text_record_source(FIRST_CASE, "p.at", &[])
                    .expect("compile");
            let mir =
                auto_av_probe::hir_adapter::lower_checked_owned(&compiled.checked, "d_main", 0x821)
                    .expect("lower");
            auto_av_probe::owned_driver::types_fp_of(&mir)
        },
    )
    .expect("restore re-verifies and executes");
    let ExecOutcome::Success { value, .. } = &restored else {
        panic!("restore must succeed, got {restored:?}")
    };
    expect_note(value);
    let _ = std::fs::remove_dir_all(&dir);
}

/// V-01/V-02 negative controls (typed fixtures, the MIR layer's mirror of
/// the source-domain `verify.*` gates):
///   * a field write through a MOVED root — 782 "字段写永不复活已移出的根"
///     (`mir.use-after-move`);
///   * a second MOVE of an already-moved source (`mir.double-move`);
///   * a self-move (`mir.reservation-conflict`, the overlap family).
/// NOTE: the "value-def-is-MOVE-of-the-same-root" field-set shape is NOT
/// MIR-typable for Str fields (acyclic record types make root==field-type
/// impossible) — the reservation code's reachable MIR form is the
/// self-move; source-shaped reservation conflicts stay with the source
/// domain's `verify.*` codes (recorded in the plan evidence).
#[test]
fn av_owned_821_first_case_move_negative_controls() {
    use auto_av_core::mir::{MirBlock, MirInstr, MirSlot, Terminator};
    fn neg_module() -> MirModule {
        let mut m = MirModule::new_owned(owned_owner("neg"));
        let t_str = m.intern_type(MirType::Str);
        let t_rec = m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "neg".into(),
            name: "Note".into(),
            fields: vec![MirFieldDef {
                name: "body".into(),
                ty: t_str,
            }],
        }));
        let t_fn = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: t_rec,
        }));
        let c_str = m.intern_const(MirConst::Str(Box::from(*b"Auto")));
        m.funcs.push(MirFunction {
            owner: m.owner.clone(),
            name: "d_main".into(),
            sig: t_fn,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Local,
                    ty: t_rec,
                    name: Some("u".into()),
                },
                MirSlot {
                    kind: SlotKind::Temp,
                    ty: t_str,
                    name: None,
                },
                MirSlot {
                    kind: SlotKind::Temp,
                    ty: t_rec,
                    name: None,
                },
            ],
            blocks: vec![MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.entry = 0;
        let _ = c_str;
        m
    }
    let ctor = |m: &mut MirModule, _c: u32| {
        // Type-table layout of neg_module: 0 = Str, 1 = Note(record), 2 = fn sig.
        let c_str = m.intern_const(MirConst::Str(Box::from(*b"Auto")));
        vec![
            MirInstr::Const {
                dst: 1,
                value: c_str,
            },
            MirInstr::Aggregate {
                dst: 0,
                record_ty: 1,
                operands: vec![1],
            },
        ]
    };
    // (a) field write through a moved root.
    let mut m = neg_module();
    let c = m.intern_const(MirConst::Str(Box::from(*b"Auto")));
    m.funcs[0].blocks[0].instrs = {
        let mut v = ctor(&mut m, c);
        v.push(MirInstr::Move { dst: 2, src: 0 });
        v.push(MirInstr::FieldSet {
            root: 0,
            path: vec![0],
            value: 1,
        });
        v
    };
    let err = mir_verify::verify_mir(&m).expect_err("dead-root field write rejected");
    assert!(
        err.code().contains("use-after-move"),
        "expected use-after-move, got {:?}",
        err.code()
    );
    // (b) double move.
    let mut m = neg_module();
    let c = m.intern_const(MirConst::Str(Box::from(*b"Auto")));
    m.funcs[0].blocks[0].instrs = {
        let mut v = ctor(&mut m, c);
        v.push(MirInstr::Move { dst: 2, src: 0 });
        v.push(MirInstr::Move { dst: 2, src: 0 });
        v
    };
    let err = mir_verify::verify_mir(&m).expect_err("double move rejected");
    assert!(
        err.code().contains("double-move") || err.code().contains("use-after-move"),
        "expected a double-move family rejection, got {:?}",
        err.code()
    );
    // (c) self-move (the overlap/reservation family's reachable form).
    let mut m = neg_module();
    let c = m.intern_const(MirConst::Str(Box::from(*b"Auto")));
    m.funcs[0].blocks[0].instrs = {
        let mut v = ctor(&mut m, c);
        v.push(MirInstr::Move { dst: 0, src: 0 });
        v
    };
    let err = mir_verify::verify_mir(&m).expect_err("self-move rejected");
    assert!(
        err.code().contains("reservation-conflict"),
        "expected the reservation discrimination, got {:?}",
        err.code()
    );
}

/// The INDEPENDENT native oracle (§5.2): the int TWIN through the 782
/// HIR-native chain must agree with the hand-audited 404 (= len*100+len).
#[test]
fn av_owned_821_first_case_native_independent_comparison() {
    let workdir = std::env::temp_dir().join(format!(
        "av821-native-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let run = auto_av_probe::native_reference::build_and_run_text_record(
        FIRST_CASE_INT_TWIN,
        &[],
        "d_main",
        &workdir,
        None,
    )
    .expect("the 782 native path builds and runs the text-record twin");
    assert_eq!(
        run.exit_code, 404,
        "native independent oracle: hand-audited 404, stderr: {}",
        run.stderr
    );
    let _ = std::fs::remove_dir_all(&workdir);
}
