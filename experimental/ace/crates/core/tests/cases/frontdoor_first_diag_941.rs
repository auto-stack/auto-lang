//! F-941-11: the shared front door's first diagnostic equals the handwritten
//! oracle byte-for-byte even when a legal lambda precedes the real error
//! (whole source, lambda fragment and public closure frontend faces).
use ace_core::atom_text::Diagnostic;
use ace_core::body_work::capture_stage_attempts;
use ace_core::source::cst::{
    self, parse_front, units::*, FrontBudget, FrontKind, FrontOutcome, FrontRequest,
    FRONT_CAPS_VERSION,
};
use ace_core::source::{parse_program_with_handwritten, SourceProfile};

const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;
const PROFILE: SourceProfile = SourceProfile::Closures;
/// 第76轮探针的合法当前源：两个 lambda，g 调用 f。
const SRC: &str = "fn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n + 1 }\n    let g = (m int) => { return f(m) + 1 }\n    return g(2)\n}\nfn helper() int { return 7 }\n";
/// 坏的第二个 lambda：`+` 后缺表达式，首错必须落在体内 `}`，不是 f 的注解。
const BAD_G: &str = "(m int) => { return f(m) + }";
const PREFIXES: [&str; 3] = [
    "",
    "// prefix drift: 中文\n\n",
    "fn before() int { return 9 }\n",
];

fn handwritten(src: &str) -> Diagnostic {
    parse_program_with_handwritten(src, PROFILE).expect_err("handwritten oracle must reject")
}

fn fresh_first(src: &str) -> Diagnostic {
    match parse_front(FrontRequest {
        src,
        profile: PROFILE,
        kind: FrontKind::WholeModule,
        budget: BUDGET,
    }) {
        FrontOutcome::Diag(ds) => ds
            .into_iter()
            .next()
            .expect("front door reports a diagnostic"),
        other => panic!("expected whole-source syntax error, got {other:?}"),
    }
}

fn four_tuple(d: &Diagnostic) -> (String, String, String, usize, usize) {
    (
        d.stage.as_str().to_string(),
        d.code.clone(),
        d.message.clone(),
        d.span.start,
        d.span.end,
    )
}

fn same(actual: &Diagnostic, oracle: &Diagnostic, what: &str) {
    assert_eq!(
        four_tuple(actual),
        four_tuple(oracle),
        "{what}: first diagnostic must equal the handwritten oracle byte for byte"
    );
}

/// 整源差分：两 parser 要么同 AST，要么同首诊断四元组。
fn assert_same_whole(src: &str, what: &str) {
    let old = parse_program_with_handwritten(src, PROFILE);
    let new = cst::parse_program_via_cst(src, PROFILE);
    match (&old, &new) {
        (Ok(a), Ok(b)) => assert_eq!(format!("{a:?}"), format!("{b:?}"), "{what}: AST mismatch"),
        (Err(d), Err(e)) => same(e, d, what),
        _ => panic!("{what}: acceptance mismatch\n--old--\n{old:#?}\n--new--\n{new:#?}"),
    }
}

fn restore(cat: UnitCatalog) -> UnitCatalog {
    let src = cat.src().to_string();
    let manifest = decode_manifest(&encode_manifest(&cat.to_manifest(FRONT_CAPS_VERSION))).unwrap();
    drop(cat);
    let (restored, work) = capture_stage_attempts(|| {
        UnitCatalog::from_restored(&src, &manifest, PROFILE, FRONT_CAPS_VERSION).unwrap()
    });
    assert_eq!(
        work.whole_total(),
        0,
        "restore must not produce a whole module"
    );
    assert!(restored.lambdas.values().all(|l| l.body.is_none()));
    restored
}

/// 片段面：目录 lambda 编辑失败的首诊断（失败保旧、不整源重解析）。
fn fragment_first(cat: &mut UnitCatalog, ordinal: u32, bad: &str) -> Diagnostic {
    let before = cat.src().to_string();
    let discarded = cat.meter.discarded;
    let (result, work) = capture_stage_attempts(|| cat.edit_lambda("main", ordinal, bad, BUDGET));
    let EditError::Diag(ds) = result.expect_err("bad lambda edit must fail") else {
        panic!("source error must remain typed Diag")
    };
    assert_eq!(
        work.whole_total(),
        0,
        "fragment diagnostic must not reparse whole source"
    );
    assert_eq!(cat.src(), before, "failed edit keeps the old source");
    assert_eq!(cat.meter.discarded, discarded + 1);
    ds.into_iter()
        .next()
        .expect("fragment reports a diagnostic")
}

