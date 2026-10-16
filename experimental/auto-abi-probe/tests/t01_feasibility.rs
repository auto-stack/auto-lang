//! PLAN-797 T-01 feasibility gate: the first three REAL interop legs and
//! the first persisted layout cache hit (plan §8 T-01).
//!
//! Every leg links objects from INDEPENDENT producers into a real PE via
//! the merged ac-core link entries and executes it — exit code 0 is the
//! observable contract, with the expected value baked into the PE from
//! the independent Rust oracle (or computed by the independent C oracle
//! for leg C). A leg that cannot reach exit 0 fails; a negative control
//! proves the comparison discriminates (a wrong expectation exits 1, so
//! a green leg cannot be a vacuous pass).

#![cfg(windows)]

use auto_abi_probe::ctool;
use auto_abi_probe::decl::{self, FROZEN_INPUTS, HAND_COMPUTED_FROZEN_EXPECTED};
use auto_abi_probe::emit;
use auto_abi_probe::layout::{self, REC_MIX};
use auto_abi_probe::link::link_pe;
use auto_abi_probe::run::{self, PeRun};
use ac_artifact_probe::store::ManagedStore;

use std::path::PathBuf;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn cside() -> PathBuf {
    manifest().join("cside")
}

/// Compile one probe C source with the resolved cl.exe into the case dir.
fn compile_c(name: &str, dir: &std::path::Path) -> PathBuf {
    let src = cside().join(name);
    let obj = dir.join(name).with_extension("obj");
    ctool::compile_c(&src, &cside(), &obj)
        .unwrap_or_else(|e| panic!("C compile {name} (independent producer): {e}"));
    obj
}

fn write_obj(dir: &std::path::Path, name: &str, bytes: &[u8]) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, bytes).unwrap_or_else(|e| panic!("write {name}: {e}"));
    p
}

fn expect_exit(pe: PeRun, want: i32, leg: &str) {
    assert_eq!(
        pe.code, want,
        "{leg}: exit {} (want {want}); stdout={:?} stderr={:?}",
        pe.code, pe.stdout, pe.stderr
    );
}

/// The Rust oracle agrees with the hand-computed frozen reference — a
/// broken oracle cannot silently redefine "expected" for every leg below.
#[test]
fn t01_oracle_matches_hand_reference() {
    assert_eq!(decl::oracle_mix_v1(&FROZEN_INPUTS), HAND_COMPUTED_FROZEN_EXPECTED);
    // Permuting two integer inputs changes the result (distinct weights):
    // the fixture is actually distinguishing, not sum-symmetric.
    let swapped = MixInputs {
        b: FROZEN_INPUTS.a,
        a: FROZEN_INPUTS.b,
        ..FROZEN_INPUTS
    };
    assert_ne!(decl::oracle_mix_v1(&swapped), HAND_COMPUTED_FROZEN_EXPECTED);
}

use decl::MixInputs;

/// Leg A: AutoABI caller -> AutoABI callee. Two separately emitted
/// objects (independent ObjectModule instances), one PE, real run.
#[test]
fn t01_leg_a_autoabi_to_autoabi() {
    let decl = emit::t01_mix_decl();
    let expected = decl::oracle_mix_v1(&FROZEN_INPUTS);
    let callee = emit::emit_callee(decl).expect("emit callee obj");
    let caller = emit::emit_caller(decl, &FROZEN_INPUTS, expected).expect("emit caller obj");

    let dir = run::scratch("leg_a");
    let objs = vec![
        write_obj(&dir, "caller.obj", &caller),
        write_obj(&dir, "callee.obj", &callee),
    ];
    let exe = link_pe(&dir, &objs, "p797_entry").expect("link leg A PE");
    let pe = run::run_pe(&exe).expect("run leg A PE");
    expect_exit(pe, 0, "leg A (AutoABI -> AutoABI)");
}

/// Leg A negative control: baking a WRONG expectation into the caller
/// must exit 1 — proving the leg's comparison discriminates and a green
/// leg A is not a vacuous pass.
#[test]
fn t01_leg_a_negative_expected_control() {
    let decl = emit::t01_mix_decl();
    let callee = emit::emit_callee(decl).expect("emit callee obj");
    let caller = emit::emit_caller(decl, &FROZEN_INPUTS, HAND_COMPUTED_FROZEN_EXPECTED + 1)
        .expect("emit caller obj");

    let dir = run::scratch("leg_a_neg");
    let objs = vec![
        write_obj(&dir, "caller.obj", &caller),
        write_obj(&dir, "callee.obj", &callee),
    ];
    let exe = link_pe(&dir, &objs, "p797_entry").expect("link negative-control PE");
    let pe = run::run_pe(&exe).expect("run negative-control PE");
    expect_exit(pe, 1, "leg A negative control (wrong expectation)");
}

