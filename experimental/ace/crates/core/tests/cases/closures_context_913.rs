//! PLAN-913 T-03/T-05: the portable closures DECLARATION context.
//!
//! The oracle is EQUIVALENCE, not "the dump round-trips": a frontend rebuilt
//! from the dump (signature-only environment + restored catalog/registry)
//! must produce the BYTE-IDENTICAL scoped document for the same single-lambda
//! edit that the cold frontend produces. Any missing declaration fact — a
//! resolver table entry, a creation-site fact, a registry mode — changes the
//! emitted Atom text or the plan and turns this test red.
//!
//! Negative controls: cross-checked evidence (a decoded lambda environment /
//! creator site list that disagrees with the persisted declarations), a
//! different source, a different module, a foreign schema and duplicate keys
//! are all controlled rejections.

use ace_core::source::closures_context::{
    dump_context, restore_context, CaptureModeDump, ContextDump, UnitEvidence, CONTEXT_DUMP_SCHEMA,
};
use ace_core::source::closures_fn::ClosuresFunctionFrontend;

const SRC: &str = "\
fn helper(n int) int { return n + 1 }
fn other() int { return 7 }
fn main() int {
    var x int = 10
    var base int = 1
    var f = (n int) => { base = base + n; return base }
    var g = (n int) => { return x - n }
    return f(20) + f(0) + g(3) + helper(1)
}
";

fn front() -> ClosuresFunctionFrontend {
    let (front, _parts, _checked) =
        ClosuresFunctionFrontend::cold(SRC, "main.at", &[]).expect("cold closures frontend");
    front
}

/// Evidence a REAL decoder+verifier would hand over: per-unit lambda
/// environment arity and per-creator creation sites (callee + modes), both
/// derived from the admitted MIR in the av layer.
fn evidence(front: &ClosuresFunctionFrontend) -> Vec<UnitEvidence> {
    let mut out = Vec::new();
    for u in front.units() {
        match &u.kind {
            ace_core::source::closures_fn::UnitKind::Lambda { .. } => {
                let decl = front
                    .registry()
                    .lambda(&format!("d_{}", u.name))
                    .expect("decl");
                out.push(UnitEvidence {
                    unit: format!("d_{}", u.name),
                    env_arity: Some(decl.env.len()),
                    sites: Vec::new(),
                });
            }
            ace_core::source::closures_fn::UnitKind::Owner { .. } => {
                let creator = format!("d_{}", u.name);
                let sites = front
                    .registry()
                    .sites_of(&creator)
                    .iter()
                    .map(|callee| {
                        let decl = front.registry().lambda(callee).expect("site decl");
                        let modes = decl
                            .site_modes
                            .iter()
                            .map(|m| match m {
                                ace_core::hir::CaptureMode::Move => CaptureModeDump::Move,
                                ace_core::hir::CaptureMode::Borrow => CaptureModeDump::Borrow,
                                ace_core::hir::CaptureMode::Shared => CaptureModeDump::Shared,
                            })
                            .collect();
                        (callee.clone(), modes)
                    })
                    .collect();
                out.push(UnitEvidence {
                    unit: creator,
                    env_arity: None,
                    sites,
                });
            }
        }
    }
    out
}

/// The single-lambda edit both frontends must agree on: `g`'s body.
fn lambda_g_edit(front: &ClosuresFunctionFrontend) -> (std::ops::Range<usize>, String) {
    let unit = front
        .units()
        .iter()
        .find(|u| u.name == "__closure_main_1")
        .expect("g unit");
    let needle = "return x - n }";
    let at = SRC.find(needle).expect("g body text");
    assert!(
        at >= unit.span.start && at + needle.len() <= unit.span.end,
        "the edit range must sit inside g's span ({}..{})",
        unit.span.start,
        unit.span.end
    );
    (at..at + needle.len(), "return x - n * 2 }".to_string())
}

#[test]
fn closures_context_913_dump_restore_emits_identical_document() {
    let mut cold = front();
    let dump = dump_context(&cold);
    assert_eq!(dump.schema, CONTEXT_DUMP_SCHEMA);
    assert_eq!(dump.units.len(), 5);
    assert_eq!(dump.fns.len(), 5);
    assert_eq!(dump.registry.len(), 2, "two lambdas are registered");
    // The wire form is data: a JSON round-trip must not lose a fact.
    let json = serde_json::to_vec(&dump).expect("serialize dump");
    let decoded: ContextDump = serde_json::from_slice(&json).expect("deserialize dump");
    assert_eq!(decoded, dump);
    let mut restored =
        restore_context(decoded, SRC, "main.at", &[], &evidence(&cold)).expect("restore context");
    // Catalog + registry + identity are restored exactly.
    assert!(restored.is_restored());
    assert_eq!(restored.source_sha(), cold.source_sha());
    assert_eq!(restored.context_revision(), cold.context_revision());
    assert_eq!(restored.next_logical_id(), cold.next_logical_id());
    assert_eq!(restored.units().len(), cold.units().len());
    for (a, b) in restored.units().iter().zip(cold.units().iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.span, b.span);
        assert_eq!(a.body_digest, b.body_digest);
        assert_eq!(a.mint_span, b.mint_span);
        assert_eq!(a.kind, b.kind);
    }
    // EQUIVALENCE ORACLE: the same single-lambda edit produces the same
    // scoped document, byte for byte.
    let (range, replacement) = lambda_g_edit(&cold);
    let sha = cold.source_sha().to_string();
    let cold_candidate = cold
        .edit_body(&sha, range.clone(), &replacement)
        .expect("cold diagnostics")
        .ok()
        .expect("cold fast route");
    let restored_candidate = restored
        .edit_body(&sha, range, &replacement)
        .expect("restored diagnostics")
        .ok()
        .expect("restored fast route");
    assert_eq!(
        cold_candidate.parts.atom_text, restored_candidate.parts.atom_text,
        "the restored context must emit the same document"
    );
    assert_eq!(
        cold_candidate.receipt.real_body_plans,
        restored_candidate.receipt.real_body_plans
    );
    assert_eq!(
        cold_candidate.receipt.scaffold_fns,
        restored_candidate.receipt.scaffold_fns
    );
    assert_eq!(
        cold_candidate.def_id_text(),
        restored_candidate.def_id_text()
    );
    assert_eq!(
        cold_candidate.real_def_ids(),
        restored_candidate.real_def_ids()
    );
    // The edit really changed the unit (a distinct new source exists).
    assert!(cold_candidate.new_source.contains("return x - n * 2 }"));
}

