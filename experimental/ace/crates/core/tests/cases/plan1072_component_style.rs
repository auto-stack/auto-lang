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
                        else_value,
                    } = &en.value
                    {
                        *found = true;
                        assert_eq!(en.key, "style");
                        assert!(
                            matches!(cond.kind, ace_core::source::ast::ExprKind::Field { .. }),
                            "cond references the field"
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

/// ⑭负控：handler 写视图数据字段——具名拒（未声明 model 字段；1071 边界
/// 的 fail-closed 面：view-data 不是可写 state 通道）。
#[test]
fn p1072_view_data_write_is_named() {
    let err = compile(
        "widget App {
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
",
    )
    .expect_err("writing a view-data field is rejected");
    assert!(
        err[0]
            .message
            .contains("assigns undeclared model field `faqs`"),
        "named reject: {:?}",
        err[0].message
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
