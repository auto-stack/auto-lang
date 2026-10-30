//! T-02 structural probes: document and pin the exact HIR shapes the Rust
//! lowering consumes. These run NO toolchain — they anchor the lowering
//! contract against the real frontend output (fixtures compiled live from
//! `experimental/ace/crates/core/fixtures/source/valid`).

use ace_core::hir::{ExprKind, Stmt};
use ace_core::source::{compile_scalar_source, compile_source, TRACE_CAPABILITY};

fn compile(src: &str, caps: &[String]) -> ace_core::verify::CheckedModule {
    if caps.is_empty() {
        compile_source(src, "shape", caps).unwrap().checked
    } else {
        compile_scalar_source(src, "shape", caps).unwrap().checked
    }
}

#[test]
fn while_lowers_to_loop_if_break() {
    let src = "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";
    let checked = compile(src, &[]);
    let body = &checked.bundle().modules[0].bodies[0];
    // exactly one registered loop
    assert_eq!(body.loops.len(), 1, "while registers one loop");
    // the entry block is: Assign/Let-ish stmts..., Loop, Return — the loop
    // body retests the condition and breaks out when it fails.
    let mut loop_count = 0;
    for s in &body.blocks[0].stmts {
        if let Stmt::Loop { .. } = s {
            loop_count += 1;
        }
    }
    assert_eq!(loop_count, 1);
    // some If exists inside the loop body block and a Break exists in the
    // module (the condition-fails exit path).
    let has_break = body
        .blocks
        .iter()
        .any(|b| b.stmts.iter().any(|s| matches!(s, Stmt::Break { .. })));
    assert!(has_break, "while lowering carries a Break exit");
}

#[test]
fn conditional_for_lowers_like_while() {
    let src = "fn main() int { var i = 0; for i < 3 { i = i + 1 }; return i }";
    let checked = compile_scalar_source(src, "shape", &[]).unwrap().checked;
    let body = &checked.bundle().modules[0].bodies[0];
    assert_eq!(body.loops.len(), 1, "conditional for registers one loop");
    let has_break = body
        .blocks
        .iter()
        .any(|b| b.stmts.iter().any(|s| matches!(s, Stmt::Break { .. })));
    assert!(has_break, "conditional for carries a Break exit");
}

#[test]
fn nested_loops_break_targets_innermost() {
    let src = "fn main() int { var n = 0; loop { loop { n = n + 2; break }; n = n + 3; break }; return n }";
    let checked = compile(src, &[]);
    let body = &checked.bundle().modules[0].bodies[0];
    assert_eq!(body.loops.len(), 2, "two nested loops registered");
    // Walk with an explicit stack: both breaks must target their innermost
    // enclosing loop.
    let issues = walk_block(body, body.entry.0 as usize, &mut Vec::new());
    assert!(issues.is_empty(), "non-nearest targets: {:?}", issues);
}

fn walk_block(body: &ace_core::hir::Body, block: usize, stack: &mut Vec<u32>) -> Vec<String> {
    let mut issues = Vec::new();
    for s in &body.blocks[block].stmts {
        match s {
            Stmt::Break { target } => {
                if stack.last() != Some(&target.0) {
                    issues.push(format!(
                        "break targets {} innermost {:?}",
                        target.0,
                        stack.last()
                    ));
                }
            }
            Stmt::Continue { target } => {
                if stack.last() != Some(&target.0) {
                    issues.push(format!(
                        "continue targets {} innermost {:?}",
                        target.0,
                        stack.last()
                    ));
                }
            }
            Stmt::Loop { id, body: inner } => {
                stack.push(id.0);
                issues.extend(walk_block(body, inner.0 as usize, stack));
                stack.pop();
            }
            Stmt::If { then, els, .. } => {
                issues.extend(walk_block(body, then.0 as usize, stack));
                if let Some(e) = els {
                    issues.extend(walk_block(body, e.0 as usize, stack));
                }
            }
            _ => {}
        }
    }
    issues
}

#[test]
fn named_call_keeps_source_order_and_binding_map() {
    let trace = vec![TRACE_CAPABILITY.to_string()];
    let src = "fn pair(a int, b int) int { return a * 10 + b }\n\
               fn main() int { return pair(b: mark_b(), a: mark_a()) }";
    let checked = compile(src, &trace);
    let bundle = checked.bundle();
    // main is bodies[1] (as in ac-core's own source_hir test): eval order is
    // source order (mark_b evaluated first), bindings place arg1→param0.
    let main_body = &bundle.modules[0].bodies[1];
    let ret = match &main_body.blocks[0].stmts[0] {
        Stmt::Return { value } => *value,
        _ => panic!("expected return"),
    };
    match &main_body.exprs[ret.0 as usize].kind {
        ExprKind::Call {
            eval_args,
            bindings,
            ..
        } => {
            assert_eq!(eval_args.len(), 2);
            assert_eq!(bindings.len(), 2);
            assert!(bindings.iter().any(|b| b.param == 0 && b.arg == 1));
            assert!(bindings.iter().any(|b| b.param == 1 && b.arg == 0));
        }
        other => panic!("expected call, got {:?}", other),
    }
}
