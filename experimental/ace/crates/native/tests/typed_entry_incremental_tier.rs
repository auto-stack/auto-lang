//! PLAN-891 typed resident-session tier (V-07/V-06): warm single-unit
//! body edits with per-production-point honesty — the edited unit (and,
//! on a signature change, its real dependents) re-verifies/re-emits while
//! callers and disjoint units produce NOTHING; A→B→A revisits the actual
//! historical object; param-only reruns produce neither units nor the
//! wrapper while the machine result follows the new arguments; a cold
//! restart re-admits from the store with zero re-emission. No structural
//! whole-module fallback ever runs.

use ace_artifacts::store::ManagedStore;
use ace_native::typed_entry::{self, TypedArg, TypedOutcome};
use ace_native::typed_fixture::{self, FixtureDoc};
use ace_native::typed_session::TypedSession;

fn tools_available() -> bool {
    ace_core::link::find_rust_lld().is_ok() && ace_core::link::find_sdk_um_dir().is_ok()
}

fn workdir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mn891i-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("workdir");
    dir
}

fn doc_from(json: &str) -> FixtureDoc {
    typed_fixture::parse_fixture(json.as_bytes()).expect("fixture decodes")
}

fn arg(kind: &str, bits: &str) -> TypedArg {
    typed_entry::parse_arg(
        typed_entry::TypedKind::from_label(kind).expect("kind"),
        bits,
    )
    .expect("arg")
}

/// caller passes through helper(a); helper identity(a); disjoint returns
/// a constant (wide arithmetic is correctly outside the data profile, so
/// the observable differences ride on identity-vs-constant bodies).
const BASE: &str = r#"{"schema":"mn-typed-fixture/1","module":"m891_sess","units":[
  {"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"a"}],"ret":"t0"},
  {"name":"disjoint","params":[],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"0000000000000001"}],"ret":"t0"},
  {"name":"main","params":[["a","i64"]],"result":"i64","body":[
     {"op":"call","dst":"t0","callee":"helper","args":["a"]}],
   "ret":"t0"}],
 "entry":"main"}"#;

fn unit_decl(json: &str) -> typed_fixture::UnitDecl {
    // A whole-document form keeps the strict decoder; lift the unit out.
    let wrapped = format!(
        r#"{{"schema":"mn-typed-fixture/1","module":"m891_sess","units":[{json}],"entry":"main"}}"#
    );
    let doc = typed_fixture::parse_fixture(wrapped.as_bytes()).expect("unit decodes");
    doc.units.into_iter().next().expect("one unit")
}

fn value_bits(run: &ace_native::typed_pipeline::TypedRun) -> String {
    match &run.outcome {
        TypedOutcome::Value { kind_code, bits } => typed_entry::canonical_hex(*kind_code, *bits),
        other => panic!("expected a value outcome, got {other:?}"),
    }
}

