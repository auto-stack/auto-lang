//! PLAN-981（SOURCE 批，IF-981-MODULE）：AGE 包面的 **AV 真实值**回归。
//!
//! 这是 979 V-09④ 要的那条腿的 AV 半边：两文件 AGE 工程 → 同一包前门 →
//! 两实例 2:1 定向事件、各 4 次 `Life.Tick` → 经共享 AV 链观察**逐实例状态与
//! 文本**，并与独立算术 oracle 对齐。
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const HELPER: &str = "pub fn seed() int { return 0 }\n\
pub fn label(n int) str {\n    if n == 6 {\n        return \"six\"\n    }\n    return \"other\"\n}\n";

const MAIN: &str = "use helper.seed\n\
use helper.label\n\
node Counter {\n\
    model { count int = seed() }\n\
    on Inc { count = count + 1 }\n\
    on Life.Tick { count = count + 1 }\n\
    view { text label(count) {} }\n\
}\n";

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn caps() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.to_string()]
}

#[test]
fn p107_legacy_chart_memo_preserves_view_and_disjoint_list_transfers() {
    use ace_av::age_package::{evaluate_widget_view, WidgetViewItem};

    let page = r#"widget Gallery {
        use { package: official from "../components" }
        view { Plot {} }
    }"#;
    let plot = r#"type PointLabel { c Str }
    widget Plot {
        msg { Init }
        model {
            left List<int> = []
            right List<int> = []
            choice int = 1
        }
        on {
            .Init -> {
                var key int = .choice
                var points List<int> = [13, 21]
                var shade str = "blue"
                var local0 List<PointLabel> = []
                var local1 List<PointLabel> = []
                if key == 0 {
                    .left = points
                    local0.push({ c: shade })
                }
                if key == 1 {
                    .right = points
                    local1.push({ c: shade })
                }
            }
        }
        view {
            memo (deps: [".left", ".right"]) {
                text f"${.left.len()}/${.right.len()}" {}
            }
        }
    }"#;
    let compile = |plot: &str| {
        compile_age_package_with_scenario(
            &[
                input("gallery", page),
                PackageModuleInput { package: "official".into(), ..input("plot", plot) },
            ],
            "app::gallery",
            LegacyWidgetAlias::Accept,
            &caps(),
            &ComponentScenario {
                instances: 1,
                ops: vec![ScenarioOp::ForNode {
                    node: 1,
                    op: Box::new(ScenarioOp::Send { instance: 0, msg: "Init".into() }),
                }],
            },
        )
    };
    let observe = |source: &str| {
        let package = compile(source).expect("checked chart package");
        let binding = &package.widget_refs[&("app::gallery".into(), "Gallery".into())][0];
        evaluate_widget_view(&package, binding, 0, &Default::default())
    };
    assert_eq!(observe(plot).unwrap(), [WidgetViewItem::Text("0/2".into())]);
    assert_eq!(
        observe(&plot.replace("choice int = 1", "choice int = 0")).unwrap(),
        [WidgetViewItem::Text("2/0".into())],
    );
    let overlap = plot.replace("if key == 1", "if key == 0");
    let error = observe(&overlap).expect_err("overlapping transfers still require distinct ownership");
    assert!(error.contains("mir.use-after-move"), "{error}");

    let bad_hint = plot.replace("memo (deps:", "memo (unknown:");
    let errors = compile(&bad_hint).expect_err("memo accepts only the deps hint");
    assert!(errors.iter().any(|diag| diag.code == "source.age.widget-view-unsupported"), "{errors:?}");
}

#[test]
fn p107_checked_widget_views_carry_active_timers_and_copy_list_inputs() {
    use ace_av::age_package::{evaluate_package_entry_view, evaluate_widget_view, WidgetTimer, WidgetViewItem};

    let page = r#"widget Gallery {
        use { package: official from "../components" }
        msg { Play, Pause }
        model {
            interval int = 400
            running str = "false"
            count int = 0
        }
        on {
            .Play -> { .running = "true" }
            .Pause -> { .running = "false" }
            .Tick -> { .count = .count + 1 }
        }
        view { Plot {} }
    }"#;
    let plot = r#"widget Plot (labels: List<str> = ["Desktop", "Mobile"]) {
        msg { Init, Fade }
        model {
            seen str = ""
            active bool = true
        }
        timer { Fade (every_ms: 33, when: .active) }
        on {
            .Init -> {
                var first = .labels
                first.push("Tablet")
                var second = .labels
                .seen = f"${first.len()}/${second.len()}/${.labels.len()}"
            }
            .Fade -> { .active = false }
        }
        view { text .seen {} }
    }"#;
    let node_event = |node, msg: &str| ScenarioOp::ForNode {
        node,
        op: Box::new(ScenarioOp::Send { instance: 0, msg: msg.into() }),
    };
    let compile = |ops| compile_age_package_with_scenario(
        &[
            input("gallery", page),
            PackageModuleInput { package: "official".into(), ..input("plot", plot) },
        ],
        "app::gallery",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario { instances: 1, ops },
    ).expect("checked timers and repeated list prop reads");
    let verify = |package: &_, live: bool| {
        let root = evaluate_package_entry_view(package, 0, &Default::default()).unwrap();
        let tick = WidgetViewItem::Timer(WidgetTimer { message: "Tick".into(), every_ms: 400 });
        assert_eq!(root.contains(&tick), live);
        let binding = &package.widget_refs[&("app::gallery".into(), "Gallery".into())][0];
        let child = evaluate_widget_view(package, binding, 0, &Default::default()).unwrap();
        let fade = WidgetViewItem::Timer(WidgetTimer { message: "Fade".into(), every_ms: 33 });
        assert_eq!(child.contains(&fade), live);
        assert_eq!(child[0], WidgetViewItem::Text("3/2/2".into()));
    };
    verify(&compile(vec![node_event(0, "Play"), node_event(1, "Init")]), true);
    verify(&compile(vec![
        node_event(0, "Play"),
        node_event(0, "Pause"),
        node_event(1, "Init"),
        node_event(1, "Fade"),
    ]), false);
}

#[test]
fn p107_widget_calls_keep_their_typed_arguments_and_instance() {
    use ace_core::atom_text::Span;
    use ace_core::source::age_component::InitialValue;

    let source = r#"widget Viewer {
        msg { Next(str), ApplyView(str) }
        model {
            applied str = ""
            requested str = ""
        }
        on {
            .Next(path str) -> {
                if true {
                    .ApplyView(path)
                    .requested = path
                }
            }
            .ApplyView(path str) -> { .applied = f"image:${path}" }
        }
        view { text f"${.applied}/${.requested}" {} }
    }"#;
    let compile = |source: &str| compile_age_package_with_scenario(
        &[input("main", source)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::send_args_from_values(
                1, "Next", &[InitialValue::Str("图像".into())], Span { start: 0, end: 0 },
            )],
        },
    );
    let compiled = compile(source).expect("checked nested widget transition");
    let observed = ace_av::age_package::observe_age_package(&compiled, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [
        vec!["/"], vec!["image:图像/图像"],
    ]);
    for (modified, code) in [
        (source.replace(".ApplyView(path)", ".ApplyView(4)"), "source.type-mismatch"),
        (source.replace(".ApplyView(path)", ".Missing(path)"), "source.age.handler-message"),
        (source.replace(".applied = f\"image:${path}\"", ".ApplyView(path)"), "source.age.handler-cycle"),
    ] {
        let errors = compile(&modified).expect_err("invalid nested widget transition");
        assert!(errors.iter().any(|diagnostic| diagnostic.code == code), "{errors:?}");
    }
}

#[test]
fn p107_chart_sites_check_their_own_record_and_replay_independent_state() {
    use ace_av::age_package::{evaluate_package_view, WidgetViewItem};
    let page = r#"widget Gallery {
        use { package: official from "../components" }
        model {
            basic = [{ month: "Jan", value: 2 }]
            multi = [{ month: "Jan", desktop: 7, mobile: 9 }]
        }
        view {
            Plot (data: .basic, key: "value") {}
            Plot (data: .multi, key: "desktop") {}
        }
    }"#;
    let plot = r#"widget Plot (data: List, key: str = "value") {
        msg { Init, Increment }
        model { total int = 0 }
        on {
            .Init -> {
                for row in .data {
                    .total = .total + row[.key]
                }
            }
            .Increment -> { .total = .total + 1 }
        }
        view { text f"${.total}" {} }
    }"#;
    let event = |node, message: &str| ScenarioOp::ForNode {
        node,
        op: Box::new(ScenarioOp::Send { instance: 0, msg: message.into() }),
    };
    let compile = |page: &str, ops| compile_age_package_with_scenario(
        &[
            input("gallery", page),
            PackageModuleInput { package: "official".into(), ..input("plot", plot) },
        ],
        "app::gallery",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario { instances: 1, ops },
    );
    let package = compile(page, vec![event(1, "Init"), event(2, "Init"), event(2, "Increment")])
        .expect("each Plot receives its authored list type");
    let tree = evaluate_package_view(&package, 0, &Default::default()).unwrap();
    let [WidgetViewItem::Element(first), WidgetViewItem::Element(second)] = tree.as_slice() else {
        panic!("{tree:?}");
    };
    assert_eq!(first.children, [WidgetViewItem::Text("2".into())]);
    assert_eq!(second.children, [WidgetViewItem::Text("8".into())]);
    let invalid = page.replace("key: \"desktop\"", "key: \"value\"");
    let errors = compile(&invalid, vec![]).expect_err("the second list has no value field");
    assert!(errors.iter().any(|diag| diag.code == "source.undefined"), "{errors:?}");
}