/// Leg B: AutoABI caller -> independent C callee (real cl.exe object).
#[test]
fn t01_leg_b_autoabi_calls_c() {
    let decl = emit::t01_mix_decl();
    let expected = decl::oracle_mix_v1(&FROZEN_INPUTS);
    let caller = emit::emit_caller(decl, &FROZEN_INPUTS, expected).expect("emit caller obj");

    let dir = run::scratch("leg_b");
    let c_obj = compile_c("callee_mix.c", &dir);
    let objs = vec![write_obj(&dir, "caller.obj", &caller), c_obj];
    let exe = link_pe(&dir, &objs, "p797_entry").expect("link leg B PE");
    let pe = run::run_pe(&exe).expect("run leg B PE");
    expect_exit(pe, 0, "leg B (AutoABI -> C)");
}

/// Leg C: independent C caller (real cl.exe object with its own oracle
/// and its own entry) -> AutoABI callee.
#[test]
fn t01_leg_c_c_calls_autoabi() {
    let decl = emit::t01_mix_decl();
    let callee = emit::emit_callee(decl).expect("emit callee obj");

    let dir = run::scratch("leg_c");
    let c_obj = compile_c("caller_main.c", &dir);
    let objs = vec![c_obj, write_obj(&dir, "callee.obj", &callee)];
    let exe = link_pe(&dir, &objs, "p797_c_entry").expect("link leg C PE");
    let pe = run::run_pe(&exe).expect("run leg C PE");
    expect_exit(pe, 0, "leg C (C -> AutoABI)");
}

/// First layout cache (plan §5.4: T-01 minimal layout query and the
/// first object are truly persisted): fresh store -> miss -> put ->
/// REOPEN a new store instance on the same root -> hit with byte-equal
/// payload -> same-key determinism. Nothing here treats "file exists" as
/// a pass: the hit is read back and compared.
#[test]
fn t01_layout_cache_first_persist() {
    let root = manifest().join("target").join("autoabi-store-t01");
    let _ = std::fs::remove_dir_all(&root);

    let computed = layout::layout_record(&REC_MIX).expect("layout");
    let key1 = layout::layout_key(&REC_MIX);

    // Fresh store: miss, then persist.
    {
        let store = ManagedStore::open(&root).expect("open store");
        assert!(
            layout::get_layout(&store, &REC_MIX).expect("get on fresh store").is_none(),
            "fresh store must miss"
        );
        let art = layout::put_layout(&store, &REC_MIX, &computed).expect("put layout");
        assert!(!art.is_empty());
    }

    // Reopen (cross-instance persistence, same process): hit, equal.
    {
        let store = ManagedStore::open(&root).expect("reopen store");
        let hit = layout::get_layout(&store, &REC_MIX)
            .expect("get after put")
            .expect("persisted layout must hit after reopen");
        assert_eq!(hit, computed, "hit payload must equal computed layout");
    }

    // Key determinism: same inputs -> same key hash, always.
    let key2 = layout::layout_key(&REC_MIX);
    assert_eq!(key1.hash(), key2.hash());
    // The key carries the frozen identity inputs.
    assert_eq!(key1.component("target"), Some(layout::TARGET));
    assert_eq!(key1.component("abi-profile"), Some(layout::ABI_PROFILE));
    assert_eq!(
        key1.component("sem-ref"),
        Some(auto_abi_probe::semref::snapshot_hash())
    );
}

/// Toolchain identity freezes (recorded in contract-decision.md §5): the
/// fingerprint must resolve, be stable in-process, and carry the pinned
/// flags. On a machine without the C toolchain this fails as an infra
/// blocker — by design.
#[test]
fn t01_ctool_fingerprint_freezes() {
    let fp = ctool::toolchain_fingerprint().expect("C toolchain fingerprint");
    assert!(fp.contains("cl:"));
    assert!(fp.contains(&ctool::CL_FLAGS.join(" ")));
    assert_eq!(fp, ctool::toolchain_fingerprint().unwrap());
    println!("toolchain_fingerprint = {fp}");
}

/// Semantic-ref snapshot stays well-formed and hashable (retirement
/// precondition at D-796 is that its identity is traceable).
#[test]
fn t01_semref_snapshot_identity() {
    auto_abi_probe::semref::verify_snapshot().expect("snapshot well-formed");
    assert_eq!(auto_abi_probe::semref::snapshot_hash().len(), 64);
}
