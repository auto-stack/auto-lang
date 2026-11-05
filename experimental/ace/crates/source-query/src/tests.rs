//! T-02 bounded feasibility checks for the SQ-A layer.
//!
//! Every check ties one plan invariant to one observable fact of the real
//! producer (never a reconstructed summary):
//!
//! | check | invariant |
//! |---|---|
//! | `sq01` | one ordinary body yields complete, structural, portable facts |
//! | `sq02` | `let`/`var` bindings are keyed by the declaration node, not the name |
//! | `sq03` | a body-only edit keeps the caller warm (signature axis reads only) |
//! | `sq04` | a callee signature change invalidates the real reader |
//! | `sq05` | whitespace/prefix drift keeps the semantic key, refreshes spans |
//! | `sq06` | a negative dependency flips when the absent name appears |
//! | `sq07` | stale/tampered pages are refused at admission |
//! | `sq08` | a cold miss really calls the solver; a warm hit does not |
//! | `sq09` | constructs outside the finite surface are controlled rejections |

use super::*;
use ace_core::source::ast::{Block, Item, Program};
use ace_core::source::{parse_program_with, SourceProfile};

const A: &str = "fn leaf(x int) int { return x + 1 }\n\nfn main() int { return leaf(41) }\n";

fn parse(src: &str) -> Program {
    parse_program_with(src, SourceProfile::Scalar).expect("scalar profile parse")
}

fn body_of<'a>(p: &'a Program, name: &str) -> &'a Block {
    for item in &p.items {
        if let Item::Fn(f) = item {
            if f.name.name == name {
                return &f.body;
            }
        }
    }
    panic!("no function {name} in program");
}

fn kinds(w: &BodyWalk) -> Vec<&'static str> {
    w.nodes.iter().map(|n| n.kind).collect()
}

#[test]
fn sq01_one_body_yields_complete_structural_facts() {
    let p = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod");

    let leaf = s.infer_body(&p, "leaf").expect("leaf facts");
    assert!(!leaf.hit, "cold query must solve");
    let f = &leaf.facts;
    assert!(f.is_complete() && f.is_clean(), "leaf facts: {f:?}");
    assert!(facts::envelope_matches(f));
    assert_eq!(f.nodes.len(), 3, "binary + var + int literal");

    let walk = walk_body(body_of(&p, "leaf")).unwrap();
    assert_eq!(
        kinds(&walk),
        vec!["stmt.return", "expr.binary", "expr.var", "expr.int"]
    );
    // Every expression node carries an i32 fact.
    assert!(f.nodes.iter().all(|n| n.ty == PortableTy::I32));
    // ... and the ids are exactly the walk's expression ids.
    let ids: Vec<NodeId> = walk.expr_ids().collect();
    let fact_ids: Vec<NodeId> = f.nodes.iter().map(|n| n.id).collect();
    assert_eq!(ids, fact_ids);

    let main = s.infer_body(&p, "main").expect("main facts");
    assert!(main.facts.is_complete() && main.facts.is_clean());
    let mwalk = walk_body(body_of(&p, "main")).unwrap();
    let call = mwalk
        .nodes
        .iter()
        .find(|n| n.kind == "expr.call")
        .expect("direct call node");
    assert_eq!(main.facts.ty_of(call.id), Some(PortableTy::I32));
    assert_eq!(main.facts.sig_reads, vec!["leaf".to_string()]);

    assert_ne!(leaf.facts.logical, main.facts.logical);
    assert_eq!(s.counters().solver_runs, 2);
}

#[test]
fn sq02_bindings_are_keyed_by_declaration_node() {
    let src = "fn plus(x int) int {\n  let y = x + 1\n  return y\n}\n";
    let p = parse(src);
    let mut s = SourceQuerySession::new("pkg", "mod");
    let out = s.infer_body(&p, "plus").expect("plus facts");
    let f = &out.facts;
    assert!(f.is_complete() && f.is_clean(), "{f:?}");
    assert_eq!(f.bindings.len(), 1);
    assert_eq!(f.bindings[0].name, "y");
    assert_eq!(f.bindings[0].ty, PortableTy::I32);
    let walk = walk_body(body_of(&p, "plus")).unwrap();
    assert_eq!(walk.node(f.bindings[0].node).unwrap().kind, "stmt.let");
}

#[test]
fn sq03_body_only_edit_keeps_the_caller_warm() {
    let p1 = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod");
    let leaf1 = s.infer_body(&p1, "leaf").expect("leaf facts");
    let main1 = s.infer_body(&p1, "main").expect("main facts");
    assert!(!leaf1.hit && !main1.hit);

    // Body-only edit of the callee (length change: all later spans drift).
    let p2 = parse(&A.replace("+ 1", "+ 1000"));
    let leaf2 = s.infer_body(&p2, "leaf").expect("leaf re-solve");
    assert!(!leaf2.hit, "the body itself must re-solve");
    assert_ne!(leaf1.facts.body_version, leaf2.facts.body_version);

    let main2 = s.infer_body(&p2, "main").expect("main warm");
    assert!(main2.hit, "signature-only reader must stay warm");
    assert_eq!(s.counters().solver_runs, 3);
    assert_eq!(s.counters().body_hits, 1);
}

