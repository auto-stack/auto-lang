//! PLAN-1028 (F02) permanent regressions: scoped observation restoration across
//! nested same-kind windows, Result::Err normal returns, panic unwinds, and
//! no-window ghost-record immunity (`OBS-01` / `OBS-02`).

use ace_core::body_work::{self, capture_body_work, capture_stage_attempts, BodyWork, Stage};
use ace_core::source::parser::{self, capture_syntax_work};
use ace_core::type_synth::{self, TypeWork};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

fn panic_payload_text(err: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = err.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic>".to_string()
    }
}

#[test]
fn obs_01_nested_same_kind_body_stage_type_and_cross_windows() {
    // Baseline: events emitted outside any window must not appear as ghost records.
    body_work::record_resolve("pre_ghost");
    body_work::record_attempt(Stage::ParseProject, "pre_ghost");
    type_synth::record_synth("pre_ghost");

    // 1. Nested capture_body_work: outer(a) -> inner(b) -> outer(c).
    let ((inner_body, outer_body_res), outer_body) = capture_body_work(|| {
        body_work::record_resolve("outer_a");
        body_work::record_typecheck("outer_a");

        let (inner_ret, inner_map) = capture_body_work(|| {
            body_work::record_plan("inner_b");
            body_work::record_bind("inner_b");
            42_i32
        });

        body_work::record_resolve("outer_c");
        (inner_ret, inner_map)
    });
    assert_eq!(inner_body, 42);
    assert_eq!(
        outer_body_res.keys().cloned().collect::<Vec<_>>(),
        vec!["inner_b".to_string()],
        "inner body window must observe only inner_b"
    );
    assert_eq!(
        outer_body_res["inner_b"],
        BodyWork {
            resolve: 0,
            typecheck: 0,
            plan: 1,
            bind: 1,
        }
    );
    assert_eq!(
        outer_body.get("outer_a").copied(),
        Some(BodyWork {
            resolve: 1,
            typecheck: 1,
            plan: 0,
            bind: 0,
        }),
        "outer pre-inner records must survive inner window"
    );
    assert_eq!(
        outer_body.get("outer_c").copied(),
        Some(BodyWork {
            resolve: 1,
            typecheck: 0,
            plan: 0,
            bind: 0,
        }),
        "outer post-inner records must continue in outer window"
    );
    assert!(
        !outer_body.contains_key("inner_b") && !outer_body.contains_key("pre_ghost"),
        "outer body window must isolate inner_b and pre_ghost: {outer_body:?}"
    );

    // 2. Nested capture_stage_attempts: outer(a) -> inner(b) -> outer(c).
    let (inner_stages, outer_stages) = capture_stage_attempts(|| {
        body_work::record_attempt(Stage::ParseProject, "outer_a");
        let ((), inner) = capture_stage_attempts(|| {
            body_work::record_attempt(Stage::MirLower, "inner_b");
        });
        body_work::record_attempt(Stage::HirVerify, "outer_c");
        inner
    });
    assert_eq!(inner_stages.of("inner_b", Stage::MirLower), 1);
    assert_eq!(inner_stages.of("outer_a", Stage::ParseProject), 0);
    assert_eq!(inner_stages.of("outer_c", Stage::HirVerify), 0);
    assert_eq!(outer_stages.of("outer_a", Stage::ParseProject), 1);
    assert_eq!(outer_stages.of("outer_c", Stage::HirVerify), 1);
    assert_eq!(outer_stages.of("inner_b", Stage::MirLower), 0);
    assert_eq!(outer_stages.of("pre_ghost", Stage::ParseProject), 0);

    // 3. Nested type_synth::capture: outer(a) -> inner(b) -> outer(c).
    let (inner_types, outer_types) = type_synth::capture(|| {
        type_synth::record_synth("outer_a");
        let ((), inner) = type_synth::capture(|| {
            type_synth::record_fact("inner_b");
        });
        type_synth::record_missing("outer_c");
        inner
    });
    assert_eq!(
        inner_types.keys().cloned().collect::<Vec<_>>(),
        vec!["inner_b".to_string()]
    );
    assert_eq!(
        inner_types["inner_b"],
        TypeWork {
            synth: 0,
            fact_checks: 1,
            fact_missing: 0,
        }
    );
    assert_eq!(
        outer_types.get("outer_a").copied(),
        Some(TypeWork {
            synth: 1,
            fact_checks: 0,
            fact_missing: 0,
        })
    );
    assert_eq!(
        outer_types.get("outer_c").copied(),
        Some(TypeWork {
            synth: 0,
            fact_checks: 0,
            fact_missing: 1,
        })
    );
    assert!(
        !outer_types.contains_key("inner_b") && !outer_types.contains_key("pre_ghost"),
        "outer type_synth window must isolate inner_b and pre_ghost: {outer_types:?}"
    );

    // 4. Cross-window independence + body_observer control.
    let observed = Arc::new(Mutex::new(Vec::<(&'static str, String, bool)>::new()));
    let observed_handle = Arc::clone(&observed);
    let observer: body_work::BodyObserver = Arc::new(move |stage, name, produced| {
        observed_handle
            .lock()
            .unwrap()
            .push((stage, name.to_string(), produced));
    });
    let ((((), cross_type), cross_body), cross_stages) =
        body_work::with_body_observer(observer, || {
            capture_stage_attempts(|| {
                capture_body_work(|| {
                    type_synth::capture(|| {
                        let ((), syn) = capture_syntax_work(|| {
                            let _ = parser::parse_program("fn f() int { return 1 }");
                        });
                        assert_eq!(syn.tokenized_ranges.len(), 1);
                        body_work::record_resolve("cross_fn");
                        body_work::record_attempt(Stage::MirVerify, "cross_fn");
                        type_synth::record_synth("cross_fn");
                    })
                })
            })
        });
    assert_eq!(cross_body["cross_fn"].resolve, 1);
    assert_eq!(cross_stages.of("cross_fn", Stage::MirVerify), 1);
    assert_eq!(cross_type["cross_fn"].synth, 1);
    assert!(observed
        .lock()
        .unwrap()
        .iter()
        .any(|(stage, name, produced)| *stage == "resolve" && name == "cross_fn" && !*produced));

    // Post-window emissions must not leak into any subsequent window.
    body_work::record_resolve("post_ghost");
    body_work::record_attempt(Stage::ParseProject, "post_ghost");
    type_synth::record_synth("post_ghost");
    let ((), clean_body) = capture_body_work(|| {});
    let ((), clean_stages) = capture_stage_attempts(|| {});
    let ((), clean_types) = type_synth::capture(|| {});
    assert!(clean_body.is_empty(), "ghost body records: {clean_body:?}");
    assert!(
        clean_stages.by_unit.is_empty(),
        "ghost stage records: {clean_stages:?}"
    );
    assert!(
        clean_types.is_empty(),
        "ghost type records: {clean_types:?}"
    );
}

#[test]
fn obs_02_panic_unwind_and_err_restoration_across_all_windows() {
    // 1. Nested panic unwind + Result::Err for capture_body_work.
    let ((err_inner, panic_msg), outer_body) = capture_body_work(|| {
        body_work::record_resolve("outer_a");
        let panic_res = catch_unwind(AssertUnwindSafe(|| {
            capture_body_work(|| {
                body_work::record_resolve("panicked_b");
                panic!("obs-body-boom");
            })
        }));
        let msg = panic_payload_text(
            panic_res
                .expect_err("original panic must propagate out of capture_body_work")
                .as_ref(),
        );
        let (err_ret, err_map) = capture_body_work(|| -> Result<(), &'static str> {
            body_work::record_plan("err_b");
            Err("expected-body-err")
        });
        body_work::record_bind("outer_c");
        ((err_ret, err_map), msg)
    });
    assert!(
        panic_msg.contains("obs-body-boom"),
        "panic msg: {panic_msg}"
    );
    assert_eq!(err_inner.0, Err("expected-body-err"));
    assert_eq!(
        err_inner.1.keys().cloned().collect::<Vec<_>>(),
        vec!["err_b".to_string()]
    );
    assert_eq!(outer_body.get("outer_a").map(|w| w.resolve), Some(1));
    assert_eq!(outer_body.get("outer_c").map(|w| w.bind), Some(1));
    assert!(
        !outer_body.contains_key("panicked_b") && !outer_body.contains_key("err_b"),
        "outer body window must remain isolated across panic & Err: {outer_body:?}"
    );

    // Top-level panic with no outer window must restore None (no ghost records).
    let top_body_panic = catch_unwind(AssertUnwindSafe(|| {
        capture_body_work(|| {
            body_work::record_resolve("top_panicked_b");
            panic!("obs-body-top-boom");
        })
    }));
    assert!(panic_payload_text(top_body_panic.unwrap_err().as_ref()).contains("obs-body-top-boom"));
    body_work::record_resolve("outside_after_panic");
    let ((), fresh_body) = capture_body_work(|| {
        body_work::record_resolve("clean_d");
    });
    assert_eq!(
        fresh_body.keys().cloned().collect::<Vec<_>>(),
        vec!["clean_d".to_string()],
        "fresh body window after top-level panic must be clean: {fresh_body:?}"
    );

    // 2. Nested panic unwind + Result::Err for capture_stage_attempts.
    let ((err_stage, panic_stage_msg), outer_stages) = capture_stage_attempts(|| {
        body_work::record_attempt(Stage::ParseProject, "outer_a");
        let panic_res = catch_unwind(AssertUnwindSafe(|| {
            capture_stage_attempts(|| {
                body_work::record_attempt(Stage::MirLower, "panicked_b");
                panic!("obs-stage-boom");
            })
        }));
        let msg = panic_payload_text(
            panic_res
                .expect_err("original panic must propagate out of capture_stage_attempts")
                .as_ref(),
        );
        let (err_ret, err_map) = capture_stage_attempts(|| -> Result<(), &'static str> {
            body_work::record_attempt(Stage::AbcEncode, "err_b");
            Err("expected-stage-err")
        });
        body_work::record_attempt(Stage::HirVerify, "outer_c");
        ((err_ret, err_map), msg)
    });
    assert!(
        panic_stage_msg.contains("obs-stage-boom"),
        "panic msg: {panic_stage_msg}"
    );
    assert_eq!(err_stage.0, Err("expected-stage-err"));
    assert_eq!(err_stage.1.of("err_b", Stage::AbcEncode), 1);
    assert_eq!(outer_stages.of("outer_a", Stage::ParseProject), 1);
    assert_eq!(outer_stages.of("outer_c", Stage::HirVerify), 1);
    assert_eq!(outer_stages.of("panicked_b", Stage::MirLower), 0);
    assert_eq!(outer_stages.of("err_b", Stage::AbcEncode), 0);

    let top_stage_panic = catch_unwind(AssertUnwindSafe(|| {
        capture_stage_attempts(|| {
            body_work::record_attempt(Stage::MirLower, "top_panicked_b");
            panic!("obs-stage-top-boom");
        })
    }));
    assert!(
        panic_payload_text(top_stage_panic.unwrap_err().as_ref()).contains("obs-stage-top-boom")
    );
    body_work::record_attempt(Stage::MirLower, "outside_after_panic");
    let ((), fresh_stages) = capture_stage_attempts(|| {
        body_work::record_attempt(Stage::MirLower, "clean_d");
    });
    assert_eq!(fresh_stages.by_unit.len(), 1);
    assert_eq!(fresh_stages.of("clean_d", Stage::MirLower), 1);

    // 3. Nested panic unwind + Result::Err for type_synth::capture.
    let ((err_type, panic_type_msg), outer_types) = type_synth::capture(|| {
        type_synth::record_synth("outer_a");
        let panic_res = catch_unwind(AssertUnwindSafe(|| {
            type_synth::capture(|| {
                type_synth::record_fact("panicked_b");
                panic!("obs-type-boom");
            })
        }));
        let msg = panic_payload_text(
            panic_res
                .expect_err("original panic must propagate out of type_synth::capture")
                .as_ref(),
        );
        let (err_ret, err_map) = type_synth::capture(|| -> Result<(), &'static str> {
            type_synth::record_missing("err_b");
            Err("expected-type-err")
        });
        type_synth::record_fact("outer_c");
        ((err_ret, err_map), msg)
    });
    assert!(
        panic_type_msg.contains("obs-type-boom"),
        "panic msg: {panic_type_msg}"
    );
    assert_eq!(err_type.0, Err("expected-type-err"));
    assert_eq!(
        err_type.1.keys().cloned().collect::<Vec<_>>(),
        vec!["err_b".to_string()]
    );
    assert_eq!(outer_types.get("outer_a").map(|w| w.synth), Some(1));
    assert_eq!(outer_types.get("outer_c").map(|w| w.fact_checks), Some(1));
    assert!(
        !outer_types.contains_key("panicked_b") && !outer_types.contains_key("err_b"),
        "outer type window must remain isolated across panic & Err: {outer_types:?}"
    );

    let top_type_panic = catch_unwind(AssertUnwindSafe(|| {
        type_synth::capture(|| {
            type_synth::record_synth("top_panicked_b");
            panic!("obs-type-top-boom");
        })
    }));
    assert!(panic_payload_text(top_type_panic.unwrap_err().as_ref()).contains("obs-type-top-boom"));
    type_synth::record_synth("outside_after_panic");
    let ((), fresh_types) = type_synth::capture(|| {
        type_synth::record_synth("clean_d");
    });
    assert_eq!(
        fresh_types.keys().cloned().collect::<Vec<_>>(),
        vec!["clean_d".to_string()],
        "fresh type window after top-level panic must be clean: {fresh_types:?}"
    );
}
