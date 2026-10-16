//! PLAN-796 T-03 differential cross-check (`infer_core_` filter):
//! the NEW limited inference vs the EXISTING core pipeline
//! (parse → resolve → typecheck) on the same sources (V-02).
//!
//! The two run independently — the new solver never calls the old
//! typecheck and relabels its output. Accept/reject must agree, and on
//! accept the inferred local binding types must match the resolver's
//! registered locals (i32/bool surface).

use ac_sema_core::adapter::{old_pipeline_verdict, parse_into_input};
use ac_sema_core::{InferOutcome, InferSession};
use auto_ac_prototype::source::SourceProfile;

fn profile() -> SourceProfile {
    SourceProfile::Core
}

/// One differential case: source + expected agreement.
fn diff_case(name: &str, src: &str) {
    let input = match parse_into_input(src, "diff", name, profile()) {
        Ok(i) => i,
        Err(e) => {
            // parse-level rejection: the OLD pipeline rejects at parse;
            // the NEW probe never sees a program (no partial AST use)
            match old_pipeline_verdict(src, profile()) {
                ac_sema_core::adapter::OldVerdict::Rejected(_) => {}
                v => panic!("{name}: parse failed for new ({e}) but old said {v:?}"),
            }
            return;
        }
    };
    // infer the LAST function (main if present)
    let target = input
        .fns
        .iter()
        .rev()
        .find(|f| f.name == "main")
        .or_else(|| input.fns.last())
        .map(|f| f.name.clone())
        .expect("at least one function");
    let new_outcome = InferSession::new(&input).infer_function(&target);
    let old_verdict = old_pipeline_verdict(src, profile());
    match (&new_outcome, &old_verdict) {
        (InferOutcome::Ok(new_sum), ac_sema_core::adapter::OldVerdict::Accepted(old)) => {
            // binding-type agreement on the target function, IN
            // PROGRAM ORDER (shadowed names correspond by declaration
            // order, not by set identity)
            let old_fn = old.fns.iter().find(|f| f.name == target).expect("target in old");
            let new_bindings: Vec<(String, &str)> = new_sum
                .facts
                .bindings
                .iter()
                .map(|(n, t)| {
                    (
                        n.clone(),
                        match t {
                            ac_sema_contract::TypeDesc::I32 => "i32",
                            ac_sema_contract::TypeDesc::Bool => "bool",
                            _ => "other",
                        },
                    )
                })
                .collect();
            let old_bindings: Vec<(String, &str)> = old_fn
                .locals
                .iter()
                .filter(|(_, t)| matches!(*t, "i32" | "bool"))
                .cloned()
                .collect();
            assert_eq!(
                new_bindings, old_bindings,
                "{name}: binding sequence (program order) must match the old resolver's locals"
            );
        }
        (InferOutcome::Diags(_), ac_sema_core::adapter::OldVerdict::Rejected(_)) => {}
        (InferOutcome::Ok(_), ac_sema_core::adapter::OldVerdict::Rejected(why)) => {
            panic!("{name}: new solver accepted but old pipeline rejected: {why}")
        }
        (InferOutcome::Diags(sum), ac_sema_core::adapter::OldVerdict::Accepted(_)) => {
            panic!(
                "{name}: old pipeline accepted but new solver diagnosed: {:?}",
                sum.facts.diagnostics
            )
        }
        (other, _) => panic!("{name}: infrastructure outcome leaked into a verdict: {other:?}"),
    }
}

#[test]
fn infer_core_parity_hand_corpus() {
    // valid: arithmetic, calls, control flow
    diff_case(
        "ok-arith",
        "fn main() int { let a = 2\nvar b = a * 3\nreturn a + b }",
    );
    diff_case(
        "ok-forward-recursion",
        "fn even(n int) bool { return odd(n - 1) }\nfn odd(n int) bool { return even(n - 1) }\nfn main() bool { return even(4) }",
    );
    diff_case(
        "ok-shadowing",
        "fn main() bool { let x = 1\nif x < 2 { let x = true\nreturn x }\nreturn x < 2 }",
    );
    diff_case(
        "ok-while",
        "fn main() int { var i = 0\nvar acc = 1\nwhile acc < 100 { acc = acc * 2\ni = i + 1 }\nreturn i }",
    );
    diff_case(
        "ok-unannotated-chain",
        "fn f(a int) int { return a + 1 }\nfn main() int { let x = f(1)\nlet y = f(x)\nreturn y }",
    );
    // invalid: annotation vs initializer
    diff_case("bad-let-ann", "fn main() int { let x int = true\nreturn 1 }");
    // invalid: cond not bool
    diff_case("bad-cond", "fn main() int { if 1 { return 1 }\nreturn 0 }");
    diff_case("bad-while-cond", "fn main() int { while 2 { }\nreturn 0 }");
    // invalid: mixed ==
    diff_case("bad-eq-mixed", "fn main() bool { return 1 == true }");
    // invalid: unknown callee / unknown local
    diff_case("bad-unknown-callee", "fn main() int { return g(1) }");
    diff_case("bad-unknown-local", "fn main() int { return x }");
    // invalid: argument type mismatch
    diff_case(
        "bad-arg",
        "fn f(a int) int { return a }\nfn main() int { return f(true) }",
    );
    // invalid: return type mismatch
    diff_case("bad-return", "fn main() int { return true }");
    // invalid: assignment value mismatch
    diff_case(
        "bad-assign",
        "fn main() int { var x = 1\nx = true\nreturn x }",
    );
}