#[test]
fn sq04_callee_signature_change_invalidates_the_reader() {
    let p1 = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod");
    s.infer_body(&p1, "leaf").expect("leaf");
    s.infer_body(&p1, "main").expect("main");

    // Same body name, different parameter type: the caller's real read
    // (leaf's signature) changes, and its call becomes type-invalid.
    let p2 = parse("fn leaf(x bool) int { return 1 }\n\nfn main() int { return leaf(41) }\n");
    let main2 = s.infer_body(&p2, "main").expect("re-solve");
    assert!(!main2.hit, "the reader must re-solve");
    assert!(
        !main2.facts.is_clean(),
        "the stale argument type is a real diagnostic: {:?}",
        main2.facts.diags
    );
    assert_eq!(s.counters().solver_runs, 3);
}

#[test]
fn sq05_whitespace_drift_keeps_the_key_and_refreshes_spans() {
    let p1 = parse(A);
    let p2 = parse(&format!("\n\n   {}", A));
    let w1 = walk_body(body_of(&p1, "main")).unwrap();
    let w2 = walk_body(body_of(&p2, "main")).unwrap();
    assert_eq!(w1.version, w2.version, "structure is identical");
    assert_eq!(kinds(&w1), kinds(&w2));
    assert_ne!(
        w1.nodes[0].span.start, w2.nodes[0].span.start,
        "the source map really moved"
    );

    let mut s = SourceQuerySession::new("pkg", "mod");
    assert!(!s.infer_body(&p1, "main").expect("cold").hit);
    let warm = s.infer_body(&p2, "main").expect("warm");
    assert!(warm.hit);
}

#[test]
fn sq06_negative_dependency_flips_when_the_name_appears() {
    let p1 = parse("fn main() int { return helper(1) }\n");
    let mut s = SourceQuerySession::new("pkg", "mod");
    let cold = s.infer_body(&p1, "main").expect("solve with absent callee");
    assert!(!cold.facts.is_complete(), "the call has no fact yet");
    assert!(cold
        .facts
        .negatives
        .iter()
        .any(|n| n.domain == "fn" && n.name == "helper"));

    let p2 = parse("fn helper(v int) int { return v }\n\nfn main() int { return helper(1) }\n");
    let warm = s.infer_body(&p2, "main").expect("re-solve after the name appears");
    assert!(!warm.hit, "the candidate set changed: the negative dep is stale");
    assert!(
        warm.facts.is_complete() && warm.facts.is_clean(),
        "the now-resolvable call must be typed: {:?}",
        warm.facts
    );
    assert!(warm.facts.negatives.is_empty());
}

#[test]
fn sq07_stale_or_tampered_pages_are_refused() {
    let p1 = parse(A);
    let limits = ProduceLimits::default();
    let facts = produce_infer_body(&p1, "pkg", "mod", "leaf", limits, None).expect("facts");
    let walk1 = walk_body(body_of(&p1, "leaf")).unwrap();
    facts.admit(&walk1).expect("own page admits");

    // Older body version: refused outright.
    let p2 = parse(&A.replace("+ 1", "+ 2"));
    let walk2 = walk_body(body_of(&p2, "leaf")).unwrap();
    assert!(matches!(
        facts.validate_against(&walk2),
        Err(FactsError::BodyVersionMismatch { .. })
    ));

    // Missing coverage (a removed fact is not "absent, so infer it").
    let mut dropped = facts.clone();
    dropped.nodes.remove(0);
    assert!(matches!(
        dropped.admit(&walk1),
        Err(FactsError::IncompleteCoverage { missing: 1 })
    ));

    // Node-kind mismatch (a fact mapped to the wrong structural node).
    let mut swapped = facts.clone();
    swapped.nodes[0].kind = "expr.bool";
    assert!(matches!(
        swapped.admit(&walk1),
        Err(FactsError::KindMismatch { .. })
    ));

    // Duplicate node.
    let mut dup = facts.clone();
    let first = dup.nodes[0];
    dup.nodes.push(first);
    assert!(matches!(
        dup.admit(&walk1),
        Err(FactsError::DuplicateNode { .. })
    ));

    // Diagnostics are not consumable facts.
    let mut with_diag = facts.clone();
    with_diag.diags.push(PortableDiag {
        code: "type-mismatch".into(),
        node: None,
        message: "x".into(),
    });
    assert!(matches!(
        with_diag.admit(&walk1),
        Err(FactsError::Diagnostics { count: 1 })
    ));
}

