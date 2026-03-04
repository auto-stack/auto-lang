//! VNode (Virtual Node) - 扁平化的视图表示
//!
//! VNode 将嵌套的 View<M> 树转换为扁平结构，通过 ID 引用建立父子关系。
//! 这解决了 GPUI Entity 系统无法嵌套不同类型组件的问题。
//!
//! ## 核心概念
//!
//! **扁平化结构**：将嵌套的 View<M> 树转换为扁平的 VNode 列表
//! **ID 引用**：使用 ID 引用替代直接嵌套，解耦视图结构与 GPUI Entity 类型
//! **增量更新**：支持精确的热重载和状态保留
//!
//! ## 使用示例
//!
//! ```ignore
//! use auto_ui::vnode::{VTree, VNode, VNodeKind, VNodeProps};
//!
//! // 创建虚拟节点树
//! let mut tree = VTree::new();
//! let root_id = tree.next_id();
//! let root = VNode::new(root_id, VNodeKind::Column, VNodeProps::Layout {
//!     spacing: 10,
//!     padding: 0
//! });
//! tree.set_root(root);
//! ```

use std::fmt;

/// VNode 唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VNodeId(u64);

impl VNodeId {
    /// 创建新的 VNode ID
    pub fn new(id: u64) -> Self {
        VNodeId(id)
    }

    /// 获取 ID 的数值表示
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for VNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VNode({})", self.0)
    }
}

/// VNode 类型枚举
///
/// 涵盖所有 AutoUI 支持的组件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VNodeKind {
    // ==================== 布局组件 ====================
    /// 垂直布局容器
    Column,

    /// 水平布局容器
    Row,

    /// 通用容器（支持 padding 和居中）
    Container,

    /// 可滚动容器
    Scrollable,

    /// 居中容器
    Center,

    // ==================== 基础组件 ====================
    /// 文本显示
    Text,

    /// 按钮
    Button,

    /// 文本输入框
    Input,

    /// 复选框
    Checkbox,

    /// 单选框
    Radio,

    /// 下拉选择框
    Select,

    // ==================== 高级组件 ====================
    /// 列表
    List,

    /// 表格
    Table,

    /// 滑块
    Slider,

    /// 进度条
    ProgressBar,

    /// 手风琴（可折叠面板）
    Accordion,

    /// 侧边栏
    Sidebar,

    /// 标签页
    Tabs,

    /// 导航栏
    NavigationRail,
}

impl fmt::Display for VNodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VNodeKind::Column => write!(f, "Column"),
            VNodeKind::Row => write!(f, "Row"),
            VNodeKind::Container => write!(f, "Container"),
            VNodeKind::Scrollable => write!(f, "Scrollable"),
            VNodeKind::Center => write!(f, "Center"),
            VNodeKind::Text => write!(f, "Text"),
            VNodeKind::Button => write!(f, "Button"),
            VNodeKind::Input => write!(f, "Input"),
            VNodeKind::Checkbox => write!(f, "Checkbox"),
            VNodeKind::Radio => write!(f, "Radio"),
            VNodeKind::Select => write!(f, "Select"),
            VNodeKind::List => write!(f, "List"),
            VNodeKind::Table => write!(f, "Table"),
            VNodeKind::Slider => write!(f, "Slider"),
            VNodeKind::ProgressBar => write!(f, "ProgressBar"),
            VNodeKind::Accordion => write!(f, "Accordion"),
            VNodeKind::Sidebar => write!(f, "Sidebar"),
            VNodeKind::Tabs => write!(f, "Tabs"),
            VNodeKind::NavigationRail => write!(f, "NavigationRail"),
        }
    }
}

/// VNode 属性
///
/// 每种 VNodeKind 对应不同的属性集合
#[derive(Debug, Clone)]
pub enum VNodeProps {
    /// 空属性（用于 Empty 或占位符）
    Empty,

    /// 文本属性
    Text { content: String },

    /// 按钮属性
    Button { label: String },

    /// 输入框属性
    Input {
        placeholder: String,
        value: String,
        password: bool,
    },

    /// 复选框属性
    Checkbox {
        label: String,
        is_checked: bool,
    },

    /// 单选框属性
    Radio {
        label: String,
        is_selected: bool,
    },

    /// 下拉选择框属性
    Select {
        options: Vec<String>,
        selected_index: Option<usize>,
    },

