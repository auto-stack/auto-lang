//! PLAN-1072 刀A（组件面 style）：`style NAME = "literal"` 作者面在
//! age-component 的开口永久用例。
//!
//! 语义（沿 981 r18 会签 (c)3 投影期字面量替换——**非** const 路径）：
//! 组件面 style = 惰性视图数据（不产 Item::Const/Fn、不进 resolve/
//! ModuleGlobalsPlan——981 三锁条件不触发）；view 内 `style: NAME` ident
//! 引用在投影期替换为字面量；重名/跨模块引用/带参调用沿包面同名拒。
//! 参数化/`pub style` 仍是包面专属（组件面文法层边界——style-params
//! 步骤 profile 门控缺席）。
//!
//! 红相（刀前）：`style` 声明使组件面整源解析失败（style-item 仅
//! age-package）——005-login 真源 lane 全体缺席的实底。
use ace_core::source::age_component::{
    compile_component_scenario_states, ComponentInitials, ComponentScenario,
};
use ace_core::source::age_h0::LegacyWidgetAlias;

#[test]
fn p107_chart_package_import_binds_a_lowercase_widget_alias() {
    use ace_core::source::{
        age_component::{ComponentEntryValue, ComponentViewItem},
        age_package::compile_age_package,
        ast::ExprKind,
        cst::FrontBudget,
        packages::{parse_package_source_age, PackageModuleInput, PACKAGES_CAPABILITY},
    };

    let source = r#"widget Charts {
    use { package: official from "components" }
    msg {}
    model { monthly = [{ m: "Jan", desktop: 186 }] }
    view {
        line-chart (fields: ["desktop", "mobile"]) {}
    }
}
"#;
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let scenario = ComponentScenario { instances: 0, ops: Vec::new() };
    let parsed = parse_package_source_age(source, &caps, FrontBudget::WORK_OWNED_64MIB, &scenario, 0)
        .expect("source package declaration and chart collection projection");
    let node = &parsed.age_components[0].0;
    assert_eq!(node.package_imports[0].package.name, "official");
    assert_eq!(node.package_imports[0].directory, "components");
    assert_eq!(node.view_data[0].name, "monthly");
    let [ComponentViewItem::Element(chart)] = node.view.tree.as_slice() else {
        panic!("expected the authored chart");
    };
    let [ComponentViewItem::Entry(fields)] = chart.items.as_slice() else {
        panic!("expected the authored fields property");
    };
    let ComponentEntryValue::Expr(expr) = &fields.value else {
        panic!("expected a list expression");
    };
    let ExprKind::ArrayLit(values) = &expr.kind else {
        panic!("expected the projected fields list");
    };
    assert!(matches!(&values[0].kind, ExprKind::StrLit(value) if value == "desktop"));
    assert!(matches!(&values[1].kind, ExprKind::StrLit(value) if value == "mobile"));

    let unit = |package: &str, id: &str, source: &str| PackageModuleInput {
        package: package.into(), id: id.into(), version: "1.0.0".into(),
        source: source.into(), provider: None,
    };
    let root = unit("app", "main", source);
    let implementation = "widget LineChart (fields: List<str>) { view { text \"curve\" {} } }";
    let chart = unit("official", "line_chart", implementation);
    let compile = |units: &[PackageModuleInput]| {
        compile_age_package(units, "app::main", LegacyWidgetAlias::Accept, &caps)
    };
    let bound = compile(&[root.clone(), chart.clone(), unit("other", "line_chart", implementation)])
        .expect("only the declared component package participates in alias lookup");
    let reference = bound.first_widget_ref().expect("resolved chart site");
    assert_eq!(reference.module, "official::line_chart");
    assert_eq!(reference.node, "LineChart");
    assert_eq!(reference.span.slice(source), r#"line-chart (fields: ["desktop", "mobile"]) {}"#);
    let missing = compile(&[root.clone()]).err().expect("the declared package is required");
    assert_eq!(missing[0].code, "source.age.component-package-unresolved");
    let manifest = PackageModuleInput::component_manifest("official".into(), "1.0.0".into());
    let lowercase = compile(&[root.clone(), manifest.clone()])
        .expect("a loaded package with no required modules can use a host tag");
    assert!(lowercase.first_widget_ref().is_none());
    let named_widget = unit("app", "main", &source.replace("line-chart", "LineChart"));
    let unresolved = compile(&[named_widget, manifest]).err()
        .expect("the manifest supplies package identity but no widget declarations");
    assert_eq!(unresolved[0].code, "source.age.widget-unresolved");
    let ambiguous = compile(&[root, chart, unit("official", "second_chart", implementation)])
        .err().expect("alias collisions are rejected");
    assert_eq!(ambiguous[0].code, "source.age.widget-ambiguous");
}

fn compile(src: &str) -> Result<(), Vec<ace_core::atom_text::Diagnostic>> {
    let caps = vec!["ac.runtime.text.v1".to_string()];
    compile_component_scenario_states(
        src,
        "p1072-style.at",
        LegacyWidgetAlias::Accept,
        &caps,
        &ComponentScenario {
            instances: 1,
            ops: vec![ace_core::source::age_component::ScenarioOp::Send {
                instance: 0,
                msg: "Submit".to_string(),
            }],
        },
        &ComponentInitials {
            per_instance: vec![Vec::new()],
        },
    )
    .map(|_| ())
}

