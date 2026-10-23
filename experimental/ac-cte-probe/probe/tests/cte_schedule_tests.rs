//! cte_schedule_ — scheduling/cancellation/Meta-broker/cycle family
//! (PLAN-817 T-03, V-01..V-04): query cycles with chains, legal shared
//! DAGs with single-flight, ordinary recursion ≠ query cycle, fuel and
//! cancellation as isolated states (never language verdicts, never
//! published), latent vs executed traps with the CT call source in the
//! diagnostic, and the Meta parameter broker incl. negative-dependency
//! freshness.
//!
//! Independent expectations are hand-computed (never AV/freeze-derived).

use ac_sema_core::{Ann, InputBlock, InputFn, InputRecord, SemaInput};
use auto_cte_contract::{
    CteCode, CteRequestV1, CteSourceProfile, CteValue, QueryNodeKind, QueryNodeV1, CTE_META_CAP,
    CTE_RECIPE, CTE_REQUEST_SCHEMA, MAX_QUERY_NODES_PER_REQUEST, MAX_QUERY_REF_DEPTH,
};
use auto_cte_core::dependencies::{build_manifest, manifest_fresh};
use auto_cte_core::schedule::resolve_queries;
use auto_cte_probe::driver::{CteDriver, StepStatus};

fn deck(src: &str, tail: &str) -> String {
    format!("deck cte817/1\nmodule main\nsource <<\n{src}\n>>\n{tail}entry main\n")
}

// ---------------------------------------------------------------------------
// V-03: cycles
// ---------------------------------------------------------------------------

#[test]
fn cte_schedule_query_cycle_small_located() {
    let d = deck(
        "fn cte_sum(a int, b int) int {\n    return a + b\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call cte_sum(q2, 1)\nquery q2 = call cte_sum(1, 2)\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(
        out.code.as_deref(),
        Some("cte.schedule.query-cycle"),
        "{:?}",
        out.message
    );
    assert!(
        out.message
            .as_deref()
            .unwrap_or_default()
            .contains("q1 -> q2"),
        "the diagnostic carries the chain: {:?}",
        out.message
    );
    assert_eq!(out.ct_execute, 0, "cycles reject before any AV call");
}

