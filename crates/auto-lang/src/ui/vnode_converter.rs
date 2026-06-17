//! View<M> → VTree 转换器
//!
//! 将嵌套的 View<M> 树转换为扁平的 VNode 树结构。
//!
//! ## 核心功能
//!
//! - **扁平化转换**：将嵌套的 View<M> 树转换为扁平的 VNode 列表
//! - **ID 引用**：使用 ID 引用建立父子关系，而非直接嵌套
//! - **完整支持**：支持所有 22 个 View 变体
//!
//! ## 使用示例
//!
//! ```ignore
//! use auto_ui::view::View;
//! use auto_ui::vnode_converter::view_to_vtree;
//!
//! let view = View::Column {
//!     children: vec![
//!         View::Text { content: "Hello".to_string(), style: None }
//!     ],
//!     spacing: 10,
//!     padding: 0,
//!     style: None
//! };
//!
//! let vtree = view_to_vtree(view);
//! assert_eq!(vtree.node_count(), 2); // Column + Text
//! ```

use super::view::View;
use super::vnode::{id_from_path, VNode, VNodeId, VNodeKind, VNodeProps, VTree};

// 导入 DynamicMessage 和 EventRouter（仅在 interpreter feature 启用时）
#[cfg(feature = "interpreter")]
use super::interpreter::DynamicMessage;

#[cfg(feature = "interpreter")]
use super::event_router::{EventRouter, EventContext, EventType};

/// 主转换函数：View<M> → VTree
///
/// 将嵌套的 View<M> 树转换为扁平的 VNode 树结构。
///
/// # 类型参数
///
/// * `M` - 消息类型，必须实现 Clone 和 Debug
///
/// # 参数
///
/// * `view` - 要转换的 View 树
///
/// # 返回
///
/// 转换后的 VTree
///
/// # 示例
///
/// ```ignore
/// let view = View::Text {
///     content: "Hello".to_string(),
///     style: None
/// };
///
/// let vtree = view_to_vtree(view);
/// ```
pub fn view_to_vtree<M>(view: View<M>) -> VTree
where
    M: Clone + std::fmt::Debug,
{
    let mut tree = VTree::new();
    let root_id = tree.next_id();

    let root_node = convert_view_to_vnode(&view, root_id, None, &mut tree);
    tree.set_root(root_node);

    tree
}

/// 带路径与 span 的 View → VTree 转换（Task 4）
///
/// 与 [`view_to_vtree`] 行为一致，区别在于：
///
/// - 每个节点的 `VNodeId` 由其逻辑 `path`（从根的子索引序列）经
///   [`id_from_path`] 派生，而非顺序分配。结构不变 → id 不变。
/// - `vnode.path` 被设置为该节点的完整 path。
/// - `vnode.source_span` 通过 `span_for` 回调按 path 查询填充（允许返回
///   `None` 表示无可用 span）。
///
/// 根节点 path = `[]`。
///
/// 现有的 [`view_to_vtree`]（顺序 id）保持不变，GPUI/headless 路径仍使用它。
pub fn view_to_vtree_with_paths<M, F>(view: View<M>, span_for: F) -> VTree
where
    M: Clone + std::fmt::Debug,
    F: Fn(&[u16]) -> Option<crate::ui::debug::SourceSpan>,
{
    let mut tree = VTree::new();
    let mut path: Vec<u16> = Vec::new();

    let root_node = convert_view_to_vnode_with_path(&view, None, &mut path, &mut tree, &span_for);
    tree.set_root(root_node);

    tree
}