const WIDGET: &str = "\nwidget App {\n    model {\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: card } }\n    }\n    on {\n        .Submit -> { .n += 1 }\n    }\n}\n";

/// ①字面量 style 声明 + view `style: NAME` 引用——整源编译通过（替换后
/// 下游无 ident 引用残留）。
#[test]
fn p1072_component_style_literal_compiles() {
    compile(&format!("style card = \"bg-card rounded\"\n{WIDGET}"))
        .expect("literal style compiles");
}

/// ②本地 ident 复制（`style alias_card = card`）——折叠为同值字面量。
/// （`alias` 是全语言面保留字 token——保留字名走文法层拒绝，不在此测。）
#[test]
fn p1072_component_style_local_copy_folds() {
    compile(&format!(
        "style card = \"bg-card rounded\"\nstyle alias_card = card\n{WIDGET}"
    ))
    .expect("local ident copy folds to the literal");
}

/// ③重名 → `source.module-duplicate-declaration`（981 (c)1 同码）。
#[test]
fn p1072_component_style_duplicate_is_named() {
    let err = compile(&format!("style card = \"a\"\nstyle card = \"b\"\n{WIDGET}"))
        .expect_err("duplicate style names are rejected");
    assert_eq!(
        err[0].code, "source.module-duplicate-declaration",
        "{err:?}"
    );
}

