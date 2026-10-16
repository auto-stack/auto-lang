//! PLAN-799：AST 模型单一实现已迁至 auto-ast（`auto_ast::ast`），本模块是
//! 其**薄门面**——re-export 同一类型与 traits，`auto_lang::ast::*` /
//! `crate::ast::*` 旧路径全部保持可用（类型身份跨 crate 互传，不保留第二
//! 份实现，无转换副本）。解析便利层（`ParserExt` 对 Code/Name/Expr/Is/
//! Arrow/OnEvents 的六类 impl）随 Parser 留在上层 `crate::ast_parse_ext`
//!（旧 `ast::parsers` 为私有模块，无外部路径消费）。

pub use auto_ast::ast::*;
