//! PLAN-767 T-03 scalar-source static tests: the new operator surface and
//! the conditional `for` loop through the real frontend
//! (parse → resolve → typecheck → scalar Atom bind → verify_scalar).
//! Native execution of these fixtures lives in `scalar_native.rs`; the
//! CLI-level dual-profile isolation lives there too.

use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::hir::HirDomain;
use auto_ac_prototype::source::{
    compile_scalar_source, parse_program_with, SourceProfile, TRACE_CAPABILITY,
};

fn fixture(kind: &str, name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("experimental/ac-core/fixtures/scalar-source")
        .join(kind)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn codes(ds: &[Diagnostic]) -> Vec<&str> {
    ds.iter().map(|d| d.code.as_str()).collect()
}

fn no_caps() -> Vec<String> {
    Vec::new()
}

fn trace_caps() -> Vec<String> {
    vec![TRACE_CAPABILITY.to_string()]
}

// -- positives: every fixture compiles through the full static pipeline ----

#[test]
fn scalar_positive_corpus_compiles() {
    let plain = [
        "p-gcd-euclid.at",
        "p-count-digits.at",
        "p-is-digit.at",
        "p-cmp-matrix.at",
        "p-a-minus-1.at",
        "p-for-loops.at",
        "p-binop-args.at",
        "p-crlf.at",
        "p-short-circuit-trap.at",
    ];
    for name in plain {
        let src = fixture("valid", name);
        let c = compile_scalar_source(&src, name, &no_caps())
            .unwrap_or_else(|ds| panic!("{}: {:?}", name, codes(&ds)));
        assert_eq!(c.profile, "auto.source.scalar-i32.draft", "{}", name);
        assert_eq!(c.checked.domain(), HirDomain::Scalar, "{}", name);
        // the generated Atom really carries the scalar identity triple
        assert!(c.atom_text.contains("auto.hir.scalar.draft"), "{}", name);
    }
    // trace fixtures resolve their intrinsics only with the capability
    for name in ["p-short-circuit-trace.at", "p-short-circuit-contexts.at"] {
        let src = fixture("valid", name);
        let ds = compile_scalar_source(&src, name, &no_caps()).expect_err(name);
        assert!(
            ds.iter().any(|d| d.code == "source.undefined"),
            "{} without capability: {:?}",
            name,
            codes(&ds)
        );
        let c = compile_scalar_source(&src, name, &trace_caps())
            .unwrap_or_else(|ds| panic!("{}: {:?}", name, codes(&ds)));
        assert_eq!(
            c.checked.required_capabilities(),
            &[TRACE_CAPABILITY.to_string()],
            "{}",
            name
        );
    }
}

// -- negatives: stage/code (and token anchoring where pinned) ---------------

#[test]
fn scalar_negative_matrix() {
    // (fixture, expected code, expected span text when token-anchored)
    let cases: &[(&str, &str, Option<&str>)] = &[
        ("e01-mixed-eq.at", "source.type-mismatch", None),
        ("e02-not-on-int.at", "source.type-mismatch", None),
        ("e03-neg-on-bool.at", "source.type-mismatch", None),
        (
            "e04-or-unknown-rhs.at",
            "source.undefined",
            // the never-executed RHS is still resolved: the callee token
            Some("unknown"),
        ),
        ("e05-and-int-rhs.at", "source.type-mismatch", None),
        ("e06-and-mixed-types.at", "source.type-mismatch", None),
        ("e07-for-nonbool-cond.at", "source.type-mismatch", None),
        ("e08-for-in.at", "source.syntax", Some("in")),
        ("e09-for-init-clause.at", "source.syntax", Some("=")),
        ("e10-for-unclosed.at", "source.syntax", None),
        (
            "e11-while-in-scalar.at",
            "source.unsupported",
            Some("while"),
        ),
        ("e12-missing-operand.at", "source.syntax", None),
        ("e13-missing-rparen.at", "source.syntax", None),
        ("e14-unclosed-call.at", "source.syntax", None),
        ("e15-literal-range.at", "source.literal-range", None),
        ("e16-neg-paren-min.at", "source.literal-range", None),
        ("e17-unary-plus.at", "source.unsupported", Some("+")),
        ("e18-or-newline.at", "source.syntax", None),
        ("e19-neg-newline.at", "source.syntax", None),
    ];
    for (name, want, span_text) in cases {
        let src = fixture("invalid", name);
        let ds = match compile_scalar_source(&src, name, &no_caps()) {
            Err(ds) => ds,
            Ok(_) => panic!("{}: expected rejection", name),
        };
        let d = ds
            .iter()
            .find(|d| d.code == *want)
            .unwrap_or_else(|| panic!("{}: wanted {} in {:?}", name, want, codes(&ds)));
        if let Some(text) = span_text {
            assert_eq!(
                d.span.slice(&src),
                *text,
                "{}: diagnostic must anchor at the `{}` token",
                name,
                text
            );
        }
    }
}

// -- V-01 (parser half): the core surface rejects scalar-only syntax --------

#[test]
fn core_surface_rejects_scalar_syntax_at_the_token() {
    let cases: &[(&str, &str)] = &[
        ("fn main() int { let x = 1 - 2\nreturn x }", "operator `-`"),
        ("fn main() int { let x = 1 / 2\nreturn x }", "operator `/`"),
        ("fn main() int { let x = 1 % 2\nreturn x }", "operator `%`"),
        ("fn main() int { let x = 1 > 2\nreturn x }", "operator `>`"),
        (
            "fn main() int { let x = 1 <= 2\nreturn x }",
            "operator `<=`",
        ),
        (
            "fn main() int { let x = 1 == 2\nreturn x }",
            "operator `==`",
        ),
        (
            "fn main() int { let x = 1 != 2\nreturn x }",
            "operator `!=`",
        ),
        (
            "fn main() bool { let x = true && false\nreturn x }",
            "operator `&&`",
        ),
        (
            "fn main() bool { let x = true || false\nreturn x }",
            "operator `||`",
        ),
        (
            "fn f(a int) int { return -a }",
            "unary minus is only supported directly before an integer literal",
        ),
        ("fn main() bool { return !true }", "unary `!`"),
        ("fn main() int { for true { }\nreturn 0 }", "`for`"),
    ];
    for (src, needle) in cases {
        let d = parse_program_with(src, SourceProfile::Core).expect_err(src);
        assert_eq!(d.code, "source.unsupported", "{}: {:?}", src, d.code);
        assert!(d.message.contains(needle), "{}: {}", src, d.message);
    }
    // while stays legal on the core surface
    parse_program_with(
        "fn main() int { var i = 0\nwhile i < 1 { i = i + 1 }\nreturn i }",
        SourceProfile::Core,
    )
    .expect("core while stays accepted");
    // and the scalar surface takes the same while source and rejects it
    let d = parse_program_with(
        "fn main() int { var i = 0\nwhile i < 1 { i = i + 1 }\nreturn i }",
        SourceProfile::Scalar,
    )
    .expect_err("scalar while rejected");
    assert_eq!(d.code, "source.unsupported");
    assert!(d.span.start < 40, "anchored at the while keyword");
}

// -- V-09: deep inputs through the full static pipeline ---------------------

#[test]
fn deep_logic_chain_reaches_checked_bundle() {
    let n = 16384usize;
    let src = format!(
        "fn main() bool {{\n    let x = true{}\n    return x\n}}\n",
        " && true".repeat(n - 1)
    );
    let c = compile_scalar_source(&src, "deep-and.at", &no_caps())
        .unwrap_or_else(|ds| panic!("{:?}", codes(&ds)));
    assert_eq!(c.checked.domain(), HirDomain::Scalar);
}

#[test]
fn deep_not_chain_reaches_checked_bundle() {
    let src = format!(
        "fn main() bool {{\n    let x = {}true\n    return x\n}}\n",
        "!".repeat(20_000)
    );
    compile_scalar_source(&src, "deep-not.at", &no_caps())
        .unwrap_or_else(|ds| panic!("{:?}", codes(&ds)));
}

#[test]
fn deep_mixed_calls_and_logic_reach_checked_bundle() {
    let depth = 4096usize;
    let src = format!(
        "fn id(x bool) bool {{ return x }}\nfn main() bool {{\n    let x = {}true && true{}\n    return x\n}}\n",
        "id(".repeat(depth),
        ")".repeat(depth)
    );
    compile_scalar_source(&src, "deep-mixed.at", &no_caps())
        .unwrap_or_else(|ds| panic!("{:?}", codes(&ds)));
}

#[test]
fn deep_unclosed_structures_are_controlled_rejections() {
    let cases: Vec<String> = vec![
        format!(
            "fn main() int {{\n    let x = {}1\n    return x\n}}\n",
            "(".repeat(4096)
        ),
        format!(
            "fn main() int {{\n    let x = {}1\n    return x\n}}\n",
            "f(".repeat(4096)
        ),
        format!(
            "fn main() bool {{\n    let x = {}true\n    return x\n}}\n",
            "!(".repeat(2048)
        ),
        format!(
            "fn main() bool {{\n    let x = true{}true\n    return x\n}}\n",
            " && ".repeat(8192)
        ),
    ];
    for src in cases {
        let ds = compile_scalar_source(&src, "deep-bad.at", &no_caps())
            .expect_err("unclosed deep structure must reject");
        assert!(
            ds.iter()
                .all(|d| d.stage == auto_ac_prototype::atom_text::Stage::Text),
            "controlled text-stage rejection, got {:?}",
            codes(&ds)
        );
    }
}