/// ④导入名引用（`use themes.themes: Auto` 后 `style: Auto`）→ `style-call`
/// 具名拒（跨模块=包面同族第四臂）。
#[test]
fn p1072_component_style_imported_reference_is_named() {
    let src = "\nwidget App {\n    model {\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: Auto } }\n    }\n    on {\n        .Submit -> { .n += 1 }\n    }\n}\n";
    let err = compile(&format!("use themes.themes: Auto, Solarized\n{src}"))
        .expect_err("imported style references are a named pending surface");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑤带参调用形（`style: card(k: "v")`）→ `style-call` 具名拒。
#[test]
fn p1072_component_style_call_form_is_named() {
    let src = "\nwidget App {\n    model {\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: card(k: \"v\") } }\n    }\n}\n";
    let err = compile(&format!("style card = \"a\"\n{src}"))
        .expect_err("style call forms are not folded");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑥未声明名引用（`style: ghost`）→ 具名拒（不静默落 ident）。
#[test]
fn p1072_component_style_unknown_reference_is_named() {
    let src = "\nwidget App {\n    model {\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: ghost } }\n    }\n    on {\n        .Submit -> { .n += 1 }\n    }\n}\n";
    let err = compile(src).expect_err("unknown style references are rejected, not silently kept");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑦复制目标非本模块字面量（`style alias_card = ghost`）→ `style-call` 具名拒。
#[test]
fn p1072_component_style_copy_of_missing_target_is_named() {
    let err = compile(&format!("style alias_card = ghost\n{WIDGET}"))
        .expect_err("copies of unknown targets are rejected");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑧参数化声明在组件面=文法层边界（style-params 步骤缺席——解析期拒，
/// 非投影期具名拒；与 `pub style` 同为包面专属形）。
#[test]
fn p1072_component_style_params_stay_package_face() {
    let err = compile(&format!("style sized(p: str) = \"text-sm\"\n{WIDGET}"))
        .expect_err("parameterised styles are a package-face form");
    assert!(
        err[0].code.starts_with("source."),
        "a syntax-level boundary, still a named source diagnostic: {:?}",
        err[0].code
    );
}

/// ⑨`style` 作普通标识符仍合法（981 (d) 负控——不进 token 表）。
#[test]
fn p1072_component_style_as_plain_identifier_stays_legal() {
    let src = "\nwidget App {\n    model {\n        var style int = 1\n    }\n    view {\n        col { text \"x\" {} }\n    }\n    on {\n        .Submit -> { .style += 1 }\n    }\n}\n";
    compile(src).expect("`style` remains legal as a plain identifier");
}

/// ⑩刀A 第二刀（005 真语料）：`type: "password"` 条目键——`type` 是全局
/// token（type-item 声明头），age-event-key 按 token 直收（spelling 兼容投
/// 影）。红相实证：修复前 005 整源解析失败（错误经恢复链归因到 λ 收尾
/// `}`——"unexpected token in an if branch"）。
#[test]
fn p1072_component_view_entry_type_key_parses() {
    compile(
        "widget App {
    model {
        var email str = \"\"
    }
    view {
        col { input (value: .email) {
            type: \"password\"
        } }
    }
    on {
        .Submit -> { .email = \"\" }
    }
}
",
    )
    .expect("the `type:` entry key parses on the component face");
}

// ---- PLAN-1072 刀B（010 真语料）：view-for + 仅视图数据字段 + 条件样式值 ----

/// ⑪无类型列表字段=仅视图数据（1071 边界）——整源编译通过；字段进
/// `view_data`（不进 fields/state）；原始字面量文本保真承载。
#[test]
fn p1072_view_data_field_carries_literal() {
    let src = "widget App {
    msg { Submit }
    model {
        var n int = 0
        var faqs = [
            { q: \"a\", a: \"b\" },
            { q: \"c\", a: \"d\" }
        ]
    }
    view {
        col { text \"x\" {} }
    }
    on {
        .Submit -> { .n = 1 }
    }
}
";
    let nodes = ace_core::source::age_component::project_component_candidates(src).expect("parses");
    assert_eq!(nodes.len(), 1);
    assert_eq!(
        nodes[0].fields.len(),
        1,
        "only the typed field is a state field"
    );
    assert_eq!(nodes[0].view_data.len(), 1, "faqs is view data");
    let vd = &nodes[0].view_data[0];
    assert_eq!(vd.name, "faqs");
    assert!(
        vd.literal.contains("{ q: \"a\", a: \"b\" }"),
        "raw literal text carried verbatim: {}",
        vd.literal
    );
    compile(src).expect("whole source compiles with the view-data field");
}

#[test]
fn p114_nested_tree_literal_is_preserved_as_read_only_view_data() {
    use ace_core::source::{
        age_component::project_component_candidates,
        cst::FrontBudget,
        packages::{parse_package_source_age, PACKAGES_CAPABILITY},
    };

    let nodes = r#"[
            { id: "docs", children: [
                { id: "docs/plans", children: [
                    { id: "docs/plans/INDEX.md", children: [] },
                ] },
            ] },
            { id: "README.md", children: [] },
        ]"#;
    let src = format!(
        r#"widget App {{
    msg {{ Submit }}
    model {{
        var count int = 0
        nodes = {nodes}
    }}
    view {{
        col {{ text "tree" {{}} }}
    }}
    on {{
        .Submit -> {{ .count = 1 }}
    }}
}}
"#
    );
    let component = project_component_candidates(&src).expect("component source parses");
    let package = parse_package_source_age(
        &src,
        &[PACKAGES_CAPABILITY.to_string()],
        FrontBudget::WORK_OWNED_64MIB,
        &ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
        0,
    )
    .expect("package source parses");

    for widget in [&component[0], &package.age_components[0].0] {
        assert_eq!(widget.fields.len(), 1);
        assert_eq!(widget.fields[0].name, "count");
        assert_eq!(widget.view_data.len(), 1);
        assert_eq!(widget.view_data[0].name, "nodes");
        assert_eq!(widget.view_data[0].literal, nodes);
    }
    compile(&src).expect("unrelated scalar handler remains executable");

    // PLAN-106 改判：view-data 字段写=**惰性收据**——可编译，不落 state。
    let writes_to_view_data = src.replace(".count = 1", ".nodes = 1");
    compile(&writes_to_view_data).expect("view-data write is an inert receipt");
    let nodes = project_component_candidates(&writes_to_view_data).expect("projects");
    assert!(
        nodes[0].view_data.iter().any(|f| f.name == "nodes"),
        "the written field stays view-data (no state slot)"
    );
    let malformed = src.replace("children: []", "children: [");
    assert!(project_component_candidates(&malformed).is_err());
}

#[test]
fn p114_package_imports_preserve_the_authored_name_and_directory() {
    use ace_core::source::{
        age_package::compile_age_package,
        cst::FrontBudget,
        packages::{parse_package_source_age, PackageModuleInput, PACKAGES_CAPABILITY},
    };

    let src = r#"widget FileTreePage {
    use { package: official from "../components" }
    use {
        package: secondary from "../shared"
    }
    msg {}
    model {
        nodes = [{ id: "docs", children: [] }]
    }
    view {
        col { FileTree {} }
    }
}
"#;
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let parse = |source| {
        parse_package_source_age(
            source,
            &caps,
            FrontBudget::WORK_OWNED_64MIB,
            &ComponentScenario {
                instances: 0,
                ops: Vec::new(),
            },
            0,
        )
    };
    let package = parse(src).expect("original import and empty message syntax");
    let widget = &package.age_components[0].0;
    assert!(widget.messages.is_empty());
    let imports: Vec<_> = widget
        .package_imports
        .iter()
        .map(|import| {
            assert_eq!(import.package.span.slice(src), import.package.name);
            (import.package.name.as_str(), import.directory.as_str())
        })
        .collect();
    assert_eq!(imports, [("official", "../components"), ("secondary", "../shared")]);
    assert!(parse(&src.replace("from \"../components\"", "from components")).is_err());

    let units = [PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    }];
    let diagnostics = match compile_age_package(&units, "app::main", LegacyWidgetAlias::Accept, &caps) {
        Err(diagnostics) => diagnostics,
        Ok(_) => panic!("unresolved component packages must not reach a renderer"),
    };
    assert_eq!(diagnostics[0].code, "source.age.component-package-unresolved");
    assert!(diagnostics[0].message.contains("official"));
    assert_eq!(
        diagnostics[0].span.slice(src),
        r#"package: official from "../components""#
    );
}

#[test]
fn p114_package_view_preserves_expanded_node_ids_in_source_order() {
    use ace_core::source::{
        age_component::{ComponentEntryValue, ComponentViewItem},
        ast::ExprKind,
        cst::FrontBudget,
        packages::{parse_package_source_age, PACKAGES_CAPABILITY},
    };

    let src = r#"widget FileTreePage {
    use { package: official from "../components" }
    msg {}
    model {
        nodes = [{ id: "docs", children: [] }]
    }
    view {
        FileTree (nodes: .nodes, default_expanded: ["docs", "docs/plans", "docs/\"draft\"",]) {}
        FileTree (nodes: .nodes, default_expanded: []) {}
    }
}
"#;
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let scenario = ComponentScenario {
        instances: 0,
        ops: Vec::new(),
    };
    let parse = |source| {
        parse_package_source_age(
            source,
            &caps,
            FrontBudget::WORK_OWNED_64MIB,
            &scenario,
            0,
        )
    };
    let syntax = parse(src).expect("file tree view properties parse");
    let tree = &syntax.age_components[0].0.view.tree;
    let expanded: Vec<Vec<&str>> = tree
        .iter()
        .map(|item| {
            let ComponentViewItem::Element(element) = item else {
                panic!("expected a FileTree element");
            };
            assert_eq!(element.tag, "FileTree");
            let value = element.items.iter().find_map(|item| match item {
                ComponentViewItem::Entry(entry) if entry.key == "default_expanded" => {
                    Some(&entry.value)
                }
                _ => None,
            });
            let Some(ComponentEntryValue::Expr(expr)) = value else {
                panic!("missing list property");
            };
            let ExprKind::ArrayLit(ids) = &expr.kind else {
                panic!("list property must retain its values");
            };
            ids.iter()
                .map(|id| match &id.kind {
                    ExprKind::StrLit(text) => text.as_str(),
                    _ => panic!("expected a text ID"),
                })
                .collect()
        })
        .collect();
    assert_eq!(expanded, [vec!["docs", "docs/plans", "docs/\"draft\""], vec![]]);
    assert!(parse(&src.replace("\"docs\", \"docs/plans\"", "\"docs\", 42")).is_err());
}

