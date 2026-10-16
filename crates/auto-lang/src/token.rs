//! Token 兼容出口（PLAN-789）：`Token`/`TokenKind`/`Pos` 的单一实现已迁至
//! auto-lex（`crates/auto-lex/src/token.rs`，逐字搬运含 `keyword_kind`/
//! `all_keywords`/Display/内联测试），此处仅 re-export 同一类型——
//! `crate::token::X` 与 `auto_lang::token::X` 旧路径全部保持可用，
//! 类型身份与 auto-lex 互传，不保留第二份实现。

pub use auto_lex::token::{Pos, Token, TokenKind};