/// Deterministic volume: generated WELL-TYPED programs (by
/// construction) both accept; one injected bool literal makes both
/// reject. 120 programs, fixed LCG seed — no flake by construction.
#[test]
fn infer_core_parity_generated_volume() {
    let mut seed: u64 = 0x796_796;
    let mut rng = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as u32
    };
    for k in 0..120u32 {
        // 3 helpers + main; helper_k(i32)->i32 built from typed expr tree
        let mut src = String::new();
        for h in 0..3u32 {
            let body = gen_i32_expr(&mut rng, 3);
            src.push_str(&format!("fn h{h}(a int) int {{ return {body} }}\n"));
        }
        let main_body = format!("h{}({})", rng() % 3, gen_i32_expr(&mut rng, 2));
        src.push_str(&format!("fn main() int {{ return {main_body} }}\n"));
        diff_case(&format!("gen-ok-{k}"), &src);

        // inject a bool into an i32 hole: `hX(true)`
        let bad = format!(
            "fn h0(a int) int {{ return a + 1 }}\nfn main() int {{ return h0(true) }}\n"
        );
        diff_case(&format!("gen-bad-{k}"), &bad);
    }
}

/// Build a well-typed i32 expression of bounded depth (int literals,
/// `a`, + * ( ) by construction).
fn gen_i32_expr(rng: &mut impl FnMut() -> u32, depth: u32) -> String {
    if depth == 0 || rng() % 3 == 0 {
        return match rng() % 4 {
            0 => "a".to_string(),
            1 => format!("{}", rng() % 97),
            2 => format!("{}", rng() % 53 + 1),
            _ => format!("-{}", rng() % 31),
        };
    }
    let lhs = gen_i32_expr(rng, depth - 1);
    let rhs = gen_i32_expr(rng, depth - 1);
    match rng() % 2 {
        0 => format!("({lhs} + {rhs})"),
        _ => format!("({lhs} * {rhs})"),
    }
}

/// Expected-type propagation: the diagnostic lands on the LITERAL, not
/// just on the let statement (期望类型传递的反例定位).
#[test]
fn infer_core_parity_expected_type_span() {
    let src = "fn main() int { let x int = true\nreturn 0 }";
    let input = parse_into_input(src, "diff", "span", profile()).unwrap();
    let outcome = InferSession::new(&input).infer_function("main");
    match outcome {
        InferOutcome::Diags(sum) => {
            let d = sum
                .facts
                .diagnostics
                .iter()
                .find(|d| d.code == "type-mismatch")
                .expect("a type-mismatch diagnostic");
            // the diagnostic must land on the `true` LITERAL (dynamic
            // position — expected-type propagation localizes errors)
            let lit_at = src.find("true").expect("literal present");
            assert_eq!(
                (d.start as usize, d.end as usize),
                (lit_at, lit_at + 4),
                "diagnostic must point at the literal"
            );
        }
        other => panic!("expected diags, got {other:?}"),
    }
}

/// Out-of-surface constructs (text profile) project to explicit
/// unsupported nodes — never a silent mistype — while the old pipeline
/// (which owns the text surface) accepts them; the limited profile
/// reports unsupported instead of claiming parity it cannot check.
#[test]
fn infer_core_parity_text_constructs_unsupported_not_mistyped() {
    let src = "fn main() int { let s = \"hello\"\nreturn 1 }";
    let input = parse_into_input(src, "diff", "text", SourceProfile::TextRecord).unwrap();
    match InferSession::new(&input).infer_function("main") {
        InferOutcome::Diags(sum) => {
            assert!(
                sum.facts.diagnostics.iter().any(|d| d.code == "unsupported-construct"),
                "str literal must surface as unsupported-construct"
            );
        }
        other => panic!("expected diags with unsupported-construct, got {other:?}"),
    }
}