#[test]
fn sq08_cold_miss_solves_and_warm_hit_does_not() {
    let p = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod").with_solver(false);
    assert!(matches!(
        s.infer_body(&p, "leaf"),
        Err(QueryError::SolverDisabled { .. })
    ));
    assert_eq!(s.counters().solver_runs, 0);

    s.set_allow_solver(true);
    assert!(!s.infer_body(&p, "leaf").expect("cold solve").hit);
    assert_eq!(s.counters().solver_runs, 1);

    s.set_allow_solver(false);
    let hit = s.infer_body(&p, "leaf").expect("warm hit with no producer");
    assert!(hit.hit);
    assert_eq!(s.counters().solver_runs, 1, "a hit never runs the solver");
}

#[test]
fn sq09_outside_the_finite_surface_is_a_controlled_rejection() {
    let p = parse(A);
    assert!(matches!(
        produce_fn_sig(&p, "pkg", "mod", "absent"),
        Err(ProduceError::UnknownFn { .. })
    ));

    // A text-profile body (string literal) is outside this layer's finite
    // surface: the structural walk refuses it instead of skipping it.
    let t = parse_program_with("fn f() str { return \"x\" }\n", SourceProfile::Text)
        .expect("text profile parse");
    let err = walk_body(body_of(&t, "f")).expect_err("str literal is unsupported");
    assert!(matches!(err, WalkError::Unsupported { what: "str-literal", .. }));
}

#[test]
fn sq10_signature_query_never_reads_the_body() {
    let p1 = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod");
    let a = s.fn_sig(&p1, "leaf").expect("sig");
    assert!(!a.hit);
    assert_eq!(a.facts.ret, PortableTy::I32);
    assert_eq!(a.facts.params, vec![("x".to_string(), PortableTy::I32)]);

    let p2 = parse(&A.replace("+ 1", "+ 999"));
    let b = s.fn_sig(&p2, "leaf").expect("sig");
    assert!(b.hit, "a body-only edit leaves the signature axis alone");
    assert_eq!(a.facts.version, b.facts.version);
    assert_eq!(s.counters().solver_runs, 0, "signatures never solve");
}

#[test]
fn sq11_walk_ids_and_versions_are_deterministic() {
    let p1 = parse(A);
    let p2 = parse(A);
    let w1 = walk_body(body_of(&p1, "leaf")).unwrap();
    let w2 = walk_body(body_of(&p2, "leaf")).unwrap();
    assert_eq!(w1.version, w2.version);
    assert_eq!(kinds(&w1), kinds(&w2));
    assert_eq!(
        w1.nodes.iter().map(|n| n.id.0).collect::<Vec<_>>(),
        w2.nodes.iter().map(|n| n.id.0).collect::<Vec<_>>()
    );

    // A different literal value is a DIFFERENT body version (values are
    // part of the canonical bytes), while the structure/ids stay equal.
    let p3 = parse(&A.replace("+ 1", "+ 2"));
    let w3 = walk_body(body_of(&p3, "leaf")).unwrap();
    assert_ne!(w1.version, w3.version);
    assert_eq!(kinds(&w1), kinds(&w3));
}

// -- T-02 port (fact → resolved program → emitted HIR) -----------------

#[test]
fn sq12_apply_facts_drives_emission_and_verification() {
    let p = parse(A);
    let mut resolved =
        ace_core::source::resolve_with(&p, &[], SourceProfile::Scalar).expect("bind");
    let mut s = SourceQuerySession::new("pkg", "mod");
    let applied = crate::apply_facts_to_resolved(&p, &mut resolved, &mut s).expect("apply facts");
    assert_eq!(applied.len(), 2);
    assert!(applied.iter().all(|a| a.report.exprs >= 1));
    assert_eq!(
        applied.iter().find(|a| a.name == "leaf").unwrap().report.exprs,
        3,
        "binary + var + literal"
    );

    let parts =
        ace_core::source::emit_resolved_parts(&resolved, A, "m.at", SourceProfile::Scalar)
            .expect("emit from facts");
    ace_core::verify::verify_scalar(parts.bundle).expect("the fact-typed HIR verifies");
    assert!(resolved.fns.iter().all(|f| f.result == ace_core::source::resolve::VT::I32));
}

#[test]
fn sq13_apply_rejects_another_owners_page() {
    let p = parse(A);
    let mut resolved =
        ace_core::source::resolve_with(&p, &[], SourceProfile::Scalar).expect("bind");
    let mut s = SourceQuerySession::new("pkg", "mod");
    let leaf = s.infer_body(&p, "leaf").expect("leaf facts").facts;
    let main_body = body_of(&p, "main");
    let rf = resolved
        .fns
        .iter_mut()
        .find(|f| f.name == "main")
        .expect("main bound");
    let err = crate::apply_body_facts(main_body, rf, &leaf).expect_err("foreign page");
    assert!(matches!(err, crate::ApplyError::Facts(_)), "{err:?}");
}