#[test]
fn p107_sparse_widget_rows_keep_optional_text_and_missing_numeric_keys() {
    use ace_av::age_package::{evaluate_package_view, WidgetViewItem};
    let page = r#"widget Gallery {
        use { package: official from "../components" }
        model {
            rows = [
                { id: "first", label: "yes" },
                { id: "second" }
            ]
        }
        view {
            Diagram (rows: .rows) {}
        }
    }"#;
    let diagram = r#"widget Diagram (rows: List) {
        msg { Init }
        model {
            title str = ""
            copies List = []
        }
        on {
            .Init -> {
                var marks List = []
                var copies List = []
                for row in .rows {
                    marks.push(row["mark"])
                    copies.push({ id: row["id"] })
                }
                .copies = copies
                var row = .rows[1]
                var saved = .copies[1]
                var mark = marks[1]
                if mark != "none" && mark == 0 {
                    var id = saved.id + 4
                    var label = row["label"]
                    .title = f"${id}:${label}:${mark}"
                }
            }
        }
        view { text .title {} }
    }"#;
    let compile = |page: &str| compile_age_package_with_scenario(
        &[
            input("gallery", page),
            PackageModuleInput { package: "official".into(), ..input("diagram", diagram) },
        ],
        "app::gallery",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::ForNode {
                node: 1,
                op: Box::new(ScenarioOp::Send { instance: 0, msg: "Init".into() }),
            }],
        },
    );
    let package = compile(page).expect("a sparse anonymous widget input");
    let tree = evaluate_package_view(&package, 0, &Default::default()).unwrap();
    let [WidgetViewItem::Element(diagram)] = tree.as_slice() else {
        panic!("{tree:?}");
    };
    assert_eq!(diagram.children, [WidgetViewItem::Text("second4::0".into())]);
    let dense = page.replace("{ id: \"second\" }", "{ id: \"second\", label: \"no\" }");
    let errors = compile(&dense).expect_err("an unknown key on a closed record is rejected");
    assert!(errors.iter().any(|diagnostic| diagnostic.code == "source.undefined"), "{errors:?}");
}

#[test]
fn p107_legacy_chart_coordinates_widen_integer_offsets_to_float() {
    let source = r#"
widget ChartGeometry {
    msg { Extend }
    model { var x float = 0.0 }
    on {
        .Extend -> {
            var ordinal int = 3
            var offset float = ordinal
            .x = offset + ordinal * 1.5
            if 2 < .x { .x = .x - 1 }
        }
    }
    view { text f"x=${.x}" {} }
}
"#;
    let check = |source| compile_age_package_with_scenario(
        &[input("main", source)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::Send { instance: 1, msg: "Extend".into() }],
        },
    );
    let package = check(source).expect("legacy layout coordinates use checked widening");
    let observed = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [vec!["x=0"], vec!["x=6.5"]]);

    let strict = source.replace("widget ChartGeometry", "node ChartGeometry");
    let errors = check(&strict).expect_err("ordinary nodes retain exact numeric operand types");
    assert!(errors.iter().any(|error| error.code == "source.type-mismatch"), "{errors:?}");
    let helper = format!("fn outside(n int) float {{ return n * 1.5 }}\n{source}");
    let errors = check(&helper).expect_err("ordinary module functions retain exact numeric operands");
    assert!(errors.iter().any(|error| error.code == "source.type-mismatch"), "{errors:?}");
}

#[test]
fn p107_legacy_chart_pointer_indices_truncate_and_saturate() {
    let source = r#"
widget ChartIndex {
    msg { Update }
    model { var indices str = "" }
    on {
        .Update -> {
            var observed int = (3.75).to_int()
            var lower int = (-3.75).to_int()
            var overflow int = (2147483648.0).to_int()
            var underflow int = (-4294967296.0).to_int()
            var limit int = (2147483520.0).to_int()
            var zero float = 0.0
            var notANumber float = zero % zero
            var fallback int = notANumber.to_int()
            .indices = f"${observed}/${lower}/${overflow}/${underflow}/${limit}/${fallback}"
        }
    }
    view { text .indices {} }
}
"#;
    let check = |source| {
        compile_age_package_with_scenario(
            &[input("main", source)],
            "app::main",
            LegacyWidgetAlias::Accept,
            &caps(),
            &ComponentScenario {
                instances: 1,
                ops: vec![ScenarioOp::Send { instance: 0, msg: "Update".into() }],
            },
        )
    };
    let package = check(source).expect("legacy coordinate conversion uses checked numeric casts");
    let result = ace_av::age_package::observe_age_package(&package, 1).unwrap();
    assert_eq!(
        result.components[0].instance_texts,
        [vec!["3/-3/2147483647/-2147483648/2147483520/0"]]
    );
    let strict = source.replace("widget ChartIndex", "node ChartIndex");
    let errors = check(&strict).expect_err("ordinary nodes do not have the compatibility conversion");
    assert!(errors.iter().any(|error| error.code == "source.undefined"), "{errors:?}");
}

#[test]
fn p107_legacy_chart_label_reads_snapshot_the_list_text() {
    let source = r#"
widget ChartLabels {
    msg { Refresh }
    model {
        var labels List<str> = ["before", "after"]
        var saved List = []
        var shown str = "none"
    }
    on {
        .Refresh -> {
            var selected List = []
            var label = .labels[0]
            var first str = label
            label = .labels[1]
            selected.push(label)
            selected.push(f"x=${label}")
            .saved = selected
            .shown = f"${.saved[0]}/${.saved[1]}/${first}/${.labels[0]}"
        }
    }
    view { text .shown {} }
}
"#;
    let check = |source| {
        compile_age_package_with_scenario(
            &[input("main", source)],
            "app::main",
            LegacyWidgetAlias::Accept,
            &caps(),
            &ComponentScenario {
                instances: 2,
                ops: vec![ScenarioOp::Send {
                    instance: 1,
                    msg: "Refresh".into(),
                }],
            },
        )
    };
    let package = check(source).expect("a legacy chart copies the selected label");
    let observed = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(
        observed.components[0].instance_texts,
        [vec!["none"], vec!["after/x=after/before/before"]]
    );
    let strict = source
        .replace("widget ChartLabels", "node ChartLabels")
        .replace("var saved List = []", "var saved List<str> = []")
        .replace("var selected List = []", "var selected List<str> = []");
    let errors = check(&strict).expect_err("ordinary nodes do not materialize element views");
    assert!(errors.iter().any(|error| error.code == "source.view-escape"), "{errors:?}");
}

#[test]
fn p107_chart_float_interpolation_keeps_the_f32_decimal() {
    let source = r#"
widget ChartGeometry {
    msg { Reset }
    model { var x float = 1.2 }
    on { .Reset -> { .x = -0.0 } }
    view { text f"x=${.x}" {} }
}
"#;
    let package = compile_age_package_with_scenario(
        &[input("main", source)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::Send { instance: 1, msg: "Reset".into() }],
        },
    )
    .expect("AGE interpolates its checked float coordinates");
    let observed = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [vec!["x=1.2"], vec!["x=-0"]]);
}

#[test]
fn p107_graph_stack_pop_discards_the_top_and_accepts_an_empty_stack() {
    let source = r#"
widget GraphWalk {
    msg { Walk }
    model { var result int = 0 }
    on {
        .Walk -> {
            var stack List = []
            stack.pop()
            stack.push(7)
            stack.push(9)
            var take bool = true
            if take { stack.pop() }
            .result = stack[0] + stack.len()
            stack.pop()
            stack.pop()
            .result += stack.len()
        }
    }
    view { text f"${.result}" {} }
}
"#;
    let package = compile_age_package_with_scenario(
        &[input("main", source)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::Send { instance: 1, msg: "Walk".into() }],
        },
    )
    .expect("a layout can discard the last item of a checked list");
    let observed = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [vec!["0"], vec!["8"]]);
}