#[test]
fn p114_package_projects_deep_typed_tree_data_with_the_standard_worker_stack() {
    use ace_core::source::{
        ast::ExprKind,
        cst::FrontBudget,
        packages::{parse_package_source_age, PACKAGES_CAPABILITY},
    };
    std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
        let mut literal = "[{ id: \"leaf\", children: [] }]".to_string();
        for _ in 0..11 {
            literal = format!("[{{ id: \"branch\", children: {literal} }}]");
        }
        let source = format!(
            "widget Tree {{\n model {{ nodes List<TreeNode> = {literal} }}\n view {{ text \"Tree\" {{}} }}\n}}",
        );
        let syntax = parse_package_source_age(
            &source,
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
            &ComponentScenario { instances: 0, ops: Vec::new() },
            0,
        ).expect("project a deep tree's typed model on the standard stack");
        let mut array = &syntax.age_components[0].0.fields[0].init;
        for level in 0..12 {
            let ExprKind::ArrayLit(nodes) = &array.kind else { panic!("level {level}") };
            assert_eq!(nodes.len(), 1);
            let ExprKind::ObjectLit { fields } = &nodes[0].kind else { panic!("level {level}") };
            assert_eq!(fields.len(), 2);
            assert_eq!(
                fields[0].value.span.slice(&source), if level == 11 { "\"leaf\"" } else { "\"branch\"" }
            );
            array = &fields[1].value;
        }
        assert!(matches!(&array.kind, ExprKind::ArrayLit(nodes) if nodes.is_empty()));
    }).unwrap().join().unwrap();
}

#[test]
fn p107_chart_list_parameters_keep_the_element_type_and_authored_default() {
    use ace_core::source::{
        ast::{ExprKind, TypeRef, TypeWord},
        cst::FrontBudget,
        packages::{parse_package_source_age, PACKAGES_CAPABILITY},
    };

    let source = r#"widget Chart (data: List, fields: List<str> = ["desktop", "mobile"], labels: List<str> = [], type: str = "grouped") {
    view { text "Chart" {} }
}
"#;
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let scenario = ComponentScenario { instances: 0, ops: Vec::new() };
    let parse = |source| parse_package_source_age(
        source, &caps, FrontBudget::WORK_OWNED_64MIB, &scenario, 0,
    );
    let parsed = parse(source).unwrap();
    let params = &parsed.age_components[0].0.params;
    assert!(params[0].collection_type.is_none());
    let declared = params[1].collection_type.as_ref().unwrap();
    assert_eq!(declared.span.slice(source), "List<str>");
    assert!(matches!(&declared.kind, TypeRef::Generic { name, args }
        if name.name == "List" && matches!(args[0].kind, TypeRef::Builtin(TypeWord::Str))));
    let ExprKind::ArrayLit(values) = &params[1].default.as_ref().unwrap().kind
        else { panic!("expected an array default") };
    assert_eq!(values.iter().map(|value| value.span.slice(source)).collect::<Vec<_>>(),
        ["\"desktop\"", "\"mobile\""]);
    assert!(matches!(&params[2].default.as_ref().unwrap().kind,
        ExprKind::ArrayLit(values) if values.is_empty()));
    assert_eq!(params[3].name, "type");

    let invalid = source.replace("\"desktop\"", "42");
    let errors = parse(&invalid).err().expect("the list default must be text");
    assert_eq!(errors[0].code, "source.age.widget-param-default");
}

