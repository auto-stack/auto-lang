//! PLAN-820 T-09: V-13 — the data profile's function-incremental
//! compatibility over the 828 public interfaces (`av_data_820` family,
//! part 5). Four subcases (contract-decision §12), each bound to
//! producer-observed facts and the session's structural gate:
//! 1. shared D828 scalar baseline — the function_incremental_828 suite in
//!    this same tree (its own family; not re-run here);
//! 2. a REAL data-program body edit: every stage (parse / recheck resolve
//!    + typecheck / emit / bind / lower / admit) touches exactly the
//!    edited function — producer-attested by the body_work window and the
//!    emission receipt — and every other function's HIR and code identity
//!    is unchanged while the semantics follow the edit;
//! 3. dependency / layout / signature / authorization changes: the rebuild
//!    set is computed from actual dependency facts (record mentioners
//!    closed over nesting, call edges), non-dependents keep identity, a
//!    cross-revision signature mismatch is a controlled LINK rejection
//!    with the previous snapshot alive, and a caps delta re-admits every
//!    unit without touching HIR identity;
//! 4. the forced-degradation counter-controls: a whole-frontend rebuild
//!    AND an equal-content object replacement ("only the hash kept") BOTH
//!    redden the same structural gate the real edit passes — if the edit
//!    path ever regresses to whole-frontend production, the in-path gate
//!    reddens the edit itself and this family (the formal script's
//!    incremental tier) goes non-zero.

use auto_av_core::contract::Value;
use auto_av_probe::data_session::DataIncrementalSession;

const SRC: &str = "\
type I { x int }
type O { inner I }
fn bump(o O) O {
    var c = o
    c.inner.x = c.inner.x + 1
    return c
}
fn main() int {
    var o = O { inner: I { x: 41 } }
    let b = bump(o)
    return b.inner.x
}
fn untouched() int {
    return 7
}
";

#[test]
fn av_data_820_v13_body_edit_is_function_level() {
    // Subcase 2: edit bump's body (+1 → +2) — bump's HIR/code identity
    // changes, main's and untouched's are IDENTICAL (the same Arc objects,
    // the same mint generations), and the producer window proves no other
    // body was walked at any stage.
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    assert!(matches!(s.run(), Ok(Value::I32(42))), "cold run 41+1=42");
    // Cold evidence: the warmup's own producer window walked every body.
    assert_eq!(
        s.cold_body_work().len(),
        3,
        "the cold pass walks all three bodies"
    );
    let before_main = s.identity_of("d_main").expect("main resident");
    let before_untouched = s.identity_of("d_untouched").expect("untouched resident");
    let before_bump = s.identity_of("d_bump").expect("bump resident");
    let before_hir_main = s.hir_identity_of("d_main").expect("main hir");
    let before_hir_untouched = s.hir_identity_of("d_untouched").expect("untouched hir");
    let before_hir_bump = s.hir_identity_of("d_bump").expect("bump hir");
    let counters_before: Vec<(String, u64)> =
        s.counters().iter().map(|(k, v)| (k.clone(), *v)).collect();

    let edited = SRC.replace("+ 1", "+ 2");
    let report = s.edit_body(&edited, "d_bump").expect("body edit");
    assert_eq!(report.changed_def, "d_bump");
    // Parse scope: the edit's parse document held exactly ONE fn item.
    assert_eq!(
        report.facts.parse_fn_items, 1,
        "the masked parse document contains only the edited function"
    );
    // Producer window: only bump's body was walked (resolve/typecheck/
    // plan/bind) — main and untouched are ABSENT, not zero-valued.
    assert_eq!(
        report.facts.body_work.len(),
        1,
        "body work must name exactly the edited function: {:?}",
        report.facts.body_work
    );
    let bump_work = report.facts.body_work.get("bump").expect("bump body work");
    assert!(
        bump_work.resolve >= 1
            && bump_work.typecheck >= 1
            && bump_work.plan >= 1
            && bump_work.bind >= 1,
        "all four producer axes walked the changed body: {bump_work:?}"
    );
    // Emission receipt: 1 real body rendered, n−1 body-less scaffolds.
    assert_eq!(report.facts.receipt.real_body_plans, 1);
    assert_eq!(report.facts.receipt.scaffold_fns, 2);
    // Edit-path IO: zero.
    assert!(
        report.facts.io.values().all(|v| *v == 0),
        "edit-path IO must be zero: {:?}",
        report.facts.io
    );
    // HIR identity: bump replaced; main/untouched the SAME Arc objects.
    assert_eq!(
        s.hir_identity_of("d_main"),
        Some(before_hir_main),
        "unchanged function's HIR object is reused as-is"
    );
    assert_eq!(
        s.hir_identity_of("d_untouched"),
        Some(before_hir_untouched),
        "non-dependent function's HIR object is reused as-is"
    );
    assert_ne!(
        s.hir_identity_of("d_bump"),
        Some(before_hir_bump),
        "the edited function's HIR object is replaced"
    );
    // Code identity: main/untouched byte-identical; bump's generation +1.
    assert_eq!(
        s.identity_of("d_main"),
        Some(before_main),
        "unchanged function's code object/generation is reused as-is"
    );
    assert_eq!(
        s.identity_of("d_untouched"),
        Some(before_untouched),
        "non-dependent function's code object/generation is reused as-is"
    );
    assert_eq!(
        s.identity_of("d_bump").map(|c| c.generation),
        Some(before_bump.generation + 1),
        "bump's code generation advanced by exactly one"
    );
    // Structural axis: only d_bump's lower counter moved.
    for (k, v) in counters_before {
        let want = if k == "lower:d_bump" { v + 1 } else { v };
        assert_eq!(s.counters().get(&k), Some(&want), "counter axis for {k}");
    }
    // Semantics: 41 + 2 = 43.
    assert!(
        matches!(s.run(), Ok(Value::I32(43))),
        "edited body executes"
    );
}