/// `view_to_vtree_with_paths` 的递归 walker。
///
/// 复用 [`extract_kind_and_props`] / [`extract_children`]，不做重复的 View 映射。
/// `path` 维护当前节点从根开始的子索引序列；进入子节点前 `push`，返回后 `pop`，
/// 保证不泄漏到兄弟节点。
fn convert_view_to_vnode_with_path<M, F>(
    view: &View<M>,
    parent_id: Option<VNodeId>,
    path: &mut Vec<u16>,
    tree: &mut VTree,
    span_for: &F,
) -> VNode
where
    M: Clone + std::fmt::Debug,
    F: Fn(&[u16]) -> Option<crate::ui::debug::SourceSpan>,
{
    let id = VNodeId::new(id_from_path(path));
    let (kind, props) = extract_kind_and_props(view);

    let mut vnode = VNode::new(id, kind, props)
        .with_label(format!("{}", kind))
        .with_path(path.clone());

    if let Some(span) = span_for(path) {
        vnode = vnode.with_source_span(span);
    }
    if let Some(parent) = parent_id {
        vnode = vnode.with_parent(parent);
    }

    // 处理子节点：按 children 数组顺序，child_index 从 0 起
    let children = extract_children(view);
    for (child_index, child_view) in children.into_iter().enumerate() {
        path.push(child_index as u16);
        let child_id = VNodeId::new(id_from_path(path));
        let child_node =
            convert_view_to_vnode_with_path(&child_view, Some(id), path, tree, span_for);
        path.pop();

        tree.add_node(child_node);
        vnode.add_child(child_id);
    }

    vnode
}

/// 将单个 View 转换为 VNode（递归处理子节点）
///
/// # 参数
///
/// * `view` - 要转换的 View
/// * `id` - 为此节点分配的 ID
/// * `parent_id` - 父节点 ID（如果有）
/// * `tree` - VTree 用于添加子节点
///
/// # 返回
///
/// 转换后的 VNode
fn convert_view_to_vnode<M>(
    view: &View<M>,
    id: VNodeId,
    parent_id: Option<VNodeId>,
    tree: &mut VTree,
) -> VNode
where
    M: Clone + std::fmt::Debug,
{
    let (kind, props) = extract_kind_and_props(view);

    let mut vnode = VNode::new(id, kind, props).with_label(format!("{}", kind));

    if let Some(parent) = parent_id {
        vnode = vnode.with_parent(parent);
    }

    // 处理子节点
    let children = extract_children(view);
    for child_view in children {
        let child_id = tree.next_id();
        let child_node = convert_view_to_vnode(&child_view, child_id, Some(id), tree);
        tree.add_node(child_node);
        vnode.add_child(child_id);
    }

    vnode
}

/// 从 View 中提取类型和属性
///
/// # 参数
///
/// * `view` - 要提取属性的 View
///
/// # 返回
///
/// (VNodeKind, VNodeProps) 元组
fn extract_kind_and_props<M>(view: &View<M>) -> (VNodeKind, VNodeProps)
where
    M: Clone + std::fmt::Debug,
{
    match view {
        View::Empty => (VNodeKind::Text, VNodeProps::Empty),

        View::Text { content, .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: content.clone(),
            },
        ),

        View::Button { label, .. } => (
            VNodeKind::Button,
            VNodeProps::Button {
                label: label.clone(),
            },
        ),

        View::Column { spacing, padding, .. } => (
            VNodeKind::Column,
            VNodeProps::Layout {
                spacing: *spacing,
                padding: *padding,
            },
        ),

        View::Row { spacing, padding, .. } => (
            VNodeKind::Row,
            VNodeProps::Layout {
                spacing: *spacing,
                padding: *padding,
            },
        ),

        View::Input {
            placeholder,
            value,
            password,
            ..
        } => (
            VNodeKind::Input,
            VNodeProps::Input {
                placeholder: placeholder.clone(),
                value: value.clone(),
                password: *password,
            },
        ),

        View::Textarea {
            placeholder,
            value,
            ..
        } => (
            VNodeKind::Textarea,
            VNodeProps::Textarea {
                placeholder: placeholder.clone(),
                value: value.clone(),
            },
        ),

        View::Checkbox {
            label, is_checked, ..
        } => (
            VNodeKind::Checkbox,
            VNodeProps::Checkbox {
                label: label.clone(),
                is_checked: *is_checked,
            },
        ),

        View::Radio {
            label, is_selected, ..
        } => (
            VNodeKind::Radio,
            VNodeProps::Radio {
                label: label.clone(),
                is_selected: *is_selected,
            },
        ),

        View::Select {
            options, selected_index, ..
        } => (
            VNodeKind::Select,
            VNodeProps::Select {
                options: options.clone(),
                selected_index: *selected_index,
            },
        ),

        View::Container {
            padding,
            center_x,
            center_y,
            ..
        } => (
            VNodeKind::Container,
            VNodeProps::Container {
                padding: *padding,
                center_x: *center_x,
                center_y: *center_y,
            },
        ),

        View::Scrollable { .. } => (VNodeKind::Scrollable, VNodeProps::Scrollable),

        View::List { spacing, .. } => (
            VNodeKind::List,
            VNodeProps::List { spacing: *spacing },
        ),

        View::Table {
            spacing, col_spacing, ..
        } => (
            VNodeKind::Table,
            VNodeProps::Table {
                spacing: *spacing,
                col_spacing: *col_spacing,
            },
        ),

        View::Slider {
            min, max, value, step, ..
        } => (
            VNodeKind::Slider,
            VNodeProps::Slider {
                min: *min,
                max: *max,
                value: *value,
                step: *step,
            },
        ),

        View::ProgressBar { progress, .. } => (
            VNodeKind::ProgressBar,
            VNodeProps::ProgressBar {
                progress: *progress,
            },
        ),

        // 高级组件（Plan 010）- 暂不支持，返回占位符
        View::Accordion { .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: "[Accordion 暂不支持]".to_string(),
            },
        ),

        View::Sidebar { .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: "[Sidebar 暂不支持]".to_string(),
            },
        ),

        View::Tabs { .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: "[Tabs 暂不支持]".to_string(),
            },
        ),

        View::NavigationRail { .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: "[NavigationRail 暂不支持]".to_string(),
            },
        ),

        // Grid (Plan 319): MVP — surface as a Column in the DevTools VTree
        // (its `gap` maps to layout spacing). A dedicated VNodeKind::Grid is
        // a follow-up. Children come from `extract_children` (cells).
        View::Grid { gap, .. } => (
            VNodeKind::Column,
            VNodeProps::Layout {
                spacing: *gap,
                padding: 0,
            },
        ),

        View::Image { .. } => (
            VNodeKind::Text,
            VNodeProps::Text {
                content: "[Image]".to_string(),
            },
        ),
    }
}