#[test]
fn p114_widget_defaults_and_nested_view_loop_bindings_keep_source_values() {
    use ace_core::source::{
        age_component::ComponentViewItem,
        ast::ExprKind,
        cst::FrontBudget,
        packages::{parse_package_source_age, PACKAGES_CAPABILITY},
    };

    let src = r#"widget Tree (nodes: List, expanded: List = ["project", "src"], selected: str = "", directory: bool = false) {
    view {
        for item in .nodes {
            for position, guide in item.guides {
                text guide {}
            }
        }
    }
}
"#;
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let scenario = ComponentScenario {
        instances: 0,
        ops: Vec::new(),
    };
    let parse = |source| {
        parse_package_source_age(
            source,
            &caps,
            FrontBudget::WORK_OWNED_64MIB,
            &scenario,
            0,
        )
    };
    let syntax = parse(src).expect("widget parameter defaults and nested view loops");
    let node = &syntax.age_components[0].0;
    assert!(node.params[0].default.is_none());
    let expanded = node.params[1].default.as_ref().expect("expanded default");
    let ExprKind::ArrayLit(items) = &expanded.kind else {
        panic!("expected a list default");
    };
    assert_eq!(expanded.span.slice(src), r#"["project", "src"]"#);
    assert_eq!(items.len(), 2);
    assert!(matches!(&items[0].kind, ExprKind::StrLit(id) if id == "project"));
    assert!(matches!(&items[1].kind, ExprKind::StrLit(id) if id == "src"));
    let [ComponentViewItem::For(outer)] = node.view.tree.as_slice() else {
        panic!("expected the outer loop");
    };
    assert_eq!(outer.index, None);
    assert_eq!(outer.value, "item");
    let [ComponentViewItem::For(inner)] = outer.items.as_slice() else {
        panic!("expected the nested loop");
    };
    assert_eq!(inner.index.as_deref(), Some("position"));
    assert_eq!(inner.value, "guide");
    assert_eq!(inner.iter.span.slice(src), "item.guides");

    let bad = src.replace("directory: bool = false", "directory: bool = \"false\"");
    let Err(diags) = parse(&bad) else {
        panic!("rejected a calculated or mismatched default");
    };
    assert_eq!(diags[0].code, "source.age.widget-param-default");
    assert_eq!(diags[0].span.slice(&bad), r#""false""#);
}

#[test]
fn p114_age_package_format_strings_preserve_their_parts_and_source_spans() {
    use ace_core::source::{
        age_package::compile_age_package,
        ast::{Arg, ExprKind},
        cst::FrontBudget,
        packages::{parse_package_source_age, PackageModuleInput, PACKAGES_CAPABILITY},
    };

    let src = r#"widget TreeStatus {
    model {
        count int = 2
        selected str = "src"
    }
    view {
        text (text: f"目录 ${.count + 1} · 选中 ${.selected}") { style: "text-sm" }
    }
}
"#;
    let caps = [PACKAGES_CAPABILITY.to_string(), "ac.runtime.text.v1".to_string()];
    let compile = |src: &str| {
        compile_age_package(
            &[PackageModuleInput {
                package: "gallery".into(),
                version: "1.0.0".into(),
                id: "status".into(),
                source: src.into(),
                provider: None,
            }],
            "gallery::status",
            LegacyWidgetAlias::Accept,
            &caps,
        )
    };
    let compiled = compile(src).expect("formatted text goes through the checked package");
    assert_eq!(compiled.description.components[0].texts.len(), 1);
    let syntax = parse_package_source_age(
        src,
        &caps,
        FrontBudget::WORK_OWNED_64MIB,
        &ComponentScenario { instances: 0, ops: Vec::new() },
        0,
    ).expect("source view retains its authored expression");
    let text = &syntax.age_components[0].0.view.texts[0].expr;
    let ExprKind::Call { callee, args } = &text.kind else { panic!("format call") };
    assert_eq!(callee.name, "__fstr");
    assert_eq!(callee.span.slice(src), "f");
    let [Arg::Pos(prefix), Arg::Pos(count), Arg::Pos(separator), Arg::Pos(selected)] =
        args.as_slice() else { panic!("expected two text and two expression parts") };
    assert_eq!(prefix.span.slice(src), "目录 ");
    assert!(matches!(&prefix.kind, ExprKind::StrLit(value) if value == "目录 "));
    assert_eq!(count.span.slice(src), ".count + 1");
    assert_eq!(separator.span.slice(src), " · 选中 ");
    assert_eq!(selected.span.slice(src), ".selected");

    let bad = src.replace(".count + 1", ".count +");
    let Err(diags) = compile(&bad) else { panic!("malformed interpolation was accepted") };
    assert_eq!(diags[0].code, "source.fstr-syntax");
    assert_eq!(diags[0].span.slice(&bad), ".count +");

    let bad = src.replace(".count + 1", "missing");
    let Err(diags) = compile(&bad) else { panic!("unknown model field was accepted") };
    assert!(diags[0].message.contains("missing"));
    assert!(diags[0].span.slice(&bad).contains("missing"));
}