#[test]
fn av_data_820_v13_dependency_changes_invalidate_precisely() {
    // Subcase 3a (top-level layout): adding a field to O changes the type
    // table AND bump's signature content (fn(O)→O embeds O's shape). The
    // units each import every def with its signature, so a
    // signature-content change re-lowers EVERY unit — including
    // `untouched`, whose own body never mentions O (its import table
    // embeds bump's signature; the snapshot link's interning gate owns
    // exactly this dependency). LAYER precision still holds: untouched's
    // HIR BODY is unchanged, so its resident HIR Arc survives.
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    let untouched_code = s.identity_of("d_untouched").expect("untouched resident");
    let untouched_hir = s.hir_identity_of("d_untouched").expect("untouched hir");
    let typed = SRC
        .replace("type O { inner I }", "type O { inner I\n    extra int }")
        .replace(
            "var o = O { inner: I { x: 41 } }",
            "var o = O { inner: I { x: 41 }, extra: 0 }",
        )
        .replace(
            "    var c = o\n    c.inner.x = c.inner.x + 1\n    return c",
            "    var c = o\n    c.inner.x = c.inner.x + 1\n    c.extra = 7\n    return c",
        );
    let rebuilt = s.apply_source(&typed).expect("type-change edit links");
    let mut rebuilt_sorted = rebuilt.clone();
    rebuilt_sorted.sort();
    assert_eq!(
        rebuilt_sorted,
        vec![
            "d_bump".to_string(),
            "d_main".to_string(),
            "d_untouched".to_string()
        ],
        "a signature-content change re-lowers every importing unit"
    );
    assert!(matches!(s.run(), Ok(Value::I32(42))), "still 41+1=42");
    assert_ne!(
        s.identity_of("d_untouched"),
        Some(untouched_code),
        "untouched's unit re-admits (its import table embeds bump's new signature)"
    );
    assert_eq!(
        s.hir_identity_of("d_untouched"),
        Some(untouched_hir),
        "untouched's HIR BODY is unchanged — its resident HIR Arc survives a layout change"
    );

    // Subcase 3a-negative (no-change control): re-applying the SAME source
    // computes an EMPTY affected set — zero rebuilds.
    let rebuilt = s.apply_source(&typed).expect("same source applies");
    assert!(
        rebuilt.is_empty(),
        "an unchanged source invalidates nothing: {rebuilt:?}"
    );

    // Subcase 3b (nested layout): renaming I's field changes the INNERMOST
    // record; O contains I, so bump's signature shape changes through O
    // and every unit re-lowers — untouched keeps its HIR Arc again.
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    let untouched_hir = s.hir_identity_of("d_untouched").expect("untouched hir");
    let nested = SRC
        .replace("type I { x int }", "type I { y int }")
        .replace("c.inner.x = c.inner.x + 1", "c.inner.y = c.inner.y + 1")
        .replace("I { x: 41 }", "I { y: 41 }")
        .replace("return b.inner.x", "return b.inner.y");
    let rebuilt = s.apply_source(&nested).expect("nested layout edit links");
    let mut rebuilt_sorted = rebuilt.clone();
    rebuilt_sorted.sort();
    assert_eq!(
        rebuilt_sorted,
        vec![
            "d_bump".to_string(),
            "d_main".to_string(),
            "d_untouched".to_string()
        ],
        "nesting closure widens I's change through O's signature to every unit"
    );
    assert_eq!(
        s.hir_identity_of("d_untouched"),
        Some(untouched_hir),
        "untouched's HIR identity survives a nested layout change"
    );
    assert!(matches!(s.run(), Ok(Value::I32(42))));

    // Subcase 3c (rejection control): a CROSS-REVISION signature change.
    // The rev2 source is fully self-consistent (bump takes I and returns
    // it; main adapts) so the whole-source front door accepts it — but
    // only d_bump is rebuilt via the explicit partial control: the
    // RESIDENT d_main (rev1, calling bump(O)) now mismatches at the LINK.
    // The previous snapshot survives and stays executable.
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    let broken = SRC
        .replace(
            "fn bump(o O) O {\n    var c = o\n    c.inner.x = c.inner.x + 1\n    return c\n}",
            "fn bump(o I) I {\n    var c = o\n    c.x = c.x + 1\n    return c\n}",
        )
        .replace("return b.inner.x", "return b.x")
        .replace("let b = bump(o)", "let b = bump(o.inner)");
    let err = s
        .apply_source_partial(&broken, &["d_bump"])
        .expect_err("signature break must reject at the link");
    assert!(err.contains("link rejected"), "got {err}");
    assert!(
        matches!(s.run(), Ok(Value::I32(42))),
        "the resident snapshot survives the rejected edit"
    );

    // Subcase 3d (authorization): capabilities embed in every av.data/1
    // unit — a caps delta re-admits ALL units while HIR identity stays
    // put; an unchanged caps list produces zero work.
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    let hir_before = s.hir_identity_all();
    let rebuilt = s
        .apply_caps(&["hir.test.trace.v1".to_string()])
        .expect("caps delta re-admits");
    assert_eq!(rebuilt.len(), 3, "every unit re-lowers on a caps delta");
    assert_eq!(
        s.hir_identity_all(),
        hir_before,
        "an authorization change never touches HIR identity"
    );
    assert!(matches!(s.run(), Ok(Value::I32(42))));
    let again = s
        .apply_caps(&["hir.test.trace.v1".to_string()])
        .expect("no-delta apply");
    assert!(
        again.is_empty(),
        "an unchanged caps list produces zero work"
    );
}

