//! Error types and diagnostics for AutoLang
//!
//! This module provides comprehensive error reporting with source locations,
//! error codes, and helpful suggestions using the `miette` diagnostic library.
//!
//! PLAN-793：诊断单一实现已迁至 auto-diagnostics（`auto_diagnostics::error`），
//! 本模块是其**薄门面**——re-export 同一类型与函数，`auto_lang::error::*`
//! 旧路径全部保持可用（类型身份跨 crate 互传，不保留第二份实现）。
//! `error → ast::Name` 依赖边随迁出切断：TypeError::CannotModifyViewParam
//! 载荷在底层直接使用 `auto_val::AutoStr`（= `ast::Name` 的同一底层类型）。
//!
//! 模块专有错误转换留在所有者侧（孤儿规则合法的本地类型 From impl）：
//! `route/discovery.rs` 的 `From<RouteDiscoveryError> for AutoError`、
//! `infer/unification.rs` 的 `From<UnificationError> for TypeError`。

// 冻结公开面（T-01 消费闭包收据 attachments/793/t01-consumer-closure.md §2）：
// crate 外 2 消费者（auto/main.rs、auto-lsp/diagnostics.rs）+ crate 内
// ~15 文件全部经下方 re-export，零改动。
pub use auto_diagnostics::error::{
    attach_source, find_best_match, format_error, pos_to_span, print_error, span_from, AutoError,
    AutoResult, ComptimeError, LexerError, LexerErrorWithSource, MietteError, MsgWithSource,
    NameError, Result, RuntimeError, SyntaxError, SyntaxErrorWithSource, TypeError, Warning,
};