    /// 布局属性（用于 Column, Row）
    Layout {
        spacing: u16,
        padding: u16,
    },

    /// 容器属性
    Container {
        padding: u16,
        center_x: bool,
        center_y: bool,
    },

    /// 可滚动容器属性
    Scrollable,

    /// 滑块属性
    Slider {
        min: f32,
        max: f32,
        value: f32,
        step: Option<f32>,
    },

    /// 进度条属性
    ProgressBar {
        progress: f32,
    },

    /// 列表属性
    List {
        spacing: u16,
    },

    /// 表格属性
    Table {
        spacing: u16,
        col_spacing: u16,
    },
}

/// 虚拟节点 - 扁平表示的 View<M>
///
/// 每个节点包含：
/// - 唯一 ID
/// - 节点类型
/// - 父节点 ID（如果有）
/// - 子节点 ID 列表
/// - 节点属性
/// - 调试标签
#[derive(Debug, Clone)]
pub struct VNode {
    /// 节点唯一 ID
    pub id: VNodeId,

    /// 节点类型
    pub kind: VNodeKind,

    /// 父节点 ID（根节点为 None）
    pub parent: Option<VNodeId>,

    /// 子节点 ID 列表
    pub children: Vec<VNodeId>,

    /// 节点属性
    pub props: VNodeProps,

    /// 调试标签（用于日志和调试）
    pub label: String,
}

impl VNode {
    /// 创建新的 VNode
    pub fn new(id: VNodeId, kind: VNodeKind, props: VNodeProps) -> Self {
        Self {
            id,
            kind,
            parent: None,
            children: Vec::new(),
            props,
            label: String::new(),
        }
    }

    /// 设置调试标签
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// 设置父节点
    pub fn with_parent(mut self, parent: VNodeId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// 添加子节点
    pub fn add_child(&mut self, child_id: VNodeId) {
        self.children.push(child_id);
    }

    /// 获取子节点数量
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// 是否为叶子节点（无子节点）
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// 是否为根节点（无父节点）
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

/// 虚拟节点树 - 管理扁平的 VNode 列表
///
/// VTree 将嵌套的 View<M> 树转换为扁平结构，所有节点存储在 `nodes` 向量中，
/// 通过 ID 引用建立父子关系。
///
/// ## 结构优势
///
/// - **扁平存储**：所有节点在同一向量中，缓存友好
/// - **ID 引用**：避免深度递归，支持增量更新
/// - **高效查找**：通过 ID 快速定位节点
/// - **状态保留**：节点 ID 稳定，热重载时保留状态
#[derive(Debug, Clone)]
pub struct VTree {
    /// 所有节点（扁平存储）
    nodes: Vec<VNode>,

    /// 根节点 ID
    root: Option<VNodeId>,

    /// ID 计数器（用于生成唯一 ID）
    next_id: u64,
}

impl VTree {
    /// 创建新的空 VTree
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
            next_id: 1,
        }
    }

    /// 生成新的 VNode ID
    pub fn next_id(&mut self) -> VNodeId {
        let id = VNodeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// 设置根节点
    ///
    /// # 参数
    ///
    /// * `node` - 要设置为根的节点
    ///
    /// # 返回
    ///
    /// 根节点的 ID
    pub fn set_root(&mut self, node: VNode) -> VNodeId {
        let id = node.id;
        self.nodes.push(node);
        self.root = Some(id);
        id
    }

    /// 添加节点到树中
    ///
    /// # 参数
    ///
    /// * `node` - 要添加的节点
    ///
    /// # 返回
    ///
    /// 节点的 ID
    pub fn add_node(&mut self, node: VNode) -> VNodeId {
        let id = node.id;
        self.nodes.push(node);
        id
    }

    /// 获取节点
    ///
    /// # 参数
    ///
    /// * `id` - 节点 ID
    ///
    /// # 返回
    ///
    /// 如果找到则返回节点引用，否则返回 None
    pub fn get(&self, id: VNodeId) -> Option<&VNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// 获取可变节点
    ///
    /// # 参数
    ///
    /// * `id` - 节点 ID
    ///
    /// # 返回
    ///
    /// 如果找到则返回可变节点引用，否则返回 None
    pub fn get_mut(&mut self, id: VNodeId) -> Option<&mut VNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// 获取根节点
    ///
    /// # 返回
    ///
    /// 如果存在根节点则返回其引用，否则返回 None
    pub fn root(&self) -> Option<&VNode> {
        self.root.and_then(|id| self.get(id))
    }

    /// 获取可变根节点
    pub fn root_mut(&mut self) -> Option<&mut VNode> {
        self.root.and_then(|id| self.get_mut(id))
    }

    /// 获取子节点列表
    ///
    /// # 参数
    ///
    /// * `id` - 父节点 ID
    ///
    /// # 返回
    ///
    /// 如果找到父节点则返回其子节点列表，否则返回 None
    pub fn children(&self, id: VNodeId) -> Option<Vec<&VNode>> {
        self.get(id)?
            .children
            .iter()
            .map(|child_id| self.get(*child_id))
            .collect()
    }

    /// 获取所有节点
    pub fn nodes(&self) -> &[VNode] {
        &self.nodes
    }

    /// 获取可变所有节点
    pub fn nodes_mut(&mut self) -> &mut [VNode] {
        &mut self.nodes
    }

    /// 计算节点总数
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 检查树是否为空
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// 清空树
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root = None;
        self.next_id = 1;
    }