#[test]
fn av_data_820_v13_forced_degradations_redden_the_gate() {
    // Subcase 4: the counter-controls. The session's structural gate runs
    // INSIDE edit_body — a degraded implementation reddens the real edit.
    // These controls prove the gate is not blind by driving deliberate
    // degradations through it.
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    let edited = SRC.replace("+ 1", "+ 2");

    // Control A — the whole-frontend degradation (the F820-R1-01 shape:
    // compile_record_source per edit, every body re-walked, fresh objects).
    let violations = s
        .forced_whole_frontend_edit(&edited, "d_bump")
        .expect("control runs");
    assert!(
        !violations.is_empty(),
        "the gate must redden a whole-frontend edit"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.contains("body work touched unchanged function")),
        "the body-work axis reddens: {violations:?}"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.contains("identity moved for unchanged")),
        "the identity axis reddens: {violations:?}"
    );

    // Control B — "only the hash kept, objects replaced": every stage fact
    // looks clean; ONLY the object-identity axis can catch it.
    let violations = s
        .forced_hash_only_replacement("d_bump")
        .expect("control runs");
    assert!(
        !violations.is_empty(),
        "the gate must redden an equal-content object replacement"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.contains("HIR identity moved for unchanged")),
        "the HIR identity axis reddens: {violations:?}"
    );

    // The controls did not mutate the session and the REAL edit on the
    // same session still passes the same gate.
    assert!(
        matches!(s.run(), Ok(Value::I32(42))),
        "controls are non-mutating"
    );
    let report = s
        .edit_body(&edited, "d_bump")
        .expect("the real edit still passes the gate after the controls");
    assert_eq!(report.facts.parse_fn_items, 1);
    assert!(matches!(s.run(), Ok(Value::I32(43))));
}