#[test]
#[cfg(windows)]
fn typed_session_warm_edit_hits_history_and_param_only_production_zero() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed session tier skipped");
        return;
    }
    let root = workdir("store");
    let store = ManagedStore::open(&root).expect("store");
    let dir = workdir("run");
    let mut s = TypedSession::start(
        doc_from(BASE),
        &dir,
        Some(store),
        ace_native::typed_pipeline::TypedCacheMode::Reuse,
    )
    .expect("session starts");

    // Baseline run: helper identity with a = 0x10.
    let run = s
        .run(&[arg("i64", "0000000000000010")], 1, None)
        .expect("baseline run");
    assert_eq!(value_bits(&run), "0000000000000010", "helper identity");
    assert_eq!(s.counters().whole_fallback, 0);

    // ---- warm body edit: helper returns a constant instead ------------
    let helper_b = unit_decl(
        r#"{"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"00000000000000ff"}],"ret":"t0"}"#,
    );
    let obs = s.edit_unit(helper_b).expect("edit B");
    assert_eq!(obs.edited_unit, "helper");
    assert_eq!(
        obs.reprepared_units,
        vec!["helper"],
        "only the edited unit re-prepares"
    );
    assert_eq!(obs.verify_mir, 1, "one single-unit verify");
    assert_eq!(obs.emit_units, 1, "the new fingerprint emits once");
    assert_eq!(obs.whole_fallback, 0, "no whole-module fallback");
    let run = s
        .run(&[arg("i64", "0000000000000010")], 2, None)
        .expect("run after edit B");
    assert_eq!(
        value_bits(&run),
        "00000000000000ff",
        "helper now returns the 0xff constant"
    );
    assert_eq!(s.counters().whole_fallback, 0);

    // ---- param-only rerun: NO unit or wrapper production --------------
    let before = s.counters().clone();
    let run = s
        .run(&[arg("i64", "0000000000000020")], 3, None)
        .expect("param-only rerun");
    assert_eq!(
        value_bits(&run),
        "00000000000000ff",
        "same constant body ignores the argument"
    );
    assert_eq!(
        s.counters().emit_units,
        before.emit_units,
        "param-only emits nothing"
    );
    assert_eq!(
        s.counters().wrapper_builds,
        before.wrapper_builds,
        "param-only builds no wrapper"
    );
    assert_eq!(
        s.counters().run_invocations,
        before.run_invocations + 1,
        "it still really runs"
    );

    // ---- A→B→A: the historical object is REUSED, not re-minted --------
    let helper_a = unit_decl(
        r#"{"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"a"}],"ret":"t0"}"#,
    );
    let obs = s.edit_unit(helper_a).expect("edit back to A");
    assert_eq!(obs.emit_units, 0, "A revisit emits nothing");
    assert_eq!(obs.cache_hits, 1, "the historical object is the hit");
    assert_eq!(obs.reprepared_units, vec!["helper"]);
    let run = s
        .run(&[arg("i64", "0000000000000010")], 4, None)
        .expect("run after edit back");
    assert_eq!(
        value_bits(&run),
        "0000000000000010",
        "helper identity is back"
    );
    assert_eq!(s.counters().whole_fallback, 0);
    s.close();
}

#[test]
#[cfg(windows)]
fn typed_session_signature_change_reprepares_real_dependents_only() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed session tier skipped");
        return;
    }
    let dir = workdir("sig");
    let mut s = TypedSession::start(
        doc_from(BASE),
        &dir,
        None,
        ace_native::typed_pipeline::TypedCacheMode::Reuse,
    )
    .expect("session starts");

    // helper gains a SECOND parameter: its signature changes, so every
    // real dependent (main) re-prepares; disjoint stays untouched.
    let helper_wide = unit_decl(
        r#"{"name":"helper","params":[["a","i64"],["b","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"b"}],"ret":"t0"}"#,
    );
    let main_wide = unit_decl(
        r#"{"name":"main","params":[["a","i64"]],"result":"i64","body":[
            {"op":"const","dst":"t9","kind":"i64","bits":"0000000000000002"},
            {"op":"call","dst":"t0","callee":"helper","args":["a","t9"]}],
          "ret":"t0"}"#,
    );
    let obs = s
        .edit_units(vec![helper_wide.clone(), main_wide.clone()])
        .expect("atomic helper+main signature change");
    assert!(
        obs.reprepared_units.iter().any(|u| u == "helper"),
        "the edited unit re-prepares"
    );
    // main still calls helper(a) — its import table is now stale, so the
    // session must re-prepare it BEFORE any link (the fixture-level
    // declaration edit below is the sanctioned way a caller follows its
    // callee). Re-preparing main with the OLD body would fail the import
    // closure — assert that honestly by re-preparing only after the edit.
    assert!(
        obs.reprepared_units.iter().any(|u| u == "main"),
        "the caller re-prepares in the same change set"
    );
    let obs2 = obs.clone();
    assert!(
        !obs.reprepared_units.iter().any(|u| u == "disjoint")
            && !obs2.reprepared_units.iter().any(|u| u == "disjoint"),
        "disjoint units produce nothing"
    );
    let run = s
        .run(&[arg("i64", "0000000000000010")], 5, None)
        .expect("run after signature change");
    assert_eq!(
        value_bits(&run),
        "0000000000000002",
        "helper now returns its second parameter"
    );
    assert_eq!(s.counters().whole_fallback, 0);
}