    /// 验证树的完整性
    ///
    /// 检查：
    /// - 所有父节点引用有效
    /// - 所有子节点引用有效
    /// - 不存在循环引用
    pub fn validate(&self) -> Result<(), String> {
        for node in &self.nodes {
            // 验证父节点引用
            if let Some(parent_id) = node.parent {
                if self.get(parent_id).is_none() {
                    return Err(format!("节点 {} 的父节点 {} 不存在", node.id, parent_id));
                }
            }

            // 验证子节点引用
            for child_id in &node.children {
                if self.get(*child_id).is_none() {
                    return Err(format!("节点 {} 的子节点 {} 不存在", node.id, child_id));
                }

                // 验证子节点的父节点是否指向当前节点
                let child = self.get(*child_id).unwrap();
                if child.parent != Some(node.id) {
                    return Err(format!(
                        "节点 {} 的子节点 {} 的父节点引用不正确",
                        node.id, child_id
                    ));
                }
            }
        }

        // TODO: 检测循环引用（需要深度优先搜索）

        Ok(())
    }

    /// 计算树的深度
    pub fn depth(&self) -> usize {
        fn compute_depth(tree: &VTree, node_id: VNodeId, current_depth: usize) -> usize {
            let node = match tree.get(node_id) {
                Some(n) => n,
                None => return current_depth,
            };

            if node.is_leaf() {
                return current_depth;
            }

            node.children
                .iter()
                .map(|&child_id| compute_depth(tree, child_id, current_depth + 1))
                .max()
                .unwrap_or(current_depth)
        }

        match self.root {
            Some(root_id) => compute_depth(self, root_id, 1),
            None => 0,
        }
    }

    /// 获取树的统计信息
    pub fn stats(&self) -> VTreeStats {
        let mut stats = VTreeStats::default();

        for node in &self.nodes {
            stats.total_nodes += 1;

            match node.kind {
                VNodeKind::Text => stats.text_nodes += 1,
                VNodeKind::Button => stats.button_nodes += 1,
                VNodeKind::Column | VNodeKind::Row => stats.layout_nodes += 1,
                _ => {}
            }

            if node.is_leaf() {
                stats.leaf_nodes += 1;
            }
        }

        stats.max_depth = self.depth();

        stats
    }
}

impl Default for VTree {
    fn default() -> Self {
        Self::new()
    }
}

/// VTree 统计信息
#[derive(Debug, Clone, Default)]
pub struct VTreeStats {
    /// 总节点数
    pub total_nodes: usize,

    /// 文本节点数
    pub text_nodes: usize,

    /// 按钮节点数
    pub button_nodes: usize,

    /// 布局节点数（Column, Row）
    pub layout_nodes: usize,

    /// 叶子节点数
    pub leaf_nodes: usize,

    /// 最大深度
    pub max_depth: usize,
}

