//! PLAN-104（U02 007 真语料）：AGE 视图**裸自闭合标签**（`spacer`/`divider`/
//! `sep`/`outlet`）——闭集 keyword token，只有标签名、无 label、无体块。
//! 判据：①四拼写都能投影为元素；②裸标签后的条目（`style:`）不被跨行吞成
//! label（007 原始失败形态）；③未知裸拼写仍具名结构拒（fail-closed）。
use ace_core::source::age_component::{project_component_candidates, ComponentViewItem};

fn project(src: &str) -> Result<Vec<ace_core::source::age_component::ComponentNode>, Vec<String>> {
    project_component_candidates(src).map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect()
    })
}

fn element_tags(items: &[ComponentViewItem], out: &mut Vec<String>) {
    for it in items {
        match it {
            ComponentViewItem::Element(el) => {
                out.push(el.tag.clone());
                element_tags(&el.items, out);
            }
            ComponentViewItem::Cond(c) => {
                element_tags(&c.then_items, out);
                element_tags(&c.else_items, out);
            }
            ComponentViewItem::For(f) => element_tags(&f.items, out),
            _ => {}
        }
    }
}

fn tags_of(src: &str) -> Vec<String> {
    let nodes = project(src).expect("source must project");
    let mut out = Vec::new();
    element_tags(&nodes[0].view.tree, &mut out);
    out
}

#[test]
fn p104_bare_tags_project_as_elements() {
    for tag in ["spacer", "divider", "sep", "outlet"] {
        let src = format!("widget App {{\n    view {{ col {{\n        {tag}\n    }} }}\n}}\n");
        let nodes = project(&src).expect("bare tag must project");
        let mut out = Vec::new();
        element_tags(&nodes[0].view.tree, &mut out);
        assert!(out.contains(&tag.to_string()), "`{tag}` must project as an element: {out:?}");
    }
}

/// 007 原始失败形态：裸标签后接 `style:` 条目——条目必须仍归属父元素，
/// 不能被 `age-element-labeled` 跨行吞成 label（吞并时条目消失/报错）。
#[test]
fn p104_bare_tag_does_not_swallow_following_entry() {
    let src = "widget App {\n    view { col {\n        spacer\n        style: \"p-4\"\n    } }\n}\n";
    let nodes = project(src).expect("bare tag + entry must project");
    let col = match &nodes[0].view.tree[0] {
        ComponentViewItem::Element(e) => e,
        other => panic!("expected col element, got {other:?}"),
    };
    let entries: Vec<&str> = col
        .items
        .iter()
        .filter_map(|it| match it {
            ComponentViewItem::Entry(en) => Some(en.key.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        entries.contains(&"style"),
        "the trailing `style:` must stay an entry of the parent: {entries:?}"
    );
}

/// 负控：未列入闭集的裸拼写仍走具名结构拒（不静默成元素）。
#[test]
fn p104_unknown_bare_tag_is_named_rejected() {
    let src = "widget App {\n    view { col {\n        bogus\n    } }\n}\n";
    let err = project(src).expect_err("unknown bare tag must be rejected");
    assert!(
        err.iter().any(|m| m.starts_with("source.")),
        "named source diagnostic expected: {err:?}"
    );
}
