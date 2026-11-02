//! PLAN-912 T-02: the closures single-function minimal full chain — the core
//! half. One owner (`main`) with TWO sibling lambdas; a pure body edit of the
//! SECOND lambda re-parses only its fragment, re-resolves only that lambda,
//! emits a scoped single-function document, mints a REAL closures credential
//! (registry-backed) and is verifiable against the retained registry. The
//! sibling lambda, the owner and every other function show ZERO work on
//! every metered stage. Negative controls: forged env, registry-missing
//! callee, hidden capture-decl change, stale base, cross-unit edit and the
//! whole-path axis.

use ace_core::body_work::{self, Stage};
use ace_core::source::closures_fn::{
    ClosuresFnCandidate, ClosuresFunctionFrontend, NonFastReason, UnitKind,
};
use ace_core::source::{parse_program_with, SourceProfile};
use ace_core::verify::{verify_closures, verify_closures_function, ClosuresVerifyContext};

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

/// `g`'s body edit: `x - n` → `x - n * 2` (g(3): 7 → 4; program 51 → 48).
fn edited_source() -> String {
    SRC.replace("return x - n }", "return x - n * 2 }")
}

fn unit_of(front: &ClosuresFunctionFrontend, name: &str) -> usize {
    front
        .units()
        .iter()
        .position(|u| u.name == name)
        .unwrap_or_else(|| panic!("no unit `{name}`"))
}

fn fast(
    result: Result<
        Result<ClosuresFnCandidate, NonFastReason>,
        Vec<ace_core::atom_text::Diagnostic>,
    >,
) -> ClosuresFnCandidate {
    match result {
        Ok(Ok(candidate)) => candidate,
        Ok(Err(reason)) => panic!("expected the fast path, got non-fast {reason:?}"),
        Err(ds) => panic!("expected the fast path, got diagnostics {ds:?}"),
    }
}

fn non_fast(
    result: Result<
        Result<ClosuresFnCandidate, NonFastReason>,
        Vec<ace_core::atom_text::Diagnostic>,
    >,
) -> NonFastReason {
    match result {
        Ok(Err(reason)) => reason,
        Ok(Ok(_)) => panic!("expected a named non-fast decline, got the fast path"),
        Err(ds) => panic!("expected a named non-fast decline, got diagnostics {ds:?}"),
    }
}

fn edit(
    front: &mut ClosuresFunctionFrontend,
    unit: usize,
    text: &str,
) -> Result<Result<ClosuresFnCandidate, NonFastReason>, Vec<ace_core::atom_text::Diagnostic>> {
    let sha = front.source_sha().to_string();
    let span = front.units()[unit].span;
    front.edit_body(&sha, span.start..span.end, text)
}

