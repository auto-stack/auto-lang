//! PLAN-105 T-02（U03 W1）永久回归：组件面 **entry 值**的解析边界。
//!
//! 判据（真语料 013/016/022 的最小复现）：元素体里的 `checked: .a == 0 && .c > 0`
//! 这类比较/逻辑绑定必须解析成普通表达式；而 `onclick: .Msg(args)` 仍是事件绑定
//! （投影为裸 `.Msg` Field、实参作宿主通道丢弃），裸 `.Submit` 走表达式面由
//! `is_event + msg 名单` 识别。修复前 component 面把裸 `.a` 贪吃成事件绑定，
//! 把 `== 0` 留给父规则 → 整 view 回滚到 item 级（`source.age.structure`）。
use ace_core::source::age_component::{
    ComponentEntryValue, ComponentScenario, ComponentViewEntry, ComponentViewItem,
};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn entry(src: &str, key: &str) -> ComponentViewEntry {
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .unwrap_or_else(|ds| panic!("source must project: {ds:?}"));
    let mut found = Vec::new();
    fn walk(items: &[ComponentViewItem], key: &str, found: &mut Vec<ComponentViewEntry>) {
        for it in items {
            match it {
                ComponentViewItem::Entry(en) => {
                    if en.key == key {
                        found.push(en.clone());
                    }
                }
                ComponentViewItem::Element(e) => walk(&e.items, key, found),
                ComponentViewItem::For(f) => walk(&f.items, key, found),
                ComponentViewItem::Cond(c) => {
                    walk(&c.then_items, key, found);
                    walk(&c.else_items, key, found);
                }
                ComponentViewItem::Text(_) => {}
            }
        }
    }
    for n in &nodes {
        walk(&n.view.tree, key, &mut found);
    }
    found.remove(0)
}

fn compiles(src: &str) -> bool {
    let caps = vec![PACKAGES_CAPABILITY.to_string()];
    let scenario = ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    compile_age_package_with_scenario(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "app".into(),
            source: src.into(),
            provider: None,
        }],
        "app::app",
        LegacyWidgetAlias::Accept,
        &caps,
        &scenario,
    )
    .is_ok()
}

/// 013 真语料形：比较 + 逻辑的 `checked` 绑定是表达式，不是事件绑定。
#[test]
fn p105_component_entry_comparison_binding_projects_as_expr() {
    let src = "widget App {\n\
        msg { T }\n\
        model {\n\
            var a int = 0\n\
            var c int = 1\n\
        }\n\
        view {\n\
            col {\n\
                checkbox {\n\
                    checked: .a == 0 && .c > 0\n\
                    onclick: .T\n\
                }\n\
            }\n\
        }\n\
    }\n";
    let en = entry(src, "checked");
    assert!(
        matches!(en.value, ComponentEntryValue::Expr(_)),
        "comparison binding must project as Expr, got {:?}",
        en.value
    );
    assert!(!en.is_event, "`checked` is not an event key");
    assert!(compiles(src), "package face must compile the same source");
}

/// `onclick: .Msg(arg)` 仍是事件绑定：投影为裸 `.Msg` Field（实参作宿主通道丢弃）。
#[test]
fn p105_component_entry_msg_with_args_is_event_binding() {
    let src = "widget App {\n\
        msg { SetMode(str) }\n\
        model { var m str = \"\" }\n\
        view {\n\
            button {\n\
                onclick: .SetMode(\"basic\")\n\
            }\n\
        }\n\
    }\n";
    let en = entry(src, "onclick");
    assert!(en.is_event, "`onclick` must be an event key");
    match &en.value {
        ComponentEntryValue::Expr(expr) => {
            assert!(
                matches!(
                    expr.kind,
                    ace_core::source::ast::ExprKind::Field { .. }
                ),
                "event binding projects to a bare field"
            );
        }
        other => panic!("expected bare-field event binding, got {other:?}"),
    }
    assert!(compiles(src));
}

