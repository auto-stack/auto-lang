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

// -- T-03: body-scoped lazy input / signature catalog / namespace ------

#[test]
fn sq14_body_scoped_projection_materialises_only_the_target_body() {
    let p = parse(A);
    let input = ace_semantics::adapter::project_body_scoped(&p, "leaf", "pkg", "mod")
        .expect("scoped projection");
    let leaf = input.lookup_fn("leaf").expect("leaf projected");
    let main = input.lookup_fn("main").expect("main projected");
    assert!(!leaf.body.stmts.is_empty(), "the target body is materialised");
    assert!(
        main.body.stmts.is_empty(),
        "an unrelated function is a signature stub"
    );
    assert_eq!(leaf.params.len(), 1);
    assert_eq!(main.params.len(), 0);

    // Flipping the target flips which body exists — no whole-module input.
    let flipped = ace_semantics::adapter::project_body_scoped(&p, "main", "pkg", "mod")
        .expect("scoped projection");
    assert!(flipped.lookup_fn("leaf").unwrap().body.stmts.is_empty());
    assert!(!flipped.lookup_fn("main").unwrap().body.stmts.is_empty());

    // An unrelated body outside this layer's finite surface is never
    // projected at all (its construct cannot block the target).
    let t = parse_program_with(
        "fn f() str { return \"x\" }\nfn main() int { return 1 }\n",
        SourceProfile::Text,
    )
    .expect("text parse");
    let input = ace_semantics::adapter::project_body_scoped(&t, "main", "pkg", "mod")
        .expect("scoped projection of a mixed module");
    assert!(input.lookup_fn("f").unwrap().body.stmts.is_empty());
    assert!(!input.lookup_fn("main").unwrap().body.stmts.is_empty());
}

#[test]
fn sq15_signature_catalog_is_declaration_only() {
    let p = parse(A);
    let cat = SignatureCatalog::build(&p, "pkg", "mod").expect("catalog");
    assert_eq!(
        cat.candidates(),
        &["leaf".to_string(), "main".to_string()],
        "candidate set is sorted"
    );
    let leaf = cat.get("leaf").expect("leaf entry");
    assert_eq!(leaf.ret, PortableTy::I32);
    assert_eq!(leaf.params, vec![("x".to_string(), PortableTy::I32)]);
    assert_eq!(cat.logical_of("leaf").unwrap(), leaf.logical);
    assert!(cat.by_logical(&leaf.logical).is_some());
    assert!(cat.get("absent").is_none());
    assert_eq!(cat.len(), 2);

    // A body-only edit changes NO catalog version.
    let p2 = parse(&A.replace("+ 1", "+ 9"));
    let cat2 = SignatureCatalog::build(&p2, "pkg", "mod").expect("catalog");
    assert_eq!(cat.version("leaf"), cat2.version("leaf"));
    assert_eq!(cat.version("main"), cat2.version("main"));
    assert_eq!(cat.candidates_fp(), cat2.candidates_fp());

    // A signature edit changes only that entry.
    let p3 = parse("fn leaf(x bool) int { return 1 }\n\nfn main() int { return leaf(41) }\n");
    let cat3 = SignatureCatalog::build(&p3, "pkg", "mod").expect("catalog");
    assert_ne!(cat.version("leaf"), cat3.version("leaf"));
    assert_eq!(cat.version("main"), cat3.version("main"));
}

