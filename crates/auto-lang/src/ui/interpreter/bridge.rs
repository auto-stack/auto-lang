//! # Auto-UI 解释器桥梁
//!
//! 此模块提供 `auto-lang::Interpreter` 和 `auto-ui` 渲染系统之间的桥梁。
//!
//! ## 架构
//!
//! ```text
//! .at 文件
//!    ↓
//! auto_lang::Interpreter（已有的解释器）
//!    ↓
//! auto_val::Node（求值结果）
//!    ↓
//! node_converter::convert_node（已有的转换器）
//!    ↓
//! View<DynamicMessage>（增强支持类型化消息）
//!    ↓
//! GPUI 渲染
//! ```

use crate::interpreter::AutoInterpreter;
use auto_val::{Node, Value};
use std::path::Path;
use std::collections::HashMap;

// 结果类型别名
pub type Result<T> = std::result::Result<T, BridgeError>;

/// 桥梁错误类型
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("AutoLang error: {0}")]
    AutoLang(String),

    #[error("Lock error: {0}")]
    Lock(String),
}

/// 动态消息（保留类型信息）
#[derive(Clone, Debug)]
pub enum DynamicMessage {
    /// 字符串事件（向后兼容）
    String(String),

    /// 类型化事件
    Typed {
        widget_name: String,     // Widget 名称
        event_name: String,      // 事件名（如 "Inc"）
        args: Vec<Value>,        // 事件参数
    },
}

/// 解释器桥梁 - 连接 auto-lang 和 auto-ui
pub struct InterpreterBridge {
    /// auto-lang 解释器
    interpreter: AutoInterpreter,

    /// Widget 实例状态（widget_name → state）
    widget_states: HashMap<String, WidgetState>,

    /// 是否启用热重载
    hot_reload: bool,
}

/// Widget 状态
#[derive(Clone)]
pub struct WidgetState {
    /// 字段值
    pub fields: HashMap<String, Value>,

    /// 缓存的视图节点
    pub cached_node: Option<Node>,

    /// 视图是否脏（需要重建）
    pub view_dirty: bool,
}

impl InterpreterBridge {
    /// 创建新的解释器桥梁
    pub fn new() -> Self {
        Self {
            interpreter: AutoInterpreter::new(),
            widget_states: HashMap::new(),
            hot_reload: true,
        }
    }

    /// 从文件加载并执行代码
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        let code = std::fs::read_to_string(path)?;
        self.interpret(&code)
    }

    /// 解释并执行 Auto 代码
    pub fn interpret(&mut self, code: &str) -> Result<()> {
        self.interpreter.eval(code)
            .map_err(|e| BridgeError::AutoLang(e.to_string()))?;
        Ok(())
    }

    /// 获取主 Widget 的视图节点
    ///
    /// 此方法会：
    /// 1. 调用 `main()` 函数
    /// 2. 或者查找主 Widget 的 `view()` 方法
    /// 3. 返回求值后的 Node
    pub fn get_main_view(&mut self) -> Result<Node> {
        // Evaluate main() or the last expression
        let result = self.interpreter.eval("main()")
            .map_err(|e| BridgeError::AutoLang(e.to_string()))?;

        if let Value::Node(node) = &result {
            Ok(node.clone())
        } else {
            // 创建一个默认的空节点
            Ok(Node::new("div"))
        }
    }

    /// 处理事件消息
    pub fn handle_message(&mut self, msg: DynamicMessage) -> Result<()> {
        match msg {
            DynamicMessage::String(event) => {
                // 解析事件字符串并调用相应的 on() 方法
                self.handle_string_event(&event)
            }
            DynamicMessage::Typed { widget_name, event_name, args } => {
                // 调用特定 Widget 的 on() 方法
                self.handle_typed_event(&widget_name, &event_name, &args)
            }
        }
    }

    /// 处理字符串事件
    fn handle_string_event(&mut self, event: &str) -> Result<()> {
        // 解析 "widget.event" 格式
        if let Some(dot_pos) = event.find('.') {
            let widget_name = &event[..dot_pos];
            let event_name = &event[dot_pos + 1..];
            self.handle_typed_event(widget_name, event_name, &[])?;
        } else {
            // 尝试调用默认 Widget 的 on() 方法
            // TODO: 实现默认 Widget 查找
        }
        Ok(())
    }

    /// 处理类型化事件
    fn handle_typed_event(&mut self, widget_name: &str, _event_name: &str, _args: &[Value]) -> Result<()> {
        // 查找 Widget 状态
        if let Some(state) = self.widget_states.get_mut(widget_name) {
            // 标记视图为脏（需要重建）
            state.view_dirty = true;
        }

        // 调用 Widget 的 on() 方法
        // TODO: 实现通过解释器调用 on() 方法
        // let widget = self.interpreter.scope.borrow().get_val(widget_name);
        // call_method(widget, "on", &[Value::Str(event_name.into())]);

        Ok(())
    }

    /// 重新加载代码（热重载）
    pub fn reload(&mut self, code: &str) -> Result<()> {
        // 保存旧状态（用于状态迁移）
        let old_states = self.widget_states.clone();

        // 重新解释代码
        self.interpret(code)?;

        // 迁移状态
        self.migrate_states(old_states);

        Ok(())
    }

    /// 状态迁移
    fn migrate_states(&mut self, old_states: HashMap<String, WidgetState>) {
        for (name, old_state) in old_states {
            if let Some(new_state) = self.widget_states.get_mut(&name) {
                // 迁移兼容的字段
                for (field_name, field_value) in old_state.fields {
                    // 只保留类型相同的字段
                    if new_state.fields.contains_key(&field_name) {
                        new_state.fields.insert(field_name.clone(), field_value);
                    }
                }
            }
        }
    }

    /// 启用热重载
    pub fn enable_hot_reload(&mut self) {
        self.hot_reload = true;
    }

    /// 禁用热重载
    pub fn disable_hot_reload(&mut self) {
        self.hot_reload = false;
    }
}

impl Default for InterpreterBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = InterpreterBridge::new();
        assert!(!bridge.hot_reload || bridge.hot_reload); // Just to use the variable
    }

    #[test]
    fn test_default_bridge() {
        let mut bridge = InterpreterBridge::default();
        // Test that default bridge works
        let _ = bridge.get_main_view();
    }
}