#[test]
fn closures_context_913_rejects_evidence_that_disagrees_with_declarations() {
    let cold = front();
    let dump = dump_context(&cold);
    let mut ev = evidence(&cold);
    // The creator that actually has creation sites (main), not a site-less
    // owner: clearing its decoded sites must be rejected.
    let owner = ev
        .iter_mut()
        .find(|e| !e.sites.is_empty())
        .expect("creator evidence");
    owner.sites.clear();
    let err = restore_context(dump.clone(), SRC, "main.at", &[], &ev)
        .err()
        .expect("a creator with fewer decoded sites must be rejected");
    assert!(
        err.iter().any(|d| d.code == "restored.context-mismatch"),
        "diagnostics: {err:?}"
    );
    // A lambda whose decoded environment arity disagrees with the registry.
    let mut ev = evidence(&cold);
    let lambda = ev
        .iter_mut()
        .find(|e| e.env_arity.is_some())
        .expect("lambda evidence");
    lambda.env_arity = Some(99);
    let err = restore_context(dump, SRC, "main.at", &[], &ev)
        .err()
        .expect("an environment arity mismatch must be rejected");
    assert!(
        err.iter().any(|d| d.code == "restored.context-mismatch"),
        "diagnostics: {err:?}"
    );
}

#[test]
fn closures_context_913_rejects_source_module_schema_and_duplicates() {
    let cold = front();
    let dump = dump_context(&cold);
    let ev = evidence(&cold);
    // A different source text (same shape) is a different accepted state.
    let other = SRC.replace("n + 1", "n + 2");
    let err = restore_context(dump.clone(), &other, "main.at", &[], &ev)
        .err()
        .expect("a different source must not restore");
    assert!(err.iter().any(|d| d.code == "restored.source-mismatch"));
    // A different logical module is a foreign context.
    let err = restore_context(dump.clone(), SRC, "elsewhere.at", &[], &ev)
        .err()
        .expect("a foreign module must not restore");
    assert!(err.iter().any(|d| d.code == "restored.context-mismatch"));
    // An unknown schema is STALE.
    let mut stale = dump.clone();
    stale.schema = "ace.closures.context-dump/0".to_string();
    let err = restore_context(stale, SRC, "main.at", &[], &ev)
        .err()
        .expect("a foreign schema must not restore");
    assert!(err.iter().any(|d| d.code == "restored.stale"));
    // Duplicate registry entries (two declarations for one lambda) are a
    // controlled rejection, never "last one wins".
    let mut dup = dump.clone();
    let first = dup.registry[0].clone();
    dup.registry.push(first);
    let err = restore_context(dup, SRC, "main.at", &[], &ev)
        .err()
        .expect("a duplicate registry entry must be rejected");
    assert!(err.iter().any(|d| d.code == "restored.duplicate"));
    // A capability mismatch is the host's decision, not the dump's.
    let err = restore_context(dump.clone(), SRC, "main.at", &["trace".to_string()], &ev)
        .err()
        .expect("a capability mismatch must be rejected");
    assert!(err.iter().any(|d| d.code == "restored.capability"));
}

#[test]
fn closures_context_913_owner_route_needs_retained_body() {
    // A restored context has signature-only entries: an edit that needs a
    // retained BODY (the owner route's lambda reuse) must NOT silently emit
    // an empty sibling body. The cold frontend takes the fast owner route;
    // the restored one must decline (the caller then runs the honest whole
    // module truth path).
    let mut cold = front();
    let dump = dump_context(&cold);
    let ev = evidence(&cold);
    let mut restored = restore_context(dump, SRC, "main.at", &[], &ev).expect("restore context");
    let sha = cold.source_sha().to_string();
    // Editing the OWNER body (main) is the owner route on a cold frontend.
    let at = SRC.find("return f(20)").expect("main tail");
    let cold_route = cold
        .edit_body(&sha, at..at + "return f(20)".len(), "return f(21)")
        .expect("cold diagnostics");
    assert!(
        cold_route.is_ok(),
        "the cold frontend must take the owner route"
    );
    let restored_route = restored
        .edit_body(&sha, at..at + "return f(20)".len(), "return f(21)")
        .expect("restored diagnostics");
    assert!(
        restored_route.is_err(),
        "a restored context must decline the owner route instead of emitting empty sibling bodies"
    );
}
