//! cte_cache_ — dependency invalidation & hit-side re-admission family
//! (PLAN-817 T-04, V-06 core): caller source unchanged while the helper
//! body/signature changes → invalidated (fresh miss, correct new value,
//! the old entry intact and hittable again after revert); unrelated-fn
//! body edits reuse under the declared closure granularity; caps changes
//! re-key; meta negative-dependency facts re-key the identity; and a
//! cache hit NEVER caches the consumer verdict (a mismatching consumer
//! fails on a hit).

use ace_comptime::driver::{CteDriver, StepStatus};
use ace_comptime_core::dependencies::{build_manifest, manifest_fp, manifest_fresh};
use ace_semantics::{Ann, InputBlock, InputFn, InputRecord, SemaInput};

fn deck_with(src: &str, tail: &str) -> String {
    format!("deck cte817/1\nmodule main\nsource <<\n{src}\n>>\n{tail}entry main\n")
}

fn sum_deck(body: &str) -> String {
    deck_with(
        &format!(
            "fn cte_sum(a int, b int) int {{\n    {body}\n}}\n\nfn other() int {{\n    return 1\n}}\n\nfn main() int {{\n    let ct = 0\n    return ct\n}}\n"
        ),
        "query q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\n",
    )
}

#[test]
fn cache_helper_body_edit_invalidates_and_old_entry_intact() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let mut d = CteDriver::with_store(&root, "v6-body").unwrap();

    let base = sum_deck("return a + b");
    let first = d.query_step(&base, None);
    assert_eq!(first.cache, Some("miss"));
    assert_eq!(
        first.values.get("main.ct").map(|v| v.render()),
        Some("i32:42".into())
    );

    // Caller source shape unchanged; the helper body changes — the value
    // invalidates and recomputes (new key, correct new value).
    let edited = sum_deck("return a + b + 1");
    let second = d.query_step(&edited, None);
    assert_eq!(second.status, StepStatus::Ok, "{:?}", second.message);
    assert_eq!(second.cache, Some("miss"), "helper body edit invalidates");
    assert_eq!(
        second.values.get("main.ct").map(|v| v.render()),
        Some("i32:43".into())
    );

    // The old entry is intact: reverting the deck HITS the first value.
    let reverted = d.query_step(&base, None);
    assert_eq!(
        reverted.cache,
        Some("hit"),
        "the old commit survived untouched"
    );
    assert_eq!(
        reverted.values.get("main.ct").map(|v| v.render()),
        Some("i32:42".into())
    );
}

#[test]
fn cache_helper_signature_edit_invalidates() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let mut d = CteDriver::with_store(&root, "v6-sig").unwrap();

    let base = sum_deck("return a + b");
    assert_eq!(d.query_step(&base, None).cache, Some("miss"));

    // New signature (extra param) + matching call — a fresh key/value.
    let edited = deck_with(
        "fn cte_sum(a int, b int, c int) int {\n    return a + b + c\n}\n\nfn other() int {\n    return 1\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call cte_sum(20, 22, 5)\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let second = d.query_step(&edited, None);
    assert_eq!(second.status, StepStatus::Ok, "{:?}", second.message);
    assert_eq!(
        second.cache,
        Some("miss"),
        "signature fingerprint invalidates"
    );
    assert_eq!(
        second.values.get("main.ct").map(|v| v.render()),
        Some("i32:47".into())
    );
}

#[test]
fn cache_unrelated_body_edit_reuses_under_declared_granularity() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let mut d = CteDriver::with_store(&root, "v6-unrelated").unwrap();

    let base = sum_deck("return a + b");
    assert_eq!(d.query_step(&base, None).cache, Some("miss"));

    // `other` is OUTSIDE the callee's closure: editing only its body
    // leaves the manifest (closure bodies + candidate sets) identical —
    // the frozen value is reused (the declared granularity honored).
    let edited = sum_deck("return a + b").replace(
        "fn other() int {\n    return 1\n}",
        "fn other() int {\n    return 2\n}",
    );
    let second = d.query_step(&edited, None);
    assert_eq!(second.status, StepStatus::Ok, "{:?}", second.message);
    assert_eq!(
        second.cache,
        Some("hit"),
        "unread faces reuse under the declared granularity"
    );
    assert_eq!(
        second.values.get("main.ct").map(|v| v.render()),
        Some("i32:42".into())
    );
}

