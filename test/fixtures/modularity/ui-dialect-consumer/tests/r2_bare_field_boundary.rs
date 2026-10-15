//! PLAN-778 R2 边界控制（F778-R2-01 闭合后的永久回归）。
//!
//! 冻结文法 `model { [var] name [type] [= expr] }` 的两个可选项可**同时省略**
//! （裸字段）。原 oracle 只覆盖 typed/带初值形态、漏掉双省组合（覆盖诊断见
//! 计划 §9 r3）——本文件补全可选组合四象限 + var×裸字段 + 非法形态负控 +
//! 内建拒绝控制（内建路径行为保持，不随外部子集扩大）。
//! 来源：复审 boundary.rs 反例收编 + 组合矩阵补全。

use std::sync::atomic::Ordering;
use std::sync::Arc;

use auto_lang::ast::{Stmt, Type};
use auto_lang::dialect::DialectPriority;
use auto_lang::parser::Parser;
use auto_lang::session::CompilerSession;
use ui_dialect_consumer::{ExternalModelDialect, Probe};

fn ui_override(code: &str, probe: Arc<Probe>) -> Parser<'_> {
    Parser::from(code)
        .with_session(CompilerSession::ui())
        .with_dialect(
            Box::new(ExternalModelDialect::new(probe)),
            DialectPriority::Override,
        )
}

/// 象限 1/4：类型+初值齐备（既有 v01 已覆盖，此处作矩阵锚点）。
#[test]
fn r2_both_type_and_init() {
    let probe = Arc::new(Probe::default());
    let ast = ui_override("model { count int = 1 }", probe.clone())
        .parse()
        .unwrap();
    let Stmt::ModelBlock(block) = &ast.stmts[0] else {
        panic!("expected ModelBlock");
    };
    assert!(matches!(block.fields[0].ty, Type::Int));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Int(1)));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

/// 象限 2/4：仅类型（init 省略 → Nil）。R2 boundary 控制收编。
#[test]
fn r2_type_only_is_supported() {
    let probe = Arc::new(Probe::default());
    let ast = ui_override("model { count int }", probe.clone())
        .parse()
        .unwrap();
    let Stmt::ModelBlock(block) = &ast.stmts[0] else {
        panic!("typed block");
    };
    assert!(matches!(block.fields[0].ty, Type::Int));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Nil));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

/// 象限 3/4：仅初值（type 省略 → Unknown）。既有 v01 untyped 形态。
#[test]
fn r2_init_only_is_supported() {
    let probe = Arc::new(Probe::default());
    let ast = ui_override("model { count = 1 }", probe.clone())
        .parse()
        .unwrap();
    let Stmt::ModelBlock(block) = &ast.stmts[0] else {
        panic!("expected ModelBlock");
    };
    assert!(matches!(block.fields[0].ty, Type::Unknown));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Int(1)));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

/// 象限 4/4：裸字段（类型/初值同时省略 → Unknown + Nil）——F778-R2-01 反例
/// 闭合。原实现此处报 `Expected type, got }`。
#[test]
fn r2_bare_field_is_supported() {
    let probe = Arc::new(Probe::default());
    let ast = ui_override("model { count }", probe.clone())
        .parse()
        .unwrap();
    let Stmt::ModelBlock(block) = &ast.stmts[0] else {
        panic!("expected ModelBlock with bare field");
    };
    assert_eq!(&*block.fields[0].name, "count");
    assert!(matches!(block.fields[0].ty, Type::Unknown));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Nil));
    assert!(!block.fields[0].mutable);
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

/// 裸字段 + 后续带类型字段（R2 反例：原实现在换行处报
/// `Expected type, got \n`，级联前 parse_stmt 直查）。
#[test]
fn r2_bare_field_then_typed_field_multiline() {
    let probe = Arc::new(Probe::default());
    let mut parser = ui_override("model { count\nother int = 2 }", probe.clone());
    let result = parser.parse_stmt();
    assert!(
        result.is_ok(),
        "冻结文法 name [type] [= expr] 包含换行前的裸字段: {result:?}"
    );
    let Stmt::ModelBlock(block) = result.unwrap() else {
        panic!("expected ModelBlock");
    };
    assert_eq!(block.fields.len(), 2);
    assert!(matches!(block.fields[0].ty, Type::Unknown));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Nil));
    assert!(matches!(block.fields[1].ty, Type::Int));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

/// var × 裸字段组合：可变裸字段（mutable=true + Unknown + Nil）。
#[test]
fn r2_var_bare_field() {
    let probe = Arc::new(Probe::default());
    let ast = ui_override("model { var flag }", probe.clone())
        .parse()
        .unwrap();
    let Stmt::ModelBlock(block) = &ast.stmts[0] else {
        panic!("expected ModelBlock");
    };
    assert!(block.fields[0].mutable);
    assert!(matches!(block.fields[0].ty, Type::Unknown));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Nil));
}

/// 负控：字段名后跟既非类型起始、非 `=`、非字段/块终止符的 token（数字）
/// → 核心类型文法报合法错误（裸字段支持不吞非法形态）。
#[test]
fn r2_malformed_after_name_still_errors() {
    let probe = Arc::new(Probe::default());
    let mut parser = ui_override("model { count 123 }", probe.clone());
    assert!(
        parser.parse_stmt().is_err(),
        "数字既非类型也非赋值/终止符，必须报错"
    );
    assert_eq!(
        probe.accepts.load(Ordering::SeqCst),
        0,
        "报错路径不得计为接管"
    );
}

/// 内建拒绝控制（R2 boundary 收编）：内建 parse_model_block_inner 无裸字段
/// 支持——默认内建路径行为保持不变（F778-R2-01 明确不扩大默认文法）；
/// 外部子集的裸字段支持是合同面差异，非默认文法漂移。
#[test]
fn r2_builtin_bare_field_still_rejected_control() {
    assert!(
        Parser::from("model { count }")
            .with_session(CompilerSession::ui())
            .parse_stmt()
            .is_err(),
        "内建路径不得随外部子集扩大裸字段支持"
    );
}