#[test]
fn av_data_820_v13_old_snapshot_survives_for_comparison() {
    // "保留合法局部编辑结果与旧快照对照": the superseded snapshot stays
    // executable with its pre-edit result — the legitimate local-edit
    // result is judged against it, not against a recomputation.
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    let old_snap = s.snapshot().clone();
    assert!(matches!(s.run_snapshot(&old_snap), Ok(Value::I32(42))));
    s.edit_body(&SRC.replace("+ 1", "+ 2"), "d_bump")
        .expect("edit");
    assert!(
        matches!(s.run_snapshot(&old_snap), Ok(Value::I32(42))),
        "the old snapshot's result is preserved for comparison"
    );
    assert!(
        matches!(s.run(), Ok(Value::I32(43))),
        "the new revision executes the edit"
    );
}

#[test]
fn av_data_820_v13_multi_function_edit_is_explicit_non_fast() {
    // An edit touching TWO function bodies is not the fast path: an
    // explicit non-fast rejection (route to apply_source), never a silent
    // whole-compile dressed as an increment. The session stays intact.
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    let two = SRC.replace("+ 1", "+ 2").replace("return 7", "return 8");
    let err = s
        .edit_body(&two, "d_bump")
        .expect_err("a two-function edit is not the single-body fast path");
    assert!(err.contains("non-fast-path"), "got {err}");
    assert!(
        matches!(s.run(), Ok(Value::I32(42))),
        "the rejected edit leaves the session intact"
    );
    // Routed by the caller: apply_source handles it with the computed
    // dependency set — exactly the two changed functions. main KEEPS its
    // identity: a helper-body change never re-lowers callers (their units
    // are content-unchanged; the snapshot binds the call to the callee's
    // slot at link time), yet the run follows the edit.
    let main_code = s.identity_of("d_main").expect("main resident");
    let rebuilt = s.apply_source(&two).expect("apply_source handles it");
    let mut rebuilt_sorted = rebuilt.clone();
    rebuilt_sorted.sort();
    assert_eq!(
        rebuilt_sorted,
        vec!["d_bump".to_string(), "d_untouched".to_string()],
        "a helper-body change rebuilds only the changed functions"
    );
    assert_eq!(
        s.identity_of("d_main"),
        Some(main_code),
        "the caller's unit is reused as-is while the call follows the new body"
    );
    assert!(matches!(s.run(), Ok(Value::I32(43))));
}
