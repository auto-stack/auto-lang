//! PLAN-801 V01 轻面测试（无解析）：本 crate 测试不以 auto-lang 作 dev
//! 依赖——场景门控、关键字面与 trait 可实现性在 crate 边界外即可验证。
//! 解析集成测试（真 Parser 派发/AST 投影/恢复）在上层 driver
//! test/fixtures/modularity/ui-dialect-consumer（见其 tests/）。

use std::sync::Arc;

use auto_parser_api::session::{CompilerSession, Scenario};
use auto_parser_api::ExternalDialect;
use dialect_api_consumer::{ExternalModelDialect, Probe};

#[test]
fn matches_gates_on_ui_scenario() {
    let d = ExternalModelDialect::new(Arc::new(Probe::default()));
    assert!(d.matches(&CompilerSession::new(Scenario::UI)));
    assert!(!d.matches(&CompilerSession::new(Scenario::Core)));
    assert!(!d.matches(&CompilerSession::new(Scenario::Shell)));
}

#[test]
fn keywords_are_the_frozen_model_subset() {
    let d = ExternalModelDialect::new(Arc::new(Probe::default()));
    assert_eq!(d.keywords(), &["model"]);
}

/// 场景构造面（V05 素材）：轻 session 在无 auto-lang 闭包内可构造并
/// 字段可读——与 auto_lang::session 同一类型身份由 driver 侧编译断言。
#[test]
fn light_session_constructible_without_lang() {
    let s = CompilerSession::ui().with_backend("react").with_debug(true);
    assert!(s.is_ui());
    assert_eq!(s.scenario, Scenario::UI);
    assert_eq!(s.backend.as_deref(), Some("react"));
    assert!(s.debug);
    assert_eq!(s.backend_or_default(), "react");
    assert_eq!(CompilerSession::default().backend_or_default(), "a2r");
}
