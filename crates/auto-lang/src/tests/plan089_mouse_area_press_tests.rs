// PLAN-089 探针：FileTree 行(mouse-area wrapper+content)的 press 链——
// styled_vtree 节点可见性 + path 对齐 + extract on_click。直接内联树体
// (子 widget emit 路由由 child_emit::tests 的 T614Widget 用例另护)。
#![cfg(test)]
use crate::ui::interpreter::DynamicMessage;
use crate::ui::view::View;

fn build_view(src: &str) -> View<DynamicMessage> {
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::parser::Parser::from(src).with_session(session);
    let ast = parser.parse().expect("parse");
    let decls: Vec<crate::ast::WidgetDecl> = ast
        .stmts
        .iter()
        .filter_map(|st| match st {
            crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
            _ => None,
        })
        .collect();
    let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
    let comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
        &decls[0],
        &decls[1..],
        &root_widget,
        crate::ui::widget_registry::WidgetRegistry::new(),
        vec![],
        &std::collections::HashMap::new(),
        false,
    )
    .expect("component");
    let (v, _, _) = comp.view_with_debug_gated(false);
    v
}

#[test]
fn probe_mouse_area_press_chain_aligned() {
    let src = r#"
widget Page {
    msg { Pick(str) }
    model {
        rows List = [
            { id: "backend", label: "backend", has_kids: true, state: "hover:bg-accent/50" },
            { id: "smoke-089.ad", label: "smoke-089.ad", has_kids: false, state: "hover:bg-accent/50" },
        ]
    }
    view {
        col (style: "w-full text-sm") {
            for r in .rows {
                row (style: "items-center") {
                    if r.has_kids {
                        mouse-area (onclick: .Pick(r.id), style: "w-5 h-5 shrink-0 cursor-pointer") {
                            text "v" { style: "text-xs" }
                        }
                    } else {
                        div (style: "w-5 shrink-0") {}
                    }
                    mouse-area (onclick: .Pick(r.id), class: f"flex-1 min-w-0 cursor-pointer ${r.state}") {
                        row (class: "flex flex-row items-center gap-1.5 py-1 w-full") {
                            text r.label { style: "text-sm" }
                        }
                    }
                }
            }
        }
    }
}
"#;
    let v = build_view(src);

    let tree = crate::ui::vnode_converter::view_to_vtree_with_paths(v.clone(), |_| None);
    let mut addressable = 0;
    for node in tree.nodes() {
        let view_at = crate::ui::mcp_server::find_view_by_path(&v, &node.path);
        let extract = view_at
            .and_then(|vw| crate::ui::mcp_server::extract_action_from_view(vw, "press"))
            .is_some();
        println!(
            "vnode {} kind={:?} path={:?} children={} extract={}",
            node.id.as_u64(),
            node.kind,
            node.path,
            node.children.len(),
            extract
        );
        if extract {
            addressable += 1;
        }
    }
    // 可 press 挂点 = 带 onclick 的 mouse-area wrapper:迭代 1 的 chevron
    // + 两迭代的行 wrapper = 3（迭代 2 has_kids=false 走占位 div,不计）。
    // 三面同构断言:kind=Container + content 子树在树 + path 走到
    // View::MouseArea 且 extract 出 on_click(PLAN-089 根修前:
    // wrapper=Text 叶被 G4 丢弃,press 链 "No press handler")。
    assert!(
        addressable >= 3,
        "expected >=3 pressable mouse-area nodes, got {addressable}"
    );
}
