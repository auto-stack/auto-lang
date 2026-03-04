//! 事件路由器 - VNode 事件到消息的转换
//!
//! 此模块提供从 VNode 事件到 DynamicMessage 的路由机制。
//!
//! ## 核心功能
//!
//! - 注册事件处理器（onClick, onChange, onToggle 等）
//! - 从 VNode 接收事件
//! - 将事件转换为 DynamicMessage
//! - 路由到解释器
//!
//! ## 使用示例
//!
//! ```ignore
//! use auto_ui::event_router::EventRouter;
//! use auto_ui::interpreter::DynamicMessage;
//!
//! let mut router = EventRouter::new();
//!
//! // 注册按钮点击事件
//! router.register_click(button_id, |context| {
//!     DynamicMessage::String("clicked".to_string())
//! });
//!
//! // 触发事件
//! router.on_click(node_id, context);
//! ```

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// 导入 VNodeId
use super::vnode::VNodeId;

// 导入 DynamicMessage（仅在 interpreter feature 启用时）
#[cfg(feature = "ui-interpreter")]
use super::interpreter::DynamicMessage;

// 为非 interpreter 模式定义一个占位符类型
#[cfg(not(feature = "ui-interpreter"))]
#[derive(Debug, Clone)]
pub enum DynamicMessage {
#[derive(Debug, Clone)]
pub enum DynamicMessage {
    String(String),
}

/// 事件上下文 - 提供事件处理时的上下文信息
#[derive(Debug, Clone)]
pub struct EventContext {
    /// 节点 ID
    pub node_id: VNodeId,

    /// 事件类型
    pub event_type: EventType,

    /// 事件数据（可选）
    pub data: Option<String>,
}

/// 事件类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    /// 点击事件
    Click,

    /// 输入变更事件
    Change(String),

    /// 切换事件
    Toggle(bool),

    /// 选择事件
    Select(usize),
}

/// 事件处理器回调
pub type EventHandler = dyn Fn(EventContext) -> DynamicMessage + Send + Sync;

/// 事件路由器
///
/// 负责管理事件处理器并将事件路由到正确的处理器。
pub struct EventRouter {
    /// 点击事件处理器
    click_handlers: HashMap<VNodeId, Arc<EventHandler>>,

    /// 输入变更事件处理器
    change_handlers: HashMap<VNodeId, Arc<EventHandler>>,

    /// 切换事件处理器
    toggle_handlers: HashMap<VNodeId, Arc<EventHandler>>,

    /// 选择事件处理器
    select_handlers: HashMap<VNodeId, Arc<EventHandler>>,
}

impl EventRouter {
    /// 创建新的事件路由器
    pub fn new() -> Self {
        Self {
            click_handlers: HashMap::new(),
            change_handlers: HashMap::new(),
            toggle_handlers: HashMap::new(),
            select_handlers: HashMap::new(),
        }
    }

    /// 注册点击事件处理器
    pub fn register_click<F>(&mut self, node_id: VNodeId, handler: F)
    where
        F: Fn(EventContext) -> DynamicMessage + Send + Sync + 'static,
    {
        self.click_handlers.insert(node_id, Arc::new(handler));
    }

    /// 注册输入变更事件处理器
    pub fn register_change<F>(&mut self, node_id: VNodeId, handler: F)
    where
        F: Fn(EventContext) -> DynamicMessage + Send + Sync + 'static,
    {
        self.change_handlers.insert(node_id, Arc::new(handler));
    }

    /// 注册切换事件处理器
    pub fn register_toggle<F>(&mut self, node_id: VNodeId, handler: F)
    where
        F: Fn(EventContext) -> DynamicMessage + Send + Sync + 'static,
    {
        self.toggle_handlers.insert(node_id, Arc::new(handler));
    }

    /// 注册选择事件处理器
    pub fn register_select<F>(&mut self, node_id: VNodeId, handler: F)
    where
        F: Fn(EventContext) -> DynamicMessage + Send + Sync + 'static,
    {
        self.select_handlers.insert(node_id, Arc::new(handler));
    }

    /// 处理点击事件
    pub fn on_click(&self, node_id: VNodeId) -> Option<DynamicMessage> {
        if let Some(handler) = self.click_handlers.get(&node_id) {
            let context = EventContext {
                node_id,
                event_type: EventType::Click,
                data: None,
            };
            Some(handler(context))
        } else {
            None
        }
    }

    /// 处理输入变更事件
    pub fn on_change(&self, node_id: VNodeId, value: String) -> Option<DynamicMessage> {
        if let Some(handler) = self.change_handlers.get(&node_id) {
            let context = EventContext {
                node_id,
                event_type: EventType::Change(value.clone()),
                data: Some(value),
            };
            Some(handler(context))
        } else {
            None
        }
    }

    /// 处理切换事件
    pub fn on_toggle(&self, node_id: VNodeId, is_checked: bool) -> Option<DynamicMessage> {
        if let Some(handler) = self.toggle_handlers.get(&node_id) {
            let context = EventContext {
                node_id,
                event_type: EventType::Toggle(is_checked),
                data: Some(is_checked.to_string()),
            };
            Some(handler(context))
        } else {
            None
        }
    }

