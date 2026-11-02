//! PLAN-912 T-02: the lowering half of the minimal full chain. A lambda body
//! edit's unit lowered from a VERIFIER-MINTED credential + the registry
//! facts (`lower_function_closures_unit_checked`) must be byte-identical to
//! the same unit lowered from the whole-module checked bundle (the
//! 865/909 path) — same mapping, trusted source instead of re-derivation.
//! Negative controls: no closure facts on the credential, a foreign-body
//! document, a non-closures credential.

use ace_av_core::abc::encode_abc;
use ace_core::source::closures_fn::ClosuresFunctionFrontend;
use ace_core::source::{compile_closures_source, SourceProfile};
use ace_core::verify::verify_closures_function;
use ace_lowering::{lower_function_closures_unit, lower_function_closures_unit_checked};

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

fn spanless(unit: &ace_av_core::mir::MirModule) -> Vec<u8> {
    let mut identity = unit.clone();
    for function in &mut identity.funcs {
        function.spans.clear();
    }
    encode_abc(&identity).expect("encode").to_flat()
}

fn edit_lambda(front: &mut ClosuresFunctionFrontend, text: &str) -> ace_core::source::closures_fn::ClosuresFnCandidate {
    let sha = front.source_sha().to_string();
    let unit = front
        .units()
        .iter()
        .position(|u| u.name == "__closure_main_1")
        .expect("g unit");
    let span = front.units()[unit].span;
    match front.edit_body(&sha, span.start..span.end, text) {
        Ok(Ok(candidate)) => candidate,
        Ok(Err(reason)) => panic!("expected the fast path, got non-fast {reason:?}"),
        Err(ds) => panic!("expected the fast path, got diagnostics {ds:?}"),
    }
}

#[test]
fn closures_fn_912_checked_unit_matches_whole_module_derivation() {
    let (mut front, _parts, _checked) =
        ClosuresFunctionFrontend::cold(SRC, "main.at", &[]).unwrap();
    let candidate = edit_lambda(&mut front, "(n int) => { return x - n * 2 }");
    let credential = verify_closures_function(
        &candidate.parts.bundle,
        candidate.def_index(),
        front.registry(),
    )
    .expect("credential");
    assert_eq!(credential.def_id_text(), "d___closure_main_1");

    // The checked path: scoped document + credential facts.
    let checked_unit = lower_function_closures_unit_checked(
        &candidate.parts.bundle,
        &credential,
        &[],
        7,
    )
    .expect("checked lower");

    // The 865/909 truth path: the whole-module checked bundle of the EDITED
    // source (the edit is semantically a body edit; compile the new source
    // whole) lowered the classic way must produce the same unit bytes.
    let new_src = SRC.replace("return x - n }", "return x - n * 2 }");
    let compiled = compile_closures_source(&new_src, "main.at", &[]).expect("whole compile");
    let whole_unit =
        lower_function_closures_unit(compiled.checked.bundle(), "d___closure_main_1", &[], 7)
            .expect("whole lower");
    assert_eq!(spanless(&checked_unit), spanless(&whole_unit));

    // The creator-side equivalence is exercised by the AV session test
    // (its cold route lowers every unit through the classic path and the
    // warm route through the credential path on the same store).
}

#[test]
fn closures_fn_912_uncredentialed_and_foreign_documents_rejected() {
    let (mut front, _parts, _checked) =
        ClosuresFunctionFrontend::cold(SRC, "main.at", &[]).unwrap();
    let candidate = edit_lambda(&mut front, "(n int) => { return x - n * 2 }");
    let credential = verify_closures_function(
        &candidate.parts.bundle,
        candidate.def_index(),
        front.registry(),
    )
    .expect("credential");

    // A WHOLE-MODULE verify (no closure facts) cannot seed the checked path.
    let whole = compile_closures_source(SRC, "main.at", &[]).expect("compile");
    let whole_fn = ace_core::verify::verify_function(whole.checked.bundle().clone(), 4)
        .expect_err("closures domain refused by the generic entry");
    assert!(whole_fn.iter().any(|d| d.code == "verify.domain"));

    // A document whose body differs from the credential's verified body.
    let other = edit_lambda(&mut front, "(n int) => { return x - n * 5 }");
    let err = lower_function_closures_unit_checked(
        &other.parts.bundle,
        &credential,
        &[],
        7,
    )
    .expect_err("foreign body must be refused");
    assert_eq!(err.code(), "lower.closure-facts");

    // The credential's own document lowers fine (control).
    lower_function_closures_unit_checked(&candidate.parts.bundle, &credential, &[], 7)
        .expect("own document lowers");
}
