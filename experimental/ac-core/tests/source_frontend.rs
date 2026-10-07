//! PLAN-745 T-02 source frontend tests: the 21 positive and 53 negative
//! corpus cases materialized under `fixtures/source/`, plus grammar, span
//! and full-consumption spot checks.
//!
//! Negative cases whose rejection belongs to a later stage (bind/verify)
//! must still PARSE here — the frontend is syntactic only. Text-stage
//! rejections are asserted by diagnostic code and token slice (EOF cases
//! assert the span position instead of a slice).

use auto_ac_prototype::atom_text::{line_col, Stage};
use auto_ac_prototype::source::ast::*;
use auto_ac_prototype::source::parse_program;

fn fixture(kind: &str, name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/source")
        .join(kind)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

const VALID: &[&str] = &[
    "p01-precedence.at",
    "p02-signed-positional.at",
    "p03-bool-param-constant-encoding.at",
    "p04-while-retest.at",
    "p05-while-zero.at",
    "p06-continue-retests-condition.at",
    "p07-break-nearest-loop.at",
    "p08-recursion.at",
    "p09-forward-signature.at",
    "p10-shadow-rhs-outer.at",
    "p11-literal-min-max.at",
    "p12-unexecuted-trap-not-lifted-x1.at",
    "p13-add-overflow-trap.at",
    "p14-mul-overflow-trap.at",
    "p15-named-eval-order-x2.at",
    "p16-observable-before-trap-x9.at",
    "p17-heterogeneous-binding-map.at",
    "p18-bool-return-local-branch.at",
    "p19-user-function-not-intrinsic.at",
    "p20-utf8-crlf-negative-min.at",
    "p21-mutual-recursion.at",
    // review r1 controls: bracketed newline stays legal; native-internal
    // names are legal source names (QA-02)
    "c02-paren-newline.at",
    "c05-ac-start-name.at",
    "c06-exitprocess-name.at",
    // r2 QA-01 legal controls: continuation after a bracketed subexpression
    // keys to the closing line; prefix-colliding names coexist
    "d01-call-multiline-then-add.at",
    "d02-paren-multiline-then-add.at",
    "d03-negative-multiline-arg.at",
    "d04-nested-paren-multiline.at",
    "d05-call-multiline-no-tail.at",
    "d07-colliding-prefix-names.at",
    "e01-paren-nesting-16.at",
    "f04-parens-99.at",
];

/// Negative cases the frontend must accept (rejection happens at bind or
/// verify, later in the pipeline).
const LATER_STAGE: &[&str] = &[
    "n03-undefined-name.at",
    "n04-type-mismatch-return.at",
    "n05-type-mismatch-if-cond.at",
    "n06-type-mismatch-bool-add.at",
    "n07-immutable-write.at",
    "n08-duplicate-local.at",
    "n09-undefined-scope-escape.at",
    "n10-arg-missing.at",
    "n11-arg-duplicate.at",
    "n12-arg-unknown-name.at",
    "n13-arg-mixed.at",
    "n14-type-mismatch-arg.at",
    "n15-break-outside-loop.at",
    "n16-continue-outside-loop.at",
    "n17-no-return-path.at",
    "n18-stmt-after-return.at",
    "n27-not-callable-local.at",
    "n28-not-callable-param.at",
    "n29-not-callable-shadow.at",
    "n32-duplicate-fn.at",
    "n33-duplicate-param.at",
    "n50-undefined-callee.at",
    "n51-type-mismatch-lt-bool.at",
    "c09-both-return-trailing.at",
    "c09-trailing-utf8-crlf.at",
    "c10-scope-escape-token.at",
    // r2 QA-03: later-stage rejections (verify-level)
    "d08-both-diverge-trailing.at",
    "d10-stray-both-branches.at",
    "d13-stray-branches-no-tail.at",
    "d14-dead-after-valid-break.at",
];

/// (fixture, code, token) — token `<EOF>` asserts the span sits at end of
/// input rather than a slice.
const TEXT_ERRORS: &[(&str, &str, &str)] = &[
    (
        "n01-literal-range-max.at",
        "source.literal-range",
        "2147483648",
    ),
    (
        "n02-literal-range-min.at",
        "source.literal-range",
        "-2147483649",
    ),
    (
        "n19-var-uninitialized.at",
        "source.unsupported",
        "var x int",
    ),
    ("n20-char-literal.at", "source.unsupported", "'a'"),
    ("n21-str-type.at", "source.unsupported", "str"),
    ("n22-i64-type.at", "source.unsupported", "i64"),
    ("n23-sub-op.at", "source.unsupported", "-"),
    ("n24-div-op.at", "source.unsupported", "/"),
    ("n25-and-op.at", "source.unsupported", "&&"),
    ("n26-checked-marker.at", "source.unsupported", "checked"),
    ("n30-eof-block.at", "source.syntax", "<EOF>"),
    ("n31-lex-at.at", "source.lex", "@"),
    ("n34-mod-op.at", "source.unsupported", "%"),
    ("n35-eq-op.at", "source.unsupported", "=="),
    ("n36-unary-minus-ident.at", "source.unsupported", "-x"),
    ("n37-suffixed-literal.at", "source.unsupported", "1i32"),
    ("n38-hex-literal.at", "source.unsupported", "0x10"),
    ("n39-toplevel-use.at", "source.unsupported", "use"),
    ("n40-toplevel-let.at", "source.unsupported", "let"),
    ("n41-call-statement.at", "source.unsupported", "mark_a()"),
    ("n42-missing-ret-ann.at", "source.unsupported", "main()"),
    ("n43-list-type.at", "source.unsupported", "List"),
    ("n44-toplevel-type.at", "source.unsupported", "type"),
    ("n45-toplevel-enum.at", "source.unsupported", "enum"),
    ("n46-for-loop.at", "source.unsupported", "for"),
    ("n47-float-literal.at", "source.unsupported", "1.5"),
    ("n48-block-comment.at", "source.unsupported", "/*"),
    ("n49-unicode-ident.at", "source.unsupported", "数"),
    ("n52-tailing-garbage.at", "source.syntax", "unexpected"),
    ("n53-empty-file.at", "source.syntax", "<EOF>"),
    // review r1 QA-01: unbracketed expressions are line-bound (LF/CRLF and
    // trailing-comment variants all reject at the crossing token)
    ("c01-rhs-newline.at", "source.syntax", "2"),
    ("c01-rhs-newline-crlf.at", "source.syntax", "2"),
    ("c01-rhs-newline-comment.at", "source.syntax", "2"),
    ("c03-call-newline.at", "source.syntax", "("),
    // review r1 QA-04: escaped literals lex as ONE excluded token; `!` is
    // an excluded operator; full-language keywords are never names
    ("c04-escaped-string.at", "source.unsupported", "\"a\\\"b\""),
    ("c15-logical-not.at", "source.unsupported", "!"),
    ("c12-keyword-name.at", "source.unsupported", "type"),
    ("c23-keyword-param-name.at", "source.unsupported", "mut"),
    ("c24-keyword-local-name.at", "source.unsupported", "enum"),
    ("c25-escaped-char.at", "source.unsupported", "'\\''"),
    // r2 QA-01/QA-04 rejects
    ("d06-unbracketed-negative-newline.at", "source.syntax", "2"),
    ("d12-keyword-named-arg.at", "source.unsupported", "type"),
    ("d11-escaped-char.at", "source.unsupported", "'\\''"),
];

#[test]
fn valid_corpus_parses() {
    for name in VALID {
        let src = fixture("valid", name);
        match parse_program(&src) {
            Ok(_) => {}
            Err(d) => panic!("{}: unexpected parse error: {}", name, d.render(name, &src)),
        }
    }
}

#[test]
fn later_stage_cases_parse() {
    for name in LATER_STAGE {
        let src = fixture("invalid", name);
        match parse_program(&src) {
            Ok(_) => {}
            Err(d) => panic!(
                "{}: frontend rejected a case owned by a later stage: {}",
                name,
                d.render(name, &src)
            ),
        }
    }
}

#[test]
fn text_stage_rejections() {
    for (name, code, token) in TEXT_ERRORS {
        let src = fixture("invalid", name);
        let err = match parse_program(&src) {
            Ok(_) => panic!("{}: expected parse failure", name),
            Err(d) => d,
        };
        assert_eq!(err.stage, Stage::Text, "{}: stage", name);
        assert_eq!(&err.code, code, "{}: code", name);
        if *token == "<EOF>" {
            assert_eq!(err.span.start, src.len(), "{}: span at EOF", name);
            assert_eq!(err.span.end, src.len(), "{}: span at EOF", name);
        } else {
            assert_eq!(err.span.slice(&src), *token, "{}: token slice", name);
        }
    }
}

// -- grammar / span spot checks ---------------------------------------------

#[test]
fn p01_precedence_shape() {
    let src = fixture("valid", "p01-precedence.at");
    let p = parse_program(&src).unwrap();
    assert_eq!(p.items.len(), 1);
    let f = &p.items[0];
    assert_eq!(f.name.name, "main");
    assert!(f.params.is_empty());
    assert_eq!(f.ret.word, TypeWord::Int);
    assert_eq!(f.body.stmts.len(), 1);
    let Stmt::Return { value, .. } = &f.body.stmts[0] else {
        panic!("expected return")
    };
    // 2 + 3 * 4 => Add(2, Mul(3, 4)); * binds tighter than +.
    let ExprKind::Binary {
        op: BinOp::Add,
        lhs,
        rhs,
    } = &value.kind
    else {
        panic!("expected add at the top")
    };
    assert!(matches!(&lhs.kind, ExprKind::IntLit(2)));
    assert!(matches!(&rhs.kind, ExprKind::Binary { op: BinOp::Mul, .. }));
    assert_eq!(value.span.slice(&src), "2 + 3 * 4");
}

#[test]
fn p02_functions_and_signed_args() {
    let src = fixture("valid", "p02-signed-positional.at");
    let p = parse_program(&src).unwrap();
    assert_eq!(p.items.len(), 2);
    assert_eq!(p.items[0].name.name, "add");
    assert_eq!(p.items[0].params.len(), 2);
    assert_eq!(p.items[0].params[0].name.name, "a");
    assert_eq!(p.items[0].params[1].ty.word, TypeWord::Int);
    let Stmt::Return { value, .. } = &p.items[0].body.stmts[0] else {
        panic!("add body")
    };
    let ExprKind::Binary {
        op: BinOp::Add,
        lhs,
        rhs,
    } = &value.kind
    else {
        panic!("add expr")
    };
    assert!(matches!(lhs.kind, ExprKind::Var(_)));
    assert!(matches!(rhs.kind, ExprKind::Var(_)));
    // main returns add(-2, 3): the negative literal is one token pair.
    let Stmt::Return { value, .. } = &p.items[1].body.stmts[0] else {
        panic!("main body")
    };
    let ExprKind::Call { callee, args } = &value.kind else {
        panic!("call")
    };
    assert_eq!(callee.name, "add");
    assert_eq!(args.len(), 2);
    assert!(matches!(&args[0], Arg::Pos(e) if matches!(e.kind, ExprKind::IntLit(-2))));
    assert!(matches!(&args[0], Arg::Pos(e) if e.span.slice(&src) == "-2"));
}

#[test]
fn p11_literal_domain() {
    let src = fixture("valid", "p11-literal-min-max.at");
    let p = parse_program(&src).unwrap();
    assert_eq!(p.items.len(), 1);
    let stmts = &p.items[0].body.stmts;
    assert_eq!(stmts.len(), 3);
    let Stmt::Let {
        name,
        ann,
        value,
        mutable,
        ..
    } = &stmts[0]
    else {
        panic!("let x")
    };
    assert_eq!(name.name, "x");
    assert!(ann.is_none());
    assert!(!*mutable);
    assert!(matches!(value.kind, ExprKind::IntLit(-2147483648)));
    assert_eq!(value.span.slice(&src), "-2147483648");
    let Stmt::Let {
        name,
        ann,
        value,
        mutable,
        ..
    } = &stmts[1]
    else {
        panic!("let y")
    };
    assert_eq!(name.name, "y");
    assert_eq!(ann.map(|a| a.word), Some(TypeWord::I32));
    assert!(!*mutable);
    assert!(matches!(value.kind, ExprKind::IntLit(2147483647)));
    assert!(matches!(&stmts[2], Stmt::If { .. }));
}

#[test]
fn p15_named_args_preserve_source_order() {
    let src = fixture("valid", "p15-named-eval-order-x2.at");
    let p = parse_program(&src).unwrap();
    let Stmt::Return { value, .. } = &p.items[1].body.stmts[0] else {
        panic!("main body")
    };
    let ExprKind::Call { args, .. } = &value.kind else {
        panic!("call")
    };
    assert_eq!(args.len(), 2);
    let Arg::Named {
        name: n0,
        value: v0,
        ..
    } = &args[0]
    else {
        panic!("named b")
    };
    assert_eq!(n0.name, "b");
    let ExprKind::Call { callee, .. } = &v0.kind else {
        panic!("mark_b call")
    };
    assert_eq!(callee.name, "mark_b");
    let Arg::Named {
        name: n1,
        value: v1,
        ..
    } = &args[1]
    else {
        panic!("named a")
    };
    assert_eq!(n1.name, "a");
    let ExprKind::Call { callee, .. } = &v1.kind else {
        panic!("mark_a call")
    };
    assert_eq!(callee.name, "mark_a");
    assert_eq!(arg_span(&args[0]).slice(&src), "b: mark_b()");
    assert_eq!(arg_span(&args[1]).slice(&src), "a: mark_a()");
}

fn arg_span(a: &Arg) -> auto_ac_prototype::atom_text::Span {
    match a {
        Arg::Pos(e) => e.span,
        Arg::Named { span, .. } => *span,
    }
}

#[test]
fn p20_crlf_utf8_spans() {
    let src = fixture("valid", "p20-utf8-crlf-negative-min.at");
    assert!(src.contains("中文注释"));
    let p = parse_program(&src).unwrap();
    let f = &p.items[0];
    assert_eq!(f.ret.word, TypeWord::I32);
    let Stmt::Let {
        ann, value, span, ..
    } = &f.body.stmts[0]
    else {
        panic!("let n")
    };
    assert_eq!(ann.map(|a| a.word), Some(TypeWord::Int));
    // `- 2147483648` with whitespace between sign and digits is ONE literal.
    assert!(matches!(value.kind, ExprKind::IntLit(-2147483648)));
    assert_eq!(value.span.slice(&src), "- 2147483648");
    assert_eq!(span.slice(&src), "let n int = - 2147483648");
    // Line/column through the shared renderer: the sign sits on line 3.
    assert_eq!(line_col(&src, value.span.start), (3, 14));
    let Stmt::If { cond, els, .. } = &f.body.stmts[1] else {
        panic!("if")
    };
    assert!(els.is_some());
    assert_eq!(cond.span.slice(&src), "n < 0");
    // CRLF: the if condition starts after a \r\n pair on line 4 (one
    // leading space, `n` lands at column 5).
    assert_eq!(line_col(&src, cond.span.start), (4, 5));
}

#[test]
fn p07_nested_loops_break_nearest() {
    let src = fixture("valid", "p07-break-nearest-loop.at");
    let p = parse_program(&src).unwrap();
    let Stmt::Let { .. } = &p.items[0].body.stmts[0] else {
        panic!("let n")
    };
    let Stmt::Loop { body: outer, .. } = &p.items[0].body.stmts[1] else {
        panic!("outer loop")
    };
    let Stmt::Loop { body: inner, .. } = &outer.stmts[0] else {
        panic!("inner loop")
    };
    let Stmt::Assign { name, .. } = &inner.stmts[0] else {
        panic!("assign n")
    };
    assert_eq!(name.name, "n");
    assert!(matches!(&inner.stmts[1], Stmt::Break { .. }));
    let Stmt::Assign { .. } = &outer.stmts[1] else {
        panic!("outer assign")
    };
    assert!(matches!(&outer.stmts[2], Stmt::Break { .. }));
    let Stmt::Return { .. } = &p.items[0].body.stmts[2] else {
        panic!("return")
    };
    // Spans: the inner break keyword slice is exactly "break".
    let Stmt::Break { span } = &inner.stmts[1] else {
        unreachable!()
    };
    assert_eq!(span.slice(&src), "break");
}

#[test]
fn p04_while_and_assignment() {
    let src = fixture("valid", "p04-while-retest.at");
    let p = parse_program(&src).unwrap();
    let stmts = &p.items[0].body.stmts;
    assert_eq!(stmts.len(), 3);
    let Stmt::Let {
        mutable,
        name,
        value,
        ..
    } = &stmts[0]
    else {
        panic!("var i")
    };
    assert!(*mutable, "var declares a mutable local");
    assert_eq!(name.name, "i");
    assert!(matches!(value.kind, ExprKind::IntLit(0)));
    let Stmt::While { cond, body, .. } = &stmts[1] else {
        panic!("while")
    };
    let ExprKind::Binary { op: BinOp::Lt, .. } = &cond.kind else {
        panic!("cond")
    };
    assert_eq!(cond.span.slice(&src), "i < 3");
    let Stmt::Assign { name, value, .. } = &body.stmts[0] else {
        panic!("assign")
    };
    assert_eq!(name.name, "i");
    assert_eq!(value.span.slice(&src), "i + 1");
    assert!(matches!(&stmts[2], Stmt::Return { .. }));
}

#[test]
fn n03_name_span_line_col_for_later_stages() {
    let src = fixture("invalid", "n03-undefined-name.at");
    let p = parse_program(&src).unwrap();
    let Stmt::Return { value, .. } = &p.items[0].body.stmts[0] else {
        panic!("return")
    };
    let ExprKind::Var(v) = &value.kind else {
        panic!("var")
    };
    assert_eq!(v.name, "missing");
    assert_eq!(v.span.slice(&src), "missing");
    // Byte-accurate projection: line 2, column 24 through the shared
    // renderer (N03's pinned coordinates).
    assert_eq!(line_col(&src, v.span.start), (2, 24));
}

#[test]
fn n31_lex_error_renders_position() {
    let src = fixture("invalid", "n31-lex-at.at");
    let err = parse_program(&src).unwrap_err();
    assert_eq!(err.code, "source.lex");
    assert_eq!(err.span.slice(&src), "@");
    assert_eq!(line_col(&src, err.span.start), (1, 24));
}

#[test]
fn review_counterexample_coordinates() {
    // QA-03: with a UTF-8 comment prefix and CRLF endings, spans still
    // resolve to byte-accurate line/column through the shared renderer.
    // c10 (bind-stage) parses at text level; its token position is pinned
    // by source_hir. Here the dead `let` of the CRLF variant is located.
    let src = fixture("invalid", "c10-scope-escape-token.at");
    let p = parse_program(&src).unwrap();
    let Stmt::Return { value, .. } = &p.items[0].body.stmts[1] else {
        panic!("second statement must be the return");
    };
    assert_eq!(value.span.slice(&src), "x");
    assert_eq!(line_col(&src, value.span.start), (3, 9));

    let src = fixture("invalid", "c09-trailing-utf8-crlf.at");
    let p = parse_program(&src).unwrap();
    // the dead `let x = 2` sits on line 4 (after comment/fn/if lines)
    let Stmt::Let { span, .. } = &p.items[0].body.stmts[1] else {
        panic!("second statement must be the let");
    };
    assert_eq!(span.slice(&src), "let x = 2");
    assert_eq!(line_col(&src, span.start), (4, 2));
}

#[test]
fn deep_unclosed_nesting_is_rejected_without_overflow() {
    // R2-QA-05: ~4KB of unclosed parentheses must yield a controlled
    // diagnostic (bounded nesting), never a stack overflow.
    let src = format!("fn main() int {{ return {}1", "(".repeat(4096));
    let err = match parse_program(&src) {
        Ok(_) => panic!("deep unclosed input must be rejected"),
        Err(d) => d,
    };
    assert_eq!(err.code, "source.syntax");
    assert!(err.message.contains("nesting"), "{}", err.message);

    // and deeply unclosed BLOCKS too
    let src = format!("fn main() int {{{}return 0", "{".repeat(5000));
    let err = match parse_program(&src) {
        Ok(_) => panic!("deep unclosed blocks must be rejected"),
        Err(d) => d,
    };
    assert_eq!(err.code, "source.syntax");
}
