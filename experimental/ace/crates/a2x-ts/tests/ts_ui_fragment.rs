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
fn checked_widget_view_and_child_property_follow_the_current_model() {
    use ace_a2x_ts::seed::{decode_coll_seed, encode_coll_seed, render_coll_seed_helpers};
    use ace_a2x_ts::{emit_function_body_fragment, script_widget_runtime_body, ts_fn_name};
    use ace_core::source::age_h0::LegacyWidgetAlias;
    use ace_core::source::age_package::{compile_age_package_widget_runtime, AgeWidgetProperty};
    use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

    let source = r#"
use support.caption: BASE

style pane_tone = "text-primary"
const START int = BASE + 2
const MARKER Str = "界" + "面"

widget Parent {
    use { package: support from "../components" }
    msg { Add(int), Rename(str) }
    model {
        count int = START
        var label str = "initial"
        var previous_label str = ""
        ids List = []
        rows List = []
    }
    on {
        .Add(step) -> {
            .count = .count + step
        }
        .Rename(value) -> {
            var rows List = []
            rows.push({ title: f"${value}" })
            .rows = rows
            var selected = .rows[0]
            var title = selected.title
            .previous_label = .label
            .label = title
        }
    }
    view {
        text f"${MARKER} Count ${.count}" { style: pane_tone }
        CaptionWidget (caption: .label) {}
    }
}
"#;
    let child_source = r#"
pub const BASE int = 3

type CaptionText { text str }

fn render_caption(value Str) Str {
    let box = CaptionText { text: value.to(str).to(Str) }
    let local = box.text
    return "Caption " + local
}

widget CaptionWidget (caption: str) {
    msg {}
    model {}
    view {
        // Wide view
        text render_caption(.caption) {}
    }
}
"#.replace("// Wide view", &(0..64)
        .map(|index| format!("text \"prefix {index}\" {{}}\n"))
        .collect::<String>());
    let entry = "app::main";
    let child_module = "support::caption";
    let compiled = compile_age_package_widget_runtime(
        &[
            PackageModuleInput {
                package: "app".into(),
                version: "1.0.0".into(),
                id: "main".into(),
                source: source.into(),
                provider: None,
            },
            PackageModuleInput {
                package: "support".into(),
                version: "1.0.0".into(),
                id: "caption".into(),
                source: child_source.into(),
                provider: None,
            },
        ],
        entry,
        LegacyWidgetAlias::Accept,
        &[PACKAGES_CAPABILITY.into()],
    )
    .expect("checked widget closure");
    let [parent, child] = compiled.widget_runtime.as_slice() else {
        panic!(
            "expected two widgets, got {:?} with bindings {:?}",
            compiled.widget_runtime, compiled.widget_refs
        );
    };
    let symbol = |name: &str| ts_fn_name(&compiled.symbol_of_in(entry, name).unwrap());
    let prop = &compiled.first_widget_ref().unwrap().properties[0];
    let AgeWidgetProperty::Value {
        runtime_evaluator: Some(caption),
        ..
    } = prop
    else {
        panic!("expected an evaluator for the controlled child property");
    };
    let view = ace_a2x_core::SemanticView::from_checked_age_package(&compiled.checked).unwrap();
    let context = FragmentProgramContext::from_view(&view);
    let seed = decode_coll_seed(&encode_coll_seed(&view).unwrap()).unwrap();
    let mut driver = String::from("import assert from 'node:assert/strict';\nfunction fixture() {\n");
    driver.push_str(&script_widget_runtime_body());
    for line in render_coll_seed_helpers(&seed, false).lines() {
        driver.push_str(line.strip_prefix("export ").unwrap_or(line));
        driver.push('\n');
    }
    for function in view.functions() {
        let fragment =
            emit_function_body_fragment(ScriptLang::JavaScript, &view, function, &context).unwrap();
        let args = fragment
            .locals
            .iter()
            .filter(|item| item.role == "param")
            .map(|item| item.emitted_name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        driver.push_str(&format!(
            "function {}({args}) {{\n{}\n}}\n",
            ts_fn_name(&function.source_name),
            fragment.text.join("\n")
        ));
    }
    for initializer in view.global_init_fns() {
        driver.push_str(&format!("{}();\n", ts_fn_name(initializer)));
    }
    driver.push_str(&format!(
        r#"
a2x_widget_keep_models([]);
const seed = {init};
const initial = seed();
const independent = seed();
const added = {add}(initial, 7);
const renamed = {rename}(added, a2x_str_to_owned(a2x_str_lit('updated')));
const cells = result => result.items.map(a2x_str_utf8);
assert.ok(cells({parent_view}(initial)).includes('界面 Count 5'));
assert.ok(cells({parent_view}(renamed)).includes('界面 Count 12'));
assert.ok(cells({parent_view}(independent)).includes('界面 Count 5'));
assert.ok(cells({parent_view}(initial)).includes('text-primary'));
assert.equal(a2x_str_utf8(renamed.previous_label), 'initial');
const renamedChild = cells({child_view}({caption}(renamed)));
const prefix = Array.from({{ length: 64 }}, (_, index) => ['text', 'prefix ' + index]).flat();
assert.deepEqual(renamedChild, [...prefix, 'text', 'Caption updated']);
assert.ok(cells({child_view}({caption}(added))).includes('Caption initial'));
}}
fixture();
fixture();
console.log('WIDGET_OK');
"#,
        init = symbol(parent.initialize.as_ref().unwrap()),
        add = symbol(&parent.handlers[0].function),
        rename = symbol(&parent.handlers[1].function),
        parent_view = symbol(&parent.view),
        child_view = ts_fn_name(&compiled.symbol_of_in(child_module, &child.view).unwrap()),
        caption = symbol(caption),
    ));
    let dir = scratch_dir("tsfrag_widget_current_model");
    let (code, stdout, stderr) = run_driver(&dir, ScriptLang::JavaScript, &driver);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!((code, stdout.as_str()), (0, "WIDGET_OK\n"), "{stderr}");
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
