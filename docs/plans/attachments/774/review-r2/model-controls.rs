//! Targeted model replacement controls for PLAN-774 delivery readiness.
//! Build: rustc --test this-file -o <774-tree>/experimental/ac-core/probes/data-resources/target/review-r2-model.exe
#![allow(dead_code)]
#[path = "D:/autostack/.wt/lang-774/auto-lang/experimental/ac-core/probes/data-resources/src/rules.rs"]
mod rules;
use rules::{run_script, Op};

#[test]
fn borrowed_param_return_is_accepted() {
    assert!(run_script(&[Op::ParamBorrowed(0), Op::ReturnViewParam(0)]).rejection.is_none());
}

#[test]
fn moved_in_param_view_return_is_rejected() {
    assert_eq!(run_script(&[Op::ParamOwned(0), Op::ReturnViewParam(0)]).rejection.unwrap().code, "verify.borrow-escape");
}

#[test]
fn borrowed_param_view_region_can_end_then_return() {
    // D-04's view region ends without invalidating the caller-owned parameter.
    let result = run_script(&[Op::ParamBorrowed(0), Op::View(0), Op::EndView(0), Op::ReturnViewParam(0)]);
    assert!(result.rejection.is_none());
}
