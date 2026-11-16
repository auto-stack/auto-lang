//! PLAN-976 T-03: the UI-neutral fragment interface
//! (`emit_stmt_fragment` / `emit_expr_fragment`) verified by REAL node
//! execution — the frozen interface PLAN-980 will splice at its four legacy
//! delegation sites, exercised here with an independent host-side oracle
//! (no UI product dependency, per plan §5 I.2).
//!
//! The host driver declares the fragment's locals with the interface's
//! emitted names, splices the emitted text, imports ONLY the reported
//! helpers, and checks the observed values against hand-computed Auto
//! semantics.

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_ts::{
    emit_expr_fragment, emit_stmt_fragment, innermost_expr_in_span, script_runtime_body,
    FragmentProgramContext, ScriptLang,
};

fn find_node() -> String {
    if let Ok(p) = std::env::var("A2TS_NODE") {
        if !p.trim().is_empty() {
            return p;
        }
    }
    for cand in ["node", "node.exe"] {
        if let Ok(out) = Command::new(cand).arg("--version").output() {
            if out.status.success() {
                return cand.to_string();
            }
        }
    }
    panic!("INFRA ERROR: node not found on PATH and A2TS_NODE not set");
}

fn scratch_dir(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2ts-unit")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn view_of(src: &str) -> ace_a2x_core::SemanticView<'static> {
    let compiled =
        ace_core::source::compile_source(src, "a2ts-frag", &[]).expect("frontend diagnostics");
    let checked = Box::leak(Box::new(compiled.checked));
    ace_a2x_core::SemanticView::from_checked(checked).expect("view admission")
}

fn facts_of<'v>(
    view: &'v ace_a2x_core::SemanticView<'static>,
    name: &str,
) -> &'v ace_a2x_core::FnFacts<'static> {
    view.functions()
        .iter()
        .find(|f| f.source_name == name)
        .expect("function present in view")
}

