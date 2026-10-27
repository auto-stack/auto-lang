//! PLAN-799：AST 模型单一实现已迁至 auto-ast（`auto_ast::ast`），本模块是
//! 其**薄门面**——re-export 同一类型与 traits，SDK `crate::ast::*`
//! 与 `auto_ast::ast::*` 类型身份相同，不保留第二份实现或转换副本。
//! auto-lang 0.2 已退出旧公开 `auto_lang::ast::*` 模块。解析便利层（`ParserExt` 对 Code/Name/Expr/Is/
//! Arrow/OnEvents 的六类 impl）随 Parser 留在上层 `crate::ast_parse_ext`
//!（旧 `ast::parsers` 为私有模块，无外部路径消费）。

pub use auto_ast::ast::*;