/// 从 View 中提取子节点列表
///
/// # 参数
///
/// * `view` - 要提取子节点的 View
///
/// # 返回
///
/// 子 View 的向量
fn extract_children<M>(view: &View<M>) -> Vec<View<M>>
where
    M: Clone + std::fmt::Debug,
{
    match view {
        View::Column { children, .. } => children.clone(),
        View::Row { children, .. } => children.clone(),
        // Grid cells are the VTree children (Plan 319). MUST be explicit —
        // the `_` arm below would silently drop them.
        View::Grid { cells, .. } => cells.clone(),
        View::Container { child, .. } => vec![*child.clone()],
        View::Scrollable { child, .. } => vec![*child.clone()],
        View::List { items, .. } => items.clone(),
        View::Table { headers, rows, .. } => {
            let mut children = headers.clone();
            for row in rows {
                for cell in row {
                    children.push(cell.clone());
                }
            }
            children
        }
        View::Tabs { contents, .. } => contents.clone(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用的简化消息类型
    #[derive(Debug, Clone, Copy)]
    enum TestMsg {
        Click,
        Change,
    }

    #[test]
    fn test_simple_text_conversion() {
        let view: View<TestMsg> = View::Text {
            content: "Hello".to_string(),
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Text);
        assert!(tree.validate().is_ok());
    }

    #[test]
    fn test_empty_conversion() {
        let view: View<TestMsg> = View::Empty;

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Text);
        assert!(matches!(root.props, VNodeProps::Empty));
    }

    #[test]
    fn test_button_conversion() {
        let view: View<TestMsg> = View::Button {
            label: "Click Me".to_string(),
            onclick: TestMsg::Click,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Button);
        if let VNodeProps::Button { label } = &root.props {
            assert_eq!(label, "Click Me");
        } else {
            panic!("Expected Button props");
        }
    }

    #[test]
    fn test_column_with_children() {
        let view: View<TestMsg> = View::Column {
            children: vec![
                View::Text {
                    content: "A".to_string(),
                    style: None,
                },
                View::Text {
                    content: "B".to_string(),
                    style: None,
                },
            ],
            spacing: 10,
            padding: 0,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 3); // 1 Column + 2 Text
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Column);
        assert_eq!(root.children.len(), 2);

        // 验证子节点
        let child1 = tree.get(root.children[0]).unwrap();
        assert_eq!(child1.kind, VNodeKind::Text);

        let child2 = tree.get(root.children[1]).unwrap();
        assert_eq!(child2.kind, VNodeKind::Text);
    }

    // Plan 319: Grid cells must surface as VTree children, not be silently
    // dropped by the `_` catch-all in extract_children.
    #[test]
    fn test_grid_cells_become_children() {
        let cells: Vec<View<TestMsg>> = (0..7)
            .map(|i| View::Text {
                content: format!("cell{}", i),
                style: None,
            })
            .collect();
        let view: View<TestMsg> = View::Grid {
            cols: 3,
            gap: 4,
            cells,
            style: None,
        };

        let tree = view_to_vtree(view);

        // 1 Grid + 7 Text cells = 8 nodes.
        assert_eq!(tree.node_count(), 8);
        let root = tree.root().unwrap();
        // MVP: grid reuses Column as its VNodeKind.
        assert_eq!(root.kind, VNodeKind::Column);
        assert_eq!(root.children.len(), 7);
        if let VNodeProps::Layout { spacing, padding } = &root.props {
            assert_eq!(*spacing, 4); // gap surfaces as spacing
            assert_eq!(*padding, 0);
        } else {
            panic!("Expected Layout props for grid");
        }
    }

    #[test]
    fn test_row_conversion() {
        let view: View<TestMsg> = View::Row {
            children: vec![
                View::Text {
                    content: "Left".to_string(),
                    style: None,
                },
                View::Text {
                    content: "Right".to_string(),
                    style: None,
                },
            ],
            spacing: 5,
            padding: 10,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 3); // 1 Row + 2 Text
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Row);

        if let VNodeProps::Layout { spacing, padding } = &root.props {
            assert_eq!(*spacing, 5);
            assert_eq!(*padding, 10);
        } else {
            panic!("Expected Layout props");
        }
    }

    #[test]
    fn test_nested_structure() {
        let view: View<TestMsg> = View::Column {
            children: vec![View::Row {
                children: vec![View::Text {
                    content: "Nested".to_string(),
                    style: None,
                }],
                spacing: 5,
                padding: 0,
                style: None,
            }],
            spacing: 10,
            padding: 0,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 3); // Column + Row + Text

        // 验证嵌套关系
        let root = tree.root().unwrap();
        assert_eq!(root.children.len(), 1);

        let row_id = root.children[0];
        let row = tree.get(row_id).unwrap();
        assert_eq!(row.kind, VNodeKind::Row);
        assert_eq!(row.parent, Some(root.id));

        let text_id = row.children[0];
        let text = tree.get(text_id).unwrap();
        assert_eq!(text.kind, VNodeKind::Text);
        assert_eq!(text.parent, Some(row_id));
    }

    #[test]
    fn test_container_conversion() {
        let view: View<TestMsg> = View::Container {
            child: Box::new(View::Text {
                content: "Centered".to_string(),
                style: None,
            }),
            padding: 20,
            width: None,
            height: None,
            center_x: true,
            center_y: true,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 2);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Container);

        if let VNodeProps::Container {
            padding,
            center_x,
            center_y,
        } = &root.props
        {
            assert_eq!(*padding, 20);
            assert!(*center_x);
            assert!(*center_y);
        } else {
            panic!("Expected Container props");
        }

        // 验证子节点
        assert_eq!(root.children.len(), 1);
        let child = tree.get(root.children[0]).unwrap();
        assert_eq!(child.kind, VNodeKind::Text);
    }

    #[test]
    fn test_input_conversion() {
        let view: View<TestMsg> = View::Input {
            placeholder: "Enter text".to_string(),
            value: "".to_string(),
            on_change: None,
            on_submit: None,
            width: None,
            password: false,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Input);

        if let VNodeProps::Input {
            placeholder,
            value,
            password,
        } = &root.props
        {
            assert_eq!(placeholder, "Enter text");
            assert_eq!(value, "");
            assert!(!(*password));
        } else {
            panic!("Expected Input props");
        }
    }

    #[test]
    fn test_checkbox_conversion() {
        let view: View<TestMsg> = View::Checkbox {
            is_checked: true,
            label: "Check me".to_string(),
            on_toggle: None,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Checkbox);

        if let VNodeProps::Checkbox { label, is_checked } = &root.props {
            assert_eq!(label, "Check me");
            assert!(*is_checked);
        } else {
            panic!("Expected Checkbox props");
        }
    }

    #[test]
    fn test_select_conversion() {
        let view: View<TestMsg> = View::Select {
            options: vec!["Option 1".to_string(), "Option 2".to_string()],
            selected_index: Some(0),
            on_select: None,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Select);

        if let VNodeProps::Select {
            options,
            selected_index,
        } = &root.props
        {
            assert_eq!(options.len(), 2);
            assert_eq!(selected_index, &Some(0));
        } else {
            panic!("Expected Select props");
        }
    }

    #[test]
    fn test_scrollable_conversion() {
        let view: View<TestMsg> = View::Scrollable {
            child: Box::new(View::Text {
                content: "Scrollable content".to_string(),
                style: None,
            }),
            width: None,
            height: None,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 2);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Scrollable);
    }

    #[test]
    fn test_list_conversion() {
        let view: View<TestMsg> = View::List {
            items: vec![
                View::Text {
                    content: "Item 1".to_string(),
                    style: None,
                },
                View::Text {
                    content: "Item 2".to_string(),
                    style: None,
                },
            ],
            spacing: 8,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 3); // 1 List + 2 Text
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::List);
    }

    #[test]
    fn test_slider_conversion() {
        let view: View<TestMsg> = View::Slider {
            min: 0.0,
            max: 100.0,
            value: 50.0,
            step: Some(1.0),
            on_change: |_v| TestMsg::Change,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Slider);

        if let VNodeProps::Slider { min, max, value, step } = &root.props {
            assert_eq!(*min, 0.0);
            assert_eq!(*max, 100.0);
            assert_eq!(*value, 50.0);
            assert_eq!(step, &Some(1.0));
        } else {
            panic!("Expected Slider props");
        }
    }

    #[test]
    fn test_progress_bar_conversion() {
        let view: View<TestMsg> = View::ProgressBar {
            progress: 0.75,
            style: None,
        };

        let tree = view_to_vtree(view);

        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::ProgressBar);

        if let VNodeProps::ProgressBar { progress } = &root.props {
            assert_eq!(*progress, 0.75);
        } else {
            panic!("Expected ProgressBar props");
        }
    }

    #[test]
    fn test_tree_validity() {
        // 测试复杂树的有效性
        let view: View<TestMsg> = View::Column {
            children: vec![
                View::Row {
                    children: vec![
                        View::Text {
                            content: "A".to_string(),
                            style: None,
                        },
                        View::Text {
                            content: "B".to_string(),
                            style: None,
                        },
                    ],
                    spacing: 5,
                    padding: 0,
                    style: None,
                },
                View::Button {
                    label: "Click".to_string(),
                    onclick: TestMsg::Click,
                    style: None,
                },
            ],
            spacing: 10,
            padding: 0,
            style: None,
        };

        let tree = view_to_vtree(view);

        // 验证树的完整性
        assert!(tree.validate().is_ok());

        // 验证节点数量
        assert_eq!(tree.node_count(), 5); // Column + Row + 2 Text + Button

        // 验证深度
        assert_eq!(tree.depth(), 3);
    }

    #[test]
    fn test_advanced_components_placeholder() {
        // 测试高级组件（暂不支持）返回占位符
        let accordion_view: View<TestMsg> = View::Accordion {
            items: vec![],
            allow_multiple: false,
            on_toggle: None,
            style: None,
        };

        let tree = view_to_vtree(accordion_view);
        let root = tree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Text);

        if let VNodeProps::Text { content } = &root.props {
            assert!(content.contains("暂不支持"));
        } else {
            panic!("Expected placeholder text");
        }
    }

    #[test]
    fn test_tree_stats() {
        let view = View::Column {
            children: vec![
                View::Text {
                    content: "Title".to_string(),
                    style: None,
                },
                View::Button {
                    label: "Click".to_string(),
                    onclick: TestMsg::Click,
                    style: None,
                },
            ],
            spacing: 10,
            padding: 0,
            style: None,
        };

        let tree = view_to_vtree(view);
        let stats = tree.stats();

        assert_eq!(stats.total_nodes, 3);
        assert_eq!(stats.text_nodes, 1);
        assert_eq!(stats.button_nodes, 1);
        assert_eq!(stats.layout_nodes, 1);
        assert_eq!(stats.leaf_nodes, 2);
        assert_eq!(stats.max_depth, 2);
    }

    // =================================================================
    // view_to_vtree_with_paths tests (Task 4)
    // =================================================================

    #[test]
    fn vtree_with_paths_assigns_path_derived_ids() {
        use super::view_to_vtree_with_paths;
        use crate::ui::vnode::id_from_path;

        // root col -> [ text, row -> [ button ] ]
        let view: View<u32> = View::Column {
            children: vec![
                View::Text { content: "a".into(), style: None },
                View::Row { children: vec![
                    View::Button { label: "b".into(), onclick: 0, style: None },
                ], spacing: 0, padding: 0, style: None },
            ],
            spacing: 0, padding: 0, style: None,
        };
        let tree = view_to_vtree_with_paths(view, |_| None);

        // root id 由空 path 派生
        let root = tree.root().expect("has root");
        assert_eq!(root.id.as_u64(), id_from_path(&[]));
        assert_eq!(root.path, Vec::<u16>::new());
        assert_eq!(root.source_span, None);

        // 第一个子节点 path [0]
        let kids = tree.children(root.id).unwrap();
        assert_eq!(kids.len(), 2);
        assert_eq!(kids[0].path, vec![0]);
        assert_eq!(kids[0].id.as_u64(), id_from_path(&[0]));
        // row 的子按钮 path [1,0]
        let row_kids = tree.children(kids[1].id).unwrap();
        assert_eq!(row_kids[0].path, vec![1, 0]);
        assert_eq!(row_kids[0].id.as_u64(), id_from_path(&[1, 0]));
    }

    #[test]
    fn vtree_with_paths_span_callback_invoked_per_path() {
        use super::view_to_vtree_with_paths;
        let view: View<u32> = View::Column {
            children: vec![View::Text { content: "x".into(), style: None }],
            spacing: 0, padding: 0, style: None,
        };
        let tree = view_to_vtree_with_paths(view, |path| {
            Some(crate::ui::debug::SourceSpan {
                offset: path.iter().map(|&x| x as usize).sum::<usize>(),
                len: 1,
            })
        });
        let root = tree.root().unwrap();
        assert_eq!(
            root.source_span,
            Some(crate::ui::debug::SourceSpan { offset: 0, len: 1 })
        );
        let kid = &tree.children(root.id).unwrap()[0];
        assert_eq!(
            kid.source_span,
            Some(crate::ui::debug::SourceSpan { offset: 0, len: 1 })
        ); // path [0] -> offset 0
    }

    #[test]
    fn vtree_with_paths_ids_stable_across_builds() {
        use super::view_to_vtree_with_paths;
        fn build() -> View<u32> {
            View::Column {
                children: vec![View::Text { content: "c".into(), style: None }],
                spacing: 0,
                padding: 0,
                style: None,
            }
        }
        let t1 = view_to_vtree_with_paths(build(), |_| None);
        let t2 = view_to_vtree_with_paths(build(), |_| None);
        assert_eq!(t1.root().unwrap().id, t2.root().unwrap().id);
        // 子节点 id 也一致
        let c1 = t1.children(t1.root().unwrap().id).unwrap()[0].id;
        let c2 = t2.children(t2.root().unwrap().id).unwrap()[0].id;
        assert_eq!(c1, c2);
    }
}

