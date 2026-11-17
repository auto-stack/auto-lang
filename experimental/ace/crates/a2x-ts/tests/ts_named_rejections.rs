//! PLAN-976 review F-R1: permanent negative controls for the plan §5 T-01 D
//! named-rejection table, exercised through the REAL frontend → view →
//! provider path (no legacy fallback may serve them):
//!
//! * `a2ts.sched-unsupported` — a scheduler-profile task source (the frozen
//!   PLAN-968 corpus fixture) is refused by NAME, never silently lowered to
//!   broken JS async;
//! * `a2ts.depth-budget` — an expression nested past `MAX_STRUCTURE_DEPTH`
//!   (64) is refused by NAME (the same constant/code discipline as a2py).
//!
//! The other two §5 D families are inspection-verified in the review record
//! (foreign: `provider.rs` `validate_view` rejects before any emission;
//! closure owned-Str capture: `lower/mod.rs` `a2ts.closure-owned-capture`)
//! and stay named limitations until a cheap in-repo fixture exists.

use ace_a2x_core::{Family, TargetProvider};
use ace_a2x_ts::{ScriptLang, TypeScriptTargetProvider};

fn view_of_scheduler(src: &str) -> ace_a2x_core::SemanticView<'static> {
    let compiled = ace_core::source::compile_scheduler_source(src, "a2ts-rej", &[])
        .expect("scheduler frontend diagnostics");
    let checked = Box::leak(Box::new(compiled.checked));
    ace_a2x_core::SemanticView::from_checked(checked).expect("view admission")
}

#[test]
fn tsrej_01_scheduler_intrinsic_is_a_named_rejection() {
    // The frozen PLAN-968 corpus fixture (task actor + await/ask surface):
    // the script target has no scheduler runtime — refusing by name is the
    // contract, a silent degradation or a fallback emitter is the bug this
    // control exists to catch.
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/ace-m2/scheduler/src/ask_reply.at");
    let src = std::fs::read_to_string(&fixture)
        .unwrap_or_else(|e| panic!("read scheduler fixture {}: {e}", fixture.display()));
    let view = view_of_scheduler(&src);
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let err = TypeScriptTargetProvider
        .produce_fragments(&view, &fns, Family::Collections)
        .expect_err("scheduler surface must be rejected, not emitted");
    assert_eq!(err.code, "a2ts.sched-unsupported", "detail: {}", err.detail);

    // Both legs reject identically (the shared lowering must not diverge).
    let err = ace_a2x_ts::JavaScriptTargetProvider
        .produce_fragments(&view, &fns, Family::Collections)
        .expect_err("JS leg must reject the same surface");
    assert_eq!(err.code, "a2ts.sched-unsupported");
}

#[test]
fn tsrej_02_depth_budget_is_a_named_rejection() {
    // 80 RIGHT-nested binaries > MAX_STRUCTURE_DEPTH (64): refused by name
    // on both legs, mirroring the a2py constant/code discipline. (Redundant
    // parentheses are folded by the frontend — the budget counts REAL
    // expression nesting in the lowered view.)
    let mut expr = "1".to_string();
    for _ in 0..80 {
        expr = format!("1 + ({expr})");
    }
    let src = format!("fn main() int {{ return {expr} }}");
    let compiled = ace_core::source::compile_source(&src, "a2ts-rej", &[])
        .expect("frontend accepts deep nesting (budget is the LOWERER's)");
    let checked = Box::leak(Box::new(compiled.checked));
    let view = ace_a2x_core::SemanticView::from_checked(checked).expect("view admission");
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let ts = TypeScriptTargetProvider;
    let js = ace_a2x_ts::JavaScriptTargetProvider;
    for provider in [&ts as &dyn TargetProvider, &js as &dyn TargetProvider] {
        let err = provider
            .produce_fragments(&view, &fns, Family::Scalar)
            .expect_err("over-deep nesting must be rejected");
        assert_eq!(err.code, "a2ts.depth-budget", "detail: {}", err.detail);
    }
    // The boundary is real NESTING, not program size: 80 sibling statements
    // (each shallow) stay inside the same budget. (Chained `a + b + c`
    // builds a left-nested tree in HIR, so it genuinely consumes depth.)
    let mut body = String::new();
    for i in 0..80 {
        body.push_str(&format!("    var v{i} = {i}\n"));
    }
    body.push_str("    return v79");
    let flat = format!("fn main() int {{\n{body}\n}}");
    let compiled =
        ace_core::source::compile_source(&flat, "a2ts-rej", &[]).expect("wide source compiles");
    let checked = Box::leak(Box::new(compiled.checked));
    let view = ace_a2x_core::SemanticView::from_checked(checked).expect("view admission");
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    assert!(
        TypeScriptTargetProvider
            .produce_fragments(&view, &fns, Family::Scalar)
            .is_ok(),
        "shallow sibling statements are inside the depth budget"
    );
    let _ = ScriptLang::TypeScript;
}