#[test]
fn p107_annotated_media_handlers_use_the_declared_payload_slots() {
    use ace_core::{
        atom_text::Span,
        source::ast::{Expr, ExprKind},
    };

    let source = r#"
widget ImagePanel {
    msg { ApplyView(str, int) }
    model {
        label str = ""
        zoom int = 1
    }
    on {
        .ApplyView(value str, level i32) -> {
            .label = value
            .zoom = level
        }
    }
    view { text f"${.label}/${.zoom}" {} }
}
"#;
    let span = Span { start: 0, end: 0 };
    let check = |source| compile_age_package_with_scenario(
        &[input("main", source)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::SendArgs {
                instance: 1,
                msg: "ApplyView".into(),
                args: vec![
                    Expr { span, kind: ExprKind::StrLit("图像".into()) },
                    Expr { span, kind: ExprKind::IntLit(2) },
                ],
            }],
        },
    );
    let package = check(source).expect("the annotations agree with the declared payload");
    let observed = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [vec!["/1"], vec!["图像/2"]]);

    let mismatched = source.replace("level i32", "level bool");
    let errors = check(&mismatched).expect_err("an annotation cannot replace the payload type");
    assert_eq!(errors[0].code, "source.age.handler-parameter-type");
    assert_eq!(errors[0].span.slice(&mismatched), "bool");
}

#[test]
fn p107_graph_edges_keep_the_to_field_in_model_rows() {
    let source = r#"
widget Diagram {
    msg { Count }
    model {
        var count int = 0
        edges = [
            { from: "intro", to: "body" },
            { from: "body", to: "end" }
        ]
    }
    on {
        .Count -> {
            for edge in .edges {
                if edge["to"] == "end" { .count += 1 }
            }
        }
    }
    view { text f"${.count}" {} }
}
"#;
    let package = compile_age_package_with_scenario(
        &[input("main", source)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::Send { instance: 1, msg: "Count".into() }],
        },
    )
    .expect("`to` is a named graph edge key");
    let observed = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [vec!["0"], vec!["1"]]);
}

#[test]
fn p107_chart_empty_model_list_uses_the_handler_record_shape() {
    let source = r#"
widget Chart {
    msg { Build }
    model { bands List = [] }
    on {
        .Build -> {
            var rows List = []
            let ordinal = 2
            rows.push({ i: ordinal, s: f"point ${ordinal}" })
            .bands = rows
        }
    }
    view {
        text f"${.bands.len()}" {}
        for band in .bands {
            row (key: band["i"], value: band["s"].len()) {}
        }
    }
}
"#;
    let check = |source| compile_age_package_with_scenario(
        &[input("main", source)], "app::main", LegacyWidgetAlias::Accept, &caps(),
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::Send { instance: 1, msg: "Build".into() }],
        },
    );
    let package = check(source).expect("the builder determines the record element type");
    assert!(package.description.components[0].fields[0].1.starts_with("List<__age_p_hidden_record_"));
    let observed = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [vec!["0"], vec!["1"]]);
    let mixed = source.replace(".bands = rows", "rows.push({ i: true, s: \"different\" })\n            .bands = rows");
    assert!(check(&mixed).is_err(), "inconsistent append shapes must fail");
    let bad_key = source.replace("band[\"i\"]", "band[\"missing\"]");
    let diagnostics = check(&bad_key).expect_err("unknown record keys must fail");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "source.undefined"
            && diagnostic.span.slice(&bad_key) == "\"missing\""
    }), "{diagnostics:?}");
}

#[test]
fn p107_gallery_legacy_assignment_supplies_the_message_payload_type() {
    use ace_core::{atom_text::Span, source::ast::{Expr, ExprKind}};

    let source = r#"widget Gallery {
    model {
        accent_color str = "indigo"
        dark_mode bool = true
    }
    on { .SetAccent(name) -> { accent_color = name } }
    view { text f"${.accent_color}" {} }
}
"#;
    let compile = |source, ops| compile_age_package_with_scenario(
        &[input("main", source)], "app::main", LegacyWidgetAlias::Accept, &caps(),
        &ComponentScenario { instances: 2, ops },
    );
    let package = compile(source, vec![ScenarioOp::SendArgs {
        instance: 1,
        msg: "SetAccent".into(),
        args: vec![Expr { span: Span { start: 0, end: 0 }, kind: ExprKind::StrLit("ocean".into()) }],
    }]).expect("the direct assignment determines the string payload");
    let observed = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [vec!["indigo"], vec!["ocean"]]);

    let ambiguous = source.replace("accent_color = name }", "accent_color = name\n        dark_mode = name }");
    let errors = compile(&ambiguous, Vec::new()).expect_err("conflicting model fields cannot pin a payload");
    assert_eq!(errors[0].code, "source.age.msg-unknown");
}

#[test]
fn p107_chart_numeric_builders_infer_list_types_in_the_handler() {
    let source = r#"widget Chart {
    msg { Build }
    model {
        steps List = []
        var rendered int = 0
    }
    on {
        .Build -> {
            var heights List = []
            heights.push(1.0)
            heights.push(3.0)
            var deltas List = []
            deltas.push(heights[1] - heights[0])
            if deltas[0] == 2.0 { .rendered = 1 }
            .steps = deltas
        }
    }
    view { text f"${.rendered}/${.steps.len()}" {} }
}
"#;
    let compile = |source| compile_age_package_with_scenario(
        &[input("main", source)], "app::main", LegacyWidgetAlias::Accept, &caps(),
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::Send { instance: 1, msg: "Build".into() }],
        },
    );
    let package = compile(source).expect("numeric append and index sites agree on their type");
    let actual = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(actual.components[0].instance_texts, [vec!["0/0"], vec!["1/1"]]);

    let incompatible = source.replace("heights.push(3.0)", "heights.push(true)");
    assert!(compile(&incompatible).is_err(), "heterogeneous append sites must fail checking");
}

#[test]
fn p107_chart_legacy_pointer_exit_only_updates_the_addressed_instance() {
    let source = r#"
widget Chart {
    msg { Reset }
    model { var opacity int = 100 }
    on {
        .Reset -> { .opacity = 100 }
        .HoverOut -> { .opacity = 40 }
    }
    view {
        mouse-area (onmouseleave: .HoverOut) {}
        text f"${.opacity}" {}
    }
}
"#;
    let package = compile_age_package_with_scenario(
        &[input("main", source)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::Send { instance: 1, msg: "HoverOut".into() }],
        },
    )
    .expect("legacy pointer exit has a checked unit message");
    let state = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(state.components[0].instance_states, [vec![100], vec![40]]);
    assert_eq!(state.components[0].instance_texts, [vec!["100"], vec!["40"]]);
}

#[test]
fn p107_chart_tick_updates_checked_mutable_locals() {
    let source = r#"
widget Chart {
    msg { Tick }
    model {
        var tickN int = 0
        var first bool = true
        var value int = 0
    }
    view { text "chart" {} }
    on {
        .Tick -> {
            .tickN = .tickN + 1
            var nd = 150 + (.tickN * 37) % 120
            var first = true
            if first {
                first = false
                var nd = 999
                nd = 997
            }
            if .first && !first {
                .value = nd
            }
        }
    }
}
"#;
    let compiled = |source: &str, ops: Vec<ScenarioOp>| {
        compile_age_package_with_scenario(
            &[input("main", source)], "app::main", LegacyWidgetAlias::Accept, &caps(),
            &ComponentScenario { instances: 2, ops },
        )
    };
    let tick = |instance| ScenarioOp::Send { instance, msg: "Tick".into() };
    let package = compiled(source, vec![tick(0), tick(1), tick(0)])
        .expect("the chart arithmetic and nested local writes verify");
    let assigned: Vec<_> = package.description.components[0].handler_assigns[0].1
        .iter().map(|(field, _)| field.as_str()).collect();
    assert_eq!(assigned, ["tickN", "value"]);
    let state = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(state.components[0].instance_states, [vec![2, 224], vec![1, 187]]);

    let immutable = source.replace("var first = true", "let first = true");
    let errors = compiled(&immutable, vec![]).expect_err("writing a let must fail without dispatch");
    assert!(errors.iter().any(|d| d.code.contains("immutable")), "{errors:?}");
}

