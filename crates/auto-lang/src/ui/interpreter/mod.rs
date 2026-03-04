//! # Auto 动态解释器
//!
//! 此模块提供了 Auto 语言的运行时动态解释器，支持真正的热重载开发体验。
//!
//! ## 核心组件
//!
//! - [`SymbolTable`] - 存储组件、类型和函数的元数据
//! - [`WidgetMetadata`] - Widget 组件的元数据
//! - [`ComponentInstance`] - 运行时组件实例
//! - [`InterpreterRuntime`] - 解释器运行时状态
//! - [`DynamicMessage`] - 增强的动态消息（保留类型信息）
//!
//! ## 使用示例
//!
//! ```rust
//! use auto_ui::interpreter::*;
//!
//! // 从 .at 文件加载并启动热重载
//! let interpreter = HotReloadInterpreter::load_and_watch("path/to/component.at")?;
//!
//! // 获取当前视图
//! let view = interpreter.view()?;
//! ```
//!
//! ## 架构
//!
//! ```
//! ┌─────────────────────────────────────────────┐
//! │           GPUI Application                  │
//! └──────────────────┬──────────────────────────┘
//!                    │
//! ┌──────────────────▼──────────────────────────┐
//! │     DynamicComponent (GPUI Render)          │
//! └──────────────────┬──────────────────────────┘
//!                    │
//! ┌──────────────────▼──────────────────────────┐
//! │        InterpreterRuntime                   │
//! │  ┌─────────────────────────────────────┐   │
//! │  │  SymbolTable (类型元数据)           │   │
//! │  │  ComponentInstance (状态、视图)     │   │
//! │  │  EventRouter (消息路由)             │   │
//! │  └─────────────────────────────────────┘   │
//! └──────────────────┬──────────────────────────┘
//!                    │
//! ┌──────────────────▼──────────────────────────┐
//! │         AutoParser (auto-lang)              │
//! └──────────────────┬──────────────────────────┘
//!                    │
//! ┌──────────────────▼──────────────────────────┐
//! │      Enhanced NodeConverter                 │
//! └─────────────────────────────────────────────┘
//! ```

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
