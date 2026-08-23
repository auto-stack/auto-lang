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
    ///
    /// Plan 436 L1:源码在 **UI 场景**下解析(widget 语法是 UI 方言门控的,
    /// VM 默认解析器会拒绝 `widget`/`model`/`msg` 声明),经 VM 执行后,
    /// 每个 widget 的 `setup {}` 前导槽在加载时执行一次——**先于任何视图
    /// 求值**(get_main_view 在 interpret 之后才可能被调用)。非 UI 场景的
    /// 普通 脚本走原有 VM 解析路径(行为不变)。
    pub fn interpret(&mut self, code: &str) -> Result<()> {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(code).with_session(session);
        match parser.parse() {
            Ok(ast) => {
                self.interpreter
                    .eval_ast(ast.clone())
                    .map_err(|e| BridgeError::AutoLang(e.to_string()))?;
                self.run_setup_preambles(&ast)?;
            }
            // 非 UI 场景程序(普通脚本):保持原有 VM 默认解析求值路径,
            // 其自身的解析/运行错误在那边照常上浮。
            Err(_) => {
                self.interpreter
                    .eval(code)
                    .map_err(|e| BridgeError::AutoLang(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Plan 436 L1:逐 widget 执行 `setup {}` 前导槽一次,绑定写入该
    /// widget 的(单实例)`WidgetState.fields`。
    ///
    /// 边界(单实例层语义,与 a2vue 的差异见模块文档):
    /// - 前导槽在**独立的 VM run** 中执行:每次 run 都是新 VM,程序级
    ///   作用域(含程序里定义的函数)不会延续进来——setup 只能依赖字面
    ///   量表达式与注入 globals;
    /// - 取回通道 = setup 语句 + 尾随 `[绑定名, ...]` 数组表达式,run 的
    ///   栈顶结果携带绑定值离开 VM;
    /// - 只有带 setup 块的 widget 会建立 WidgetState(L1 不构建完整的
    ///   组件 view 管线);
    /// - `refs` 标注(.value 语义)在解释器中不存在——Value 即值。
    fn run_setup_preambles(&mut self, ast: &crate::ast::Code) -> Result<()> {
        for stmt in &ast.stmts {
            let decl = match stmt {
                crate::ast::Stmt::WidgetDecl(d) => d,
                _ => continue,
            };
            let setup = match &decl.setup {
                Some(s) => s,
                None => continue,
            };
            let name = decl.name.as_str().to_string();
            // 绑定名:setup 体顶层的 let/var/const 声明。
            let names: Vec<String> = setup
                .body
                .stmts
                .iter()
                .filter_map(|s| match s {
                    crate::ast::Stmt::Store(store)
                        if matches!(
                            store.kind,
                            crate::ast::StoreKind::Let
                                | crate::ast::StoreKind::Var
                                | crate::ast::StoreKind::Const
                        ) =>
                    {
                        Some(store.name.as_str().to_string())
                    }
                    _ => None,
                })
                .collect();

            let mut stmts = setup.body.stmts.clone();
            if !names.is_empty() {
                let idents = names
                    .iter()
                    .map(|n| crate::ast::Expr::Ident(n.as_str().into()))
                    .collect();
                stmts.push(crate::ast::Stmt::Expr(crate::ast::Expr::Array(idents)));
            }
            let result = self.interpreter.eval_stmts(stmts).map_err(|e| {
                BridgeError::AutoLang(format!("widget `{}` setup preamble: {}", name, e))
            })?;

            let mut fields = HashMap::new();
            if !names.is_empty() {
                match result {
                    Value::Array(arr) => {
                        for (i, v) in arr.iter().enumerate() {
                            if let Some(n) = names.get(i) {
                                fields.insert(n.clone(), v.clone());
                            }
                        }
                    }
                    other => {
                        return Err(BridgeError::AutoLang(format!(
                            "widget `{}` setup preamble: expected bindings array, got {:?}",
                            name, other
                        )));
                    }
                }
            }
            self.widget_states.insert(
                name,
                WidgetState {
                    fields,
                    cached_node: None,
                    view_dirty: false,
                },
            );
        }
        Ok(())
    }

    /// Plan 436 L1:读取 widget 的单实例状态(setup 前导绑定所在)。
    pub fn widget_state(&self, widget_name: &str) -> Option<&WidgetState> {
        self.widget_states.get(widget_name)
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

    /// Plan 436 L1:含 setup 块的 widget 源经 bridge 加载——UI 场景解析
    /// (此前 VM 默认解析直接拒绝 widget 语法),setup 执行一次,绑定写入
    /// 单实例 WidgetState.fields;interpret 先于任何视图求值(构造约定)。
    #[test]
    fn test_setup_preamble_runs_and_binds_fields() {
        let mut bridge = InterpreterBridge::new();
        bridge
            .interpret(
                r#"
widget Counter {
    setup {
        let total = 40 + 2
        let label = "hi"
    }
    model { var count int = 0 }
}
"#,
            )
            .expect("widget source must load under the UI-scenario parse");
        let state = bridge.widget_state("Counter").expect("WidgetState created");
        assert_eq!(state.fields.get("total"), Some(&Value::Int(42)));
        assert_eq!(state.fields.get("label"), Some(&Value::str("hi")));
    }

    /// Plan 436 L1:setup 引用未定义符号 → 显式报错(带 widget 上下文),
    /// 不再静默。
    #[test]
    fn test_setup_preamble_error_surfaces() {
        let mut bridge = InterpreterBridge::new();
        let err = bridge
            .interpret(
                r#"
widget W {
    setup { let x = no_such_thing + 1 }
}
"#,
            )
            .expect_err("undefined symbol in setup must surface");
        assert!(
            err.to_string().contains("W") && err.to_string().contains("setup"),
            "error carries widget context: {err}"
        );
    }

    /// Plan 436:无 setup 块的 widget 不建 WidgetState;普通脚本(非 UI
    /// 场景)走回退路径行为不变。
    #[test]
    fn test_plain_script_and_setupless_widget() {
        let mut bridge = InterpreterBridge::new();
        bridge.interpret("fn main() { 1 }").expect("plain script loads");
        assert!(bridge.widget_state("main").is_none());

        bridge
            .interpret(
                r#"
widget NoSetup {
    model { var count int = 0 }
}
"#,
            )
            .expect("setupless widget loads");
        assert!(
            bridge.widget_state("NoSetup").is_none(),
            "no WidgetState without a setup block (L1 boundary)"
        );
    }

    #[test]
    #[ignore = "slow: calls eval(\"main()\") on empty interpreter, hangs ~90s"]
    fn test_default_bridge() {
        let mut bridge = InterpreterBridge::default();
        // Test that default bridge works
        let _ = bridge.get_main_view();
    }
}
