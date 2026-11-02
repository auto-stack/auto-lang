//! PLAN-912 T-02: the AV half of the minimal full chain. The closures
//! FUNCTION session's warm single-unit route: value oracles (hand-computed),
//! per-unit zero production for every unchanged unit (P axes + body work),
//! `Arc` object identity + generation stability, the warm structural gate,
//! route reports (cold / fast-unit / zero-hit / non-fast), failure recovery
//! and the forced-whole negative control through the 865 path.

use std::sync::Arc;

use ace_av::closures_fn_session::{ClosuresFunctionSession, RouteReport};
use ace_av::closures_865_support::ClosuresSourceSession;
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
    let dir = std::env::temp_dir().join(format!(
        "av912-fn-session-{tag}-{}",
        std::process::id()
    ));
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
    assert!(Arc::ptr_eq(session.unit("d___closure_main_0").unwrap(), &f_obj));
    assert!(Arc::ptr_eq(session.unit("d_helper").unwrap(), &helper_obj));
    assert!(Arc::ptr_eq(session.unit("d_other").unwrap(), &other_obj));
    let new_g = session.unit("d___closure_main_1").unwrap().clone();
    assert!(!Arc::ptr_eq(&new_g, &g_obj), "the edited unit gets a new object");
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
    assert!(Arc::ptr_eq(session.unit("d___closure_main_1").unwrap(), &new_g));

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
    let no_write = commented.replace(
        "base = base + n; return base",
        "return base + n",
    );
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
    assert!(owner.body.get("__closure_main_1").is_none(), "reused g has no body work");
    assert!(owner.body.get("helper").is_none());
    // main's wire changes (operand shape + cell local) and f's wire changes.
    let mut names = owner.submitted_unit_names.clone();
    names.sort();
    assert_eq!(names, vec!["d___closure_main_0", "d_main"]);
    assert!(Arc::ptr_eq(session.unit("d___closure_main_1").unwrap(), &g_before));
    assert!(Arc::ptr_eq(session.unit("d_helper").unwrap(), &helper_before));

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
    let retry = session.apply_source(SRC).expect("identical re-apply after failure");
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
    assert!(stages.total(Stage::WholeParse) >= 1, "the 865 path parses whole");
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
    assert!(Arc::ptr_eq(session.unit("d___closure_main_0").unwrap(), &f_obj));
    assert!(Arc::ptr_eq(session.unit("d___closure_main_1").unwrap(), &g_obj));
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
    assert!(matches!(run.route, RouteReport::FastOwner { .. }), "{:?}", run.route);
    assert!(run.warm_gate.as_ref().unwrap().pass(), "{:?}", run.warm_gate);
    assert!(Arc::ptr_eq(session.unit("d___closure_main_1").unwrap(), &g_obj));
    // Now a lambda-only edit of `g` at its SHIFTED position.
    let lambda_edit = shifted.replace("return x - n }", "return x - n * 2 }");
    let run = session.apply_source(&lambda_edit).expect("lambda edit after shift");
    assert_eq!(run.value, 48);
    assert_eq!(
        run.route,
        RouteReport::FastUnit {
            unit: "d___closure_main_1".to_string()
        }
    );
    assert!(run.warm_gate.as_ref().unwrap().pass(), "{:?}", run.warm_gate);
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
        other => panic!("expected diagnostics for a later-declared capture, got {}", match other { Ok(r) => format!("value {}", r.value), Err(e) => format!("{e:?}") }),
    }
    assert_eq!(session.apply_source(&src).expect("zero hit").route, RouteReport::ZeroHit);
    // A lambda PARAMETER named like an owner local is the param, never a
    // capture: g(n) with `x` renamed param shadows the owner's x.
    let shadow = src.replace("var g = (n int) => { return x - n }", "var g = (x int) => { return x * 2 }");
    // The lambda's capture set changed (x no longer captured) → owner route.
    let run = session.apply_source(&shadow).expect("shadowing edit");
    // g(3) = 6 ⇒ 21 + 21 + 6 + 2 + 5 = 55.
    assert_eq!(run.value, 55);
    assert!(matches!(run.route, RouteReport::FastOwner { .. }), "{:?}", run.route);
    assert!(run.warm_gate.as_ref().unwrap().pass(), "{:?}", run.warm_gate);
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
            assert!(reason.starts_with("owner-topology-changed:2->3"), "{reason}");
        }
        other => panic!("expected a named topology non-fast, got {other:?}"),
    }
    assert!(run.stages.whole_total() > 0, "the truth path is honestly whole");
    let ids_after: std::collections::BTreeMap<String, u64> =
        lambda_ids(&session).into_iter().collect();
    assert_eq!(ids_after.len(), 3);
    // f was __closure_main_0 (id a), g __closure_main_1 (id b); now z is
    // __closure_main_0, f is _1, g is _2 — f and g keep their logical ids.
    let (f_id, g_id) = (ids_before[0].1, ids_before[1].1);
    assert_eq!(ids_after["__closure_main_1"], f_id, "f keeps its logical id");
    assert_eq!(ids_after["__closure_main_2"], g_id, "g keeps its logical id");
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
