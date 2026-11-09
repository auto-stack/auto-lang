//! F-941-12: a recovery-artifact script region (the CST engine closes an
//! unterminated outer body at the last complete nested block `}` and
//! re-reads the trailing statements as a top-level script) must not let
//! entry-* capability rejections preempt the structural first diagnosis;
//! the first diagnostic equals the handwritten oracle byte for byte.
use ace_core::source::cst::{
    self, parse_front, FrontBudget, FrontKind, FrontOutcome, FrontRequest,
};
use ace_core::source::{parse_program_with_handwritten, SourceProfile};

const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;
const PREFIXES: [&str; 3] = [
    "",
    "// prefix drift: 中文\n\n",
    "fn before() int { return 9 }\n",
];

/// A：完整 lambda 块 + 尾部语句 + `fn`，main 未闭合（尾部杂 `}`）。
const SHAPE_LAMBDA_FN: &str = "fn main() int {\n    let g = (m int) => { return m }\n    return g(2)\nfn helper() int { return 7 }\n}\n";
/// H：完整 if 块 + 尾部语句 + `fn`（非 lambda 触发器）。
const SHAPE_IF_FN: &str = "fn main() int {\n    let x int = 1\n    if x > 0 { return x }\n    return 2\nfn helper() int { return 7 }\n}\n";
/// I：完整 lambda 块 + 尾部语句 + EOF，main 未闭合。
const SHAPE_EOF: &str = "fn main() int {\n    let g = (m int) => { return m }\n    return g(2)\n";
/// 产物 run：完整块后多段尾部语句。
const SHAPE_RUN: &str = "fn main() int {\n    let g = (m int) => { return m }\n    let y int = 3\n    return g(y)\nfn helper() int { return 7 }\n";

fn handwritten(src: &str, profile: SourceProfile) -> ace_core::atom_text::Diagnostic {
    parse_program_with_handwritten(src, profile).expect_err("handwritten oracle must reject")
}

fn cst_first(src: &str, profile: SourceProfile) -> ace_core::atom_text::Diagnostic {
    match parse_front(FrontRequest {
        src,
        profile,
        kind: FrontKind::WholeModule,
        budget: BUDGET,
    }) {
        FrontOutcome::Diag(ds) => ds
            .into_iter()
            .next()
            .expect("front door reports a diagnostic"),
        other => panic!("expected syntax error, got {other:?}"),
    }
}

fn four_tuple(d: &ace_core::atom_text::Diagnostic) -> (String, String, String, usize, usize) {
    (
        d.stage.as_str().to_string(),
        d.code.clone(),
        d.message.clone(),
        d.span.start,
        d.span.end,
    )
}

fn same_as_handwritten(src: &str, profile: SourceProfile, what: &str) {
    let oracle = handwritten(src, profile);
    let actual = cst_first(src, profile);
    assert_eq!(
        four_tuple(&actual),
        four_tuple(&oracle),
        "{what}: first diagnostic must equal the handwritten oracle byte for byte"
    );
}

/// 结构层期望：分歧点在尾部 `fn`（或 EOF），不是更早的 entry-*。
fn assert_fn_divergence(src: &str, profile: SourceProfile, what: &str) {
    let oracle = handwritten(src, profile);
    let tail = src.rfind("fn helper").expect("shape carries a trailing fn");
    assert_eq!(
        four_tuple(&oracle),
        (
            "text".to_string(),
            "source.syntax".to_string(),
            "expected statement, found `fn`".to_string(),
            tail,
            tail + 2
        ),
        "{what}: oracle shape pinned independently of the CST"
    );
    same_as_handwritten(src, profile, what);
}

#[test]
fn artifact_script_regions_match_handwritten_first_diagnosis() {
    for prefix in PREFIXES {
        for (what, shape) in [
            (
                "A complete lambda block then unterminated main + trailing fn",
                SHAPE_LAMBDA_FN,
            ),
            (
                "H complete if block then unterminated main + trailing fn",
                SHAPE_IF_FN,
            ),
            ("artifact run with two trailing statements", SHAPE_RUN),
        ] {
            let src = format!("{prefix}{shape}");
            assert_fn_divergence(&src, SourceProfile::Closures, what);
        }
        // H/I 触发器与 profile 无关：entry-unit 面（script 形主场）同样必须以
        // main 体内分歧点为首诊断。
        assert_fn_divergence(
            &format!("{prefix}{SHAPE_IF_FN}"),
            SourceProfile::EntryUnit,
            "H on the entry-unit surface",
        );
        let src = format!("{prefix}{SHAPE_EOF}");
        let oracle = handwritten(&src, SourceProfile::Closures);
        assert_eq!(
            four_tuple(&oracle),
            (
                "text".to_string(),
                "source.syntax".to_string(),
                "expected statement or `}`, found end of file".to_string(),
                src.len(),
                src.len()
            )
        );
        same_as_handwritten(&src, SourceProfile::Closures, "I unterminated main at EOF");
    }
}

