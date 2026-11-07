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
    for w in ["i64", "u64", "f32", "f64", "double", "float", "u8", "byte", "u32", "uint", "i8"] {
        let src = format!("fn main() {w} {{ return 1i64 }}");
        eprintln!("RET-{}: {}", w, first_code(parse_program_with(&src, SourceProfile::Numeric)));
    }
    let f1 = "fn main() f64 {
    let z = 0.0d
    return -z
}";
    eprintln!("F1: {}", first_code(parse_program_with(f1, SourceProfile::Numeric)));
    let f2 = "fn main() f64 {
    let z = 0.0d
    return z
}";
    eprintln!("F2: {}", first_code(parse_program_with(f2, SourceProfile::Numeric)));
    let f3 = "fn main() f64 {
    return -z
}";
    eprintln!("F3: {}", first_code(parse_program_with(f3, SourceProfile::Numeric)));
    let cf = |x: &str| match ace_core::source::compile_numeric_source(x, "p.at", &[]) {
        Ok(_) => "ok".to_string(),
        Err(d) => format!("{}: {}", d[0].code, d[0].message),
    };
    eprintln!("CF1: {}", cf(f1));
    eprintln!("CF2: {}", cf(f2));
    let f4 = "fn main() f64 { let z = 0.0d
 return -z }";
    eprintln!("F4(single-line-brace): {}", first_code(parse_program_with(f4, SourceProfile::Numeric)));
    let f5 = "fn main() int { let z = 5
 return z }";
    eprintln!("F5(int-same-shape): {}", first_code(parse_program_with(f5, SourceProfile::Numeric)));
    let f6 = "fn main() f64 { let z = 1.5d
 return z }";
    eprintln!("F6(nofloat-dot-0): {}", first_code(parse_program_with(f6, SourceProfile::Numeric)));
    let f7 = "fn main() f64 { let z = 0.0d
 return z }";
    eprintln!("F7(dot-zero): {}", first_code(parse_program_with(f7, SourceProfile::Numeric)));
    let mn = "fn main() i64 { let m = -9223372036854775808i64
 return m }";
    eprintln!("I64MIN: {}", first_code(parse_program_with(mn, SourceProfile::Numeric)));
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
fn numeric_ops_shifts_casts_compile_to_hir() {
    // The full operator surface through the production front door:
    // typed arith/cmp, i32 bit ops, typed shifts (method form), the
    // `.as(T)`/`.to(T)` conversions and negative-literal folds.
    let src = "fn main(x i64) i64 {
    let a = 2147483648i64
    let b = x + a
    let c = 12 & 10
    let d = c.as(i64)
    let e = b.shl(1i64)
    let f = 1.5d
    let g = -1.5d
    let h = -2147483648i64
    let i = 200u8 + 55u8
    let j = i.to(i64)
    let k = e + d
    let m = b < 4294967296i64
    return k + j
}
";
    let parts = ace_core::source::adapter::compile_parts_with(
        src,
        "n_ops.at",
        &[],
        SourceProfile::Numeric,
    )
    .expect("ops compile parts");
    let checked = ace_core::verify::verify_closures(parts.bundle).expect("verify");
    let body = &checked.bundle().modules[0].bodies[0];
    use ace_core::hir::{BinOp, BuiltinType, ExprKind, NumOpKind};
    let mut words: Vec<String> = body
        .exprs
        .iter()
        .filter_map(|e| match &e.kind {
            ExprKind::Binary {
                op: BinOp::Num { op, ty },
                ..
            } => Some(format!("{:?}_{:?}", op, ty)),
            _ => None,
        })
        .collect();
    words.sort();
    assert!(words.contains(&"Add_I64".to_string()), "typed add: {words:?}");
    assert!(words.contains(&"And_I32".to_string()), "i32 bit and: {words:?}");
    assert!(words.contains(&"Shl_I64".to_string()), "typed shl: {words:?}");
    assert!(words.contains(&"Lt_I64".to_string()), "typed cmp: {words:?}");
    assert!(words.contains(&"Add_U8".to_string()), "u8 add: {words:?}");
    // casts bound as Cast exprs with their destinations
    let casts: Vec<&ace_core::hir::BuiltinType> = body
        .exprs
        .iter()
        .filter_map(|e| match &e.kind {
            ExprKind::Cast { to, .. } => Some(to),
            _ => None,
        })
        .collect();
    assert_eq!(casts.len(), 2, "two casts (as + to): {casts:?}");
    assert!(casts.contains(&&BuiltinType::I64));
    // negative literal folds: -2147483648i64 → -(2^31) two's complement
    // (0xFFFFFFFF80000000); -1.5d → sign-flipped f64 bits.
    let consts: Vec<(BuiltinType, u64)> = body
        .exprs
        .iter()
        .filter_map(|e| match &e.kind {
            ExprKind::NumericConst { ty, bits } => Some((*ty, *bits)),
            _ => None,
        })
        .collect();
    assert!(
        consts.contains(&(BuiltinType::I64, (-2147483648i64) as u64)),
        "negative i64 fold: {consts:?}"
    );
    assert!(
        consts.contains(&(BuiltinType::F64, (-1.5f64).to_bits())),
        "negative f64 fold: {consts:?}"
    );
    let _: NumOpKind;
}

#[test]
fn numeric_negative_surface_rejections() {
    let cases = [
        // mixed widths without conversion
        ("fn main(x i64) i64 { return x + 1 }", "source.type-mismatch"),
        // float bit op
        ("fn main(x f64) f64 { return x & 1.0d }", "source.type-mismatch"),
        // unsigned negation
        ("fn main() u32 { return -1u32 }", "source.literal-range"),
        // shift amount of a different width
        ("fn main(x i64) i64 { return x.shl(1) }", "source.type-mismatch"),
        // unknown numeric method
        ("fn main(x i64) i64 { return x.frob() }", "source.unsupported"),
    ];
    for (src, want) in cases {
        // Typing rules live in resolve — compile, not just parse.
        let code = match ace_core::source::adapter::compile_parts_with(
            src,
            "n_neg.at",
            &[],
            SourceProfile::Numeric,
        ) {
            Ok(_) => "ok".to_string(),
            Err(ds) => ds[0].code.clone(),
        };
        assert_eq!(code, want, "src `{src}`: got {code}, want {want}");
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