#[test]
#[cfg(windows)]
fn typed_session_cold_restart_readmits_from_store_without_remission() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed session tier skipped");
        return;
    }
    let root = workdir("restart-store");
    let store = ManagedStore::open(&root).expect("store");
    let dir = workdir("restart-run");
    {
        let mut s = TypedSession::start(
            doc_from(BASE),
            &dir,
            Some(store),
            ace_native::typed_pipeline::TypedCacheMode::Fresh,
        )
        .expect("first session");
        let run = s
            .run(&[arg("i64", "0000000000000010")], 6, None)
            .expect("first run");
        assert_eq!(value_bits(&run), "0000000000000010");
        s.close();
    }
    // COLD restart: a brand-new session on the SAME store re-admits every
    // unit from its stored bytes — zero emission, the PE still runs.
    let store2 = ManagedStore::open(&root).expect("store reopens");
    let mut s2 = TypedSession::restart(doc_from(BASE), &dir, store2).expect("restart");
    assert_eq!(s2.counters().emit_units, 0, "cold restart emits nothing");
    assert_eq!(
        s2.counters().cache_hits,
        3,
        "all three units re-admitted from the store"
    );
    let run = s2
        .run(&[arg("i64", "0000000000000020")], 7, None)
        .expect("restart run");
    assert_eq!(
        value_bits(&run),
        "0000000000000020",
        "helper identity from cold bytes"
    );
    assert_eq!(s2.counters().emit_units, 0, "still nothing re-emitted");
    assert_eq!(
        s2.counters().wrapper_builds,
        0,
        "wrapper restored, not rebuilt"
    );
    s2.close();
}

// ==== PLAN-1032 F24: typed batch-edit atomicity ==========================
//
// A multi-unit change set (a callee edit plus caller updates, or unrelated
// units submitted together) is ONE acceptance transaction. The oracle here is
// PUBLIC state only: resident (fingerprint, generation, object hash), the
// entry plan, and an actual run of the previous closure. A failure anywhere in
// the batch must leave all of it identical and the old entry executable; the
// repaired batch then installs the whole set. (Document/history/generation
// internals are asserted by the in-module oracle in typed_session.rs — no
// production accessor was added for them.)

const ATOMIC_HELPER_B: &str = r#"{"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"00000000000000ff"}],"ret":"t0"}"#;
const ATOMIC_HELPER_WIDE: &str = r#"{"name":"helper","params":[["a","i64"],["b","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"b"}],"ret":"t0"}"#;
const ATOMIC_DISJOINT_BAD: &str = r#"{"name":"disjoint","params":[],"result":"i64","body":[],"ret":"missing"}"#;
const ATOMIC_DISJOINT_GOOD: &str = r#"{"name":"disjoint","params":[],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"0000000000000003"}],"ret":"t0"}"#;
const ATOMIC_DISJOINT_LATER: &str = r#"{"name":"disjoint","params":[],"result":"i64","body":[{"op":"const","dst":"t0","kind":"i64","bits":"0000000000000007"}],"ret":"t0"}"#;

fn residents_snapshot(
    session: &TypedSession,
) -> Vec<(String, String, u64, String)> {
    session
        .residents()
        .map(|r| {
            (
                r.name.clone(),
                r.fp.clone(),
                r.generation,
                ace_native::cache::content_hash(&r.coff),
            )
        })
        .collect()
}

