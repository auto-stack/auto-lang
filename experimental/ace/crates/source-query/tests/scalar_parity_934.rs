//! PLAN-934 T-08: product-entry parity over the frozen core SOURCE corpus.
//!
//! The product entry (`compile_scalar_product`) is what the AV/native/A2X
//! hosts now call instead of `ace_core::source::compile_scalar_source`. Its
//! contract with those hosts is:
//!
//! * an ACCEPTED ordinary scalar program always takes the FACTS lane and
//!   emits byte-identical HIR (same profile, same input hash) — so every
//!   downstream end keeps its artifact;
//! * a REJECTED program reports exactly the diagnostics the frontend
//!   reported before the wiring (stage, code, message, span), in order;
//! * the facts lane is the only lane that produces types: no accepted
//!   program may leave it (the `diagnostic_verdicts` counter is the
//!   producer-side observation, not a test-side guess).
//!
//! The corpus is the frozen PLAN-767 scalar-source fixture set: 11 valid
//! programs covering literals/locals/loops/operators/calls and 19 invalid
//! shapes covering parser, bind and verify rejections.

use ace_core::source::{
    compile_scalar_source, resolve_env, resolve_env_with_supply, SourceProfile,
};
use ace_source_query::{compile_scalar_product, CompileLane, SourceQuerySession};
use std::path::PathBuf;

fn fixture(kind: &str, name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../core/fixtures/scalar-source")
        .join(kind)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn diag_key(ds: &[ace_core::atom_text::Diagnostic]) -> Vec<(String, String, String, usize, usize)> {
    ds.iter()
        .map(|d| {
            (
                d.stage.as_str().to_string(),
                d.code.clone(),
                d.message.clone(),
                d.span.start,
                d.span.end,
            )
        })
        .collect()
}

fn product(
    src: &str,
    name: &str,
) -> Result<ace_source_query::ProductCompile, Vec<ace_core::atom_text::Diagnostic>> {
    let mut session = SourceQuerySession::new("sq934", name);
    compile_scalar_product(src, name, &[], "sq934", name, &mut session)
}

const VALID: &[&str] = &[
    "p-gcd-euclid.at",
    "p-count-digits.at",
    "p-is-digit.at",
    "p-cmp-matrix.at",
    "p-a-minus-1.at",
    "p-for-loops.at",
    "p-binop-args.at",
    "p-crlf.at",
    "p-short-circuit-trap.at",
    "p-short-circuit-trace.at",
    "p-short-circuit-contexts.at",
];

const INVALID: &[&str] = &[
    "e01-mixed-eq.at",
    "e02-not-on-int.at",
    "e03-neg-on-bool.at",
    "e04-or-unknown-rhs.at",
    "e05-and-int-rhs.at",
    "e06-and-mixed-types.at",
    "e07-for-nonbool-cond.at",
    "e08-for-in.at",
    "e09-for-init-clause.at",
    "e10-for-unclosed.at",
    "e11-while-in-scalar.at",
    "e12-missing-operand.at",
    "e13-missing-rparen.at",
    "e14-unclosed-call.at",
    "e15-literal-range.at",
    "e16-neg-paren-min.at",
    "e17-unary-plus.at",
    "e18-or-newline.at",
    "e19-neg-newline.at",
];

#[test]
fn sq934_product_entry_keeps_the_corpus_artifacts() {
    for name in VALID {
        let src = fixture("valid", name);
        // The two trace fixtures only compile WITH the trace capability.
        let caps: Vec<String> = if name.starts_with("p-short-circuit-trace")
            || name.starts_with("p-short-circuit-contexts")
        {
            vec![ace_core::source::TRACE_CAPABILITY.to_string()]
        } else {
            Vec::new()
        };
        let legacy = compile_scalar_source(&src, name, &caps)
            .unwrap_or_else(|ds| panic!("legacy must accept {name}: {:?}", diag_key(&ds)));
        let mut session = SourceQuerySession::new("sq934", name);
        let got = compile_scalar_product(&src, name, &caps, "sq934", name, &mut session)
            .unwrap_or_else(|ds| panic!("{name}: product must accept: {:?}", diag_key(&ds)));
        assert_eq!(got.lane, CompileLane::Facts, "{name}: accepted via facts");
        assert_eq!(
            session.counters().diagnostic_verdicts,
            0,
            "{name}: an accepted program never asks for a diagnostic verdict"
        );
        assert_eq!(got.compiled.profile, legacy.profile, "{name}: profile");
        assert_eq!(
            got.compiled.input_hash, legacy.input_hash,
            "{name}: input hash"
        );
        assert_eq!(
            got.compiled.atom_text, legacy.atom_text,
            "{name}: the emitted HIR is byte-identical to the legacy entry"
        );
        // The retained-environment eduction from the SAME facts resolves
        // and carries every function of the legacy environment.
        let program = ace_core::source::parse_program_with(&src, SourceProfile::Scalar)
            .expect("parse for env");
        let factless = resolve_env(&program, &caps, SourceProfile::Scalar).expect("legacy env");
        let factual =
            resolve_env_with_supply(&program, &caps, SourceProfile::Scalar, Some(&got.supply))
                .unwrap_or_else(|ds| {
                    panic!(
                        "{name}: fact-supplied env must resolve: {:?}",
                        diag_key(&ds)
                    )
                });
        assert_eq!(
            ace_core::source::env_fn_count(&factual),
            ace_core::source::env_fn_count(&factless),
            "{name}: the retained environment keeps every function"
        );
        assert_eq!(
            ace_core::source::env_fn_names(&factual),
            ace_core::source::env_fn_names(&factless),
            "{name}: the retained environment keeps the declaration order"
        );
    }
}

#[test]
fn sq934_product_entry_reports_the_legacy_diagnostics_verbatim() {
    for name in INVALID {
        let src = fixture("invalid", name);
        let legacy = compile_scalar_source(&src, name, &[])
            .err()
            .unwrap_or_else(|| panic!("{name}: the corpus shape must be rejected"));
        let got = product(&src, name)
            .err()
            .unwrap_or_else(|| panic!("{name}: the product entry must reject it too"));
        assert_eq!(
            diag_key(&got),
            diag_key(&legacy),
            "{name}: the product reports the frontend's own diagnostic verdict"
        );
    }
}