    /// 处理选择事件
    pub fn on_select(&self, node_id: VNodeId, index: usize) -> Option<DynamicMessage> {
        if let Some(handler) = self.select_handlers.get(&node_id) {
            let context = EventContext {
                node_id,
                event_type: EventType::Select(index),
                data: Some(index.to_string()),
            };
            Some(handler(context))
        } else {
            None
        }
    }

    /// 清除所有事件处理器
    pub fn clear(&mut self) {
        self.click_handlers.clear();
        self.change_handlers.clear();
        self.toggle_handlers.clear();
        self.select_handlers.clear();
    }

    /// 移除特定节点的事件处理器
    pub fn remove_node(&mut self, node_id: VNodeId) {
        self.click_handlers.remove(&node_id);
        self.change_handlers.remove(&node_id);
        self.toggle_handlers.remove(&node_id);
        self.select_handlers.remove(&node_id);
    }

    /// 检查节点是否有事件处理器
    pub fn has_handlers(&self, node_id: VNodeId) -> bool {
        self.click_handlers.contains_key(&node_id)
            || self.change_handlers.contains_key(&node_id)
            || self.toggle_handlers.contains_key(&node_id)
            || self.select_handlers.contains_key(&node_id)
    }
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// 线程安全的事件路由器包装器
///
/// 使用 RwLock 包装 EventRouter 以支持多线程访问。
pub struct SharedEventRouter {
    router: Arc<RwLock<EventRouter>>,
}

impl SharedEventRouter {
    /// 创建新的共享事件路由器
    pub fn new() -> Self {
        Self {
            router: Arc::new(RwLock::new(EventRouter::new())),
        }
    }

    /// 注册点击事件处理器
    pub fn register_click<F>(&self, node_id: VNodeId, handler: F)
    where
        F: Fn(EventContext) -> DynamicMessage + Send + Sync + 'static,
    {
        if let Ok(mut router) = self.router.write() {
            router.register_click(node_id, handler);
        }
    }

    /// 处理点击事件
    pub fn on_click(&self, node_id: VNodeId) -> Option<DynamicMessage> {
        if let Ok(router) = self.router.read() {
            router.on_click(node_id)
        } else {
            None
        }
    }

    /// 处理输入变更事件
    pub fn on_change(&self, node_id: VNodeId, value: String) -> Option<DynamicMessage> {
        if let Ok(router) = self.router.read() {
            router.on_change(node_id, value)
        } else {
            None
        }
    }

    /// 处理切换事件
    pub fn on_toggle(&self, node_id: VNodeId, is_checked: bool) -> Option<DynamicMessage> {
        if let Ok(router) = self.router.read() {
            router.on_toggle(node_id, is_checked)
        } else {
            None
        }
    }

    /// 处理选择事件
    pub fn on_select(&self, node_id: VNodeId, index: usize) -> Option<DynamicMessage> {
        if let Ok(router) = self.router.read() {
            router.on_select(node_id, index)
        } else {
            None
        }
    }

    /// 清除所有事件处理器
    pub fn clear(&self) {
        if let Ok(mut router) = self.router.write() {
            router.clear();
        }
    }

    /// 移除特定节点的事件处理器
    pub fn remove_node(&self, node_id: VNodeId) {
        if let Ok(mut router) = self.router.write() {
            router.remove_node(node_id);
        }
    }

    /// 克隆 Arc（用于多线程共享）
    pub fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
        }
    }
}

impl Default for SharedEventRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_router_creation() {
        let router = EventRouter::new();
        assert!(!router.has_handlers(VNodeId::new(1)));
    }

    #[test]
    fn test_register_click() {
        let mut router = EventRouter::new();
        let node_id = VNodeId::new(1);

        router.register_click(node_id, |ctx| {
            DynamicMessage::String(format!("clicked: {}", ctx.node_id.as_u64()))
        });

        assert!(router.has_handlers(node_id));
    }

    #[test]
    fn test_on_click() {
        let mut router = EventRouter::new();
        let node_id = VNodeId::new(1);

        router.register_click(node_id, |_ctx| DynamicMessage::String("clicked".to_string()));

        let result = router.on_click(node_id);
        assert!(result.is_some());

        if let Some(DynamicMessage::String(msg)) = result {
            assert_eq!(msg, "clicked");
        } else {
            panic!("Expected String message");
        }
    }

    #[test]
    fn test_clear() {
        let mut router = EventRouter::new();
        let node_id = VNodeId::new(1);

        router.register_click(node_id, |_ctx| DynamicMessage::String("test".to_string()));
        assert!(router.has_handlers(node_id));

        router.clear();
        assert!(!router.has_handlers(node_id));
    }

    #[test]
    fn test_remove_node() {
        let mut router = EventRouter::new();
        let node_id = VNodeId::new(1);

        router.register_click(node_id, |_ctx| DynamicMessage::String("test".to_string()));
        assert!(router.has_handlers(node_id));

        router.remove_node(node_id);
        assert!(!router.has_handlers(node_id));
    }

    #[test]
    fn test_shared_event_router() {
        let router = SharedEventRouter::new();
        let node_id = VNodeId::new(1);

        router.register_click(node_id, |_ctx| DynamicMessage::String("test".to_string()));

        let result = router.on_click(node_id);
        assert!(result.is_some());
    }
}
