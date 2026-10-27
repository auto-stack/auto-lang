//! PLAN-801 V03（AC-02）：新接口方言的生产派发——新旧混排、同段顺序、
//! with_session 保留。None/Err 恢复与递归传播腿由 v01_v05/r1（model 实现
//! 已走新接口）与 r1 内联旧 trait 方言共同覆盖，本文件补混合与顺序差异面。
//!
//! oracle 区分度：顺序/命中由 attempts/accepts 计数与产出节点共同证明，
//! 不只断言无 panic。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use auto_ast::ast::{Expr, Stmt};
use auto_diagnostics::error::AutoResult;
use auto_lang_core::parser::Parser;
use auto_lang_core::session::CompilerSession;
use auto_parser_api::{ExternalDialect, ParseContext};
use auto_ui_dialect::{UiCode, UiStmt};
use dialect_api_consumer::{ExternalUiDialect, Probe};
use ui_dialect_consumer::{LegacyProbe, LegacyProbeDialect};

/// UI 领域组合与非 UI 旧 trait 分别经各自公开入口真实接管。
#[test]
fn v03_mixed_new_and_old_dialects_dispatch_independently() {
    let model_probe = Arc::new(Probe::default());
    let legacy_probe = Arc::new(LegacyProbe::default());
    let model_ast = Parser::from("model { count int = 1 }")
        .with_session(CompilerSession::ui())
        .parse_domain(&ExternalUiDialect::new(model_probe.clone()))
        .map(UiCode)
        .expect("domain model parse");
    let legacy_ast = Parser::from("probe 7")
        .with_session(CompilerSession::ui())
        .with_dialect(
            Box::new(LegacyProbeDialect::new(legacy_probe.clone())),
            auto_lang::dialect::DialectPriority::Append,
        )
        .parse()
        .expect("legacy Core parse");
    assert_eq!(model_probe.accepts.load(Ordering::SeqCst), 1);
    assert_eq!(legacy_probe.accepts.load(Ordering::SeqCst), 1);
    assert!(matches!(
        model_ast.domain_stmts().next(),
        Some(UiStmt::ModelBlock(_))
    ));
    assert!(matches!(&legacy_ast.stmts[0], Stmt::Expr(Expr::Int(7))));
}

/// 同段（Override）内按注册顺序咨询：先注册者先被调用，其 Ok(None) 让行后
/// 后注册者接管——新接口方言间同样遵循 778 冻结的同段顺序语义。
struct YieldingDialect {
    attempts: Arc<AtomicUsize>,
}

impl ExternalDialect for YieldingDialect {
    fn matches(&self, _s: &CompilerSession) -> bool {
        true
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["pick"]
    }
    fn try_parse_stmt(
        &self,
        _ctx: &mut dyn ParseContext,
        _keyword: &str,
    ) -> AutoResult<Option<Stmt>> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        Ok(None)
    }
}

#[test]
fn v03_same_section_registration_order_is_consulted_in_order() {
    let first_attempts = Arc::new(AtomicUsize::new(0));
    let ast = Parser::from("pick")
        .with_session(CompilerSession::ui())
        .with_external_dialect(
            Box::new(YieldingDialect {
                attempts: first_attempts.clone(),
            }),
            auto_parser_api::DialectPriority::Override,
        )
        .with_external_dialect(
            Box::new(RecordingPickerDialect::default()),
            auto_parser_api::DialectPriority::Override,
        )
        .parse()
        .expect("让行后后注册方言接管 pick");
    assert_eq!(first_attempts.load(Ordering::SeqCst), 1, "先注册者先被咨询");
    assert!(
        matches!(&ast.stmts[0], Stmt::Expr(Expr::Int(11))),
        "后注册者接管 pick"
    );
}

struct RecordingPickerDialect {
    accepts: AtomicUsize,
}

impl Default for RecordingPickerDialect {
    fn default() -> Self {
        Self {
            accepts: AtomicUsize::new(0),
        }
    }
}

impl ExternalDialect for RecordingPickerDialect {
    fn matches(&self, _s: &CompilerSession) -> bool {
        true
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["pick"]
    }
    fn try_parse_stmt(
        &self,
        ctx: &mut dyn ParseContext,
        _keyword: &str,
    ) -> AutoResult<Option<Stmt>> {
        self.accepts.fetch_add(1, Ordering::SeqCst);
        ctx.expect_keyword("pick")?;
        Ok(Some(Stmt::Expr(Expr::Int(11))))
    }
}

/// 显式领域组合在 session 设置后仍由同一实例执行。
#[test]
fn v03_new_registration_survives_with_session() {
    let probe = Arc::new(Probe::default());
    let ast = Parser::from("model { count int = 1 }")
        .with_session(CompilerSession::ui())
        .parse_domain(&ExternalUiDialect::new(probe.clone()))
        .map(UiCode)
        .unwrap();
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
    assert!(matches!(
        ast.domain_stmts().next(),
        Some(UiStmt::ModelBlock(_))
    ));
}

/// Core 场景门控在新路径派发时生效：matches() 拒绝即零咨询。
#[test]
fn v03_core_scenario_gates_new_dialect_at_dispatch() {
    let probe = Arc::new(Probe::default());
    let ast = Parser::from("model = 1")
        .parse_domain(&ExternalUiDialect::new(probe.clone()))
        .expect("Core 下 model = 1 必须按普通赋值解析");
    assert_eq!(probe.attempts.load(Ordering::SeqCst), 0);
    assert_eq!(ast.items.len(), 1);
}