/// 裸 `.Submit`（无括号）走表达式面；投影层据事件键 + msg 名单仍识别为事件。
#[test]
fn p105_component_entry_bare_msg_stays_field() {
    let src = "widget App {\n\
        msg { Submit }\n\
        model { var s str = \"\" }\n\
        view {\n\
            button {\n\
                onclick: .Submit\n\
            }\n\
        }\n\
    }\n";
    let en = entry(src, "onclick");
    assert!(en.is_event);
    assert!(
        matches!(en.value, ComponentEntryValue::Expr(_)),
        "bare `.Submit` stays a field expression"
    );
    assert!(compiles(src));
}

/// 条件条目值（1072 刀B）与上面的收窄共存：`style: if …` 仍投影为 If。
#[test]
fn p105_component_entry_conditional_style_still_projects() {
    let src = "widget App {\n\
        msg { T }\n\
        model { var d bool = false }\n\
        view {\n\
            button {\n\
                onclick: .T\n\
                style: if .d { \"a\" } else { \"b\" }\n\
            }\n\
        }\n\
    }\n";
    let en = entry(src, "style");
    assert!(
        matches!(en.value, ComponentEntryValue::If { .. }),
        "conditional style keeps its structural carrier"
    );
    assert!(compiles(src));
}

/// 016 真语料形：msg 逐行一成员、每行（含末项）以逗号结尾；载荷变体同行括号。
#[test]
fn p105_msg_trailing_comma_and_per_line_payloads() {
    let src = "widget App {\n\
        msg {\n\
            PrevMonth,\n\
            NextMonth,\n\
            Today,\n\
            SelectDay(str),\n\
            ToggleSettings,\n\
            ToggleDarkMode,\n\
            SetTheme(str),\n\
            SetAccent(str),\n\
        }\n\
        model { var n int = 0 }\n\
        view { text \"x\" {} }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("016-shaped msg block projects");
    let names: Vec<String> = nodes[0]
        .messages
        .iter()
        .map(|m| m.name.clone())
        .collect();
    assert_eq!(
        names,
        vec![
            "PrevMonth",
            "NextMonth",
            "Today",
            "SelectDay",
            "ToggleSettings",
            "ToggleDarkMode",
            "SetTheme",
            "SetAccent"
        ]
    );
    // 载荷保留（SelectDay(str) 等三例）。
    assert!(nodes[0]
        .messages
        .iter()
        .any(|m| m.name == "SelectDay" && m.payload.len() == 1));
}

/// 016 真语料形：单行逗号分隔的视图数据列表（无类型 model 字段）。
#[test]
fn p105_view_data_inline_list_literal() {
    let src = "widget App {\n\
        model {\n\
            var weekday_labels = [\"Su\", \"Mo\", \"Tu\", \"We\", \"Th\", \"Fr\", \"Sa\"]\n\
        }\n\
        view { text \"x\" {} }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("inline view-data list projects");
    let field = nodes[0]
        .view_data
        .iter()
        .find(|f| f.name == "weekday_labels")
        .expect("weekday_labels is view data");
    assert!(
        field.literal.contains("\"Su\"") && field.literal.contains("\"Sa\""),
        "literal keeps the original source text"
    );
}

/// 016 真语料形：`text f"${.store.year}"` 在组件面投影为文本叶（不因 f-string 回滚）。
#[test]
fn p105_fstring_text_leaf_projects() {
    let src = "widget App {\n\
        model {\n\
            var n int = 0\n\
            var s str = \"\"\n\
        }\n\
        view {\n\
            col {\n\
                text f\"v=${.s}/{.n}\" { style: \"x\" }\n\
            }\n\
        }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("f-string text leaf projects");
    assert_eq!(nodes.len(), 1);
    assert!(
        !nodes[0].view.texts.is_empty(),
        "the f-string text leaf is projected"
    );
}

/// 013/016 真语料形：`text .n`（int 字段）走 TextDisplay，组件面/包面都过 verify。
#[test]
fn p105_int_field_text_leaf_displays() {
    let src = "widget App {\n\
        msg { T }\n\
        model { var n int = 0\n var s str = \"\" }\n\
        view {\n\
            col {\n\
                text .n {}\n\
                text .s {}\n\
            }\n\
        }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("int+str text leaves project");
    assert_eq!(nodes[0].view.texts.len(), 2);
assert!(compiles(src), "package face verifies the same source");
}

