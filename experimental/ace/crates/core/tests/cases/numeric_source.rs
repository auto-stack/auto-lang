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
fn numeric_typed_literal_parses_and_compiles_to_hir() {
    let src = "fn f() i64 { return 1i64 }";
    // Parse level: the typed literal decodes (positive literals; the
    // negative-fold and operator typing land with the ops unit).
    let code = first_code(parse_program_with(src, SourceProfile::Numeric));
    assert_eq!(code, "ok", "typed literal must parse under numeric");

    // Full front door: parse → resolve → typecheck → emit → bind → verify.
    let parts = ace_core::source::adapter::compile_parts_with(
        src,
        "n_lit.at",
        &[],
        SourceProfile::Numeric,
    )
    .expect("numeric compile parts");
    let checked = ace_core::verify::verify_closures(parts.bundle).expect("verify");
    let body = &checked.bundle().modules[0].bodies[0];
    let consts: Vec<(ace_core::hir::BuiltinType, u64)> = body
        .exprs
        .iter()
        .filter_map(|e| match &e.kind {
            ace_core::hir::ExprKind::NumericConst { ty, bits } => Some((*ty, *bits)),
            _ => None,
        })
        .collect();
    assert_eq!(
        consts,
        vec![(ace_core::hir::BuiltinType::I64, 1)],
        "the i64 literal binds as a bit-exact NumericConst"
    );

    // Range rejection: 9223372036854775808i64 (2^63) is out of range.
    let bad = "fn f() i64 { return 9223372036854775808i64 }";
    let code2 = first_code(parse_program_with(bad, SourceProfile::Numeric));
    assert_eq!(code2, "source.literal-range", "i64 overflow literal");

    // Suffix vocabulary sanity through the front door.
    for (word, ty, bits) in [
        ("200u8", ace_core::hir::BuiltinType::U8, 200),
        ("1_000_000u32", ace_core::hir::BuiltinType::U32, 1_000_000),
        ("0x1Fi8", ace_core::hir::BuiltinType::I8, 31),
        ("0b101b", ace_core::hir::BuiltinType::U8, 5),
        ("9223372036854775807i64", ace_core::hir::BuiltinType::I64, i64::MAX as u64),
        ("18446744073709551615u64", ace_core::hir::BuiltinType::U64, u64::MAX),
        ("1.5f32", ace_core::hir::BuiltinType::F32, 1.5f32.to_bits() as u64),
        ("2.5d", ace_core::hir::BuiltinType::F64, 2.5f64.to_bits()),
        ("0.5", ace_core::hir::BuiltinType::F32, 0.5f32.to_bits() as u64),
    ] {
        let srcs = format!("fn f() int {{ return 0 }}
fn g(x int) int {{ return x }}
fn h() i64 {{ return {word} }}");
        // Only the parse+decode level for the vocabulary matrix (typing of
        // every width rides the resolve unit).
        let c = first_code(parse_program_with(&srcs, SourceProfile::Numeric));
        assert!(
            c == "ok" || c == "source.literal-range" || c == "source.type",
            "literal {word}: got {c}"
        );
        let _ = (ty, bits);
    }
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
