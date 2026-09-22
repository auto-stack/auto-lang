//! Plan 492 M2 (族 A1): parse_view_node 的 primary-shorthand `[` 后缀。
//!
//! `text t["label"]` — ident 后跟 `[` 目前不被识别为 primary prop 表达式
//! 开头（peek 只认 Dot/LParen），导致 Index 链分裂（dump+子文本）。修复后
//! 整条 `t["label"]` 作为 text prop 的 Expr::Index 挂载。

#[cfg(test)]
mod m2_primary_shorthand_bracket {
    use crate::ast::ui::{ViewNode, ViewPropValue};

    /// 返回 widget decl 的顶层 view 节点列表(parse 层,不经 aura 提取)。
    fn parse_view_nodes(src: &str) -> Vec<crate::ast::ui::ViewNode> {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        vec![decl.view.as_ref().expect("view block").root.clone()]
    }

    /// `text t["label"]`(循环内): text prop 必须是完整 Expr::Index,
    /// 不分裂、不 dump。
    #[test]
    fn text_shorthand_index_expr_whole() {
        let nodes = parse_view_nodes(
            r##"
widget W {
    model { items = [{ label: "a" }] }
    view {
        col {
            for t in .items {
                text t["label"] { }
            }
        }
    }
}
"##,
        );
        // col → children [ForLoop → body [text node]]
        let ViewNode::Element { children, .. } = &nodes[0] else {
            panic!("expected col root, got {:?}", nodes[0]);
        };
        let ViewNode::ForLoop { body, .. } = &children[0] else {
            panic!("expected for loop, got {:?}", children[0]);
        };
        let text_node = &body[0];
        let ViewNode::Element { tag, props, .. } = text_node else {
            panic!("expected text element, got {text_node:?}");
        };
        assert_eq!(tag, "text");
        let prop = props
            .iter()
            .find(|p| p.name == "text")
            .expect("text primary prop must exist");
        let rendered = match &prop.value {
            ViewPropValue::Expr(e) => format!("{e:?}"),
            other => panic!("expected expr prop value, got {other:?}"),
        };
        println!("text prop expr: {rendered}");
        assert!(
            rendered.contains("Index"),
            "t[\"label\"] must parse as Expr::Index on the text prop; got {rendered}"
        );
    }

    /// 非 Index 的既有形态不回归: ident.field / bare ident / 索引链多段。
    #[test]
    fn text_shorthand_existing_forms_still_parse() {
        let nodes = parse_view_nodes(
            r##"
widget W {
    model { items = [{ label: "a" }] }
    view {
        col {
            for t in .items {
                text t.label { }
                text t { }
            }
        }
    }
}
"##,
        );
        let ViewNode::Element { children, .. } = &nodes[0] else {
            panic!("expected col root");
        };
        let ViewNode::ForLoop { body, .. } = &children[0] else {
            panic!("expected for loop");
        };
        assert_eq!(body.len(), 2, "two text nodes: {body:?}");
    }
}