#[test]
fn p107_handler_for_copies_borrowed_rows_and_preserves_each_seed() {
    let source = r#"
widget Chart {
    msg { Reset }
    model {
        interval int = 400
        seed = [
            { label: "Jan", value: 186 },
            { label: "Feb", value: 305 }
        ]
        monthly = [
            { value: 186, label: "Jan" },
            { value: 305, label: "Feb" }
        ]
    }
    view { text f"${.seed.len()} seed / ${.monthly.len()} current" {} }
    on {
        .Tick -> {
            var out = []
            var first = true
            for row in .monthly {
                if first {
                    first = false
                } else {
                    out.push(row)
                }
            }
            .monthly = out
        }
        .Reset -> {
            var out = []
            for row in .seed {
                out.push(row)
            }
            out.push({ label: "Next", value: 237 })
            .monthly = out
        }
    }
}
"#;
    let compile = |source, ops| {
        compile_age_package_with_scenario(
            &[input("main", source)],
            "app::main",
            LegacyWidgetAlias::Accept,
            &caps(),
            &ComponentScenario { instances: 2, ops },
        )
    };
    let event = |instance, msg: &str| ScenarioOp::Send { instance, msg: msg.into() };
    let package = compile(source, vec![event(0, "Tick"), event(1, "Tick"), event(1, "Reset")])
        .unwrap_or_else(|ds| panic!("checked handler loops: {ds:?}"));
    let description = &package.description.components[0];
    assert_eq!(description.fields[1].1, description.fields[2].1);
    assert!(description.members.iter().any(|row| row.role == "timer"
        && row.name == "Tick" && row.num == Some(400)));
    let observed = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [
        vec!["2 seed / 1 current"], vec!["2 seed / 3 current"],
    ]);

    let invalid = source.replace("out.push(row)", ".monthly.push(row)");
    let failure = compile(&invalid, Vec::new()).expect_err("cannot extend the list being iterated");
    assert!(failure.iter().any(|d| d.code == "source.iterate-mutate"), "{failure:?}");

    let invalid = source.replace("value: 237", "value: false");
    let failure = compile(&invalid, Vec::new()).expect_err("the append must match the record fields");
    assert!(failure.iter().any(|d| d.code == "source.type-mismatch"
        && d.span.slice(&invalid) == "value: false"), "{failure:?}");

    let invalid = source.replace("interval int = 400", "interval int = 0");
    let failure = compile(&invalid, Vec::new()).expect_err("implicit Tick needs a positive interval");
    assert!(failure.iter().any(|d| d.code == "source.age.msg-unknown"), "{failure:?}");
}

#[test]
fn p107_chart_handler_while_checks_timer_guard_and_indexed_updates() {
    let source = r#"widget Chart {
    msg { Sample }
    model {
        values List<int> = [7, 11, 13]
        active bool = true
        total int = 0
    }
    timer {
        Sample (every_ms: 33, when: .active)
    }
    view { text f"${.total}" {} }
    on {
        .Sample -> {
            var ms List<int> = []
            for d in .values { ms.push(d) }
            var mi = 1
            while mi < ms.len() {
                ms[mi] = ms[mi] + ms[mi - 1]
                mi = mi + 1
            }
            .total = ms[2]
        }
    }
}
"#;
    let compile = |source: &str, ops| compile_age_package_with_scenario(
        &[input("main", source)], "app::main", LegacyWidgetAlias::Accept, &caps(),
        &ComponentScenario { instances: 2, ops },
    );
    let package = compile(source, vec![ScenarioOp::Send { instance: 1, msg: "Sample".into() }])
        .unwrap_or_else(|ds| panic!("checked chart loop: {ds:?}"));
    let timer = &package.description.components[0].members[0];
    assert_eq!((timer.name.as_str(), timer.num), ("Sample", Some(33)));
    assert_eq!(timer.value.as_ref().unwrap().span.slice(source), ".active");
    let observed = ace_av::age_package::observe_age_package(&package, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [vec!["0"], vec!["31"]]);

    for invalid in [source.replace("when: .active", "when: .total"),
        source.replace("ms[mi] + ms[mi - 1]", "true")] {
        let errors = compile(&invalid, Vec::new()).expect_err("invalid guard or indexed assignment");
        assert!(errors.iter().any(|diag| diag.code == "source.type-mismatch"), "{errors:?}");
    }
}

#[test]
fn p107_package_chart_math_uses_the_checked_stdlib() {
    let src = r#"
node Chart {
    view { text geometry() {} }
}
fn geometry() str {
    let s float = math.sin(4.71238898)
    let c float = math.cos(3.14159265)
    let a float = math.atan2(-1.0, -1.0)
    let b float = math.atan2(0.0, -1.0)
    let radius float = math.sqrt(3.0 * 3.0 + 4.0 * 4.0)
    if s > -0.999998 || c > -0.999998 { return "arc" }
    if a < -2.356197 || a > -2.356192 { return "quadrant" }
    if b < 3.141590 || b > 3.141595 { return "axis" }
    if radius < 4.999998 || radius > 5.000002 { return "radius" }
    return "geometry"
}
"#;
    let scenario = ComponentScenario { instances: 1, ops: vec![] };
    let compiled = compile_age_package_with_scenario(
        &[input("main", src)], "app::main", LegacyWidgetAlias::Reject, &caps(), &scenario,
    )
    .expect("the chart math namespace is bound through the AGE package");
    assert!(compiled.description.modules.iter().any(|module| module.key == "std::age_math"));
    let observed = ace_av::age_package::observe_age_package(&compiled, 1)
        .expect("the AV chart geometry executes");
    assert_eq!(observed.components[0].instance_texts, vec![vec!["geometry"]]);

    let shadowed = r#"
node Chart {
    view { text geometry() {} }
}
fn geometry() str {
    if math.sin(0.0) == 7.0 { return "local math" }
    return "wrong math"
}
"#;
    let local_math = "pub fn sin(x float) float { return 7.0 }";
    let compiled = compile_age_package_with_scenario(
        &[input("main", shadowed), input("math", local_math)],
        "app::main", LegacyWidgetAlias::Reject, &caps(), &scenario,
    ).expect("a local math module takes precedence");
    assert!(!compiled.description.modules.iter().any(|module| module.key == "std::age_math"));
    let observed = ace_av::age_package::observe_age_package(&compiled, 1).unwrap();
    assert_eq!(observed.components[0].instance_texts, vec![vec!["local math"]]);
}

#[test]
fn p107_age_package_float_helper_tail_executes_for_each_instance() {
    let main = r#"
use geometry.scale
use geometry.label

node Chart {
    msg { Move(float) }
    model { extent f32 = 0.0 }
    view { text label(.extent) {} }
    on { .Move(x) -> { .extent = scale(x) } }
}
"#;
    let geometry = r#"
pub fn scale(x float) float {
    let factor float = 0.5
    x * factor
}

pub fn label(x f32) str {
    if x > 5.0 {
        return "wide"
    }
    "narrow"
}
"#;
    use ace_core::source::age_component::InitialValue;
    let start = main.find("Move(float)").expect("message declaration");
    let span = ace_core::atom_text::Span { start, end: start + "Move(float)".len() };
    let scenario = ComponentScenario {
        instances: 2,
        ops: [11.0, 9.5]
            .into_iter()
            .enumerate()
            .map(|(instance, extent)| {
                ScenarioOp::send_args_from_values(instance, "Move", &[InitialValue::Float(extent)], span)
            })
            .collect(),
    };
    let compiled = compile_age_package_with_scenario(
        &[input("main", main), input("geometry", geometry)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
        &scenario,
    )
    .expect("the package verifies the imported float helper and both tail results");
    let observed = ace_av::age_package::observe_age_package(&compiled, 2)
        .expect("the shared AV chain observes both instances");
    assert_eq!(
        observed.components[0].instance_texts,
        vec![vec!["wide"], vec!["narrow"]]
    );
}

/// 两实例 2:1 事件 + 各 4 次 Tick：
///   实例 0：2 次 Inc + 4 次 Tick = 6  → 文本 "six"
///   实例 1：1 次 Inc + 4 次 Tick = 5  → 文本 "other"
fn mixed_scenario() -> ComponentScenario {
    let mut ops = Vec::new();
    // 定向事件按 2:1 交错投递（顺序本身也必须不影响各实例自己的计数）。
    ops.push(ScenarioOp::Send {
        instance: 0,
        msg: "Inc".into(),
    });
    ops.push(ScenarioOp::Send {
        instance: 1,
        msg: "Inc".into(),
    });
    ops.push(ScenarioOp::Send {
        instance: 0,
        msg: "Inc".into(),
    });
    for _ in 0..4 {
        ops.push(ScenarioOp::Tick { instance: 0 });
        ops.push(ScenarioOp::Tick { instance: 1 });
    }
    ComponentScenario { instances: 2, ops }
}

#[test]
fn p114_tree_ids_toggle_with_the_checked_helper_loop() {
    use ace_core::source::age_component::InitialValue;

    let helper = r#"pub fn toggle_id(list List<str>, id str) List<str> {
    var out List<str> = []
    var found bool = false
    var i int = 0
    while i < list.len() {
        if list[i] == id {
            found = true
        } else {
            out.push(list[i])
        }
        i = i + 1
    }
    if !found {
        out.push(id)
    }
    return out
}
"#;
    let page = r#"use ids.toggle_id
widget TreePage {
    msg { Toggle(str) }
    model {
        expanded List = ["docs", "src", "docs"]
        selected str = ""
    }
    on {
        .Toggle(id) -> {
            .expanded = toggle_id(.expanded, id)
            .selected = .expanded.get(.expanded.len() - 1).clone()
        }
    }
    view {
        text f"${.expanded.len()}:${.selected}" {}
    }
}
"#;
    let anchor = ace_core::atom_text::Span { start: 0, end: page.len() };
    let scenario = ComponentScenario {
        instances: 2,
        ops: ["docs", "docs/计划"]
            .into_iter()
            .enumerate()
            .map(|(instance, id)| ScenarioOp::send_args_from_values(
                instance, "Toggle", &[InitialValue::Str(id.into())], anchor,
            ))
            .collect(),
    };
    let compiled = compile_age_package_with_scenario(
        &[input("ids", helper), input("tree", page)],
        "app::tree",
        LegacyWidgetAlias::Accept,
        &caps(),
        &scenario,
    ).expect("check the authored traversal and cross-module helper call");
    let observed = ace_av::age_package::observe_age_package(&compiled, 2)
        .expect("execute the checked toggle");
    assert_eq!(observed.components[0].instance_texts, [vec!["1:src"], vec!["4:docs/计划"]]);
    use ace_av::age_component::ComponentFieldValue::Str;
    assert_eq!(observed.components[0].instance_scalar_states, [
        vec![("selected".into(), Str("src".into()))],
        vec![("selected".into(), Str("docs/计划".into()))],
    ]);
}