/// 018/021/023 真语料形：`link (to: "/") { … }`——`link` 是保留 token，
/// 元素标签位必须单独收形（此前整块回滚到 bare-element 兜底）。
#[test]
fn p105_link_element_tag_projects() {
    let src = "widget App {\n\
        model { var n int = 0 }\n\
        view {\n\
            col {\n\
                link (to: \"/\") { text \"Library\" {} }\n\
                link (to: \"/settings\") {}\n\
            }\n\
        }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("link elements project");
    let mut tags = Vec::new();
    fn walk(items: &[ace_core::source::age_component::ComponentViewItem], tags: &mut Vec<String>) {
        for it in items {
            match it {
                ace_core::source::age_component::ComponentViewItem::Element(e) => {
                    tags.push(e.tag.clone());
                    walk(&e.items, tags);
                }
                ace_core::source::age_component::ComponentViewItem::For(f) => walk(&f.items, tags),
                ace_core::source::age_component::ComponentViewItem::Cond(c) => {
                    walk(&c.then_items, tags);
                    walk(&c.else_items, tags);
                }
                _ => {}
            }
        }
    }
    walk(&nodes[0].view.tree, &mut tags);
    assert_eq!(tags, vec!["col", "link", "link"]);
}

/// 012 真语料形：SVG 元素属性连字符键（`stroke-width` / `stroke-linecap`）。
#[test]
fn p105_dashed_entry_keys_project() {
    let src = "widget App {\n\
        model { var n int = 0 }\n\
        view {\n\
            col {\n\
                circle (cx: \"120\", stroke-width: \"5\", stroke-linecap: \"round\") {}\n\
            }\n\
        }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("dashed keys project");
    let mut keys: Vec<String> = Vec::new();
    fn walk(items: &[ace_core::source::age_component::ComponentViewItem], out: &mut Vec<String>) {
        for it in items {
            if let ace_core::source::age_component::ComponentViewItem::Element(e) = it {
                for child in &e.items {
                    if let ace_core::source::age_component::ComponentViewItem::Entry(en) = child {
                        out.push(en.key.clone());
                    }
                }
                walk(&e.items, out);
            }
        }
    }
    walk(&nodes[0].view.tree, &mut keys);
    assert_eq!(keys, vec!["cx", "stroke-width", "stroke-linecap"]);
    assert!(compiles(src), "package face compiles the dashed keys too");
}

/// 016/018 真语料形：括号属性元素在**包面**也必须可解析（此前 component-only）。
#[test]
fn p105_props_element_compiles_in_package() {
    let src = "widget App {\n\
        msg { T }\n\
        model { var s str = \"\" }\n\
        view {\n\
            col {\n\
                input (value: .s) {}\n\
            }\n\
        }\n\
    }\n";
    assert!(compiles(src), "props element is admitted by the package face");
}

/// 012 真语料形：view 级 `for` 的**单变量**形（`for r in .w_rows`）与双变量形
/// 必须同时可用；单变量形的 index 为空串。
#[test]
fn p105_view_for_single_and_indexed_variables() {
    let src = "widget App {\n\
        model {\n\
            var rows = [\"x\"]\n\
        }\n\
        view {\n\
            col {\n\
                for r in .rows {\n\
                    text \"a\" {}\n\
                }\n\
                for i, r in .rows {\n\
                    text \"b\" {}\n\
                }\n\
            }\n\
        }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("both for shapes project");
    let mut fors: Vec<(String, String)> = Vec::new();
    fn walk(
        items: &[ace_core::source::age_component::ComponentViewItem],
        out: &mut Vec<(String, String)>,
    ) {
        for it in items {
            match it {
                ace_core::source::age_component::ComponentViewItem::For(f) => {
                    out.push((f.index.clone(), f.value.clone()));
                    walk(&f.items, out);
                }
                ace_core::source::age_component::ComponentViewItem::Element(e) => {
                    walk(&e.items, out)
                }
                _ => {}
            }
        }
    }
    walk(&nodes[0].view.tree, &mut fors);
    assert_eq!(
        fors,
        vec![
            (String::new(), "r".to_string()),
            ("i".to_string(), "r".to_string())
        ]
    );
    assert!(compiles(src), "package face admits both for shapes");
}

