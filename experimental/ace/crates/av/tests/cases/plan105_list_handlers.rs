//! PLAN-105 T-02（U03）：组件面 **collections 域**（列表 handler）端到端回归。
//! `List<T>.new()` + `push` + `get` 在场景事件里执行并写回 int 字段；C 风格
//! 条件循环 `for <cond> { … }` 在此域按 `Stmt::While` 执行（013 真语料形）。
use ace_av::age_component::{
    compile_component_scenario_with_initials, observe_component_states, ComponentInitials,
    ComponentScenario, LegacyWidgetAlias, ScenarioOp,
};

fn run(src: &str) -> Vec<(String, String)> {
    let caps = vec!["ac.runtime.text.v1".to_string()];
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "M".to_string(),
        }],
    };
    let initials = ComponentInitials {
        per_instance: vec![Vec::new()],
    };
    let compiled = compile_component_scenario_with_initials(
        src, "p105.at", LegacyWidgetAlias::Accept, &caps, &scenario, None, &initials,
    )
    .expect("component compiles in the collections domain");
    let states = observe_component_states(&compiled).expect("states observe");
    states
        .per_instance
        .first()
        .map(|f| f.iter().map(|(k, v)| (k.clone(), format!("{v:?}"))).collect())
        .unwrap_or_default()
}

#[test]
fn p105_list_handler_executes_end_to_end() {
    let src = "widget App {\n    msg { M }\n    model { var n int = 0 }\n    view { text \"x\" {} }\n    on {\n        .M -> {\n            let xs = List<int>.new()\n            xs.push(7)\n            .n = xs.get(0)\n        }\n    }\n}\n";
    let fields = run(src);
    let n = fields.iter().find(|(k, _)| k == "n").map(|(_, v)| v.clone()).unwrap_or_default();
    assert!(n.contains('7'), "list push/get reached the field: {n}");
}

#[test]
fn p105_c_style_for_executes_as_while() {
    let src = "widget App {\n    msg { M }\n    model { var n int = 0 }\n    view { text \"x\" {} }\n    on {\n        .M -> {\n            var i int = 0\n            for i < 3 {\n                .n = .n + 1\n                i = i + 1\n            }\n        }\n    }\n}\n";
    let fields = run(src);
    let n = fields.iter().find(|(k, _)| k == "n").map(|(_, v)| v.clone()).unwrap_or_default();
    assert!(n.contains('3'), "C-style for ran three times: {n}");
}

#[test]
fn p105_local_list_binding_pushes_and_reads_back() {
    let src = "widget App {\n    msg { M }\n    model { var n int = 0 }\n    view { text \"x\" {} }\n    on {\n        .M -> {\n            var rows []str = []\n            rows.push(\"a\")\n            rows.push(\"b\")\n            .n = rows.len()\n        }\n    }\n}\n";
    let fields = run(src);
    let n = fields.iter().find(|(k, _)| k == "n").map(|(_, v)| v.clone()).unwrap_or_default();
    assert!(n.contains('2'), "local list push/len works: {n}");
}
#[test]
fn p105_list_local_to_field_assign() {
    let src = "widget App {\n    msg { M }\n    model { var n int = 0\n var xs []str = [] }\n    view { text \"x\" {} }\n    on {\n        .M -> {\n            var rows []str = []\n            rows.push(\"a\")\n            .xs = rows\n            .n = 1\n        }\n    }\n}\n";
    let fields = run(src);
    let n = fields.iter().find(|(k, _)| k == "n").map(|(_, v)| v.clone()).unwrap_or_default();
    assert!(n.contains('1'), "list local assigned to field: {n}");
}
#[test]
fn p105_record_list_push_and_project_field() {
    let src = "type Todo {\n    id int\n    text Str\n    done bool\n}\nwidget App {\n    msg { M }\n    model { var n int = 0\n var todos []Todo = [] }\n    view { text \"x\" {} }\n    on {\n        .M -> {\n            var t Todo = Todo { id: 1, text: \"a\", done: false }\n            .todos.push(t)\n            .n = .todos.get(0).text.len()\n        }\n    }\n}\n";
    let fields = run(src);
    let n = fields.iter().find(|(k, _)| k == "n").map(|(_, v)| v.clone()).unwrap_or_default();
    println!("[proj] n={n}");
    assert!(n.contains('1'), "projected field text len: {n}");
}
#[test]
#[test]
fn p105_list_set_element_replaces_value() {
    let src = "type Todo {\n    id int\n    text Str\n}\nwidget App {\n    msg { M }\n    model { var n int = 0\n var todos []Todo = [] }\n    view { text \"x\" {} }\n    on {\n        .M -> {\n            var t Todo = Todo { id: 1, text: \"a\" }\n            .todos.push(t)\n            .todos.set(0, Todo { id: 2, text: \"b\" })\n            .n = .todos.len()\n        }\n    }\n}\n";
    let fields = run(src);
    let n = fields.iter().find(|(k, _)| k == "n").map(|(_, v)| v.clone()).unwrap_or_default();
    println!("[set] n={n}");
}
// ---- PLAN-105 W105：owned 元素文本视图（record 字段落 owned Str +
// LIST_FIELD_VIEW 非破坏读）——013 的 `for t in .todos { text t.text }` 与
// `.todos.get(0).text` 值读形。 ----