#[test]
fn cte_schedule_query_cycle_long_located() {
    let d = deck(
        "fn f(a int) int {\n    return a\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query qa = call f(qb)\nquery qb = call f(qc)\nquery qc = call f(qa)\nbind main.ct = qa expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.code.as_deref(), Some("cte.schedule.query-cycle"));
    let msg = out.message.as_deref().unwrap_or_default();
    assert!(
        msg.contains("qa -> qb") && msg.contains("qc -> qa"),
        "chain names the full cycle incl. the closing edge: {msg}"
    );
    assert_eq!(out.ct_execute, 0);
}

#[test]
fn cte_schedule_legal_dag_single_flight() {
    // Two identical calls dedupe (single-flight); an earlier result feeds
    // a later call — a legal DAG, never misjudged as a cycle.
    let d = deck(
        "fn add(a int, b int) int {\n    return a + b\n}\n\nfn scale(n int) int {\n    return n * 2\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call add(20, 22)\nquery q2 = call add(20, 22)\nquery q3 = call scale(q1)\nbind main.ct = q3 expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(
        out.ct_execute, 2,
        "single-flight: one add + one scale (not three calls)"
    );
    assert_eq!(
        out.values.get("main.ct").map(|v| v.render()),
        Some("i32:84".to_string()),
        "scale(add(20,22)) = 84 (hand-computed)"
    );
    assert_eq!(out.hir_proof.get("main.ct").map(String::as_str), Some("84"));
}

#[test]
fn cte_schedule_normal_recursion_is_not_a_cycle() {
    let d = deck(
        "fn down(n int) int {\n    if n < 1 {\n        return 0\n    }\n    return down(n + -1)\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call down(5)\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(out.ct_execute, 1);
    assert_eq!(
        out.values.get("main.ct").map(|v| v.render()),
        Some("i32:0".into())
    );
}

// ---------------------------------------------------------------------------
// V-03: fuel / cancellation — isolated states, never language verdicts
// ---------------------------------------------------------------------------

#[test]
fn cte_schedule_fuel_recursion_resource_class() {
    let d = deck(
        "fn spin(n int) int {\n    return spin(n)\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call spin(1)\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(
        out.code.as_deref(),
        Some("cte.resource.exhausted"),
        "{:?}",
        out.code
    );
    assert!(
        !out.language_class,
        "fuel exhaustion is a resource state, not a language verdict"
    );
    assert_eq!(out.ct_execute, 1, "the execution fact is recorded");
}

#[test]
fn cte_schedule_cancel_before_request_isolated() {
    let d = deck(
        "fn f() int {\n    return 7\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call f()\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    drv.cancel();
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(out.code.as_deref(), Some("cte.cancelled"));
    assert!(
        !out.language_class,
        "cancellation is never a language verdict"
    );
    assert_eq!(out.ct_execute, 0);
    // The flag is per-request: the next request runs normally.
    let next = drv.query_step(&d, None);
    assert_eq!(
        next.status,
        StepStatus::Ok,
        "auto-cleared: {:?}",
        next.message
    );
    assert_eq!(next.ct_execute, 1);
}

#[test]
fn cte_schedule_cancel_midflight_publishes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let store = tmp.path().join("store");
    // Exponential calls burn the step budget slowly enough that a 5ms
    // cancellation lands MID-EXECUTION (the AV polls every 1024 steps).
    let d = deck(
        "fn burn(n int) int {\n    if n < 1 {\n        return 1\n    }\n    return burn(n + -1) + burn(n + -1)\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call burn(22)\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::with_store(&store, "cancel-midflight").unwrap();
    let flag = drv.cancel_flag();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(5));
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Diag, "{:?}", out.message);
    assert!(
        out.code.as_deref() == Some("cte.cancelled")
            || out.code.as_deref() == Some("cte.resource.exhausted"),
        "cancelled mid-flight (or the budget fired first — both isolated): {:?}",
        out.code
    );
    assert!(!out.language_class);
    // A cancelled/exhausted request never publishes: the frozen layer
    // has no burn entry. A later COMPLETABLE request against the same
    // store misses (nothing was restored from the interrupted run) and
    // its own entry publishes normally.
    let ok_deck = deck(
        "fn f() int {
    return 7
}

fn main() int {
    let ct = 0
    return ct
}
",
        "query q1 = call f()
bind main.ct = q1 expect i32 placeholder 0
",
    );
    let mut clean = CteDriver::with_store(&store, "cancel-midflight").unwrap();
    let after = clean.query_step(&ok_deck, None);
    assert_eq!(after.status, StepStatus::Ok, "{:?}", after.message);
    assert_eq!(after.cache, Some("miss"), "no late snapshot was published");
    let repeat = clean.query_step(&ok_deck, None);
    assert_eq!(
        repeat.cache,
        Some("hit"),
        "the clean entry published normally"
    );
}

// ---------------------------------------------------------------------------
// V-02: latent vs executed traps, call source in the diagnostic,
//       consumer legs skipped when the CT diags
// ---------------------------------------------------------------------------

#[test]
fn cte_schedule_trap_latent_in_unexecuted_branch() {
    let d = deck(
        "fn guard(x int) int {\n    if 100 < x {\n        return 2147483647 + 1\n    }\n    return x\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call guard(1)\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(
        out.values.get("main.ct").map(|v| v.render()),
        Some("i32:1".into())
    );
    assert_eq!(out.hir_proof.get("main.ct").map(String::as_str), Some("1"));
}

#[test]
fn cte_schedule_trap_executed_diag_names_callee() {
    let d = deck(
        "fn boom() int {\n    return 2147483647 + 1\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call boom()\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(
        out.code.as_deref(),
        Some("cte.trap.int-overflow"),
        "{:?}",
        out.code
    );
    assert!(
        out.language_class,
        "an executed compiletime trap IS a language diagnostic"
    );
    assert_eq!(out.ct_execute, 1);
    assert!(
        out.message.as_deref().unwrap_or_default().contains("boom"),
        "the diagnostic carries the CT call source: {:?}",
        out.message
    );
    // The consumer legs never ran on a diaged CT value.
    assert!(out.hir_proof.is_empty());
    assert!(out.consumer_types.is_empty());
    assert_eq!(out.native_exit, None);
}

