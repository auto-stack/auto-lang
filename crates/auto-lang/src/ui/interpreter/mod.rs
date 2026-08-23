//! # Auto 动态解释器(UI 桥梁)
//!
//! 此模块连接 `crate::interpreter::AutoInterpreter`(AutoVM 之上的求值
//! 接口)与 UI 渲染系统。实际架构(Plan 436 修正——旧头注释引用的
//! `SymbolTable`(widget 元数据版)/`WidgetMetadata`/`ComponentInstance`/
//! `InterpreterRuntime`/`HotReloadInterpreter` 并不存在,属文档腐烂):
//!
//! ```text
//! .at 源码(UI 场景解析:widget/model/msg 为 UI 方言门控)
//!    ↓
//! InterpreterBridge::interpret —— 整程序经 AutoInterpreter(AutoVM)求值
//!    ↓                        └─ Plan 436 L1:逐 widget 执行 setup {} 前导槽
//! WidgetState { fields }       (单实例;绑定经尾随数组表达式带出 VM run)
//!    ↓
//! get_main_view —— eval("main()") → auto_val::Node
//!    ↓
//! node_converter / 渲染层
//! ```
//!
//! ## 边界(Plan 436 T0 调研定稿)
//!
//! - **单实例**:`widget_states` 按类型名键控,每个 widget 一个状态——
//!   无子组件实例化机制(L2 真每实例不可及,边界见 docs/syntax.md
//!   三相位矩阵);
//! - **生命周期**:`.Init`/`.Destroy` 事件路由未实现(bridge 的
//!   `handle_typed_event` 仅置脏标记);setup 是唯一落地的相位,加载时
//!   执行一次、先于任何视图求值;
//! - **setup 前导槽**:在独立 VM run 中执行(每次 run 均为新 VM,程序级
//!   函数/globals 不延续),绑定名取自 setup 体顶层 let/var/const;
//!   `refs` 标注无 `.value` 语义(Value 即值)。

mod bridge;

pub use bridge::*;

/// 动态解释器错误类型
pub type Result<T> = std::result::Result<T, InterpreterError>;

/// 动态解释器错误
#[derive(Debug, thiserror::Error)]
pub enum InterpreterError {
    /// 解析错误
    #[error("Parse error: {0}")]
    Parse(String),

    /// 组件未找到
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// 字段未找到
    #[error("Field not found: {0}")]
    FieldNotFound(String),

    /// 类型不匹配
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    /// 锁错误
    #[error("Lock error: {0}")]
    LockError(String),

    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 其他错误
    #[error("Unknown error: {0}")]
    Unknown(String),
}
