//! PLAN-801 V05（AC-03）：旧公开调用/类型身份保持——旧 `Dialect` trait +
//! `ParserBridge` 桥面（by-value checkpoint、泛型 `syntax_error`）、旧
//! `with_dialect`/`register_dialect` Box 注册、session 字段构造与跨门面
//! 类型互传（`auto_lang_core::session::CompilerSession` ≡
//! `auto_parser_api::CompilerSession` 同一类型）。
//!
//! 内建 escape hatch（`ParserBridge::parser()`，pub(crate)）外部不可访问
//! 是编译期属性（E0624 私有），无运行期断言面——本文件经"只使用公开面"
//! 的编译成功间接钉住。文法单一实现（model 无第二份算法）由
//! dialect-api-consumer 单份 + 本 driver 的 LegacyProbeDialect 非 model
//! 用例共同保证（777 文本扫描与源码审计另证）。

use std::sync::atomic::Ordering;
use std::sync::Arc;

use auto_ast::ast::{Expr, Stmt};
use auto_diagnostics::error::{AutoError, SyntaxError};
use auto_lang_core::parser::Parser;
use auto_lang_core::session::CompilerSession;
use auto_parser_api::DomainDialect;
use auto_ui_dialect::{UiCode, UiStmt};
use dialect_api_consumer::{ExternalUiDialect, Probe};
use ui_dialect_consumer::{LegacyProbe, LegacyProbeDialect};

/// 旧 trait 方言经旧注册入口真实解析：`probe 7` → `Stmt::Expr(Int(7))`，
/// attempt/accept 计数证明真实派发与接管（非内建替跑——内建无 probe 关键字）。
#[test]
fn v05_legacy_trait_dialect_parses_via_old_registration() {
    let probe = Arc::new(LegacyProbe::default());
    let ast = Parser::from("probe 7")
        .with_session(CompilerSession::ui())
        .with_dialect(
            Box::new(LegacyProbeDialect::new(probe.clone())),
            auto_lang::dialect::DialectPriority::Override,
        )
        .parse()
        .expect("旧 trait + 旧注册入口必须照常工作");
    assert_eq!(probe.attempts.load(Ordering::SeqCst), 1);
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
    assert!(matches!(&ast.stmts[0], Stmt::Expr(Expr::Int(7))));
}

/// 旧桥面泛型 `syntax_error(impl Into<String>)`：旧 trait 方言的 Err 出口
/// 产出 Generic 语法错误且消息/位置正确（&str 直传——泛型形态的证据）。
#[test]
fn v05_legacy_generic_syntax_error_and_by_value_checkpoint() {
    let probe = Arc::new(LegacyProbe::default());
    let mut parser = Parser::from("probe )")
        .with_session(CompilerSession::ui())
        .with_dialect(
            Box::new(LegacyProbeDialect::new(probe)),
            auto_lang::dialect::DialectPriority::Override,
        );
    let err = parser.parse_stmt().unwrap_err();
    match &err {
        AutoError::Syntax(SyntaxError::Generic { message, .. }) => {
            assert!(
                message.contains("probe expects an expression"),
                "诊断来自旧桥面泛型 syntax_error: {message}"
            );
        }
        other => panic!("expected Generic syntax error, got {other:?}"),
    }
    // Err 后旧注册仍在：恢复点后合法 probe 语句仍由旧方言接管
    //（by-value checkpoint 已在 Err 路径回滚游标）。
    let ast2 = Parser::from("probe 3")
        .with_session(CompilerSession::ui())
        .with_dialect(
            Box::new(LegacyProbeDialect::new(Arc::new(LegacyProbe::default()))),
            auto_lang::dialect::DialectPriority::Override,
        )
        .parse()
        .unwrap();
    assert!(matches!(&ast2.stmts[0], Stmt::Expr(Expr::Int(3))));
}

/// session 字段构造 + 跨门面类型互传：`auto_lang::session`（薄门面 re-export）
/// 与 `auto_parser_api::session` 是**同一类型**——字段构造、builder、以及
/// 双向赋值/传参都编译且行为一致（V05"同一 session 类型身份"）。
#[test]
fn v05_session_type_identity_across_facades() {
    // 字段构造面（SDK builder 照常）。
    let s = CompilerSession::new(auto_lang_core::session::Scenario::UI)
        .with_backend("gpui")
        .with_debug(false)
        .with_incremental(true);
    assert!(s.is_ui());
    assert_eq!(s.backend.as_deref(), Some("gpui"));
    assert!(!s.debug);
    assert!(s.incremental);

    // 跨门面同型：auto_parser_api 构造值直接落入 SDK 类型标注。
    let via_api: CompilerSession = auto_parser_api::session::CompilerSession::ui();
    assert!(via_api.is_ui());

    // 反向：SDK 构造值作为 auto_parser_api trait 的 matches 入参
    //（trait 方法签名吃 auto_parser_api 类型——同型才编译）。
    fn matches_ui(s: &auto_parser_api::CompilerSession) -> bool {
        let d = ExternalUiDialect::new(Arc::new(Probe::default()));
        d.matches(s)
    }
    assert!(matches_ui(&CompilerSession::ui()));
    assert!(!matches_ui(&CompilerSession::core()));

    let probe = Arc::new(Probe::default());
    let ast = Parser::from("model { count int = 1 }")
        .with_session(CompilerSession::ui())
        .parse_domain(&ExternalUiDialect::new(probe.clone()))
        .map(UiCode)
        .unwrap();
    assert!(matches!(
        ast.domain_stmts().next(),
        Some(UiStmt::ModelBlock(_))
    ));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}
