//! Compiler Session - Scenario-based compilation context
//!
//! This module implements the "Scenario Programming" architecture where
//! the compiler's behavior is driven by the current scenario (Core, UI, Shell).
//!
//! PLAN-801：轻 session 单一定义已迁至 auto-parser-api
//!（`auto_parser_api::session`，含其原单元测试），本模块是其**薄门面**——
//! re-export 同一类型，`auto_lang::session::*` / `crate::session::*` 旧路径
//! 全部保持可用（类型身份跨 crate 互传，不保留第二份实现）。执行会话
//!（`compile::CompileSession` 等重上下文）不随迁，仍在 auto-lang。
//!
//! ## Usage
//!
//! ```rust
//! use auto_lang_core::session::{CompilerSession, Scenario};
//!
//! // Create a UI scenario session
//! let session = CompilerSession::new(Scenario::UI);
//!
//! // Create with backend
//! let session = CompilerSession::new(Scenario::UI).with_backend("react");
//! ```

pub use auto_parser_api::session::{CompilerSession, Scenario};