#[test]
fn p114_directory_listing_payload_updates_only_the_addressed_instance() {
    use ace_core::atom_text::Span;
    use ace_core::source::ast::{Expr, ExprKind, Ident, NamedCtorField, TypeAnn, TypeRef};

    let page = r#"type DirEntry {
    name str
    path str
    is_dir bool
    size int
    mtime int
}
type DirListing {
    ok bool
    path str
    entries List<DirEntry>
}
widget Browser {
    msg { DirectoryListed(DirListing), ApplyListing(DirListing) }
    model {
        var current_path str = "home"
        var first_name str = "old.txt"
        var entry_count int = 1
    }
    on {
        .DirectoryListed(listing DirListing) -> { .ApplyListing(listing.move) }
        .ApplyListing(listing DirListing) -> {
            if listing.ok {
                .current_path = listing.path
                .entry_count = listing.entries.len()
                .first_name = ""
                if .entry_count > 0 {
                    .first_name = listing.entries[0].name
                }
            }
        }
    }
    view { text f"${.current_path}:${.first_name}:${.entry_count}" {} }
}
"#;
    let span = Span { start: 0, end: 0 };
    let value = |kind| Expr { kind, span };
    let string = |text: &str| value(ExprKind::StrLit(text.to_owned()));
    let name = |text: &str| Ident { name: text.into(), span };
    let record = |ty, entries: Vec<(&str, Expr)>| value(ExprKind::CtorNamed {
        ty: name(ty),
        args: Vec::new(),
        fields: entries.into_iter().map(|(field, value)| NamedCtorField {
            name: name(field), value, span,
        }).collect(),
    });
    let entry = record("DirEntry", vec![
        ("name", string("计划.txt")), ("path", string("docs/计划.txt")),
        ("is_dir", value(ExprKind::BoolLit(false))), ("size", value(ExprKind::IntLit(17))),
        ("mtime", value(ExprKind::IntLit(1700000000))),
    ]);
    let listing = |ok, path, entries: Vec<Expr>| {
        let entries = if entries.is_empty() {
            value(ExprKind::MethodCall {
                recv: Box::new(value(ExprKind::GenericType {
                    name: name("List"),
                    args: vec![TypeAnn { kind: TypeRef::Record(name("DirEntry")), span }],
                })),
                name: name("new"),
                args: Vec::new(),
                instance: None,
            })
        } else {
            value(ExprKind::ArrayLit(entries))
        };
        record("DirListing", vec![
            ("ok", value(ExprKind::BoolLit(ok))), ("path", string(path)), ("entries", entries),
        ])
    };
    let result = |instance, payload| ScenarioOp::SendArgs {
        instance, msg: "DirectoryListed".into(), args: vec![payload],
    };
    let modules = [input("app", page)];
    let compile = |ops| compile_age_package_with_scenario(
        &modules, "app::app", LegacyWidgetAlias::Accept, &caps(),
        &ComponentScenario { instances: 2, ops },
    );
    let compiled = compile(vec![
        result(0, listing(true, "docs", vec![entry])),
        result(0, listing(false, "outside", vec![])),
    ]).expect("check the typed directory reply and run it through the scenario");
    let observed = ace_av::age_package::observe_age_package(&compiled, 2).unwrap();
    assert_eq!(observed.components[0].instance_texts, [vec!["docs:计划.txt:1"], vec!["home:old.txt:1"]]);
    let errors = compile(vec![result(0, string("outside"))]).err().unwrap();
    assert!(errors.iter().any(|error| error.code == "source.type-mismatch"), "{errors:?}");

    let aliasing = page.replace(".ApplyListing(listing.move)", ".ApplyListing(listing)");
    let errors = compile_age_package_with_scenario(
        &[input("app", &aliasing)], "app::app", LegacyWidgetAlias::Accept, &caps(),
        &ComponentScenario { instances: 1, ops: Vec::new() },
    ).err().unwrap();
    assert!(errors.iter().any(|error| error.code == "source.age.handler-record-transfer"), "{errors:?}");
}

#[test]
fn p114_package_model_carries_typed_node_rows_to_the_helper_and_event_scope() {
    use ace_core::source::age_component::InitialValue;

    let helpers = r#"pub type TreeNode {
    id str
    parent int
}
pub fn count_children(nodes List<TreeNode>, parent int) int {
    var i int = 0
    var count int = 0
    while i < nodes.len() {
        if nodes[i].parent == parent { count = count + 1 }
        i = i + 1
    }
    return count
}
"#;
    let page = r#"use nodes: TreeNode, count_children
widget TreePage {
    msg { Select(str) }
    model {
        rows List<TreeNode> = [
            TreeNode { id: "docs", parent: -1 },
            TreeNode { id: "docs/计划", parent: 0 },
            TreeNode { id: "docs/archive", parent: 0 },
        ]
        selected str = ""
    }
    on {
        .Select(id) -> { .selected = id }
    }
    computed {
        visible => .rows
        total => count_children(.visible, 0)
        choice => .selected
    }
    view {
        text f"children: ${.total}; selected: ${.choice}" {}
        for row in .visible {
            button "Select" { onclick: .Select(row.id) }
        }
    }
}
"#;
    let compiled = compile_age_package_with_scenario(
        &[input("nodes", helpers), input("tree", page)],
        "app::tree",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 2,
            ops: vec![ScenarioOp::send_args_from_values(
                1,
                "Select",
                &[InitialValue::Str("docs/计划".into())],
                ace_core::atom_text::Span { start: 0, end: page.len() },
            )],
        },
    ).expect("typed row model, imported helper and projected row payload all check");
    let component = &compiled.description.components[0];
    assert_eq!(component.view_events[0].2.slice(page), "onclick: .Select(row.id)");
    let observed = ace_av::age_package::observe_age_package(&compiled, 2)
        .expect("AV reads the typed node rows through the helper");
    assert_eq!(observed.components[0].instance_texts, [
        vec!["children: 2; selected: "],
        vec!["children: 2; selected: docs/计划"],
    ]);
}

#[test]
fn p114_widget_properties_check_and_evaluate_the_parent_and_default_values() {
    use ace_av_core::contract::Value;
    use ace_core::source::age_component::InitialValue;
    use ace_core::source::age_package::AgeWidgetProperty;

    let helper = r#"pub type TreeNode {
    id str
    children List<TreeNode>
}
"#;
    let widget = r#"use nodes: TreeNode
widget FileTree (nodes: List<TreeNode>, expanded: List<str> = ["docs"], on_toggle: msg) {
    msg { Init, Pick(str, int, bool) }
    model { opened List<str> = [] }
    on { .Init -> { .opened = .expanded } }
    computed {
        open => .opened
        summary => f"${.nodes[0].id}:${.open.len()}"
    }
    view {
        for position, node in .nodes {
            if position == 0 {
                text "First" {}
            }
            row (key: node.id, onclick: .Pick(node.id, position, position == 0)) {
                text node.id {}
            }
        }
    }
}
"#;
    let page = r#"use components.nodes: TreeNode