fn with_second_lambda(src: &str, replacement: &str) -> String {
    let cat = UnitCatalog::build(src, PROFILE, BUDGET).expect("valid catalog");
    let span = cat.lambdas[&("main".to_string(), 1)].span;
    format!("{}{}{}", &src[..span.start], replacement, &src[span.end..])
}

#[test]
fn bad_second_lambda_first_diag_equals_handwritten_on_all_faces() {
    for prefix in PREFIXES {
        for restored in [false, true] {
            let src = format!("{prefix}{SRC}");
            let mut cat = UnitCatalog::build(&src, PROFILE, BUDGET).expect("valid catalog");
            if restored {
                cat = restore(cat);
            }
            let span = cat.lambdas[&("main".to_string(), 1)].span;
            let attempted = format!("{}{}{}", &src[..span.start], BAD_G, &src[span.end..]);
            let oracle = handwritten(&attempted);
            // oracle 形状固定（不从 CST 复制）：g 体内 `+` 后的 `}`。
            let close = span.start + attempted[span.start..].find("+ }").unwrap() + 2;
            assert_eq!(
                four_tuple(&oracle),
                (
                    "text".to_string(),
                    "source.syntax".to_string(),
                    "expected expression, found `}`".to_string(),
                    close,
                    close + 1
                )
            );
            let what = format!("prefix={prefix:?} restored={restored}");
            same(
                &fresh_first(&attempted),
                &oracle,
                &format!("whole source ({what})"),
            );
            same(
                &fragment_first(&mut cat, 1, BAD_G),
                &oracle,
                &format!("lambda fragment ({what})"),
            );
        }
    }
}

#[test]
fn legal_lambda_shapes_parse_identically_on_both_parsers() {
    for (what, second) in [
        ("original", "(m int) => { return f(m) + 1 }"),
        ("expression body", "(m int) => f(m) + 1"),
        ("empty params", "() => f(1)"),
        ("multi params", "(m int, k int) => f(m) + k"),
        (
            "fn-type and word params",
            "(k fn(int) int, m int, s Str, c char) => k(m)",
        ),
        (
            "nested lambda",
            "(m int) => { let k = (b int) => f(m) + b\n        return k(1) }",
        ),
        ("move prefix", "move (m int) => f(m)"),
        (
            "lambda as call argument",
            "(m int) => f(((k int) => k + m)(1))",
        ),
        ("parenthesized value", "(f(1) + 2)"),
    ] {
        for prefix in PREFIXES {
            let src = with_second_lambda(&format!("{prefix}{SRC}"), second);
            assert!(
                parse_program_with_handwritten(&src, PROFILE).is_ok(),
                "{what}: handwritten oracle must accept"
            );
            assert_same_whole(&src, what);
        }
    }
}

#[test]
fn bad_lambda_head_shapes_after_legal_lambda_match_handwritten() {
    for (what, second, code, message) in [
        (
            "literal parameter",
            "(1) => 1",
            "source.syntax",
            "expected lambda parameter, found integer literal",
        ),
        (
            "missing annotation",
            "(m) => m",
            "source.syntax",
            "expected a type, found `)`",
        ),
        (
            "trailing comma",
            "(m int,) => m",
            "source.syntax",
            "expected lambda parameter, found `)`",
        ),
        (
            "bad separator",
            "(m int k int) => m",
            "source.syntax",
            "expected `,` or `)` in lambda parameters",
        ),
        // 无 `=>` 的 `( 名 注解 )` 不是 lambda 头（前瞻要求匹配 `)` 后随 `=>`）：
        // 两 parser 都按括号表达式在 `int` 报错。
        (
            "arrow-less parens stay a paren expression",
            "(m int) m",
            "source.syntax",
            "expected `)`",
        ),
        (
            "lowercase str annotation",
            "(m str) => 1",
            "source.unsupported",
            "type `str` is not part of this source profile",
        ),
        (
            "keyword parameter",
            "(fn int) => 1",
            "source.unsupported",
            "keyword `fn` cannot be used as a name",
        ),
        (
            "keyword body",
            "(m int) => return",
            "source.syntax",
            "expected expression, found `return`",
        ),
        // 不含“未闭合 lambda 块体吞掉 main 的 `}`”形状：完整块 lambda 之后 main
        // 未闭合时 CST 恢复树把其后语句落为顶层 script，投影的 entry-* 能力拒绝
        // 位置更早而胜出——这是恢复/投影面（PLAN-941 F-941-12，另行具名），不是
        // 期望游走器的 lambda 头识别；无 lambda 的同形恢复两 parser 一致。
    ] {
        for prefix in PREFIXES {
            let src = with_second_lambda(&format!("{prefix}{SRC}"), second);
            let oracle = handwritten(&src);
            assert_eq!(
                (oracle.code.as_str(), oracle.message.as_str()),
                (code, message),
                "{what}"
            );
            assert_same_whole(&src, what);
        }
    }
}

