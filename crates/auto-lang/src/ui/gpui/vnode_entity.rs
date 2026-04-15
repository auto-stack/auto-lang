//! VNode Entity - GPUI 渲染器
//!
//! 此模块提供 VTree 的 GPUI 渲染支持，通过单一 Entity 渲染整个扁平化的 VNode 树。
//!
//! ## 核心设计
//!
//! **VNodeEntity** 是一个单一的 GPUI Entity，它：
//! 1. 持有 VTree（扁平化的节点树）
//! 2. 递归渲染 VNode 树为 GPUI div 元素
//! 3. 不需要嵌套其他 Entity，避免 Context<T> 类型限制
//! 4. 支持事件处理回调
//!
//! ## 使用示例
//!
//! ```ignore
//! use auto_lang::ui::vnode_converter::view_to_vtree;
//! use auto_lang::ui::gpui::vnode_entity::VNodeEntity;
//!
//! let view = View::col()
//!     .child(View::text("Hello"))
//!     .build();
//!
//! let vtree = view_to_vtree(view);
//!
//! // 在 GPUI 中使用
//! let entity = cx.new(|cx| VNodeEntity::new(vtree, cx));
//! ```

use gpui_lib::{prelude::*, InteractiveElement, *};
use std::sync::Arc;

// 导入 VNode 类型
use crate::ui::vnode::{VNodeId, VNodeKind, VNodeProps, VTree};

#[cfg(feature = "ui-interpreter")]
use crate::ui::interpreter::DynamicMessage;

#[cfg(feature = "ui-interpreter")]
use crate::ui::event_router::EventType;

/// VNode Entity - GPUI 渲染器
///
/// 这个 Entity 持有并渲染整个 VTree，不依赖嵌套的 Entity。
pub struct VNodeEntity {
    /// 虚拟节点树
    vtree: VTree,

    /// 焦点句柄
    focus_handle: FocusHandle,

    /// 错误信息（如果有）
    error: Option<String>,
}

impl VNodeEntity {
    /// 创建新的 VNode Entity
    pub fn new(vtree: VTree, _cx: &mut Context<Self>) -> Self {
        let focus_handle = _cx.focus_handle();

        Self {
            vtree,
            focus_handle,
            error: None,
        }
    }

    /// 获取 VTree 的引用
    pub fn vtree(&self) -> &VTree {
        &self.vtree
    }

    /// 获取 VTree 的可变引用
    pub fn vtree_mut(&mut self) -> &mut VTree {
        &mut self.vtree
    }

    /// 更新 VTree
    pub fn update_vtree(&mut self, vtree: VTree, cx: &mut Context<Self>) {
        self.vtree = vtree;
        cx.notify();
    }

    /// 渲染单个 VNode 为 GPUI 元素
    fn render_vnode(&self, node_id: VNodeId, cx: &mut Context<Self>) -> AnyElement {
        let node = match self.vtree.get(node_id) {
            Some(n) => n,
            None => {
                return div()
                    .text_color(rgb(0xff6b6b))
                    .child(format!("❌ 节点 {} 不存在", node_id))
                    .into_any()
            }
        };

        match &node.kind {
            VNodeKind::Text => self.render_text(node),
            VNodeKind::Button => self.render_button(node, cx),
            VNodeKind::Column => self.render_column(node, cx),
            VNodeKind::Row => self.render_row(node, cx),
            VNodeKind::Container => self.render_container(node, cx),
            VNodeKind::Scrollable => self.render_scrollable(node, cx),
            VNodeKind::Input => self.render_input(node, cx),
            VNodeKind::Checkbox => self.render_checkbox(node, cx),
            VNodeKind::Radio => self.render_radio(node, cx),
            VNodeKind::Select => self.render_select(node),
            VNodeKind::List => self.render_list(node, cx),
            VNodeKind::Table => self.render_table(node, cx),
            VNodeKind::Slider => self.render_slider(node),
            VNodeKind::ProgressBar => self.render_progress_bar(node),
            VNodeKind::Center => self.render_center(node, cx),
            // 高级组件占位符
            VNodeKind::Accordion | VNodeKind::Sidebar | VNodeKind::Tabs | VNodeKind::NavigationRail => {
                self.render_placeholder(node)
            }
        }
    }