widget Gallery {
    use { package: official from "../components" }
    msg { Toggle(str) }
    model {
        nodes List<TreeNode> = [{ id: "docs/计划", children: [] }, { id: "src/项目", children: [] }]
        expanded List<str> = ["ordinary"]
    }
    on { .Toggle(id) -> { .expanded = [id.clone()] } }
    view {
        FileTree (nodes: .nodes, on_toggle: .Toggle) {}
        FileTree (nodes: .nodes, expanded: ["src", "docs/计划"], on_toggle: .Toggle) {}
        FileTree (nodes: .nodes, expanded: .expanded, on_toggle: .Toggle) {}
        for position, row in .nodes {
            if row.id != "" {
                row (key: row.id) {
                    text row.id {}
                    TreeIcon (name: row.id, position: position) {}
                }
            }
        }
    }
}
"#;
    let compile = |source: &str| {
        let official = |id: &str, src: &str| PackageModuleInput {
            package: "official".into(),
            ..input(id, src)
        };
        compile_age_package_with_scenario(
            &[
                input("gallery", source), official("nodes", helper), official("tree", widget),
                official("icon", r#"widget TreeIcon (name: str, position: int) { view { text f"${.name}:${.position}" {} } }"#),
            ],
            "app::gallery",
            LegacyWidgetAlias::Accept,
            &caps(),
            &ComponentScenario {
                instances: 2,
                ops: vec![
                    ScenarioOp::ForNode {
                        node: 0,
                        op: Box::new(ScenarioOp::send_args_from_values(
                            1, "Toggle", &[InitialValue::Str("changed/计划".into())],
                            ace_core::atom_text::Span { start: 0, end: 3 },
                        )),
                    },
                    ScenarioOp::ForNode {
                        node: 1,
                        op: Box::new(ScenarioOp::Send { instance: 0, msg: "Init".into() }),
                    },
                    ScenarioOp::ForNode {
                        node: 1,
                        op: Box::new(ScenarioOp::Send { instance: 1, msg: "Init".into() }),
                    },
                ],
            },
        )
    };
    let compiled = compile(page).unwrap_or_else(|ds| panic!("typed child arguments: {ds:?}"));
    let bindings = &compiled.widget_refs[&("app::gallery".into(), "Gallery".into())];
    assert_eq!(bindings.len(), 4);
    let run = |binding: &AgeWidgetProperty, instance| {
        let value = ace_av::age_package::evaluate_widget_property(&compiled, binding, instance).unwrap();
        let Value::List(value) = value else {
            panic!("{value:?}");
        };
        let items = value.rc.borrow().elems.clone();
        items
    };
    let text = |item: &Value| {
        let Value::Str(value) = item else { panic!("{item:?}") };
        String::from_utf8(value.bytes().to_vec()).unwrap()
    };
    let nodes = run(&bindings[0].properties[0], 0);
    let [Value::Record(root), Value::Record(second)] = nodes.as_slice() else { panic!("{nodes:?}") };
    assert_eq!(text(&root.fields[0]), "docs/计划");
    assert_eq!(text(&second.fields[0]), "src/项目");
    assert_eq!(run(&bindings[0].properties[1], 1).iter().map(text).collect::<Vec<_>>(), ["docs"]);
    assert_eq!(run(&bindings[1].properties[1], 0).iter().map(text).collect::<Vec<_>>(), ["src", "docs/计划"]);
    assert_eq!(run(&bindings[2].properties[1], 0).iter().map(text).collect::<Vec<_>>(), ["ordinary"]);
    assert_eq!(run(&bindings[2].properties[1], 1).iter().map(text).collect::<Vec<_>>(), ["changed/计划"]);
    let computed = |site: usize, instance: usize, name: &str| {
        ace_av::age_package::evaluate_widget_computed(
            &compiled, &bindings[site], instance, name, &Default::default(),
        ).unwrap()
    };
    assert_eq!(text(&computed(0, 0, "summary")), "docs/计划:1");
    assert_eq!(text(&computed(1, 0, "summary")), "docs/计划:2");
    let Value::List(first) = computed(2, 0, "open") else { panic!("expected a list") };
    let Value::List(second) = computed(2, 1, "open") else { panic!("expected a list") };
    assert_eq!(text(&first.rc.borrow().elems[0]), "ordinary");
    assert_eq!(text(&second.rc.borrow().elems[0]), "changed/计划");
    let view = ace_av::age_package::evaluate_widget_view(&compiled, &bindings[0], 0, &Default::default())
        .unwrap();
    use ace_av::age_package::WidgetViewItem;
    use ace_av::age_component::ComponentFieldValue;
    let [WidgetViewItem::Text(first), WidgetViewItem::Element(row_a), WidgetViewItem::Element(row_b)] =
        view.as_slice() else { panic!("{view:?}") };
    assert_eq!(first, "First");
    assert_eq!(row_a.site, row_b.site);
    for (row, id, index, first) in [(row_a, "docs/计划", 0, true), (row_b, "src/项目", 1, false)] {
        assert_eq!(row.properties, [("key".into(), id.into())]);
        assert_eq!(row.children, [WidgetViewItem::Text(id.into())]);
        assert_eq!(row.events[0].message, "Pick");
        assert_eq!(row.events[0].args, [
            ComponentFieldValue::Str(id.into()),
            ComponentFieldValue::Int(index),
            ComponentFieldValue::Bool(first),
        ]);
    }
    let complete = ace_av::age_package::evaluate_package_view(&compiled, 0, &Default::default()).unwrap();
    let mut icons = Vec::new();
    for row in &complete {
        if let WidgetViewItem::Element(row) = row {
            icons.extend(row.children.iter().filter_map(|item| {
                let WidgetViewItem::Element(icon) = item else { return None };
                (icon.tag == "TreeIcon").then(|| icon.children.as_slice())
            }));
        }
    }
    assert_eq!(icons, [
        [WidgetViewItem::Text("docs/计划:0".into())].as_slice(),
        [WidgetViewItem::Text("src/项目:1".into())].as_slice(),
    ]);
    assert!(matches!(&bindings[0].properties[2],
        AgeWidgetProperty::Message { name, target, payload }
            if name == "on_toggle" && target == "Toggle" && payload == &["str"]));
    assert!(matches!(&bindings[3].properties[0],
        AgeWidgetProperty::ScopedValue { name, span, .. } if name == "name"
            && span.slice(page) == "row.id"));
    assert!(matches!(&bindings[3].properties[1],
        AgeWidgetProperty::ScopedValue { name, span, .. } if name == "position"
            && span.slice(page) == "position"));

    for (wrong, code) in [
        (page.replace("nodes: .nodes", "nodes: [\"docs\"]"), "source.type-mismatch"),
        (page.replace("nodes: .nodes, ", ""), "source.age.widget-prop-required"),
        (page.replace("nodes: .nodes,", "nodes: .nodes, nodes: .nodes,"), "source.age.widget-prop-duplicate"),
        (page.replace("on_toggle: .Toggle", "on_toggle: \"Toggle\""), "source.age.widget-prop-message"),
        (page.replace("position: position", "position: row.id"), "source.type-mismatch"),
        (page.replace("key: row.id", "key: absent.id"), "source.undefined"),
        (page.replace("if row.id != \"\"", "if (position)"), "source.type-mismatch"),
    ] {
        let errors = compile(&wrong).err()
            .unwrap_or_else(|| panic!("invalid widget binding was accepted, expected {code}"));
        assert!(errors.iter().any(|d| d.code == code), "expected {code}: {errors:?}");
    }
}

#[test]
fn p114_package_walks_shared_child_lists_and_uses_checked_text_predicates() {
    let helpers = r#"pub type TreeNode {
    id str
    children List<TreeNode>
}
fn has_name(nodes List<TreeNode>, id str) bool {
    return nodes[0].id == id
}
pub fn label(nodes List<TreeNode>) Str {
    let kids = nodes[0].children
    if has_name(kids, "目录/计划.rs") && has_name(nodes, "目录") && kids.len() == 1 {
        let name = kids[0].id.clone()
        if name.ends_with(".rs") && name.contains("计划") && !name.contains("draft") {
            return name
        }
    }
    return "unexpected"
}
"#;
    let page = r#"use nodes: TreeNode, label
widget TreePage {
    model {
        nodes List<TreeNode> = [{ id: "目录", children: [
            { id: "目录/计划.rs", children: [] },
        ] }]
    }
    view { text label(.nodes) {} }
}
"#;
    let compiled = compile_age_package_with_scenario(
        &[input("nodes", helpers), input("tree", page)],
        "app::tree",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario { instances: 1, ops: Vec::new() },
    ).unwrap_or_else(|ds| panic!("shared checked tree: {ds:?}"));
    let observed = ace_av::age_package::observe_age_package(&compiled, 1).unwrap();
    assert_eq!(observed.components[0].instance_texts, [vec!["目录/计划.rs"]]);
}

#[test]
fn p114_package_model_checks_recursive_record_literal_and_observes_children() {
    let helpers = r#"pub type TreeNode {
    id str
    children List<TreeNode>
}
pub fn sample(nodes List<TreeNode>) Str {
    var root = nodes.remove(0)
    var child = root.children.remove(0)
    return child.children.get(0).id.clone()
}
"#;
    let page = r#"use nodes: TreeNode, sample
