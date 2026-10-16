//! The AutoDown sources/oracles of `test/cases/ac-av/` executed through the
//! Rust-directed path (PLAN-798 T-01: formal runner selection arrives with
//! T-07 after D-796/D-788; these are the SAME inputs and oracles the formal
//! recipe will consume — no weaker oracle here).

use auto_av_probe::driver::Driver;

fn case_src(rel: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("test/cases/ac-av")
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("{}: {}", rel, e))
}

#[test]
fn av_binding_case_vertical_two_fn() {
    let r = run(&case_src("av/core/vertical_two_fn.at"), &[]);
    assert_eq!(r.value_i32(), Some(0), "oracle: success exit 0");
}

#[test]
fn av_binding_case_shortcircuit_latent_trap() {
    let r = run(&case_src("av/core/shortcircuit_latent_trap.at"), &[]);
    assert_eq!(r.value_i32(), Some(0), "oracle: success exit 0");
}

#[test]
fn av_binding_case_trap_rem_overflow() {
    let r = run(&case_src("av/core/trap_rem_overflow.at"), &[]);
    match &r.av {
        Some(auto_av_core::contract::ExecOutcome::Trap { kind, .. }) => {
            assert_eq!(kind.code(), "av.trap.rem-overflow", "oracle: rem-overflow trap");
        }
        other => panic!("oracle: run-stage trap, got {:?}", other),
    }
}

#[test]
fn av_binding_case_trace_order() {
    let r = run(
        &case_src("av/core/trace_order.at"),
        &["hir.test.trace.v1".to_string()],
    );
    assert_eq!(r.value_i32(), Some(1), "oracle: success exit 1");
    use auto_av_core::mir::TraceEvent::*;
    assert_eq!(r.trace, vec![MarkA, MarkB, MarkA], "oracle: trace a,b,a");
}

#[test]
fn av_binding_case_front_door_nesting() {
    let src = case_src("av/resources/nesting_over_budget.at");
    let mut d = Driver::ephemeral();
    let r = d.request(&src, "d_main", &[]);
    let diag = r.diag.expect("oracle: admission-stage rejection");
    assert_eq!(diag.stage, "admission");
    assert_eq!(diag.code, "av.resource.frontend-nesting");
}

fn run(src: &str, caps: &[String]) -> auto_av_probe::driver::RequestOutcome {
    let mut d = Driver::ephemeral();
    let r = d.request(src, "d_main", caps);
    assert!(r.diag.is_none(), "case must reach execution: {:?}", r.diag);
    r
}