#[test]
fn cache_caps_change_invalidates() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let mut d = CteDriver::with_store(&root, "v6-caps").unwrap();

    let plain = sum_deck("return a + b");
    assert_eq!(d.query_step(&plain, None).cache, Some("miss"));

    // Same query, same value — but a granted capability is part of the
    // request identity: a different caps set re-keys and re-executes.
    let capped = plain.replacen(
        "deck cte817/1\n",
        "deck cte817/1\ncaps cte.meta.fields/v1\n",
        1,
    );
    let second = d.query_step(&capped, None);
    assert_eq!(second.status, StepStatus::Ok, "{:?}", second.message);
    assert_eq!(second.cache, Some("miss"), "caps are an identity axis");
    // And back to the plain deck: the original entry still hits.
    assert_eq!(d.query_step(&plain, None).cache, Some("hit"));
}

#[test]
fn cache_meta_negative_dependency_rekeys_identity() {
    // The source-level meta+AV combo is out of the record/scalar profile
    // split (contract-decision r1.2 boundary); the identity-level proof:
    // the manifest (and hence the cache key) folds the meta facts, so a
    // probed-absent field APPEARING re-keys the request.
    let caps = vec!["cte.meta.fields/v1".to_string()];
    let probes = vec![("Pair".to_string(), "zzz".to_string())];
    let input = || SemaInput {
        package: "cte817".into(),
        module: "main".into(),
        source_fp: String::new(),
        fns: vec![InputFn {
            name: "scale".into(),
            params: vec![("n".into(), Ann::I32)],
            ret: Ann::I32,
            body: InputBlock { stmts: vec![] },
        }],
        records: vec![InputRecord {
            name: "Pair".into(),
            fields: vec![("left".into(), Ann::I32), ("right".into(), Ann::Bool)],
            methods: vec![],
        }],
    };
    let i1 = input();
    let m1 = build_manifest(&i1, "scale", &caps, &["Pair".into()], &probes).unwrap();
    let fp1 = manifest_fp(&m1);
    assert!(m1.meta_facts.contains(&"Pair.zzz=false".to_string()));
    assert!(manifest_fresh(&m1, &i1, &caps));

    // The probed-absent field appears: the negative dependency flips, the
    // manifest (and key) change.
    let mut i2 = input();
    i2.records[0].fields.push(("zzz".into(), Ann::I32));
    assert!(!manifest_fresh(&m1, &i2, &caps));
    let m2 = build_manifest(&i2, "scale", &caps, &["Pair".into()], &probes).unwrap();
    assert_ne!(
        fp1,
        manifest_fp(&m2),
        "a different manifest fp = a different cache key"
    );
}

#[test]
fn cache_hit_never_caches_the_consumer_verdict() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let mut d = CteDriver::with_store(&root, "v6-consumer").unwrap();

    let good = sum_deck("return a + b");
    assert_eq!(d.query_step(&good, None).cache, Some("miss"));

    // Same queries/callee — the frozen VALUE hits — but this consumer
    // declares the wrong expected type: the consumer re-admission runs
    // on EVERY hit and FAILS. A cached PASS must not exist.
    let bad = deck_with(
        "fn cte_sum(a int, b int) int {\n    return a + b\n}\n\nfn other() int {\n    return 1\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect bool placeholder 0\n",
    );
    let out = d.query_step(&bad, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(out.code.as_deref(), Some("cte.consumer.type-mismatch"));
    assert_eq!(out.ct_execute, 0, "the VALUE came from the frozen layer");
    // The cache verdict surfaced the hit BEFORE the consumer check —
    // observable via the mismatch happening with zero executions.
    let again = d.query_step(&good, None);
    assert_eq!(again.cache, Some("hit"), "the entry itself is still good");
    assert_eq!(again.status, StepStatus::Ok);
}