#[test]
#[cfg(windows)]
fn audit1032_ntx_01_failed_batch_keeps_previous_entry_runnable() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — PLAN-1032 NTX-01 skipped");
        return;
    }
    let dir = workdir("atomic");
    let mut session = TypedSession::start(
        doc_from(BASE),
        &dir,
        None,
        ace_native::typed_pipeline::TypedCacheMode::Reuse,
    )
    .expect("session starts");

    let run = session
        .run(&[arg("i64", "0000000000000010")], 11, None)
        .expect("baseline run");
    assert_eq!(value_bits(&run), "0000000000000010", "helper identity");
    let residents = residents_snapshot(&session);
    let plan = session.entry_plan().expect("entry plan");
    assert_eq!(session.counters().whole_fallback, 0);

    // (1) A single invalid replacement: rejected, nothing installed.
    let err = session
        .edit_unit(unit_decl(ATOMIC_DISJOINT_BAD))
        .expect_err("an invalid replacement must be rejected");
    assert!(
        matches!(err, ace_native::MnError::Unsupported(..)),
        "unexpected error class: {}",
        err.code()
    );
    assert_eq!(residents_snapshot(&session), residents, "no resident moved");
    assert_eq!(
        session.entry_plan().expect("entry plan"),
        plan,
        "the entry plan is untouched"
    );

    // (2) Valid head, LATE verify failure in the tail: the head must not be
    // installed by its own success.
    let err = session
        .edit_units(vec![
            unit_decl(ATOMIC_HELPER_B),
            unit_decl(ATOMIC_DISJOINT_BAD),
        ])
        .expect_err("the tail failure must reject the whole change set");
    assert!(matches!(err, ace_native::MnError::Unsupported(..)));
    assert_eq!(
        residents_snapshot(&session),
        residents,
        "the already-prepared head must not be installed"
    );
    assert_eq!(session.entry_plan().expect("entry plan"), plan);

    // (3) BAD-FIRST neighbor: same rejection, so (2) is not an ordering
    // artifact of the fixture decoder.
    let err = session
        .edit_units(vec![
            unit_decl(ATOMIC_DISJOINT_BAD),
            unit_decl(ATOMIC_HELPER_B),
        ])
        .expect_err("a failing head must reject the whole change set");
    assert!(matches!(err, ace_native::MnError::Unsupported(..)));
    assert_eq!(residents_snapshot(&session), residents);

    // (4) Signature change whose real dependent is stale: the callee's own
    // success must not install it while the caller fails verify.
    let err = session
        .edit_unit(unit_decl(ATOMIC_HELPER_WIDE))
        .expect_err("the stale caller must reject the change set");
    assert!(matches!(err, ace_native::MnError::Unsupported(..)));
    assert_eq!(
        residents_snapshot(&session),
        residents,
        "the widened callee must not be installed without its caller"
    );

    // The previously accepted closure still links AND runs.
    let run = session
        .run(&[arg("i64", "0000000000000010")], 12, None)
        .expect("the old accepted closure still runs");
    assert_eq!(
        value_bits(&run),
        "0000000000000010",
        "the pre-failure entry plan and objects are still the accepted ones"
    );
    assert_eq!(session.counters().whole_fallback, 0);

    // (5) Repair the defect and retry the SAME change set: it installs and
    // the batch result is the helper constant.
    let obs = session
        .edit_units(vec![
            unit_decl(ATOMIC_HELPER_B),
            unit_decl(ATOMIC_DISJOINT_GOOD),
        ])
        .expect("the repaired change set installs");
    assert_eq!(
        obs.reprepared_units,
        vec!["helper".to_string(), "disjoint".to_string()],
        "both replacements re-prepare in change-set order"
    );
    assert_eq!(obs.emit_units, 2, "two genuine new identities emit once");
    assert_eq!(obs.whole_fallback, 0);
    let run = session
        .run(&[arg("i64", "0000000000000010")], 13, None)
        .expect("run the repaired batch");
    assert_eq!(
        value_bits(&run),
        "00000000000000ff",
        "the repaired batch is the new accepted result"
    );

    // (6) Legitimate single-unit neighbor still produces exactly one object
    // (the atomic path did not freeze normal edits).
    let obs = session
        .edit_unit(unit_decl(ATOMIC_DISJOINT_LATER))
        .expect("a legitimate single edit still installs");
    assert_eq!(obs.reprepared_units, vec!["disjoint".to_string()]);
    assert_eq!(obs.emit_units, 1, "one new disjoint identity emits");
    assert_eq!(obs.cache_hits, 0);
    session.close();
}