#[test]
fn cte_schedule_trap_helper_chain_names_requested_callee() {
    let d = deck(
        "fn helper() int {\n    return 2147483647 * 2\n}\n\nfn outer() int {\n    return helper()\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call outer()\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.code.as_deref(), Some("cte.trap.int-overflow"));
    assert!(
        out.message.as_deref().unwrap_or_default().contains("outer"),
        "the call source is the REQUESTED callee: {:?}",
        out.message
    );
}

#[test]
fn cte_schedule_execution_precedes_consumer_not_completed() {
    // V-02 ordering: the CT value executes FIRST (ct_execute=1) and the
    // consumer then fails its expected-type check — a diaged consumer
    // never erases the execution fact, and the whole program was never
    // required to be Checked beforehand.
    let d = deck(
        "fn f() int {\n    return 3\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call f()\nbind main.ct = q1 expect bool placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(out.code.as_deref(), Some("cte.consumer.type-mismatch"));
    assert_eq!(
        out.ct_execute, 1,
        "the callee ran before the consumer check"
    );
}

// ---------------------------------------------------------------------------
// V-03/AC-02: ceilings (both sides)
// ---------------------------------------------------------------------------

fn unit_request(queries: Vec<QueryNodeV1>) -> CteRequestV1 {
    CteRequestV1 {
        schema: CTE_REQUEST_SCHEMA,
        recipe: CTE_RECIPE,
        module: "m".into(),
        source: String::new(),
        queries,
        bindings: Vec::new(),
        entry: "main".into(),
        caps: Vec::new(),
        native: false,
        profile: CteSourceProfile::Core,
        exec: auto_cte_contract::CteExecMode::Closure,
        meta_source: None,
    }
}

fn input() -> SemaInput {
    SemaInput {
        package: "cte817".into(),
        module: "m".into(),
        source_fp: String::new(),
        fns: vec![InputFn {
            name: "main".into(),
            params: vec![],
            ret: Ann::I32,
            body: InputBlock { stmts: vec![] },
        }],
        records: vec![InputRecord {
            name: "Pair".into(),
            fields: vec![("left".into(), Ann::I32), ("right".into(), Ann::Bool)],
            methods: vec![],
        }],
    }
}

#[test]
fn cte_schedule_query_node_ceiling_both_sides() {
    let ok: Vec<QueryNodeV1> = (0..MAX_QUERY_NODES_PER_REQUEST)
        .map(|i| QueryNodeV1 {
            id: format!("q{i}"),
            kind: QueryNodeKind::Literal(CteValue::I32(i as i32)),
        })
        .collect();
    assert!(resolve_queries(&unit_request(ok), &input()).is_ok());
    let over: Vec<QueryNodeV1> = (0..=MAX_QUERY_NODES_PER_REQUEST)
        .map(|i| QueryNodeV1 {
            id: format!("q{i}"),
            kind: QueryNodeKind::Literal(CteValue::I32(i as i32)),
        })
        .collect();
    let e = resolve_queries(&unit_request(over), &input()).unwrap_err();
    assert_eq!(e.code, CteCode::TooManyQueries);
}

#[test]
fn cte_schedule_ref_depth_ceiling_boundary() {
    // A 128-node literal->ref chain: the last node sits at depth 127, one
    // under the frozen <=128 ceiling — resolves clean. (The node cap and
    // the depth cap are both 128, so a legal node count can never exceed
    // the depth ceiling — the depth guard is the invariant backstop for
    // future node-cap changes.)
    let mut chain = Vec::new();
    for i in 0..MAX_QUERY_REF_DEPTH {
        let kind = if i == 0 {
            QueryNodeKind::Literal(CteValue::I32(1))
        } else {
            QueryNodeKind::QueryRef(format!("q{}", i - 1))
        };
        chain.push(QueryNodeV1 {
            id: format!("q{i}"),
            kind,
        });
    }
    let p = resolve_queries(&unit_request(chain), &input()).unwrap();
    assert_eq!(
        p.values.get(&format!("q{}", MAX_QUERY_REF_DEPTH - 1)),
        Some(&CteValue::I32(1)),
        "the full legal chain resolves"
    );
}

// ---------------------------------------------------------------------------
// V-01/V-03: the Meta parameter broker (unit level over a real SemaInput)
// ---------------------------------------------------------------------------

fn meta_request(caps: &[String], queries: Vec<QueryNodeV1>) -> CteRequestV1 {
    let mut r = unit_request(queries);
    r.caps = caps.to_vec();
    r
}

#[test]
fn cte_schedule_meta_broker_gated_and_valued() {
    let cap = vec![CTE_META_CAP.to_string()];
    let qs = vec![
        QueryNodeV1 {
            id: "n".into(),
            kind: QueryNodeKind::FieldCount {
                record: "Pair".into(),
            },
        },
        QueryNodeV1 {
            id: "h".into(),
            kind: QueryNodeKind::HasField {
                record: "Pair".into(),
                field: "zzz".into(),
            },
        },
        QueryNodeV1 {
            id: "ok".into(),
            kind: QueryNodeKind::HasField {
                record: "Pair".into(),
                field: "left".into(),
            },
        },
    ];
    // Capability gate first.
    let e = resolve_queries(&meta_request(&[], qs.clone()), &input()).unwrap_err();
    assert_eq!(e.code, CteCode::MetaNotGranted);
    // With the cap: real values, facts and probe bookkeeping.
    let p = resolve_queries(&meta_request(&cap, qs), &input()).unwrap();
    assert_eq!(p.meta.get("n"), Some(&CteValue::I32(2)));
    assert_eq!(
        p.meta.get("h"),
        Some(&CteValue::Bool(false)),
        "absent field of an EXISTING record is false + a negative dep"
    );
    assert_eq!(p.meta.get("ok"), Some(&CteValue::Bool(true)));
    assert!(p.meta_facts.contains(&"Pair#2".to_string()));
    assert!(p.meta_facts.contains(&"Pair.zzz=false".to_string()));
    assert!(p.meta_probes.contains(&("Pair".into(), "zzz".into())));
    assert!(p.av_calls.is_empty(), "meta reads never mint AV executions");
    // A missing record is a semantic error (never a runtime value).
    let bad = vec![QueryNodeV1 {
        id: "x".into(),
        kind: QueryNodeKind::FieldCount {
            record: "Nope".into(),
        },
    }];
    let e = resolve_queries(&meta_request(&cap, bad), &input()).unwrap_err();
    assert_eq!(e.code, CteCode::MetaRecordNotFound);
}

#[test]
fn cte_schedule_meta_freshness_negative_dependency() {
    let cap = vec![CTE_META_CAP.to_string()];
    let probes = vec![("Pair".to_string(), "zzz".to_string())];
    let i1 = input();
    let m = build_manifest(&i1, "main", &cap, &["Pair".to_string()], &probes).unwrap();
    assert!(m.meta_facts.contains(&"Pair#2".to_string()));
    assert!(m.meta_facts.contains(&"Pair.zzz=false".to_string()));
    assert!(manifest_fresh(&m, &i1, &cap), "unchanged input stays fresh");
    // The probed-ABSENT field appears → the negative dependency flips.
    let mut i2 = input();
    i2.records[0].fields.push(("zzz".into(), Ann::I32));
    assert!(
        !manifest_fresh(&m, &i2, &cap),
        "Pair.zzz appearing must invalidate (negative dependency)"
    );
    // A field-count change (add) also flips the fact.
    let mut i3 = input();
    i3.records[0].fields.push(("extra".into(), Ann::I32));
    assert!(
        !manifest_fresh(&m, &i3, &cap),
        "Pair#2 -> Pair#3 invalidates"
    );
}

#[test]
fn cte_schedule_meta_deck_record_profile_vertical() {
    // Deck-level pure-Meta vertical on the record profile: the meta value
    // feeds a consumer binding end-to-end (infer + HIR backfill + native).
    let d = "deck cte817/1
module main
profile record
caps cte.meta.fields/v1
source <<
type Pair {
    left int
    right bool
}

fn main() int {
    let n = 0
    return n
}
>>
query n = field_count Pair
query gate = has_field Pair zzz
bind main.n = n expect i32 placeholder 0
entry main
native off
";
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(out.meta_query, 2, "two meta reads");
    assert_eq!(out.ct_execute, 0, "pure meta never executes the AV");
    assert_eq!(
        out.values.get("main.n").map(|v| v.render()),
        Some("i32:2".into())
    );
    assert_eq!(out.hir_proof.get("main.n").map(String::as_str), Some("2"));
}

// ---------------------------------------------------------------------------
// F-817-R1-05: the contracted Meta → REAL-AV-argument path (meta block)
// ---------------------------------------------------------------------------

fn meta_arg_deck(fields: usize, sentinel_field: bool) -> String {
    let mut fields_txt = String::from("    left int\n    right bool\n");
    for i in 0..fields {
        fields_txt.push_str(&format!("    extra{i} int\n"));
    }
    if sentinel_field {
        fields_txt.push_str("    zzz int\n");
    }
    let mut d = String::new();
    d.push_str("deck cte817/1\nmodule main\ncaps cte.meta.fields/v1\nmeta <<\ntype Pair {\n");
    d.push_str(&fields_txt);
    d.push_str("}\n>>\nsource <<\nfn scale(n int) int {\n    return n * 100\n}\n\nfn main() int {\n    let v = 0\n    let h = false\n    if h {\n        return v + 1\n    }\n    return v\n}\n>>\nquery n = field_count Pair\nquery h = has_field Pair zzz\nquery q = call scale(n)\nbind main.v = q expect i32 placeholder 0\nbind main.h = h expect bool placeholder false\nentry main\nnative off\n");
    d
}

#[test]
fn cte_schedule_meta_argument_real_av_execution() {
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&meta_arg_deck(0, false), None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(out.meta_query, 2, "FieldCount + HasField");
    assert_eq!(out.ct_execute, 1, "the meta result fed a REAL AV call");
    assert_eq!(
        out.values.get("main.v").map(|v| v.render()),
        Some("i32:200".into()),
        "scale(field_count=2) = 200 (hand-computed)"
    );
    assert_eq!(
        out.values.get("main.h").map(|v| v.render()),
        Some("bool:false".into())
    );
    assert_eq!(out.hir_proof.get("main.v").map(String::as_str), Some("200"));
}

#[test]
fn cte_schedule_meta_argument_invalidates_precisely() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let mut d = CteDriver::with_store(&root, "meta-arg").unwrap();
    let base = meta_arg_deck(0, false);
    assert_eq!(d.query_step(&base, None).cache, Some("miss"));

    // Field-set change: FieldCount flips 2→3 AND the absent field zzz
    // APPEARS (negative dependency) — the value recomputes for real.
    let grown = meta_arg_deck(1, false);
    let out = d.query_step(&grown, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(
        out.cache,
        Some("miss"),
        "the meta fact change invalidated the frozen value"
    );
    assert_eq!(
        out.values.get("main.v").map(|v| v.render()),
        Some("i32:300".into())
    );
    // Absent-field flip: zzz appears → HasField true (consumer branch!).
    let flipped = meta_arg_deck(0, true);
    let out = d.query_step(&flipped, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(out.cache, Some("miss"));
    assert_eq!(
        out.values.get("main.h").map(|v| v.render()),
        Some("bool:true".into())
    );
    // The ORIGINAL deck still hits its own untouched entry.
    assert_eq!(d.query_step(&base, None).cache, Some("hit"));
}

// ---------------------------------------------------------------------------
// F-817-R1-05 closure: meta-input negative controls (resource/decode/
// permission/parse classification for the NEW meta block surface)
// ---------------------------------------------------------------------------

#[test]
fn cte_schedule_meta_block_negatives() {
    // (a) No capability granted: FieldCount is refused BEFORE any AV.
    let no_cap = meta_arg_deck(0, false).replace("caps cte.meta.fields/v1\n", "");
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&no_cap, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(out.code.as_deref(), Some("cte.capability.meta"));
    assert_eq!(out.ct_execute, 0);

    // (b) Missing record in the meta block: a semantic error naming it.
    let bad_rec =
        meta_arg_deck(0, false).replace("query n = field_count Pair", "query n = field_count Nope");
    let out = d.query_step(&bad_rec, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(
        out.code.as_deref(),
        Some("cte.meta.record-not-found"),
        "{:?}",
        out.code
    );
    assert_eq!(out.ct_execute, 0);

    // (c) A malformed meta block is a LANGUAGE-class diagnostic.
    let broken = meta_arg_deck(0, false).replace("type Pair {", "type Pair {{{[");
    let out = d.query_step(&broken, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert!(
        out.code.as_deref().unwrap_or_default().starts_with("cte."),
        "typed code: {:?}",
        out.code
    );
    assert!(
        out.language_class,
        "meta parse failure is a language diagnostic"
    );
    assert_eq!(out.ct_execute, 0);

    // (d) The 4MiB deep-input ceiling covers the meta block too.
    let mut pad = String::from("# meta padding\n");
    while pad.len() <= 4 * 1024 * 1024 + 64 {
        pad.push_str("# meta padding padding padding padding\n");
    }
    let fat = meta_arg_deck(0, false).replace("deck cte817/1\n", &format!("deck cte817/1\n{pad}"));
    let out = d.query_step(&fat, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert!(
        out.message.as_deref().unwrap_or_default().contains("input"),
        "the deep-input ceiling rejects: {:?}",
        out.message
    );
    assert_eq!(out.ct_execute, 0);
}
