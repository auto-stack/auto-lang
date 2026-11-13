//! PLAN-1028 (F02) real native cold-start consumer regressions (`OBS-01` / `OBS-02`).

#[path = "../common/mod.rs"]
mod common;

use ace_core::body_work::{self, capture_body_work, capture_stage_attempts, Stage};
use ace_core::type_synth;
use ace_native::closures_incremental::ClosuresNativeSession;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn corpus(name: &str) -> String {
    let path = common::repo_root()
        .join("crates/auto-lang/test/cases/ace-m1/src")
        .join(format!("{name}.at"));
    std::fs::read_to_string(&path).expect("corpus source")
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "plan1028-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn obs_02_native_cold_start_nested_and_post_panic_observation() {
    assert!(
        common::tools_available(),
        "native tools are required; no skip pass"
    );
    let source = corpus("K02");

    // 1. Real ClosuresNativeSession::new (which internally opens capture_body_work)
    // nested inside an outer capture_body_work + capture_stage_attempts window.
    let dir_nested = temp_dir("nested");
    let ((cold_res, outer_body), outer_stages) = capture_stage_attempts(|| {
        capture_body_work(|| {
            body_work::record_resolve("outer_a");
            body_work::record_attempt(Stage::ParseProject, "outer_a");
            let res = ClosuresNativeSession::new(&source, "main", &dir_nested, None);
            body_work::record_resolve("outer_c");
            body_work::record_attempt(Stage::HirVerify, "outer_c");
            res
        })
    });
    let (_session, run) = cold_res.expect("K02 cold start succeeds");
    assert_eq!(run.exit_code, Some(42));
    assert!(
        run.work.bodies.contains_key("main"),
        "inner cold_bodies must observe main: {:?}",
        run.work.bodies
    );
    assert_eq!(
        outer_body.get("outer_a").map(|w| w.resolve),
        Some(1),
        "outer_a lost across ClosuresNativeSession::new: {outer_body:?}"
    );
    assert_eq!(
        outer_body.get("outer_c").map(|w| w.resolve),
        Some(1),
        "outer_c lost after ClosuresNativeSession::new: {outer_body:?}"
    );
    assert!(
        !outer_body.contains_key("main"),
        "inner cold_bodies leaked into outer window: {outer_body:?}"
    );
    assert_eq!(outer_stages.of("outer_a", Stage::ParseProject), 1);
    assert_eq!(outer_stages.of("outer_c", Stage::HirVerify), 1);
    assert!(
        outer_stages.of("d_main", Stage::MirLower) >= 1,
        "outer stage window must still see cold_units MirLower: {outer_stages:?}"
    );
    let _ = std::fs::remove_dir_all(&dir_nested);

    // 2. Catch unwind on panicking inner capture windows inside outer windows,
    // then run a real ClosuresNativeSession::new + Result::Err control.
    let dir_recover = temp_dir("recover");
    let dir_err = temp_dir("err");
    let ((((panic_res, cold_after, err_res), outer_types_2), outer_body_map), outer_stages_2) =
        capture_stage_attempts(|| {
            capture_body_work(|| {
                type_synth::capture(|| {
                    body_work::record_resolve("outer_before");
                    body_work::record_attempt(Stage::ParseProject, "outer_before");
                    type_synth::record_synth("outer_before");

                    let pan = catch_unwind(AssertUnwindSafe(|| {
                        capture_stage_attempts(|| {
                            capture_body_work(|| {
                                type_synth::capture(|| {
                                    body_work::record_resolve("panicked_b");
                                    body_work::record_attempt(Stage::MirLower, "panicked_b");
                                    type_synth::record_fact("panicked_b");
                                    panic!("native-obs-boom");
                                })
                            })
                        })
                    }));
                    assert!(pan.is_err(), "panic must remain visible");

                    let cold = ClosuresNativeSession::new(&source, "main", &dir_recover, None);
                    let bad = ClosuresNativeSession::new(
                        "fn main() int { return missing_ident }",
                        "main",
                        &dir_err,
                        None,
                    );

                    body_work::record_resolve("outer_after");
                    body_work::record_attempt(Stage::HirVerify, "outer_after");
                    type_synth::record_missing("outer_after");
                    (pan, cold, bad)
                })
            })
        });
    assert!(panic_res.is_err());
    let (_s2, run2) = cold_after.expect("cold start after caught panic succeeds");
    assert_eq!(run2.exit_code, Some(42));
    assert!(run2.work.bodies.contains_key("main"));
    assert!(!run2.work.bodies.contains_key("panicked_b"));
    assert!(err_res.is_err(), "invalid source must return Err normally");

    assert_eq!(
        outer_body_map.get("outer_before").map(|w| w.resolve),
        Some(1)
    );
    assert_eq!(
        outer_body_map.get("outer_after").map(|w| w.resolve),
        Some(1)
    );
    assert!(!outer_body_map.contains_key("panicked_b"));
    assert_eq!(outer_stages_2.of("outer_before", Stage::ParseProject), 1);
    assert_eq!(outer_stages_2.of("outer_after", Stage::HirVerify), 1);
    assert_eq!(outer_stages_2.of("panicked_b", Stage::MirLower), 0);
    assert_eq!(outer_types_2.get("outer_before").map(|w| w.synth), Some(1));
    assert_eq!(
        outer_types_2.get("outer_after").map(|w| w.fact_missing),
        Some(1)
    );
    assert!(!outer_types_2.contains_key("panicked_b"));

    let _ = std::fs::remove_dir_all(&dir_recover);
    let _ = std::fs::remove_dir_all(&dir_err);
}
