//! PLAN-961 T-02 numeric-domain HIR tests: descriptor/schema identity,
//! the typed-constant payload words, typed binary/neg/cast binding, the
//! same-type operand rule and canonical roundtrip. Pure in-process
//! verification — MIR/AV/native/a2x legs live in their own crates.

use ace_core::descriptor;
use ace_core::hir::{self, BinOp, ExprKind, HirDomain};
use ace_core::{numeric_descriptor, verify};

fn fixture(kind: &str, name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/numeric-hir")
        .join(kind)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn bind_numeric_ok(src: &str) -> hir::Bundle {
    let b = descriptor::bind(numeric_descriptor(), src).expect("numeric bind must succeed");
    assert_eq!(
        b.domain,
        HirDomain::Closures,
        "numeric documents verify under the closures-superset domain"
    );
    b
}

#[test]
fn numeric_descriptor_parses_with_the_typed_vocabulary() {
    // The embedded schema must carry the 107 typed BinOp names, the four
    // typed negation words and the numeric TypeKind cases.
    let d = numeric_descriptor();
    assert!(d.is_numeric());
    let bin = d.enum_cases("BinOp").expect("BinOp enum");
    for word in [
        "add_i64", "sub_u32", "mul_i8", "div_u64", "rem_f32", "and_i32", "or_u8", "xor_i64",
        "shl_u32", "shr_i8", "lt_f64", "le_u64", "gt_i8", "ge_f32", "eq_u8", "ne_i64",
    ] {
        assert!(bin.contains(&word.to_string()), "missing BinOp case {word}");
    }
    let un = d.enum_cases("UnaryOp").expect("UnaryOp enum");
    for word in ["neg_i8", "neg_i64", "neg_f32", "neg_f64"] {
        assert!(
            un.contains(&word.to_string()),
            "missing UnaryOp case {word}"
        );
    }
    let tk = d.enum_cases("TypeKind").expect("TypeKind enum");
    for word in ["i8", "u8", "u32", "i64", "u64", "f32", "f64"] {
        assert!(
            tk.contains(&word.to_string()),
            "missing TypeKind case {word}"
        );
    }
    // Float bit-ops are NOT in the vocabulary (frozen rule).
    for word in ["and_f32", "shl_f64", "xor_f64"] {
        assert!(
            !bin.contains(&word.to_string()),
            "float bit op {word} must not be a case"
        );
    }
}

#[test]
fn numeric_ops_bind_and_verify() {
    let bundle = bind_numeric_ok(&fixture("valid", "n01-numeric-ops.atom"));
    let checked = verify::verify_closures(bundle).expect("numeric verify must succeed");
    let body = &checked.bundle().modules[0].bodies[0];
    let mut typed_ops: Vec<String> = body
        .exprs
        .iter()
        .filter_map(|e| match &e.kind {
            ExprKind::Binary {
                op: BinOp::Num { op, ty },
                ..
            } => Some(hir::num_op_atom(*op, *ty)),
            _ => None,
        })
        .collect();
    typed_ops.sort();
    assert_eq!(
        typed_ops,
        vec![
            "add_i64".to_string(),
            "add_i64".to_string(),
            "add_i64".to_string(),
            "add_i64".to_string(),
            "add_u8".to_string(),
            "and_i32".to_string(),
            "lt_i64".to_string(),
        ],
        "the fixture's typed op set must bind exactly"
    );
    // the typed constants carry exact payloads
    let consts: Vec<(hir::BuiltinType, u64)> = body
        .exprs
        .iter()
        .filter_map(|e| match &e.kind {
            ExprKind::NumericConst { ty, bits } => Some((*ty, *bits)),
            _ => None,
        })
        .collect();
    assert_eq!(consts.len(), 7, "seven typed consts: {consts:?}");
    assert!(consts.contains(&(hir::BuiltinType::I64, 2147483648)));
    assert!(consts.contains(&(hir::BuiltinType::U8, 200)));
    // 1.5f64 == 0x3FF8000000000000
    assert!(
        consts.contains(&(hir::BuiltinType::F64, 0x3FF8_0000_0000_0000)),
        "f64 bits exact: {consts:?}"
    );
    // the casts bound as Cast exprs
    let casts = body
        .exprs
        .iter()
        .filter(|e| matches!(e.kind, ExprKind::Cast { .. }))
        .count();
    assert_eq!(casts, 3, "three casts in the fixture");
}

#[test]
fn numeric_canonical_roundtrip_is_stable() {
    let src = fixture("valid", "n01-numeric-ops.atom");
    let b = bind_numeric_ok(&src);
    let text = descriptor::write_canonical(&b, numeric_descriptor()).expect("canonical write");
    let b2 = descriptor::bind(numeric_descriptor(), &text).expect("roundtrip bind");
    let text2 = descriptor::write_canonical(&b2, numeric_descriptor()).expect("second write");
    assert_eq!(text, text2, "canonical form must be a fixed point");
    // the typed words survive the roundtrip verbatim
    assert!(
        text.contains("op: add_i64"),
        "typed op word in canonical: {text}"
    );
    assert!(
        text.contains("value: 4609434218613702656f64"),
        "f64 bits word"
    );
}

#[test]
fn numeric_mixed_width_operands_are_rejected() {
    // Same-type discipline: add_i64 with one i32 operand must fail verify.
    let src = r#"
hir(schema: "auto.hir.numeric.draft", revision: 1u32, profile: "source-numeric-draft") {
    module m_s {
        types {
            type t_i32 i32 {}
            type t_i64 i64 {}
            type t_bool bool {}
            type t_f function { params: []; result: "t_bool" }
        }
        declarations {
            function d_main t_f { name: "nf_mix"; body: "b_main" }
        }
        bodies {
            body b_main {
                owner: "d_main"
                entry: "k0"
                locals {
                }
                expressions {
                    expr e0 constant { type: t_i64; value: 1i64 }
                    expr e1 constant { type: t_i32; value: 2i32 }
                    expr e2 binary { type: t_i64; op: add_i64; overflow: trap; lhs: e0; rhs: e1 }
                    expr e3 binary { type: t_bool; op: lt_i64; lhs: e0; rhs: e0 }
                }
                blocks {
                    block k0 {
                        return(value: e3)
                    }
                }
            }
        }
    }
}
"#;
    let bundle = descriptor::bind(numeric_descriptor(), src).expect("bind");
    let diags = verify::verify_closures(bundle).expect_err("mixed widths must fail");
    assert!(
        diags.iter().any(|d| d.code == "verify.type-mismatch"),
        "mixed-width diagnostic: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}