#[test]
fn p114_age_package_checks_expanded_ids_as_owned_lists() {
    use ace_core::source::{
        age_package::compile_age_package_with_scenario,
        age_component::ScenarioOp,
        packages::{PackageModuleInput, PACKAGES_CAPABILITY},
    };

    let source = r#"widget TreePage {
    msg { Expand, Collapse }
    model {
        expanded List = ["docs", "src"]
    }
    on {
        .Expand -> {
            .expanded = ["docs", "docs/plans", "src"]
        }
        .Collapse -> {
            .expanded = []
        }
    }
    view {
        text f"expanded: ${.expanded.len()}" {}
    }
}
"#;
    let compile = |source: &str| {
        compile_age_package_with_scenario(
            &[PackageModuleInput {
                package: "gallery".into(),
                version: "1.0.0".into(),
                id: "tree".into(),
                source: source.into(),
                provider: None,
            }],
            "gallery::tree",
            LegacyWidgetAlias::Accept,
            &[PACKAGES_CAPABILITY.into()],
            &ComponentScenario {
                instances: 1,
                ops: ["Expand", "Collapse"].into_iter().map(|msg| ScenarioOp::Send {
                    instance: 0,
                    msg: msg.into(),
                }).collect(),
            },
        )
    };
    let compiled = compile(source).expect("checked model list initial, assignment and reset");
    assert_eq!(compiled.checked.domain(), ace_core::hir::HirDomain::Closures);
    assert!(compiled.atom_text.contains("list_new"));
    assert!(compiled.atom_text.contains("list_len"));
    let bad = source.replace(".expanded = []", ".expanded = 42");
    let Err(diagnostics) = compile(&bad) else { panic!("expected a list type error") };
    assert_eq!(diagnostics[0].code, "source.type-mismatch");
    assert_eq!(diagnostics[0].span.slice(&bad), "42");
}

#[test]
fn p114_package_view_binds_widget_sites_to_the_declared_module_closure() {
    use ace_core::source::{
        age_package::compile_age_package,
        packages::{PackageModuleInput, PACKAGES_CAPABILITY},
    };

    let unit = |package: &str, id: &str, source: &str| PackageModuleInput {
        package: package.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    };
    let page = r#"use components.labels: default_label
widget GalleryPage {
    use { package: official from "../components" }
    view {
        col {
            text default_label() {}
            FileTree {}
        }
    }
}
"#;
    let entry = unit("gallery", "filetree_page", page);
    let filetree = unit(
        "official",
        "filetree",
        "use components.labels: default_label\nwidget FileTree {\n    view {\n        text default_label() {}\n        TreeIcon {}\n    }\n}\n",
    );
    let helper = unit(
        "official",
        "labels",
        "pub fn default_label() str { return \"File tree\" }\n",
    );
    let icon = unit(
        "official",
        "tree_icon",
        "widget TreeIcon {\n    view { text \"Icon\" {} }\n}\n",
    );
    let unrelated = unit(
        "other",
        "filetree",
        "widget FileTree {\n    view { text \"Different\" {} }\n}\n",
    );
    let compile = |units: &[PackageModuleInput]| {
        compile_age_package(
            units,
            "gallery::filetree_page",
            LegacyWidgetAlias::Accept,
            &[PACKAGES_CAPABILITY.to_string()],
        )
    };
    let compiled = compile(&[
        entry.clone(), filetree.clone(), icon.clone(), helper.clone(), unrelated,
    ]).expect("widget closure and both directory-qualified imports bind");
    let root_refs = &compiled.widget_refs[&("gallery::filetree_page".into(), "GalleryPage".into())];
    assert_eq!(root_refs.len(), 1);
    assert_eq!(root_refs[0].module, "official::filetree");
    assert_eq!(root_refs[0].node, "FileTree");
    assert_eq!(root_refs[0].span.slice(page), "FileTree {}");
    let child_refs = &compiled.widget_refs[&("official::filetree".into(), "FileTree".into())];
    assert_eq!(child_refs.len(), 1);
    assert_eq!(child_refs[0].module, "official::tree_icon");

    let diagnostics_for = |units: &[PackageModuleInput]| match compile(units) {
        Ok(_) => panic!("incorrect widget closure was accepted"),
        Err(diagnostics) => diagnostics,
    };
    let hiding = unit(
        "gallery", "filetree_page", &page.replace("    use { package: official from \"../components\" }\n", ""),
    );
    let undeclared = diagnostics_for(&[hiding, helper.clone()]);
    assert_eq!(undeclared[0].code, "source.module-missing");
    let missing = diagnostics_for(&[entry.clone(), filetree.clone(), helper.clone()]);
    assert_eq!(missing[0].code, "source.age.widget-unresolved");
    assert!(missing[0].message.contains("TreeIcon"));
    let duplicate = unit("official", "other_icon", &icon.source);
    let ambiguous = diagnostics_for(&[entry, filetree, icon, helper, duplicate]);
    assert_eq!(ambiguous[0].code, "source.age.widget-ambiguous");
    assert!(ambiguous[0].message.contains("official::tree_icon"));
    assert!(ambiguous[0].message.contains("official::other_icon"));
}

#[test]
fn p114_view_message_calls_check_arity_and_lexical_payloads() {
    use ace_core::source::{
        age_package::compile_age_package,
        packages::{PackageModuleInput, PACKAGES_CAPABILITY},
    };

    let source = r#"widget Tree {
    msg { Pick(str, int) }
    model {
        ids List = ["docs", "docs/计划"]
        selected str = ""
    }
    on {
        .Pick(id, position) -> { .selected = id }
    }
    view {
        for pos, row in .ids {
            button "Select" { onclick: .Pick(row, pos) }
        }
        button "Default" { onclick: .Pick(.selected, 0) }
    }
}
"#;
    let compile = |source: &str| compile_age_package(
        &[PackageModuleInput {
            package: "gallery".into(),
            version: "1.0.0".into(),
            id: "tree".into(),
            source: source.into(),
            provider: None,
        }],
        "gallery::tree",
        LegacyWidgetAlias::Accept,
        &[PACKAGES_CAPABILITY.into()],
    );
    let result = compile(source).expect("the current row and index satisfy Pick(str, int)");
    let events = &result.description.components[0].view_events;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].1, "Pick");
    assert_eq!(events[0].2.slice(source), "onclick: .Pick(row, pos)");
    for (invalid, code) in [
        (".Pick(row)", "source.age.event-payload-arity"),
        (".Pick(row, \"wrong\")", "source.type-mismatch"),
        (".Pick(other, pos)", "source.undefined"),
        (".Pik(row, pos)", "source.age.msg-unknown"),
    ] {
        let source = source.replace(".Pick(row, pos)", invalid);
        let diagnostics = compile(&source).err()
            .unwrap_or_else(|| panic!("invalid event `{invalid}` was accepted"));
        assert!(diagnostics.iter().any(|d| d.code == code), "{invalid}: {diagnostics:?}");
    }
}