    /// 渲染文本节点
    fn render_text(&self, node: &crate::ui::vnode::VNode) -> AnyElement {
        let content = match &node.props {
            VNodeProps::Text { content } => content.clone(),
            VNodeProps::Empty => String::new(),
            _ => String::from("(无效的文本属性)"),
        };

        div()
            .text_sm()
            .child(content)
            .into_any()
    }

    /// 渲染按钮节点
    fn render_button(&self, node: &crate::ui::vnode::VNode, _cx: &mut Context<Self>) -> AnyElement {
        let label = match &node.props {
            VNodeProps::Button { label } => label.clone(),
            _ => String::from("Button"),
        };

        div()
            .px_4()
            .py_2()
            .bg(rgb(0x3b82f6))
            .hover(|div| div.bg(rgb(0x2563eb)))
            .border_1()
            .border_color(rgb(0x1d4ed8))
            .rounded_md()
            .cursor_pointer()
            .child(label)
            .into_any()
    }

    /// 渲染列布局节点
    fn render_column(&self, node: &crate::ui::vnode::VNode, cx: &mut Context<Self>) -> AnyElement {
        let (spacing, padding) = match &node.props {
            VNodeProps::Layout { spacing, padding } => (*spacing, *padding),
            _ => (10, 0),
        };

        let mut col = div()
            .flex()
            .flex_col()
            .gap(px(spacing as f32))
            .p(px(padding as f32));

        // 递归渲染子节点
        for child_id in &node.children {
            col = col.child(self.render_vnode(*child_id, cx));
        }

        col.into_any()
    }

    /// 渲染行布局节点
    fn render_row(&self, node: &crate::ui::vnode::VNode, cx: &mut Context<Self>) -> AnyElement {
        let (spacing, padding) = match &node.props {
            VNodeProps::Layout { spacing, padding } => (*spacing, *padding),
            _ => (10, 0),
        };

        let mut row = div()
            .flex()
            .flex_row()
            .gap(px(spacing as f32))
            .p(px(padding as f32));

        // 递归渲染子节点
        for child_id in &node.children {
            row = row.child(self.render_vnode(*child_id, cx));
        }

        row.into_any()
    }

    /// 渲染容器节点
    fn render_container(&self, node: &crate::ui::vnode::VNode, cx: &mut Context<Self>) -> AnyElement {
        let (padding, center_x, center_y) = match &node.props {
            VNodeProps::Container {
                padding,
                center_x,
                center_y,
            } => (*padding, *center_x, *center_y),
            _ => (0, false, false),
        };

        let mut container = div().flex().p(px(padding as f32));

        if center_x {
            container = container.items_center();
        }
        if center_y {
            container = container.justify_center();
        }

        // 渲染子节点（容器只有一个子节点）
        if let Some(child_id) = node.children.first() {
            container = container.child(self.render_vnode(*child_id, cx));
        }

        container.into_any()
    }

    /// 渲染滚动容器节点
    fn render_scrollable(&self, node: &crate::ui::vnode::VNode, cx: &mut Context<Self>) -> AnyElement {
        let mut scrollable = div()
            .flex()
            .flex_col()
            .size_full();

        // 渲染子节点
        if let Some(child_id) = node.children.first() {
            scrollable = scrollable.child(self.render_vnode(*child_id, cx));
        }

        scrollable.into_any()
    }

    /// 渲染输入框节点
    fn render_input(&self, node: &crate::ui::vnode::VNode, _cx: &mut Context<Self>) -> AnyElement {
        let (placeholder, value, _password) = match &node.props {
            VNodeProps::Input {
                placeholder,
                value,
                password,
            } => (placeholder.clone(), value.clone(), *password),
            _ => (String::new(), String::new(), false),
        };

        // 简化版本：显示当前值
        let display_text = if value.is_empty() {
            placeholder.clone()
        } else {
            value.clone()
        };

        div()
            .px_3()
            .py_2()
            .bg(rgb(0x2a2a2a))
            .border_1()
            .border_color(rgb(0x4a4a4a))
            .rounded_md()
            .text_sm()
            .child(display_text)
            .into_any()
    }

