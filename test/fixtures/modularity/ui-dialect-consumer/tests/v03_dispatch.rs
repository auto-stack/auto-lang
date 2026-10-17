//! PLAN-801 V03（AC-02）：新接口方言的生产派发——新旧混排、同段顺序、
//! with_session 保留。None/Err 恢复与递归传播腿由 v01_v05/r1（model 实现
//! 已走新接口）与 r1 内联旧 trait 方言共同覆盖，本文件补混合与顺序差异面。
//!
//! oracle 区分度：顺序/命中由 attempts/accepts 计数与产出节点共同证明，
//! 不只断言无 panic。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use auto_lang::ast::{Expr, Stmt};
use auto_lang::error::AutoResult;
use auto_lang::parser::Parser;
use auto_lang::session::CompilerSession;
use auto_parser_api::{ExternalDialect, ParseContext};
use dialect_api_consumer::{ExternalModelDialect, Probe};
use ui_dialect_consumer::{LegacyProbe, LegacyProbeDialect};

/// 新旧混排：Override 段新接口 model 方言 + Append 段旧 trait probe 方言，
/// 与内建 UiDialect（widget/model）同表共存——各自关键字各自接管，互不串扰。
#[test]
fn v03_mixed_new_and_old_dialects_dispatch_independently() {
    let model_probe = Arc::new(Probe::default());
    let legacy_probe = Arc::new(LegacyProbe::default());
    let code = "model { count int = 1 }\nprobe 7";
    let ast = Parser::from(code)
        .with_session(CompilerSession::ui())
        .with_external_dialect(
            Box::new(ExternalModelDialect::new(model_probe.clone())),
            auto_parser_api::DialectPriority::Override,
        )
        .with_dialect(
            Box::new(LegacyProbeDialect::new(legacy_probe.clone())),
            auto_lang::dialect::DialectPriority::Append,
        )
        .parse()
        .expect("新旧方言同表混排必须照常解析");
    assert_eq!(
        model_probe.accepts.load(Ordering::SeqCst),
        1,
        "model 由新接口方言接管"
    );
    assert_eq!(
        legacy_probe.accepts.load(Ordering::SeqCst),
        1,
        "probe 由旧 trait 方言接管"
    );
    assert!(matches!(&ast.stmts[0], Stmt::ModelBlock(_)));
    assert!(
        matches!(&ast.stmts[1], Stmt::Expr(Expr::Int(7))),
        "probe 产出表达式语句"
    );
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

/// with_session 只重建内建段：新接口注册跨 with_session 保留（V-03 传播
/// 不变量在新路径成立；先注册、后设 session 的顺序不得清空外部注册）。
#[test]
fn v03_new_registration_survives_with_session() {
    let probe = Arc::new(Probe::default());
    let ast = Parser::from("model { count int = 1 }")
        .with_external_dialect(
            Box::new(ExternalModelDialect::new(probe.clone())),
            auto_parser_api::DialectPriority::Override,
        )
        .with_session(CompilerSession::ui())
        .parse()
        .unwrap();
    assert_eq!(
        probe.accepts.load(Ordering::SeqCst),
        1,
        "with_session 不得丢弃已注册新接口方言"
    );
    assert!(matches!(&ast.stmts[0], Stmt::ModelBlock(_)));
}

/// Core 场景门控在新路径派发时生效：matches() 拒绝即零咨询（与 v02_core
/// 同 oracle，换注册入口复验——matches 契约不因接口迁移漂移）。
#[test]
fn v03_core_scenario_gates_new_dialect_at_dispatch() {
    let probe = Arc::new(Probe::default());
    let ast = Parser::from("model = 1")
        .with_external_dialect(
            Box::new(ExternalModelDialect::new(probe.clone())),
            auto_parser_api::DialectPriority::Override,
        )
        .parse()
        .expect("Core 下 model = 1 必须按普通赋值解析");
    assert_eq!(
        probe.attempts.load(Ordering::SeqCst),
        0,
        "Core 场景不得咨询外部方言"
    );
    assert!(!matches!(&ast.stmts[0], Stmt::ModelBlock(_)));
}