#[test]
fn sq16_foreign_namespace_pages_are_never_reinterpreted() {
    let p = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod");
    let body = s.infer_body(&p, "leaf").expect("facts").facts;
    let walk = walk_body(body_of(&p, "leaf")).unwrap();
    body.admit(&walk).expect("own page admits");

    // The new namespace is not the old 796 page schema: an old page can
    // never be a hit for this layer.
    assert_ne!(SQ_SCHEMA, ace_semantics_contract::SEMA_PAGE_SCHEMA_V2);

    let mut old_schema = body.clone();
    old_schema.schema = ace_semantics_contract::SEMA_PAGE_SCHEMA_V2;
    assert!(matches!(
        old_schema.admit(&walk),
        Err(FactsError::Envelope { .. })
    ));

    let mut foreign_producer = body.clone();
    foreign_producer.producer = "ac796.sema-probe/1";
    assert!(matches!(
        foreign_producer.admit(&walk),
        Err(FactsError::Envelope { .. })
    ));

    let mut wrong_algo = body.clone();
    wrong_algo.algo = "ac796-bidir-worklist/0";
    assert!(matches!(
        wrong_algo.admit(&walk),
        Err(FactsError::Envelope { .. })
    ));

    let mut wrong_profile = body.clone();
    wrong_profile.profile = "core-i32/1";
    assert!(matches!(
        wrong_profile.admit(&walk),
        Err(FactsError::Envelope { .. })
    ));

    // Signature pages carry the same envelope discipline.
    let sig = s.fn_sig(&p, "leaf").expect("sig").facts;
    sig.check_envelope().expect("own signature page");
    let mut bad_sig = sig.clone();
    bad_sig.schema = ace_semantics_contract::SEMA_PAGE_SCHEMA_V2;
    assert!(matches!(
        bad_sig.check_envelope(),
        Err(FactsError::Envelope { .. })
    ));
    let mut bad_sig = sig;
    bad_sig.producer = "ac796.sema-probe/1";
    assert!(matches!(
        bad_sig.check_envelope(),
        Err(FactsError::Envelope { .. })
    ));
}

#[test]
fn sq17_session_signature_query_matches_the_catalog() {
    let p = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod");
    let cat = s.catalog(&p).expect("catalog");
    let from_session = s.fn_sig(&p, "leaf").expect("sig").facts;
    let from_catalog = cat.get("leaf").expect("catalog entry");
    assert_eq!(from_session.version, from_catalog.version);
    assert_eq!(from_session.logical, from_catalog.logical);
    assert_eq!(from_session.params, from_catalog.params);
    assert_eq!(from_session.ret, from_catalog.ret);
}

// -- T-04: named-argument input ---------------------------------------

const NAMED_SRC: &str =
    "fn pick(a int, b bool) int { if b { return a }\n  return 0 }\n\nfn main() int { return pick(b: true, a: 7) }\n";

#[test]
fn sq18_named_arguments_produce_clean_facts_and_a_distinct_key() {
    let p = parse(NAMED_SRC);
    let mut s = SourceQuerySession::new("pkg", "mod");
    let main = s.infer_body(&p, "main").expect("named-arg facts");
    assert!(
        main.facts.is_complete() && main.facts.is_clean(),
        "{:?}",
        main.facts
    );
    assert_eq!(main.facts.sig_reads, vec!["pick".to_string()]);

    // The positional reading of the same operands is a real type error:
    // the fact page carries diagnostics (never silently mistyped).
    let pos_src = NAMED_SRC.replace("pick(b: true, a: 7)", "pick(true, 7)");
    let p2 = parse(&pos_src);
    let mut s2 = SourceQuerySession::new("pkg", "mod");
    let bad = s2.infer_body(&p2, "main").expect("positional facts");
    assert!(!bad.facts.is_clean(), "{:?}", bad.facts);
    assert!(bad
        .facts
        .diags
        .iter()
        .any(|d| d.code == "type-mismatch"));

    // Binding names are part of the structural key: the two spellings can
    // never share a body version.
    let walk_named = walk_body(body_of(&p, "main")).unwrap();
    let walk_pos = walk_body(body_of(&p2, "main")).unwrap();
    assert_ne!(walk_named.version, walk_pos.version);
}

// -- T-04: fact-supplied bind (types authoritative during bind) --------

#[test]
fn sq19_fact_supplied_bind_installs_query_types_and_verifies() {
    let p = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod");
    let (resolved, applied, supply) =
        crate::bind_with_facts(&p, &[], "pkg", "mod", &mut s, SourceProfile::Scalar)
            .expect("fact-supplied bind");
    assert_eq!(applied.len(), 2);
    assert_eq!(supply.admitted_bodies(), 2);
    assert!(supply.expr_count() >= 5, "all scalar nodes supplied");

    // Every resolved expression type is the QUERY's; nothing was left to
    // the resolver's own rule (which the meter records as zero on this
    // path — see the av sq934 checks).
    for f in &resolved.fns {
        assert_eq!(f.result, ace_core::source::resolve::VT::I32);
    }
    let parts = ace_core::source::emit_resolved_parts(&resolved, A, "m.at", SourceProfile::Scalar)
        .expect("emit");
    ace_core::verify::verify_scalar(parts.bundle).expect("fact-bound HIR verifies");
}

