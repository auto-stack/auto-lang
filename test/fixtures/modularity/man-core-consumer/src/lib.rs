//! PLAN-809 T-06 V-01: external consumer of the management rules core.
//! Consumes only `auto-man-core` — proving declaration normalization, link
//! closure (with the bounded-admission cycle contract) and selection/
//! planning rules work outside the crate boundary with no language/Man/VM
//! stack in the closure.

use auto_man_core::{
    assemble_major_targets, pac_from_decl_with_probe, resolve_target_closure, BackendName,
    GraphError, RenderSelection, Target, TargetKind, TargetOrigin,
};
use auto_val::{AutoStr, Node, Obj, Value};
use std::collections::HashMap;

fn root_with(props: &[(&str, Value)], kids: Vec<Node>) -> Node {
    let mut root = Node::new("root");
    for (k, v) in props {
        root.set_prop(*k, v.clone());
    }
    for k in kids {
        root.add_kid(k);
    }
    root
}

fn target_node(kind: &str, name: &str, links: &[&str]) -> Node {
    let mut n = Node::new(kind);
    n.args.add_pos(Value::Str(name.into()));
    if !links.is_empty() {
        n.set_prop(
            "links",
            Value::str_array(links.iter().map(|l| AutoStr::from(*l)).collect()),
        );
    }
    n
}

/// Declaration normalization + closure + planning over hand-built nodes:
/// the full management-rule chain with zero language dependencies.
#[test]
fn consumer_runs_full_rule_chain() {
    let root = root_with(
        &[
            ("name", Value::Str("demo".into())),
            ("render", Value::Str("vue".into())),
            ("title", Value::Str("Demo".into())),
            ("title_zh", Value::Str("演示".into())),
            ("scene", Value::Str("ui".into())),
        ],
        vec![
            target_node("app", "main", &["liba", "libb", "ext"]),
            target_node("lib", "liba", &["libc"]),
            target_node("lib", "libb", &["libc"]),
            target_node("lib", "libc", &[]),
        ],
    );
    let decl = pac_from_decl_with_probe(&root, &Obj::new(), |d| d == "apps/main");
    assert_eq!(decl.name, "demo");
    assert_eq!(decl.render, RenderSelection::Single(BackendName::Vue));
    assert_eq!(decl.title_zh.as_deref(), Some("演示"));
    assert_eq!(decl.targets.len(), 4);

    // link closure with external link allowed
    let mut map: HashMap<AutoStr, Target> = auto_man_core::targets_map(decl.targets.clone());
    resolve_target_closure(&mut map).expect("DAG + external link resolves");
    let app = &map[&AutoStr::from("main")];
    let ids: std::collections::HashSet<String> =
        app.all_links.iter().map(|l| l.id().to_string()).collect();
    assert!(ids.contains("liba") && ids.contains("libc"));
    assert!(
        !app.dep_names.iter().any(|d| d.as_str() == "ext"),
        "外部名不入 dep_names"
    );

    // major assembly + projections
    let majors = assemble_major_targets(&mut map);
    assert_eq!(majors.len(), 4, "App/Lib 保留（main/liba/libb/libc）");
    let node = auto_man_core::pac_to_node(
        &AutoStr::from("demo"),
        &AutoStr::from("0.1.0"),
        &majors,
        &Obj::new(),
        &AutoStr::from("."),
    );
    assert_eq!(node.get_prop("relative_path").to_astr(), "");

    // selection/planning rules
    let sel = auto_man_core::select_port_binding(&decl_targets_port(), &decl.backend);
    assert_eq!(sel.builder_name, "ninja");
    assert!(sel.show_headers);
    assert!(auto_man_core::select_rust_package(&majors, &AutoStr::from("demo")).is_none());
}

fn decl_targets_port() -> auto_man_core::Port {
    let mut port = auto_man_core::Port::default();
    port.builder = "ninja".into();
    port.at = "build".into();
    port
}

/// The bounded-admission contract is part of the public surface: internal
/// cycles produce a named NoProgress error, not a hang.
#[test]
fn consumer_sees_cycle_contract() {
    let mut a = Target::new("a", TargetKind::App);
    let mut b = Target::new("b", TargetKind::Lib);
    let mut link_b = Node::new("link");
    link_b.args.add_pos(Value::Str("b".into()));
    a.links.push(link_b);
    let mut link_a = Node::new("link");
    link_a.args.add_pos(Value::Str("a".into()));
    b.links.push(link_a);
    let mut map = auto_man_core::targets_map(vec![a, b]);
    let err = resolve_target_closure(&mut map).unwrap_err();
    match err {
        GraphError::NoProgress { ref unresolved, .. } => {
            assert!(unresolved.contains(&AutoStr::from("a")));
        }
    }
    assert!(err.to_string().contains("no progress"));
}

/// Target declaration forms (origin inference, at/renaming) stay available
/// at the old shapes.
#[test]
fn consumer_target_declaration_forms() {
    let mut dep = Node::new("dep");
    dep.args.add_pos(Value::Str("common".into()));
    dep.set_prop("path", Value::Str("../common".into()));
    let t = Target::from_with_probe(dep, AutoStr::from("app"), |d| d == "deps/common");
    assert_eq!(t.origin, TargetOrigin::Local);
    assert_eq!(t.from, "../common");
    assert_eq!(t.at, "deps/common");
    assert_eq!(t.rename, "common");
    assert_eq!(t.version, "latest");
}