    /// 渲染复选框节点
    fn render_checkbox(&self, node: &crate::ui::vnode::VNode, _cx: &mut Context<Self>) -> AnyElement {
        let (label, is_checked) = match &node.props {
            VNodeProps::Checkbox { label, is_checked } => (label.clone(), *is_checked),
            _ => (String::new(), false),
        };

        div()
            .flex()
            .items_center()
            .gap_2()
            .cursor_pointer()
            .child(
                div()
                    .w_4()
                    .h_4()
                    .border_1()
                    .border_color(if is_checked {
                        rgb(0x3b82f6)
                    } else {
                        rgb(0x6c6c6c)
                    })
                    .bg(if is_checked {
                        rgb(0x3b82f6)
                    } else {
                        rgb(0x2a2a2a)
                    })
                    .rounded_sm(),
            )
            .child(label)
            .into_any()
    }

    /// 渲染单选框节点
    fn render_radio(&self, node: &crate::ui::vnode::VNode, _cx: &mut Context<Self>) -> AnyElement {
        let (label, is_selected) = match &node.props {
            VNodeProps::Radio { label, is_selected } => (label.clone(), *is_selected),
            _ => (String::new(), false),
        };

        div()
            .flex()
            .items_center()
            .gap_2()
            .cursor_pointer()
            .child(
                div()
                    .w_4()
                    .h_4()
                    .border_1()
                    .border_color(if is_selected {
                        rgb(0x3b82f6)
                    } else {
                        rgb(0x6c6c6c)
                    })
                    .rounded_full(),
            )
            .child(label)
            .into_any()
    }

    /// 渲染选择框节点
    fn render_select(&self, node: &crate::ui::vnode::VNode) -> AnyElement {
        let (options, selected_index) = match &node.props {
            VNodeProps::Select {
                options,
                selected_index,
            } => (options.clone(), *selected_index),
            _ => (vec![], None),
        };

        let selected = selected_index
            .and_then(|i| options.get(i))
            .cloned()
            .unwrap_or_else(|| "Select...".to_string());

        div()
            .px_3()
            .py_2()
            .bg(rgb(0x2a2a2a))
            .border_1()
            .border_color(rgb(0x4a4a4a))
            .rounded_md()
            .text_sm()
            .child(selected)
            .into_any()
    }

    /// 渲染列表节点
    fn render_list(&self, node: &crate::ui::vnode::VNode, cx: &mut Context<Self>) -> AnyElement {
        let spacing = match &node.props {
            VNodeProps::List { spacing } => *spacing,
            _ => 8,
        };

        let mut list = div().flex().flex_col().gap(px(spacing as f32));

        // 递归渲染子节点
        for child_id in &node.children {
            list = list.child(self.render_vnode(*child_id, cx));
        }

        list.into_any()
    }

    /// 渲染表格节点
    fn render_table(&self, node: &crate::ui::vnode::VNode, cx: &mut Context<Self>) -> AnyElement {
        let (spacing, col_spacing) = match &node.props {
            VNodeProps::Table {
                spacing,
                col_spacing,
            } => (*spacing, *col_spacing),
            _ => (5, 10),
        };

        let mut table = div().flex().flex_col().gap(px(spacing as f32));

        // 递归渲染所有子节点（headers + rows）
        for child_id in &node.children {
            let child = match self.vtree.get(*child_id) {
                Some(c) => c,
                None => continue,
            };

            // 为每一行创建 flex_row
            let mut row_div = div().flex().flex_row().gap(px(col_spacing as f32));

            // 渲染该行的子节点（单元格）
            for cell_id in &child.children {
                row_div = row_div.child(self.render_vnode(*cell_id, cx));
            }

            table = table.child(row_div);
        }

        table.into_any()
    }