/// ⑫view `for <i>, <v> in .field` 结构承载——For 项进树（index/value/iter/
/// 子树）；循环体文本叶**不**进 texts（循环变量无绑定域）。
#[test]
fn p1072_view_for_projects_inert() {
    use ace_core::source::age_component::ComponentViewItem;
    let src = "widget App {
    msg { Submit }
    model {
        var n int = 0
        var faqs = [
            { q: \"a\", a: \"b\" }
        ]
    }
    view {
        col {
            for i, f in .faqs {
                text f.q {}
            }
        }
    }
    on {
        .Submit -> { .n = 1 }
    }
}
";
    let nodes = ace_core::source::age_component::project_component_candidates(src).expect("parses");
    let tree = &nodes[0].view.tree;
    fn walk<'a>(items: &'a [ComponentViewItem], has_for: &mut bool, leaf_texts: &mut usize) {
        for it in items {
            match it {
                ComponentViewItem::For(_) => *has_for = true,
                ComponentViewItem::Text(_) => *leaf_texts += 1,
                ComponentViewItem::Element(e) => walk(&e.items, has_for, leaf_texts),
                ComponentViewItem::Cond(c) => {
                    walk(&c.then_items, has_for, leaf_texts);
                    walk(&c.else_items, has_for, leaf_texts);
                }
                ComponentViewItem::Entry(_) => {}
            }
        }
    }
    let mut has_for = false;
    let mut leaf_texts = 0;
    walk(tree, &mut has_for, &mut leaf_texts);
    assert!(has_for, "the loop projects structurally");
    assert_eq!(
        leaf_texts, 0,
        "loop-body leaves do not flatten into texts (loop vars have no binding)"
    );
    assert!(
        ace_core::source::age_component::view_contains_for(tree).is_some(),
        "view_contains_for finds the loop"
    );
}

/// ⑬条件样式值 `style: if <cond> { <lit> } else { <lit> }`——结构承载
///（design 三段同形；10x 真语料形态）。
#[test]
fn p1072_view_entry_conditional_style_value() {
    use ace_core::source::age_component::{ComponentEntryValue, ComponentViewItem};
    let src = "widget App {
    msg { Submit }
    model {
        var dark bool = false
    }
    view {
        col { text \"x\" { style: if .dark { \"a\" } else { \"b\" } } }
    }
    on {
        .Submit -> { .dark = true }
    }
}
";
    let nodes = ace_core::source::age_component::project_component_candidates(src).expect("parses");
    let mut found = false;
    fn walk(items: &[ComponentViewItem], found: &mut bool) {
        for it in items {
            match it {
                ComponentViewItem::Entry(en) => {
                    if let ComponentEntryValue::If {
                        cond,
                        then_value,
                        else_chain,
                        else_value,
                    } = &en.value
                    {
                        *found = true;
                        assert_eq!(en.key, "style");
                        assert!(
                            matches!(cond.kind, ace_core::source::ast::ExprKind::Field { .. }),
                            "cond references the field"
                        );
                        assert!(
                            else_chain.is_empty(),
                            "a single-else conditional carries no chain arms"
                        );
                        assert!(matches!(
                            then_value.kind,
                            ace_core::source::ast::ExprKind::StrLit(_)
                        ));
                        assert!(matches!(
                            else_value.kind,
                            ace_core::source::ast::ExprKind::StrLit(_)
                        ));
                    }
                }
                ComponentViewItem::Element(e) => walk(&e.items, found),
                ComponentViewItem::For(f) => walk(&f.items, found),
                ComponentViewItem::Cond(c) => {
                    walk(&c.then_items, found);
                    walk(&c.else_items, found);
                }
                ComponentViewItem::Text(_) => {}
            }
        }
    }
    walk(&nodes[0].view.tree, &mut found);
    assert!(found, "the conditional style value projects");
    compile(src).expect("whole source compiles");
}