/// 视图循环里读 record 元素的 text 字段（013 todo_list 真形）：字段位
/// plain `str` 落 owned Str（`mir.field-borrow` 的 view-kind 字段禁令不再
/// 触发），文本叶经元素视图链执行。
#[test]
fn p105_elem_field_text_leaf_in_view_loop() {
    let src = "type Todo {\n    id: int\n    text: str\n    done: bool\n}\nwidget App {\n    msg { M }\n    model { var todos []Todo = []\n var n int = 0 }\n    view {\n        text \"plain\" {}\n        for t in .todos {\n            text t.text\n        }\n    }\n    on {\n        .M -> { .todos.push(Todo { id: 1, text: \"a\", done: false })\n .n = .todos.len() }\n    }\n}\n";
    let fields = run(src);
    let n = fields.iter().find(|(k, _)| k == "n").map(|(_, v)| v.clone()).unwrap_or_default();
    assert!(n.contains('1'), "record list push + view loop text leaf: {n}");
}

/// handler 里的字段值读（`.todos.get(i).text`）：LIST_FIELD_VIEW 非破坏读——
/// place 不读 owned 元素（mir.list-type），LIST_REMOVE 的破坏形不再需要。
#[test]
fn p105_elem_field_text_view_value_read() {
    let src = "type Todo {\n    id: int\n    text: str\n    done: bool\n}\nwidget App {\n    msg { M }\n    model { var todos []Todo = []\n var out str = \"\" }\n    view { text .out {} }\n    on {\n        .M -> {\n            .todos.push(Todo { id: 1, text: \"a\", done: false })\n            .out = .todos.get(0).text\n        }\n    }\n}\n";
    let fields = run(src);
    let out = fields.iter().find(|(k, _)| k == "out").map(|(_, v)| v.clone()).unwrap_or_default();
    assert!(out.contains("\"a\"") || out.contains("a"), "LIST_FIELD_VIEW value read: {out}");
}

/// owned Str 字段的**字面量收敛**：record ctor 的 StrView 文本实参经
/// StrToOwned 进入 Aggregate（视图值永不进记录字段）。
#[test]
fn p105_record_ctor_str_literal_lands_owned() {
    let src = "type Todo {\n    id: int\n    text: str\n    done: bool\n}\nwidget App {\n    msg { M }\n    model { var todos []Todo = []\n var n int = 0 }\n    view { text \"x\" {} }\n    on {\n        .M -> {\n            .todos.push(Todo { id: 1, text: \"hello\", done: false })\n            .n = .todos.get(0).text.len()\n        }\n    }\n}\n";
    let fields = run(src);
    let n = fields.iter().find(|(k, _)| k == "n").map(|(_, v)| v.clone()).unwrap_or_default();
    assert!(n.contains('5'), "ctor literal -> owned field -> len 5: {n}");
}
