//! Lexer 兼容出口（PLAN-789）：`Lexer`/`LexerState` 的单一实现已迁至
//! auto-lex（`crates/auto-lex/src/lexer.rs`，逐字搬运；`AutoResult` 签名
//! 换为词法层 `LexResult`，错误映射见 `crate::error::From<LexError>`）。
//! 本模块保持 crate 私有（`mod lexer`，抽取前同为私有）：Lexer 不是
//! auto-lang 公开面，消费方（parser/bridge/loader）经 `crate::lexer::X`
//! 引用不变；外部消费者请直接依赖 auto-lex。

pub use auto_lex::lexer::{Lexer, LexerState};