// ============================================================================
// Phase 3 完整版: 事件感知的 VTree 转换
// ============================================================================

/// 带 EventRouter 的 View → VTree 转换（Phase 3 完整版）
///
/// 此函数在转换 VTree 的同时,提取 View 中的事件处理器并注册到 EventRouter。
///
/// # 返回
///
/// (VTree, EventRouter) 元组
///
/// # 示例
///
/// ```ignore
/// let view = View::Button {
///     label: "Click".to_string(),
///     onclick: DynamicMessage::String("click-event".to_string()),
///     style: None
/// };
///
/// let (vtree, router) = view_to_vtree_with_events(view);
///
/// // 获取按钮点击消息
/// let button_id = vtree.root().unwrap().id;
/// if let Some(msg) = router.on_click(button_id) {
///     assert!(matches!(msg, DynamicMessage::String(_)));
/// }
/// ```
#[cfg(feature = "interpreter")]
pub fn view_to_vtree_with_events<M>(view: View<M>) -> (VTree, EventRouter)
where
    M: Clone + std::fmt::Debug + 'static,
{
    let mut tree = VTree::new();
    let mut router = EventRouter::new();
    let root_id = tree.next_id();

    let root_node = convert_view_to_vnode_with_events(&view, root_id, None, &mut tree, &mut router);
    tree.set_root(root_node);

    (tree, router)
}