widget TreePage {
    model {
        rows List<TreeNode> = [
            { id: "docs", children: [
                { id: "guide", children: [
                    { id: "docs/计划", children: [] },
                ] },
            ] },
        ]
    }
    computed { first => sample(.rows) }
    view { text .first {} }
}
"#;
    let compile = |page| {
        compile_age_package_with_scenario(
            &[input("nodes", helpers), input("tree", page)],
            "app::tree",
            LegacyWidgetAlias::Accept,
            &caps(),
            &ComponentScenario { instances: 2, ops: Vec::new() },
        )
    };
    let compiled = compile(page).unwrap_or_else(|ds| panic!("nested checked model: {ds:?}"));
    let observed = ace_av::age_package::observe_age_package(&compiled, 2)
        .expect("AV executes the recursively typed model");
    assert_eq!(observed.components[0].instance_texts, [
        vec!["docs/计划"],
        vec!["docs/计划"],
    ]);

    let invalid = page.replace("id: \"docs/计划\"", "id: 42");
    let errors = compile(&invalid).err().expect("invalid record field is refused");
    assert!(errors.iter().any(|d| d.code == "source.type-mismatch"
        && d.span.slice(&invalid) == "id: 42"), "{errors:?}");
}

#[test]
fn p114_tree_expanded_lists_replace_and_clear_in_separate_av_instances() {
    let source = r#"widget TreePage {
    msg { Expand, Collapse }
    model {
        expanded List = ["docs", "src"]
        selected str = "docs"
    }
    on {
        .Expand -> {
            .expanded = ["docs", "docs/计划", "src"]
            .selected = .expanded.get(1).clone()
        }
        .Collapse -> {
            .expanded = []
            .selected = ""
        }
    }
    view {
        text f"expanded: ${.expanded.len()} · selected: ${.selected}" {}
    }
}
"#;
    let compile = |ops| {
        compile_age_package_with_scenario(
            &[input("tree", source)],
            "app::tree",
            LegacyWidgetAlias::Accept,
            &caps(),
            &ComponentScenario { instances: 2, ops },
        ).expect("check the package list model")
    };
    let initial = compile(Vec::new());
    let initial = ace_av::age_package::observe_age_package(&initial, 2)
        .expect("observe both initial lists");
    assert_eq!(initial.components[0].instance_texts,
        [vec!["expanded: 2 · selected: docs"], vec!["expanded: 2 · selected: docs"]]);
    let changed = compile(vec![
        ScenarioOp::Send { instance: 0, msg: "Collapse".into() },
        ScenarioOp::Send { instance: 1, msg: "Expand".into() },
    ]);
    let changed = ace_av::age_package::observe_age_package(&changed, 2)
        .expect("observe both changed lists");
    assert_eq!(changed.components[0].instance_texts,
        [vec!["expanded: 0 · selected: "], vec!["expanded: 3 · selected: docs/计划"]]);
}

#[test]
fn p981_age_package_two_instances_two_to_one_events_and_four_ticks() {
    let scenario = mixed_scenario();
    let compiled = compile_age_package_with_scenario(
        &[input("helper", HELPER), input("main", MAIN)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
        &scenario,
    )
    .expect("two-instance AGE package compiles");
    let observed = ace_av::age_package::observe_age_package(&compiled, 2)
        .expect("the shared AV chain runs both instances");

    assert_eq!(observed.components.len(), 1);
    let c = &observed.components[0];
    assert_eq!(c.name, "Counter");

    // 独立算术 oracle：6 / 5（helper 初值 0，实例 0 得 2 Inc、实例 1 得 1 Inc）。
    assert_eq!(
        c.instance_states,
        vec![vec![6], vec![5]],
        "instance 0 = 2 Inc + 4 Tick, instance 1 = 1 Inc + 4 Tick"
    );
    // 同一 node 定义、两个**独立**实例：文本随各自状态取不同分支。
    assert_eq!(
        c.instance_texts,
        vec![vec!["six".to_string()], vec!["other".to_string()]]
    );
    // Tick 恰推进实例自己的计数（不串扰）：各实例 4 次。
    assert_eq!(
        c.handler_msgs,
        vec!["Inc".to_string(), "Life.Tick".to_string()]
    );
}

/// 改被导入 helper 的初值 → 同一场景下两个实例的值都必须变（跨模块真受检）。
#[test]
fn p981_age_package_imported_helper_value_drives_both_instances() {
    let changed = HELPER.replace("return 0 }", "return 10 }");
    let scenario = mixed_scenario();
    let compiled = compile_age_package_with_scenario(
        &[input("helper", &changed), input("main", MAIN)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
        &scenario,
    )
    .expect("changed helper compiles");
    let observed =
        ace_av::age_package::observe_age_package(&compiled, 2).expect("AV observes both");
    assert_eq!(
        observed.components[0].instance_states,
        vec![vec![16], vec![15]]
    );
    // 两个实例都越过了 6，故都走 "other" 分支 —— 文本不是常量补的。
    assert_eq!(
        observed.components[0].instance_texts,
        vec![vec!["other".to_string()], vec!["other".to_string()]]
    );
}

/// 场景自检：未声明的消息与越界实例在编译期具名拒绝（不进入运行）。
#[test]
fn p981_age_package_scenario_failures_are_named() {
    let mut bad_msg = mixed_scenario();
    bad_msg.ops.push(ScenarioOp::Send {
        instance: 0,
        msg: "Nope".into(),
    });
    let ds = compile_age_package_with_scenario(
        &[input("helper", HELPER), input("main", MAIN)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
        &bad_msg,
    )
    .expect_err("an undeclared message is rejected");
    assert!(
        ds.iter().any(|d| d.code == "source.age.component-event"),
        "{ds:?}"
    );
}

/// R64-4：两个 node 同名字段各自可观察（状态函数命名已按节点消歧）。
#[test]
fn p981_dual_nodes_same_field_are_observed_separately() {
    let src = "node A {\n    model { count int = 0 }\n    on Tick { count = count + 1 }\n    view {\n        text \"a\" {}\n    }\n}\nnode B {\n    model { count int = 0 }\n    on Tick { count = count + 1 }\n    view {\n        text \"b\" {}\n    }\n}\n";
    let compiled = compile_age_package_with_scenario(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 2,
            ops: vec![
                ScenarioOp::Tick { instance: 0 },
                ScenarioOp::Tick { instance: 0 },
            ],
        },
    )
    .expect("two nodes sharing a field name compile");
    let observed = ace_av::age_package::observe_age_package(&compiled, 2)
        .expect("observation resolves both components");
    assert_eq!(observed.components.len(), 2);
    // 实例级调度：两个组件都声明了 `on Tick`，实例 0 各 Tick 两次。
    assert_eq!(observed.components[0].instance_states[0][0], 2);
    assert_eq!(observed.components[1].instance_states[0][0], 2);
}

/// B-LANGUAGE-V（AV 腿）：真实 980/A Counter 源在 AV 观察下与 A2R oracle 同值。
#[test]
fn p981_ma_counter_av_leg_matches_the_arithmetic_oracle() {
    const SRC: &str = include_str!("../../../core/tests/fixtures_ma/ace_node_counter.at");
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Dec".into(),
            },
        ],
    };
    let compiled = compile_age_package_with_scenario(
        &[input("main", SRC)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    )
    .expect("the delivered A source compiles");
    let observed = ace_av::age_package::observe_age_package(&compiled, 1)
        .expect("AV observes the real A source");
    assert_eq!(observed.components.len(), 1);
    assert_eq!(observed.components[0].name, "Counter");
    // 2 Inc + 1 Dec = 1：与 A2R 二进制 oracle 同值（独立算术）。
    assert_eq!(observed.components[0].instance_states, vec![vec![1]]);
    assert_eq!(
        observed.components[0].instance_texts,
        vec![vec!["Counter".to_string()]],
        "the real view text is observed"
    );
}

/// V-EVT-A-UNIT：匿名 unit Counter 的**终值 + 单步 oracle**（0→1 与 2→0）；
/// 以及具名 unit 镜面（`msg Msg { Inc }`）同轨迹。
#[test]
fn p981_evt_a_unit_counter_trace_and_named_mirror() {
    const SRC: &str = include_str!("../../../core/tests/fixtures_ma/ace_node_counter.at");
    let ops = || {
        vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Dec".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Reset".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Reset".into(),
            },
        ]
    };
    let compiled = compile_age_package_with_scenario(
        &[input("main", SRC)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: ops(),
        },
    )
    .expect("the anonymous unit counter compiles");
    // 终值 oracle：Inc,Dec,Reset,Reset 后为 0；Reset/Dec 的区分由下面的单步用例承担。
    let observed = ace_av::age_package::observe_age_package(&compiled, 1).expect("observes");
    assert_eq!(observed.components[0].instance_states, vec![vec![0]]);

    // 具名 unit 镜面：同一 business 轨迹，消息集合取名不改变语义。
    let named = "node Counter {\n    msg Msg {\n        Inc,\n        Dec,\n        Reset\n    }\n    model {\n        var count int = 0\n    }\n    on {\n        .Inc -> { .count += 1 }\n        .Dec -> { .count -= 1 }\n        .Reset -> { .count = 0 }\n    }\n    view {\n        text \"Counter\" {}\n    }\n}\n";
    let compiled = compile_age_package_with_scenario(
        &[input("main", named)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: ops(),
        },
    )
    .expect("the named unit mirror compiles");
    assert_eq!(
        compiled.description.components[0]
            .message_set_name
            .as_deref(),
        Some("Msg")
    );
    let observed = ace_av::age_package::observe_age_package(&compiled, 1).expect("observes");
    assert_eq!(observed.components[0].instance_states, vec![vec![0]]);

    // 单步 oracle：Inc 后 1；Inc,Inc,Reset 后 0（Reset 若被当 no-op 会剩 2 → 红）。
    for (ops, want) in [
        (
            vec![ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            }],
            1i64,
        ),
        (
            vec![
                ScenarioOp::Send {
                    instance: 0,
                    msg: "Inc".into(),
                },
                ScenarioOp::Send {
                    instance: 0,
                    msg: "Inc".into(),
                },
                ScenarioOp::Send {
                    instance: 0,
                    msg: "Reset".into(),
                },
            ],
            0i64,
        ),
    ] {
        let compiled = compile_age_package_with_scenario(
            &[input("main", SRC)],
            "app::main",
            LegacyWidgetAlias::Reject,
            &[PACKAGES_CAPABILITY.to_string()],
            &ComponentScenario { instances: 1, ops },
        )
        .expect("compiles");
        let observed = ace_av::age_package::observe_age_package(&compiled, 1).expect("observes");
        assert_eq!(
            observed.components[0].instance_states,
            vec![vec![want as i32]]
        );
    }
}

