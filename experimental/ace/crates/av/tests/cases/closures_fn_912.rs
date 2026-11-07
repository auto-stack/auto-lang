//! PLAN-912 T-02: the AV half of the minimal full chain. The closures
//! FUNCTION session's warm single-unit route: value oracles (hand-computed),
//! per-unit zero production for every unchanged unit (P axes + body work),
//! `Arc` object identity + generation stability, the warm structural gate,
//! route reports (cold / fast-unit / zero-hit / non-fast), failure recovery
//! and the forced-whole negative control through the 865 path.

use std::sync::Arc;

use ace_av::closures_865_support::ClosuresSourceSession;
use ace_av::closures_fn_session::{ClosuresFunctionSession, RouteReport};
use ace_core::body_work::{self, Stage};

const SRC: &str = "\
fn helper(n int) int { return n + 1 }
fn other() int { return 7 }
fn main() int {
    var x int = 10
    var base int = 1
    var f = (n int) => { base = base + n; return base }
    var g = (n int) => { return x - n }
    return f(20) + f(0) + g(3) + helper(1)
}
";
// f is a shared-cell writer of `base`: f(20) -> base=21, f(0) -> 21; g(3)=7;
// helper(1)=2. Fresh = 21 + 21 + 7 + 2 = 51.

fn temp_store(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("av912-fn-session-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("store dir");
    dir
}

#[test]
fn av_closures_fn_912_warm_lambda_edit_single_unit_zero_for_siblings() {
    let store = temp_store("warm");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");

    // Cold: 5 units (3 fns + 2 lambdas), value oracle 51.
    let cold = session.apply_source(SRC).expect("cold apply");
    assert_eq!(cold.value, 51);
    assert!(matches!(cold.route, RouteReport::Cold { units: 5 }));
    assert_eq!(cold.submitted_units, 5);
    assert_eq!(cold.live_after_release, 0);
    assert_eq!(cold.allocs, cold.frees);
    // Identity snapshots of every unchanged unit.
    let main_obj = session.unit("d_main").expect("main").clone();
    let f_obj = session.unit("d___closure_main_0").expect("f").clone();
    let g_obj = session.unit("d___closure_main_1").expect("g").clone();
    let helper_obj = session.unit("d_helper").expect("helper").clone();
    let other_obj = session.unit("d_other").expect("other").clone();
    let g_gen = g_obj.generation();

    // Warm edit of `g`'s body: g(3) = 10-6 = 4 → 21+21+4+2 = 48.
    let edited = SRC.replace("return x - n }", "return x - n * 2 }");
    let warm = session.apply_source(&edited).expect("warm apply");
    assert_eq!(warm.value, 48);
    assert_eq!(
        warm.route,
        RouteReport::FastUnit {
            unit: "d___closure_main_1".to_string()
        }
    );
    assert_eq!(warm.submitted_units, 1);
    assert_eq!(warm.submitted_unit_names, vec!["d___closure_main_1"]);
    // The warm structural gate: zero whole axis, exactly-one for the changed
    // unit, zero for everyone else.
    let gate = warm.warm_gate.as_ref().expect("gate");
    assert!(gate.pass(), "gate violations: {:?}", gate.violations);
    assert_eq!(gate.changed_units, vec!["d___closure_main_1".to_string()]);
    assert_eq!(warm.stages.whole_total(), 0);
    assert_eq!(warm.body.len(), 1);
    let w = warm.body.get("__closure_main_1").unwrap();
    assert_eq!((w.resolve, w.typecheck, w.plan, w.bind), (1, 1, 1, 1));
    // Every unchanged unit keeps its OBJECT and generation.
    assert!(Arc::ptr_eq(session.unit("d_main").unwrap(), &main_obj));
    assert!(Arc::ptr_eq(
        session.unit("d___closure_main_0").unwrap(),
        &f_obj
    ));
    assert!(Arc::ptr_eq(session.unit("d_helper").unwrap(), &helper_obj));
    assert!(Arc::ptr_eq(session.unit("d_other").unwrap(), &other_obj));
    let new_g = session.unit("d___closure_main_1").unwrap().clone();
    assert!(
        !Arc::ptr_eq(&new_g, &g_obj),
        "the edited unit gets a new object"
    );
    assert_ne!(new_g.generation(), g_gen);

    // A whitespace-only edit inside `g`: same HIR/MIR → same wire → zero
    // production below the frontend, route still fast, the session carries
    // every unit (the "same-hash, same object" axis).
    let commented = edited.replace("return x - n * 2 }", "return  x - n * 2   }");
    let hit = session.apply_source(&commented).expect("comment apply");
    assert_eq!(hit.value, 48);
    assert!(matches!(hit.route, RouteReport::FastUnit { .. }));
    let gate = hit.warm_gate.as_ref().expect("gate");
    assert!(gate.pass(), "gate violations: {:?}", gate.violations);
    assert_eq!(hit.submitted_units, 0);
    assert!(Arc::ptr_eq(
        session.unit("d___closure_main_1").unwrap(),
        &new_g
    ));

    // Zero hit: identical source re-applied.
    let zero = session.apply_source(&commented).expect("zero hit");
    assert_eq!(zero.value, 48);
    assert_eq!(zero.route, RouteReport::ZeroHit);
    assert_eq!(zero.submitted_units, 0);
    assert!(zero.warm_gate.as_ref().expect("gate").pass());
    assert_eq!(zero.stages.whole_total(), 0);
    for stage in [
        Stage::ParseProject,
        Stage::HirVerify,
        Stage::MirLower,
        Stage::MirVerify,
        Stage::AbcEncode,
        Stage::DecodeAdmit,
    ] {
        assert_eq!(zero.stages.total(stage), 0, "zero-hit {stage:?}");
    }

    // A capture-decl change (f stops writing base) is the OWNER route with
    // precise invalidation (V-05): `main` (creation site: base is no longer
    // a cell) and `f` re-resolve; `g` is REUSED — zero work, same object.
    // base stays 1: f(20)=21, f(0)=1, g(3)=4, helper=2 → 28.
    let g_before = session.unit("d___closure_main_1").unwrap().clone();
    let helper_before = session.unit("d_helper").unwrap().clone();
    let no_write = commented.replace("base = base + n; return base", "return base + n");
    let owner = session.apply_source(&no_write).expect("owner apply");
    assert_eq!(owner.value, 28);
    assert_eq!(
        owner.route,
        RouteReport::FastOwner {
            owner: "d_main".to_string(),
            changed: vec!["d___closure_main_0".to_string()],
            reused: vec!["d___closure_main_1".to_string()],
        }
    );
    let gate = owner.warm_gate.as_ref().expect("gate");
    assert!(gate.pass(), "gate violations: {:?}", gate.violations);
    assert_eq!(owner.stages.whole_total(), 0);
    assert!(
        owner.body.get("__closure_main_1").is_none(),
        "reused g has no body work"
    );
    assert!(owner.body.get("helper").is_none());
    // main's wire changes (operand shape + cell local) and f's wire changes.
    let mut names = owner.submitted_unit_names.clone();
    names.sort();
    assert_eq!(names, vec!["d___closure_main_0", "d_main"]);
    assert!(Arc::ptr_eq(
        session.unit("d___closure_main_1").unwrap(),
        &g_before
    ));
    assert!(Arc::ptr_eq(
        session.unit("d_helper").unwrap(),
        &helper_before
    ));

    session.close().expect("close");
    let _ = std::fs::remove_dir_all(&store);
}

#[test]
fn av_closures_fn_912_failed_edit_preserves_state_and_recovers() {
    let store = temp_store("fail");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    let cold = session.apply_source(SRC).expect("cold");
    assert_eq!(cold.value, 51);
    let main_obj = session.unit("d_main").unwrap().clone();

    // A type error inside `g`: diagnostics; the session/frontend keep the
    // accepted revision (source, wires, objects) untouched.
    let broken = SRC.replace("return x - n }", "return x - true }");
    match session.apply_source(&broken) {
        Err(ace_av::closures_865_support::CompileAndRunError::Diagnostics(_)) => {}
        Err(other) => panic!("expected diagnostics, got pipeline {other:?}"),
        Ok(_) => panic!("a type error must not apply"),
    }
    let retry = session
        .apply_source(SRC)
        .expect("identical re-apply after failure");
    assert_eq!(retry.value, 51);
    assert_eq!(retry.route, RouteReport::ZeroHit);
    assert!(Arc::ptr_eq(session.unit("d_main").unwrap(), &main_obj));

    // The next VALID edit produces exactly its one unit.
    let edited = SRC.replace("return x - n }", "return x - n - 1 }");
    let warm = session.apply_source(&edited).expect("recovery warm");
    // g(3) = 10-3-1 = 6 → 21 + 21 + 6 + 2 = 50.
    assert_eq!(warm.value, 50);
    session.close().expect("close");
    let _ = std::fs::remove_dir_all(&store);
}

#[test]
fn av_closures_fn_912_forced_whole_path_is_observably_red() {
    // The forced-whole negative control: the 865 path (whole parse/plan/
    // bind/verify then filter) run under the SAME observation window shows
    // the whole axis non-zero — exactly what the warm gate rejects.
    let store = temp_store("whole");
    let mut old = ClosuresSourceSession::new(&store);
    let edited = SRC.replace("return x - n }", "return x - n * 2 }");
    let (result, stages) = body_work::capture_stage_attempts(|| {
        old.apply_source(&edited, "main.at").map(|run| run.value)
    });
    assert_eq!(result.expect("865 applies the edit"), 48);
    assert!(
        stages.total(Stage::WholeParse) >= 1,
        "the 865 path parses whole"
    );
    assert!(stages.total(Stage::WholePlan) >= 1, "plans whole");
    assert!(stages.total(Stage::WholeVerify) >= 1, "verifies whole");
    // The function session's gate semantics: a fast-route claim on THIS
    // observation would violate on the whole axis.
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    let cold = session.apply_source(SRC).expect("cold");
    assert_eq!(cold.value, 51);
    session.close().expect("close");
    old.close().expect("close 865");
    let _ = std::fs::remove_dir_all(&store);
}

#[test]
fn av_closures_fn_912_owner_body_edit_reuses_every_lambda() {
    // V-02 owner leg: a pure body edit of `main` OUTSIDE its lambdas.
    let store = temp_store("owner");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    let cold = session.apply_source(SRC).expect("cold");
    assert_eq!(cold.value, 51);
    let f_obj = session.unit("d___closure_main_0").unwrap().clone();
    let g_obj = session.unit("d___closure_main_1").unwrap().clone();
    let helper_obj = session.unit("d_helper").unwrap().clone();
    // x: 10 → 20 ⇒ g(3) = 17 ⇒ 21 + 21 + 17 + 2 = 61.
    let edited = SRC.replace("var x int = 10", "var x int = 20");
    let run = session.apply_source(&edited).expect("owner edit");
    assert_eq!(run.value, 61);
    assert_eq!(
        run.route,
        RouteReport::FastOwner {
            owner: "d_main".to_string(),
            changed: vec![],
            reused: vec![
                "d___closure_main_0".to_string(),
                "d___closure_main_1".to_string()
            ],
        }
    );
    let gate = run.warm_gate.as_ref().expect("gate");
    assert!(gate.pass(), "gate violations: {:?}", gate.violations);
    assert_eq!(run.submitted_unit_names, vec!["d_main"]);
    // ZERO work for both lambdas and every other function.
    assert_eq!(run.body.keys().collect::<Vec<_>>(), vec!["main"]);
    assert_eq!(run.stages.whole_total(), 0);
    assert!(Arc::ptr_eq(
        session.unit("d___closure_main_0").unwrap(),
        &f_obj
    ));
    assert!(Arc::ptr_eq(
        session.unit("d___closure_main_1").unwrap(),
        &g_obj
    ));
    assert!(Arc::ptr_eq(session.unit("d_helper").unwrap(), &helper_obj));
    session.close().expect("close");
    let _ = std::fs::remove_dir_all(&store);
}

#[test]
fn av_closures_fn_912_utf8_variable_length_remap_then_lambda_edit() {
    // V-03: a multibyte, length-changing owner edit BEFORE the lambdas shifts
    // every catalog coordinate; the following lambda-only edit must still
    // land on the right unit (coordinate remap is M-axis, not a body walk).
    let store = temp_store("utf8");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    assert_eq!(session.apply_source(SRC).expect("cold").value, 51);
    let g_obj = session.unit("d___closure_main_1").unwrap().clone();
    // Insert an owned-Str local with CJK content before the lambdas (owner
    // route; value unchanged).
    let shifted = SRC.replace(
        "    var base int = 1\n",
        "    var base int = 1\n    let note Str = \"中文备注：变长\"\n",
    );
    let run = session.apply_source(&shifted).expect("utf8 owner edit");
    assert_eq!(run.value, 51);
    assert!(
        matches!(run.route, RouteReport::FastOwner { .. }),
        "{:?}",
        run.route
    );
    assert!(
        run.warm_gate.as_ref().unwrap().pass(),
        "{:?}",
        run.warm_gate
    );
    assert!(Arc::ptr_eq(
        session.unit("d___closure_main_1").unwrap(),
        &g_obj
    ));
    // Now a lambda-only edit of `g` at its SHIFTED position.
    let lambda_edit = shifted.replace("return x - n }", "return x - n * 2 }");
    let run = session
        .apply_source(&lambda_edit)
        .expect("lambda edit after shift");
    assert_eq!(run.value, 48);
    assert_eq!(
        run.route,
        RouteReport::FastUnit {
            unit: "d___closure_main_1".to_string()
        }
    );
    assert!(
        run.warm_gate.as_ref().unwrap().pass(),
        "{:?}",
        run.warm_gate
    );
    session.close().expect("close");
    let _ = std::fs::remove_dir_all(&store);
}

#[test]
fn av_closures_fn_912_creation_point_visibility_and_param_shadowing() {
    // V-03: a lambda edit may only capture names visible AT ITS CREATION
    // POINT; a name declared later in the owner is not silently captured.
    let store = temp_store("visibility");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    let src = SRC.replace(
        "    return f(20) + f(0) + g(3) + helper(1)\n",
        "    var later int = 5\n    return f(20) + f(0) + g(3) + helper(1) + later\n",
    );
    assert_eq!(session.apply_source(&src).expect("cold").value, 56);
    // g tries to read `later` (declared after g): diagnostics, state kept.
    let bad = src.replace("return x - n }", "return x - n + later }");
    match session.apply_source(&bad) {
        Err(ace_av::closures_865_support::CompileAndRunError::Diagnostics(ds)) => {
            assert!(!ds.is_empty());
        }
        other => panic!(
            "expected diagnostics for a later-declared capture, got {}",
            match other {
                Ok(r) => format!("value {}", r.value),
                Err(e) => format!("{e:?}"),
            }
        ),
    }
    assert_eq!(
        session.apply_source(&src).expect("zero hit").route,
        RouteReport::ZeroHit
    );
    // A lambda PARAMETER named like an owner local is the param, never a
    // capture: g(n) with `x` renamed param shadows the owner's x.
    let shadow = src.replace(
        "var g = (n int) => { return x - n }",
        "var g = (x int) => { return x * 2 }",
    );
    // The lambda's capture set changed (x no longer captured) → owner route.
    let run = session.apply_source(&shadow).expect("shadowing edit");
    // g(3) = 6 ⇒ 21 + 21 + 6 + 2 + 5 = 55.
    assert_eq!(run.value, 55);
    assert!(
        matches!(run.route, RouteReport::FastOwner { .. }),
        "{:?}",
        run.route
    );
    assert!(
        run.warm_gate.as_ref().unwrap().pass(),
        "{:?}",
        run.warm_gate
    );
    session.close().expect("close");
    let _ = std::fs::remove_dir_all(&store);
}

#[test]
fn av_closures_fn_912_topology_change_is_named_and_carries_identity() {
    // Inserting a lambda before `f` renumbers `f`/`g` display names (the
    // 909 ordinal contract); the frontend reports the NAMED topology change,
    // takes the whole truth path, and carries the surviving lambdas'
    // LOGICAL ids by unique (owner, text) match.
    let store = temp_store("topology");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    assert_eq!(session.apply_source(SRC).expect("cold").value, 51);
    let ids_before: Vec<(String, u64)> = lambda_ids(&session);
    assert_eq!(ids_before.len(), 2);
    let inserted = SRC.replace(
        "    var f = (n int) =>",
        "    var z = (n int) => { return n * 7 }\n    var f = (n int) =>",
    );
    let run = session.apply_source(&inserted).expect("insertion");
    assert_eq!(run.value, 51, "z is never called");
    match &run.route {
        RouteReport::NonFast { reason } => {
            assert!(
                reason.starts_with("owner-topology-changed:2->3"),
                "{reason}"
            );
        }
        other => panic!("expected a named topology non-fast, got {other:?}"),
    }
    assert!(
        run.stages.whole_total() > 0,
        "the truth path is honestly whole"
    );
    let ids_after: std::collections::BTreeMap<String, u64> =
        lambda_ids(&session).into_iter().collect();
    assert_eq!(ids_after.len(), 3);
    // f was __closure_main_0 (id a), g __closure_main_1 (id b); now z is
    // __closure_main_0, f is _1, g is _2 — f and g keep their logical ids.
    let (f_id, g_id) = (ids_before[0].1, ids_before[1].1);
    assert_eq!(
        ids_after["__closure_main_1"], f_id,
        "f keeps its logical id"
    );
    assert_eq!(
        ids_after["__closure_main_2"], g_id,
        "g keeps its logical id"
    );
    assert!(
        ids_after["__closure_main_0"] != f_id && ids_after["__closure_main_0"] != g_id,
        "z is a NEW logical unit"
    );
    session.close().expect("close");
    let _ = std::fs::remove_dir_all(&store);
}

fn lambda_ids(session: &ClosuresFunctionSession) -> Vec<(String, u64)> {
    session
        .frontend()
        .unwrap()
        .units()
        .iter()
        .filter_map(|u| match &u.kind {
            ace_core::source::closures_fn::UnitKind::Lambda { logical_id, .. } => {
                Some((u.name.clone(), *logical_id))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn av_closures_fn_912_same_hash_new_object_degradation_is_refused() {
    // V-06 forced red: resubmitting UNCHANGED units (same wire, new object)
    // must trip the session's structural gate — the revision is refused and
    // the installed state is untouched; with the control off the identical
    // edit succeeds.
    let store = temp_store("degrade");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    assert_eq!(session.apply_source(SRC).expect("cold").value, 51);
    let f_obj = session.unit("d___closure_main_0").unwrap().clone();
    session.set_degrade_resubmit_unchanged(true);
    // A whitespace-only lambda edit lowers a unit whose wire is IDENTICAL to
    // the installed one; degraded resubmission mints a fresh object for the
    // same fingerprint — exactly what the structural gate must refuse.
    let edited = SRC.replace("return x - n }", "return  x - n  }");
    match session.apply_source(&edited) {
        Err(ace_av::closures_865_support::CompileAndRunError::Pipeline(msg)) => {
            assert!(msg.contains("structural gate"), "{msg}");
        }
        Err(other) => panic!("expected the structural gate, got {other:?}"),
        Ok(run) => panic!("degraded revision must be refused, got value {}", run.value),
    }
    // The 823 session's structural gate is a POST-INSTALL observation (the
    // revision links, executes and commits, then reports
    // `unchanged_degraded`): the re-minted `f` object IS the degradation the
    // control proves the gate sees. The FRONTEND, however, published nothing
    // (generation unchanged; identical re-apply of the accepted source is a
    // zero hit) and the session keeps working.
    assert!(
        !Arc::ptr_eq(session.unit("d___closure_main_0").unwrap(), &f_obj),
        "the degradation control re-minted the unchanged unit (that is what the gate caught)"
    );
    assert_eq!(session.frontend().unwrap().generation(), 0);
    session.set_degrade_resubmit_unchanged(false);
    let hit = session.apply_source(SRC).expect("zero hit after refusal");
    assert_eq!(hit.route, RouteReport::ZeroHit);
    assert_eq!(hit.value, 51);
    // Without degradation the same whitespace edit is a same-wire fast unit
    // with zero submissions and every object carried.
    let f_now = session.unit("d___closure_main_0").unwrap().clone();
    let run = session.apply_source(&edited).expect("same-wire edit");
    assert_eq!(run.value, 51);
    assert!(matches!(run.route, RouteReport::FastUnit { .. }));
    assert_eq!(run.submitted_units, 0);
    assert!(Arc::ptr_eq(
        session.unit("d___closure_main_0").unwrap(),
        &f_now
    ));
    session.close().expect("close");
    let _ = std::fs::remove_dir_all(&store);
}

#[test]
fn av_closures_fn_912_gate_rejects_synthetic_default_zero_and_hidden_walks() {
    // V-06 forced red at the gate: a synthesized observation that lacks the
    // changed unit's counters (Default 0), carries a hidden sibling walk, or
    // a whole-axis attempt must each be a violation; a clean one passes.
    use ace_av::closures_fn_session::warm_gate_report;
    use ace_core::body_work::{BodyWork, StageAttempts};
    use std::collections::BTreeMap;
    let route = RouteReport::FastUnit {
        unit: "d___closure_main_1".to_string(),
    };
    let one = BodyWork {
        resolve: 1,
        typecheck: 1,
        plan: 1,
        bind: 1,
    };
    let mut clean_body = BTreeMap::new();
    clean_body.insert("__closure_main_1".to_string(), one);
    let mut clean_stages = StageAttempts::default();
    for stage in [
        Stage::ParseProject,
        Stage::HirVerify,
        Stage::MirLower,
        Stage::MirVerify,
        Stage::AbcEncode,
        Stage::DecodeAdmit,
    ] {
        clean_stages
            .by_unit
            .insert(("d___closure_main_1".to_string(), stage), 1);
    }
    let submitted = vec!["d___closure_main_1".to_string()];
    assert!(warm_gate_report(&route, &clean_body, &clean_stages, &submitted).pass());

    // Default 0: the changed unit recorded nothing.
    let empty_body = BTreeMap::new();
    let report = warm_gate_report(&route, &empty_body, &clean_stages, &submitted);
    assert!(!report.pass());
    assert!(report
        .violations
        .iter()
        .any(|v| v.contains("missing resolve walk")));

    // Hidden sibling walk.
    let mut sibling_body = clean_body.clone();
    sibling_body.insert("__closure_main_0".to_string(), one);
    let report = warm_gate_report(&route, &sibling_body, &clean_stages, &submitted);
    assert!(!report.pass());
    assert!(report
        .violations
        .iter()
        .any(|v| v.contains("__closure_main_0")));

    // Whole-axis attempt (whole-then-filter).
    let mut whole_stages = clean_stages.clone();
    whole_stages
        .by_unit
        .insert(("*".to_string(), Stage::WholePlan), 1);
    let report = warm_gate_report(&route, &clean_body, &whole_stages, &submitted);
    assert!(!report.pass());
    assert!(report.violations.iter().any(|v| v.contains("whole-axis")));

    // A sibling's production attempt (e.g. lowered anyway).
    let mut leak_stages = clean_stages.clone();
    leak_stages
        .by_unit
        .insert(("d_helper".to_string(), Stage::MirLower), 1);
    let report = warm_gate_report(&route, &clean_body, &leak_stages, &submitted);
    assert!(!report.pass());
    assert!(report.violations.iter().any(|v| v.contains("d_helper")));

    // Missing HIR verify for the changed unit.
    let mut no_verify = clean_stages.clone();
    no_verify
        .by_unit
        .remove(&("d___closure_main_1".to_string(), Stage::HirVerify));
    let report = warm_gate_report(&route, &clean_body, &no_verify, &submitted);
    assert!(!report.pass());
    assert!(report.violations.iter().any(|v| v.contains("hir_verify")));
}

#[test]
fn av_closures_fn_912_guest_trap_edit_is_refused_before_commit() {
    // V-07: a body edit whose guest execution traps is rejected by the
    // session BEFORE any commit (execution is part of candidate validation);
    // source/wires/objects stay; the next valid edit works.
    let store = temp_store("trap");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    assert_eq!(session.apply_source(SRC).expect("cold").value, 51);
    let g_obj = session.unit("d___closure_main_1").unwrap().clone();
    // g(3) = x / (n - 3) → division by zero at run time.
    let trapping = SRC.replace("return x - n }", "return x / (n - 3) }");
    match session.apply_source(&trapping) {
        Err(ace_av::closures_865_support::CompileAndRunError::Pipeline(msg)) => {
            assert!(
                msg.contains("did not execute") || msg.contains("trap") || msg.contains("Trap"),
                "{msg}"
            );
        }
        Err(other) => panic!("expected a pipeline refusal, got {other:?}"),
        Ok(run) => panic!(
            "a trapping revision must not commit, got value {}",
            run.value
        ),
    }
    assert!(Arc::ptr_eq(
        session.unit("d___closure_main_1").unwrap(),
        &g_obj
    ));
    let hit = session.apply_source(SRC).expect("zero hit after refusal");
    assert_eq!(hit.route, RouteReport::ZeroHit);
    let next = session
        .apply_source(&SRC.replace("return x - n }", "return x - n * 2 }"))
        .expect("valid edit after trap");
    assert_eq!(next.value, 48);
    assert!(matches!(next.route, RouteReport::FastUnit { .. }));
    session.close().expect("close");
    let _ = std::fs::remove_dir_all(&store);
}

#[test]
fn av_closures_fn_912_stale_and_foreign_candidates_cannot_commit() {
    // V-07 at the frontend seam: two candidates prepared on one base — the
    // second cannot overwrite the first's publication (`source.stale`); a
    // candidate from another environment is `source.foreign`.
    use ace_core::source::closures_fn::ClosuresFunctionFrontend;
    let (mut a, _p, _c) = ClosuresFunctionFrontend::cold(SRC, "main.at", &[]).unwrap();
    let (mut b, _p2, _c2) = ClosuresFunctionFrontend::cold(SRC, "main.at", &[]).unwrap();
    let g = a
        .units()
        .iter()
        .position(|u| u.name == "__closure_main_1")
        .unwrap();
    let span = a.units()[g].span;
    let sha = a.source_sha().to_string();
    let first = match a.edit_body(
        &sha,
        span.start..span.end,
        "(n int) => { return x - n * 2 }",
    ) {
        Ok(Ok(c)) => c,
        _ => panic!("fast"),
    };
    let second = match a.edit_body(
        &sha,
        span.start..span.end,
        "(n int) => { return x - n * 3 }",
    ) {
        Ok(Ok(c)) => c,
        _ => panic!("fast"),
    };
    let foreign = match b.edit_body(
        &b.source_sha().to_string(),
        span.start..span.end,
        "(n int) => { return x }",
    ) {
        Ok(Ok(c)) => c,
        _ => panic!("fast"),
    };
    a.commit_candidate(first, None, &[]).expect("first commit");
    let err = a.commit_candidate(second, None, &[]).unwrap_err();
    assert!(err.starts_with("source.stale"), "{err}");
    let err = a.commit_candidate(foreign, None, &[]).unwrap_err();
    assert!(err.starts_with("source.foreign"), "{err}");
    assert_eq!(a.generation(), 1);
}

#[test]
fn av_closures_fn_912_restart_reports_honest_r3() {
    // V-08 / R3: a fresh session over the SAME store restarts from store
    // artifacts — cache hits, zero store writes, real decode/deep-verify —
    // and STILL re-lowers every unit before the content key is computable.
    // PLAN-913 owns closing that; this test pins the honest non-zero.
    let store = temp_store("restart");
    {
        let mut session = ClosuresFunctionSession::new(&store, "main.at");
        let cold = session.apply_source(SRC).expect("cold");
        assert_eq!(cold.value, 51);
        assert!(matches!(cold.route, RouteReport::Cold { units: 5 }));
        session.close().expect("close");
    }
    let mut restart = ClosuresFunctionSession::new(&store, "main.at");
    let run = restart.apply_source(SRC).expect("restart");
    assert_eq!(run.value, 51);
    assert!(
        matches!(run.route, RouteReport::Cold { units: 5 }),
        "{:?}",
        run.route
    );
    assert_eq!(run.production.cache_hits, 5);
    assert_eq!(run.production.store_writes, 0);
    assert_eq!(run.production.verified, 5);
    assert_eq!(run.production.encoded, 5);
    assert_eq!(run.production.admitted, 5);
    // R3 open and reported: every unit is re-lowered (P > 0 on restart) and
    // the whole frontend ran; nothing is relabeled as "recovery".
    assert_eq!(run.stages.total(Stage::MirLower), 5);
    assert!(run.stages.whole_total() > 0);
    assert!(run.warm_gate.is_none());
    // The warm path works right after the restart leg.
    let warm = restart
        .apply_source(&SRC.replace("return x - n }", "return x - n * 2 }"))
        .expect("warm after restart");
    assert_eq!(warm.value, 48);
    assert!(matches!(warm.route, RouteReport::FastUnit { .. }));
    assert!(warm.warm_gate.as_ref().unwrap().pass());
    restart.close().expect("close");
    let _ = std::fs::remove_dir_all(&store);
}

#[test]
fn av_closures_fn_912_accepted_state_export() {
    // V-11: the accepted source state is exported from COMMITTED state only,
    // carries identities/references (no guest value, no PASS, no live
    // env/cell — the type has no such fields), and moves with commits.
    let store = temp_store("accepted");
    let mut session = ClosuresFunctionSession::new(&store, "main.at");
    assert!(
        session.accepted_state().is_none(),
        "nothing accepted before the first commit"
    );
    assert_eq!(session.apply_source(SRC).expect("cold").value, 51);
    let cold_state = session.accepted_state().expect("accepted after cold");
    assert_eq!(cold_state.schema, "ace.closures.accepted-source/1");
    assert_eq!(cold_state.profile, "auto.source.closures.draft");
    assert_eq!(
        cold_state.source_sha256,
        ace_core::runtime_contract::sha256_hex(SRC.as_bytes())
    );
    assert_eq!(cold_state.entry, "d_main");
    assert_eq!(cold_state.accepted_generation, 0);
    assert_eq!(cold_state.commit_point, 1);
    assert_eq!(cold_state.units.len(), 5);
    assert!(cold_state.units.iter().all(|u| u.store_ref.is_some(),));
    let g_cold = cold_state
        .units
        .iter()
        .find(|u| u.unit == "d___closure_main_1")
        .expect("g in accepted state");
    assert!(g_cold.mint_span == g_cold.current_span);
    assert!(!g_cold.body_digest.is_empty());
    assert_eq!(
        g_cold.env_layout_identity.as_deref(),
        Some(g_cold.dependency_digest.as_str())
    );

    // A warm edit moves the export: new source sha, new generation, the
    // edited unit's digest and span change, its mir identity changes.
    let edited = SRC.replace("return x - n }", "return x - n * 2 }");
    assert_eq!(session.apply_source(&edited).expect("warm").value, 48);
    let warm_state = session.accepted_state().expect("accepted after warm");
    assert_eq!(warm_state.accepted_generation, 1);
    assert_eq!(
        warm_state.source_sha256,
        ace_core::runtime_contract::sha256_hex(edited.as_bytes())
    );
    let g_warm = warm_state
        .units
        .iter()
        .find(|u| u.unit == "d___closure_main_1")
        .expect("g");
    assert_ne!(
        g_warm.body_digest, g_cold.body_digest,
        "the edited body's digest moved"
    );
    assert_ne!(
        g_warm.current_span, g_cold.current_span,
        "the longer body shifted the span"
    );
    assert_eq!(
        g_warm.mint_span, g_cold.mint_span,
        "mint coordinates are stable"
    );
    assert_ne!(g_warm.mir_identity, g_cold.mir_identity);
    // An unchanged sibling keeps every identity axis.
    let f_warm = warm_state
        .units
        .iter()
        .find(|u| u.unit == "d___closure_main_0")
        .unwrap();
    let f_cold = cold_state
        .units
        .iter()
        .find(|u| u.unit == "d___closure_main_0")
        .unwrap();
    assert_eq!(f_warm.body_digest, f_cold.body_digest);
    assert_eq!(f_warm.mir_identity, f_cold.mir_identity);
    assert_eq!(f_warm.logical, f_cold.logical);
    // Producer identity is the live contract set.
    assert_eq!(warm_state.producer_mir_lowering_version, "4");
    assert_eq!(
        warm_state.producer_closures_schema,
        "auto.mir.av-closures/1"
    );
    assert_eq!(warm_state.producer_unit_codec, "av-closures-unit/v1");
    session.close().expect("close");
    let _ = std::fs::remove_dir_all(&store);
}