#[test]
fn closures_fn_912_lambda_body_edit_single_unit_chain_and_identity() {
    let (mut front, _parts, checked) =
        ClosuresFunctionFrontend::cold(SRC, "main.at", &[]).unwrap();
    // The cold truth still verifies whole and builds the registry.
    assert!(checked.bundle().domain == ace_core::hir::HirDomain::Closures);
    assert_eq!(front.registry().lambda_count(), 2);
    // Sibling lambda identity: f=__closure_main_0, g=__closure_main_1.
    let g = unit_of(&front, "__closure_main_1");
    match &front.units()[g].kind {
        UnitKind::Lambda { site, .. } => {
            assert_eq!(site.ordinal, 1);
            assert_eq!(site.capture_uses, vec![("x".to_string(), false)]);
        }
        other => panic!("expected a lambda unit, got {other:?}"),
    }

    let new_src = edited_source();
    let sha = front.source_sha().to_string();
    let g_span = front.units()[g].span;
    let ((result, body), stages) = body_work::capture_stage_attempts(|| {
        body_work::capture_body_work(|| {
            front.edit_body(&sha, g_span.start..g_span.end, "(n int) => { return x - n * 2 }")
        })
    });
    let candidate = fast(result);
    assert_eq!(candidate.unit_name, "__closure_main_1");
    assert_eq!(candidate.receipt.real_body_plans, 1);
    // The scoped document scaffolded every other unit (5 fns total).
    assert_eq!(candidate.receipt.scaffold_fns, 4);

    // ZERO work for every other unit; exactly one walk of the edited lambda.
    let walked: Vec<&String> = body.keys().collect();
    assert_eq!(
        walked,
        vec![&"__closure_main_1".to_string()],
        "only the edited lambda was walked"
    );
    let work = body.get("__closure_main_1").unwrap();
    assert_eq!(
        (work.resolve, work.typecheck, work.plan, work.bind),
        (1, 1, 1, 1)
    );
    // Whole axis strictly zero; the ONE parse attempt belongs to the edited unit.
    assert_eq!(stages.whole_total(), 0);
    assert_eq!(stages.of("d___closure_main_1", Stage::ParseProject), 1);

    // The REAL credential against the retained registry (HirVerify attempt).
    let ((credential, verify_body), verify_stages) = body_work::capture_stage_attempts(|| {
        body_work::capture_body_work(|| {
            verify_closures_function(
                &candidate.parts.bundle,
                candidate.def_index(),
                front.registry(),
            )
        })
    });
    let credential = credential.expect("credential");
    assert!(verify_body.is_empty());
    assert_eq!(verify_stages.of("d___closure_main_1", Stage::HirVerify), 1);
    assert_eq!(credential.def_id_text(), "d___closure_main_1");
    assert_eq!(credential.domain(), ace_core::hir::HirDomain::Closures);
    // The lambda's own env is a verifier fact (g captures `x` read-only).
    let facts = credential.closure_facts().expect("closure facts");
    let env = facts.env.as_ref().expect("lambda env");
    assert_eq!(env.len(), 1);
    assert!(!env[0].cell);
    assert!(!env[0].mutable);
    // coll facts exist for the closures domain credential.
    assert!(credential.coll_facts().is_some());

    // A scaffold TARGET is refused (`verify.body-ref`): the sibling's
    // credential exists only in a document where IT is the real body — the
    // 828 T01-C04 admission control carried to closures.
    let sibling = verify_closures_function(
        &candidate.parts.bundle,
        candidate.def_index() - 1,
        front.registry(),
    )
    .unwrap_err();
    assert!(
        sibling.iter().any(|d| d.code == "verify.body-ref"),
        "scaffold target must be refused: {sibling:?}"
    );

    // Commit and verify the catalog survived with shifted coordinates.
    let sha_before = front.source_sha().to_string();
    front.commit_candidate(candidate).expect("commit");
    assert_ne!(front.source_sha(), sha_before);
    assert_eq!(front.source(), new_src.as_str());
    assert_eq!(front.generation(), 1);
    // The next edit of the SAME lambda chains (second warm edit).
    let span2 = front.units()[unit_of(&front, "__closure_main_1")].span;
    let text2 = &front.source()[span2.start..span2.end];
    assert!(text2.contains("n * 2"));
    let u = unit_of(&front, "__closure_main_1");
    let candidate2 = fast(edit(
        &mut front,
        u,
        "(n int) => { return x - n * 3 }",
    ));
    assert_eq!(candidate2.unit_name, "__closure_main_1");
    front.commit_candidate(candidate2).expect("commit2");
    assert_eq!(front.generation(), 2);
}

#[test]
fn closures_fn_912_scoped_document_shape_and_whole_verify_refusal() {
    let (mut front, _parts, _checked) =
        ClosuresFunctionFrontend::cold(SRC, "main.at", &[]).unwrap();
    let u = unit_of(&front, "__closure_main_1");
    let candidate = fast(edit(
        &mut front,
        u,
        "(n int) => { return x - n * 2 }",
    ));
    let m = &candidate.parts.bundle.modules[0];
    assert_eq!(m.defs.len(), 5, "the scoped document keeps every def");
    let real = m
        .defs
        .iter()
        .filter(|d| {
            matches!(&d.kind,
                ace_core::hir::DefKind::Function { body, .. }
                    if m.bodies.get(body.0 as usize).is_some())
        })
        .count();
    assert_eq!(
        real, 1,
        "exactly the edited unit's body is real; the rest are scaffolds"
    );
    // The whole-module verifier REFUSES the scoped document (scaffold
    // bodies) — the per-function entry is the only credential source.
    assert!(verify_closures(candidate.parts.bundle.clone()).is_err());
}