impl fmt::Display for VTreeStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VTreeStats: 节点={}, 文本={}, 按钮={}, 布局={}, 叶子={}, 深度={}",
            self.total_nodes,
            self.text_nodes,
            self.button_nodes,
            self.layout_nodes,
            self.leaf_nodes,
            self.max_depth
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vnode_id() {
        let id = VNodeId::new(1);
        assert_eq!(id.as_u64(), 1);
        assert_eq!(id.to_string(), "VNode(1)");
    }

    #[test]
    fn test_vtree_creation() {
        let tree = VTree::new();
        assert!(tree.root().is_none());
        assert_eq!(tree.node_count(), 0);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_vtree_default() {
        let tree = VTree::default();
        assert!(tree.is_empty());
        assert_eq!(tree.node_count(), 0);
    }

    #[test]
    fn test_vtree_next_id() {
        let mut tree = VTree::new();
        let id1 = tree.next_id();
        let id2 = tree.next_id();
        let id3 = tree.next_id();

        assert_eq!(id1.as_u64(), 1);
        assert_eq!(id2.as_u64(), 2);
        assert_eq!(id3.as_u64(), 3);
    }

    #[test]
    fn test_vtree_add_root() {
        let mut tree = VTree::new();
        let id = tree.next_id();
        let node = VNode::new(id, VNodeKind::Text, VNodeProps::Text {
            content: "Hello".to_string()
        });

        tree.set_root(node);

        assert!(tree.root().is_some());
        assert_eq!(tree.node_count(), 1);
        assert_eq!(tree.root().unwrap().id, id);
    }

    #[test]
    fn test_vtree_add_node() {
        let mut tree = VTree::new();
        let id = tree.next_id();
        let node = VNode::new(id, VNodeKind::Text, VNodeProps::Text {
            content: "Test".to_string()
        });

        tree.add_node(node);

        assert_eq!(tree.node_count(), 1);
        assert!(tree.get(id).is_some());
    }

    #[test]
    fn test_vtree_get() {
        let mut tree = VTree::new();
        let id = tree.next_id();
        let node = VNode::new(id, VNodeKind::Button, VNodeProps::Button {
            label: "Click".to_string()
        });

        tree.set_root(node);

        let retrieved = tree.get(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);

        assert!(tree.get(VNodeId(999)).is_none());
    }

    #[test]
    fn test_vtree_get_mut() {
        let mut tree = VTree::new();
        let id = tree.next_id();
        let node = VNode::new(id, VNodeKind::Text, VNodeProps::Text {
            content: "Original".to_string()
        });

        tree.set_root(node);

        if let Some(node_mut) = tree.get_mut(id) {
            if let VNodeProps::Text { content } = &mut node_mut.props {
                *content = "Modified".to_string();
            }
        }

        let retrieved = tree.get(id).unwrap();
        if let VNodeProps::Text { content } = &retrieved.props {
            assert_eq!(content, "Modified");
        }
    }

    #[test]
    fn test_vtree_parent_child() {
        let mut tree = VTree::new();

        // 创建根节点
        let root_id = tree.next_id();
        let mut root = VNode::new(root_id, VNodeKind::Column, VNodeProps::Layout {
            spacing: 10,
            padding: 0
        });

        // 创建子节点
        let child_id = tree.next_id();
        let child = VNode::new(child_id, VNodeKind::Text, VNodeProps::Text {
            content: "Child".to_string()
        })
        .with_parent(root_id);

        root.add_child(child_id);
        tree.set_root(root);
        tree.add_node(child);

        // 验证父子关系
        let root_node = tree.get(root_id).unwrap();
        assert_eq!(root_node.children.len(), 1);
        assert_eq!(root_node.children[0], child_id);
        assert_eq!(root_node.child_count(), 1);
        assert!(!root_node.is_leaf());
        assert!(root_node.is_root());

        let child_node = tree.get(child_id).unwrap();
        assert_eq!(child_node.parent, Some(root_id));
        assert!(child_node.is_leaf());
        assert!(!child_node.is_root());
    }

    #[test]
    fn test_vtree_children() {
        let mut tree = VTree::new();

        let root_id = tree.next_id();
        let mut root = VNode::new(root_id, VNodeKind::Row, VNodeProps::Layout {
            spacing: 5,
            padding: 0
        });

        // 添加多个子节点
        for i in 1..=3 {
            let child_id = tree.next_id();
            let child = VNode::new(
                child_id,
                VNodeKind::Text,
                VNodeProps::Text {
                    content: format!("Child{}", i),
                },
            )
            .with_parent(root_id);

            tree.add_node(child);
            root.add_child(child_id);
        }

        tree.set_root(root);

        // 验证子节点列表
        let children = tree.children(root_id).unwrap();
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn test_vtree_validate() {
        let mut tree = VTree::new();

        let root_id = tree.next_id();
        let mut root = VNode::new(root_id, VNodeKind::Column, VNodeProps::Layout {
            spacing: 10,
            padding: 0,
        });

        let child_id = tree.next_id();
        let child = VNode::new(child_id, VNodeKind::Text, VNodeProps::Text {
            content: "Child".to_string()
        })
        .with_parent(root_id);

        root.add_child(child_id);
        tree.set_root(root);
        tree.add_node(child);

        // 验证应该成功
        assert!(tree.validate().is_ok());
    }

    #[test]
    fn test_vtree_validate_invalid_parent() {
        let mut tree = VTree::new();

        let id = tree.next_id();
        let node = VNode::new(id, VNodeKind::Text, VNodeProps::Text {
            content: "Orphan".to_string()
        })
        .with_parent(VNodeId(999)); // 不存在的父节点

        tree.add_node(node);

        // 验证应该失败
        assert!(tree.validate().is_err());
    }

    #[test]
    fn test_vtree_depth() {
        let mut tree = VTree::new();

        // 构建三层结构: Root -> Child -> Grandchild
        let root_id = tree.next_id();
        let mut root = VNode::new(root_id, VNodeKind::Column, VNodeProps::Layout {
            spacing: 10,
            padding: 0,
        });

        let child_id = tree.next_id();
        let mut child = VNode::new(child_id, VNodeKind::Row, VNodeProps::Layout {
            spacing: 5,
            padding: 0,
        })
        .with_parent(root_id);

        let grandchild_id = tree.next_id();
        let grandchild = VNode::new(
            grandchild_id,
            VNodeKind::Text,
            VNodeProps::Text {
                content: "Grandchild".to_string()
            }
        )
        .with_parent(child_id);

        child.add_child(grandchild_id);
        root.add_child(child_id);

        tree.set_root(root);
        tree.add_node(child);
        tree.add_node(grandchild);

        assert_eq!(tree.depth(), 3);
    }

    #[test]
    fn test_vtree_stats() {
        let mut tree = VTree::new();

        let root_id = tree.next_id();
        let mut root = VNode::new(root_id, VNodeKind::Column, VNodeProps::Layout {
            spacing: 10,
            padding: 0,
        });

        // 添加文本节点
        let text_id = tree.next_id();
        let text = VNode::new(
            text_id,
            VNodeKind::Text,
            VNodeProps::Text {
                content: "Hello".to_string()
            }
        )
        .with_parent(root_id);

        // 添加按钮节点
        let button_id = tree.next_id();
        let button = VNode::new(
            button_id,
            VNodeKind::Button,
            VNodeProps::Button {
                label: "Click".to_string()
            }
        )
        .with_parent(root_id);

        root.add_child(text_id);
        root.add_child(button_id);

        tree.set_root(root);
        tree.add_node(text);
        tree.add_node(button);

        let stats = tree.stats();
        assert_eq!(stats.total_nodes, 3);
        assert_eq!(stats.text_nodes, 1);
        assert_eq!(stats.button_nodes, 1);
        assert_eq!(stats.layout_nodes, 1);
        assert_eq!(stats.leaf_nodes, 2);
        assert_eq!(stats.max_depth, 2);
    }

    #[test]
    fn test_vtree_clear() {
        let mut tree = VTree::new();

        let id = tree.next_id();
        let node = VNode::new(id, VNodeKind::Text, VNodeProps::Text {
            content: "Test".to_string()
        });

        tree.set_root(node);
        assert_eq!(tree.node_count(), 1);

        tree.clear();
        assert!(tree.is_empty());
        assert!(tree.root().is_none());
        assert_eq!(tree.next_id().as_u64(), 1); // ID 重置为 1
    }

    #[test]
    fn test_vnode_kind_display() {
        assert_eq!(VNodeKind::Text.to_string(), "Text");
        assert_eq!(VNodeKind::Button.to_string(), "Button");
        assert_eq!(VNodeKind::Column.to_string(), "Column");
    }

    #[test]
    fn test_vnode_with_label() {
        let id = VNodeId::new(1);
        let node = VNode::new(id, VNodeKind::Text, VNodeProps::Text {
            content: "Test".to_string()
        })
        .with_label("MyTextNode");

        assert_eq!(node.label, "MyTextNode");
    }
}
