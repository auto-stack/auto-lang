//! PLAN-788 R1 (F08/F10) probe-level controls: the compiler recipe
//! component in the cache keys, and the observed stage spans — through the
//! real driver on the real store.

#![cfg(windows)]

mod common;

use ac_artifact_probe::driver::{
    vertical_and_run_observed, vertical_observed, DriverCacheMode, VerticalCfg,
};
use ac_artifact_probe::store::ManagedStore;
use common::fixture_path;

#[test]
fn r1_compiler_recipe_change_invalidates_hir_and_object() {
    let root = common::scratch("r1-compiler-fp");
    let store_root = root.join("store");
    let work = root.join("work");
    std::fs::create_dir_all(&work).unwrap();
    let src = fixture_path("p04-while-retest.at");

    let cfg_for = |fp: &str| VerticalCfg {
        project: "r1".into(),
        source: src.clone(),
        logical_name: "main".into(),
        toolchain_fp: "lld-v1".into(),
        compiler_fp: Some(fp.to_string()),
        work_dir: work.clone(),
        ..Default::default()
    };

    let store = ManagedStore::open(&store_root).unwrap();
    let a =
        vertical_and_run_observed(&store, &cfg_for("recipe-a"), DriverCacheMode::Reuse).unwrap();
    assert_eq!(a.outcome.counters.parse, 1, "first request computes");
    assert_eq!(a.outcome.exit_code, Some(3));

    // Same recipe: object hit (frontend skipped, link+run real).
    let b =
        vertical_and_run_observed(&store, &cfg_for("recipe-a"), DriverCacheMode::Reuse).unwrap();
    assert_eq!(b.outcome.counters.parse, 0);
    assert_eq!(b.outcome.hits.object, "hit");

    // DIFFERENT compiler recipe: the stored HIR/object keys no longer
    // match — full recompute (a stale artifact from another compiler is
    // never resumed).
    let c =
        vertical_and_run_observed(&store, &cfg_for("recipe-b"), DriverCacheMode::Reuse).unwrap();
    assert_eq!(c.outcome.counters.parse, 1, "recipe change must miss");
    assert_eq!(c.outcome.counters.lower, 1);
    assert_eq!(c.outcome.hits.object, "lowered");
    assert_eq!(c.outcome.exit_code, Some(3));
}

#[test]
fn r1_observed_spans_carry_real_frontend_boundaries() {
    let root = common::scratch("r1-spans");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let work = root.join("work");
    std::fs::create_dir_all(&work).unwrap();
    let observed = vertical_observed(
        &store,
        &VerticalCfg {
            source: fixture_path("p04-while-retest.at"),
            work_dir: work,
            ..Default::default()
        },
        DriverCacheMode::Fresh,
    )
    .unwrap();
    let stages: Vec<&str> = observed.stage_ms.iter().map(|(s, _)| *s).collect();
    for expect in [
        "parse",
        "resolve",
        "typecheck",
        "emit",
        "bind",
        "verify",
        "lower",
        "link",
        "total",
    ] {
        assert!(
            stages.contains(&expect),
            "observed spans {:?} missing {}",
            stages,
            expect
        );
    }
    // A cache read on a later run is its own span (not folded into build).
    let work2 = root.join("work2");
    std::fs::create_dir_all(&work2).unwrap();
    let observed = vertical_observed(
        &store,
        &VerticalCfg {
            source: fixture_path("p04-while-retest.at"),
            work_dir: work2,
            ..Default::default()
        },
        DriverCacheMode::Reuse,
    )
    .unwrap();
    let stages: Vec<&str> = observed.stage_ms.iter().map(|(s, _)| *s).collect();
    assert!(
        stages.contains(&"cache-read"),
        "cache-read span: {:?}",
        stages
    );
}