#[test]
fn closures_fn_912_forged_env_and_missing_registry_rejected() {
    let (mut front, _parts, _checked) =
        ClosuresFunctionFrontend::cold(SRC, "main.at", &[]).unwrap();
    let u = unit_of(&front, "__closure_main_1");
    let candidate = fast(edit(
        &mut front,
        u,
        "(n int) => { return x - n * 2 }",
    ));
    let index = candidate.def_index();
    let bundle = candidate.parts.bundle.clone();

    // (a) A registry MISSING the lambda: the credential is refused, never
    // silently weakened (this is the check the whole-module path SKIPS when
    // the callee body is a scaffold).
    let empty = ClosuresVerifyContext::default();
    let err = verify_closures_function(&bundle, index, &empty).unwrap_err();
    assert!(
        err.iter().any(|d| d.code == "verify.closure-env-mismatch"
            || d.code == "verify.closure-site"),
        "unexpected diagnostics: {err:?}"
    );

    // (b) A body whose declared env disagrees with the registry: flip the
    // capture local's writability in the document and re-verify.
    let mut forged = bundle.clone();
    {
        let m = &mut forged.modules[0];
        let def = &m.defs[index];
        let ace_core::hir::DefKind::Function { body, .. } = &def.kind else {
            panic!("the unit is a function")
        };
        let body = &mut m.bodies[body.0 as usize];
        for local in &mut body.locals {
            if local.role == ace_core::hir::LocalRole::Capture {
                local.mutable = !local.mutable;
            }
        }
    }
    let err = verify_closures_function(&forged, index, front.registry()).unwrap_err();
    assert!(
        err.iter().any(|d| d.code == "verify.closure-env-mismatch"),
        "mutated env must be rejected: {err:?}"
    );
}

#[test]
fn closures_fn_912_capture_decl_change_is_a_named_non_fast() {
    let (mut front, _parts, _checked) =
        ClosuresFunctionFrontend::cold(SRC, "main.at", &[]).unwrap();
    // f stops WRITING `base`: the raw capture fact changes (written=true →
    // false) — the owner's creation site changed; the fast path declines.
    let u = unit_of(&front, "__closure_main_0");
    let outcome = non_fast(edit(
        &mut front,
        u,
        "(n int) => { return base + n }",
    ));
    match outcome {
        NonFastReason::LambdaSiteChanged(_) => {}
        other => panic!("expected a lambda-site change, got {other:?}"),
    }
    // The frontend is untouched by the decline.
    assert_eq!(front.generation(), 0);
    assert_eq!(front.source(), SRC);
}

#[test]
fn closures_fn_912_stale_base_and_cross_unit_edits_refused() {
    let (mut front, _parts, _checked) =
        ClosuresFunctionFrontend::cold(SRC, "main.at", &[]).unwrap();
    let g = unit_of(&front, "__closure_main_1");
    // Stale base SHA.
    let stale = front.units()[g].span;
    let r = non_fast(front.edit_body("0000", stale.start..stale.end, "(n int) => { return x }"));
    assert_eq!(r, NonFastReason::StaleBase);
    // A range crossing units (owner body + lambda together).
    let main_body = front.units()[unit_of(&front, "main")].body_span;
    let sha = front.source_sha().to_string();
    let reason = non_fast(front.edit_body(
        &sha,
        main_body.start..main_body.start + 20,
        "x int = 10 var base int = 1  ",
    ));
    assert!(
        matches!(reason, NonFastReason::CrossesUnits | NonFastReason::OwnerBodyEdit),
        "cross-unit edit classified as {reason:?}"
    );
    // A type error inside the edited lambda: diagnostics, state preserved.
    let r = match edit(&mut front, g, "(n int) => { return x - true }") {
        Err(ds) => ds,
        Ok(Ok(_)) => panic!("a type error must be diagnostics"),
        Ok(Err(reason)) => panic!("a type error must be diagnostics, not non-fast {reason:?}"),
    };
    assert!(
        r.iter().any(|d| d.stage == ace_core::atom_text::Stage::Bind
            || d.stage == ace_core::atom_text::Stage::Verify),
        "unexpected diagnostics: {r:?}"
    );
    assert_eq!(front.generation(), 0);
    assert_eq!(front.source(), SRC);
    // The next valid edit still works (recovery).
    let recovery = fast(edit(&mut front, g, "(n int) => { return x - n }"));
    assert_eq!(recovery.unit_name, "__closure_main_1");
}

#[test]
fn closures_fn_912_whole_path_axis_counts_the_module_stages() {
    // The forced-whole negative control at the core seam: the whole path's
    // own observation counts the whole axis — a warm apply that routes
    // through it is observably non-zero on the module axis.
    let ((result, _body), stages) = body_work::capture_stage_attempts(|| {
        body_work::capture_body_work(|| {
            let _ = parse_program_with(SRC, SourceProfile::Closures).expect("parse");
            ClosuresFunctionFrontend::cold(SRC, "main.at", &[])
        })
    });
    assert!(result.is_ok());
    assert!(stages.total(Stage::WholeParse) >= 1);
    assert!(stages.total(Stage::WholePlan) >= 1);
    assert!(stages.total(Stage::WholeVerify) >= 1);
}