/// ⑭（PLAN-106 授权改判）：handler 写 view-data 字段=**惰性收据**——赋值
/// 可 bind、可编译（展示读 view_data 初值，不落 state cell、不提升为
/// model 字段）；真正未知字段仍具名拒（见 plan106 用例组）。
#[test]
fn p1072_view_data_write_is_inert_receipt() {
    use ace_core::source::age_component::project_component_candidates;
    let src = "widget App {
    msg { Submit }
    model {
        var n int = 0
        var faqs = [
            { q: \"a\", a: \"b\" }
        ]
    }
    view {
        col { text \"x\" {} }
    }
    on {
        .Submit -> { .faqs = .n }
    }
}
";
    compile(src).expect("a view-data write is an inert receipt, not a state write");
    let nodes = project_component_candidates(src).expect("the source projects");
    let node = &nodes[0];
    assert!(
        node.view_data.iter().any(|f| f.name == "faqs"),
        "the list stays view-data (display source)"
    );
    assert!(
        !node.fields.iter().any(|f| f.name == "faqs"),
        "no state slot is created for the inert receipt"
    );
}

/// ⑮负控：无类型非列表初值——文法层拒（age-model-field expect；不开推断）。
#[test]
fn p1072_untyped_scalar_field_stays_rejected() {
    let err = compile(
        "widget App {
    msg { Submit }
    model {
        var n = 5
    }
    view {
        col { text \"x\" {} }
    }
    on {
        .Submit -> { .n = 1 }
    }
}
",
    )
    .expect_err("untyped scalar initializers stay syntax-rejected (no inference)");
    assert!(
        err[0].message.contains("view-data list literal"),
        "the age view-data expect names the boundary: {:?}",
        err[0].message
    );
}

/// ⑯T-03 对齐（expose 点前缀）：`expose { .Inc, .count }` 与多行无逗号形
/// `expose {\n .Inc\n .count\n}` 在组件面编译通过并投影为 `role == "expose"`
/// 行；既有裸名形 `expose { alpha, beta }` 同律保留。
#[test]
fn p1072_expose_dot_prefixed_names_project_and_compile() {
    let dot_comma = "widget App {
    msg { Inc }
    model {
        var count int = 0
    }
    expose { .Inc, .count }
    view {
        col { text \"x\" {} }
    }
    on {
        .Inc -> { .count += 1 }
    }
}
";
    let dot_multiline = "widget App {
    msg { Inc }
    model {
        var count int = 0
    }
    expose {
        .Inc
        .count
    }
    view {
        col { text \"x\" {} }
    }
    on {
        .Inc -> { .count += 1 }
    }
}
";
    let bare = "widget App {
    msg { Inc }
    model {
        var count int = 0
    }
    expose { alpha, beta }
    view {
        col { text \"x\" {} }
    }
    on {
        .Inc -> { .count += 1 }
    }
}
";
    for (label, src, want) in [
        ("dot-comma", dot_comma, vec!["Inc", "count"]),
        ("dot-multiline", dot_multiline, vec!["Inc", "count"]),
        ("bare", bare, vec!["alpha", "beta"]),
    ] {
        let nodes = ace_core::source::age_component::project_component_candidates(src)
            .unwrap_or_else(|e| panic!("{label} project failed: {e:?}"));
        let names: Vec<&str> = nodes[0]
            .members
            .iter()
            .filter(|m| m.role == "expose")
            .map(|m| m.name.as_str())
            .collect();
        assert_eq!(names, want, "{label} expose member names");
        let caps = vec!["ac.runtime.text.v1".to_string()];
        ace_core::source::age_component::compile_component_scenario_states(
            src,
            "p1072-expose.at",
            LegacyWidgetAlias::Accept,
            &caps,
            &ComponentScenario {
                instances: 1,
                ops: vec![ace_core::source::age_component::ScenarioOp::Send {
                    instance: 0,
                    msg: "Inc".to_string(),
                }],
            },
            &ComponentInitials {
                per_instance: vec![Vec::new()],
            },
        )
        .unwrap_or_else(|e| panic!("{label} compile failed: {e:?}"));
    }
}

/// ⑰T-04 负控：条件样式两臂含未声明标识符 `ghost` → `source.age.style-call` 具名拒。
#[test]
fn p1072_conditional_style_unknown_ident_is_named() {
    let src = "\nwidget App {\n    model {\n        var dark bool = false\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: if .dark { ghost } else { \"b\" } } }\n    }\n    on {\n        .Submit -> { .n += 1 }\n    }\n}\n";
    let err = compile(src).expect_err("conditional style unknown ident must fail closed");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑱T-04 负控：条件样式两臂含跨模块导入样式名 `Auto` → `source.age.style-call` 具名拒。
#[test]
fn p1072_conditional_style_imported_ident_is_named() {
    let src = "use themes.themes: Auto, Solarized\nwidget App {\n    model {\n        var dark bool = false\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: if .dark { \"a\" } else { Auto } } }\n    }\n    on {\n        .Submit -> { .n += 1 }\n    }\n}\n";
    let err = compile(src).expect_err("conditional style imported ident must fail closed");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑲T-04 负控：条件样式两臂含参数化或未声明调用形 → `source.age.style-call` 具名拒。
#[test]
fn p1072_conditional_style_call_form_is_named() {
    let declared_call = "style card = \"p-2\"\nwidget App {\n    model {\n        var dark bool = false\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: if .dark { card(k: \"v\") } else { \"b\" } } }\n    }\n    on {\n        .Submit -> { .n += 1 }\n    }\n}\n";
    let err1 =
        compile(declared_call).expect_err("conditional declared style call must fail closed");
    assert_eq!(err1[0].code, "source.age.style-call", "{err1:?}");

    let undeclared_call = "widget App {\n    model {\n        var dark bool = false\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: if .dark { \"a\" } else { ghost_fn(1) } } }\n    }\n    on {\n        .Submit -> { .n += 1 }\n    }\n}\n";
    let err2 =
        compile(undeclared_call).expect_err("conditional undeclared style call must fail closed");
    assert_eq!(err2[0].code, "source.age.style-call", "{err2:?}");
}
