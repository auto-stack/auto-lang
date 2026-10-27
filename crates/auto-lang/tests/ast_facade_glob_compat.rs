//! PLAN-799 F-799-R1-01 永久回归：SDK glob 兼容（编译期断言）。
//!
//! 反例（review-r1/glob-compat.rs）：模型 crate 曾把 `auto_val::AutoResult`
// 历史上经 `auto_lang::ast` 单行 glob 门面泄漏；迁移后的合法组合
//! `use auto_lang_core::ast::*; use auto_lang_core::error::*;` 中 `AutoResult` 产生
//! E0659 歧义（两个**不同 item**：auto_val 的 Box<dyn Error> 版 vs
//! error/auto_diagnostics 的富 AutoError 版）。
//!
//! 不变量：`auto_lang_core::ast::*` 不得新增与其他常用 glob 面（error/token/
//! lexer/parser/vm）**不同 item** 同名的公开导出。本文件编译即断言——
//! 歧义或类型漂移都在构建期红；`token::*` 组合同时钉住 Pos 的同一 item
//! 性（同 item 经两个 glob 进入不歧义）。

use auto_lang_core::ast::*;
use auto_lang_core::error::*;
use auto_lang_core::parser::ParserExt;
use auto_lang::token::Pos as TokenPos;

// AutoResult 必须仍唯一解析到 error::AutoResult（Result<T, AutoError>）——
// 若 ast::* 再次引入 auto_val 版，此处 E0659；若解析到错误类型，E0308。
fn parse_via_compat_globs() -> AutoResult<Code> {
    Code::parse("let x = 1")
}

// Pos 同一 item：ast 路径（auto-ast 同源 re-export）与 token 路径（auto-lex
// 同源 re-export）互传无需任何转换——不同类型在此签名下必然编译红。
fn pos_same_item_through_two_paths(p: Pos) -> TokenPos {
    p
}

#[test]
fn facade_glob_compat_error_autoresult() {
    let parsed = parse_via_compat_globs();
    assert!(parsed.is_ok());
    // Pos 双 glob 同一 item（auto_lex::token::Pos）可互传。
    let p = Pos {
        line: 1,
        at: 0,
        pos: 0,
        len: 0,
    };
    let _ = pos_same_item_through_two_paths(p);
}