#[test]
fn sq20_fact_supplied_bind_rejects_a_missing_fact() {
    let p = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod");
    let facts = s.infer_body(&p, "leaf").expect("leaf facts").facts;
    let walk = walk_body(body_of(&p, "leaf")).unwrap();
    facts.admit(&walk).expect("own page");

    // Drop one node fact: the supply no longer covers the body, and the
    // bind must reject rather than fall back to the resolver's rule.
    let mut partial = facts.clone();
    partial.nodes.pop();
    let mut supply = crate::BodyTypeSupply::new();
    let err = supply
        .add_body(body_of(&p, "leaf"), &partial)
        .expect_err("a partial page must not be admitted");
    assert!(matches!(err, crate::ApplyError::Facts(_)), "{err:?}");
}

// -- T-05: complete keys, positive/negative deps, SCC, refresh --------

const MUTUAL: &str = "fn even(n int) bool { if n == 0 { return true }\n  return odd(n - 1) }\n\nfn odd(n int) bool { if n == 0 { return false }\n  return even(n - 1) }\n";

#[test]
fn sq21_complete_key_covers_every_axis_and_capabilities() {
    let p = parse(A);
    let s = SourceQuerySession::new("pkg", "mod");
    let key = s.body_key(&p, "leaf").expect("body key");
    assert_eq!(key.query, QueryKind::InferBody);
    assert!(key.body_version.is_some());
    assert_eq!(key.profile, crate::SQ_PROFILE);
    assert_eq!(key.schema, crate::SQ_SCHEMA);
    assert_eq!(key.target, crate::TARGET_SEMANTICS);
    assert!(key.capabilities.is_empty());

    // Body edit changes the body axis only.
    let p2 = parse(&A.replace("+ 1", "+ 2"));
    let key2 = s.body_key(&p2, "leaf").expect("body key");
    assert_ne!(key.body_version, key2.body_version);
    assert_eq!(key.sig_version, key2.sig_version);
    assert_ne!(key.canonical(), key2.canonical());

    // Signature edit changes the signature axis (and therefore the key).
    let p3 = parse("fn leaf(x bool) int { return 1 }\n\nfn main() int { return leaf(41) }\n");
    let key3 = s.body_key(&p3, "leaf").expect("body key");
    assert_ne!(key.sig_version, key3.sig_version);

    // Capabilities are part of the key, order-free.
    let caps_a = SourceQuerySession::new("pkg", "mod").with_capabilities(["b", "a"]);
    let caps_b = SourceQuerySession::new("pkg", "mod").with_capabilities(["a", "b", "a"]);
    let ka = caps_a.body_key(&p, "leaf").expect("key");
    let kb = caps_b.body_key(&p, "leaf").expect("key");
    assert_eq!(ka.canonical(), kb.canonical(), "caller order does not matter");
    assert_eq!(ka.capabilities, vec!["a".to_string(), "b".to_string()]);
    let kc = SourceQuerySession::new("pkg", "mod")
        .with_capabilities(["a"])
        .body_key(&p, "leaf")
        .expect("key");
    assert_ne!(ka.canonical(), kc.canonical());
}

#[test]
fn sq22_capability_change_invalidates_warm_pages() {
    let p = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod");
    assert!(!s.infer_body(&p, "leaf").expect("cold").hit);
    assert!(s.infer_body(&p, "leaf").expect("warm").hit);
    let runs = s.counters().solver_runs;
    s.set_capabilities(["trace-capability"]);
    let out = s.infer_body(&p, "leaf").expect("re-solve after a capability change");
    assert!(!out.hit, "a different capability set is a different meaning");
    assert_eq!(s.counters().solver_runs, runs + 1);
}