/// 将 View 转换为 VNode,同时提取事件并注册到 EventRouter
#[cfg(feature = "interpreter")]
fn convert_view_to_vnode_with_events<M>(
    view: &View<M>,
    id: VNodeId,
    parent_id: Option<VNodeId>,
    tree: &mut VTree,
    router: &mut EventRouter,
) -> VNode
where
    M: Clone + std::fmt::Debug + 'static,
{
    let (kind, props) = extract_kind_and_props(view);

    let mut vnode = VNode::new(id, kind, props).with_label(format!("{}", kind));

    if let Some(parent) = parent_id {
        vnode = vnode.with_parent(parent);
    }

    // 🎯 Phase 3 关键: 提取事件并注册到 EventRouter
    extract_and_register_events(view, id, router);

    // 处理子节点
    let children = extract_children(view);
    for child_view in children {
        let child_id = tree.next_id();
        let child_node = convert_view_to_vnode_with_events(&child_view, child_id, Some(id), tree, router);
        tree.add_node(child_node);
        vnode.add_child(child_id);
    }

    vnode
}

/// 从 View 中提取事件并注册到 EventRouter
///
/// 此函数检查 View 中的 onclick, on_change, on_toggle, on_select 等事件处理器,
/// 将它们转换为 DynamicMessage 并注册到 EventRouter 中。
#[cfg(feature = "interpreter")]
fn extract_and_register_events<M>(view: &View<M>, node_id: VNodeId, router: &mut EventRouter)
where
    M: Clone + std::fmt::Debug + 'static,
{
    use std::any::TypeId;

    // 检查 M 是否是 DynamicMessage
    let is_dynamic_message = TypeId::of::<M>() == TypeId::of::<DynamicMessage>();

    match view {
        // Button onclick 事件
        View::Button { onclick, .. } => {
            let msg = if is_dynamic_message {
                // 安全:我们知道 M 是 DynamicMessage
                unsafe {
                    let any_msg = onclick as *const M as *const DynamicMessage;
                    (*any_msg).clone()
                }
            } else {
                // 对于静态类型,使用类型名称作为事件标识符
                DynamicMessage::String(format!("{:}", std::any::type_name::<M>()))
            };

            router.register_click(node_id, move |_ctx| msg.clone());
        }

        // Input on_change 事件
        View::Input { on_change, .. } => {
            if let Some(change_handler) = on_change {
                let msg = if is_dynamic_message {
                    unsafe {
                        let any_msg = change_handler as *const _ as *const DynamicMessage;
                        (*any_msg).clone()
                    }
                } else {
                    DynamicMessage::String(format!("change:{}", std::any::type_name::<M>()))
                };

                router.register_change(node_id, move |ctx| {
                    if let EventType::Change(value) = &ctx.event_type {
                        // 注意:这里应该生成包含新值的消息
                        // 暂时返回原始消息
                        msg.clone()
                    } else {
                        msg.clone()
                    }
                });
            }
        }

        // Checkbox on_toggle 事件
        View::Checkbox { on_toggle, .. } => {
            if let Some(toggle_handler) = on_toggle {
                let msg = if is_dynamic_message {
                    unsafe {
                        let any_msg = toggle_handler as *const _ as *const DynamicMessage;
                        (*any_msg).clone()
                    }
                } else {
                    DynamicMessage::String(format!("toggle:{}", std::any::type_name::<M>()))
                };

                router.register_toggle(node_id, move |_ctx| msg.clone());
            }
        }

        // Slider on_change 事件
        View::Slider { on_change, .. } => {
            let msg = if is_dynamic_message {
                unsafe {
                    let any_msg = on_change as *const _ as *const DynamicMessage;
                    (*any_msg).clone()
                }
            } else {
                DynamicMessage::String(format!("slider_change:{}", std::any::type_name::<M>()))
            };

            router.register_change(node_id, move |_ctx| msg.clone());
        }

        // Select on_select 事件
        View::Select { on_select, .. } => {
            if let Some(select_handler) = on_select {
                let msg = if is_dynamic_message {
                    unsafe {
                        let any_msg = select_handler as *const _ as *const DynamicMessage;
                        (*any_msg).clone()
                    }
                } else {
                    DynamicMessage::String(format!("select:{}", std::any::type_name::<M>()))
                };

                router.register_select(node_id, move |_ctx| msg.clone());
            }
        }

        // 其他 View 类型没有事件,忽略
        _ => {}
    }
}