#[test]
fn genuine_script_forms_and_adjacent_shapes_stay_equal() {
    for prefix in PREFIXES {
        for profile in [SourceProfile::Closures, SourceProfile::EntryUnit] {
            for (what, src) in [
                ("clean top-level return keeps entry-top-return", "return 1\n"),
                ("clean top-level let parses on both", "let x int = 1\n"),
                (
                    "clean script then fn keeps entry-order",
                    "let x int = 1\nfn f() int { return 1 }\n",
                ),
                (
                    "R04 head type keeps its positional rejection",
                    "type T int = int\nfn f() int { return 1 }\n",
                ),
                (
                    "D no nested block: both parsers already agree",
                    "fn main() int {\n    let g int = 2\n    return g\nfn helper() int { return 7 }\n}\n",
                ),
                (
                    "E unterminated block swallowing main brace: already agree",
                    "fn main() int {\n    let x int = 1\n    if x > 0 { return x\n    return 2\n}\nfn helper() int { return 7 }\n",
                ),
                (
                    "genuine script after an erroneous fn item",
                    "fn a() int { return }\nlet x int = 1\n",
                ),
            ] {
                let src = format!("{prefix}{src}");
                let old = parse_program_with_handwritten(&src, profile);
                let new = cst::parse_program_via_cst(&src, profile);
                match (&old, &new) {
                    (Ok(a), Ok(b)) => assert_eq!(format!("{a:?}"), format!("{b:?}"), "{what}"),
                    (Err(d), Err(e)) => assert_eq!(
                        four_tuple(e),
                        four_tuple(d),
                        "{what} ({profile:?}): must stay byte-equal"
                    ),
                    _ => panic!(
                        "{what} ({profile:?}): acceptance mismatch\n--old--\n{old:#?}\n--new--\n{new:#?}"
                    ),
                }
            }
        }
    }
}

#[test]
fn fragment_face_artifact_shape_matches_handwritten() {
    use ace_core::source::closures_fn::ClosuresFunctionFrontend;
    // 公开片段面：g 的坏编辑把完整 if 块 + 尾部语句 + 体内 `fn` 放进单元内容
    // （分歧点在内容内部，无边界EOF歧义），同一投影入口必须同样以结构层分歧
    // 点为首诊断并与整源手写 oracle 逐字节全等。
    const SRC: &str = "fn main() int {\n    let g = (m int) => { if m > 0 { return m }\n        return m }\n    return g(2)\n}\n";
    const BAD: &str =
        "(m int) => { if m > 0 { return m }\n    return m\nfn helper() int { return 7 }";
    let src = format!("// 中文 prefix\n{SRC}");
    let (mut frontend, _parts, _checked) =
        ClosuresFunctionFrontend::cold(&src, "f12.at", &[]).unwrap();
    let lambda = frontend
        .units()
        .iter()
        .find(|u| u.name == "__closure_main_0")
        .unwrap()
        .span;
    let sha = frontend.source_sha().to_string();
    let attempted = format!("{}{}{}", &src[..lambda.start], BAD, &src[lambda.end..]);
    let oracle = handwritten(&attempted, SourceProfile::Closures);
    assert_eq!(oracle.message, "expected statement, found `fn`");
    let Err(ds) = frontend.edit_body(&sha, lambda.start..lambda.end, BAD) else {
        panic!("invalid public frontend edit must return diagnostics")
    };
    assert_eq!(
        four_tuple(&ds[0]),
        four_tuple(&oracle),
        "fragment face: first diagnostic must equal the handwritten oracle"
    );
    assert_eq!(frontend.source(), src);
    assert_eq!(frontend.source_sha(), sha);
}