#[test]
fn nested_move_and_argument_lambda_bad_bodies_match_handwritten() {
    for (what, second) in [
        (
            "nested inner body",
            "(m int) => { let k = (b int) => { return m + b + }\n        return k(1) }",
        ),
        ("move prefix body", "move (m int) => { return f(m) + }"),
        (
            "call-argument lambda body",
            "(m int) => f(((k int) => { return k + })(m))",
        ),
        ("expression body", "(m int) => f(m) +"),
        (
            "lexical error after legal lambda",
            "(m int) => { return f(m) @ 1 }",
        ),
    ] {
        for prefix in PREFIXES {
            let src = with_second_lambda(&format!("{prefix}{SRC}"), second);
            handwritten(&src);
            assert_same_whole(&src, what);
        }
    }
}

#[test]
fn async_and_plain_paren_controls_match_handwritten() {
    for prefix in PREFIXES {
        let src = with_second_lambda(&format!("{prefix}{SRC}"), "async (m int) => f(m)");
        let oracle = handwritten(&src);
        assert_eq!(oracle.code, "source.closure-unsupported");
        assert_same_whole(&src, "async lambda keeps its dedicated rejection");
        let src = with_second_lambda(&format!("{prefix}{SRC}"), "(f(1) + )");
        let oracle = handwritten(&src);
        assert_eq!(oracle.message, "expected expression, found `)`");
        assert_same_whole(&src, "plain parenthesized expression error");
        let src = with_second_lambda(&format!("{prefix}{SRC}"), "(f(1) + 2) + }");
        let oracle = handwritten(&src);
        assert_eq!(oracle.message, "expected expression, found `}`");
        assert_same_whole(&src, "paren value then dangling operator");
    }
}

#[test]
fn public_closures_frontend_first_diag_equals_handwritten() {
    use ace_core::source::closures_fn::ClosuresFunctionFrontend;
    // 公开前端做完整 bind：第二个 lambda 只捕获 int 局部（闭包捕获不在其合同内）。
    const PUB_SRC: &str = "fn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n + 1 }\n    let g = (m int) => { return base + m + 1 }\n    return f(1) + g(2)\n}\nfn helper() int { return 7 }\n";
    const PUB_BAD_G: &str = "(m int) => { return base + m + }";
    let src = format!("// 中文 prefix\n{PUB_SRC}");
    let (mut frontend, _parts, _checked) =
        ClosuresFunctionFrontend::cold(&src, "f11.at", &[]).unwrap();
    let lambda = frontend
        .units()
        .iter()
        .find(|u| u.name == "__closure_main_1")
        .unwrap()
        .span;
    let sha = frontend.source_sha().to_string();
    let attempted = format!(
        "{}{}{}",
        &src[..lambda.start],
        PUB_BAD_G,
        &src[lambda.end..]
    );
    let Err(ds) = frontend.edit_body(&sha, lambda.start..lambda.end, PUB_BAD_G) else {
        panic!("invalid public frontend edit must return diagnostics")
    };
    same(&ds[0], &handwritten(&attempted), "public closures frontend");
    assert_eq!(frontend.source(), src);
    assert_eq!(frontend.source_sha(), sha);
}