/// B-PROOF-V[A]（AV 腿）：UI 事件节点的 2:1 定向事件与文本观察。
///
/// 具名边界：2D/3D 场景包的 AV 观察会在 `av.owned/1` 的 F32 类型处具名拒
/// （`lower.type-use`），场景呈现事实由前门用例冻结（`plan981_proof_a`），
/// AV 的 2D/3D 观察格归 B-AV（`D-981-PROOF-AV-SCENE`）。
#[test]
fn p981_proof_a_av_leg_two_instances_two_to_one() {
    const UI: &str = "node Ui {\n    model { count int = 0 }\n    on Inc { count = count + 1 }\n    view {\n        text \"ui\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "Inc".into(),
            },
        ],
    };
    let compiled = compile_age_package_with_scenario(
        &[input("main", UI)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    )
    .expect("the UI event node compiles");
    let observed =
        ace_av::age_package::observe_age_package(&compiled, 2).expect("AV observes both");
    assert_eq!(
        observed.components[0].instance_states,
        vec![vec![2], vec![1]]
    );
    assert_eq!(
        observed.components[0].instance_texts,
        vec![vec!["ui".to_string()], vec!["ui".to_string()]]
    );
}

/// B-PROOF-V[A]（AV 边界，具名）：2D 场景包本身可观察为空状态；而**同包 UI+2D 混合**
/// 会在 `av.owned/1` 的 F32 处具名拒（`lower.type-use`）——不静默给假状态。
/// 场景呈现事实由前门用例冻结（`plan981_proof_a`），该混合观察格归 B-AV
/// （`D-981-PROOF-AV-SCENE`）。
#[test]
fn p981_proof_a_av_scene_package_and_mixed_profile() {
    let g01: &str = include_str!("../../../core/tests/fixtures_g01/main.at");
    let res: &str = include_str!("../../../core/tests/fixtures_g01/resources.at");
    let shapes: &str = include_str!("../../../core/tests/fixtures_g01/shapes.at");
    let compiled = compile_age_package_with_scenario(
        &[
            input("main", g01),
            input("resources", res),
            input("shapes", shapes),
        ],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .expect("the 2D package compiles at the front door");
    let observed = ace_av::age_package::observe_age_package(&compiled, 1)
        .expect("a scene-only package observes its scene values in AV");
    assert_eq!(observed.components[0].name, "Main");
    assert!(observed.components[0]
        .instance_states
        .iter()
        .all(|s| s.is_empty()));
    // PLAN-981 B-W1（IF-981-PROOF-AV）：纯场景包不再只是空行——场景值函数在
    // AV 下给出真值，对齐 g01 原几何（dbb4b75 基线：红圆(-20,10)d30、蓝
    // quad(0,0)30×30、绿圆(25,-15)d24、黄quad(-45,-25)12×12 hidden）。
    use ace_av::age_package::SceneObserved;
    let val = |row: &str, attr: &str, comp: Option<usize>| {
        observed.components[0]
            .scene_values
            .iter()
            .find(|v| v.row == row && v.attr == attr && v.component == comp)
            .map(|v| v.value.clone())
            .unwrap_or_else(|| panic!("missing scene value {row}.{attr}{comp:?}"))
    };
    assert_eq!(val("red_disc", "x", None), SceneObserved::F32(-20.0));
    assert_eq!(val("red_disc", "y", None), SceneObserved::F32(10.0));
    assert_eq!(val("red_disc", "diameter", None), SceneObserved::F32(30.0));
    assert_eq!(
        val("blue_quad", "position", Some(0)),
        SceneObserved::F32(0.0)
    );
    assert_eq!(
        val("blue_quad", "position", Some(1)),
        SceneObserved::F32(0.0)
    );
    assert_eq!(val("blue_quad", "size", Some(0)), SceneObserved::F32(30.0));
    assert_eq!(val("blue_quad", "size", Some(1)), SceneObserved::F32(30.0));
    assert_eq!(val("green_disc", "x", None), SceneObserved::F32(25.0));
    assert_eq!(val("green_disc", "y", None), SceneObserved::F32(-15.0));
    assert_eq!(
        val("green_disc", "diameter", None),
        SceneObserved::F32(24.0)
    );
    assert_eq!(
        val("yellow_quad", "position", Some(0)),
        SceneObserved::F32(-45.0)
    );
    assert_eq!(
        val("yellow_quad", "position", Some(1)),
        SceneObserved::F32(-25.0)
    );
    assert_eq!(
        val("yellow_quad", "size", Some(0)),
        SceneObserved::F32(12.0)
    );
    assert_eq!(
        val("yellow_quad", "size", Some(1)),
        SceneObserved::F32(12.0)
    );
    assert_eq!(
        val("yellow_quad", "visible", None),
        SceneObserved::Bool(false)
    );

    // 混合 UI+2D 观察（D-981-PROOF-AV-SCENE 关闭）：同一包里 UI 文本/事件
    // 状态与 2D 场景值**同一次观察**都给真值——owned 车道已映射 f32，不再
    // 整模块具名拒。
    const MIX: &str = "node Mix {\n    model {\n        count int = 0\n    }\n    msg Msg { Bump(int) }\n    on {\n        .Bump(delta) -> {\n            .count += delta\n        }\n    }\n    view {\n        text \"ui\" {}\n    }\n    view-2d {\n        shape2d bar {\n            kind: quad\n            position: (3.5, 0.0)\n            size: (10.0, 4.0)\n        }\n    }\n}\n";
    use ace_core::source::age_component::InitialValue;
    let mixed = compile_age_package_with_scenario(
        &[input("main", MIX)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: vec![
                ace_core::source::age_component::ScenarioOp::send_args_from_values(
                    0,
                    "Bump",
                    &[InitialValue::Int(7)],
                    ace_core::atom_text::Span { start: 0, end: 0 },
                ),
            ],
        },
    )
    .expect("the mixed package compiles at the front door");
    let o = ace_av::age_package::observe_age_package(&mixed, 1)
        .expect("mixed UI+2D observes both channels in one pass");
    assert_eq!(o.components[0].instance_states, vec![vec![7]]);
    assert_eq!(o.components[0].instance_texts, vec![vec!["ui".to_string()]]);
    let mval = |attr: &str, comp: Option<usize>| {
        o.components[0]
            .scene_values
            .iter()
            .find(|v| v.attr == attr && v.component == comp)
            .map(|v| v.value.clone())
            .unwrap_or_else(|| panic!("missing mixed scene value {attr}{comp:?}"))
    };
    assert_eq!(mval("position", Some(0)), SceneObserved::F32(3.5));
    assert_eq!(mval("position", Some(1)), SceneObserved::F32(0.0));
    assert_eq!(mval("size", Some(0)), SceneObserved::F32(10.0));
    assert_eq!(mval("size", Some(1)), SceneObserved::F32(4.0));
}

/// R70-1：场景节点带 model 时 AV **具名拒**（不静默给空状态行）。
#[test]
fn p981_proof_a_av_scene_model_is_named_rejected() {
    const SRC: &str = "node Scene {\n    model { count int = 0 }\n    view-2d {\n        shape2d bar {\n            kind: quad\n            position: (0.0, 0.0)\n            size: (10.0, 4.0)\n        }\n    }\n}\n";
    let compiled = compile_age_package_with_scenario(
        &[input("main", SRC)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .expect("the scene node with a model compiles at the front door");
    let err = ace_av::age_package::observe_age_package(&compiled, 1)
        .expect_err("a model field without a text leaf must be named-rejected");
    assert!(
        err.contains("no text leaves") && err.contains("named rejection"),
        "the rejection explains why: {err}"
    );
}