/// Run a host driver that splices the fragment; assert the observed line.
fn run_driver(dir: &PathBuf, lang: ScriptLang, driver: &str) -> (i32, String, String) {
    std::fs::write(dir.join("a2x_rt.mjs"), script_runtime_body(lang, &[])).unwrap();
    let file = if lang.typed() { "host.ts" } else { "host.mjs" };
    std::fs::write(dir.join(file), driver).unwrap();
    let node = find_node();
    let out = Command::new(&node)
        .arg(file)
        .current_dir(dir)
        .output()
        .expect("spawn node");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn tsfrag_01_expr_fragment_runs_on_both_legs() {
    // The expression `count + 1` inside `handler`, plus a checked-mul host
    // expression `total * 7` — the classic UI binding shapes.
    let src = "fn handler(count int, total int) int {\n    return count + 1\n}\nfn other(total int) int {\n    return total * 7\n}";
    let view = view_of(src);
    let facts = facts_of(&view, "handler");
    // Locate `count + 1` by its source span (the host locator).
    let plus = src.find("count + 1").unwrap();
    let id = innermost_expr_in_span(&facts, plus..plus + 9).expect("expr found by span");

    let ctx = FragmentProgramContext::from_view(&view);
    for lang in [ScriptLang::TypeScript, ScriptLang::JavaScript] {
        let out = emit_expr_fragment(lang, &view, &facts, id, &ctx).expect("expr fragment");
        let expr = out.expr.as_ref().unwrap();
        assert!(
            out.helpers.iter().any(|h| h == "a2x_add"),
            "helpers must name a2x_add: {:?}",
            out.helpers
        );
        // The checked-add helper rides a helper line (`const t0 = a2x_add(..)`)
        // and the expr is the temp reference — both must be spliced.
        assert!(
            out.text.iter().any(|l| l.contains("a2x_add")),
            "helper line expected: {:?}",
            out.text
        );

        // Host driver: params bound by emitted name, only reported helpers imported.
        let imports = if out.helpers.is_empty() {
            String::new()
        } else {
            format!(
                "import {{ {} }} from \"./a2x_rt.mjs\";\n",
                out.helpers.join(", ")
            )
        };
        let p_count = out
            .locals
            .iter()
            .find(|l| l.source_name == "count")
            .unwrap()
            .emitted_name
            .clone();
        let driver = format!(
            "{imports}const {p_count} = 41;\n{}\nconst v = {expr};\nconsole.log(v === 42 ? \"FRAG_OK\" : `FRAG_FAIL ${{v}}`);\n",
            out.text.join("\n")
        );
        let dir = scratch_dir(&format!("tsfrag01_{:?}", lang));
        let (code, stdout, stderr) = run_driver(&dir, lang, &driver);
        assert_eq!(
            (code, stdout.as_str()),
            (0, "FRAG_OK\n"),
            "{lang:?}: stderr={stderr}\ndriver:\n{driver}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn tsfrag_02_stmt_fragment_runs_and_binds_locals() {
    // `sum = sum + item` — an assignment statement fragment on a MUTABLE
    // binding (params are immutable in Auto).
    let src = "fn step(item int) int {\n    var sum = 40\n    sum = sum + item\n    return sum\n}";
    let view = view_of(src);
    let facts = facts_of(&view, "step");
    // stmt 1 of the entry block is the assignment (stmt 0 is the var decl).
    let entry = facts.body.entry.0;
    let ctx = FragmentProgramContext::from_view(&view);
    for lang in [ScriptLang::TypeScript, ScriptLang::JavaScript] {
        let out = emit_stmt_fragment(lang, &view, &facts, entry, 1, &ctx).expect("stmt fragment");
        assert!(!out.text.is_empty(), "assignment must emit a line");
        let sum_name = out
            .locals
            .iter()
            .find(|l| l.source_name == "sum")
            .unwrap()
            .emitted_name
            .clone();
        let item_name = out
            .locals
            .iter()
            .find(|l| l.source_name == "item")
            .unwrap()
            .emitted_name
            .clone();
        let imports = if out.helpers.is_empty() {
            String::new()
        } else {
            format!(
                "import {{ {} }} from \"./a2x_rt.mjs\";\n",
                out.helpers.join(", ")
            )
        };
        let driver = format!(
            "{imports}let {sum_name} = 40;\nconst {item_name} = 2;\n{}\nconsole.log({sum_name} === 42 ? \"FRAG_OK\" : `FRAG_FAIL ${{{sum_name}}}`);\n",
            out.text.join("\n")
        );
        let dir = scratch_dir(&format!("tsfrag02_{:?}", lang));
        let (code, stdout, stderr) = run_driver(&dir, lang, &driver);
        assert_eq!(
            (code, stdout.as_str()),
            (0, "FRAG_OK\n"),
            "{lang:?}: stderr={stderr}\ndriver:\n{driver}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn tsfrag_03_context_mismatch_is_a_named_rejection() {
    // A record-typed local whose struct is NOT in the host context must be
    // rejected by name, never spliced silently.
    let src = "type Point { x int; y int }\nfn area(p Point) int {\n    return p.x * p.y\n}";
    let compiled =
        ace_core::source::compile_text_record_source(src, "a2ts-frag", &[]).expect("diagnostics");
    let checked = Box::leak(Box::new(compiled.checked));
    let view = ace_a2x_core::SemanticView::from_checked(checked).expect("view");
    let facts = facts_of(&view, "area");

    let empty_ctx = FragmentProgramContext::default();
    let err = emit_stmt_fragment(
        ScriptLang::TypeScript,
        &view,
        &facts,
        facts.body.entry.0,
        0,
        &empty_ctx,
    )
    .expect_err("empty context must be rejected");
    assert_eq!(err.code, "a2ts.fragment-context", "{}", err.detail);
    assert!(err.detail.contains("Point"), "{}", err.detail);

    // With the struct declared the same fragment emits.
    let full_ctx = FragmentProgramContext::from_view(&view);
    assert!(full_ctx.structs.iter().any(|s| s == "Point"));
    let ok = emit_stmt_fragment(
        ScriptLang::TypeScript,
        &view,
        &facts,
        facts.body.entry.0,
        0,
        &full_ctx,
    );
    assert!(ok.is_ok(), "{:?}", ok.err());
}

#[test]
fn tsfrag_04_out_of_range_locator_rejected() {
    let src = "fn main() int { return 1 + 2 }";
    let view = view_of(src);
    let facts = facts_of(&view, "main");
    let ctx = FragmentProgramContext::from_view(&view);
    let err = emit_stmt_fragment(ScriptLang::TypeScript, &view, &facts, 99, 0, &ctx)
        .expect_err("bad block must be rejected");
    assert_eq!(err.code, "a2ts.fragment");
    let err = emit_stmt_fragment(ScriptLang::TypeScript, &view, &facts, 0, 99, &ctx)
        .expect_err("bad stmt index must be rejected");
    assert_eq!(err.code, "a2ts.fragment");
    let err = emit_expr_fragment(
        ScriptLang::TypeScript,
        &view,
        &facts,
        ace_core::hir::ExprId(9999),
        &ctx,
    )
    .expect_err("bad expr id must be rejected");
    assert_eq!(err.code, "a2ts.fragment");
}