    /// 渲染滑块节点
    fn render_slider(&self, node: &crate::ui::vnode::VNode) -> AnyElement {
        let (min, max, value, _step) = match &node.props {
            VNodeProps::Slider {
                min,
                max,
                value,
                step,
            } => (*min, *max, *value, *step),
            _ => (0.0, 100.0, 50.0, None),
        };

        let range = max - min;
        let percentage = ((value - min) / range).clamp(0.0, 1.0);

        div()
            .h(px(16.0))
            .w(px(300.0))
            .relative()
            .child(
                div()
                    .absolute()
                    .left(px(0.0))
                    .top(px(6.0))
                    .h(px(4.0))
                    .w(px(300.0))
                    .bg(rgb(0x333333))
                    .rounded_md(),
            )
            .child(
                div()
                    .absolute()
                    .left(px(0.0))
                    .top(px(6.0))
                    .h(px(4.0))
                    .w(px(percentage * 300.0))
                    .bg(rgb(0x3b82f6))
                    .rounded_md(),
            )
            .child(
                div()
                    .absolute()
                    .left(px(percentage * 300.0 - 8.0))
                    .top(px(0.0))
                    .w(px(16.0))
                    .h(px(16.0))
                    .bg(rgb(0xffffff))
                    .rounded_full()
                    .border_2()
                    .border_color(rgb(0x3b82f6))
                    .shadow_lg(),
            )
            .into_any()
    }

    /// 渲染进度条节点
    fn render_progress_bar(&self, node: &crate::ui::vnode::VNode) -> AnyElement {
        let progress = match &node.props {
            VNodeProps::ProgressBar { progress } => *progress,
            _ => 0.0,
        };

        let filled_width = (progress * 200.0) as f32;

        div()
            .w(px(200.0))
            .h(px(20.0))
            .bg(rgb(0x222222))
            .border_1()
            .border_color(rgb(0x444444))
            .child(div().w(px(filled_width)).h(px(20.0)).bg(rgb(0x3b82f6)))
            .into_any()
    }

    /// 渲染居中容器节点
    fn render_center(&self, node: &crate::ui::vnode::VNode, cx: &mut Context<Self>) -> AnyElement {
        let mut center = div().flex().items_center().justify_center().size_full();

        // 渲染子节点
        if let Some(child_id) = node.children.first() {
            center = center.child(self.render_vnode(*child_id, cx));
        }

        center.into_any()
    }

    /// 渲染高级组件占位符
    fn render_placeholder(&self, node: &crate::ui::vnode::VNode) -> AnyElement {
        let name = format!("{:?}", node.kind);

        div()
            .text_color(rgb(0xf59e0b))
            .child(format!("🔧 {} 组件暂未实现", name))
            .into_any()
    }
}

impl Focusable for VNodeEntity {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for VNodeEntity {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 如果有错误，显示错误信息
        if let Some(ref error) = self.error {
            return div()
                .size_full()
                .bg(rgb(0x1a1a1a))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .p_4()
                        .bg(rgb(0x3c1a1a))
                        .text_color(rgb(0xff6b6b))
                        .child(format!("❌ {}", error)),
                );
        }

        // 获取根节点并渲染
        match self.vtree.root() {
            Some(root) => {
                // 验证树结构
                if let Err(e) = self.vtree.validate() {
                    self.error = Some(format!("树结构无效: {}", e));
                    cx.notify();
                    return div()
                        .size_full()
                        .bg(rgb(0x1a1a1a))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(0xff6b6b))
                        .child("❌ 树结构验证失败");
                }

                // 递归渲染整棵树 - 将 AnyElement 包装为 Div
                div().child(self.render_vnode(root.id, cx))
            }
            None => div()
                .size_full()
                .bg(rgb(0x1a1a1a))
                .flex()
                .items_center()
                .justify_center()
                .child("⏳ 空树"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::vnode::{VNode, VNodeKind, VNodeProps};

    #[test]
    fn test_vnode_entity_creation() {
        // 创建一个简单的 VTree
        let mut vtree = VTree::new();
        let id = vtree.next_id();

        let node = VNode::new(
            id,
            VNodeKind::Text,
            VNodeProps::Text {
                content: "Hello".to_string(),
            },
        );

        vtree.set_root(node);

        // 注意：这个测试只是编译通过，实际运行需要 GPUI 环境
        // 在实际测试中，我们不需要创建 Context，因为 VNodeEntity::new 需要它
    }
}
