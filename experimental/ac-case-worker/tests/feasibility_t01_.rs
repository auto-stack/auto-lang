//! PLAN-788 T-01 whole-path feasibility gate (plan §8 T-01), run from the
//! worker crate's own seat — the consumer path every later worker request
//! will take. Proves on the real toolchain:
//!
//! 1. a core source compiles Checked → COFF → PE and runs natively with its
//!    exit-code oracle (`t01_core_source_to_pe_and_run`);
//! 2. a raw-HIR negative is a TYPED rejection (stage + code + span), never
//!    a lossy String error (`t01_raw_hir_negative_is_typed_rejection`);
//! 3. the 783 unified store yields a fresh-then-hit counter trace the worker
//!    observes through the real driver (`t01_fresh_then_hit_trace`).
//!
//! Windows-only, mirroring the ac-core/probe native gates. Non-Windows CI
//! sees the whole file skip (required-native stays not-satisfied there, it
//! is never faked green — plan §5.3).

#![cfg(windows)]

use ac_artifact_probe::driver::{vertical_and_run, VerticalCfg};
use ac_artifact_probe::store::ManagedStore;
use std::path::{Path, PathBuf};

fn scratch(family: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ac788-t01-{}", family));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn ac_fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ac-core/fixtures")
        .join(rel)
}

/// (1) Real source → PE → native run. p04 is the canonical while-retest
/// fixture whose body edit ("3改4") is the plan §5.5 representative native
/// turnaround oracle; here it only needs its fresh exit-3 oracle.
#[test]
fn t01_core_source_to_pe_and_run() {
    let work = scratch("source-pe-run");
    let src = std::fs::read_to_string(ac_fixture("source/valid/p04-while-retest.at")).unwrap();

    let compiled = auto_ac_prototype::source::compile_source(&src, "main", &[])
        .expect("p04 core source must compile through the real frontend");
    assert_eq!(compiled.profile, "auto.source.core-i32.draft");
    assert!(
        !compiled.atom_text.is_empty(),
        "canonical atom text emitted"
    );

    let sel = auto_ac_prototype::native::find_entry(&compiled.checked, "d_main")
        .expect("d_main entry must resolve");
    let obj_bytes = auto_ac_prototype::native::lower_object(
        &compiled.checked,
        &sel,
        &auto_ac_prototype::native::RuntimeInjection::default(),
    )
    .expect("lowering must produce a COFF object");
    assert!(
        obj_bytes.len() > 64,
        "real COFF object, {} bytes",
        obj_bytes.len()
    );
    assert_eq!(
        &obj_bytes[0..2],
        b"\x64\x86",
        "COFF machine image signature"
    );

    let tmp_obj = work.join("p04.obj.tmp-788");
    std::fs::write(&tmp_obj, &obj_bytes).unwrap();
    let lld = auto_ac_prototype::link::find_rust_lld().expect("rust-lld discovered");
    let sdk = auto_ac_prototype::link::find_sdk_um_dir().expect("Windows SDK um/x64 discovered");
    let exe_path = work.join("p04.exe");
    let staged = auto_ac_prototype::link::link_object_staged(
        &lld,
        &sdk,
        &tmp_obj,
        &exe_path,
        auto_ac_prototype::native::START_SYMBOL,
        &[],
    )
    .expect("real staged link");
    let receipt =
        auto_ac_prototype::link::publish_artifacts(&staged, &tmp_obj, &work.join("p04.obj"))
            .expect("publish transaction");
    assert!(receipt.exe.is_file(), "published exe exists");

    let exe_bytes = std::fs::read(&receipt.exe).unwrap();
    assert_eq!(&exe_bytes[0..2], b"MZ", "published artifact is a PE image");

    let code = auto_ac_prototype::link::run_exe(&receipt.exe).expect("native run completes");
    assert_eq!(code, 3, "p04 exit-code oracle (while retest leaves i == 3)");
}

/// (2) Typed raw-HIR rejection. `dangling-expr-ref` binds through the real
/// descriptor binder and is refused with a structured bind-stage diagnostic
/// whose span points at the offending reference — the shape v2
/// `ac/check` must surface verbatim (plan §5.2), not a rendered String.
#[test]
fn t01_raw_hir_negative_is_typed_rejection() {
    let src = std::fs::read_to_string(ac_fixture("invalid/dangling-expr-ref.atom")).unwrap();
    let diags = auto_ac_prototype::bind_document(&src)
        .expect_err("dangling-expr-ref must be rejected by the real binder");
    assert!(
        diags.iter().any(|d| d.code == "bind.dangling-ref"),
        "typed code bind.dangling-ref expected, got {:#?}",
        diags
    );
    let d = diags
        .iter()
        .find(|d| d.code == "bind.dangling-ref")
        .unwrap();
    assert_eq!(d.stage, auto_ac_prototype::atom_text::Stage::Bind);
    let slice = src.get(d.span.start..d.span.end).expect("span in range");
    assert!(slice.contains("e_missing"), "span points at `{}`", slice);
    // render() exists for humans only; the typed fields above are the oracle.
    let rendered = d.render("case.atom", &src);
    assert!(rendered.contains("bind.dangling-ref"));
}

/// (3) Minimal fresh/hit trace through the 783 store. The worker observes
/// real counters: full pipeline on the first request, object hit (frontend
/// and lowering skipped, link + run still real) on the second — the exact
/// resume semantics plan §5.4 builds its cache assertions on.
#[test]
fn t01_fresh_then_hit_trace() {
    let root = scratch("fresh-hit");
    let store_root = root.join("store");
    let work = root.join("work");
    std::fs::create_dir_all(&work).unwrap();
    let src = ac_fixture("source/valid/p04-while-retest.at");

    let store = ManagedStore::open(&store_root).unwrap();
    let fresh = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "t01".into(),
            source: src.clone(),
            work_dir: work.clone(),
            ..Default::default()
        },
    )
    .expect("fresh vertical");
    assert_eq!(fresh.exit_code, Some(3), "fresh run oracle");
    assert_eq!(
        (
            fresh.counters.parse,
            fresh.counters.verify,
            fresh.counters.lower,
            fresh.counters.link,
            fresh.counters.run
        ),
        (1, 1, 1, 1, 1),
        "fresh: every stage ran once"
    );
    assert_eq!(fresh.hits.object, "lowered");

    let hot = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "t01".into(),
            source: src,
            work_dir: work.clone(),
            ..Default::default()
        },
    )
    .expect("hot vertical");
    assert_eq!(hot.exit_code, Some(3), "hot run oracle re-observed");
    assert_eq!(hot.hits.object, "hit", "second request is an object hit");
    assert_eq!(
        (
            hot.counters.parse,
            hot.counters.verify,
            hot.counters.lower,
            hot.counters.link,
            hot.counters.run
        ),
        (0, 0, 0, 1, 1),
        "hit: frontend/verify/lower skipped, link+run still real"
    );
    assert!(hot.total_ms > 0, "driver reports wall time");
}
