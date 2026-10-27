//! PLAN-799 跨 SDK 与模型路径的 AST 类型同一性（Cargo 解析的永久 consumer）。
//!
//! review-r1/parse-identity.rs 的探针固化：SDK 路径（`auto_lang_core::ast`）
//! 与模型路径（`auto_ast::ast`）的 AST 是**同一 Rust 类型**（re-export，
//! 非包装/转换副本），可双向互传；六类解析便利入口的坏输入确定性拒绝。
//! 由 Cargo 统一解析依赖实例（SDK 与模型依赖指向同一 auto-ast），
//! 不手选缓存 rlib。

use auto_ast::ast as model;
use auto_lang_core::ast::{Arrow, Code, Expr, Is, Name, OnEvents, ToAtom};
use auto_lang_core::parser::ParserExt;

fn old_to_new(x: Code) -> model::Code {
    x
}

fn new_to_old(x: model::Code) -> Code {
    x
}

#[test]
fn cross_path_type_identity_and_parse_rejection() {
    assert!(Code::parse("let =").is_err());
    assert!(Expr::parse("(").is_err());
    assert!(Is::parse("is").is_err());
    assert!(Arrow::parse("->").is_err());
    assert!(OnEvents::parse("on {").is_err());
    assert_eq!(Name::parse("x").unwrap().as_str(), "x");
    let x = new_to_old(old_to_new(Code::parse("let x = 1").unwrap()));
    assert_eq!(x.stmts.len(), 1);
}

#[test]
fn cross_path_trait_projection_same_output() {
    // 同一 AST 值经模型 trait 方法（auto_ast::ToAtom）与 SDK 路径投影
    // 输出一致——不靠包装/复制模型。
    let code = Code::parse("let x = 1").unwrap();
    let m: &model::Code = &code;
    assert_eq!(
        auto_lang_core::ast::ToAtom::to_atom(m).as_str(),
        code.to_atom().as_str()
    );
}