#[test]
fn sq23_mutual_recursion_is_not_a_query_cycle() {
    let p = parse(MUTUAL);
    let mut s = SourceQuerySession::new("pkg", "mod");
    let even = s.infer_body(&p, "even").expect("even facts");
    let odd = s.infer_body(&p, "odd").expect("odd facts");
    assert!(even.facts.is_complete() && even.facts.is_clean(), "{:?}", even.facts);
    assert!(odd.facts.is_complete() && odd.facts.is_clean(), "{:?}", odd.facts);
    assert_eq!(even.facts.sig_reads, vec!["odd".to_string()]);
    assert_eq!(odd.facts.sig_reads, vec!["even".to_string()]);

    // Editing odd's BODY keeps even warm: even reads odd's signature only.
    let edited = MUTUAL.replace("return false", "return n == 7");
    let p2 = parse(&edited);
    assert!(!s.infer_body(&p2, "odd").expect("odd re-solve").hit);
    assert!(
        s.infer_body(&p2, "even").expect("even warm").hit,
        "an SCC member's body edit must not invalidate its signature readers"
    );

    // Self-recursion resolves against its own signature (no cycle).
    let rec = parse("fn sum(n int) int { if n == 0 { return 0 }\n  return n + sum(n - 1) }\n");
    let mut s2 = SourceQuerySession::new("pkg", "mod");
    let facts = s2.infer_body(&rec, "sum").expect("self-recursive facts");
    assert!(facts.facts.is_complete() && facts.facts.is_clean(), "{:?}", facts.facts);
}

#[test]
fn sq24_audit_reports_each_dependency_axis() {
    let p = parse(A);
    let mut s = SourceQuerySession::new("pkg", "mod");
    s.infer_body(&p, "leaf").expect("leaf");
    s.infer_body(&p, "main").expect("main");
    let audit = s.audit(&p).expect("audit");
    assert_eq!(audit.len(), 2);
    assert!(audit.iter().all(|a| a.fresh && a.reason == "ok"), "{audit:?}");

    // Body-only edit: only the edited owner is stale.
    let p2 = parse(&A.replace("+ 1", "+ 2"));
    let audit = s.audit(&p2).expect("audit");
    let leaf = audit.iter().find(|a| a.name == "leaf").unwrap();
    let main = audit.iter().find(|a| a.name == "main").unwrap();
    assert!(!leaf.fresh && leaf.reason == "body-changed", "{leaf:?}");
    assert!(main.fresh && main.reason == "ok", "{main:?}");

    // New function appearing: a page that recorded an absent name flips.
    let missing = parse("fn main() int { return helper(1) }\n");
    let mut s2 = SourceQuerySession::new("pkg", "mod");
    s2.infer_body(&missing, "main").expect("negative dep");
    let with_helper = parse("fn helper(v int) int { return v }\n\nfn main() int { return helper(1) }\n");
    let audit = s2.audit(&with_helper).expect("audit");
    assert_eq!(audit.len(), 1);
    assert!(!audit[0].fresh && audit[0].reason == "negative-flipped", "{audit:?}");
}

#[test]
fn sq25_source_map_refresh_keeps_the_key_and_the_bind_works() {
    let drifted = format!("\n\n// shifted\n{}", A);
    let p1 = parse(A);
    let p2 = parse(&drifted);
    let s = SourceQuerySession::new("pkg", "mod");
    let k1 = s.body_key(&p1, "leaf").expect("key");
    let k2 = s.body_key(&p2, "leaf").expect("key");
    assert_eq!(k1.canonical(), k2.canonical(), "spans are not identity");

    // The drifted source still binds from facts and verifies.
    let mut session = SourceQuerySession::new("pkg", "mod");
    let (resolved, applied, supply) =
        crate::bind_with_facts(&p2, &[], "pkg", "mod", &mut session, SourceProfile::Scalar)
            .expect("bind drifted source");
    assert_eq!(applied.len(), 2);
    assert_eq!(supply.admitted_bodies(), 2);
    let parts = ace_core::source::emit_resolved_parts(&resolved, &drifted, "m.at", SourceProfile::Scalar)
        .expect("emit");
    ace_core::verify::verify_scalar(parts.bundle).expect("drifted fact-bound HIR verifies");
}