/// 012 真语料形：handler 体内的 `var` 局部量（widget 拼写）+ `let` 拼写同义；
/// `if` 分支与 f-string 初值同时成立。局部量**再赋值**（`b = "0" + b`）仍具名
/// 拒绝（binder 按模型字段判；hook 局部再赋值是后续刀）。
#[test]
fn p105_handler_var_locals_and_fstring() {
    let src_ok = "widget App {\n\
        msg { T }\n\
        model { var n int = 0\n var s str = \"\" }\n\
        view { text \"x\" {} }\n\
        on {\n\
            .T -> {\n\
                var a = .n % 60\n\
                var b = f\"${a}\"\n\
                .s = f\"${b}\"\n\
            }\n\
        }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src_ok)
        .expect("var locals and f-strings project");
    let handler = &nodes[0].handlers[0];
    let names: Vec<&str> = handler
        .stmts
        .iter()
        .filter_map(|s| match s {
            ace_core::source::age_component::ComponentStmt::Let { name, .. } => {
                Some(name.name.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(names, vec!["a", "b"]);
    assert!(compiles(src_ok), "package face admits var locals and handler f-strings");

    // 局部量再赋值（012 秒表 `if cc < 10 { cc_str = "0" + cc_str }`）：裸名先查
    // 执行路径绑定表（hook-local 遮蔽同拼写 model 字段），命中即局部写；被再
    // 赋值的 let 绑定自动翻成可变（verify 的可变写要求）。两面都必须通过。
    let src_reassign = src_ok.replace(
        ".s = f\"${b}\"",
        "if a < 10 { b = \"0\" + b }\n                .s = b",
    );
    let nodes = ace_core::source::age_component::project_component_candidates(&src_reassign)
        .expect("local reassignment projects");
    let handler = &nodes[0].handlers[0];
    let has_if = handler
        .stmts
        .iter()
        .any(|s| matches!(s, ace_core::source::age_component::ComponentStmt::If { .. }));
    assert!(has_if, "the reassignment stays inside its `if` branch");
    assert!(
        compiles(&src_reassign),
        "package face admits hook-local reassignment (mutable let)"
    );
}
/// 012 真语料形：元素体**单行逗号分隔**多 entry
/// （`button "时−" { onclick: .ADnH, style: stepper_btn }`）。
#[test]
fn p105_single_line_comma_entries_project() {
    let src = "style stepper_btn = \"w-8 h-8\"\nwidget App {\n    msg { ADnH }\n    model { var n int = 0 }\n    view {\n        col {\n            button \"时−\" { onclick: .ADnH, style: stepper_btn }\n        }\n    }\n}\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("single-line comma entries project");
    let mut keys: Vec<String> = Vec::new();
    fn walk(items: &[ace_core::source::age_component::ComponentViewItem], out: &mut Vec<String>) {
        for it in items {
            if let ace_core::source::age_component::ComponentViewItem::Element(e) = it {
                for child in &e.items {
                    if let ace_core::source::age_component::ComponentViewItem::Entry(en) = child {
                        out.push(en.key.clone());
                    }
                }
                walk(&e.items, out);
            }
        }
    }
    walk(&nodes[0].view.tree, &mut keys);
    assert_eq!(keys, vec!["onclick", "style"]);
    assert!(compiles(src), "package face admits single-line comma bodies");
}

/// 012 真语料形：handler 里**无 else 的 if** + 局部量再赋值（秒表/倒计时段）。
#[test]
fn p105_handler_if_without_else() {
    let src = "widget App {\n\
        msg { T }\n\
        model { var n int = 0\n var s str = \"\" }\n\
        view { text \"x\" {} }\n\
        on {\n\
            .T -> {\n\
                var a = f\"${.n}\"\n\
                if .n < 10 { a = \"0\" + a }\n\
                .s = a\n\
            }\n\
        }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("if-without-else projects");
    let handler = &nodes[0].handlers[0];
    assert!(handler
        .stmts
        .iter()
        .any(|s| matches!(s, ace_core::source::age_component::ComponentStmt::If { els: None, .. })));
    assert!(compiles(src), "package face admits if-without-else handlers");
}

/// 012 真语料形：列表索引表达式 `.offsets[.local_city]`——AGE 两面此前**没有**
/// 索引后缀步（索引步只开在 aggregates/auto/closures/scheduler/config-document），
/// 整个 handler 回滚成结构错误。现在解析/投影通过；包面以具名
/// `component-unsupported` 拒（列表 state 语义未接，不静默当 0）。
#[test]
fn p105_list_index_expr_projects() {
    let src = "widget App {\n\
        msg { T }\n\
        model { var offsets = [8, 0, -5]\n var local_city int = 0\n var s str = \"\" }\n\
        view { text \"x\" {} }\n\
        on {\n\
            .T -> {\n\
                var ls = .offsets[.local_city] * 3600\n\
                .s = f\"${ls}\"\n\
            }\n\
        }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("list index expression projects");
    let handler = &nodes[0].handlers[0];
    let has_let = handler
        .stmts
        .iter()
        .any(|s| matches!(s, ace_core::source::age_component::ComponentStmt::Let { .. }));
    assert!(has_let, "index expression survives as a hook-local let");
    // 包面：解析过，但列表索引语义未接 → 具名 component-unsupported（非结构回滚）。
    let caps = vec![PACKAGES_CAPABILITY.to_string()];
    let scenario = ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    let err = compile_age_package_with_scenario(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "app".into(),
            source: src.into(),
            provider: None,
        }],
        "app::app",
        LegacyWidgetAlias::Accept,
        &caps,
        &scenario,
    )
    .expect_err("list index stays a named pending surface in the package face");
    assert!(
        err.iter().any(|d| d.code == "source.age.component-unsupported"),
        "named reject expected, got {err:?}"
    );
}

/// 016 真语料形：**连字符元素标签**（`grid-item`）——自定义元素名在标签位
/// 收 `ident ('-' ident)*`，投影按键节点源文本取整名。
#[test]
fn p105_hyphenated_element_tags_project() {
    let src = "widget App {\n\
        model { var n int = 0 }\n\
        view {\n\
            col {\n\
                grid-item { text \"x\" {} }\n\
                scroll-area { text \"y\" {} }\n\
            }\n\
        }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("hyphenated tags project");
    let mut tags: Vec<String> = Vec::new();
    fn walk(items: &[ace_core::source::age_component::ComponentViewItem], out: &mut Vec<String>) {
        for it in items {
            match it {
                ace_core::source::age_component::ComponentViewItem::Element(e) => {
                    out.push(e.tag.clone());
                    walk(&e.items, out);
                }
                ace_core::source::age_component::ComponentViewItem::For(f) => walk(&f.items, out),
                ace_core::source::age_component::ComponentViewItem::Cond(c) => {
                    walk(&c.then_items, out);
                    walk(&c.else_items, out);
                }
                _ => {}
            }
        }
    }
    walk(&nodes[0].view.tree, &mut tags);
    assert_eq!(tags, vec!["col", "grid-item", "scroll-area"]);
    assert!(compiles(src), "package face admits hyphenated tags");
}

/// 012 真语料形：handler 内的**复合赋值**（`civil_y += 1` / `if … { civil_m = mp - 9 }`）。
#[test]
fn p105_handler_compound_assign() {
    let src = "widget App {\n\
        msg { T }\n\
        model { var n int = 0 }\n\
        view { text \"x\" {} }\n\
        on {\n\
            .T -> {\n\
                var y = 1\n\
                if y < 5 { y += 2 }\n\
                .n += y\n\
            }\n\
        }\n\
    }\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("compound assignments project");
    let handler = &nodes[0].handlers[0];
    let mut binaries = 0usize;
    fn walk(stmts: &[ace_core::source::age_component::ComponentStmt], n: &mut usize) {
        for st in stmts {
            match st {
                ace_core::source::age_component::ComponentStmt::Assign(a) => {
                    if matches!(a.rhs.kind, ace_core::source::ast::ExprKind::Binary { .. }) {
                        *n += 1;
                    }
                }
                ace_core::source::age_component::ComponentStmt::If { then, els, .. } => {
                    walk(then, n);
                    if let Some(e) = els {
                        walk(e, n);
                    }
                }
                _ => {}
            }
        }
    }
    walk(&handler.stmts, &mut binaries);
    assert!(binaries >= 2, "both compound assignments desugar to binary writes");
    assert!(compiles(src), "package face admits compound assignments on bound names");
}