#[cfg(test)]
#[cfg(feature = "interpreter")]
mod tests_with_events {
    use super::*;
    use super::interpreter::DynamicMessage;

    #[test]
    fn test_button_event_extraction() {
        let view: View<DynamicMessage> = View::Button {
            label: "Click Me".to_string(),
            onclick: DynamicMessage::String("button-clicked".to_string()),
            style: None,
        };

        let (vtree, router) = view_to_vtree_with_events(view);

        // 验证 VTree 结构
        assert_eq!(vtree.node_count(), 1);
        let root = vtree.root().unwrap();
        assert_eq!(root.kind, VNodeKind::Button);

        // 验证事件已注册
        assert!(router.has_handlers(root.id));

        // 验证可以获取事件消息
        let msg = router.on_click(root.id);
        assert!(msg.is_some());
        if let Some(DynamicMessage::String(s)) = msg {
            assert_eq!(s, "button-clicked");
        } else {
            panic!("Expected String message");
        }
    }

    #[test]
    fn test_column_with_button_events() {
        let view: View<DynamicMessage> = View::Column {
            children: vec![
                View::Button {
                    label: "Button 1".to_string(),
                    onclick: DynamicMessage::String("click-1".to_string()),
                    style: None,
                },
                View::Button {
                    label: "Button 2".to_string(),
                    onclick: DynamicMessage::String("click-2".to_string()),
                    style: None,
                },
            ],
            spacing: 10,
            padding: 0,
            style: None,
        };

        let (vtree, router) = view_to_vtree_with_events(view);

        // 验证树结构
        assert_eq!(vtree.node_count(), 3); // Column + 2 Buttons

        let root = vtree.root().unwrap();
        assert_eq!(root.children.len(), 2);

        // 验证两个按钮的事件都已注册
        let button1_id = root.children[0];
        let button2_id = root.children[1];

        assert!(router.has_handlers(button1_id));
        assert!(router.has_handlers(button2_id));

        // 验证事件消息不同
        let msg1 = router.on_click(button1_id).unwrap();
        let msg2 = router.on_click(button2_id).unwrap();

        if let (DynamicMessage::String(s1), DynamicMessage::String(s2)) = (msg1, msg2) {
            assert_eq!(s1, "click-1");
            assert_eq!(s2, "click-2");
        } else {
            panic!("Expected String messages");
        }
    }

    #[test]
    fn test_checkbox_toggle_event() {
        let view: View<DynamicMessage> = View::Checkbox {
            is_checked: true,
            label: "Check me".to_string(),
            on_toggle: Some(DynamicMessage::String("toggled".to_string())),
            style: None,
        };

        let (vtree, router) = view_to_vtree_with_events(view);

        let root = vtree.root().unwrap();

        // 验证 toggle 事件已注册
        assert!(router.has_handlers(root.id));

        let msg = router.on_toggle(root.id, true);
        assert!(msg.is_some());

        if let Some(DynamicMessage::String(s)) = msg {
            assert_eq!(s, "toggled");
        } else {
            panic!("Expected String message");
        }
    }
}
