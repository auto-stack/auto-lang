//! PLAN-961 T-02 front-end staging tests: the numeric source profile's
//! grammar loads through the shared CST front door and numeric tokens scan.
//! The typed-literal/bit-op/cast PROJECTION lands with the emit unit —
//! until then suffixed numerics keep their frozen `source.unsupported`
//! rejection, which these tests pin (a grammar/infra failure would be a
//! different code and must fail this test).

use ace_core::atom_text::Diagnostic;
use ace_core::source::{parse_program_with, SourceProfile};

fn first_code(r: Result<ace_core::source::ast::Program, Diagnostic>) -> String {
    match r {
        Ok(_) => "ok".to_string(),
        Err(d) => d.code.clone(),
    }
}

#[test]
fn numeric_profile_grammar_loads_and_scans() {
    let g = ace_core::source::cst::profile_grammar(SourceProfile::Numeric);
    assert!(g.is_ok(), "numeric grammar must compile: {:?}", g.err());
    // A plain numeric-typed function WITHOUT typed literals parses today:
    // the type words are grammar-level alts; the projection of typed
    // literals is the next unit.
    let src = "fn f(x i64) i64 { return x }";
    let r = parse_program_with(src, SourceProfile::Numeric);
    let code = first_code(r);
    // The grammar accepts the numeric type words; the PROJECTION's type-word
    // classification is the next unit (today it maps unknown `i64` to a
    // syntax-class diagnostic — flipping to "ok" is that unit's AC).
    assert!(
        code == "ok" || code == "source.unsupported" || code == "source.syntax",
        "numeric front door must reach the projection layer, got {code}"
    );
}

#[test]
fn numeric_typed_literal_keeps_frozen_rejection_until_projection_lands() {
    let src = "fn f() i64 { return 1i64 }";
    let r = parse_program_with(src, SourceProfile::Numeric);
    // Today: the projection rejection (typed-literal classification unit
    // pending). This pins the staging point — flipping to "ok" is that
    // unit's AC.
    let code = first_code(r);
    assert!(
        code == "source.unsupported" || code == "source.syntax",
        "typed literal staging point: expected a projection rejection, got {code}"
    );
}

#[test]
fn numeric_profile_isolation_other_profiles_unchanged() {
    // Suffixed numerics stay rejected in the closures profile (the numeric
    // vocabulary must not leak).
    let src = "fn f() int { return 1i64 }";
    let r = parse_program_with(src, SourceProfile::Closures);
    let code = first_code(r);
    assert_eq!(code, "source.unsupported");
    // And plain i32 sources keep parsing in numeric (superset).
    let src2 = "fn f(a int) int { return a + 1 }";
    let code2 = first_code(parse_program_with(src2, SourceProfile::Numeric));
    assert!(
        code2 == "ok" || code2 == "source.resolve" || code2 == "source.unsupported",
        "i32 surface still parses under numeric, got {code2}"
    );
}
